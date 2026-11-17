#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::check_syntax_item_node;
use ra_ap_syntax::{Edition, SourceFile};

/// The signature of every item node of `source`, in source order.
fn print_source_item_signatures(source: &str) -> Vec<String> {
    SourceFile::parse(source, Edition::Edition2021)
        .syntax_node()
        .descendants()
        .filter(check_syntax_item_node)
        .map(|item| print_item_signature_text(&item))
        .collect()
}

/// REQ-RUST-069.0 S1, S2: one case per item kind of pinned itoa, and the cuts of S1.
#[test]
fn test_item_signature_text_printed() {
    let cases = [
        (
            "/// doc\n#[inline]\npub fn new() -> Buffer { Buffer { bytes: [0; 4] } }",
            vec!["pub fn new() -> Buffer"],
        ),
        (
            "pub fn format<I: Integer>(&mut self, i: I) -> &str { i.write(self) }",
            vec!["pub fn format<I: Integer>(&mut self, i: I) -> &str"],
        ),
        (
            "unsafe fn slice_buffer_to_str(buf: &[MaybeUninit<u8>], offset: usize) -> &str { x }",
            vec!["unsafe fn slice_buffer_to_str(buf: &[MaybeUninit<u8>], offset: usize) -> &str"],
        ),
        (
            "fn enc_16lsd<const OFFSET: usize>(buf: &mut [MaybeUninit<u8>], n: u64) {}",
            vec!["fn enc_16lsd<const OFFSET: usize>(buf: &mut [MaybeUninit<u8>], n: u64)"],
        ),
        (
            "fn divmod100(value: u32) -> (u32, u32) { (0, 0) }",
            vec!["fn divmod100(value: u32) -> (u32, u32)"],
        ),
        (
            "pub(crate) fn mulhi(x: u128, y: u128) -> u128 { x }",
            vec!["pub(crate) fn mulhi(x: u128, y: u128) -> u128"],
        ),
        (
            "pub struct Buffer {\n    bytes: [MaybeUninit<u8>; i128::MAX_STR_LEN],\n}",
            vec![
                "pub struct Buffer",
                "bytes: [MaybeUninit<u8>; i128::MAX_STR_LEN]",
            ],
        ),
        (
            "struct DecimalPairs([u8; 200]);",
            vec!["struct DecimalPairs", "[u8; 200]"],
        ),
        (
            "static DECIMAL_PAIRS: DecimalPairs = DecimalPairs(*b\"00\");",
            vec!["static DECIMAL_PAIRS: DecimalPairs"],
        ),
        (
            "const _: () = { fn hidden() {} };",
            vec!["const _: ()", "fn hidden()"],
        ),
        (
            "pub trait Integer: private::Sealed {\n    const MAX_STR_LEN: usize;\n}",
            vec![
                "pub trait Integer: private::Sealed",
                "const MAX_STR_LEN: usize",
            ],
        ),
        (
            "trait Sealed: Copy { type Buffer: 'static; fn write(self, buf: &mut Self::Buffer) -> &str; }",
            vec![
                "trait Sealed: Copy",
                "type Buffer: 'static",
                "fn write(self, buf: &mut Self::Buffer) -> &str",
            ],
        ),
        (
            "impl Sealed for u8 { type Buffer = <u64 as private::Sealed>::Buffer; }",
            vec![
                "impl Sealed for u8",
                "type Buffer = <u64 as private::Sealed>::Buffer",
            ],
        ),
        ("mod u128_ext;", vec!["mod u128_ext"]),
        ("mod private { }", vec!["mod private"]),
        (
            "use core::mem::{self, MaybeUninit};",
            vec!["use core::mem::{self, MaybeUninit}"],
        ),
        (
            "macro_rules! impl_Integer { ($a:ty) => {}; }",
            vec!["macro_rules! impl_Integer"],
        ),
        (
            "impl_Integer_size!(isize as i64 #[cfg(target_pointer_width = \"64\")]);",
            vec!["impl_Integer_size!(isize as i64 #[cfg(target_pointer_width = \"64\")])"],
        ),
        (
            "fn bound<'a, T: ?Sized + 'a>(value: &'a T, raw: *const u8) -> impl Fn(u8) -> bool where T: Copy, {}",
            vec![
                "fn bound<'a, T: ?Sized + 'a>(value: &'a T, raw: *const u8) -> impl Fn(u8) -> bool where T: Copy,",
            ],
        ),
        (
            "fn sized(value: [u8; { 1 + 2 }]) {}",
            vec!["fn sized(value: [u8; {1 + 2}])"],
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(expected, print_source_item_signatures(source), "{source}");
    }
}

/// REQ-RUST-069.0 S3: the token-spaced text of a macro expansion prints the same
/// signature as the source text.
#[test]
fn test_expanded_signature_text_equal() {
    let written = "impl private::Sealed for u8 { type Buffer = [MaybeUninit<u8>; Self::MAX_STR_LEN]; fn write(self, buf: &mut Self::Buffer) -> &str { x } }";
    let expanded = "impl private ::Sealed for u8 { type Buffer = [MaybeUninit <u8 >; Self ::MAX_STR_LEN ]; fn write (self , buf : & mut Self ::Buffer ) -> & str { x } }";
    let signatures = print_source_item_signatures(written);
    assert_eq!(
        vec![
            "impl private::Sealed for u8",
            "type Buffer = [MaybeUninit<u8>; Self::MAX_STR_LEN]",
            "fn write(self, buf: &mut Self::Buffer) -> &str"
        ],
        signatures
    );
    assert_eq!(signatures, print_source_item_signatures(expanded));
}

/// REQ-RUST-069.0 V1, V7: the declared visibility text of each item form.
#[test]
fn test_item_visibility_text_read() {
    let source = "pub fn a() {} pub(crate) fn b() {} pub(in crate::m) fn c() {} fn d() {} \
        #[macro_export] macro_rules! e { () => {}; } macro_rules! f { () => {}; } \
        #[macro_export(local_inner_macros)] macro_rules! g { () => {}; } #[inline] pub struct H;";
    let read = SourceFile::parse(source, Edition::Edition2021)
        .syntax_node()
        .descendants()
        .filter(check_syntax_item_node)
        .map(|item| read_item_visibility_text(&item))
        .collect::<Vec<_>>();
    assert_eq!(
        vec![
            "pub",
            "pub(crate)",
            "pub(in crate::m)",
            "private",
            "macro_export",
            "private",
            "macro_export",
            "pub"
        ],
        read
    );
}

/// REQ-RUST-071.0 F3: a field is its tokens without attributes; a variant is its name,
/// without its fields and without `=` and a discriminant.
#[test]
fn test_field_signature_text_printed() {
    let cases = [
        (
            "pub struct P { /// doc\n pub x: T, #[serde(skip)] pub(crate) y: &'static str }",
            vec!["pub struct P", "pub x: T", "pub(crate) y: &'static str"],
        ),
        (
            "enum E { A, B(u8), C { x: u8 } = 3, #[default] D }",
            vec!["enum E", "A", "B", "u8", "C", "x: u8", "D"],
        ),
        (
            "union U { a: u8, b: i8 }",
            vec!["union U", "a: u8", "b: i8"],
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(expected, print_source_item_signatures(source), "{source}");
    }
}
