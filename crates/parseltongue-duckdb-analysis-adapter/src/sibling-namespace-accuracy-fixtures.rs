use super::*;

#[test]
fn restricted_reexport_does_not_leak_through_path_or_glob() {
    let mut input = build_fixture_resolver_input(&[&[], &["definitions"], &["private"]]);
    input.name_symbol_binding_rows = vec![
        build_fixture_module_binding(&[], "definitions", &["definitions"], 1),
        build_fixture_module_binding(&[], "private", &["private"], 2),
        build_fixture_name_binding(
            &["definitions"],
            "PublicItem",
            NamespaceBindingKindValue::Type,
            3,
        ),
    ];
    let mut import = build_fixture_import_request(
        &["private"],
        &["crate", "definitions", "PublicItem"],
        Some("PrivateAlias"),
        false,
        4,
    );
    import.import_visibility_scope_kind =
        NameBindingVisibilityKind::Restricted(build_fixture_scope_identity(&["private"]));
    input.name_import_request_rows = vec![
        import,
        build_fixture_import_request(&[], &["private"], None, true, 5),
    ];
    input.name_signature_path_rows = vec![
        build_fixture_path_request(
            &[],
            &["private", "PrivateAlias"],
            NameNamespaceFilterKind::Type,
            6,
        ),
        build_fixture_path_request(&[], &["PrivateAlias"], NameNamespaceFilterKind::Type, 7),
        build_fixture_path_request(
            &["private"],
            &["PrivateAlias"],
            NameNamespaceFilterKind::Type,
            8,
        ),
    ];
    let output = run_fixture_name_resolution(&input);
    assert_eq!(
        output
            .resolved_signature_path_rows
            .iter()
            .map(|answer| answer.path_resolution_state_kind)
            .collect::<Vec<_>>(),
        vec![
            NamePathAnswerKind::Unresolved,
            NamePathAnswerKind::Unresolved,
            NamePathAnswerKind::Resolved
        ]
    );
}

#[test]
fn raw_identifier_import_alias_normalized() {
    let mut input = build_fixture_resolver_input(&[&[], &["definitions"]]);
    input.name_symbol_binding_rows = vec![
        build_fixture_module_binding(&[], "definitions", &["definitions"], 1),
        build_fixture_name_binding(
            &["definitions"],
            "r#type",
            NamespaceBindingKindValue::Type,
            2,
        ),
    ];
    input.name_import_request_rows = vec![build_fixture_import_request(
        &[],
        &["definitions", "r#type"],
        Some("r#async"),
        false,
        3,
    )];
    input.name_signature_path_rows = vec![build_fixture_path_request(
        &[],
        &["r#async"],
        NameNamespaceFilterKind::Type,
        4,
    )];
    let output = run_fixture_name_resolution(&input);
    assert_eq!(
        output.resolved_signature_path_rows[0].path_resolution_state_kind,
        NamePathAnswerKind::Resolved
    );
}

#[test]
fn anonymous_trait_import_and_enum_glob() {
    let mut input =
        build_fixture_resolver_input(&[&[], &["definitions"], &["definitions", "Enum"]]);
    input.name_symbol_binding_rows = vec![
        build_fixture_module_binding(&[], "definitions", &["definitions"], 1),
        build_fixture_name_binding(
            &["definitions"],
            "Trait",
            NamespaceBindingKindValue::Type,
            2,
        ),
        build_fixture_module_binding(&["definitions"], "Enum", &["definitions", "Enum"], 3),
        build_fixture_name_binding(
            &["definitions", "Enum"],
            "Variant",
            NamespaceBindingKindValue::Value,
            4,
        ),
    ];
    input.name_import_request_rows = vec![
        build_fixture_import_request(&[], &["definitions", "Trait"], Some("_"), false, 5),
        build_fixture_import_request(&[], &["definitions", "Enum"], None, true, 6),
    ];
    input.name_signature_path_rows = vec![
        build_fixture_path_request(&[], &["Variant"], NameNamespaceFilterKind::Value, 7),
        build_fixture_path_request(&[], &["Trait"], NameNamespaceFilterKind::Type, 8),
    ];
    let output = run_fixture_name_resolution(&input);
    assert_eq!(
        output.resolved_signature_path_rows[0].path_resolution_state_kind,
        NamePathAnswerKind::Resolved
    );
    assert_eq!(
        output.resolved_signature_path_rows[1].path_resolution_state_kind,
        NamePathAnswerKind::Unresolved
    );
    assert!(
        output
            .resolved_symbol_binding_rows
            .iter()
            .all(|binding| binding.binding_resolved_name_text != "_")
    );
}
