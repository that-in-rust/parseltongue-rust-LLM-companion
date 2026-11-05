use super::{serialize_use_tree_tokens, Candidate, Example, Scope};
use anyhow::{bail, ensure, Context, Result};
use ort::{session::Session, value::Tensor};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    rc::Rc,
};
use syn::{visit::Visit, Item, ReturnType, Signature, Visibility};
use tiny_reexport::facts::Facts;

type Pos = (String, usize, usize);
type Imports = Rc<Vec<Vec<String>>>;
type Def = (Vec<String>, Pos);
struct Site {
    module: Vec<String>,
    pos: Pos,
    fun: String,
    path: Vec<String>,
    imports: Imports,
}
#[derive(Default)]
struct Index {
    defs: BTreeMap<String, Vec<Def>>,
    sites: Vec<Site>,
    files: usize,
    failures: Vec<String>,
}

fn collect_rust_source_paths(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect_rust_source_paths(&entry.path(), files)?;
        } else if entry.file_type()?.is_file()
            && entry.path().extension().is_some_and(|e| e == "rs")
        {
            files.push(entry.path());
        }
    }
    Ok(())
}

struct File<'a> {
    rel: String,
    text: &'a str,
    lines: Vec<usize>,
}
impl File<'_> {
    fn locate_source_identifier_span(&self, ident: &syn::Ident) -> Pos {
        let off = ident.span().byte_range().start;
        let line = self.lines.partition_point(|&i| i <= off) - 1;
        (
            self.rel.clone(),
            line,
            self.text[self.lines[line]..off].encode_utf16().count(),
        )
    }
}
struct Paths(Vec<syn::Path>);
impl<'a> Visit<'a> for Paths {
    fn visit_type_path(&mut self, p: &'a syn::TypePath) {
        if p.qself.is_none() {
            self.0.push(p.path.clone());
        }
        syn::visit::visit_type_path(self, p);
    }
}

impl Index {
    fn record_signature_type_sites(
        &mut self,
        f: &File,
        module: &[String],
        sig: &Signature,
        imports: &Imports,
    ) {
        let ReturnType::Type(_, ty) = &sig.output else {
            return;
        };
        let mut paths = Paths(vec![]);
        paths.visit_type(ty);
        for path in paths.0 {
            let Some(last) = path.segments.last() else {
                continue;
            };
            self.sites.push(Site {
                module: module.to_owned(),
                pos: f.locate_source_identifier_span(&last.ident),
                fun: sig.ident.to_string(),
                path: path.segments.iter().map(|s| s.ident.to_string()).collect(),
                imports: Rc::clone(imports),
            });
        }
    }

    fn index_module_source_items(&mut self, f: &File, module: &[String], items: &[Item]) {
        let imports = Rc::new(
            items
                .iter()
                .filter_map(|i| {
                    if let Item::Use(u) = i {
                        let mut tokens = vec![];
                        serialize_use_tree_tokens(&u.tree, &mut tokens);
                        Some(tokens)
                    } else {
                        None
                    }
                })
                .collect(),
        );
        for item in items {
            let name = match item {
                Item::Struct(v) => Some(&v.ident),
                Item::Enum(v) => Some(&v.ident),
                Item::Type(v) => Some(&v.ident),
                Item::Union(v) => Some(&v.ident),
                Item::Trait(v) => Some(&v.ident),
                _ => None,
            };
            if let Some(n) = name {
                self.defs
                    .entry(n.to_string())
                    .or_default()
                    .push((module.to_vec(), f.locate_source_identifier_span(n)));
            }
            match item {
                Item::Mod(m) => {
                    if let Some((_, children)) = &m.content {
                        let mut sub = module.to_vec();
                        sub.push(m.ident.to_string());
                        self.index_module_source_items(f, &sub, children);
                    }
                }
                Item::Fn(v) if matches!(v.vis, Visibility::Public(_)) => {
                    self.record_signature_type_sites(f, module, &v.sig, &imports)
                }
                Item::Impl(v) => {
                    for i in &v.items {
                        if let syn::ImplItem::Fn(m) = i {
                            if matches!(m.vis, Visibility::Public(_)) {
                                self.record_signature_type_sites(f, module, &m.sig, &imports);
                            }
                        }
                    }
                }
                Item::Trait(v) if matches!(v.vis, Visibility::Public(_)) => {
                    for i in &v.items {
                        if let syn::TraitItem::Fn(m) = i {
                            self.record_signature_type_sites(f, module, &m.sig, &imports);
                        }
                    }
                }
                _ => (),
            }
        }
    }

    fn scan_real_crate_files(crate_root: &Path) -> Result<Self> {
        let src = crate_root.join("src");
        let mut files = vec![];
        collect_rust_source_paths(&src, &mut files)?;
        files.sort();
        let mut idx = Self::default();
        for path in files {
            idx.files += 1;
            let rel = path
                .strip_prefix(crate_root)?
                .to_string_lossy()
                .into_owned();
            let text = std::fs::read_to_string(&path)?;
            let ast = match syn::parse_file(&text) {
                Ok(v) => v,
                Err(e) => {
                    idx.failures.push(format!("{rel}: {e}"));
                    continue;
                }
            };
            let mut module: Vec<_> = path
                .strip_prefix(&src)?
                .with_extension("")
                .iter()
                .map(|x| x.to_string_lossy().into_owned())
                .collect();
            if module
                .last()
                .is_some_and(|x| matches!(x.as_str(), "lib" | "main" | "mod"))
            {
                module.pop();
            }
            let lines = std::iter::once(0)
                .chain(text.match_indices('\n').map(|(i, _)| i + 1))
                .collect();
            idx.index_module_source_items(
                &File {
                    rel,
                    text: &text,
                    lines,
                },
                &module,
                &ast.items,
            );
        }
        Ok(idx)
    }
}

fn distinguish_candidate_module_names(a: &[String], b: &[String]) -> Option<[String; 2]> {
    for i in 0..a.len().max(b.len()) {
        let x = a.iter().rev().nth(i).map_or("crate", String::as_str);
        let y = b.iter().rev().nth(i).map_or("crate", String::as_str);
        if x != y {
            return Some([x.into(), y.into()]);
        }
    }
    None
}

fn read_scalar_model_output(out: &ort::session::SessionOutputs, name: &str) -> Result<f32> {
    let (_, v) = out[name].try_extract_tensor::<f32>()?;
    ensure!(v.len() == 2, "expected two candidate scores");
    Ok(v[0] - v[1])
}

pub(super) fn run_real_crate_scan(args: Vec<String>) -> Result<()> {
    ensure!(args.len() >= 3, "real MODEL.onnx META.json CRATE...");
    ort::init_from(std::env::var_os("ORT_DYLIB_PATH").context("set ORT_DYLIB_PATH")?)?.commit();
    let mut s = Session::builder()?
        .with_intra_threads(1)
        .map_err(ort::Error::<()>::from)?
        .with_inter_threads(1)
        .map_err(ort::Error::<()>::from)?
        .commit_from_file(&args[0])?;
    let meta: Value = serde_json::from_slice(&std::fs::read(&args[1])?)?;
    let features = meta["features"]
        .as_array()
        .context("missing feature vocabulary")?;
    let ids = &meta["tokens"];
    for root in &args[2..] {
        let root = Path::new(root);
        let release = root
            .file_name()
            .context("missing crate name")?
            .to_string_lossy();
        let idx = Index::scan_real_crate_files(root)?;
        let mut eligible = 0;
        for site in &idx.sites {
            let Some(leaf) = site.path.last() else {
                continue;
            };
            let Some(c) = idx.defs.get(leaf) else {
                continue;
            };
            if c.len() != 2 {
                continue;
            }
            let Some(names) = distinguish_candidate_module_names(&c[0].0, &c[1].0) else {
                continue;
            };
            let ex = Example {
                candidates: names
                    .iter()
                    .map(|m| Candidate {
                        module: m.clone(),
                        export: leaf.clone(),
                        local: Facts::default(),
                        address: 0,
                    })
                    .collect(),
                query: Scope {
                    function: site.fun.clone(),
                    returns: site.path.clone(),
                    imports: (*site.imports).clone(),
                },
                parent_imports: vec![],
            };
            let rule = ex.match_simple_explicit_binding();
            let local: Vec<_> = c
                .iter()
                .enumerate()
                .filter(|(_, d)| d.0 == site.module)
                .map(|(i, _)| i)
                .collect();
            let local_rule =
                rule.or_else(|| (site.path.len() == 1 && local.len() == 1).then(|| local[0]));
            let mut tokens = vec![];
            let mut mask = vec![];
            let mut bits = vec![];
            for candidate in &ex.candidates {
                let (feats, words) = ex.extract_candidate_context_features(candidate);
                bits.extend(features.iter().map(|k| {
                    if feats.contains(k.as_str().unwrap_or("")) {
                        1f32
                    } else {
                        0.
                    }
                }));
                let mut seq = vec![101i64];
                for w in words.iter().take(62) {
                    seq.push(
                        ids[w]
                            .as_i64()
                            .with_context(|| format!("unknown prompt symbol {w}"))?,
                    );
                }
                seq.push(102);
                mask.extend((0..64).map(|n| i64::from(n < seq.len())));
                seq.resize(64, 0);
                tokens.extend(seq);
            }
            let out=s.run(ort::inputs!["input_ids"=>Tensor::from_array(([2,64],tokens))?,
                "attention_mask"=>Tensor::from_array(([2,64],mask))?, "token_type_ids"=>Tensor::from_array(([2,64],vec![0i64;128]))?,
                "lexical_features"=>Tensor::from_array(([2,features.len()],bits))?])?;
            let lex = read_scalar_model_output(&out, "lexical_score")?;
            let enc = read_scalar_model_output(&out, "encoder_score")?;
            let fused = read_scalar_model_output(&out, "binding_score")?;
            if !fused.is_finite() || (fused - lex - enc).abs() > 0.0001 {
                bail!("invalid fused logits")
            }
            let pick = |v: f32| (v.abs() >= 4f32.ln()).then_some(usize::from(v < 0.));
            println!(
                "{}",
                json!({"crate":release,"site":site.pos,"function":site.fun,"path":site.path,
                "candidates":[c[0].1,c[1].1],"modules":[c[0].0,c[1].0],"rule":rule,"local_rule":local_rule,
                "fused":pick(fused),"lexical":pick(lex),"encoder":pick(enc),"margins":[lex,enc,fused]})
            );
            eligible += 1;
        }
        eprintln!(
            "{}",
            json!({"crate":release,"files":idx.files,"parse_failures":idx.failures,
            "return_type_mentions":idx.sites.len(),"eligible":eligible})
        );
    }
    Ok(())
}
