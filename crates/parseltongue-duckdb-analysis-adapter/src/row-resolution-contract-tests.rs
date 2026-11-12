#![allow(clippy::unwrap_used)]

use super::*;
use parseltongue_core_row_types::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[path = "sibling-namespace-accuracy-fixtures.rs"]
mod sibling_namespace_accuracy_fixtures;

fn build_fixture_unit_identity() -> CrateCompilationUnitKey {
    CrateCompilationUnitKey {
        unit_cargo_package_identifier: "path+file:///fixture#fixture@0.1.0".into(),
        unit_cargo_target_name: "fixture".into(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [1; 32],
    }
}

fn build_fixture_scope_identity(parts: &[&str]) -> NameResolutionScopeKey {
    NameResolutionScopeKey {
        scope_compilation_unit_key: build_fixture_unit_identity(),
        scope_semantic_path_parts: parts.iter().map(|part| (*part).into()).collect(),
    }
}

fn build_fixture_symbol_occurrence(index: usize) -> SemanticOccurrenceKeyValue {
    SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        &build_fixture_unit_identity(),
        "src/lib.rs",
        SpanByteRangeValue::try_from((index as u64, index as u64 + 1)).unwrap(),
        SpanDigestIdValue::from([index as u8; 32]),
        &[],
    )
    .unwrap()
}

fn build_fixture_scope_record(parts: &[&str]) -> NameScopeRecordRow {
    NameScopeRecordRow {
        scope_module_identity_key: build_fixture_scope_identity(parts),
        scope_parent_identity_key: (!parts.is_empty())
            .then(|| build_fixture_scope_identity(&parts[..parts.len() - 1])),
        scope_parent_lookup_flag: false,
        scope_module_boundary_flag: true,
        scope_implicit_prelude_key: None,
    }
}

fn build_fixture_name_binding(
    scope: &[&str],
    name: &str,
    namespace: NamespaceBindingKindValue,
    index: usize,
) -> NameResolvedBindingRow {
    NameResolvedBindingRow {
        binding_lexical_scope_key: build_fixture_scope_identity(scope),
        binding_resolved_name_text: name.into(),
        binding_namespace_kind_value: namespace,
        binding_target_identity_key: NameSymbolTargetKey::Definition(
            build_fixture_symbol_occurrence(index),
        ),
        binding_target_child_scope: None,
        binding_visibility_scope_kind: NameBindingVisibilityKind::Public,
        binding_selection_priority_kind: NameBindingPriorityKind::Declaration,
        binding_import_evidence_keys: Vec::new(),
    }
}

fn build_fixture_module_binding(
    parent: &[&str],
    name: &str,
    child: &[&str],
    index: usize,
) -> NameResolvedBindingRow {
    let mut binding =
        build_fixture_name_binding(parent, name, NamespaceBindingKindValue::Type, index);
    binding.binding_target_child_scope = Some(build_fixture_scope_identity(child));
    binding
}

fn build_fixture_resolver_input(scopes: &[&[&str]]) -> NameResolutionInputRows {
    NameResolutionInputRows {
        unit_compilation_identity_key: build_fixture_unit_identity(),
        unit_rust_edition_kind: RustSourceEditionKind::Edition2024,
        unit_extern_prelude_key: None,
        name_shared_namespace_rows: Vec::new(),
        name_module_scope_rows: scopes
            .iter()
            .map(|parts| build_fixture_scope_record(parts))
            .collect(),
        name_symbol_binding_rows: Vec::new(),
        name_import_request_rows: Vec::new(),
        name_signature_path_rows: Vec::new(),
        name_macro_invocation_rows: Vec::new(),
    }
}

fn build_fixture_path_segments(segments: &[&str]) -> Vec<NamePathSegmentKind> {
    segments
        .iter()
        .map(|segment| match *segment {
            "crate" => NamePathSegmentKind::CrateRoot,
            "self" => NamePathSegmentKind::SelfModule,
            "super" => NamePathSegmentKind::ParentModule,
            "::" => NamePathSegmentKind::GlobalRoot,
            name => NamePathSegmentKind::Name(name.into()),
        })
        .collect()
}

fn build_fixture_path_request(
    scope: &[&str],
    path: &[&str],
    namespace: NameNamespaceFilterKind,
    index: usize,
) -> NamePathRequestRow {
    NamePathRequestRow {
        path_site_occurrence_key: build_fixture_symbol_occurrence(index),
        path_lexical_scope_key: build_fixture_scope_identity(scope),
        path_source_segment_rows: build_fixture_path_segments(path),
        path_requested_namespace_kind: namespace,
        path_resolution_source_kind: NameResolutionPathKind::Signature,
        path_unavailable_reason_kind: None,
    }
}

fn build_fixture_import_request(
    scope: &[&str],
    path: &[&str],
    alias: Option<&str>,
    glob: bool,
    index: usize,
) -> NameImportRequestRow {
    NameImportRequestRow {
        import_site_occurrence_key: build_fixture_symbol_occurrence(index),
        import_lexical_scope_key: build_fixture_scope_identity(scope),
        import_path_segment_rows: build_fixture_path_segments(path),
        import_binding_alias_text: alias.map(str::to_owned),
        import_glob_expansion_flag: glob,
        import_visibility_scope_kind: NameBindingVisibilityKind::Public,
        import_resolution_source_kind: NameResolutionPathKind::Import,
    }
}

struct FixtureEmptyMacroPort;

#[test]
fn test_shared_namespace_rows_remain_shared() {
    use std::sync::Arc;
    let mut dependency = build_fixture_unit_identity();
    dependency.unit_cargo_target_name = "dependency".into();
    let dependency_root = NameResolutionScopeKey {
        scope_compilation_unit_key: dependency.clone(),
        scope_semantic_path_parts: vec![],
    };
    let shared = Arc::new(NameSharedNamespaceRows {
        name_module_scope_rows: vec![NameScopeRecordRow {
            scope_module_identity_key: dependency_root.clone(),
            scope_parent_identity_key: None,
            scope_parent_lookup_flag: false,
            scope_module_boundary_flag: true,
            scope_implicit_prelude_key: None,
        }],
        name_symbol_binding_rows: (0..1000)
            .map(|index| {
                let mut row = build_fixture_name_binding(
                    &[],
                    &format!("Item{index}"),
                    NamespaceBindingKindValue::Type,
                    index + 1,
                );
                row.binding_lexical_scope_key = dependency_root.clone();
                row
            })
            .collect(),
    });
    let mut input = build_fixture_resolver_input(&[&[]]);
    let mut module =
        build_fixture_name_binding(&[], "dependency", NamespaceBindingKindValue::Type, 1001);
    module.binding_target_identity_key = NameSymbolTargetKey::CrateRoot(dependency);
    module.binding_target_child_scope = Some(dependency_root);
    input.name_symbol_binding_rows.push(module);
    input
        .name_signature_path_rows
        .push(build_fixture_path_request(
            &[],
            &["dependency", "Item999"],
            NameNamespaceFilterKind::Type,
            1002,
        ));
    input.name_shared_namespace_rows.push(shared.clone());
    let cloned = input.clone();
    assert!(Arc::ptr_eq(&shared, &cloned.name_shared_namespace_rows[0]));
    drop(cloned);
    let output = resolve_crate_name_bindings(
        &input,
        &FixtureEmptyMacroPort,
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    assert_eq!(
        output.resolved_signature_path_rows[0].path_resolution_state_kind,
        NamePathAnswerKind::Resolved
    );
    assert_eq!(output.resolved_symbol_binding_rows.len(), 1);
    assert_eq!(output.resolved_module_scope_rows.len(), 1);
    let output = resolve_owned_crate_bindings(
        input,
        &FixtureEmptyMacroPort,
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    assert_eq!(output.resolved_symbol_binding_rows.len(), 1);
    assert_eq!(Arc::strong_count(&shared), 1);
}

#[test]
fn test_owned_namespace_empty_and_cancelled() {
    let output = resolve_owned_crate_bindings(
        build_fixture_resolver_input(&[&[]]),
        &FixtureEmptyMacroPort,
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    assert!(output.resolved_symbol_binding_rows.is_empty());
    let cancel = RunCancelFlagValue::default();
    cancel.request_stage_work_cancellation();
    assert!(matches!(
        resolve_owned_crate_bindings(
            build_fixture_resolver_input(&[&[]]),
            &FixtureEmptyMacroPort,
            &cancel
        ),
        Err(NameResolutionStageError::Cancelled)
    ));
}

impl ItemMacroExpansionPort for FixtureEmptyMacroPort {
    fn expand_resolved_macro_items(
        &self,
        _: &NameMacroExpansionQuery<'_>,
    ) -> Result<NameExpansionRoundRows, NameResolutionStageError> {
        Ok(NameExpansionRoundRows::default())
    }
}

#[test]
fn test_grouped_self_import_namespace() {
    let mut input = build_fixture_resolver_input(&[&[]]);
    input.name_symbol_binding_rows = vec![
        build_fixture_name_binding(&[], "Chosen", NamespaceBindingKindValue::Type, 1),
        build_fixture_name_binding(&[], "Chosen", NamespaceBindingKindValue::Value, 1),
    ];
    input.name_import_request_rows = vec![build_fixture_import_request(
        &[],
        &["Chosen", "self"],
        Some("Alias"),
        false,
        2,
    )];
    input.name_signature_path_rows = vec![
        build_fixture_path_request(&[], &["Alias"], NameNamespaceFilterKind::Type, 3),
        build_fixture_path_request(&[], &["Alias"], NameNamespaceFilterKind::Value, 4),
    ];
    let output = resolve_crate_name_bindings(
        &input,
        &FixtureEmptyMacroPort,
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    let type_answer = output
        .resolved_signature_path_rows
        .iter()
        .find(|row| row.path_site_occurrence_key == build_fixture_symbol_occurrence(3))
        .unwrap();
    assert_eq!(
        type_answer.path_resolution_state_kind,
        NamePathAnswerKind::Resolved
    );
    assert_eq!(type_answer.path_resolved_target_rows.len(), 1);
    assert_eq!(
        type_answer.path_resolved_target_rows[0].binding_namespace_kind_value,
        NamespaceBindingKindValue::Type
    );
    let value_answer = output
        .resolved_signature_path_rows
        .iter()
        .find(|row| row.path_site_occurrence_key == build_fixture_symbol_occurrence(4))
        .unwrap();
    assert_eq!(
        value_answer.path_resolution_state_kind,
        NamePathAnswerKind::Unresolved
    );
}

fn run_fixture_name_resolution(input: &NameResolutionInputRows) -> NameResolutionOutputRows {
    resolve_crate_name_bindings(
        input,
        &FixtureEmptyMacroPort,
        &RunCancelFlagValue::default(),
    )
    .unwrap()
}

#[test]
fn test_namespace_alias_bindings() {
    let mut input = build_fixture_resolver_input(&[&[], &["defs"]]);
    input.name_symbol_binding_rows = vec![
        build_fixture_module_binding(&[], "defs", &["defs"], 1),
        build_fixture_name_binding(&["defs"], "Thing", NamespaceBindingKindValue::Type, 2),
        build_fixture_name_binding(&["defs"], "Thing", NamespaceBindingKindValue::Value, 3),
    ];
    input.name_import_request_rows = vec![
        build_fixture_import_request(&[], &["Alias"], Some("Second"), false, 20),
        build_fixture_import_request(&[], &["defs", "Thing"], Some("Alias"), false, 21),
    ];
    input.name_signature_path_rows = vec![
        build_fixture_path_request(&[], &["Second"], NameNamespaceFilterKind::Type, 30),
        build_fixture_path_request(&[], &["Second"], NameNamespaceFilterKind::Value, 31),
    ];
    let output = run_fixture_name_resolution(&input);
    assert_eq!(output.resolved_signature_path_rows.len(), 2);
    for (answer, index) in output.resolved_signature_path_rows.iter().zip([2, 3]) {
        assert_eq!(
            answer.path_resolution_state_kind,
            NamePathAnswerKind::Resolved
        );
        assert_eq!(answer.path_resolved_target_rows.len(), 1);
        assert_eq!(
            answer.path_resolved_target_rows[0].binding_target_identity_key,
            NameSymbolTargetKey::Definition(build_fixture_symbol_occurrence(index))
        );
    }
    input.name_module_scope_rows.reverse();
    input.name_symbol_binding_rows.reverse();
    input.name_import_request_rows.reverse();
    input.name_signature_path_rows.reverse();
    assert_eq!(run_fixture_name_resolution(&input), output);
}

#[test]
fn test_module_resolution_fixture_rules() {
    let mut input = build_fixture_resolver_input(&[&[], &["a"], &["b"], &["c"]]);
    input.name_symbol_binding_rows = vec![
        build_fixture_module_binding(&[], "a", &["a"], 1),
        build_fixture_module_binding(&[], "b", &["b"], 2),
        build_fixture_module_binding(&[], "c", &["c"], 3),
        build_fixture_name_binding(&["a"], "Thing", NamespaceBindingKindValue::Type, 10),
        build_fixture_name_binding(&["c"], "Thing", NamespaceBindingKindValue::Type, 11),
    ];
    let mut hidden =
        build_fixture_name_binding(&["a"], "Hidden", NamespaceBindingKindValue::Type, 12);
    hidden.binding_visibility_scope_kind =
        NameBindingVisibilityKind::Restricted(build_fixture_scope_identity(&["a"]));
    input.name_symbol_binding_rows.push(hidden);
    input.name_import_request_rows = vec![
        build_fixture_import_request(&["a"], &["crate", "b"], None, true, 20),
        build_fixture_import_request(&["b"], &["crate", "a"], None, true, 21),
        build_fixture_import_request(&[], &["a"], None, true, 22),
        build_fixture_import_request(&[], &["c"], None, true, 23),
    ];
    input.name_signature_path_rows = vec![
        build_fixture_path_request(&[], &["Thing"], NameNamespaceFilterKind::Type, 30),
        build_fixture_path_request(&[], &["Hidden"], NameNamespaceFilterKind::Type, 31),
        build_fixture_path_request(&["b"], &["Thing"], NameNamespaceFilterKind::Type, 32),
    ];
    let ambiguous = run_fixture_name_resolution(&input);
    assert_eq!(
        ambiguous.resolved_signature_path_rows[0].path_resolution_state_kind,
        NamePathAnswerKind::Ambiguous
    );
    assert!(
        ambiguous.resolved_signature_path_rows[0]
            .path_resolved_target_rows
            .is_empty()
    );
    assert_eq!(
        ambiguous.resolved_signature_path_rows[0]
            .path_candidate_target_rows
            .len(),
        2
    );
    assert_eq!(
        ambiguous.resolved_signature_path_rows[1].path_unresolved_reason_kind,
        Some(SiteUnresolvedReasonKind::ExternalSourceMissing)
    );
    assert_eq!(
        ambiguous.resolved_signature_path_rows[2].path_resolution_state_kind,
        NamePathAnswerKind::Resolved
    );
    input
        .name_import_request_rows
        .push(build_fixture_import_request(
            &[],
            &["a", "Thing"],
            Some("Thing"),
            false,
            24,
        ));
    let explicit = run_fixture_name_resolution(&input);
    assert_eq!(
        explicit.resolved_signature_path_rows[0].path_resolution_state_kind,
        NamePathAnswerKind::Resolved
    );
    assert_eq!(
        explicit.resolved_signature_path_rows[0].path_resolved_target_rows[0]
            .binding_target_identity_key,
        NameSymbolTargetKey::Definition(build_fixture_symbol_occurrence(10))
    );
}

#[test]
fn test_parent_prelude_boundaries() {
    let mut input = build_fixture_resolver_input(&[&[], &["child"], &["@generic"], &["@prelude"]]);
    input
        .name_module_scope_rows
        .iter_mut()
        .filter(|row| row.scope_module_identity_key.scope_semantic_path_parts != ["@prelude"])
        .for_each(|row| {
            row.scope_implicit_prelude_key = Some(build_fixture_scope_identity(&["@prelude"]))
        });
    let generic = input
        .name_module_scope_rows
        .iter_mut()
        .find(|row| row.scope_module_identity_key.scope_semantic_path_parts == ["@generic"])
        .unwrap();
    generic.scope_parent_lookup_flag = true;
    generic.scope_module_boundary_flag = false;
    input.name_symbol_binding_rows = vec![
        build_fixture_name_binding(&[], "RootType", NamespaceBindingKindValue::Type, 1),
        build_fixture_name_binding(&["@prelude"], "u8", NamespaceBindingKindValue::Type, 2),
    ];
    input.name_signature_path_rows = vec![
        build_fixture_path_request(&["child"], &["RootType"], NameNamespaceFilterKind::Type, 10),
        build_fixture_path_request(
            &["@generic"],
            &["RootType"],
            NameNamespaceFilterKind::Type,
            11,
        ),
        build_fixture_path_request(&["child"], &["u8"], NameNamespaceFilterKind::Type, 12),
        build_fixture_path_request(
            &["child"],
            &["self", "u8"],
            NameNamespaceFilterKind::Type,
            13,
        ),
    ];
    let output = run_fixture_name_resolution(&input);
    assert_eq!(
        output
            .resolved_signature_path_rows
            .iter()
            .map(|row| row.path_resolution_state_kind)
            .collect::<Vec<_>>(),
        vec![
            NamePathAnswerKind::Unresolved,
            NamePathAnswerKind::Resolved,
            NamePathAnswerKind::Resolved,
            NamePathAnswerKind::Unresolved
        ]
    );
}

#[test]
fn test_edition_prelude_resolution() {
    let mut input = build_fixture_resolver_input(&[&[], &["child"], &["@extern"], &["shadow"]]);
    let mut dependency = build_fixture_scope_identity(&[]);
    dependency
        .scope_compilation_unit_key
        .unit_cargo_package_identifier = "path+file:///dependency#dependency@0.1.0".into();
    dependency.scope_compilation_unit_key.unit_cargo_target_name = "dependency".into();
    input.name_module_scope_rows.push(NameScopeRecordRow {
        scope_module_identity_key: dependency.clone(),
        scope_parent_identity_key: None,
        scope_parent_lookup_flag: false,
        scope_module_boundary_flag: true,
        scope_implicit_prelude_key: None,
    });
    let mut dep_binding =
        build_fixture_name_binding(&[], "Thing", NamespaceBindingKindValue::Type, 1);
    dep_binding.binding_lexical_scope_key = dependency.clone();
    let mut alias =
        build_fixture_name_binding(&["@extern"], "chosen", NamespaceBindingKindValue::Type, 2);
    alias.binding_target_child_scope = Some(dependency);
    input.unit_extern_prelude_key = Some(build_fixture_scope_identity(&["@extern"]));
    input.name_symbol_binding_rows = vec![
        dep_binding,
        alias,
        build_fixture_module_binding(&[], "chosen", &["shadow"], 3),
        build_fixture_name_binding(&["shadow"], "Thing", NamespaceBindingKindValue::Type, 4),
    ];
    input.name_signature_path_rows = vec![
        build_fixture_path_request(
            &["child"],
            &["chosen", "Thing"],
            NameNamespaceFilterKind::Type,
            10,
        ),
        build_fixture_path_request(&[], &["chosen", "Thing"], NameNamespaceFilterKind::Type, 11),
        build_fixture_path_request(
            &[],
            &["::", "chosen", "Thing"],
            NameNamespaceFilterKind::Type,
            12,
        ),
    ];
    let output = run_fixture_name_resolution(&input);
    assert_eq!(
        output
            .resolved_signature_path_rows
            .iter()
            .map(|row| row.path_resolved_target_rows[0]
                .binding_target_identity_key
                .clone())
            .collect::<Vec<_>>(),
        vec![1, 4, 1]
            .into_iter()
            .map(|index| NameSymbolTargetKey::Definition(build_fixture_symbol_occurrence(index)))
            .collect::<Vec<_>>()
    );
    input.unit_rust_edition_kind = RustSourceEditionKind::Edition2015;
    input.name_signature_path_rows.truncate(1);
    assert_eq!(
        run_fixture_name_resolution(&input).resolved_signature_path_rows[0]
            .path_resolution_state_kind,
        NamePathAnswerKind::Unresolved
    );
    input.name_signature_path_rows[0].path_resolution_source_kind = NameResolutionPathKind::Import;
    assert_eq!(
        run_fixture_name_resolution(&input).resolved_signature_path_rows[0]
            .path_resolved_target_rows[0]
            .binding_target_identity_key,
        NameSymbolTargetKey::Definition(build_fixture_symbol_occurrence(4))
    );
}

#[test]
fn test_empty_inactive_inputs() {
    assert!(
        run_fixture_name_resolution(&build_fixture_resolver_input(&[]))
            .resolved_symbol_binding_rows
            .is_empty()
    );
    let mut input = build_fixture_resolver_input(&[&[]]);
    let mut path = build_fixture_path_request(&[], &["Gone"], NameNamespaceFilterKind::Type, 1);
    path.path_unavailable_reason_kind = Some(SiteUnresolvedReasonKind::CfgInactive);
    input.name_signature_path_rows.push(path);
    assert_eq!(
        run_fixture_name_resolution(&input).resolved_signature_path_rows[0]
            .path_unresolved_reason_kind,
        Some(SiteUnresolvedReasonKind::CfgInactive)
    );
}

#[test]
fn test_invalid_cancelled_inputs() {
    let mut input = build_fixture_resolver_input(&[&[]]);
    input
        .name_signature_path_rows
        .push(build_fixture_path_request(
            &["absent"],
            &["Thing"],
            NameNamespaceFilterKind::Type,
            1,
        ));
    assert!(matches!(
        resolve_crate_name_bindings(
            &input,
            &FixtureEmptyMacroPort,
            &RunCancelFlagValue::default()
        ),
        Err(NameResolutionStageError::Input(_))
    ));
    let cancel = RunCancelFlagValue::default();
    cancel.request_stage_work_cancellation();
    assert_eq!(
        resolve_crate_name_bindings(
            &build_fixture_resolver_input(&[&[]]),
            &FixtureEmptyMacroPort,
            &cancel
        ),
        Err(NameResolutionStageError::Cancelled)
    );
    let mut cyclic = build_fixture_resolver_input(&[&[]]);
    cyclic.name_module_scope_rows[0].scope_parent_identity_key =
        Some(build_fixture_scope_identity(&[]));
    assert!(matches!(
        resolve_crate_name_bindings(
            &cyclic,
            &FixtureEmptyMacroPort,
            &RunCancelFlagValue::default()
        ),
        Err(NameResolutionStageError::Input(_))
    ));
}

struct FixtureImportMacroPort(AtomicUsize);
impl ItemMacroExpansionPort for FixtureImportMacroPort {
    fn expand_resolved_macro_items(
        &self,
        query: &NameMacroExpansionQuery<'_>,
    ) -> Result<NameExpansionRoundRows, NameResolutionStageError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        assert!(
            query
                .expansion_known_binding_rows
                .iter()
                .any(|row| row.binding_resolved_name_text == "Ready")
        );
        Ok(NameExpansionRoundRows {
            expansion_module_scope_rows: vec![build_fixture_scope_record(&["generated"])],
            expansion_symbol_binding_rows: vec![
                build_fixture_module_binding(&[], "generated", &["generated"], 40),
                build_fixture_name_binding(
                    &["generated"],
                    "Exported",
                    NamespaceBindingKindValue::Type,
                    41,
                ),
            ],
            expansion_import_request_rows: vec![build_fixture_import_request(
                &[],
                &["generated", "Exported"],
                Some("Alias"),
                false,
                42,
            )],
            expansion_signature_path_rows: vec![build_fixture_path_request(
                &[],
                &["Alias"],
                NameNamespaceFilterKind::Type,
                43,
            )],
            expansion_completed_invocation_keys: query
                .expansion_pending_invocation_rows
                .iter()
                .map(|row| row.invocation_site_occurrence_key.clone())
                .collect(),
            expansion_semantic_output_rows: vec![StageTableRowRecord {
                stage_owned_table_name: "macro-expanded-item-rows".into(),
                stage_row_record_key: "generated-item".into(),
                stage_row_payload_text: "fixture-output".into(),
            }],
            ..NameExpansionRoundRows::default()
        })
    }
}

#[test]
fn test_macro_import_fixed_point() {
    let mut input = build_fixture_resolver_input(&[&[], &["defs"]]);
    input.name_symbol_binding_rows = vec![
        build_fixture_module_binding(&[], "defs", &["defs"], 1),
        build_fixture_name_binding(&["defs"], "Ready", NamespaceBindingKindValue::Type, 2),
    ];
    input.name_import_request_rows = vec![build_fixture_import_request(
        &[],
        &["defs", "Ready"],
        None,
        false,
        3,
    )];
    input.name_macro_invocation_rows = vec![NameMacroInvocationRow {
        invocation_site_occurrence_key: build_fixture_symbol_occurrence(4),
        invocation_lexical_scope_key: build_fixture_scope_identity(&[]),
        invocation_macro_path_segments: build_fixture_path_segments(&["make"]),
        invocation_argument_source_text: String::new(),
    }];
    let port = FixtureImportMacroPort(AtomicUsize::new(0));
    let output =
        resolve_crate_name_bindings(&input, &port, &RunCancelFlagValue::default()).unwrap();
    assert_eq!(port.0.load(Ordering::Relaxed), 1);
    assert_eq!(output.resolved_changing_macro_rounds, 1);
    assert_eq!(output.resolved_semantic_output_rows.len(), 1);
    assert_eq!(output.resolved_completed_invocation_keys.len(), 1);
    assert_eq!(
        output.resolved_signature_path_rows[0].path_resolved_target_rows[0]
            .binding_target_identity_key,
        NameSymbolTargetKey::Definition(build_fixture_symbol_occurrence(41))
    );
}

struct FixtureBudgetMacroPort {
    expansion_changing_round_limit: usize,
    expansion_observed_round_count: AtomicUsize,
}
impl ItemMacroExpansionPort for FixtureBudgetMacroPort {
    fn expand_resolved_macro_items(
        &self,
        query: &NameMacroExpansionQuery<'_>,
    ) -> Result<NameExpansionRoundRows, NameResolutionStageError> {
        let round = self
            .expansion_observed_round_count
            .fetch_add(1, Ordering::Relaxed);
        let pending = &query.expansion_pending_invocation_rows[0];
        let next = (round < self.expansion_changing_round_limit).then(|| NameMacroInvocationRow {
            invocation_site_occurrence_key: build_fixture_symbol_occurrence(round + 100),
            invocation_argument_source_text: (round + 1).to_string(),
            ..pending.clone()
        });
        Ok(NameExpansionRoundRows {
            expansion_completed_invocation_keys: vec![
                pending.invocation_site_occurrence_key.clone(),
            ],
            expansion_macro_invocation_rows: next.into_iter().collect(),
            ..NameExpansionRoundRows::default()
        })
    }
}

#[test]
fn test_fixed_point_limit_errors() {
    let mut input = build_fixture_resolver_input(&[&[]]);
    input.name_macro_invocation_rows = vec![NameMacroInvocationRow {
        invocation_site_occurrence_key: build_fixture_symbol_occurrence(1),
        invocation_lexical_scope_key: build_fixture_scope_identity(&[]),
        invocation_macro_path_segments: build_fixture_path_segments(&["again"]),
        invocation_argument_source_text: "0".into(),
    }];
    let port = FixtureBudgetMacroPort {
        expansion_changing_round_limit: FIXED_POINT_LIMIT,
        expansion_observed_round_count: AtomicUsize::new(0),
    };
    assert_eq!(
        resolve_crate_name_bindings(&input, &port, &RunCancelFlagValue::default())
            .unwrap()
            .resolved_changing_macro_rounds,
        8192
    );
    assert_eq!(
        port.expansion_observed_round_count.load(Ordering::Relaxed),
        8193
    );
    let port = FixtureBudgetMacroPort {
        expansion_changing_round_limit: FIXED_POINT_LIMIT + 1,
        expansion_observed_round_count: AtomicUsize::new(0),
    };
    assert!(matches!(
        resolve_crate_name_bindings(&input, &port, &RunCancelFlagValue::default()),
        Err(NameResolutionStageError::Stuck(_))
    ));
    assert_eq!(
        port.expansion_observed_round_count.load(Ordering::Relaxed),
        8193
    );
}
