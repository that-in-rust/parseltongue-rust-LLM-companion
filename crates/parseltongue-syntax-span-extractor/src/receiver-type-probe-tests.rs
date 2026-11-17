#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::extract_member_declaration_rows;

const CORE_FIXTURE_SOURCE_TEXT: &str = "\
impl<T> [T] {\n\
    pub const fn len(&self) -> usize { 0 }\n\
    pub unsafe fn get_unchecked<I>(&self, index: I) -> &I::Output { loop {} }\n\
    pub const fn as_mut_ptr(&mut self) -> *mut T { loop {} }\n\
    pub fn first(&self) -> Option<&T> { None }\n\
}\n\
impl [u8] { pub fn first(&self) -> u8 { 0 } }\n\
impl<T, const N: usize> [T; N] { pub fn as_slice(&self) -> &[T] { self } }\n\
impl<T> MaybeUninit<T> {\n\
    pub const fn write(&mut self, val: T) -> &mut T { loop {} }\n\
    pub const fn as_mut_ptr(&mut self) -> *mut T { loop {} }\n\
}\n\
impl<T: Copy> Cell<T> { pub fn get(&self) -> T { loop {} } }\n\
impl<T> Guard<T> where T: Copy { pub fn take(self) -> T { loop {} } }\n\
impl<T: ?Sized> *mut T { pub const fn cast<U>(self) -> *mut U { loop {} } }\n\
impl Flag { #[cfg(target_os = \"none\")] pub fn raise(&self) {} }\n\
impl str { pub const fn len(&self) -> usize { 0 } }\n\
pub union MaybeUninit<T> { value: T }\n\
pub struct Cell<T>(T);\npub struct Guard<T>(T);\npub struct Flag;\n\
mod a { pub struct Twice<T>(T); }\nmod b { pub struct Twice<T>(T); }\n\
impl<T> Twice<T> { pub fn get(&self) {} }\n\
macro_rules! index { ($t:ty) => { impl SliceIndex for $t { fn get_unchecked(self) {} } } }\n\
pub trait ExactSizeIterator { fn len(&self) -> usize { 0 } }\n\
macro_rules! int_impl { () => { pub const fn ilog10(self) -> u32 { 0 } } }\n";

fn read_fixture_declaration_rows() -> Vec<Value> {
    extract_member_declaration_rows("core/src/fixture.rs", CORE_FIXTURE_SOURCE_TEXT)
}

fn shape(text: &str) -> WrittenTypeShapeValue {
    parse_written_type_shape(text, &[])
}

/// The probe's outcome as text: `<impl self type>::<name> at <step>`, or the withheld detail.
fn probe(receiver: &str, name: &str, traits: &[ScopeTraitMethodValue]) -> String {
    let rows = read_fixture_declaration_rows();
    match probe_core_member_declaration(&shape(receiver), name, &rows, traits, &[]) {
        CoreMemberProbeOutcome::Named {
            declaration_row_index,
            probe_step_text,
            prelude_trait_assumed_names,
        } => format!(
            "{}::{} at {probe_step_text}{}",
            rows[declaration_row_index]["impl_self_type_text"]
                .as_str()
                .unwrap(),
            rows[declaration_row_index]["member_declared_name_value"]
                .as_str()
                .unwrap(),
            if prelude_trait_assumed_names.is_empty() {
                String::new()
            } else {
                format!(" assuming {}", prelude_trait_assumed_names.join(", "))
            }
        ),
        CoreMemberProbeOutcome::Withheld(detail) => format!("withheld {detail}"),
    }
}

/// REQ-RUST-056.0 P1
#[test]
fn test_written_type_shape_parse() {
    let named = |name: &str, arguments: Vec<WrittenTypeShapeValue>| WrittenTypeShapeValue::Named {
        name: name.to_owned(),
        arguments,
    };
    assert_eq!(
        WrittenTypeShapeValue::Reference {
            mutable: true,
            inner: Box::new(WrittenTypeShapeValue::Slice(Box::new(named(
                "MaybeUninit",
                vec![named("u8", vec![])]
            )))),
        },
        shape("&mut [core::mem::MaybeUninit<u8>]")
    );
    assert_eq!(
        WrittenTypeShapeValue::Array(Box::new(named("u8", vec![]))),
        shape("[u8; 200]")
    );
    assert_eq!(
        WrittenTypeShapeValue::Pointer {
            mutable: false,
            inner: Box::new(WrittenTypeShapeValue::Wildcard),
        },
        parse_written_type_shape("*const T", &["T".to_owned()])
    );
    assert_eq!(
        WrittenTypeShapeValue::Reference {
            mutable: false,
            inner: Box::new(named("str", vec![])),
        },
        shape("&'static str")
    );
    // A parameter written twice: only its first use is a wildcard.
    assert_eq!(
        named(
            "Pair",
            vec![
                WrittenTypeShapeValue::Wildcard,
                WrittenTypeShapeValue::Opaque
            ]
        ),
        parse_written_type_shape("Pair<T, T>", &["T".to_owned()])
    );
    assert_eq!(
        WrittenTypeShapeValue::Opaque,
        parse_written_type_shape("T::Item", &["T".to_owned()])
    );
    for text in [
        "Self::Buffer",
        "<u8 as Sealed>::Buffer",
        "(u8, u8)",
        "dyn Any",
        "_",
        "Self",
        "",
    ] {
        assert_eq!(WrittenTypeShapeValue::Opaque, shape(text), "{text}");
    }
}

/// REQ-RUST-056.0 P2, P3
#[test]
fn test_slice_member_probe_steps() {
    // The reference itself is the receiver type of a `&self` member: the first step.
    assert_eq!(
        "[T]::get_unchecked at step 0 by value",
        probe("&[MaybeUninit<u8>]", "get_unchecked", &[])
    );
    // A place of the element type takes `&mut` of itself.
    assert_eq!(
        "MaybeUninit<T>::write at step 0 by mutable reference",
        probe("MaybeUninit<u8>", "write", &[])
    );
    // `&mut [T]` is dereferenced once before `&self` fits.
    assert_eq!(
        "[T]::get_unchecked at step 1 by reference",
        probe("&mut [u8]", "get_unchecked", &[])
    );
    assert_eq!(
        "*mut T::cast at step 0 by value",
        probe("*mut u8", "cast", &[])
    );
    assert_eq!("str::len at step 0 by value", probe("&str", "len", &[]));
    assert_eq!(
        "withheld no_member_declared",
        probe("&[u8]", "missing", &[])
    );
    // Two inherent impls fit `[u8]`.
    assert_eq!(
        "withheld inherent_member_ambiguous",
        probe("&[u8]", "first", &[])
    );
    assert_eq!(
        "[T]::first at step 0 by value",
        probe("&[u16]", "first", &[])
    );
}

/// REQ-RUST-056.0 P2
#[test]
fn test_array_unsize_probe_order() {
    assert_eq!(
        "[T]::get_unchecked at step 1 by reference",
        probe("[u8; 200]", "get_unchecked", &[])
    );
    assert_eq!(
        "[T]::as_mut_ptr at step 1 by mutable reference",
        probe("[MaybeUninit<u8>; 40]", "as_mut_ptr", &[])
    );
    assert_eq!(
        "[T; N]::as_slice at step 0 by reference",
        probe("[u8; 4]", "as_slice", &[])
    );
    assert_eq!(
        "[T]::len at step 2 by reference",
        probe("&mut [u8; 4]", "len", &[])
    );
}

/// REQ-RUST-056.0 P3: what the rows cannot decide withholds the name.
#[test]
fn test_unproven_member_stays_withheld() {
    // An impl with a bound may not apply; rustc would then look further.
    assert_eq!(
        "withheld inherent_member_unproven",
        probe("Cell<u8>", "get", &[])
    );
    assert_eq!(
        "withheld inherent_member_unproven",
        probe("Guard<u8>", "take", &[])
    );
    // A member under `cfg` may not exist.
    assert_eq!(
        "withheld inherent_member_unproven",
        probe("Flag", "raise", &[])
    );
    // A member written in a macro body has no known owner.
    assert_eq!(
        "withheld macro_member_owner_unknown",
        probe("u32", "ilog10", &[])
    );
    // A receiver type the model does not read.
    assert_eq!(
        "withheld receiver_type_not_modelled",
        probe("Self::Buffer", "len", &[])
    );
    assert_eq!(
        "withheld inherent_member_unproven",
        probe("&[Self::Item]", "first", &[])
    );
    // A type name core declares twice, or not at all, does not say which type it is.
    assert_eq!(
        "withheld receiver_type_name_unproven",
        probe("Twice<u8>", "get", &[])
    );
    assert_eq!(
        "withheld receiver_type_name_unproven",
        probe("&[Absent]", "len", &[])
    );
}

/// REQ-RUST-056.0 P4, P5
#[test]
fn test_scope_trait_blocks_target() {
    let unread = |kind: &str| ScopeTraitMethodValue {
        member_self_parameter_kind: kind.to_owned(),
        trait_impl_self_shapes: None,
        prelude_trait_name_value: None,
    };
    let stored = |kind: &str, types: &[&str]| ScopeTraitMethodValue {
        member_self_parameter_kind: kind.to_owned(),
        trait_impl_self_shapes: Some(types.iter().map(|text| shape(text)).collect()),
        prelude_trait_name_value: None,
    };
    let prelude = |kind: &str, name: &str| ScopeTraitMethodValue {
        prelude_trait_name_value: Some(name.to_owned()),
        ..unread(kind)
    };
    // REQ-RUST-060.0 U1, U3: a trait of `core` in scope through the prelude only does
    // not end the probe; the name it then finds carries the trait's name.
    assert_eq!(
        "[T]::len at step 2 by reference assuming ExactSizeIterator",
        probe(
            "&mut [u8; 4]",
            "len",
            &[prelude("reference", "ExactSizeIterator")]
        )
    );
    // U4: nothing named after it: the trait's method may be the call's.
    assert_eq!(
        "withheld scope_trait_may_fit",
        probe(
            "&mut [u8; 4]",
            "absent",
            &[prelude("reference", "ExactSizeIterator")]
        )
    );
    // U5: any other trait that may fit still ends the probe.
    assert_eq!(
        "withheld scope_trait_may_fit",
        probe(
            "&mut [u8; 4]",
            "len",
            &[
                prelude("reference", "ExactSizeIterator"),
                unread("reference")
            ]
        )
    );
    // U6: an inherent member at the first form carries no assumption.
    assert_eq!(
        "[T]::len at step 0 by value",
        probe("&[u8]", "len", &[prelude("reference", "ExactSizeIterator")])
    );
    // `ExactSizeIterator::len(&self)` could fit `&mut [u8; 4]` taken by reference, which
    // comes before the slice.
    assert_eq!(
        "withheld scope_trait_may_fit",
        probe("&mut [u8; 4]", "len", &[unread("reference")])
    );
    // A trait whose impls are all stored and none fits does not block.
    assert_eq!(
        "MaybeUninit<T>::write at step 0 by mutable reference",
        probe(
            "MaybeUninit<u8>",
            "write",
            &[stored("value", &["u8", "i64"])]
        )
    );
    // One of its impls fits: the call is that trait's method, not a member of core.
    assert_eq!(
        "withheld scope_trait_may_fit",
        probe(
            "MaybeUninit<u8>",
            "write",
            &[stored("value", &["MaybeUninit<u8>"])]
        )
    );
    // A by-value trait method whose impls are not read blocks at the first step.
    assert_eq!(
        "withheld scope_trait_may_fit",
        probe("MaybeUninit<u8>", "write", &[unread("value")])
    );
    // An inherent member wins over a trait method at the same receiver type.
    assert_eq!(
        "[T]::get_unchecked at step 0 by value",
        probe("&[u8]", "get_unchecked", &[unread("reference")])
    );
}

/// REQ-RUST-056.0 P1, REQ-RUST-054.0 D12: a macro that takes its member's name from its
/// caller may declare the probed name unless every call of it in `core` is read and
/// none writes that name.
#[test]
fn test_macro_name_parameter_calls() {
    let outcome = |extra: &str| {
        let source = format!(
            "impl<T> [T] {{ pub const fn len(&self) -> usize {{ 0 }} }}\n\
             macro_rules! member {{ ($name:ident) => {{ fn $name(self) {{}} }} }}\n{extra}"
        );
        let rows = extract_member_declaration_rows("core/src/fixture.rs", &source);
        match probe_core_member_declaration(&shape("&[u8]"), "len", &rows, &[], &[]) {
            CoreMemberProbeOutcome::Named { .. } => "named",
            CoreMemberProbeOutcome::Withheld(detail) => detail,
        }
    };
    let unknown = "macro_member_owner_unknown";
    assert_eq!(unknown, outcome(""));
    assert_eq!("named", outcome("impl Iter { member! { fold } }"));
    assert_eq!(
        unknown,
        outcome("impl Iter { member! { fold } member!(len); }")
    );
    let passed = "impl Iter { member! { fold } }\n\
        macro_rules! outer { ($n:ident) => { member!($n); } }";
    assert_eq!(unknown, outcome(passed));
    let called = "impl Iter { member! { fold } }\n\
        macro_rules! outer { ($m:ident) => { $m!(fold); } }";
    // A call of a macro variable is a call of `member` only when a call in `core`
    // writes the name `member` in its arguments.
    assert_eq!("named", outcome(called));
    assert_eq!(unknown, outcome(&format!("{called}\nouter!(member);")));
    assert_eq!(
        unknown,
        outcome("impl Iter { member! { fold } } use member as other;")
    );
}

/// REQ-RUST-062.0 C4, C5: in a return type, a generic argument that the declaration
/// proves `Sized` fits a generic parameter of an impl; one it does not prove fits nothing.
#[test]
fn test_return_type_sized_arguments() {
    let rows = extract_member_declaration_rows(
        "core/src/result.rs",
        "pub enum Result<T, E> { Ok(T), Err(E) }\npub enum Option<T> { None, Some(T) }\n\
         impl<T, E> Result<T, E> { pub fn expect(self, msg: &str) -> T { loop {} } }\n\
         impl<T> Option<T> { pub fn expect(self, msg: &str) -> T { loop {} } }\n",
    );
    let names = |values: &[&str]| {
        values
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>()
    };
    let outcome = |sized: &[&str]| {
        let shape =
            parse_return_type_shape("Result<T, Self::Error>", &names(sized), &names(&["T"]));
        match probe_core_member_declaration(&shape, "expect", &rows, &[], &[]) {
            CoreMemberProbeOutcome::Named {
                declaration_row_index,
                probe_step_text,
                ..
            } => format!(
                "{}::expect at {probe_step_text}",
                rows[declaration_row_index]["impl_self_type_text"]
                    .as_str()
                    .unwrap()
            ),
            CoreMemberProbeOutcome::Withheld(detail) => format!("withheld {detail}"),
        }
    };
    assert_eq!(
        "Result<T, E>::expect at step 0 by value",
        outcome(&["T", "Error"])
    );
    // `?Sized` written for one of them: it may be a type an impl's parameter cannot be.
    assert_eq!("withheld inherent_member_unproven", outcome(&["Error"]));
    assert_eq!("withheld inherent_member_unproven", outcome(&["T"]));
    assert_eq!(
        WrittenTypeShapeValue::Opaque,
        parse_return_type_shape("Self", &[], &[])
    );
}
