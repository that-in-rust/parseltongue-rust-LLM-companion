use super::*;
use parseltongue_core_row_types::HldMaterializationValidationError;
use parseltongue_table_store_traits::FullGraphInputStorePort;

/// REQ-RUST-041.0, REQ-RUST-045.0
#[test]
fn test_full_sealed_markers_immutable() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("sealed-completeness");
    let input = build_full_graph_fixture(&scope);
    let source = freeze_full_source_fixture(&store, &scope);
    let frozen = store
        .write_full_graph_input(&source, &scope, &input, &RunCancelFlagValue::default())
        .unwrap();
    assert!(crate::stage_table_store_backend::hld_graph_input_turso::write_graph_source_marker_metadata(
        &store, &scope, &source, &scope, 1,
        HldInputCompletenessKind::CompleteTypedInput, &RunCancelFlagValue::default(),
    ).is_err());
    assert_eq!(
        store
            .assemble_full_graph_input(&frozen, &RunCancelFlagValue::default())
            .unwrap(),
        input
    );
}

fn build_full_graph_fixture(scope: &StageRunScopeValue) -> HldMaterializationInputRecord {
    let mut input = build_structural_hld_input(scope);
    input.source_coverage_kind = HldSourceCoverageKind::FullSemantic;
    input.semantic_input_completeness_kind = HldInputCompletenessKind::IncompleteTypedInput;
    input.input_node_records[1].node_effective_visibility_kind =
        Some(HldEffectiveVisibilityKind::Unknown);
    input
}

fn freeze_full_source_fixture(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
) -> StageRunScopeValue {
    let input = build_full_graph_fixture(scope);
    write_hld_scope_configuration(store, scope, &[], Some("full-semantic"));
    write_hld_unit_scope_provenance(
        store,
        scope,
        &input.input_unit_provenance_records[0].unit_compilation_identity_key,
    );
    complete_hld_source_stages(store, scope);
    // The bounded full map uses impl-header work, not a completed signature lowerer.
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: "signature-type-lowering-stage",
                stage_unit_work_key: "impl-header-source-work",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    seal_hld_fixture_snapshot(store, scope)
}

fn append_full_source_work(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    stage: &str,
) {
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: stage,
                stage_unit_work_key: "late-source-work",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
}

/// REQ-RUST-041.0, REQ-RUST-044.0, REQ-RUST-045.0
#[test]
fn test_full_incomplete_checkpoint_roundtrip() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("full-roundtrip");
    let input = build_full_graph_fixture(&scope);
    let source = freeze_full_source_fixture(&store, &scope);
    let frozen = store
        .write_full_graph_input(&source, &scope, &input, &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(
        store
            .assemble_full_graph_input(&frozen, &RunCancelFlagValue::default())
            .unwrap(),
        input
    );
    assert_eq!(store.read_workspace_publication_scope().unwrap(), None);
    let markers = store
        .read_stage_table_rows(&frozen, "stage-run-generation-records")
        .unwrap();
    for owner in [
        "item-effective-visibility-stage",
        "semantic-edge-evidence-stage",
    ] {
        let row = markers
            .iter()
            .find(|row| row.stage_row_record_key == format!("__marker__/{owner}/__complete__"))
            .unwrap();
        let payload: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
        assert_eq!(
            payload["semantic_input_completeness_kind"],
            "IncompleteTypedInput"
        );
        assert_eq!(
            payload["input_snapshot_attempt_key"],
            source.stage_attempt_sequence_value
        );
        assert_eq!(
            payload["input_snapshot_base_attempt_key"],
            scope.stage_attempt_sequence_value
        );
        assert_eq!(payload["input_snapshot_revision"], 1);
    }
}

/// REQ-RUST-045.0
#[test]
fn test_full_known_visibility_incomplete() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("known-incomplete");
    let mut input = build_full_graph_fixture(&scope);
    input.input_node_records[1].node_effective_visibility_kind =
        Some(HldEffectiveVisibilityKind::Public);
    input.input_node_records[1].node_reachable_outside_crate_flag = Some(true);
    write_stage_completion_marker(&store, &scope, "signature-type-lowering-stage");
    let source = freeze_full_source_fixture(&store, &scope);
    assert!(matches!(
        store.write_hld_graph_input(&source, &scope, &input, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::InvalidTypedInput(
            HldMaterializationValidationError::IncompleteTypedInput
        ))
    ));
    let frozen = store
        .write_full_graph_input(&source, &scope, &input, &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(
        store
            .assemble_full_graph_input(&frozen, &RunCancelFlagValue::default())
            .unwrap(),
        input
    );
    assert!(matches!(
        store.assemble_hld_graph_input(&frozen, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::InvalidTypedInput(
            HldMaterializationValidationError::IncompleteTypedInput
        ))
    ));
    // Make every old HLD prerequisite present, to exercise the new completeness gate itself.
    let published_source = seal_hld_fixture_snapshot(&store, &scope);
    assert!(matches!(
        store.publish_hld_graph_materialization(
            &published_source,
            &scope,
            &build_empty_hld_output(),
            &RunCancelFlagValue::default()
        ),
        Err(HldPublicationPortError::IncompleteGraphInput)
    ));
    assert_eq!(store.read_workspace_publication_scope().unwrap(), None);
}

/// REQ-RUST-041.0, REQ-RUST-045.0
#[test]
fn test_full_mismatched_scopes_rejected() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("scope-mismatch");
    let input = build_full_graph_fixture(&scope);
    let source = freeze_full_source_fixture(&store, &scope);
    let mut wrong = input.clone();
    wrong.run_generation_digest_value = RunGenerationDigestValue([1; 32]);
    wrong
        .input_node_records
        .iter_mut()
        .for_each(|row| row.run_generation_digest_value = wrong.run_generation_digest_value);
    wrong
        .input_edge_records
        .iter_mut()
        .for_each(|row| row.run_generation_digest_value = wrong.run_generation_digest_value);
    assert!(matches!(
        store.write_full_graph_input(&source, &scope, &wrong, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::GenerationMismatch)
    ));
    wrong = input.clone();
    wrong.stage_attempt_sequence_value = "other".into();
    assert!(matches!(
        store.write_full_graph_input(&source, &scope, &wrong, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::AttemptMismatch)
    ));
    let other_scope = build_hld_fixture_scope("other");
    let other_source = freeze_full_source_fixture(&store, &other_scope);
    assert!(matches!(
        store.write_full_graph_input(
            &other_source,
            &scope,
            &input,
            &RunCancelFlagValue::default()
        ),
        Err(HldGraphInputStoreError::IncompatibleSourceScope)
    ));
    let unfrozen = build_hld_fixture_scope("never-frozen");
    assert!(matches!(
        store.write_full_graph_input(
            &unfrozen,
            &unfrozen,
            &build_full_graph_fixture(&unfrozen),
            &RunCancelFlagValue::default()
        ),
        Err(HldGraphInputStoreError::UnfrozenSourceSnapshot)
    ));
    assert_eq!(store.read_workspace_publication_scope().unwrap(), None);
}

/// REQ-RUST-041.0, REQ-RUST-045.0
#[test]
fn test_full_stale_source_rejected() {
    for stage in ["syntax-spans", "signature-type-lowering-stage"] {
        let store = build_hld_fixture_store();
        let scope = build_hld_fixture_scope(stage);
        let source = freeze_full_source_fixture(&store, &scope);
        append_full_source_work(&store, &scope, stage);
        assert!(
            matches!(store.write_full_graph_input(&source, &scope, &build_full_graph_fixture(&scope), &RunCancelFlagValue::default()),
            Err(HldGraphInputStoreError::StaleStageMarker { stage_name }) if stage_name == stage)
        );
        assert_eq!(store.read_workspace_publication_scope().unwrap(), None);
    }
}

/// REQ-RUST-041.0, REQ-RUST-045.0
#[test]
fn test_full_resnapshot_source_rejected() {
    for stage in [
        "syntax-spans",
        "signature-type-lowering-stage",
        "semantic-edge-evidence-stage",
    ] {
        let store = build_hld_fixture_store();
        let scope = build_hld_fixture_scope(stage);
        let source = freeze_full_source_fixture(&store, &scope);
        let frozen = store
            .write_full_graph_input(
                &source,
                &scope,
                &build_full_graph_fixture(&scope),
                &RunCancelFlagValue::default(),
            )
            .unwrap();
        append_full_source_work(&store, &scope, stage);
        assert!(
            matches!(store.assemble_full_graph_input(&frozen, &RunCancelFlagValue::default()), Err(HldGraphInputStoreError::StaleStageMarker { stage_name }) if stage_name == stage)
        );
        if stage != "semantic-edge-evidence-stage" {
            let contaminated = seal_hld_fixture_snapshot(&store, &scope);
            assert!(
                matches!(store.assemble_full_graph_input(&contaminated, &RunCancelFlagValue::default()), Err(HldGraphInputStoreError::StaleStageMarker { stage_name }) if stage_name == stage)
            );
        }
    }
}

/// REQ-RUST-041.0, REQ-RUST-045.0
#[test]
fn test_full_cancelled_checkpoint_rejected() {
    let store = build_hld_fixture_store();
    let scope = build_hld_fixture_scope("cancel-checkpoint");
    let source = freeze_full_source_fixture(&store, &scope);
    let cancel = RunCancelFlagValue::default();
    cancel.request_stage_work_cancellation();
    assert!(matches!(
        store.write_full_graph_input(&source, &scope, &build_full_graph_fixture(&scope), &cancel),
        Err(HldGraphInputStoreError::Cancelled)
    ));
    assert_eq!(
        count_raw_stage_markers(&store, &scope, "item-effective-visibility-stage"),
        0
    );
    let frozen = store
        .write_full_graph_input(
            &source,
            &scope,
            &build_full_graph_fixture(&scope),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    assert!(matches!(
        store.assemble_full_graph_input(&frozen, &cancel),
        Err(HldGraphInputStoreError::Cancelled)
    ));
}

/// REQ-RUST-041.0, REQ-RUST-045.0
#[test]
fn test_full_previous_serving_unchanged() {
    let store = build_hld_fixture_store();
    let prior_scope = build_hld_fixture_scope("prior-serving");
    complete_hld_fixture_stages(&store, &prior_scope);
    let prior = seal_hld_fixture_snapshot(&store, &prior_scope);
    let serving = store
        .publish_hld_graph_materialization(
            &prior,
            &prior_scope,
            &build_single_path_hld_output(),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let before =
        HLD_OUTPUT_TABLE_NAMES.map(|table| store.read_stage_table_rows(&serving, table).unwrap());
    let scope = build_hld_fixture_scope("next-checkpoint");
    let source = freeze_full_source_fixture(&store, &scope);
    let input = build_full_graph_fixture(&scope);
    let frozen = store
        .write_full_graph_input(&source, &scope, &input, &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        Some(serving.clone())
    );
    assert!(matches!(
        store.write_full_graph_input(&frozen, &scope, &input, &RunCancelFlagValue::default()),
        Err(HldGraphInputStoreError::IncompatibleSourceScope)
    ));
    let after =
        HLD_OUTPUT_TABLE_NAMES.map(|table| store.read_stage_table_rows(&serving, table).unwrap());
    assert_eq!(after, before);
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        Some(serving)
    );
}

/// REQ-RUST-045.0
#[test]
fn test_full_original_kinds_roundtrip() {
    for kind in [
        HldItemKindValue::Union,
        HldItemKindValue::MacroDefinition,
        HldItemKindValue::MacroInvocation,
        HldItemKindValue::ExternBlock,
        HldItemKindValue::ExternCrate,
        HldItemKindValue::InlineAssembly,
        HldItemKindValue::Function,
        HldItemKindValue::TypeAlias,
        HldItemKindValue::Static,
    ] {
        let store = build_hld_fixture_store();
        let scope = build_hld_fixture_scope("original-kind");
        let mut input = build_full_graph_fixture(&scope);
        input.input_node_records[1].node_item_kind_value = Some(kind);
        if matches!(
            kind,
            HldItemKindValue::Function | HldItemKindValue::TypeAlias | HldItemKindValue::Static
        ) {
            input.input_node_records[1].node_owner_kind_value =
                Some(HldOwnerKindValue::ExternBlock);
        }
        let source = freeze_full_source_fixture(&store, &scope);
        let frozen = store
            .write_full_graph_input(&source, &scope, &input, &RunCancelFlagValue::default())
            .unwrap();
        assert_eq!(
            store
                .assemble_full_graph_input(&frozen, &RunCancelFlagValue::default())
                .unwrap(),
            input
        );
    }
}
