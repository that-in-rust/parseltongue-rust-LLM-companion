#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::find_macro_impl_member;
use crate::sysroot_member_declaration_records::extract_member_declaration_rows;
use serde_json::{Value, json};

const NUM: &str = "core/src/num/mod.rs";
const UINT: &str = "core/src/num/uint_macros.rs";
const NUM_TEXT: &str = "mod imp;\n#[macro_use] mod int_macros;\n#[macro_use] mod uint_macros;\n\
    impl u8 { uint_impl! { Self = u8 } other! {} }\nimpl i8 { int_impl! {} }\n";
const UINT_TEXT: &str = "macro_rules! uint_impl { (Self = $S:ty) => {\n\
    pub const MAX: Self = !0;\n pub const fn ilog10(self) -> u32 { 0 }\n}; }\n";
const INT_TEXT: &str =
    "macro_rules! int_impl { () => { pub const fn ilog10(self) -> u32 { 1 } }; }\n";
const BITS_TEXT: &str = "macro_rules! uint_impl { ($U:ident) => { mod $U { fn ilog10() {} } }; }\n";

/// The declaration rows of a small `core::num`, each file replaced or joined by `swap`.
fn list_macro_fixture_rows(swap: &[(&str, &str)]) -> Vec<Value> {
    [
        (NUM, NUM_TEXT),
        (UINT, UINT_TEXT),
        ("core/src/num/int_macros.rs", INT_TEXT),
        ("core/src/num/imp/int_bits.rs", BITS_TEXT),
    ]
    .into_iter()
    .chain(swap.iter().copied())
    .collect::<std::collections::BTreeMap<_, _>>()
    .into_iter()
    .flat_map(|(path, text)| extract_member_declaration_rows(path, text))
    .collect()
}

fn build_site_value(path: &str, text: &str, name: &str) -> Value {
    let start = text.find(name).unwrap();
    json!({
        "declaration_source_relative_path":path,
        "member_name_start_offset":start,
        "member_name_end_offset":start + name.len()
    })
}

/// REQ-RUST-063.0 M4, M5: a const and a method that a macro of a `#[macro_use]` module
/// writes into `impl u8`, with a second macro of the same name elsewhere in `core`.
#[test]
fn test_macro_impl_member_proven() {
    let rows = list_macro_fixture_rows(&[]);
    let method = find_macro_impl_member(&rows, "u8", "ilog10", "fn").unwrap();
    assert_eq!(
        json!({
            "builtin_type_name_value":"u8",
            "member_declared_name_value":"ilog10",
            "member_item_kind_value":"fn",
            "member_written_type_text":null,
            "impl_macro_call_site_value":build_site_value(NUM, NUM_TEXT, "uint_impl"),
            "macro_scope_proof_value":{
                "macro_scope_proof_kind":"macro_use_module",
                "macro_definition_site_value":build_site_value(UINT, UINT_TEXT, "uint_impl"),
                "macro_arm_count_value":1,
                "macro_use_module_site_value":build_site_value(NUM, NUM_TEXT, "uint_macros")
            },
            "macro_body_item_site_value":build_site_value(UINT, UINT_TEXT, "ilog10"),
            "inherent_lookup_rule_kind":"inherent_item_declared_once"
        }),
        method
    );
    let constant = find_macro_impl_member(&rows, "u8", "MAX", "const").unwrap();
    assert_eq!("Self", constant["member_written_type_text"]);
    // `int_impl` is found behind the `#[macro_use]` module that does not define it.
    let signed = find_macro_impl_member(&rows, "i8", "ilog10", "fn").unwrap();
    assert_eq!(
        "core/src/num/int_macros.rs",
        signed["macro_body_item_site_value"]["declaration_source_relative_path"]
    );
    // A definition written in the file itself, after the module.
    let local = NUM_TEXT.replace("impl u8", &format!("{UINT_TEXT}impl u8"));
    let rows = list_macro_fixture_rows(&[(NUM, &local)]);
    let found = find_macro_impl_member(&rows, "u8", "ilog10", "fn").unwrap();
    assert_eq!(
        "same_file",
        found["macro_scope_proof_value"]["macro_scope_proof_kind"]
    );
}

/// REQ-RUST-063.0 M4, M5: each form that leaves the member unproven.
#[test]
fn test_macro_impl_member_unproven() {
    let num = |from: &str, to: &str| (NUM, NUM_TEXT.replace(from, to));
    let uint = |from: &str, to: &str| (UINT, UINT_TEXT.replace(from, to));
    let cases = [
        (
            "module under cfg",
            num("#[macro_use] mod uint", "#[cfg(x)] #[macro_use] mod uint"),
        ),
        (
            "module with a path",
            num(
                "#[macro_use] mod uint",
                "#[path = \"a.rs\"] #[macro_use] mod uint",
            ),
        ),
        (
            "module without macro_use",
            num("#[macro_use] mod uint", "mod uint"),
        ),
        (
            "a macro call before the impl",
            num("impl u8", "make!();\nimpl u8"),
        ),
        (
            "an extern crate under macro_use",
            num("impl u8", "#[macro_use] extern crate x;\nimpl u8"),
        ),
        ("impl under cfg", num("impl u8", "#[cfg(x)] impl u8")),
        (
            "call under cfg",
            num("uint_impl! {", "#[cfg(x)] uint_impl! {"),
        ),
        ("a trait impl", num("impl u8", "impl Shown for u8")),
        ("a generic impl", num("impl u8", "impl<T> u8")),
        ("two calls", num("other! {}", "uint_impl! {}")),
        ("written out too", num("other! {}", "fn ilog10(self) {}")),
        (
            "a second definition",
            uint("}; }\n", "}; }\nmacro_rules! uint_impl { () => {}; }\n"),
        ),
        (
            "a macro call after the definition",
            uint("}; }\n", "}; }\nmake!();\n"),
        ),
        (
            "definition under cfg",
            uint("macro_rules!", "#[cfg(x)] macro_rules!"),
        ),
        (
            "two arms",
            uint("(Self = $S:ty) =>", "() => {}; (Self = $S:ty) =>"),
        ),
        (
            "the name twice",
            uint("pub const MAX", "fn ilog10(self) {} pub const MAX"),
        ),
        (
            "in inner braces",
            uint(
                "pub const fn ilog10(self) -> u32 { 0 }",
                "mod m { fn ilog10(self) {} }",
            ),
        ),
        (
            "in a repetition",
            uint(
                "pub const fn ilog10(self) -> u32 { 0 }",
                "$( fn ilog10(self) {} )*",
            ),
        ),
        (
            "item under cfg",
            uint("pub const fn ilog10", "#[cfg(x)] pub const fn ilog10"),
        ),
        (
            "attributes from the call",
            uint("pub const fn ilog10", "$(#[$m])* pub const fn ilog10"),
        ),
        ("no self", uint("ilog10(self)", "ilog10(value: u8)")),
        ("the name from the call", uint("fn ilog10", "fn $name")),
    ];
    let proven = cases
        .iter()
        .filter(|(_, (path, text))| {
            let rows = list_macro_fixture_rows(&[(path, text)]);
            assert_ne!(
                rows,
                list_macro_fixture_rows(&[]),
                "the swap changed nothing"
            );
            find_macro_impl_member(&rows, "u8", "ilog10", "fn").is_some()
        })
        .map(|(label, _)| *label)
        .collect::<Vec<_>>();
    assert!(proven.is_empty(), "{proven:?}");
    // The method is in a macro of the same name that is not the file-level one.
    let inner = format!("macro_rules! uint_impl {{ () => {{}}; }}\nmod inner {{ {UINT_TEXT} }}\n");
    let rows = list_macro_fixture_rows(&[(UINT, &inner)]);
    assert!(find_macro_impl_member(&rows, "u8", "ilog10", "fn").is_none());
    let rows = list_macro_fixture_rows(&[]);
    assert!(find_macro_impl_member(&rows, "u8", "ilog10", "const").is_none());
    assert!(find_macro_impl_member(&rows, "u16", "ilog10", "fn").is_none());
}
