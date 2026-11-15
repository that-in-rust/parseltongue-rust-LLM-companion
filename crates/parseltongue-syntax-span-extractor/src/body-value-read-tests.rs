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
        ["T", "T::N", "g"],
        "no callee, no call prefix, no signature or initializer path; \
         a function named as a value is listed for resolution to drop"
    );
}
