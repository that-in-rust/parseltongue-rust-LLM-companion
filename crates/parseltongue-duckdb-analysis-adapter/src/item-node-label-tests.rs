use super::*;
use serde_json::json;

fn build_fixture_header_value(self_text: Value, trait_text: Option<&str>) -> Value {
    json!({
        "impl_self_type_resolution": {"type_source_evidence": {"type_source_text": self_text}},
        "impl_trait_type_resolution": trait_text
            .map(|text| json!({"type_source_evidence": {"type_source_text": text}})),
    })
}

fn build_fixture_span_value(kind: &str, name: &str, text: &str) -> Value {
    json!({
        "span_syntax_kind_value": kind,
        "span_declared_name_value": name,
        "span_source_code_text": text,
    })
}

/// REQ-RUST-050.0 L2: the impl header segment is the header's own text.
#[test]
fn test_impl_header_label_segment() {
    assert_eq!(
        build_impl_header_segment(&build_fixture_header_value(
            json!("Buffer"),
            Some("Default")
        )),
        "<Buffer as Default>"
    );
    assert_eq!(
        build_impl_header_segment(&build_fixture_header_value(json!("Buffer"), None)),
        "Buffer"
    );
    assert_eq!(
        build_impl_header_segment(&build_fixture_header_value(
            json!("u8"),
            Some("private ::Sealed")
        )),
        "<u8 as private::Sealed>",
        "the expander writes a space before `::`"
    );
    assert_eq!(
        build_impl_header_segment(&build_fixture_header_value(Value::Null, Some("Default"))),
        "IMPL",
        "no readable self text: the syntax kind, not a guess"
    );
}

/// REQ-RUST-050.0 L6: comments dropped, whitespace joined, `;` dropped, 160 characters.
#[test]
fn test_source_text_label_rule() {
    assert_eq!(
        normalize_label_source_text("impl_Integer!(i8, u8);"),
        "impl_Integer!(i8, u8)"
    );
    assert_eq!(
        normalize_label_source_text("/// doc\n// note\nuse a::{\n    b,\n    c,\n};"),
        "use a::{ b, c, }"
    );
    assert_eq!(
        normalize_label_source_text("#[cfg(x)]\nuse a :: b ;"),
        "#[cfg(x)] use a::b"
    );
    let long = format!("m!({});", "é".repeat(200));
    let cut = normalize_label_source_text(&long);
    assert_eq!(
        cut.chars().count(),
        162,
        "160 characters, a space and the mark"
    );
    assert!(cut.starts_with("m!(é") && cut.ends_with(" …"));
    assert_eq!(
        normalize_label_source_text(&"a".repeat(160))
            .chars()
            .count(),
        160
    );
}

/// REQ-RUST-050.0 L3, L4, L5, L7: the last segment by item kind.
#[test]
fn test_unnamed_const_label_text() {
    let call = build_fixture_span_value("MACRO_CALL", "", "make!(u8);");
    assert_eq!(build_item_name_segment(&call, None), "make!(u8)");
    assert_eq!(
        build_item_name_segment(&build_fixture_span_value("USE", "", "use a::b;"), None),
        "use a::b"
    );
    let unnamed = build_fixture_span_value("CONST", "", "const _: () = ();");
    assert_eq!(build_item_name_segment(&unnamed, None), "const _");
    assert_eq!(
        build_item_name_segment(&unnamed, Some(&call)),
        "const _ #make!(u8)"
    );
    let named = build_fixture_span_value("FN", "r#write", "fn r#write() {}");
    assert_eq!(build_item_name_segment(&named, Some(&call)), "write");
    assert_eq!(
        build_item_name_segment(&build_fixture_span_value("IMPL", "", "impl A {}"), None),
        "IMPL"
    );
}

/// REQ-RUST-050.0 L1: crate target name first, the package id nowhere.
#[test]
fn test_label_crate_name_prefix() {
    let unit = CrateCompilationUnitKey {
        unit_cargo_package_identifier: "path+file:///checkout/itoa#1.0.18".into(),
        unit_cargo_target_name: "itoa".into(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [0; 32],
    };
    let header = build_fixture_header_value(json!("u8"), Some("private ::Sealed"));
    assert_eq!(
        join_node_label_segments(&unit, "src/lib.rs", &[], Some(&header), "write"),
        "itoa::src/lib.rs::<u8 as private::Sealed>::write"
    );
    assert_eq!(
        join_node_label_segments(&unit, "src/lib.rs", &["private".into()], None, "Sealed"),
        "itoa::src/lib.rs::private::Sealed"
    );
}
