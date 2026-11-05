use anyhow::{bail, ensure, Result};
use syn::{ext::IdentExt, Fields, Ident, Item, ReturnType, Type, UseTree, Visibility};

pub const SLOTS: usize = 8;

#[derive(Default)]
pub struct Facts {
    pub names: Vec<String>,
    pub defs: [u16; SLOTS],
    pub aliases: [[u16; SLOTS]; SLOTS],
    pub queries: Vec<(String, String, usize)>,
}

impl Facts {
    fn intern_source_type_name(&mut self, ident: &Ident) -> Result<usize> {
        let name = ident.unraw().to_string();
        if let Some(i) = self.names.iter().position(|n| *n == name) {
            return Ok(i);
        }
        ensure!(
            self.names.len() < SLOTS,
            "capacity: at most {SLOTS} distinct type names"
        );
        self.names.push(name);
        Ok(self.names.len() - 1)
    }

    pub fn parse_source_into_facts(source: &str) -> Result<Self> {
        let file = syn::parse_file(source)?;
        ensure!(file.attrs.is_empty(), "crate attributes not supported");
        let mut facts = Self::default();
        for item in file.items {
            match item {
                Item::Struct(s) if s.attrs.is_empty() && matches!(s.vis, Visibility::Public(_))
                    && matches!(s.fields, Fields::Unit) && s.generics.params.is_empty()
                    && s.generics.where_clause.is_none() => {
                    let i = facts.intern_source_type_name(&s.ident)?;
                    facts.defs[i] += 1;
                }
                Item::Use(u) if u.attrs.is_empty() && matches!(u.vis, Visibility::Public(_))
                    && u.leading_colon.is_none() => {
                    let tree = match &u.tree {
                        UseTree::Path(p) if p.ident == "self" => p.tree.as_ref(),
                        other => other,
                    };
                    let UseTree::Rename(r) = tree else { bail!("expected pub use [self::]Name as Alias") };
                    let a = facts.intern_source_type_name(&r.rename)?;
                    let t = facts.intern_source_type_name(&r.ident)?;
                    facts.aliases[a][t] += 1;
                }
                Item::Fn(f) if f.attrs.is_empty() && matches!(f.vis, Visibility::Public(_))
                    && f.sig.inputs.is_empty() && f.sig.generics.params.is_empty()
                    && f.sig.generics.where_clause.is_none() && f.sig.asyncness.is_none() => {
                    let ReturnType::Type(_, ty) = &f.sig.output else { bail!("expected a named return type") };
                    let Type::Path(p) = ty.as_ref() else { bail!("expected an unqualified type name") };
                    let name = p.path.get_ident().filter(|_| p.qself.is_none())
                        .ok_or_else(|| anyhow::anyhow!("expected an unqualified type name"))?;
                    let i = facts.intern_source_type_name(name)?;
                    let function = f.sig.ident.unraw().to_string();
                    ensure!(!facts.queries.iter().any(|(f, _, _)| f == &function), "duplicate function name");
                    facts.queries.push((function, name.to_string(), i));
                }
                _ => bail!("unsupported item: expected an unconditional public unit struct, re-export or function"),
            }
        }
        Ok(facts)
    }

    pub fn resolve_native_return_target(&self, mut at: usize) -> Option<usize> {
        let mut seen = [false; SLOTS];
        while !seen[at] {
            seen[at] = true;
            let row = &self.aliases[at];
            if self.defs[at] + row.iter().sum::<u16>() != 1 {
                return None;
            }
            if self.defs[at] == 1 {
                return Some(at);
            }
            at = row.iter().position(|&v| v == 1)?;
        }
        None
    }
}
