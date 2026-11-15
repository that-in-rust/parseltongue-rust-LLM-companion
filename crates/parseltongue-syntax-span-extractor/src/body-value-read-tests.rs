#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use ra_ap_syntax::{AstNode, Edition, SourceFile};

/// The text of every path in `source` that is a body value path, in order.
fn read_body_value_paths(source: &str) -> Vec<String> {
    SourceFile::parse(source, Edition::Edition2021)
        .tree()
        .syntax()
        .descendants()
        .filter_map(ast::Path::cast)
        .filter(check_body_value_path)
        .map(|path| path.syntax().text().to_string())
        .collect()
}

/// REQ-RUST-048.0
#[test]
fn test_static_path_value_listed() {
    assert_eq!(
        read_body_value_paths(
            "static S: u8 = 1; mod m { pub const C: u8 = 2; } \
             fn f() -> u8 { let a = S; a + m::C + crate::S.min(3) }"
        ),
        ["S", "m::C", "crate::S"],
        "each whole path expression once; the local a is not listed"
    );
}

/// REQ-RUST-048.0
#[test]
fn test_local_binding_not_listed() {
    assert_eq!(
        read_body_value_paths(
            "struct T; impl T { fn f(&self, p: u8, (q, r): (u8, u8)) -> u8 { \
             let s = p; let t = |u: u8| u + s; \
             for v in 0..q { if let Some(w) = Some(v) { return w + r; } } \
             match s { x if x > 1 => x, _ => { let _ = self; t(s) } } } }"
        ),
        Vec::<String>::new(),
        "parameters, let, closure, for, if-let and match bindings and self"
    );
}

/// REQ-RUST-048.0
#[test]
fn test_binding_scope_ends_listed() {
    assert_eq!(
        read_body_value_paths(
            "fn f(p: u8) -> u8 { fn g() -> u8 { p } let a = b; let b = 1; \
             { let c = 2; } match a { d => 0, _ => d }; \
             if let Some(e) = Some(a) { 0 } else { e }; c }"
        ),
        ["p", "b", "d", "e", "c"],
        "a name is listed where its binding is not in scope: a nested fn, \
         before its let, another match arm, an else branch, after its block"
    );
}

/// REQ-RUST-048.0
#[test]
fn test_callee_path_not_listed() {
    assert_eq!(
        read_body_value_paths(
            "static S: u8 = T::N; struct T; impl T { const N: u8 = 1; fn new() -> T { T } } \
             fn g(a: [u8; S as usize]) { g([0; T::N as usize]); T::new(); let _ = g; }"
        ),
        ["T::N", "T", "S", "T::N", "g"],
        "no callee and no call prefix; a static's initializer and an array length \
         in a signature are listed (REQ-RUST-052.0 G7); a function named as a value \
         is listed for resolution to drop"
    );
}

/// REQ-RUST-052.0 G7: a path expression in the initializer of a `const` or `static`, or in
/// an expression of a type (an array length, an enum discriminant), is listed. An item
/// written in a function sees none of the function's parameters or locals; a `let` in a
/// const's own block still binds. A path in an attribute of the file is in no item.
#[test]
fn test_item_initializer_paths_listed() {
    assert_eq!(
        read_body_value_paths(
            "#![z = Z] const A: u8 = B; static S: [u8; A as usize] = [C; D as usize]; \
             type T = [u8; E]; struct U([u8; F]); enum V { W = G } \
             trait X { const Y: u8 = H; } impl X for u8 { const Y: u8 = I; } \
             fn f(p: u8) { let a = 1; const J: u8 = a + p; \
             const K: u8 = { let b = 2; b }; struct L([u8; p]); }"
        ),
        ["B", "A", "C", "D", "E", "F", "G", "H", "I", "a", "p", "p"],
    );
}

/// REQ-RUST-052.0 G5: the one-name qualifier of a two-segment body value path
/// is requested as a type; `Self`, a longer path and a callee's prefix are not.
#[test]
fn test_value_prefix_path_listed() {
    let source = "trait A { const N: u8; fn g() -> u8; } \
                  impl A for u8 { const N: u8 = 1; fn g() -> u8 { 1 } } \
                  mod m { pub mod n { pub const C: u8 = 2; } } \
                  impl A for u16 { const N: u8 = u8::N; \
                  fn g() -> u8 { let a = u8::N + Self::N + m::n::C; a + u8::g() } }";
    let prefixes = SourceFile::parse(source, Edition::Edition2021)
        .tree()
        .syntax()
        .descendants()
        .filter_map(ast::Path::cast)
        .filter(check_body_value_prefix)
        .map(|path| path.syntax().text().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        prefixes,
        ["u8", "u8"],
        "for `u8::N` in the const initializer (REQ-RUST-052.0 G7) and in the body of g"
    );
}
