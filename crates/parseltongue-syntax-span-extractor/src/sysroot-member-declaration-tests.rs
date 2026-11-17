#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::extract_member_declaration_rows;
use serde_json::{Value, json};

/// The rows of `source` that are not file scope rows (REQ-RUST-062.0 C9), for the tests
/// of the older row kinds.
fn list_member_declaration_rows(path: &str, source: &str) -> Vec<Value> {
    extract_member_declaration_rows(path, source)
        .into_iter()
        .filter(|row| row["declaration_owner_kind_value"] != "file_scope_name_binding")
        .collect()
}

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
    let rows = list_member_declaration_rows("mem/maybe_uninit.rs", source);
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
            "owner_file_scope_flag":true,
            "trait_generic_parameter_names":null,
            "trait_associated_type_names":null,
            "owner_unsized_relaxed_names":[],
            "member_generic_parameter_names":[],
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
    let rows = list_member_declaration_rows("convert/mod.rs", source);
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
    // REQ-RUST-062.0 C3: the names a return type may write, and those `?Sized` relaxes.
    assert_eq!(
        json!([["T"], ["Error"], [], []]),
        json!([
            declared["trait_generic_parameter_names"],
            declared["trait_associated_type_names"],
            declared["owner_unsized_relaxed_names"],
            declared["member_generic_parameter_names"]
        ])
    );
    assert_eq!(
        json!([null, null, ["T"], ["U"]]),
        json!([
            cast["trait_generic_parameter_names"],
            cast["trait_associated_type_names"],
            cast["owner_unsized_relaxed_names"],
            cast["member_generic_parameter_names"]
        ])
    );
    let relaxed = extract_member_declaration_rows(
        "ops/deref.rs",
        "pub trait Borrow<B: ?Sized, C> { type Target: ?Sized; type Kept; fn get(&self) -> Pair<B, C>; }\n\
         pub trait Late<A> where A: ?Sized { type Out; fn late(&self) -> Pair<A, Self::Out>; }\n\
         impl<T: ?Sized, U> Cell<T, U> { fn held(&self) -> Pair<T, U> { loop {} } }\n\
         impl<T: PointeeSized, U: MetaSized, V> Ptr<T, U, V> { fn at(&self) -> Pair<T, U> { loop {} } }\n",
    );
    let names = |name: &str| {
        relaxed
            .iter()
            .find(|row| row["member_declared_name_value"] == name)
            .map(|row| row["owner_unsized_relaxed_names"].clone())
    };
    assert_eq!(Some(json!(["B", "Target"])), names("get"));
    // A where clause that writes `?Sized` counts for every name of the trait.
    assert_eq!(Some(json!(["A", "Out"])), names("late"));
    assert_eq!(Some(json!(["T"])), names("held"));
    // `PointeeSized` and `MetaSized` as a bound take the implicit `Sized` away too.
    assert_eq!(Some(json!(["T", "U"])), names("at"));
}

/// REQ-RUST-054.0 D3, D5
#[test]
fn test_macro_body_member_rows() {
    let source = "macro_rules! uint_impl {\n    (Self = $SelfT:ty, $name:ident) => {\n        impl $SelfT {\n            pub const fn ilog10(self) -> u32 { self.checked_ilog10().unwrap() }\n            pub fn $name(self) {}\n        }\n    };\n}\nuint_impl! { Self = u8, extra }\n";
    let rows = list_member_declaration_rows("num/uint_macros.rs", source)
        .into_iter()
        .filter(|row| row["declaration_owner_kind_value"] != "macro_call_name_tokens")
        .collect::<Vec<_>>();
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
    let rows = list_member_declaration_rows("a.rs", source);
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
    // REQ-RUST-062.0 C9: the name each leaf binds and the full path it names.
    let bound = extract_member_declaration_rows(
        "core/src/prelude/v1.rs",
        "pub use crate::result::Result::{self, Err, Ok as _};\npub use super::v1::*;\n",
    )
    .iter()
    .filter(|row| row["declaration_owner_kind_value"] == "prelude_use_leaf")
    .map(|row| json!([row["use_leaf_bound_name_value"], row["use_leaf_path_text"]]))
    .collect::<Vec<_>>();
    assert_eq!(
        vec![
            json!(["Result", "crate::result::Result"]),
            json!(["Err", "crate::result::Result::Err"]),
            json!([null, "crate::result::Result::Ok"]),
            json!([null, "super::v1"]),
        ],
        bound
    );
}

/// REQ-RUST-054.0 D11: a `use .. as NAME` row keeps the last name of the path it
/// renames, which for `self` is the last name before the braces. An
/// `extern crate .. as NAME` names a crate and is a row of its own kind.
#[test]
fn test_rename_source_name_rows() {
    let source = "pub use crate::iter::Hidden as Shown;\n\
        pub use crate::fmt::{self as text, Write as Emit};\n\
        extern crate self as inner;\npub use super as up;\n";
    let found = extract_member_declaration_rows("core/src/slice/mod.rs", source)
        .into_iter()
        .filter(|row| row["declaration_owner_kind_value"] == "type_name_declaration")
        .map(|row| {
            format!(
                "{} {} from {}",
                row["type_declaration_kind_value"].as_str().unwrap(),
                row["member_declared_name_value"].as_str().unwrap(),
                row["rename_source_name_value"].as_str().unwrap_or("-")
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        vec![
            "use_rename Shown from Hidden",
            "use_rename text from fmt",
            "use_rename Emit from Write",
            "extern_crate_rename inner from -",
            "use_rename up from -"
        ],
        found
    );
}

/// REQ-RUST-054.0 D12: per macro name a file calls, the names written in those calls and
/// whether a call passes a macro variable on. A call of a macro variable has no name.
#[test]
fn test_macro_call_token_rows() {
    let source = "macro_rules! member { ($name:ident) => { fn $name(self) {} } }\n\
        impl A { member! { first } crate::member!(second); }\n\
        macro_rules! outer { ($n:ident, $m:ident) => { impl B { member!($n); $m!(third); } } }\n\
        fn f() { assert!(!(a != b)); }\n";
    let found = extract_member_declaration_rows("core/src/slice/mod.rs", source)
        .into_iter()
        .filter(|row| row["declaration_owner_kind_value"] == "macro_call_name_tokens")
        .map(|row| {
            format!(
                "{} {} {}",
                row["owner_macro_name_value"].as_str().unwrap_or("-"),
                row["macro_call_identifier_values"],
                row["macro_call_metavariable_flag"]
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        vec![
            r#"member ["first","n","second"] true"#,
            r#"- ["third"] false"#,
            r#"assert ["a","b"] false"#
        ],
        found
    );
}

/// REQ-RUST-062.0 C9: one row per direct child of a file that may bind a type name.
#[test]
fn test_file_scope_binding_rows() {
    let source = "#[stable] pub enum Result<T, E> { Ok(T), Err(E) }\n\
        struct Unit; union Bits { a: u8 } trait Shown {} type Alias = Unit;\n\
        #[path = \"x.rs\"] mod moved; pub mod inline { struct Inner; }\n\
        extern crate self as core; extern crate alloc;\n\
        #[prelude_import] use prelude::rust_2024::*;\n\
        pub use crate::result::Result::{self, Err as Bad, Ok as _};\n\
        include!(\"docs.rs\"); extern \"C\" { type Foreign; }\n\
        fn free() { struct Local; } const VALUE: u8 = 0; impl Unit {}\n";
    let rows = extract_member_declaration_rows("core/src/lib.rs", source);
    let read = rows
        .iter()
        .filter(|row| row["declaration_owner_kind_value"] == "file_scope_name_binding")
        .map(|row| {
            json!([
                row["scope_binding_form_kind"],
                row["type_declaration_kind_value"],
                row["member_declared_name_value"],
                row["use_leaf_path_text"]
            ])
        })
        .collect::<Vec<_>>();
    assert_eq!(
        vec![
            json!(["item", "enum", "Result", null]),
            json!(["item", "struct", "Unit", null]),
            json!(["item", "union", "Bits", null]),
            json!(["item", "trait", "Shown", null]),
            json!(["item", "type_alias", "Alias", null]),
            json!(["module", "module_file", "moved", null]),
            json!(["module", "module_inline", "inline", null]),
            json!(["extern_crate", null, "core", null]),
            json!(["extern_crate", null, "alloc", null]),
            json!(["use_glob", null, null, "prelude::rust_2024"]),
            json!(["use_leaf", null, "Result", "crate::result::Result"]),
            json!(["use_leaf", null, "Bad", "crate::result::Result::Err"]),
            json!(["unread_form", null, null, null]),
            json!(["unread_form", null, null, null]),
        ],
        read
    );
    let bound = |name: &str| {
        rows.iter()
            .find(|row| {
                row["declaration_owner_kind_value"] == "file_scope_name_binding"
                    && row["member_declared_name_value"] == name
            })
            .unwrap()
    };
    assert_eq!(
        json!(["#[stable]"]),
        bound("Result")["member_attribute_text_values"]
    );
    assert_eq!(
        json!(["#[path = \"x.rs\"]"]),
        bound("moved")["member_attribute_text_values"]
    );
    let glob = rows
        .iter()
        .find(|row| row["scope_binding_form_kind"] == "use_glob")
        .unwrap();
    assert_eq!(
        json!(["#[prelude_import]"]),
        glob["member_attribute_text_values"]
    );
    // A member row says whether its trait or impl is a direct child of the file.
    let rows = extract_member_declaration_rows(
        "core/src/a.rs",
        "impl A { fn top(&self) {} } mod m { impl A { fn deep(&self) {} } }",
    );
    assert_eq!(
        json!([true, false]),
        json!([
            find_member_row_named(&rows, "inherent_impl", "top")["owner_file_scope_flag"],
            find_member_row_named(&rows, "inherent_impl", "deep")["owner_file_scope_flag"]
        ])
    );
}

/// REQ-RUST-063.0 M1 to M3: a macro call in a file-level impl, a file-level
/// `macro_rules!`, and the `const` and `fn` items of a macro body with where they are.
#[test]
fn test_macro_impl_scope_rows() {
    let source = "macro_rules! two { () => {}; ($x:ident) => { fn second(self) {} }; }\n\
        #[macro_use] mod uint_macros;\n\
        macro_rules! uint_impl { (Self = $SelfT:ty) => {\n\
        #[doc = \"d\"]\n/// text\n#[stable] pub const MAX: Self = !0;\n\
        #[cfg(a)] #[inline] pub(crate) const fn gated(self) {}\n\
        pub const fn ilog10(self) -> u32 { const INNER: u8 = 0; 0 }\n\
        $(#[$m])* fn spliced(self) {}\n\
        $( fn repeated(self) {} )*\n\
        impl<const N: usize> Other<N> {}\n\
        }; }\n\
        #[lang = \"u8\"] impl u8 { #[doc = \"x\"] uint_impl! { Self = u8 } fn written(self) {} }\n\
        impl<T> Shown for Wrapper<T> { other!(); }\n\
        fn free() { impl u8 { local!(); } }\n";
    let rows = extract_member_declaration_rows("core/src/num/mod.rs", source);
    let scope = rows
        .iter()
        .filter(|row| row["scope_binding_form_kind"] == "macro_definition")
        .map(|row| {
            json!([
                row["member_declared_name_value"],
                row["macro_arm_count_value"]
            ])
        })
        .collect::<Vec<_>>();
    assert_eq!(
        vec![json!(["two", 2]), json!(["uint_impl", 1])],
        scope,
        "M2"
    );
    let calls = rows
        .iter()
        .filter(|row| row["declaration_owner_kind_value"] == "impl_macro_call")
        .map(|row| {
            let start = row["member_name_start_offset"].as_u64().unwrap() as usize;
            let end = row["member_name_end_offset"].as_u64().unwrap() as usize;
            json!([
                &source[start..end],
                row["owner_macro_name_value"],
                row["impl_self_type_text"],
                row["owner_trait_path_text"],
                row["impl_generic_parameter_text"],
                row["owner_attribute_text_values"],
                row["member_attribute_text_values"]
            ])
        })
        .collect::<Vec<_>>();
    assert_eq!(
        vec![
            json!([
                "uint_impl",
                "uint_impl",
                "u8",
                null,
                null,
                ["#[lang = \"u8\"]"],
                ["#[doc = \"x\"]"]
            ]),
            json!(["other", "other", "Wrapper<T>", "Shown", "<T>", [], []]),
        ],
        calls,
        "M1"
    );
    let body = rows
        .iter()
        .filter(|row| row["declaration_owner_kind_value"] == "macro_definition_body")
        .map(|row| {
            json!([
                row["member_declared_name_value"],
                row["member_item_kind_value"],
                row["member_written_type_text"],
                row["macro_arm_ordinal_value"],
                row["macro_body_nesting_depth"],
                row["macro_body_repetition_flag"],
                row["member_attribute_text_values"],
                row["macro_self_parameter_flag"],
                row["macro_item_boundary_flag"]
            ])
        })
        .collect::<Vec<_>>();
    assert_eq!(
        vec![
            json!(["second", "fn", null, 1, 0, false, [], true, true]),
            json!([
                "MAX",
                "const",
                "Self",
                0,
                0,
                false,
                ["#[doc = \"d\"]", "#[stable]"],
                false,
                true
            ]),
            json!([
                "gated",
                "fn",
                null,
                0,
                0,
                false,
                ["#[cfg(a)]", "#[inline]"],
                true,
                true
            ]),
            json!(["ilog10", "fn", null, 0, 0, false, [], true, true]),
            json!(["INNER", "const", "u8", 0, 1, false, [], false, true]),
            json!(["spliced", "fn", null, 0, 0, false, [], true, false]),
            json!(["repeated", "fn", null, 0, 1, true, [], true, true]),
        ],
        body,
        "M3"
    );
}
