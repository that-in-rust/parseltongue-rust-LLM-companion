#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::extract_member_declaration_rows;
use serde_json::{Value, json};

fn find_member_row_named<'a>(rows: &'a [Value], owner: &str, name: &str) -> &'a Value {
    rows.iter()
        .find(|row| {
            row["declaration_owner_kind_value"] == owner
                && row["member_declared_name_value"] == name
        })
        .unwrap_or_else(|| panic!("no {owner} row named {name} in {rows:#?}"))
}

/// REQ-RUST-054.0 D1, D2
#[test]
fn test_inherent_member_row_fields() {
    let source = "#[lang = \"maybe_uninit\"]\nimpl<T> MaybeUninit<T> {\n    #[stable(feature = \"maybe_uninit_write\", since = \"1.55.0\")]\n    pub const fn write(&mut self, val: T) -> &mut T { loop {} }\n    pub const fn uninit() -> MaybeUninit<T> { loop {} }\n}\nfn free() { impl Local { fn inner(self: Box<Self>) {} } }\n";
    let rows = extract_member_declaration_rows("mem/maybe_uninit.rs", source);
    assert_eq!(3, rows.len(), "{rows:#?}");
    let write = find_member_row_named(&rows, "inherent_impl", "write");
    let offset = source.find("write(").unwrap();
    assert_eq!(
        json!({
            "declaration_source_relative_path":"mem/maybe_uninit.rs",
            "member_name_start_offset":offset,
            "member_name_end_offset":offset + 5,
            "member_declared_name_value":"write",
            "member_name_parameter_flag":false,
            "declaration_owner_kind_value":"inherent_impl",
            "impl_self_type_text":"MaybeUninit<T>",
            "impl_generic_parameter_text":"<T>",
            "impl_generic_parameter_names":["T"],
            "impl_bound_restriction_flag":false,
            "macro_self_parameter_flag":null,
            "macro_owner_shape_kind":null,
            "type_declaration_kind_value":null,
            "owner_trait_path_text":null,
            "owner_macro_name_value":null,
            "member_self_parameter_kind":"mutable_reference",
            "member_self_parameter_text":"&mut self",
            "member_return_type_text":"&mut T",
            "member_attribute_text_values":["#[stable(feature = \"maybe_uninit_write\", since = \"1.55.0\")]"],
            "owner_attribute_text_values":["#[lang = \"maybe_uninit\"]"],
            "owner_function_body_flag":false,
        }),
        *write
    );
    let uninit = find_member_row_named(&rows, "inherent_impl", "uninit");
    assert_eq!(json!("none"), uninit["member_self_parameter_kind"]);
    assert_eq!(Value::Null, uninit["member_self_parameter_text"]);
    assert_eq!(json!("MaybeUninit<T>"), uninit["member_return_type_text"]);
    let inner = find_member_row_named(&rows, "inherent_impl", "inner");
    assert_eq!(json!(true), inner["owner_function_body_flag"]);
    assert_eq!(json!("typed"), inner["member_self_parameter_kind"]);
    assert_eq!(
        json!("self: Box<Self>"),
        inner["member_self_parameter_text"]
    );
    assert_eq!(Value::Null, inner["member_return_type_text"]);
}

/// REQ-RUST-054.0 D1, D2
#[test]
fn test_trait_member_row_fields() {
    let source = "pub trait TryInto<T>: Sized {\n    type Error;\n    fn try_into(self) -> Result<T, Self::Error>;\n}\nimpl<T, U> TryInto<U> for T where U: TryFrom<T> {\n    fn try_into(self) -> Result<U, U::Error> { loop {} }\n}\nimpl<T> [T] { pub const fn len(&self) -> usize { 0 } }\nimpl<T: PointeeSized> *mut T { pub const fn cast<U>(self) -> *mut U { loop {} } }\n";
    let rows = extract_member_declaration_rows("convert/mod.rs", source);
    assert_eq!(4, rows.len(), "{rows:#?}");
    let declared = find_member_row_named(&rows, "trait_declaration", "try_into");
    assert_eq!(json!("TryInto"), declared["owner_trait_path_text"]);
    assert_eq!(Value::Null, declared["impl_self_type_text"]);
    assert_eq!(json!("value"), declared["member_self_parameter_kind"]);
    assert_eq!(
        json!("Result<T, Self::Error>"),
        declared["member_return_type_text"]
    );
    let implemented = find_member_row_named(&rows, "trait_impl", "try_into");
    assert_eq!(json!("TryInto<U>"), implemented["owner_trait_path_text"]);
    assert_eq!(json!("T"), implemented["impl_self_type_text"]);
    assert_eq!(json!("<T, U>"), implemented["impl_generic_parameter_text"]);
    let len = find_member_row_named(&rows, "inherent_impl", "len");
    assert_eq!(json!("[T]"), len["impl_self_type_text"]);
    assert_eq!(json!("reference"), len["member_self_parameter_kind"]);
    assert_eq!(json!("usize"), len["member_return_type_text"]);
    let cast = find_member_row_named(&rows, "inherent_impl", "cast");
    assert_eq!(json!("*mut T"), cast["impl_self_type_text"]);
    assert_eq!(
        json!("<T: PointeeSized>"),
        cast["impl_generic_parameter_text"]
    );
    assert_eq!(json!("value"), cast["member_self_parameter_kind"]);
}

/// REQ-RUST-054.0 D3, D5
#[test]
fn test_macro_body_member_rows() {
    let source = "macro_rules! uint_impl {\n    (Self = $SelfT:ty, $name:ident) => {\n        impl $SelfT {\n            pub const fn ilog10(self) -> u32 { self.checked_ilog10().unwrap() }\n            pub fn $name(self) {}\n        }\n    };\n}\nuint_impl! { Self = u8, extra }\n";
    let rows = extract_member_declaration_rows("num/uint_macros.rs", source);
    assert_eq!(2, rows.len(), "{rows:#?}");
    let ilog10 = find_member_row_named(&rows, "macro_definition_body", "ilog10");
    assert_eq!(json!("uint_impl"), ilog10["owner_macro_name_value"]);
    assert_eq!(Value::Null, ilog10["impl_self_type_text"]);
    assert_eq!(Value::Null, ilog10["member_self_parameter_kind"]);
    assert_eq!(json!(false), ilog10["member_name_parameter_flag"]);
    assert_eq!(
        json!(source.find("ilog10").unwrap()),
        ilog10["member_name_start_offset"]
    );
    let named = find_member_row_named(&rows, "macro_definition_body", "$name");
    assert_eq!(json!(true), named["member_name_parameter_flag"]);
}

/// REQ-RUST-054.0 D4
#[test]
fn test_member_rows_offset_order() {
    let source = "impl A { fn second_one(&self) {} fn first_one(self) {} }\ntrait B { fn third_one(&mut self); }\nmacro_rules! m { () => { fn fourth_one() {} } }\n";
    let rows = extract_member_declaration_rows("a.rs", source);
    let names = rows
        .iter()
        .map(|row| row["member_declared_name_value"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        vec!["second_one", "first_one", "third_one", "fourth_one"],
        names
    );
    let offsets = rows
        .iter()
        .map(|row| row["member_name_start_offset"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert!(
        offsets.windows(2).all(|pair| pair[0] < pair[1]),
        "{offsets:?}"
    );
}

/// REQ-RUST-054.0 D6: what an impl's header says about which types it applies to.
#[test]
fn test_impl_bound_restriction_rows() {
    let source = "impl<T> A<T> { fn a(&self) {} }\n\
        impl<T: ?Sized> B<T> { fn b(&self) {} }\n\
        impl<T: Copy> C<T> { fn c(&self) {} }\n\
        impl<T> D<T> where T: Copy { fn d(&self) {} }\n\
        impl<'a, T, const N: usize> E<'a, T, N> { fn e(&self) {} }\n\
        trait Tr { fn t(&self); }\n";
    let rows = extract_member_declaration_rows("x.rs", source);
    let read = |name: &str| {
        let row = find_member_row_named(&rows, "inherent_impl", name);
        (
            row["impl_bound_restriction_flag"].clone(),
            row["impl_generic_parameter_names"].clone(),
        )
    };
    assert_eq!((json!(false), json!(["T"])), read("a"));
    assert_eq!((json!(false), json!(["T"])), read("b"));
    assert_eq!((json!(true), json!(["T"])), read("c"));
    assert_eq!((json!(true), json!(["T"])), read("d"));
    assert_eq!((json!(false), json!(["T", "N"])), read("e"));
    let declared = find_member_row_named(&rows, "trait_declaration", "t");
    assert_eq!(Value::Null, declared["impl_bound_restriction_flag"]);
}

/// REQ-RUST-054.0 D7: what the tokens around a `fn` in a macro body say about its owner.
#[test]
fn test_macro_owner_shape_rows() {
    let source = "macro_rules! m { ($t:ty, $n:ident) => {\n\
        impl Index for $t { fn get(self) {} $(fn $n(&self) {})* }\n\
        impl $t { fn put(&mut self) {} }\n\
        impl<T> Wrap<for<'a> fn(&'a T)> { fn call(&self) {} }\n\
        impl<T> Held<T> where for<'a> T: Fn(&'a u8) { fn held(&self) {} }\n\
        trait Local { fn decl(&self); }\n\
        fn $n(val: u8) {}\n\
        fn bare(self) {}\n\
        } }\n";
    let rows = extract_member_declaration_rows("x.rs", source);
    let read = |name: &str| {
        let row = find_member_row_named(&rows, "macro_definition_body", name);
        (
            row["macro_self_parameter_flag"].as_bool(),
            row["macro_owner_shape_kind"].as_str().map(str::to_owned),
        )
    };
    let shape = |flag: bool, kind: &str| (Some(flag), Some(kind.to_owned()));
    assert_eq!(shape(true, "trait_impl"), read("get"));
    assert_eq!(shape(true, "inherent_impl"), read("put"));
    assert_eq!(shape(true, "inherent_impl"), read("call"));
    assert_eq!(shape(true, "inherent_impl"), read("held"));
    assert_eq!(shape(true, "trait_declaration"), read("decl"));
    assert_eq!(shape(true, "unknown"), read("bare"));
    let named = rows
        .iter()
        .filter(|row| row["member_declared_name_value"] == "$n")
        .map(|row| {
            (
                row["macro_self_parameter_flag"].as_bool(),
                row["macro_owner_shape_kind"].as_str().map(str::to_owned),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        vec![shape(true, "trait_impl"), shape(false, "unknown")],
        named
    );
}

/// REQ-RUST-054.0 D8: a `fn` written in the arguments of a macro call.
#[test]
fn test_macro_call_argument_rows() {
    let source = "forward! { impl Foo { fn made(&self) {} } }\n\
        fn body() { other!(fn inner() {}); }\n";
    let rows = extract_member_declaration_rows("x.rs", source);
    let made = find_member_row_named(&rows, "macro_call_argument", "made");
    assert_eq!(json!("forward"), made["owner_macro_name_value"]);
    assert_eq!(json!(true), made["macro_self_parameter_flag"]);
    assert_eq!(json!("inherent_impl"), made["macro_owner_shape_kind"]);
    assert_eq!(json!(false), made["owner_function_body_flag"]);
    let inner = find_member_row_named(&rows, "macro_call_argument", "inner");
    assert_eq!(json!("other"), inner["owner_macro_name_value"]);
    assert_eq!(json!(false), inner["macro_self_parameter_flag"]);
    assert_eq!(json!(true), inner["owner_function_body_flag"]);
}

/// REQ-RUST-054.0 D9: the type names a file declares or renames.
#[test]
fn test_type_name_declaration_rows() {
    let source = "pub struct S<T>(T);\nenum E {}\nunion U { a: u8 }\ntype A = u8;\n\
        use x::Y as Z;\nuse x::W as _;\nimpl S<u8> { type Inner = u8; }\ntrait Tr { type Out; }\n";
    let rows = extract_member_declaration_rows("x.rs", source);
    let found = rows
        .iter()
        .filter(|row| row["declaration_owner_kind_value"] == "type_name_declaration")
        .map(|row| {
            let start = row["member_name_start_offset"].as_u64().unwrap() as usize;
            let end = row["member_name_end_offset"].as_u64().unwrap() as usize;
            assert_eq!(
                json!(&source[start..end]),
                row["member_declared_name_value"]
            );
            format!(
                "{} {}",
                row["type_declaration_kind_value"].as_str().unwrap(),
                &source[start..end]
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        vec![
            "struct S",
            "enum E",
            "union U",
            "type_alias A",
            "use_rename Z"
        ],
        found
    );
}

/// REQ-RUST-054.0 D10: the names a prelude file re-exports, one row per `use` leaf. A
/// `use` in a private inline module is not a re-export of the prelude; a glob keeps its path.
#[test]
fn test_prelude_use_leaf_rows() {
    let source = "pub use crate::iter::{ExactSizeIterator, Iterator as _};\n\
        pub use crate::convert::TryInto;\n\
        mod hidden { pub use crate::*; pub use crate::Secret; }\n\
        pub mod rust_2021 { pub use super::v1::*; pub use crate::future::Future; }\n";
    let read = |path: &str| {
        extract_member_declaration_rows(path, source)
            .iter()
            .filter(|row| row["declaration_owner_kind_value"] == "prelude_use_leaf")
            .map(|row| {
                let start = row["member_name_start_offset"].as_u64().unwrap() as usize;
                let end = row["member_name_end_offset"].as_u64().unwrap() as usize;
                assert_eq!(
                    json!(&source[start..end]),
                    row["member_declared_name_value"]
                );
                match row["impl_self_type_text"].as_str() {
                    Some(path) => format!("{} of {path}", &source[start..end]),
                    None => source[start..end].to_owned(),
                }
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        vec![
            "ExactSizeIterator",
            "Iterator",
            "TryInto",
            "* of super::v1",
            "Future"
        ],
        read("core/src/prelude/v1.rs")
    );
    assert!(read("core/src/iter/mod.rs").is_empty());
}
