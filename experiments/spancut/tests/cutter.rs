use spancut::*;
fn names(spans: &[Span]) -> Vec<(Kind, String)> {
    spans
        .iter()
        .map(|s| (s.kind.clone(), s.name.clone()))
        .collect()
}
#[test]
fn test_cut_two_items_one_line() {
    let s = cut_item_spans("pub struct A; pub struct B;");
    assert_eq!(
        names(&s),
        vec![(Kind::Struct, "A".into()), (Kind::Struct, "B".into())]
    );
    assert_eq!((s[0].start, s[0].end), (0, 13));
    assert_eq!((s[1].start, s[1].end), (14, 27));
}
#[test]
fn test_cut_brace_inside_string() {
    let s = cut_item_spans("pub fn f() { let x = \"}\"; } pub fn g() {}");
    assert_eq!(
        names(&s),
        vec![(Kind::Fn, "f".into()), (Kind::Fn, "g".into())]
    );
}
#[test]
fn test_attach_attrs_docs_above() {
    let src = "/// doc\n#[cfg(test)]\npub fn f() {}";
    let s = cut_item_spans(src);
    assert_eq!(s[0].start, 0);
    assert_eq!(s[0].end, src.len());
}
#[test]
fn test_resync_after_missing_brace() {
    let s = cut_item_spans("pub fn f() { let a = 1;\npub fn g() {}\npub struct S;");
    assert_eq!(
        names(&s),
        vec![
            (Kind::Fn, "f".into()),
            (Kind::Fn, "g".into()),
            (Kind::Struct, "S".into())
        ]
    );
    assert!(s[0].confidence < 0.5, "unterminated item must be grey");
    assert!(s[1].confidence > 0.9);
}
#[test]
fn test_grey_half_typed_tail() {
    let s = cut_item_spans("pub struct S;\npub fn send(");
    assert_eq!(s.len(), 2);
    assert!(s[1].confidence < 0.5);
}
#[test]
fn test_macro_rules_braces_not_block() {
    let s = cut_item_spans("macro_rules! m { ($x:expr) => { { $x } } }\npub fn f() {}");
    assert_eq!(
        names(&s),
        vec![(Kind::Macro, "m".into()), (Kind::Fn, "f".into())]
    );
}
#[test]
fn test_nested_items_not_module_level() {
    let s = cut_item_spans("pub fn f() { fn inner() {} struct X; }");
    assert_eq!(s.len(), 1);
}
#[test]
fn test_cfg_duplicate_names_two_spans() {
    let s = cut_item_spans("#[cfg(a)] pub fn f() {}\n#[cfg(not(a))] pub fn f() {}");
    assert_eq!(s.len(), 2);
    assert_eq!(s[0].name, "f");
    assert_eq!(s[1].name, "f");
}
#[test]
fn test_visibility_variants_classified() {
    let s = cut_item_spans("pub(crate) fn a() {} pub(super) fn b() {} fn c() {} pub fn d() {}");
    assert_eq!(
        s.iter().map(|x| x.vis.clone()).collect::<Vec<_>>(),
        vec![Vis::PubRestricted, Vis::PubRestricted, Vis::Priv, Vis::Pub]
    );
}
#[test]
fn test_line_col_derived_only() {
    assert_eq!(line_col("ab\ncd", 3), (2, 0));
}
#[test]
fn test_use_tree_ends_semicolon() {
    let src = "use a::{b, c};\nuse d;";
    let s = cut_item_spans(src);
    assert_eq!(
        names(&s),
        vec![(Kind::Use, "a".into()), (Kind::Use, "d".into())]
    );
    assert_eq!(s[0].end, 14);
}
#[test]
fn test_tuple_struct_ends_semicolon() {
    let s = cut_item_spans("pub struct T(u32);\npub fn f() {}");
    assert_eq!(s.len(), 2);
    assert_eq!(s[0].end, 18);
}
#[test]
fn test_const_block_ends_semicolon() {
    let s = cut_item_spans(
        "const X: u8 = { 1 };\nstatic Y: &str = \"}\";\ntype Z = [u8; 3];\npub fn f() {}",
    );
    assert_eq!(s.len(), 4);
    assert_eq!(s[0].end, 20);
}
#[test]
fn test_extern_crate_kind_classified() {
    let s = cut_item_spans("extern crate alloc;\nextern \"C\" { fn x(); }\nmod a;");
    assert_eq!(
        s.iter().map(|x| x.kind.clone()).collect::<Vec<_>>(),
        vec![Kind::ExternCrate, Kind::ExternBlock, Kind::Mod]
    );
}
#[test]
fn test_impl_return_type_not_resync() {
    let s = cut_item_spans(
        "pub fn f<'a>() -> impl 'a + Iterator<Item = u8> {\n    x\n}\npub fn g(h: fn(u8) -> u8) {}",
    );
    assert_eq!(s.len(), 2);
    assert!(s.iter().all(|x| x.confidence > 0.9));
}
#[test]
fn test_path_macro_call_classified() {
    let s = cut_item_spans("kernel::impl_deref!(unsafe { Device });\npub fn f() {}");
    assert_eq!(s[0].kind, Kind::MacroCall);
    assert_eq!(s[0].end, 39);
    assert_eq!(s.len(), 2);
}
#[test]
fn test_extern_abi_fn_modifier() {
    let s =
        cut_item_spans("#[no_mangle]\nunsafe extern \"C\" fn cb(x: u8) -> u8 { x }\npub fn f() {}");
    assert_eq!(
        names(&s),
        vec![(Kind::Fn, "cb".into()), (Kind::Fn, "f".into())]
    );
    assert!(s[0].confidence > 0.9);
}
#[test]
fn test_half_item_not_swallow_next() {
    let s = cut_item_spans(
        "pub struct A;\npub fn half_typed(\n/// doc\n#[inline]\npub fn g() {}\npub struct S;",
    );
    assert_eq!(
        names(&s),
        vec![
            (Kind::Struct, "A".into()),
            (Kind::Fn, "half_typed".into()),
            (Kind::Fn, "g".into()),
            (Kind::Struct, "S".into())
        ]
    );
    assert!(s[1].confidence < 0.5 && s[2].confidence > 0.9);
}
