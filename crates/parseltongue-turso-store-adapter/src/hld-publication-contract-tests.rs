#![allow(clippy::unwrap_used)]

use super::*;

#[path = "full-graph-checkpoint-tests.rs"]
mod full_graph_checkpoint_tests;
use parseltongue_core_row_types::{
    CompilationUnitRoleKind, CrateCompilationUnitKey, FileAreaIdentityKey, HldDependencyOriginKind,
    HldEdgeInputRecord, HldEdgeResolutionValue, HldEffectiveVisibilityKind,
    HldEndpointRollupBucket, HldEvidencePathRecord, HldEvidenceRowKey, HldFileRollupRecord,
    HldFolderEndpointBucket, HldFolderRollupRecord, HldGraphEdgeKey, HldInputCompletenessKind,
    HldItemKindValue, HldMaterializationInputRecord, HldMaterializationOutputRecord,
    HldMaterializedEdgeRecord, HldNodeInputCategory, HldNodeInputRecord, HldOutputCompletenessKind,
    HldOutputEdgeStatus, HldOwnerKindValue, HldRelationKindValue, HldSelectedNodeRecord,
    HldSourceCoverageKind, HldTargetIdentityValue, HldUnitOriginKind, HldUnitProvenanceRecord,
    HldViaKindValue, NameSymbolTargetKey, RunGenerationDigestValue, SemanticOccurrenceKeyValue,
    SpanByteRangeValue, SpanDigestIdValue, StageTableRowRecord, StageUnitWriteRequest,
};
use parseltongue_table_store_traits::{
    HldGraphInputStoreError, HldGraphInputStorePort, HldPublicationPortError, StageTableStorePort,
};

const HLD_OUTPUT_TABLE_NAMES: [&str; 5] = [
    "hld-current-interface-nodes",
    "hld-current-dependency-edges",
    "hld-edge-evidence-paths",
    "hld-file-level-rollup",
    "hld-folder-level-rollup",
];

/// REQ-RUST-040.0
#[test]
fn test_hld_symbol_codec_unchanged() {
    let target_identity = HldTargetIdentityValue::from(NameSymbolTargetKey::Builtin("u8".into()));

    assert_eq!(
        encode_target_identity_json(&target_identity),
        serde_json::json!({"kind": "Builtin", "builtin_name": "u8"})
    );
}

/// REQ-RUST-040.0
#[test]
fn test_hld_area_identity_published() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("area-identity-persisted");
    complete_hld_fixture_stages(&store, &scope);
    let frozen = seal_hld_unmanifested_snapshot(&store, &scope);
    let mut output = build_edge_hld_output();
    let unit = match &output.selected_node_records[0].node_target_reference {
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::CrateRoot(unit)) => unit.clone(),
        _ => unreachable!(),
    };
    let file_area = parseltongue_core_row_types::FileAreaIdentityKey::build_file_area_identity(
        &unit,
        "src/lib.rs",
    )
    .unwrap();
    output.selected_node_records.push(HldSelectedNodeRecord {
        run_generation_digest_value: output.run_generation_digest_value,
        node_target_reference: HldTargetIdentityValue::FilePublicArea(file_area),
        node_qualified_path_text: "file:src/lib.rs".into(),
        node_source_relative_path: Some("src/lib.rs".into()),
        node_effective_visibility_kind: None,
        node_reachable_outside_crate_flag: None,
        node_signature_text: None,
    });
    let published = store
        .publish_hld_graph_materialization(&frozen, &scope, &output, &RunCancelFlagValue::default())
        .unwrap();
    let node_rows = store
        .read_stage_table_rows(&published, HLD_OUTPUT_TABLE_NAMES[0])
        .unwrap();
    let area_payload = node_rows
        .iter()
        .map(|row| serde_json::from_str::<serde_json::Value>(&row.stage_row_payload_text).unwrap())
        .find(|payload| payload["node_target_identity"]["kind"] == "FilePublicArea")
        .unwrap();
    assert_eq!(
        area_payload["node_target_identity"]["file_source_relative_path"],
        "src/lib.rs"
    );
    assert_eq!(
        area_payload["node_target_identity"]["unit_compilation_identity"]["cargo_package_identifier"],
        unit.unit_cargo_package_identifier
    );
    let edge_row = store
        .read_stage_table_rows(&published, HLD_OUTPUT_TABLE_NAMES[1])
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let edge_payload: serde_json::Value =
        serde_json::from_str(&edge_row.stage_row_payload_text).unwrap();
    assert_eq!(
        edge_payload["edge_key"]["target_identity"],
        serde_json::json!({"kind":"Builtin", "builtin_name":"u8"})
    );
}

fn build_hld_fixture_store() -> TursoWorkspaceStoreAdapter {
    let store = TursoWorkspaceStoreAdapter::open_turso_workspace_store(":memory:").unwrap();
    store.initialize_workspace_stage_schema().unwrap();
    store
}

fn write_hld_required_stage_config(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    required: &[&str],
) {
    let row = StageTableRowRecord {
        stage_owned_table_name: "workspace-config-state-records".into(),
        stage_row_record_key: "workspace".into(),
        stage_row_payload_text: serde_json::json!({"workspace_required_stage_names": required})
            .to_string(),
    };
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: "crate-graph",
                stage_unit_work_key: "fixture-config",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[row],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
}

fn build_hld_fixture_scope(attempt: &str) -> StageRunScopeValue {
    StageRunScopeValue {
        run_generation_digest_value: "00".repeat(32),
        stage_attempt_sequence_value: attempt.into(),
    }
}

fn write_stage_completion_marker(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    stage: &str,
) {
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: stage,
                stage_unit_work_key: "__complete__",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
}

fn complete_hld_fixture_stages(store: &TursoWorkspaceStoreAdapter, scope: &StageRunScopeValue) {
    for stage in [
        "crate-graph",
        "source-ingest",
        "syntax-spans",
        "syntax-namespace-input-records",
        "name-resolution",
        "signature-type-lowering-stage",
        "item-effective-visibility-stage",
        "semantic-edge-evidence-stage",
    ] {
        write_stage_completion_marker(store, scope, stage);
    }
}

fn seal_hld_fixture_snapshot(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
) -> StageRunScopeValue {
    store
        .publish_analysis_input_snapshot(scope, &RunCancelFlagValue::default())
        .unwrap()
}

fn write_hld_scope_configuration(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    required: &[&str],
    mapping_scope_kind: Option<&str>,
) {
    let mut configuration = serde_json::json!({"workspace_required_stage_names": required});
    if let Some(mapping_scope_kind) = mapping_scope_kind {
        configuration["workspace_mapping_scope_kind"] = serde_json::json!(mapping_scope_kind);
    }
    let row = StageTableRowRecord {
        stage_owned_table_name: "workspace-config-state-records".into(),
        stage_row_record_key: "workspace".into(),
        stage_row_payload_text: configuration.to_string(),
    };
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: "crate-graph",
                stage_unit_work_key: "fixture-config",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[row],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
}

fn complete_hld_source_stages(store: &TursoWorkspaceStoreAdapter, scope: &StageRunScopeValue) {
    for stage in [
        "crate-graph",
        "source-ingest",
        "syntax-spans",
        "syntax-namespace-input-records",
        "name-resolution",
    ] {
        write_stage_completion_marker(store, scope, stage);
    }
}

fn write_hld_unit_scope_provenance(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    unit: &CrateCompilationUnitKey,
) {
    let raw_unit_key = "raw-unit-key";
    write_hld_raw_unit_record(store, scope, raw_unit_key, unit, true, None);
    let unit_scope = StageTableRowRecord {
        stage_owned_table_name: "syntax-namespace-input-scopes".into(),
        stage_row_record_key: "root-scope/raw-unit-key".into(),
        stage_row_payload_text: serde_json::json!({
            "scope_cargo_unit_key": raw_unit_key,
            "scope_module_identity_key": {
                "scope_compilation_unit_key": {
                    "unit_cargo_package_identifier": unit.unit_cargo_package_identifier,
                    "unit_cargo_target_name": unit.unit_cargo_target_name,
                    "unit_cargo_target_kind": unit.unit_cargo_target_kind,
                    "unit_compilation_role_kind": "target",
                    "unit_configuration_digest_full": unit.unit_configuration_digest_full.iter()
                        .map(|byte| format!("{byte:02x}")).collect::<String>()
                },
                "scope_semantic_path_parts": []
            }
        })
        .to_string(),
    };
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: "syntax-namespace-input-records",
                stage_unit_work_key: "fixture-root-scope",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[unit_scope],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
}

fn write_hld_raw_unit_record(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    raw_unit_key: &str,
    unit: &CrateCompilationUnitKey,
    mapped: bool,
    sysroot_flag: Option<serde_json::Value>,
) {
    let mut payload = serde_json::json!({
        "unit_compilation_identity_key": raw_unit_key,
        "unit_cargo_package_identifier": unit.unit_cargo_package_identifier,
        "unit_cargo_target_name": unit.unit_cargo_target_name,
        "unit_cargo_target_kind": unit.unit_cargo_target_kind,
        "unit_compilation_role_kind": "target",
        "unit_mapped_target_flag": mapped
    });
    if let Some(sysroot_flag) = sysroot_flag {
        payload["unit_sysroot_context_flag"] = sysroot_flag;
    }
    let work_key = format!("fixture-unit/{raw_unit_key}");
    let raw_unit = StageTableRowRecord {
        stage_owned_table_name: "crate-dependency-graph-table".into(),
        stage_row_record_key: raw_unit_key.into(),
        stage_row_payload_text: payload.to_string(),
    };
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: "crate-graph",
                stage_unit_work_key: &work_key,
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[raw_unit],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
}

fn build_structural_hld_input(scope: &StageRunScopeValue) -> HldMaterializationInputRecord {
    let generation = RunGenerationDigestValue([0; 32]);
    let unit = CrateCompilationUnitKey {
        unit_cargo_package_identifier: "fixture-package".into(),
        unit_cargo_target_name: "fixture-package".into(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [4; 32],
    };
    let occurrence = SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        &unit,
        "src/lib.rs",
        SpanByteRangeValue::try_from((0, 6)).unwrap(),
        SpanDigestIdValue::from([6; 32]),
        &[],
    )
    .unwrap();
    let area = HldTargetIdentityValue::FilePublicArea(
        FileAreaIdentityKey::build_file_area_identity(&unit, "src/lib.rs").unwrap(),
    );
    let definition =
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(occurrence.clone()));
    HldMaterializationInputRecord {
        run_generation_digest_value: generation,
        stage_attempt_sequence_value: scope.stage_attempt_sequence_value.clone(),
        semantic_input_completeness_kind: HldInputCompletenessKind::CompleteTypedInput,
        source_coverage_kind: HldSourceCoverageKind::OriginalTopLevelContainment,
        input_unit_provenance_records: vec![HldUnitProvenanceRecord {
            unit_compilation_identity_key: unit,
            unit_origin_kind: HldUnitOriginKind::Workspace,
        }],
        input_node_records: vec![
            HldNodeInputRecord {
                run_generation_digest_value: generation,
                node_target_reference: area.clone(),
                node_input_category: HldNodeInputCategory::FilePublicArea,
                node_item_kind_value: Some(HldItemKindValue::FilePublicArea),
                node_owner_kind_value: None,
                node_function_body_ancestry_flag: false,
                node_effective_visibility_kind: None,
                node_reachable_outside_crate_flag: None,
                node_source_relative_path: Some("src/lib.rs".into()),
                node_qualified_path_text: "file:src/lib.rs".into(),
                node_signature_text: None,
            },
            HldNodeInputRecord {
                run_generation_digest_value: generation,
                node_target_reference: definition.clone(),
                node_input_category: HldNodeInputCategory::LocalItem,
                node_item_kind_value: Some(HldItemKindValue::Struct),
                node_owner_kind_value: Some(HldOwnerKindValue::CrateRoot),
                node_function_body_ancestry_flag: false,
                node_effective_visibility_kind: Some(HldEffectiveVisibilityKind::Public),
                node_reachable_outside_crate_flag: None,
                node_source_relative_path: Some("src/lib.rs".into()),
                node_qualified_path_text: "Buffer".into(),
                node_signature_text: None,
            },
        ],
        input_edge_records: vec![HldEdgeInputRecord {
            run_generation_digest_value: generation,
            edge_source_reference: area.clone(),
            edge_relation_kind_value: HldRelationKindValue::Contains,
            edge_resolution_value: HldEdgeResolutionValue::Extracted {
                target: definition.clone(),
            },
            edge_evidence_row_key: HldEvidenceRowKey {
                evidence_site_occurrence_key: occurrence,
                evidence_source_occurrence_key: area,
                evidence_relation_kind_value: HldRelationKindValue::Contains,
                evidence_target_reference_value: Some(definition),
            },
            edge_dispatch_metadata_text: None,
        }],
    }
}

fn build_empty_hld_output() -> HldMaterializationOutputRecord {
    HldMaterializationOutputRecord {
        run_generation_digest_value: RunGenerationDigestValue([0; 32]),
        output_completeness_kind: HldOutputCompletenessKind::CompleteGraph,
        source_coverage_kind: HldSourceCoverageKind::FullSemantic,
        selected_node_records: Vec::new(),
        materialized_edge_records: Vec::new(),
        evidence_path_records: Vec::new(),
        file_rollup_records: Some(Vec::new()),
        folder_rollup_records: Some(Vec::new()),
    }
}

fn build_edge_hld_output() -> HldMaterializationOutputRecord {
    let generation = RunGenerationDigestValue([0; 32]);
    let unit = CrateCompilationUnitKey {
        unit_cargo_package_identifier: "fixture-package".into(),
        unit_cargo_target_name: "fixture-package".into(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [4; 32],
    };
    let source = HldTargetIdentityValue::from(NameSymbolTargetKey::CrateRoot(unit.clone()));
    let target = HldTargetIdentityValue::from(NameSymbolTargetKey::Builtin("u8".into()));
    let occurrence_one = SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        &unit,
        "src/lib.rs",
        SpanByteRangeValue::try_from((0, 1)).unwrap(),
        SpanDigestIdValue::from([1; 32]),
        &[],
    )
    .unwrap();
    let occurrence_two = SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        &unit,
        "src/lib.rs",
        SpanByteRangeValue::try_from((2, 3)).unwrap(),
        SpanDigestIdValue::from([2; 32]),
        &[],
    )
    .unwrap();
    let edge_key = HldGraphEdgeKey {
        run_generation_digest_value: generation,
        edge_source_occurrence_key: source.clone(),
        edge_target_reference_value: target.clone(),
        edge_relation_kind_value: HldRelationKindValue::UsesType,
        edge_collapse_via_kind: HldViaKindValue::Direct,
    };
    let evidence_rows =
        [occurrence_one.clone(), occurrence_two.clone()].map(|site| HldEvidenceRowKey {
            evidence_site_occurrence_key: site,
            evidence_source_occurrence_key: source.clone(),
            evidence_relation_kind_value: HldRelationKindValue::UsesType,
            evidence_target_reference_value: Some(target.clone()),
        });
    let evidence_paths = evidence_rows
        .iter()
        .cloned()
        .map(|evidence| HldEvidencePathRecord {
            edge_key: edge_key.clone(),
            ordered_endpoint_path: vec![source.clone(), target.clone()],
            ordered_site_occurrence_keys: vec![evidence.evidence_site_occurrence_key.clone()],
            ordered_evidence_row_keys: vec![evidence],
        })
        .collect();
    HldMaterializationOutputRecord {
        run_generation_digest_value: generation,
        output_completeness_kind: HldOutputCompletenessKind::CompleteGraph,
        source_coverage_kind: HldSourceCoverageKind::FullSemantic,
        selected_node_records: vec![HldSelectedNodeRecord {
            run_generation_digest_value: generation,
            node_target_reference: source,
            node_qualified_path_text: "crate".into(),
            node_source_relative_path: None,
            node_effective_visibility_kind: None,
            node_reachable_outside_crate_flag: None,
            node_signature_text: None,
        }],
        materialized_edge_records: vec![HldMaterializedEdgeRecord {
            edge_key,
            edge_source_qualified_path: "crate".into(),
            edge_target_qualified_path: "u8".into(),
            edge_source_signature_text: None,
            edge_target_signature_text: None,
            edge_resolution_status_kind: HldOutputEdgeStatus::Extracted,
            edge_evidence_row_count: 2,
            edge_dependency_origin_kind: HldDependencyOriginKind::StandardLibrary,
        }],
        evidence_path_records: evidence_paths,
        file_rollup_records: Some(Vec::new()),
        folder_rollup_records: Some(Vec::new()),
    }
}

fn build_single_path_hld_output() -> HldMaterializationOutputRecord {
    let mut output = build_edge_hld_output();
    output.evidence_path_records.truncate(1);
    output.materialized_edge_records[0].edge_evidence_row_count = 1;
    let unit = CrateCompilationUnitKey {
        unit_cargo_package_identifier: "fixture-package".into(),
        unit_cargo_target_name: "fixture-package".into(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [4; 32],
    };
    let target = NameSymbolTargetKey::Builtin("u8".into());
    output.file_rollup_records = Some(vec![HldFileRollupRecord {
        source_endpoint_bucket: HldEndpointRollupBucket::CrateRoot {
            unit_compilation_identity_key: unit.clone(),
        },
        target_endpoint_bucket: HldEndpointRollupBucket::CanonicalExternalIdentity {
            endpoint_target_reference: target.clone().into(),
        },
        edge_relation_kind_value: HldRelationKindValue::UsesType,
        edge_collapse_via_kind: HldViaKindValue::Direct,
        edge_row_count: 1,
    }]);
    output.folder_rollup_records = Some(vec![HldFolderRollupRecord {
        source_folder_bucket: HldFolderEndpointBucket::CrateRoot {
            unit_compilation_identity_key: unit,
        },
        target_folder_bucket: HldFolderEndpointBucket::CanonicalExternalIdentity {
            endpoint_target_reference: target.into(),
        },
        edge_relation_kind_value: HldRelationKindValue::UsesType,
        edge_collapse_via_kind: HldViaKindValue::Direct,
        edge_row_count: 1,
    }]);
    output
}

fn write_hld_fixture_rows(store: &TursoWorkspaceStoreAdapter, scope: &StageRunScopeValue) {
    let rows = [StageTableRowRecord {
        stage_owned_table_name: "lowered-signature-type-table".into(),
        stage_row_record_key: "fixture/semantic-type".into(),
        stage_row_payload_text: r#"{"kind":"Primitive","name":"u8"}"#.into(),
    }];
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: "signature-type-lowering-stage",
                stage_unit_work_key: "fixture-unit",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &rows,
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
}

fn seal_hld_unmanifested_snapshot(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
) -> StageRunScopeValue {
    store.store_runtime_handle_value.block_on(async {
        let mut connection = store.store_database_handle_value.connect().unwrap();
        let transaction = connection
            .transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent)
            .await
            .unwrap();
        let published = seal_snapshot_publication(&transaction, scope)
            .await
            .unwrap();
        transaction.commit().await.unwrap();
        published
    })
}

fn count_hld_raw_rows(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    table: &str,
) -> usize {
    store.store_runtime_handle_value.block_on(async {
        let connection = store.store_database_handle_value.connect().unwrap();
        let mut rows = connection
            .query(
                format!("SELECT count(*) FROM \"{table}\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=?"),
                turso::params![
                    scope.run_generation_digest_value.clone(),
                    scope.stage_attempt_sequence_value.clone()
                ],
            )
            .await
            .unwrap();
        let count: i64 = rows.next().await.unwrap().unwrap().get(0).unwrap();
        usize::try_from(count).unwrap()
    })
}

fn count_raw_stage_markers(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    stage: &str,
) -> usize {
    store.store_runtime_handle_value.block_on(async {
        let connection = store.store_database_handle_value.connect().unwrap();
        let mut rows = connection
            .query(
                "SELECT count(*) FROM \"stage-run-generation-records\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND stage_row_record_key IN (?,?)",
                turso::params![
                    scope.run_generation_digest_value.clone(),
                    scope.stage_attempt_sequence_value.clone(),
                    format!("__marker__/{stage}/hld/current"),
                    format!("__marker__/{stage}/__complete__")
                ],
            )
            .await
            .unwrap();
        let count: i64 = rows.next().await.unwrap().unwrap().get(0).unwrap();
        usize::try_from(count).unwrap()
    })
}

fn count_generation_snapshot_rows(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
) -> usize {
    store.store_runtime_handle_value.block_on(async {
        let connection = store.store_database_handle_value.connect().unwrap();
        let mut rows = connection
            .query(
                "SELECT count(*) FROM \"frozen-stage-publication-records\" WHERE generation_key=? AND source_attempt_key=?",
                turso::params![
                    scope.run_generation_digest_value.clone(),
                    scope.stage_attempt_sequence_value.clone()
                ],
            )
            .await
            .unwrap();
        let count: i64 = rows.next().await.unwrap().unwrap().get(0).unwrap();
        usize::try_from(count).unwrap()
    })
}

#[test]
/// REQ-RUST-031.0
fn test_hld_missing_input_markers() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("missing-markers");
    write_hld_fixture_rows(&store, &scope);
    for stage in ["crate-graph", "source-ingest", "syntax-spans"] {
        write_stage_completion_marker(&store, &scope, stage);
    }
    let frozen = seal_hld_unmanifested_snapshot(&store, &scope);
    assert_eq!(
        store
            .count_stage_table_rows(&frozen, "lowered-signature-type-table")
            .unwrap(),
        1
    );
    let output = build_empty_hld_output();
    let result = store.publish_hld_graph_materialization(
        &frozen,
        &scope,
        &output,
        &RunCancelFlagValue::default(),
    );
    assert!(matches!(
        result,
        Err(HldPublicationPortError::MissingInputStageMarker { .. })
    ));
    write_stage_completion_marker(&store, &scope, "syntax-namespace-input-records");
    assert_eq!(
        count_raw_stage_markers(&store, &scope, "syntax-namespace-input-records"),
        1
    );
    let late = store.publish_hld_graph_materialization(
        &frozen,
        &scope,
        &output,
        &RunCancelFlagValue::default(),
    );
    assert!(matches!(
        late,
        Err(HldPublicationPortError::MissingInputStageMarker { stage_name })
            if stage_name == "syntax-namespace-input-records"
    ));
}

#[test]
/// REQ-RUST-031.0
fn test_hld_generation_input_isolation() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("isolation");
    complete_hld_fixture_stages(&store, &scope);
    let frozen = seal_hld_fixture_snapshot(&store, &scope);
    let output = build_empty_hld_output();
    let wrong_base = StageRunScopeValue {
        stage_attempt_sequence_value: "other".into(),
        ..scope.clone()
    };
    assert!(matches!(
        store.publish_hld_graph_materialization(
            &frozen,
            &wrong_base,
            &output,
            &RunCancelFlagValue::default()
        ),
        Err(HldPublicationPortError::IncompatibleDestinationScope)
    ));
    let wrong_generation = StageRunScopeValue {
        run_generation_digest_value: "other".into(),
        ..scope.clone()
    };
    assert!(matches!(
        store.publish_hld_graph_materialization(
            &frozen,
            &wrong_generation,
            &output,
            &RunCancelFlagValue::default()
        ),
        Err(HldPublicationPortError::IncompatibleDestinationScope)
    ));
    let partial = HldMaterializationOutputRecord {
        output_completeness_kind: HldOutputCompletenessKind::GraphPathsOnly,
        ..output.clone()
    };
    assert!(matches!(
        store.publish_hld_graph_materialization(
            &frozen,
            &scope,
            &partial,
            &RunCancelFlagValue::default()
        ),
        Err(HldPublicationPortError::IncompleteOutput)
    ));
    let mut foreign = output.clone();
    foreign.run_generation_digest_value = RunGenerationDigestValue([1; 32]);
    assert!(matches!(
        store.publish_hld_graph_materialization(
            &frozen,
            &scope,
            &foreign,
            &RunCancelFlagValue::default()
        ),
        Err(HldPublicationPortError::GenerationMismatch)
    ));
    let mut embedded_generation = build_edge_hld_output();
    embedded_generation.materialized_edge_records[0]
        .edge_key
        .run_generation_digest_value = RunGenerationDigestValue([1; 32]);
    assert!(matches!(
        store.publish_hld_graph_materialization(
            &frozen,
            &scope,
            &embedded_generation,
            &RunCancelFlagValue::default()
        ),
        Err(HldPublicationPortError::GenerationMismatch)
    ));
    let unfrozen_scope = StageRunScopeValue {
        stage_attempt_sequence_value: "unfrozen".into(),
        ..scope.clone()
    };
    complete_hld_fixture_stages(&store, &unfrozen_scope);
    assert!(matches!(
        store.publish_hld_graph_materialization(
            &unfrozen_scope,
            &unfrozen_scope,
            &build_empty_hld_output(),
            &RunCancelFlagValue::default()
        ),
        Err(HldPublicationPortError::UnfrozenSourceSnapshot)
    ));
    store
        .publish_hld_graph_materialization(&frozen, &scope, &output, &RunCancelFlagValue::default())
        .unwrap();
    assert!(matches!(
        store.publish_hld_graph_materialization(
            &frozen,
            &scope,
            &output,
            &RunCancelFlagValue::default()
        ),
        Err(HldPublicationPortError::SealedOutputUnit)
    ));
}

#[test]
/// REQ-RUST-031.0
fn test_hld_cancel_preserves_publication() {
    let store = build_hld_fixture_store();
    let prior_scope = build_hld_fixture_scope("prior");
    complete_hld_fixture_stages(&store, &prior_scope);
    let prior_frozen = seal_hld_fixture_snapshot(&store, &prior_scope);
    let prior_published = store
        .publish_hld_graph_materialization(
            &prior_frozen,
            &prior_scope,
            &build_single_path_hld_output(),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let scope = build_hld_fixture_scope("cancel");
    complete_hld_fixture_stages(&store, &scope);
    let frozen = seal_hld_unmanifested_snapshot(&store, &scope);
    let before = store.read_workspace_publication_scope().unwrap();
    let cancel = RunCancelFlagValue::default();
    let result = store.publish_hld_materialization_checkpointed(
        &frozen,
        &scope,
        &build_edge_hld_output(),
        &cancel,
        |checkpoint| {
            if checkpoint
                == HldPublicationCheckpointValue::OutputTableWritten("hld-current-interface-nodes")
            {
                cancel.request_stage_work_cancellation();
            }
        },
    );
    assert!(matches!(result, Err(HldPublicationPortError::Cancelled)));
    assert_eq!(store.read_workspace_publication_scope().unwrap(), before);
    for table in HLD_OUTPUT_TABLE_NAMES {
        assert_eq!(count_hld_raw_rows(&store, &scope, table), 0);
    }
    assert_eq!(
        count_raw_stage_markers(&store, &scope, "hld-current-materialization-stage"),
        0
    );
    assert_eq!(count_generation_snapshot_rows(&store, &scope), 1);
    for (table, expected_count) in HLD_OUTPUT_TABLE_NAMES.iter().zip([1, 1, 1, 1, 1]) {
        assert_eq!(
            store
                .count_stage_table_rows(&prior_published, table)
                .unwrap(),
            expected_count
        );
    }
}

#[test]
/// REQ-RUST-031.0
fn test_hld_failed_write_rollback() {
    let store = build_hld_fixture_store();
    let prior_scope = build_hld_fixture_scope("prior");
    complete_hld_fixture_stages(&store, &prior_scope);
    let prior_frozen = seal_hld_fixture_snapshot(&store, &prior_scope);
    let prior_published = store
        .publish_hld_graph_materialization(
            &prior_frozen,
            &prior_scope,
            &build_single_path_hld_output(),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let scope = build_hld_fixture_scope("rollback");
    complete_hld_fixture_stages(&store, &scope);
    let frozen = seal_hld_unmanifested_snapshot(&store, &scope);
    let previous_manifest = store.read_workspace_publication_scope().unwrap();
    store.store_runtime_handle_value.block_on(async {
        let connection = store.store_database_handle_value.connect().unwrap();
        connection.execute("CREATE UNIQUE INDEX fail_second_hld_path ON \"hld-edge-evidence-paths\"(run_generation_digest_value, stage_attempt_sequence_value)", ()).await.unwrap();
    });
    let result = store.publish_hld_graph_materialization(
        &frozen,
        &scope,
        &build_edge_hld_output(),
        &RunCancelFlagValue::default(),
    );
    assert!(matches!(
        result,
        Err(HldPublicationPortError::Backend {
            operation: HldPublicationOperationKind::WriteOutputRows,
            ..
        })
    ));
    for table in HLD_OUTPUT_TABLE_NAMES {
        assert_eq!(count_hld_raw_rows(&store, &scope, table), 0);
    }
    assert_eq!(
        count_raw_stage_markers(&store, &scope, "hld-current-materialization-stage"),
        0
    );
    assert_eq!(count_generation_snapshot_rows(&store, &scope), 1);
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        previous_manifest
    );
    for (table, expected_count) in HLD_OUTPUT_TABLE_NAMES.iter().zip([1, 1, 1, 1, 1]) {
        assert_eq!(
            store
                .count_stage_table_rows(&prior_published, table)
                .unwrap(),
            expected_count
        );
    }
}

#[test]
/// REQ-RUST-031.0
fn test_hld_complete_empty_snapshot() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("empty");
    complete_hld_fixture_stages(&store, &scope);
    let frozen = seal_hld_fixture_snapshot(&store, &scope);
    let published = store
        .publish_hld_graph_materialization(
            &frozen,
            &scope,
            &build_empty_hld_output(),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    for table in HLD_OUTPUT_TABLE_NAMES {
        assert!(
            store
                .read_stage_table_rows(&published, table)
                .unwrap()
                .is_empty()
        );
    }
    let markers = store
        .read_stage_table_rows(&published, "stage-run-generation-records")
        .unwrap();
    let global = markers
        .iter()
        .find(|row| {
            row.stage_row_record_key == "__marker__/hld-current-materialization-stage/__complete__"
        })
        .unwrap();
    let metadata: serde_json::Value = serde_json::from_str(&global.stage_row_payload_text).unwrap();
    assert_eq!(
        metadata["input_snapshot_attempt_key"],
        frozen.stage_attempt_sequence_value
    );
    assert_eq!(
        metadata["input_snapshot_base_attempt_key"],
        scope.stage_attempt_sequence_value
    );
    assert_eq!(metadata["input_snapshot_revision"], 1);
    assert_eq!(
        count_raw_stage_markers(&store, &scope, "hld-current-materialization-stage"),
        2
    );
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        Some(published)
    );
}

#[test]
/// REQ-RUST-041.0
fn test_hld_graph_input_roundtrip() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("graph-roundtrip");
    let input = build_structural_hld_input(&scope);
    let unit = input.input_unit_provenance_records[0]
        .unit_compilation_identity_key
        .clone();
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_unit_scope_provenance(&store, &scope, &unit);
    complete_hld_source_stages(&store, &scope);
    let source = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    let frozen = store
        .write_hld_graph_input(&source, &scope, &input, &RunCancelFlagValue::default())
        .unwrap();
    let assembled = store
        .assemble_hld_graph_input(&frozen, &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(assembled, input);
    assert_eq!(store.read_workspace_publication_scope().unwrap(), None);
    let mut output = build_empty_hld_output();
    output.source_coverage_kind = HldSourceCoverageKind::OriginalTopLevelContainment;
    output.selected_node_records = input
        .input_node_records
        .iter()
        .map(|node| HldSelectedNodeRecord {
            run_generation_digest_value: node.run_generation_digest_value,
            node_target_reference: node.node_target_reference.clone(),
            node_qualified_path_text: node.node_qualified_path_text.clone(),
            node_source_relative_path: node.node_source_relative_path.clone(),
            node_effective_visibility_kind: node.node_effective_visibility_kind,
            node_reachable_outside_crate_flag: node.node_reachable_outside_crate_flag,
            node_signature_text: node.node_signature_text.clone(),
        })
        .collect();
    let published = store
        .publish_hld_graph_materialization(&frozen, &scope, &output, &RunCancelFlagValue::default())
        .unwrap();
    let markers = store
        .read_stage_table_rows(&published, "stage-run-generation-records")
        .unwrap();
    let marker = markers
        .iter()
        .find(|row| {
            row.stage_row_record_key == "__marker__/hld-current-materialization-stage/__complete__"
        })
        .unwrap();
    let payload: serde_json::Value = serde_json::from_str(&marker.stage_row_payload_text).unwrap();
    assert_eq!(
        payload["mapping_scope_kind"],
        "original-top-level-containment"
    );
    assert_eq!(
        payload["source_coverage_kind"],
        "original-file-top-level-only"
    );
    assert_eq!(payload["unknown_reachability_diagnostic_count"], 1);
}

#[test]
/// REQ-RUST-041.0
fn test_hld_unfrozen_source_scope() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("unfrozen-source");
    let input = build_structural_hld_input(&scope);
    let unit = input.input_unit_provenance_records[0]
        .unit_compilation_identity_key
        .clone();
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_unit_scope_provenance(&store, &scope, &unit);
    complete_hld_source_stages(&store, &scope);
    assert!(matches!(
        store.write_hld_graph_input(&scope, &scope, &input, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::UnfrozenSourceSnapshot)
    ));
}

#[test]
/// REQ-RUST-041.0
fn test_hld_missing_graph_marker() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("missing-graph-marker");
    let input = build_structural_hld_input(&scope);
    let unit = input.input_unit_provenance_records[0]
        .unit_compilation_identity_key
        .clone();
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_unit_scope_provenance(&store, &scope, &unit);
    complete_hld_source_stages(&store, &scope);
    let frozen = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    assert!(matches!(
        store.assemble_hld_graph_input(&frozen, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::MissingStageMarker { .. })
    ));
}

#[test]
/// REQ-RUST-041.0
fn test_hld_valid_empty_graph_input() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("valid-empty-graph-input");
    let input = build_structural_hld_input(&scope);
    let unit = input.input_unit_provenance_records[0]
        .unit_compilation_identity_key
        .clone();
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_unit_scope_provenance(&store, &scope, &unit);
    complete_hld_source_stages(&store, &scope);
    let source = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    let mut empty_input = input;
    empty_input.input_node_records.clear();
    empty_input.input_edge_records.clear();
    let frozen = store
        .write_hld_graph_input(
            &source,
            &scope,
            &empty_input,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let assembled = store
        .assemble_hld_graph_input(&frozen, &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(
        assembled.source_coverage_kind,
        HldSourceCoverageKind::OriginalTopLevelContainment
    );
    assert!(assembled.input_node_records.is_empty());
    assert!(assembled.input_edge_records.is_empty());
}

#[test]
/// REQ-RUST-041.0
fn test_hld_full_semantic_scope_marker() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("full-semantic-scope-marker");
    write_hld_scope_configuration(&store, &scope, &[], Some("FullSemantic"));
    complete_hld_fixture_stages(&store, &scope);
    let frozen = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    let published = store
        .publish_hld_graph_materialization(
            &frozen,
            &scope,
            &build_empty_hld_output(),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let markers = store
        .read_stage_table_rows(&published, "stage-run-generation-records")
        .unwrap();
    for owner in [
        "item-effective-visibility-stage",
        "semantic-edge-evidence-stage",
    ] {
        let marker = markers
            .iter()
            .find(|row| row.stage_row_record_key == format!("__marker__/{owner}/__complete__"))
            .unwrap();
        let payload: serde_json::Value =
            serde_json::from_str(&marker.stage_row_payload_text).unwrap();
        assert_eq!(payload["mapping_scope_kind"], "FullSemantic");
        assert_eq!(payload["source_coverage_kind"], "FullSemantic");
    }
}

#[test]
/// REQ-RUST-041.0
fn test_hld_conflicting_graph_row_key() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("conflicting-graph-row-key");
    let input = build_structural_hld_input(&scope);
    let unit = input.input_unit_provenance_records[0]
        .unit_compilation_identity_key
        .clone();
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_unit_scope_provenance(&store, &scope, &unit);
    complete_hld_source_stages(&store, &scope);
    let source = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    let mut node_rows = super::super::hld_graph_input_turso::encode_graph_node_rows(
        &input,
        HldSourceCoverageKind::OriginalTopLevelContainment,
    )
    .unwrap();
    let Some(first_node_row) = node_rows.first_mut() else {
        panic!("structural fixture has no nodes");
    };
    first_node_row.stage_row_record_key.push_str("/conflict");
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: &scope,
                stage_owner_name_value: "item-effective-visibility-stage",
                stage_unit_work_key: "conflicting-fixture-node",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &node_rows,
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    write_stage_completion_marker(&store, &scope, "item-effective-visibility-stage");
    write_stage_completion_marker(&store, &scope, "semantic-edge-evidence-stage");
    super::super::hld_graph_input_turso::write_graph_source_marker_metadata(
        &store,
        &scope,
        &source,
        &scope,
        1,
        HldInputCompletenessKind::CompleteTypedInput,
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    let frozen = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    assert!(matches!(
        store.assemble_hld_graph_input(&frozen, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::MalformedPersistedRow {
            table,
            ..
        }) if table == "item-effective-visibility-records"
    ));
}

#[test]
/// REQ-RUST-041.0
fn test_hld_frozen_scope_mismatch() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("scope-mismatch");
    let input = build_structural_hld_input(&scope);
    let unit = input.input_unit_provenance_records[0]
        .unit_compilation_identity_key
        .clone();
    write_hld_scope_configuration(&store, &scope, &[], None);
    write_hld_unit_scope_provenance(&store, &scope, &unit);
    complete_hld_source_stages(&store, &scope);
    let frozen = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    assert!(matches!(
        store.write_hld_graph_input(&frozen, &scope, &input, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::CoverageScopeMismatch)
    ));
}

#[test]
/// REQ-RUST-041.0
fn test_hld_missing_root_unit_scope() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("missing-root-unit-scope");
    let input = build_structural_hld_input(&scope);
    let unit = &input.input_unit_provenance_records[0].unit_compilation_identity_key;
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_raw_unit_record(&store, &scope, "mapped-without-root", unit, true, None);
    complete_hld_source_stages(&store, &scope);
    let source = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    assert!(matches!(
        store.write_hld_graph_input(&source, &scope, &input, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::MissingRootUnitScope { raw_unit_key })
            if raw_unit_key == "mapped-without-root"
    ));
}

#[test]
/// REQ-RUST-041.0
fn test_hld_malformed_sysroot_flag() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("malformed-sysroot-flag");
    let input = build_structural_hld_input(&scope);
    let unit = &input.input_unit_provenance_records[0].unit_compilation_identity_key;
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_unit_scope_provenance(&store, &scope, unit);
    write_hld_raw_unit_record(
        &store,
        &scope,
        "raw-invalid-sysroot",
        unit,
        false,
        Some(serde_json::json!("false")),
    );
    complete_hld_source_stages(&store, &scope);
    let source = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    assert!(matches!(
        store.write_hld_graph_input(&source, &scope, &input, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::MalformedPersistedRow {
            table,
            record_key,
            ..
        }) if table == "crate-dependency-graph-table" && record_key == "raw-invalid-sysroot"
    ));
}

#[test]
/// REQ-RUST-041.0
fn test_hld_malformed_scope_config() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("malformed-scope-config");
    let row = StageTableRowRecord {
        stage_owned_table_name: "workspace-config-state-records".into(),
        stage_row_record_key: "workspace".into(),
        stage_row_payload_text: serde_json::json!({
            "workspace_mapping_scope_kind": false,
            "workspace_required_stage_names": []
        })
        .to_string(),
    };
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: &scope,
                stage_owner_name_value: "crate-graph",
                stage_unit_work_key: "fixture-config",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[row],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    complete_hld_source_stages(&store, &scope);
    let source = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    assert!(matches!(
        store.publish_hld_graph_materialization(
            &source,
            &scope,
            &build_empty_hld_output(),
            &RunCancelFlagValue::default()
        ),
        Err(HldPublicationPortError::InvalidFrozenWorkspaceConfig)
    ));
}

#[test]
/// REQ-RUST-041.0
fn test_hld_stale_graph_marker() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("stale-graph-marker");
    let input = build_structural_hld_input(&scope);
    let unit = input.input_unit_provenance_records[0]
        .unit_compilation_identity_key
        .clone();
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_unit_scope_provenance(&store, &scope, &unit);
    complete_hld_source_stages(&store, &scope);
    let frozen = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    write_stage_completion_marker(&store, &scope, "item-effective-visibility-stage");
    assert!(matches!(
        store.assemble_hld_graph_input(&frozen, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::StaleStageMarker { .. })
    ));
    let mut output = build_empty_hld_output();
    output.source_coverage_kind = HldSourceCoverageKind::OriginalTopLevelContainment;
    assert!(matches!(
        store.publish_hld_graph_materialization(
            &frozen,
            &scope,
            &output,
            &RunCancelFlagValue::default()
        ),
        Err(HldPublicationPortError::StaleInputSnapshot { stage_name })
            if stage_name == "item-effective-visibility-stage"
    ));
}

#[test]
/// REQ-RUST-041.0
fn test_hld_source_race_rejected() {
    let store = build_hld_fixture_store();
    let prior_scope = build_hld_fixture_scope("source-race-prior");
    complete_hld_fixture_stages(&store, &prior_scope);
    let prior_source = seal_hld_fixture_snapshot(&store, &prior_scope);
    let prior_published = store
        .publish_hld_graph_materialization(
            &prior_source,
            &prior_scope,
            &build_empty_hld_output(),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let prior_manifest = store.read_workspace_publication_scope().unwrap();
    let prior_counts = [
        "hld-current-interface-nodes",
        "hld-current-dependency-edges",
        "hld-edge-evidence-paths",
        "hld-file-level-rollup",
        "hld-folder-level-rollup",
    ]
    .map(|table| count_hld_raw_rows(&store, &prior_published, table));

    let scope = build_hld_fixture_scope("source-race-after-derivation");
    let input = build_structural_hld_input(&scope);
    let unit = input.input_unit_provenance_records[0]
        .unit_compilation_identity_key
        .clone();
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_unit_scope_provenance(&store, &scope, &unit);
    complete_hld_source_stages(&store, &scope);
    let source = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    let node_rows = super::super::hld_graph_input_turso::encode_graph_node_rows(
        &input,
        HldSourceCoverageKind::OriginalTopLevelContainment,
    )
    .unwrap();
    let edge_rows = super::super::hld_graph_input_turso::encode_graph_edge_rows(
        &input,
        HldSourceCoverageKind::OriginalTopLevelContainment,
    )
    .unwrap();
    let source_stages = [
        "crate-graph",
        "source-ingest",
        "syntax-spans",
        "syntax-namespace-input-records",
        "name-resolution",
    ];
    for (owner, rows) in [
        ("item-effective-visibility-stage", node_rows.as_slice()),
        ("semantic-edge-evidence-stage", edge_rows.as_slice()),
    ] {
        store
            .write_stage_table_unit(
                &StageUnitWriteRequest {
                    stage_run_scope_value: &scope,
                    stage_owner_name_value: owner,
                    stage_unit_work_key: "hld-graph-input",
                    stage_input_stage_names: &source_stages,
                    stage_unit_output_rows: rows,
                },
                &RunCancelFlagValue::default(),
            )
            .unwrap();
        store
            .write_stage_table_unit(
                &StageUnitWriteRequest {
                    stage_run_scope_value: &scope,
                    stage_owner_name_value: owner,
                    stage_unit_work_key: "__complete__",
                    stage_input_stage_names: &source_stages,
                    stage_unit_output_rows: &[],
                },
                &RunCancelFlagValue::default(),
            )
            .unwrap();
    }
    super::super::hld_graph_input_turso::write_graph_source_marker_metadata(
        &store,
        &scope,
        &source,
        &scope,
        1,
        HldInputCompletenessKind::CompleteTypedInput,
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: &scope,
                stage_owner_name_value: "syntax-spans",
                stage_unit_work_key: "late-source-work",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    assert!(matches!(
        super::super::hld_graph_input_turso::seal_graph_analysis_input_snapshot(
            &store,
            &source,
            &scope,
            1,
            HldSourceCoverageKind::OriginalTopLevelContainment,
            &RunCancelFlagValue::default(),
        ),
        Err(HldGraphInputStoreError::SourceSnapshotValidation(
            HldPublicationPortError::StaleInputSnapshot { stage_name }
        )) if stage_name == "syntax-spans"
    ));
    let contaminated_snapshot = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    assert!(matches!(
        store.assemble_hld_graph_input(&contaminated_snapshot, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::StaleStageMarker { stage_name })
            if stage_name == "syntax-spans"
    ));
    let mut output = build_empty_hld_output();
    output.source_coverage_kind = HldSourceCoverageKind::OriginalTopLevelContainment;
    assert!(matches!(
        store.publish_hld_graph_materialization(
            &contaminated_snapshot,
            &scope,
            &output,
            &RunCancelFlagValue::default(),
        ),
        Err(HldPublicationPortError::StaleInputSnapshot { stage_name })
            if stage_name == "syntax-spans"
    ));
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        prior_manifest
    );
    let after_counts = [
        "hld-current-interface-nodes",
        "hld-current-dependency-edges",
        "hld-edge-evidence-paths",
        "hld-file-level-rollup",
        "hld-folder-level-rollup",
    ]
    .map(|table| count_hld_raw_rows(&store, &prior_published, table));
    assert_eq!(after_counts, prior_counts);
}

#[test]
/// REQ-RUST-041.0
fn test_hld_source_marker_mismatch() {
    let node = Some((
        "generation".into(),
        "attempt".into(),
        "generation".into(),
        "base".into(),
        1,
    ));
    let edge = Some((
        "generation".into(),
        "attempt".into(),
        "generation".into(),
        "base".into(),
        9,
    ));
    assert!(matches!(
        graph_source_marker_pair(
            node,
            edge,
            HldSourceCoverageKind::OriginalTopLevelContainment,
        ),
        Err(HldPublicationPortError::InvalidGraphSourceProvenance)
    ));
    assert!(matches!(
        graph_source_marker_pair(
            None,
            None,
            HldSourceCoverageKind::OriginalTopLevelContainment,
        ),
        Err(HldPublicationPortError::InvalidGraphSourceProvenance)
    ));
    assert_eq!(
        graph_source_marker_pair(None, None, HldSourceCoverageKind::FullSemantic).unwrap(),
        None
    );
}

#[test]
/// REQ-RUST-041.0
fn test_hld_source_marker_missing() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("source-marker-missing");
    let input = build_structural_hld_input(&scope);
    let unit = input.input_unit_provenance_records[0]
        .unit_compilation_identity_key
        .clone();
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_unit_scope_provenance(&store, &scope, &unit);
    complete_hld_source_stages(&store, &scope);
    let source = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    for owner in [
        "item-effective-visibility-stage",
        "semantic-edge-evidence-stage",
    ] {
        write_stage_completion_marker(&store, &scope, owner);
    }
    let graph_snapshot = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    let mut output = build_empty_hld_output();
    output.source_coverage_kind = HldSourceCoverageKind::OriginalTopLevelContainment;
    assert!(matches!(
        store.publish_hld_graph_materialization(
            &graph_snapshot,
            &scope,
            &output,
            &RunCancelFlagValue::default(),
        ),
        Err(HldPublicationPortError::InvalidGraphSourceProvenance)
    ));
    assert_eq!(store.read_workspace_publication_scope().unwrap(), None);
    assert_eq!(
        source.run_generation_digest_value,
        scope.run_generation_digest_value
    );
}

#[test]
/// REQ-RUST-041.0
fn test_hld_assembly_cancel_checkpoints() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("assembly-cancel-checkpoints");
    let input = build_structural_hld_input(&scope);
    let unit = input.input_unit_provenance_records[0]
        .unit_compilation_identity_key
        .clone();
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_unit_scope_provenance(&store, &scope, &unit);
    complete_hld_source_stages(&store, &scope);
    let source = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    let frozen = store
        .write_hld_graph_input(&source, &scope, &input, &RunCancelFlagValue::default())
        .unwrap();

    let cancel_after_node_read = RunCancelFlagValue::default();
    assert!(matches!(
        super::super::hld_graph_input_turso::assemble_graph_input_checkpoint(
            &store,
            &frozen,
            &cancel_after_node_read,
            |checkpoint| {
                if checkpoint
                    == super::super::hld_graph_input_turso::GraphAssemblyCheckpointKind::NodeRowsRead
                {
                    cancel_after_node_read.request_stage_work_cancellation();
                }
            },
        ),
        Err(HldGraphInputStoreError::Cancelled)
    ));

    let cancel_after_revalidation = RunCancelFlagValue::default();
    assert!(matches!(
        super::super::hld_graph_input_turso::assemble_graph_input_checkpoint(
            &store,
            &frozen,
            &cancel_after_revalidation,
            |checkpoint| {
                if checkpoint
                    == super::super::hld_graph_input_turso::GraphAssemblyCheckpointKind::SourceRevalidated
                {
                    cancel_after_revalidation.request_stage_work_cancellation();
                }
            },
        ),
        Err(HldGraphInputStoreError::Cancelled)
    ));
}

#[test]
/// REQ-RUST-041.0
fn test_hld_unknown_reachability_scope() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("unknown-structural-reachability");
    let input = build_structural_hld_input(&scope);
    let unit = input.input_unit_provenance_records[0]
        .unit_compilation_identity_key
        .clone();
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_unit_scope_provenance(&store, &scope, &unit);
    complete_hld_source_stages(&store, &scope);
    let source = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    let frozen = store
        .write_hld_graph_input(&source, &scope, &input, &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(
        store
            .assemble_hld_graph_input(&frozen, &RunCancelFlagValue::default())
            .unwrap(),
        input
    );
    let full_scope = build_hld_fixture_scope("unknown-full-reachability");
    let mut full_input = build_structural_hld_input(&full_scope);
    full_input.source_coverage_kind = HldSourceCoverageKind::FullSemantic;
    write_hld_scope_configuration(&store, &full_scope, &[], None);
    write_hld_unit_scope_provenance(
        &store,
        &full_scope,
        &full_input.input_unit_provenance_records[0].unit_compilation_identity_key,
    );
    complete_hld_source_stages(&store, &full_scope);
    let full_source = store
        .publish_analysis_input_snapshot(&full_scope, &RunCancelFlagValue::default())
        .unwrap();
    assert!(matches!(
        store.write_hld_graph_input(
            &full_source,
            &full_scope,
            &full_input,
            &RunCancelFlagValue::default()
        ),
        Err(HldGraphInputStoreError::InvalidTypedInput(_))
    ));
}

#[test]
/// REQ-RUST-041.0
fn test_hld_structural_publication_rollback() {
    let store = build_hld_fixture_store();
    let prior_scope = build_hld_fixture_scope("structural-prior");
    complete_hld_fixture_stages(&store, &prior_scope);
    let prior_snapshot = seal_hld_fixture_snapshot(&store, &prior_scope);
    let prior_published = store
        .publish_hld_graph_materialization(
            &prior_snapshot,
            &prior_scope,
            &build_single_path_hld_output(),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let prior_counts = HLD_OUTPUT_TABLE_NAMES.map(|table| {
        store
            .count_stage_table_rows(&prior_published, table)
            .unwrap()
    });
    let scope = build_hld_fixture_scope("structural-cancel");
    let input = build_structural_hld_input(&scope);
    let unit = input.input_unit_provenance_records[0]
        .unit_compilation_identity_key
        .clone();
    write_hld_scope_configuration(&store, &scope, &[], Some("original-top-level-containment"));
    write_hld_unit_scope_provenance(&store, &scope, &unit);
    complete_hld_source_stages(&store, &scope);
    let source = store
        .publish_analysis_input_snapshot(&scope, &RunCancelFlagValue::default())
        .unwrap();
    let frozen = store
        .write_hld_graph_input(&source, &scope, &input, &RunCancelFlagValue::default())
        .unwrap();
    let mut output = build_empty_hld_output();
    output.source_coverage_kind = HldSourceCoverageKind::OriginalTopLevelContainment;
    let before = store.read_workspace_publication_scope().unwrap();
    let cancel = RunCancelFlagValue::default();
    let result = store.publish_hld_materialization_checkpointed(
        &frozen,
        &scope,
        &output,
        &cancel,
        |checkpoint| {
            if checkpoint
                == HldPublicationCheckpointValue::OutputTableWritten("hld-current-interface-nodes")
            {
                cancel.request_stage_work_cancellation();
            }
        },
    );
    assert!(matches!(result, Err(HldPublicationPortError::Cancelled)));
    assert_eq!(store.read_workspace_publication_scope().unwrap(), before);
    for table in HLD_OUTPUT_TABLE_NAMES {
        assert_eq!(count_hld_raw_rows(&store, &scope, table), 0);
    }
    assert_eq!(
        HLD_OUTPUT_TABLE_NAMES.map(|table| store
            .count_stage_table_rows(&prior_published, table)
            .unwrap()),
        prior_counts
    );
}

#[test]
/// REQ-RUST-031.0
fn test_hld_configured_stage_gate() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("configured-gate");
    complete_hld_fixture_stages(&store, &scope);
    write_hld_required_stage_config(&store, &scope, &["hld-current-materialization-stage"]);
    let frozen = seal_hld_fixture_snapshot(&store, &scope);
    assert!(
        store
            .publish_hld_graph_materialization(
                &frozen,
                &scope,
                &build_empty_hld_output(),
                &RunCancelFlagValue::default(),
            )
            .is_ok()
    );
}

#[test]
/// REQ-RUST-031.0
fn test_hld_missing_extra_required() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("missing-extra");
    complete_hld_fixture_stages(&store, &scope);
    write_hld_required_stage_config(&store, &scope, &["extra-required-stage"]);
    let frozen = seal_hld_fixture_snapshot(&store, &scope);
    assert!(
        store
            .publish_hld_graph_materialization(
                &frozen,
                &scope,
                &build_empty_hld_output(),
                &RunCancelFlagValue::default(),
            )
            .is_err()
    );
    for table in HLD_OUTPUT_TABLE_NAMES {
        assert_eq!(count_hld_raw_rows(&store, &scope, table), 0);
    }
}

#[test]
/// REQ-RUST-031.0
fn test_hld_stale_input_snapshot() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("stale-input");
    complete_hld_fixture_stages(&store, &scope);
    let frozen = seal_hld_fixture_snapshot(&store, &scope);
    let late_row = StageTableRowRecord {
        stage_owned_table_name: "edge-evidence-record-table".into(),
        stage_row_record_key: "late-evidence".into(),
        stage_row_payload_text: "{}".into(),
    };
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: &scope,
                stage_owner_name_value: "semantic-edge-evidence-stage",
                stage_unit_work_key: "late-unit",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[late_row],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let prior_manifest = store.read_workspace_publication_scope().unwrap();
    let result = store.publish_hld_graph_materialization(
        &frozen,
        &scope,
        &build_empty_hld_output(),
        &RunCancelFlagValue::default(),
    );
    assert!(matches!(
        result,
        Err(HldPublicationPortError::StaleInputSnapshot { stage_name })
            if stage_name == "semantic-edge-evidence-stage"
    ));
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        prior_manifest
    );
    for table in HLD_OUTPUT_TABLE_NAMES {
        assert_eq!(count_hld_raw_rows(&store, &scope, table), 0);
    }
}

#[test]
/// REQ-RUST-031.0
fn test_hld_sealed_node_rewrite() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("sealed-rewrite");
    complete_hld_fixture_stages(&store, &scope);
    let frozen = seal_hld_fixture_snapshot(&store, &scope);
    let published = store
        .publish_hld_graph_materialization(
            &frozen,
            &scope,
            &build_edge_hld_output(),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let old_rows = store
        .read_stage_table_rows(&published, HLD_OUTPUT_TABLE_NAMES[0])
        .unwrap();
    let result = store.write_stage_table_unit(
        &StageUnitWriteRequest {
            stage_run_scope_value: &scope,
            stage_owner_name_value: "hld-current-materialization-stage",
            stage_unit_work_key: "other-unit",
            stage_input_stage_names: &[],
            stage_unit_output_rows: &old_rows,
        },
        &RunCancelFlagValue::default(),
    );
    assert!(result.is_err());
    assert_eq!(
        store
            .read_stage_table_rows(&published, HLD_OUTPUT_TABLE_NAMES[0])
            .unwrap(),
        old_rows
    );
    let new_unit_row = StageTableRowRecord {
        stage_owned_table_name: HLD_OUTPUT_TABLE_NAMES[0].into(),
        stage_row_record_key: "new-unit-node".into(),
        stage_row_payload_text: "{\"new\":true}".into(),
    };
    let new_unit_write = store.write_stage_table_unit(
        &StageUnitWriteRequest {
            stage_run_scope_value: &scope,
            stage_owner_name_value: "hld-current-materialization-stage",
            stage_unit_work_key: "other-new-unit",
            stage_input_stage_names: &[],
            stage_unit_output_rows: &[new_unit_row],
        },
        &RunCancelFlagValue::default(),
    );
    assert!(new_unit_write.is_err());
    let completion = store.write_stage_table_unit(
        &StageUnitWriteRequest {
            stage_run_scope_value: &scope,
            stage_owner_name_value: "hld-current-materialization-stage",
            stage_unit_work_key: "__complete__",
            stage_input_stage_names: &[],
            stage_unit_output_rows: &[],
        },
        &RunCancelFlagValue::default(),
    );
    assert!(completion.is_err());
    assert_eq!(
        store
            .read_stage_table_rows(&published, HLD_OUTPUT_TABLE_NAMES[0])
            .unwrap(),
        old_rows
    );
}

#[test]
/// REQ-RUST-031.0
fn test_hld_sealed_any_marker() {
    let mut results = Vec::new();
    for (attempt, work_key) in [
        ("sealed-global-marker", "__complete__"),
        ("sealed-other-unit", "other-unit"),
    ] {
        let store = build_hld_fixture_store();
        let serving_scope_before = store.read_workspace_publication_scope().unwrap();
        let scope = build_hld_fixture_scope(attempt);
        complete_hld_fixture_stages(&store, &scope);
        if work_key == "__complete__" {
            write_stage_completion_marker(&store, &scope, "hld-current-materialization-stage");
        } else {
            let row = StageTableRowRecord {
                stage_owned_table_name: HLD_OUTPUT_TABLE_NAMES[0].into(),
                stage_row_record_key: "other-unit-node".into(),
                stage_row_payload_text: "{\"stale\":true}".into(),
            };
            store
                .write_stage_table_unit(
                    &StageUnitWriteRequest {
                        stage_run_scope_value: &scope,
                        stage_owner_name_value: "hld-current-materialization-stage",
                        stage_unit_work_key: work_key,
                        stage_input_stage_names: &[],
                        stage_unit_output_rows: &[row],
                    },
                    &RunCancelFlagValue::default(),
                )
                .unwrap();
        }
        let frozen = seal_hld_fixture_snapshot(&store, &scope);
        let old_markers = store
            .read_stage_table_rows(&frozen, "stage-run-generation-records")
            .unwrap();
        let result = store.publish_hld_graph_materialization(
            &frozen,
            &scope,
            &build_empty_hld_output(),
            &RunCancelFlagValue::default(),
        );
        results.push(matches!(
            result,
            Err(HldPublicationPortError::SealedOutputUnit)
        ));
        results.push(
            store
                .read_stage_table_rows(&frozen, "stage-run-generation-records")
                .unwrap()
                == old_markers,
        );
        results.push(store.read_workspace_publication_scope().unwrap() == serving_scope_before);
    }
    assert_eq!(results, [true, true, true, true, true, true]);
}

#[test]
/// REQ-RUST-031.0
fn test_hld_unsealed_other_staging() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("unsealed-other-staging");
    complete_hld_fixture_stages(&store, &scope);
    let frozen = seal_hld_fixture_snapshot(&store, &scope);
    let old_markers = store
        .read_stage_table_rows(&frozen, "stage-run-generation-records")
        .unwrap();
    let stale_row = StageTableRowRecord {
        stage_owned_table_name: HLD_OUTPUT_TABLE_NAMES[0].into(),
        stage_row_record_key: "stale-node".into(),
        stage_row_payload_text: "{\"stale\":true}".into(),
    };
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: &scope,
                stage_owner_name_value: "hld-current-materialization-stage",
                stage_unit_work_key: "other-unit",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[stale_row],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let published = store
        .publish_hld_graph_materialization(
            &frozen,
            &scope,
            &build_empty_hld_output(),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    assert_eq!(
        count_hld_raw_rows(&store, &scope, HLD_OUTPUT_TABLE_NAMES[0]),
        0
    );
    let markers = store
        .read_stage_table_rows(&published, "stage-run-generation-records")
        .unwrap();
    assert!(!markers.iter().any(|row| {
        row.stage_row_record_key == "__marker__/hld-current-materialization-stage/other-unit"
    }));
    assert_eq!(
        store
            .read_stage_table_rows(&frozen, "stage-run-generation-records")
            .unwrap(),
        old_markers
    );
}
