use anyhow::{bail, ensure, Context, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use syn::{Item, ReturnType, Type, UseTree};
use tiny_reexport::{facts::Facts, model};
#[path = "pretrained/scan.rs"]
mod pretrained;

struct Candidate {
    module: String,
    export: String,
    local: Facts,
    address: usize,
}

#[derive(Clone)]
struct Scope {
    imports: Vec<Vec<String>>,
    function: String,
    returns: Vec<String>,
}

fn serialize_use_tree_tokens(tree: &UseTree, out: &mut Vec<String>) {
    match tree {
        UseTree::Path(p) => {
            out.extend([p.ident.to_string(), "::".into()]);
            serialize_use_tree_tokens(&p.tree, out);
        }
        UseTree::Name(n) => out.push(n.ident.to_string()),
        UseTree::Rename(r) => out.extend([r.ident.to_string(), "as".into(), r.rename.to_string()]),
        UseTree::Glob(_) => out.push("*".into()),
        UseTree::Group(g) => {
            out.push("{".into());
            for (i, t) in g.items.iter().enumerate() {
                if i > 0 {
                    out.push(",".into());
                }
                serialize_use_tree_tokens(t, out);
            }
            out.push("}".into());
        }
    }
}

fn inspect_binding_source_scope(items: &[Item]) -> Result<Scope> {
    let mut scope = Scope {
        imports: vec![],
        returns: vec![],
        function: String::new(),
    };
    for item in items {
        match item {
            Item::Use(u) => {
                ensure!(
                    u.attrs.is_empty() && u.leading_colon.is_none(),
                    "unsupported import"
                );
                let mut tokens = vec![];
                serialize_use_tree_tokens(&u.tree, &mut tokens);
                scope.imports.push(tokens);
            }
            Item::Fn(f) => {
                ensure!(
                    scope.returns.is_empty()
                        && f.attrs.is_empty()
                        && matches!(f.vis, syn::Visibility::Public(_))
                        && f.sig.generics.params.is_empty()
                        && f.sig.generics.where_clause.is_none()
                        && f.sig.asyncness.is_none()
                        && f.sig.inputs.is_empty(),
                    "expected one public non-generic zero-argument query"
                );
                let ReturnType::Type(_, ty) = &f.sig.output else {
                    bail!("missing return")
                };
                let Type::Path(p) = ty.as_ref() else {
                    bail!("expected type path")
                };
                ensure!(
                    p.qself.is_none()
                        && p.path.leading_colon.is_none()
                        && p.path.segments.iter().all(|s| s.arguments.is_empty()),
                    "unsupported qualified or generic path"
                );
                scope.returns = p
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                scope.function = f.sig.ident.to_string();
            }
            Item::Mod(_) => (),
            _ => bail!("unsupported consumer item"),
        }
    }
    Ok(scope)
}

struct Example {
    candidates: Vec<Candidate>,
    query: Scope,
    parent_imports: Vec<Vec<String>>,
}

impl Example {
    fn parse_binding_source_file(source: &str) -> Result<Self> {
        let file = syn::parse_file(source)?;
        ensure!(file.attrs.is_empty(), "conditional crates not supported");
        let mut query = inspect_binding_source_scope(&file.items)?;
        let mut parent_imports = vec![];
        let mut candidates = vec![];
        for item in &file.items {
            let Item::Mod(m) = item else { continue };
            ensure!(
                m.attrs.is_empty() && matches!(m.vis, syn::Visibility::Public(_)),
                "expected an unconditional public inline module"
            );
            let (_, items) = m.content.as_ref().context("expected inline module")?;
            if items.iter().any(|i| matches!(i, Item::Fn(_))) {
                ensure!(
                    query.returns.is_empty() && !items.iter().any(|i| matches!(i, Item::Mod(_))),
                    "unsupported query scope"
                );
                parent_imports = query.imports;
                query = inspect_binding_source_scope(items)?;
                query.function = format!("{}::{}", m.ident, query.function);
                continue;
            }
            let mut structs = vec![];
            let mut aliases = vec![];
            for i in items {
                match i {
                    Item::Struct(s) => structs.push(s.ident.to_string()),
                    Item::Use(u) => {
                        let mut tokens = vec![];
                        serialize_use_tree_tokens(&u.tree, &mut tokens);
                        aliases.push(tokens.last().context("empty local alias")?.clone());
                    }
                    _ => bail!("unsupported candidate member"),
                }
            }
            ensure!(
                structs.len() == 1 && aliases.len() <= 1,
                "expected one struct and optional re-export"
            );
            let local = Facts::parse_source_into_facts(&quote::quote!(#(#items)*).to_string())?;
            let export = aliases.pop().unwrap_or_else(|| structs.remove(0));
            let address = local
                .names
                .iter()
                .position(|n| n == &export)
                .context("missing export")?;
            candidates.push(Candidate {
                module: m.ident.to_string(),
                export,
                local,
                address,
            });
        }
        ensure!(
            candidates.len() == 2
                && candidates[0].export == candidates[1].export
                && candidates[0].module != candidates[1].module
                && !query.returns.is_empty(),
            "expected two colliding names and a return path"
        );
        Ok(Self {
            candidates,
            query,
            parent_imports,
        })
    }

    // Describe both candidates identically. No resolver, labels, module spellings,
    // body tokens, or source positions enter the learned feature vector.
    fn extract_candidate_context_features(
        &self,
        candidate: &Candidate,
    ) -> (BTreeSet<String>, Vec<String>) {
        let mut out = BTreeSet::new();
        let mut words = Vec::new();
        let ret = &self.query.returns;
        let normalize = |s: &str| -> &str {
            if s == candidate.module {
                "M"
            } else if self.candidates.iter().any(|c| c.module == s) {
                "O"
            } else if ret.last().is_some_and(|v| v == s) {
                "Q"
            } else if ret.len() > 1 && ret[..ret.len() - 1].iter().any(|v| v == s && v != "self") {
                "P"
            } else if s == candidate.export {
                "T"
            } else {
                match s {
                    "as" => "as",
                    "::" => "::",
                    "{" => "{",
                    "}" => "}",
                    "," => ",",
                    "*" => "*",
                    "self" => "self",
                    "super" => "super",
                    _ => "id",
                }
            }
        };
        let mut add = |kind: &str, tokens: &[String]| {
            let mut norm = vec!["^"];
            norm.extend(tokens.iter().map(|t| normalize(t)));
            norm.push("$");
            words.push(kind.to_owned());
            words.extend(norm[1..norm.len() - 1].iter().map(|s| (*s).to_owned()));
            for n in 1..=6 {
                for window in norm.windows(n) {
                    out.insert(format!("{kind}:{}", window.join(" ")));
                }
            }
        };
        add("return", ret);
        for u in &self.query.imports {
            add("use", u);
        }
        for u in &self.parent_imports {
            add("outer", u);
        }
        (out, words)
    }

    // Deliberately small, non-learned control: qualified returns and explicit
    // direct imports/renames only. No glob, group or multi-hop resolution.
    fn match_simple_explicit_binding(&self) -> Option<usize> {
        let ret: Vec<_> = self
            .query
            .returns
            .iter()
            .map(String::as_str)
            .filter(|x| *x != "self")
            .collect();
        let mut hits = BTreeSet::new();
        for (i, c) in self.candidates.iter().enumerate() {
            if ret == [c.module.as_str(), c.export.as_str()] {
                hits.insert(i);
            }
            for u in &self.query.imports {
                let tokens: Vec<_> = u
                    .iter()
                    .map(String::as_str)
                    .filter(|s| !matches!(*s, "self" | "super" | "::"))
                    .collect();
                let found = match tokens.as_slice() {
                    [m, t] => *m == c.module && *t == c.export && ret == [*t],
                    [m, t, "as", a] => *m == c.module && *t == c.export && ret == [*a],
                    [m, "as", a] => *m == c.module && ret == [*a, c.export.as_str()],
                    _ => false,
                };
                if found {
                    hits.insert(i);
                }
            }
        }
        if hits.len() == 1 {
            hits.into_iter().next()
        } else {
            None
        }
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mode = args
        .next()
        .context("features FILE... | predict WEIGHTS FILE...")?;
    if mode == "real" {
        return pretrained::run_real_crate_scan(args.collect());
    }
    ensure!(
        matches!(mode.as_str(), "features" | "predict"),
        "unknown mode"
    );
    let weights: BTreeMap<String, f64> = if mode == "predict" {
        serde_json::from_slice(&std::fs::read(args.next().context("missing weights")?)?)?
    } else {
        BTreeMap::new()
    };
    let mut session = (mode == "predict")
        .then(model::load_cpu_model_session)
        .transpose()?;
    let mut count = 0;
    for path in args {
        let input: Box<dyn Read> = if path == "-" {
            Box::new(std::io::stdin().lock())
        } else {
            Box::new(std::fs::File::open(&path)?)
        };
        let mut text = String::new();
        input.take(65537).read_to_string(&mut text)?;
        ensure!(text.len() <= 65536, "source exceeds 64 KiB");
        let ex = Example::parse_binding_source_file(&text).with_context(|| path.clone())?;
        let (feats, tokens): (Vec<_>, Vec<_>) = ex
            .candidates
            .iter()
            .map(|c| ex.extract_candidate_context_features(c))
            .unzip();
        let names: Vec<_> = ex.candidates.iter().map(|c| &c.module).collect();
        let result: Value = if let Some(s) = &mut session {
            let scores: Vec<f64> = feats
                .iter()
                .map(|f| f.iter().filter_map(|key| weights.get(key)).sum())
                .collect();
            let margin = scores[0] - scores[1];
            let confidence = 1. / (1. + (-margin.abs()).exp());
            let c = &ex.candidates[usize::from(margin < 0.)];
            let target = if confidence >= 0.80 {
                model::resolve_model_return_target(s, &c.local, c.address)?
                    .map(|i| format!("{}::{}", c.module, c.local.names[i]))
            } else {
                None
            };
            json!({"from":ex.query.function,"role":"return","spelling":ex.query.returns.join("::"),
                "status":if target.is_some() {"model_proposed"} else {"unresolved"},
                "target":target,"score":confidence})
        } else {
            json!({"candidates":names,"features":feats,"tokens":tokens,"simple":ex.match_simple_explicit_binding()})
        };
        println!("{}", serde_json::to_string(&result)?);
        count += 1;
    }
    ensure!(count > 0, "provide at least one file");
    Ok(())
}
