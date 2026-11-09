use std::fs;
use syn::spanned::Spanned;
fn vis(v: &syn::Visibility) -> &'static str { match v { syn::Visibility::Public(_) => "pub", syn::Visibility::Restricted(_) => "pub(x)", syn::Visibility::Inherited => "priv" } }
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let src = fs::read_to_string(&path).unwrap();
    let file = match syn::parse_file(&src) { Ok(f) => f, Err(e) => { println!("PARSE_ERROR {}", e); return; } };
    for item in &file.items {
        let (kind, name, v) = match item {
            syn::Item::Fn(i) => ("fn", i.sig.ident.to_string(), vis(&i.vis)),
            syn::Item::Struct(i) => ("struct", i.ident.to_string(), vis(&i.vis)),
            syn::Item::Enum(i) => ("enum", i.ident.to_string(), vis(&i.vis)),
            syn::Item::Trait(i) => ("trait", i.ident.to_string(), vis(&i.vis)),
            syn::Item::Impl(i) => ("impl", i.self_ty.span().start().line.to_string(), "n/a"),
            syn::Item::Mod(i) => ("mod", i.ident.to_string(), vis(&i.vis)),
            syn::Item::Use(i) => ("use", String::new(), vis(&i.vis)),
            syn::Item::Const(i) => ("const", i.ident.to_string(), vis(&i.vis)),
            syn::Item::Static(i) => ("static", i.ident.to_string(), vis(&i.vis)),
            syn::Item::Type(i) => ("type", i.ident.to_string(), vis(&i.vis)),
            syn::Item::Macro(i) => ("macro", i.ident.as_ref().map(|x| x.to_string()).unwrap_or_default(), "n/a"),
            syn::Item::Union(i) => ("union", i.ident.to_string(), vis(&i.vis)),
            syn::Item::ExternCrate(i) => ("extern_crate", i.ident.to_string(), vis(&i.vis)),
            syn::Item::ForeignMod(_) => ("extern_block", String::new(), "n/a"),
            syn::Item::TraitAlias(i) => ("trait_alias", i.ident.to_string(), vis(&i.vis)),
            _ => ("other", String::new(), "n/a"),
        };
        let s = item.span(); let a = s.start(); let b = s.end();
        println!("{}\t{}\t{}\t{}:{}\t{}:{}", kind, name, v, a.line, a.column, b.line, b.column);
    }
}
