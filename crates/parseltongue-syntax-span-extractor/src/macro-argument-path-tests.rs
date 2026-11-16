use super::*;
use ra_ap_syntax::{AstNode, Edition, SourceFile};

fn read_fixture_argument_rows(source: &str) -> Vec<(String, &'static str)> {
    let parsed = SourceFile::parse(source, Edition::Edition2024);
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());
    parsed
        .tree()
        .syntax()
        .descendants()
        .filter_map(ast::MacroCall::cast)
        .flat_map(|call| read_macro_argument_paths(&call))
        .map(|row| {
            assert_eq!(
                &source[row.argument_path_start_offset..row.argument_path_end_offset],
                row.argument_path_syntax_node.syntax().text().to_string()
            );
            (
                source[row.argument_path_start_offset..row.argument_path_end_offset].to_owned(),
                row.argument_path_context_kind,
            )
        })
        .collect()
}

/// REQ-RUST-052.0 G15
#[test]
fn test_macro_argument_paths_read() {
    let value = |text: &str| (text.to_owned(), "body_value_path");
    let prefix = |text: &str| (text.to_owned(), "body_value_type_prefix");
    assert_eq!(
        read_fixture_argument_rows(
            "const _: () = { assert !(i16 ::BITS == u16 ::BITS , \"need counterparts\" ) ; };"
        ),
        [
            value("i16 ::BITS"),
            prefix("i16"),
            value("u16 ::BITS"),
            prefix("u16")
        ]
    );
    assert_eq!(
        read_fixture_argument_rows("fn f() { debug_assert_eq!(A, m::B, \"{}\", C,); }"),
        [value("A"), value("m::B"), prefix("m"), value("C")]
    );
    // A name bound where the call is written, or inside the arguments, is not read.
    assert_eq!(
        read_fixture_argument_rows(
            "fn f(value: u32) { let n = 1; assert!(value < LIMIT && n < { let k = 2; k }); }"
        ),
        [value("LIMIT")]
    );
    // After the compared expressions come the message and its arguments. The name of a
    // named argument is not a path.
    assert_eq!(
        read_fixture_argument_rows("fn f() { assert!(A, \"{n} {}\", B, n = C); }"),
        [value("A"), value("B"), value("C")]
    );
    assert_eq!(
        read_fixture_argument_rows("fn f() { assert_ne!(A, B, \"{n}\", n = C); }"),
        [value("A"), value("B"), value("C")]
    );
    // A callee is a call, not a read.
    assert_eq!(
        read_fixture_argument_rows("fn f() { assert!(g(A)); }"),
        [value("A")]
    );
    [
        // Not an assertion macro, or named by a path.
        "fn f() { m!(A); }",
        "fn f() { core::assert!(A); }",
        // Not in expression position.
        "assert!(A);",
        // Arguments that are not a list of expressions.
        "fn f() { assert!(A =>); }",
        "fn f() { assert!(A; B); }",
        // Fewer arguments than the macro compares.
        "fn f() { assert!(); }",
        "fn f() { debug_assert_eq!(A); }",
        // A path through `$crate`.
        "fn f() { assert!(_crate::A); }",
        // A nested macro call's arguments are not read.
        "fn f() { assert!(matches!(x, A)); }",
    ]
    .iter()
    .for_each(|source| {
        assert_eq!(read_fixture_argument_rows(source), [], "{source}");
    });
}
