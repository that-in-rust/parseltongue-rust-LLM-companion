#![allow(clippy::unwrap_used)]

use super::*;
use parseltongue_core_row_types::{
    CompilationUnitRoleKind, CrateCompilationUnitKey, HldDependencyOriginKind, HldEdgeInputRecord,
    HldEdgeResolutionValue, HldEffectiveVisibilityKind, HldEvidenceRowKey, HldGraphEdgeKey,
    HldInputCompletenessKind, HldItemKindValue, HldMaterializationInputRecord,
    HldNodeInputCategory, HldNodeInputRecord, HldOutputCompletenessKind, HldOutputEdgeStatus,
    HldOwnerKindValue, HldRelationKindValue, HldUnitOriginKind, HldUnitProvenanceRecord,
    HldViaKindValue, NameSymbolTargetKey, RunCancelFlagValue, RunGenerationDigestValue,
    SemanticOccurrenceKeyValue, SpanByteRangeValue, SpanDigestIdValue,
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
