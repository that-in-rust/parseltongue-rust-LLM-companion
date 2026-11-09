use ra_ap_syntax::{ast, ast::HasModuleItem, ast::HasName, AstNode, Edition, SourceFile};
fn lc(src: &str, off: usize) -> String { let l = src[..off].matches('\n').count() + 1; let c = off - src[..off].rfind('\n').map(|p| p + 1).unwrap_or(0); format!("{l}:{c}") }
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let src = std::fs::read_to_string(&path).unwrap();
    let parse = SourceFile::parse(&src, Edition::Edition2021);
    let errs = parse.errors().len();
    let err_ranges: Vec<_> = parse.errors().iter().map(|e| e.range()).collect();
    let file = parse.tree();
    for item in file.items() {
        let r = item.syntax().text_range();
        let (s, e) = (usize::from(r.start()), usize::from(r.end()));
        let (kind, name) = match &item {
            ast::Item::Fn(x) => ("fn", x.name().map(|n| n.to_string())),
            ast::Item::Struct(x) => ("struct", x.name().map(|n| n.to_string())),
            ast::Item::Enum(x) => ("enum", x.name().map(|n| n.to_string())),
            ast::Item::Union(x) => ("union", x.name().map(|n| n.to_string())),
            ast::Item::Trait(x) => ("trait", x.name().map(|n| n.to_string())),
            ast::Item::Impl(_) => ("impl", None),
            ast::Item::Module(x) => ("mod", x.name().map(|n| n.to_string())),
            ast::Item::Use(_) => ("use", None),
            ast::Item::Const(x) => ("const", x.name().map(|n| n.to_string())),
            ast::Item::Static(x) => ("static", x.name().map(|n| n.to_string())),
            ast::Item::TypeAlias(x) => ("type", x.name().map(|n| n.to_string())),
            ast::Item::MacroRules(x) => ("macro", x.name().map(|n| n.to_string())),
            ast::Item::MacroDef(x) => ("macro", x.name().map(|n| n.to_string())),
            ast::Item::MacroCall(_) => ("macro", None),
            ast::Item::ExternBlock(_) => ("extern_block", None),
            ast::Item::ExternCrate(x) => ("extern_crate", x.name_ref().map(|n| n.to_string())),
            _ => ("other", None),
        };
        let vis = match item.syntax().children().find_map(ast::Visibility::cast) { None => "priv".to_string(), Some(v) => if v.to_string() == "pub" { "pub".into() } else { "pub(x)".into() } };
        // a span containing an ERROR node is grey
        let grey = item.syntax().descendants().any(|n| n.kind() == ra_ap_syntax::SyntaxKind::ERROR)
            || err_ranges.iter().any(|er| er.start() <= r.end() && r.start() <= er.end());
        println!("{kind}\t{}\t{vis}\t{}\t{}\t{}", name.unwrap_or_default(), lc(&src, s), lc(&src, e), if grey { 0.3 } else { 1.0 });
    }
    // errors outside any item (top-level ERROR nodes) → one grey line each
    for n in file.syntax().children().filter(|n| n.kind() == ra_ap_syntax::SyntaxKind::ERROR) {
        let r = n.text_range(); println!("error\t\tn/a\t{}\t{}\t0", lc(&src, r.start().into()), lc(&src, r.end().into()));
    }
    eprintln!("parse_errors={errs}");
}
