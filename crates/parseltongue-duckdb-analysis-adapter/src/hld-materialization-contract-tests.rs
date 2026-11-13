#![allow(clippy::unwrap_used)]

use super::*;
use parseltongue_core_row_types::{
    CompilationUnitRoleKind, CrateCompilationUnitKey, HldDependencyOriginKind, HldEdgeInputRecord,
    HldEdgeResolutionValue, HldEffectiveVisibilityKind, HldEvidenceRowKey, HldGraphEdgeKey,
    HldInputCompletenessKind, HldItemKindValue, HldMaterializationInputRecord,
    HldMaterializationValidationError, HldNodeInputCategory, HldNodeInputRecord,
    HldOutputCompletenessKind, HldOutputEdgeStatus, HldOwnerKindValue, HldRelationKindValue,
    HldUnitOriginKind, HldUnitProvenanceRecord, HldUnresolvedReasonKind, HldViaKindValue,
    NameSymbolTargetKey, RunCancelFlagValue, RunGenerationDigestValue, SemanticOccurrenceKeyValue,
    SpanByteRangeValue, SpanDigestIdValue, validate_hld_materialization_input,
};
use parseltongue_table_store_traits::{HldAnalysisPortError, HldGraphAnalysisPort};
use std::collections::{BTreeMap, BTreeSet};

fn build_hld_fixture_unit(package_name: &str, context_byte: u8) -> CrateCompilationUnitKey {
    CrateCompilationUnitKey {
        unit_cargo_package_identifier: format!(
            "registry+https://example.invalid#{package_name}@1.0.0"
        ),
        unit_cargo_target_name: package_name.to_owned(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [context_byte; 32],
    }
}

fn build_hld_fixture_occurrence(
    unit_identity: &CrateCompilationUnitKey,
    file_path: &str,
    start_offset: u64,
    digest_byte: u8,
) -> SemanticOccurrenceKeyValue {
    SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        unit_identity,
        file_path,
        SpanByteRangeValue::try_from((start_offset, start_offset + 1)).unwrap(),
        SpanDigestIdValue::from([digest_byte; 32]),
        &[],
    )
    .unwrap()
}

#[allow(clippy::too_many_arguments)]
fn build_hld_fixture_node(
    generation: RunGenerationDigestValue,
    target: NameSymbolTargetKey,
    category: HldNodeInputCategory,
    item_kind: Option<HldItemKindValue>,
    owner_kind: Option<HldOwnerKindValue>,
    in_function_body: bool,
    visibility: Option<HldEffectiveVisibilityKind>,
    reachable_outside_crate: bool,
    source_path: Option<&str>,
    qualified_path: &str,
) -> HldNodeInputRecord {
    HldNodeInputRecord {
        run_generation_digest_value: generation,
        node_target_reference: target,
        node_input_category: category,
        node_item_kind_value: item_kind,
        node_owner_kind_value: owner_kind,
        node_function_body_ancestry_flag: in_function_body,
        node_effective_visibility_kind: visibility,
        node_reachable_outside_crate_flag: reachable_outside_crate,
        node_source_relative_path: source_path.map(str::to_owned),
        node_qualified_path_text: qualified_path.to_owned(),
        node_signature_text: None,
    }
}

fn build_hld_fixture_edge(
    generation: RunGenerationDigestValue,
    source: NameSymbolTargetKey,
    target: NameSymbolTargetKey,
    relation: HldRelationKindValue,
    evidence: SemanticOccurrenceKeyValue,
) -> HldEdgeInputRecord {
    HldEdgeInputRecord {
        run_generation_digest_value: generation,
        edge_source_reference: source.clone(),
        edge_relation_kind_value: relation,
        edge_resolution_value: HldEdgeResolutionValue::Extracted {
            target: target.clone(),
        },
        edge_evidence_row_key: HldEvidenceRowKey {
            evidence_site_occurrence_key: evidence,
            evidence_source_occurrence_key: source,
            evidence_relation_kind_value: relation,
            evidence_target_reference_value: Some(target),
        },
        edge_dispatch_metadata_text: None,
    }
}

fn build_hld_fixture_input(
    generation: RunGenerationDigestValue,
    provenance: Vec<HldUnitProvenanceRecord>,
    nodes: Vec<HldNodeInputRecord>,
    edges: Vec<HldEdgeInputRecord>,
) -> HldMaterializationInputRecord {
    HldMaterializationInputRecord {
        run_generation_digest_value: generation,
        stage_attempt_sequence_value: "attempt-a".into(),
        semantic_input_completeness_kind: HldInputCompletenessKind::CompleteTypedInput,
        input_unit_provenance_records: provenance,
        input_node_records: nodes,
        input_edge_records: edges,
    }
}

fn build_hld_fixture_store() -> DuckdbAnalysisStoreAdapter {
    DuckdbAnalysisStoreAdapter {
        _connection: std::sync::Mutex::new(duckdb::Connection::open_in_memory().unwrap()),
    }
}

fn materialize_hld_fixture_graph(
    input: &HldMaterializationInputRecord,
) -> parseltongue_core_row_types::HldMaterializationOutputRecord {
    let store = build_hld_fixture_store();
    let output = store
        .materialize_hld_graph_output(input, &RunCancelFlagValue::default())
        .unwrap();
    eprintln!(
        "HLD_TEST_DUCKDB_MEMORY={}",
        store.read_namespace_memory_usage().unwrap()
    );
    output
}

/// REQ-RUST-030.0 / REQ-RUST-019.1
#[test]
fn test_hld_node_visibility_rules() {
    let generation = RunGenerationDigestValue([7; 32]);
    let unit_identity = build_hld_fixture_unit("visibility-fixture", 3);
    let cases = [
        (
            "public in private parent",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/private.rs",
                    1,
                    1,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::Function),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::Public),
                false,
                Some("src/private.rs"),
                "crate::private::visible",
            ),
            true,
        ),
        (
            "crate restricted",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/module.rs",
                    2,
                    2,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::Function),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::RestrictedNonPrivate),
                false,
                Some("src/module.rs"),
                "crate::module::internal_api",
            ),
            true,
        ),
        (
            "trait inherited member",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/trait.rs",
                    3,
                    3,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::AssociatedFunction),
                Some(HldOwnerKindValue::Trait),
                false,
                Some(HldEffectiveVisibilityKind::Public),
                false,
                Some("src/trait.rs"),
                "crate::HiddenTrait::member",
            ),
            true,
        ),
        (
            "private equivalent",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/module.rs",
                    4,
                    4,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::Function),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::PrivateEquivalent),
                false,
                Some("src/module.rs"),
                "crate::module::private_item",
            ),
            false,
        ),
        (
            "function body item",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/body.rs",
                    5,
                    5,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::Function),
                Some(HldOwnerKindValue::FunctionBody),
                true,
                Some(HldEffectiveVisibilityKind::Public),
                false,
                Some("src/body.rs"),
                "crate::function::local_item",
            ),
            false,
        ),
        (
            "impl container",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/impl.rs",
                    6,
                    6,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::ImplContainer),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::Public),
                false,
                Some("src/impl.rs"),
                "crate::Implementation",
            ),
            false,
        ),
        (
            "degree zero selected",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/unused.rs",
                    7,
                    7,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::Struct),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::Public),
                true,
                Some("src/unused.rs"),
                "crate::UnusedPublicType",
            ),
            true,
        ),
    ];
    let input = build_hld_fixture_input(
        generation,
        vec![HldUnitProvenanceRecord {
            unit_compilation_identity_key: unit_identity,
            unit_origin_kind: HldUnitOriginKind::Workspace,
        }],
        cases.iter().map(|(_, node, _)| node.clone()).collect(),
        Vec::new(),
    );

    let output = materialize_hld_fixture_graph(&input);
    let selected = output
        .selected_node_records
        .iter()
        .map(|node| node.node_target_reference.clone())
        .collect::<BTreeSet<_>>();
    let expected_reachability = cases
        .iter()
        .filter(|(_, _, selected)| *selected)
        .map(|(_, node, _)| {
            (
                node.node_target_reference.clone(),
                node.node_reachable_outside_crate_flag,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let actual_reachability = output
        .selected_node_records
        .iter()
        .map(|node| {
            (
                node.node_target_reference.clone(),
                node.node_reachable_outside_crate_flag,
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(selected, expected_reachability.keys().cloned().collect());
    assert_eq!(actual_reachability, expected_reachability);
}

/// REQ-RUST-030.0
#[test]
fn test_hld_direct_edge_projection() {
    let generation = RunGenerationDigestValue([9; 32]);
    let source_unit = build_hld_fixture_unit("source-fixture", 5);
    let dependency_unit = CrateCompilationUnitKey {
        unit_cargo_package_identifier: "registry+https://github.com/rust-lang/rust#core@0.0.0"
            .into(),
        unit_cargo_target_name: "core".into(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [11; 32],
    };
    let standard_unit = CrateCompilationUnitKey {
        unit_cargo_package_identifier: "registry+https://example.invalid#alloc@1.0.0".into(),
        unit_cargo_target_name: "alloc".into(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [12; 32],
    };
    let source_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &source_unit,
        "src/source.rs",
        1,
        1,
    ));
    let declaration_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &source_unit,
        "src/interface.rs",
        2,
        2,
    ));
    let implementation_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &source_unit,
        "src/implementation.rs",
        3,
        3,
    ));
    let root_key = NameSymbolTargetKey::CrateRoot(source_unit.clone());
    let builtin_key = NameSymbolTargetKey::Builtin("u8".into());
    let external_definition_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &dependency_unit,
        "src/mem/mod.rs",
        4,
        4,
    ));
    let standard_definition_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &standard_unit,
        "src/vec/mod.rs",
        5,
        5,
    ));
    assert_ne!(declaration_key, implementation_key);

    let mut nodes = vec![
        build_hld_fixture_node(
            generation,
            source_key.clone(),
            HldNodeInputCategory::LocalItem,
            Some(HldItemKindValue::Function),
            Some(HldOwnerKindValue::Module),
            false,
            Some(HldEffectiveVisibilityKind::Public),
            false,
            Some("src/source.rs"),
            "crate::source",
        ),
        build_hld_fixture_node(
            generation,
            declaration_key.clone(),
            HldNodeInputCategory::LocalItem,
            Some(HldItemKindValue::AssociatedFunction),
            Some(HldOwnerKindValue::Trait),
            false,
            Some(HldEffectiveVisibilityKind::Public),
            false,
            Some("src/interface.rs"),
            "crate::Interface::format",
        ),
        build_hld_fixture_node(
            generation,
            implementation_key.clone(),
            HldNodeInputCategory::LocalItem,
            Some(HldItemKindValue::AssociatedFunction),
            Some(HldOwnerKindValue::Impl),
            false,
            Some(HldEffectiveVisibilityKind::Public),
            false,
            Some("src/implementation.rs"),
            "crate::ConcreteImplementation::format",
        ),
        build_hld_fixture_node(
            generation,
            root_key.clone(),
            HldNodeInputCategory::CrateRoot,
            None,
            None,
            false,
            None,
            true,
            None,
            "crate",
        ),
        build_hld_fixture_node(
            generation,
            builtin_key.clone(),
            HldNodeInputCategory::Builtin,
            None,
            None,
            false,
            None,
            true,
            None,
            "u8",
        ),
        build_hld_fixture_node(
            generation,
            external_definition_key.clone(),
            HldNodeInputCategory::ExternalDefinition,
            None,
            None,
            false,
            None,
            true,
            Some("src/mem/mod.rs"),
            "core::mem::MaybeUninit::uninit",
        ),
        build_hld_fixture_node(
            generation,
            standard_definition_key.clone(),
            HldNodeInputCategory::ExternalDefinition,
            None,
            None,
            false,
            None,
            true,
            Some("src/vec/mod.rs"),
            "alloc::vec::Vec",
        ),
    ];
    nodes[0].node_signature_text = Some("pub fn source() -> u8".into());
    nodes[1].node_signature_text = Some("fn format(&self)".into());
    nodes[5].node_signature_text = Some("pub unsafe fn uninit() -> MaybeUninit<T>".into());
    let edges = vec![
        build_hld_fixture_edge(
            generation,
            source_key.clone(),
            declaration_key.clone(),
            HldRelationKindValue::Calls,
            build_hld_fixture_occurrence(&source_unit, "src/source.rs", 20, 20),
        ),
        build_hld_fixture_edge(
            generation,
            root_key.clone(),
            source_key.clone(),
            HldRelationKindValue::Contains,
            build_hld_fixture_occurrence(&source_unit, "src/source.rs", 21, 21),
        ),
        build_hld_fixture_edge(
            generation,
            source_key.clone(),
            builtin_key.clone(),
            HldRelationKindValue::UsesType,
            build_hld_fixture_occurrence(&source_unit, "src/source.rs", 22, 22),
        ),
        build_hld_fixture_edge(
            generation,
            source_key.clone(),
            external_definition_key.clone(),
            HldRelationKindValue::Calls,
            build_hld_fixture_occurrence(&source_unit, "src/source.rs", 23, 23),
        ),
        build_hld_fixture_edge(
            generation,
            source_key.clone(),
            standard_definition_key.clone(),
            HldRelationKindValue::Calls,
            build_hld_fixture_occurrence(&source_unit, "src/source.rs", 24, 24),
        ),
    ];
    let input = build_hld_fixture_input(
        generation,
        vec![
            HldUnitProvenanceRecord {
                unit_compilation_identity_key: source_unit.clone(),
                unit_origin_kind: HldUnitOriginKind::Workspace,
            },
            HldUnitProvenanceRecord {
                unit_compilation_identity_key: dependency_unit,
                unit_origin_kind: HldUnitOriginKind::ExternalCrate,
            },
            HldUnitProvenanceRecord {
                unit_compilation_identity_key: standard_unit,
                unit_origin_kind: HldUnitOriginKind::StandardLibrary,
            },
        ],
        nodes,
        edges,
    );
    let output = materialize_hld_fixture_graph(&input);

    let expected_edge_keys = [
        HldGraphEdgeKey {
            run_generation_digest_value: generation,
            edge_source_occurrence_key: source_key.clone(),
            edge_target_reference_value: declaration_key.clone(),
            edge_relation_kind_value: HldRelationKindValue::Calls,
            edge_collapse_via_kind: HldViaKindValue::Direct,
        },
        HldGraphEdgeKey {
            run_generation_digest_value: generation,
            edge_source_occurrence_key: root_key.clone(),
            edge_target_reference_value: source_key.clone(),
            edge_relation_kind_value: HldRelationKindValue::Contains,
            edge_collapse_via_kind: HldViaKindValue::Direct,
        },
        HldGraphEdgeKey {
            run_generation_digest_value: generation,
            edge_source_occurrence_key: source_key.clone(),
            edge_target_reference_value: builtin_key.clone(),
            edge_relation_kind_value: HldRelationKindValue::UsesType,
            edge_collapse_via_kind: HldViaKindValue::Direct,
        },
        HldGraphEdgeKey {
            run_generation_digest_value: generation,
            edge_source_occurrence_key: source_key.clone(),
            edge_target_reference_value: external_definition_key.clone(),
            edge_relation_kind_value: HldRelationKindValue::Calls,
            edge_collapse_via_kind: HldViaKindValue::Direct,
        },
        HldGraphEdgeKey {
            run_generation_digest_value: generation,
            edge_source_occurrence_key: source_key.clone(),
            edge_target_reference_value: standard_definition_key.clone(),
            edge_relation_kind_value: HldRelationKindValue::Calls,
            edge_collapse_via_kind: HldViaKindValue::Direct,
        },
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let actual_edge_keys = output
        .materialized_edge_records
        .iter()
        .map(|edge| edge.edge_key.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(output.materialized_edge_records.len(), 5);
    assert_eq!(actual_edge_keys, expected_edge_keys);
    assert!(output.materialized_edge_records.iter().all(|edge| {
        edge.edge_resolution_status_kind == HldOutputEdgeStatus::Extracted
            && edge.edge_evidence_row_count == 1
    }));
    assert_eq!(output.evidence_path_records.len(), 5);
    assert!(output.evidence_path_records.iter().all(|path| {
        path.ordered_evidence_row_keys.len() == 1 && path.ordered_site_occurrence_keys.len() == 1
    }));
    let path_edge_keys = output
        .evidence_path_records
        .iter()
        .map(|path| path.edge_key.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(path_edge_keys, expected_edge_keys);
    assert_eq!(
        output.output_completeness_kind,
        HldOutputCompletenessKind::DirectProjectionOnly
    );
    assert_eq!(output.file_rollup_records, None);
    assert_eq!(output.folder_rollup_records, None);
    assert!(
        output
            .selected_node_records
            .iter()
            .any(|node| node.node_target_reference == implementation_key)
    );
    assert!(!output.selected_node_records.iter().any(|node| {
        node.node_target_reference == external_definition_key
            || node.node_target_reference == standard_definition_key
            || node.node_target_reference == builtin_key
    }));
    assert!(
        !actual_edge_keys
            .iter()
            .any(|key| { key.edge_target_reference_value == implementation_key })
    );

    let edge_origins = output
        .materialized_edge_records
        .iter()
        .map(|edge| {
            (
                edge.edge_key.edge_target_reference_value.clone(),
                edge.edge_dependency_origin_kind,
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        edge_origins,
        [
            (declaration_key.clone(), HldDependencyOriginKind::SameCrate),
            (source_key.clone(), HldDependencyOriginKind::SameCrate),
            (
                builtin_key.clone(),
                HldDependencyOriginKind::StandardLibrary
            ),
            (
                external_definition_key.clone(),
                HldDependencyOriginKind::ExternalCrate,
            ),
            (
                standard_definition_key.clone(),
                HldDependencyOriginKind::StandardLibrary,
            ),
        ]
        .into_iter()
        .collect()
    );

    let external_call = output
        .materialized_edge_records
        .iter()
        .find(|edge| edge.edge_key.edge_target_reference_value == external_definition_key)
        .unwrap();
    assert_eq!(external_call.edge_source_qualified_path, "crate::source");
    assert_eq!(
        external_call.edge_target_qualified_path,
        "core::mem::MaybeUninit::uninit"
    );
    assert_eq!(
        external_call.edge_source_signature_text.as_deref(),
        Some("pub fn source() -> u8")
    );
    assert_eq!(
        external_call.edge_target_signature_text.as_deref(),
        Some("pub unsafe fn uninit() -> MaybeUninit<T>")
    );
    assert_eq!(
        external_call.edge_dependency_origin_kind,
        HldDependencyOriginKind::ExternalCrate
    );

    let standard_call = output
        .materialized_edge_records
        .iter()
        .find(|edge| edge.edge_key.edge_target_reference_value == standard_definition_key)
        .unwrap();
    assert_eq!(
        standard_call.edge_dependency_origin_kind,
        HldDependencyOriginKind::StandardLibrary
    );
}

/// REQ-RUST-030.0
#[test]
fn test_hld_private_traversal_rejected() {
    let generation = RunGenerationDigestValue([13; 32]);
    let unit_identity = build_hld_fixture_unit("private-path-fixture", 13);
    let source_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &unit_identity,
        "src/source.rs",
        1,
        1,
    ));
    let private_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &unit_identity,
        "src/private.rs",
        2,
        2,
    ));
    let target_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &unit_identity,
        "src/target.rs",
        3,
        3,
    ));
    let nodes = vec![
        build_hld_fixture_node(
            generation,
            source_key.clone(),
            HldNodeInputCategory::LocalItem,
            Some(HldItemKindValue::Function),
            Some(HldOwnerKindValue::Module),
            false,
            Some(HldEffectiveVisibilityKind::Public),
            false,
            Some("src/source.rs"),
            "crate::source",
        ),
        build_hld_fixture_node(
            generation,
            private_key.clone(),
            HldNodeInputCategory::LocalItem,
            Some(HldItemKindValue::Function),
            Some(HldOwnerKindValue::Module),
            false,
            Some(HldEffectiveVisibilityKind::PrivateEquivalent),
            false,
            Some("src/private.rs"),
            "crate::private::helper",
        ),
        build_hld_fixture_node(
            generation,
            target_key.clone(),
            HldNodeInputCategory::LocalItem,
            Some(HldItemKindValue::Function),
            Some(HldOwnerKindValue::Module),
            false,
            Some(HldEffectiveVisibilityKind::Public),
            false,
            Some("src/target.rs"),
            "crate::target",
        ),
    ];
    let input = build_hld_fixture_input(
        generation,
        vec![HldUnitProvenanceRecord {
            unit_compilation_identity_key: unit_identity.clone(),
            unit_origin_kind: HldUnitOriginKind::Workspace,
        }],
        nodes,
        vec![
            build_hld_fixture_edge(
                generation,
                source_key.clone(),
                private_key.clone(),
                HldRelationKindValue::Calls,
                build_hld_fixture_occurrence(&unit_identity, "src/source.rs", 20, 20),
            ),
            build_hld_fixture_edge(
                generation,
                private_key,
                target_key,
                HldRelationKindValue::Calls,
                build_hld_fixture_occurrence(&unit_identity, "src/private.rs", 21, 21),
            ),
        ],
    );

    assert!(matches!(
        build_hld_fixture_store()
            .materialize_hld_graph_output(&input, &RunCancelFlagValue::default(),),
        Err(HldAnalysisPortError::UnsupportedTraversal)
    ));
}

fn build_hld_validation_input() -> (
    HldMaterializationInputRecord,
    CrateCompilationUnitKey,
    NameSymbolTargetKey,
    NameSymbolTargetKey,
) {
    let generation = RunGenerationDigestValue([21; 32]);
    let unit_identity = build_hld_fixture_unit("validation-fixture", 21);
    let root_key = NameSymbolTargetKey::CrateRoot(unit_identity.clone());
    let source_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &unit_identity,
        "src/source.rs",
        1,
        1,
    ));
    let target_key = NameSymbolTargetKey::Builtin("u8".into());
    let input = build_hld_fixture_input(
        generation,
        vec![HldUnitProvenanceRecord {
            unit_compilation_identity_key: unit_identity.clone(),
            unit_origin_kind: HldUnitOriginKind::Workspace,
        }],
        vec![
            build_hld_fixture_node(
                generation,
                root_key,
                HldNodeInputCategory::CrateRoot,
                None,
                None,
                false,
                None,
                false,
                None,
                "crate",
            ),
            build_hld_fixture_node(
                generation,
                source_key.clone(),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::Function),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::Public),
                false,
                Some("src/source.rs"),
                "crate::source",
            ),
            build_hld_fixture_node(
                generation,
                target_key.clone(),
                HldNodeInputCategory::Builtin,
                None,
                None,
                false,
                None,
                false,
                None,
                "u8",
            ),
        ],
        vec![build_hld_fixture_edge(
            generation,
            source_key.clone(),
            target_key.clone(),
            HldRelationKindValue::Calls,
            build_hld_fixture_occurrence(&unit_identity, "src/callsite.rs", 8, 8),
        )],
    );
    (input, unit_identity, source_key, target_key)
}

fn set_hld_builtin_target(input: &mut HldMaterializationInputRecord, name: &str) {
    let target = NameSymbolTargetKey::Builtin(name.into());
    input.input_node_records[2].node_target_reference = target.clone();
    input.input_node_records[2].node_qualified_path_text = name.into();
    input.input_edge_records[0].edge_resolution_value = HldEdgeResolutionValue::Extracted {
        target: target.clone(),
    };
    input.input_edge_records[0]
        .edge_evidence_row_key
        .evidence_target_reference_value = Some(target);
}

fn collect_hld_validation_failures(
    cases: Vec<(
        &str,
        HldMaterializationInputRecord,
        HldMaterializationValidationError,
    )>,
) -> Vec<String> {
    cases
        .into_iter()
        .filter_map(|(case_name, input, expected_error)| {
            let result = validate_hld_materialization_input(&input);
            (result != Err(expected_error)).then(|| format!("{case_name}: {result:?}"))
        })
        .collect()
}

/// REQ-RUST-030.0
#[test]
fn test_hld_identity_validation_rules() {
    let (base_input, unit_identity, _, _) = build_hld_validation_input();
    let supported_builtins = [
        "bool", "char", "str", "usize", "isize", "u8", "u16", "u32", "u64", "u128", "i8", "i16",
        "i32", "i64", "i128", "f16", "f32", "f64", "f128",
    ];
    for builtin_name in supported_builtins {
        let mut input = base_input.clone();
        set_hld_builtin_target(&mut input, builtin_name);
        assert_eq!(validate_hld_materialization_input(&input), Ok(()));
    }

    let mut invalid_cases = Vec::new();
    for builtin_name in ["not_a_primitive", "u8 ", "MaybeUninit", "uninit"] {
        let mut input = base_input.clone();
        set_hld_builtin_target(&mut input, builtin_name);
        invalid_cases.push((
            "invalid builtin identity",
            input,
            HldMaterializationValidationError::MalformedBuiltinIdentity,
        ));
    }
    let mut namespaced_builtin = base_input.clone();
    set_hld_builtin_target(&mut namespaced_builtin, "core::mem::MaybeUninit::uninit");
    invalid_cases.push((
        "ordinary core definition as builtin",
        namespaced_builtin,
        HldMaterializationValidationError::MalformedBuiltinIdentity,
    ));

    let external_same_unit = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &unit_identity,
        "src/external.rs",
        22,
        22,
    ));
    let mut external_masquerade = base_input.clone();
    external_masquerade
        .input_node_records
        .push(build_hld_fixture_node(
            external_masquerade.run_generation_digest_value,
            external_same_unit,
            HldNodeInputCategory::ExternalDefinition,
            None,
            None,
            false,
            None,
            false,
            None,
            "external::same_unit",
        ));
    invalid_cases.push((
        "external definition in local unit",
        external_masquerade,
        HldMaterializationValidationError::InvalidNodeCategory,
    ));

    let mut function_body_owner = base_input.clone();
    function_body_owner.input_node_records[1].node_owner_kind_value =
        Some(HldOwnerKindValue::FunctionBody);
    invalid_cases.push((
        "function body owner without ancestry",
        function_body_owner,
        HldMaterializationValidationError::InvalidNodeCategory,
    ));
    let mut unknown_owner = base_input.clone();
    unknown_owner.input_node_records[1].node_owner_kind_value = Some(HldOwnerKindValue::Other);
    invalid_cases.push((
        "unknown owner kind",
        unknown_owner,
        HldMaterializationValidationError::InvalidNodeCategory,
    ));
    for item_kind in [HldItemKindValue::Field, HldItemKindValue::EnumVariant] {
        let mut unsupported_owner = base_input.clone();
        unsupported_owner.input_node_records[1].node_item_kind_value = Some(item_kind);
        invalid_cases.push((
            "unsupported item owner semantics",
            unsupported_owner,
            HldMaterializationValidationError::InvalidNodeCategory,
        ));
    }
    for (node_index, expected_error) in [
        (0, HldMaterializationValidationError::InvalidNodeCategory),
        (
            2,
            HldMaterializationValidationError::MalformedBuiltinIdentity,
        ),
    ] {
        let mut invalid_ancestry = base_input.clone();
        invalid_ancestry.input_node_records[node_index].node_function_body_ancestry_flag = true;
        invalid_cases.push((
            "nonlocal endpoint has body ancestry",
            invalid_ancestry,
            expected_error,
        ));
    }
    let mut external_unit = build_hld_fixture_unit("other-workspace", 22);
    while external_unit == unit_identity {
        external_unit.unit_configuration_digest_full[0] = 23;
    }
    let mut external_workspace_input = base_input.clone();
    external_workspace_input
        .input_unit_provenance_records
        .push(HldUnitProvenanceRecord {
            unit_compilation_identity_key: external_unit.clone(),
            unit_origin_kind: HldUnitOriginKind::Workspace,
        });
    external_workspace_input
        .input_node_records
        .push(build_hld_fixture_node(
            external_workspace_input.run_generation_digest_value,
            NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                &external_unit,
                "src/dependency.rs",
                23,
                23,
            )),
            HldNodeInputCategory::ExternalDefinition,
            None,
            None,
            false,
            None,
            false,
            None,
            "dependency::external",
        ));
    assert_eq!(
        validate_hld_materialization_input(&external_workspace_input),
        Ok(())
    );
    let mut external_body_ancestry = external_workspace_input.clone();
    external_body_ancestry.input_node_records[3].node_function_body_ancestry_flag = true;
    invalid_cases.push((
        "external definition has body ancestry",
        external_body_ancestry,
        HldMaterializationValidationError::InvalidNodeCategory,
    ));

    let mut nested_module = base_input.clone();
    nested_module.input_edge_records.clear();
    nested_module.input_node_records[1].node_item_kind_value = Some(HldItemKindValue::Module);
    nested_module.input_node_records[1].node_function_body_ancestry_flag = true;
    assert_eq!(validate_hld_materialization_input(&nested_module), Ok(()));
    let nested_output = materialize_hld_fixture_graph(&nested_module);
    assert!(
        !nested_output
            .selected_node_records
            .iter()
            .any(|node| node.node_target_reference
                == nested_module.input_node_records[1].node_target_reference)
    );

    let failures = collect_hld_validation_failures(invalid_cases);
    assert!(failures.is_empty(), "{failures:?}");
}

/// REQ-RUST-030.0
#[test]
fn test_hld_invalid_input_rejection() {
    let (base_input, unit_identity, source_key, target_key) = build_hld_validation_input();
    let mut cases = Vec::new();
    let mut incomplete = base_input.clone();
    incomplete.semantic_input_completeness_kind = HldInputCompletenessKind::IncompleteTypedInput;
    cases.push((
        "incomplete input",
        incomplete,
        HldMaterializationValidationError::IncompleteTypedInput,
    ));

    let mut generation_mismatch = base_input.clone();
    generation_mismatch.input_node_records[1].run_generation_digest_value =
        RunGenerationDigestValue([99; 32]);
    cases.push((
        "mismatched generation",
        generation_mismatch,
        HldMaterializationValidationError::GenerationMismatch,
    ));

    let missing_endpoint = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &unit_identity,
        "src/missing.rs",
        30,
        30,
    ));
    let mut missing_target = base_input.clone();
    missing_target.input_edge_records[0].edge_resolution_value =
        HldEdgeResolutionValue::Extracted {
            target: missing_endpoint.clone(),
        };
    missing_target.input_edge_records[0]
        .edge_evidence_row_key
        .evidence_target_reference_value = Some(missing_endpoint);
    cases.push((
        "missing endpoint",
        missing_target,
        HldMaterializationValidationError::MissingLocalEndpoint,
    ));

    let mut missing_provenance = base_input.clone();
    missing_provenance.input_unit_provenance_records.clear();
    cases.push((
        "missing unit provenance",
        missing_provenance,
        HldMaterializationValidationError::MissingUnitProvenance,
    ));

    let mut invalid_evidence = base_input.clone();
    invalid_evidence.input_edge_records[0]
        .edge_evidence_row_key
        .evidence_relation_kind_value = HldRelationKindValue::Derives;
    cases.push((
        "invalid evidence identity",
        invalid_evidence,
        HldMaterializationValidationError::InvalidEvidenceIdentity,
    ));

    let other_unit = build_hld_fixture_unit("other-evidence-unit", 24);
    let mut cross_unit_site = base_input.clone();
    cross_unit_site
        .input_unit_provenance_records
        .push(HldUnitProvenanceRecord {
            unit_compilation_identity_key: other_unit.clone(),
            unit_origin_kind: HldUnitOriginKind::Workspace,
        });
    cross_unit_site.input_edge_records[0]
        .edge_evidence_row_key
        .evidence_site_occurrence_key =
        build_hld_fixture_occurrence(&other_unit, "src/other.rs", 24, 24);
    cases.push((
        "cross-unit evidence site",
        cross_unit_site,
        HldMaterializationValidationError::InvalidEvidenceIdentity,
    ));

    let failures = collect_hld_validation_failures(cases);
    assert!(failures.is_empty(), "{failures:?}");
    assert_eq!(
        source_key,
        base_input.input_edge_records[0].edge_source_reference
    );

    for (resolution, evidence_target) in [
        (
            HldEdgeResolutionValue::Inferred {
                candidate_target: Some(target_key.clone()),
            },
            Some(target_key.clone()),
        ),
        (
            HldEdgeResolutionValue::Proposed {
                candidate_target: Some(target_key.clone()),
            },
            Some(target_key.clone()),
        ),
        (
            HldEdgeResolutionValue::Unresolved {
                reason: HldUnresolvedReasonKind::TraitSolverNeeded,
            },
            None,
        ),
    ] {
        let mut input = base_input.clone();
        input.input_edge_records[0].edge_resolution_value = resolution.clone();
        input.input_edge_records[0]
            .edge_evidence_row_key
            .evidence_target_reference_value = evidence_target;
        let output = materialize_hld_fixture_graph(&input);
        assert!(output.materialized_edge_records.is_empty());
        assert_eq!(
            input.input_edge_records[0].edge_resolution_value,
            resolution
        );
    }
}

/// REQ-RUST-030.0
#[test]
fn test_hld_duplicate_conflict_rejection() {
    let (base_input, _, _, _) = build_hld_validation_input();
    let mut identical_duplicates = base_input.clone();
    identical_duplicates
        .input_unit_provenance_records
        .push(identical_duplicates.input_unit_provenance_records[0].clone());
    identical_duplicates
        .input_node_records
        .push(identical_duplicates.input_node_records[1].clone());
    identical_duplicates
        .input_edge_records
        .push(identical_duplicates.input_edge_records[0].clone());
    assert_eq!(
        validate_hld_materialization_input(&identical_duplicates),
        Ok(())
    );

    let mut conflicting_unit = base_input.clone();
    conflicting_unit
        .input_unit_provenance_records
        .push(HldUnitProvenanceRecord {
            unit_compilation_identity_key: conflicting_unit.input_unit_provenance_records[0]
                .unit_compilation_identity_key
                .clone(),
            unit_origin_kind: HldUnitOriginKind::ExternalCrate,
        });
    assert_eq!(
        validate_hld_materialization_input(&conflicting_unit),
        Err(HldMaterializationValidationError::ConflictingUnitProvenance)
    );

    let mut conflicting_node = base_input.clone();
    let mut node_conflict = conflicting_node.input_node_records[1].clone();
    node_conflict.node_signature_text = Some("different signature".into());
    conflicting_node.input_node_records.push(node_conflict);
    assert_eq!(
        validate_hld_materialization_input(&conflicting_node),
        Err(HldMaterializationValidationError::ConflictingNodeRecord)
    );

    let mut conflicting_evidence = base_input.clone();
    let mut evidence_conflict = conflicting_evidence.input_edge_records[0].clone();
    evidence_conflict.edge_dispatch_metadata_text = Some("different dispatch".into());
    conflicting_evidence
        .input_edge_records
        .push(evidence_conflict);
    assert_eq!(
        validate_hld_materialization_input(&conflicting_evidence),
        Err(HldMaterializationValidationError::ConflictingEvidenceRecord)
    );

    let mut same_site_distinct_relation = base_input;
    let mut second_relation = same_site_distinct_relation.input_edge_records[0].clone();
    second_relation.edge_relation_kind_value = HldRelationKindValue::UsesType;
    second_relation
        .edge_evidence_row_key
        .evidence_relation_kind_value = HldRelationKindValue::UsesType;
    same_site_distinct_relation
        .input_edge_records
        .push(second_relation);
    assert_eq!(
        validate_hld_materialization_input(&same_site_distinct_relation),
        Ok(())
    );
}
