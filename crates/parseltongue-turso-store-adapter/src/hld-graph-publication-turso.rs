use super::{
    TursoWorkspaceStoreAdapter, resolve_snapshot_read_scope, seal_snapshot_publication,
    validate_workspace_publication_gates,
};
use parseltongue_core_row_types::{
    CrateCompilationUnitKey, HldDependencyOriginKind, HldEndpointRollupBucket, HldEvidenceRowKey,
    HldExternalNameKind, HldFolderEndpointBucket, HldGraphEdgeKey, HldInputCompletenessKind,
    HldMaterializationOutputRecord, HldOutputCompletenessKind, HldOutputEdgeStatus,
    HldRelationKindValue, HldSourceCoverageKind, HldTargetIdentityValue, HldViaKindValue,
    NameSymbolTargetKey, RunCancelFlagValue, RunGenerationDigestValue, SemanticOccurrenceKeyValue,
    StageRunScopeValue, StageTableRowRecord,
};
use parseltongue_table_store_traits::{
    HldGraphPublicationPort, HldPublicationOperationKind, HldPublicationPortError,
};
use serde_json::{Value, json};

const HLD_INPUT_STAGE_NAMES: &[&str] = &[
    "crate-graph",
    "source-ingest",
    "syntax-spans",
    "syntax-namespace-input-records",
    "name-resolution",
    "signature-type-lowering-stage",
    "item-effective-visibility-stage",
    "semantic-edge-evidence-stage",
];

const HLD_STRUCTURAL_INPUT_STAGE_NAMES: &[&str] = &[
    "crate-graph",
    "source-ingest",
    "syntax-spans",
    "syntax-namespace-input-records",
    "name-resolution",
    "item-effective-visibility-stage",
    "semantic-edge-evidence-stage",
];

const HLD_OUTPUT_TABLE_NAMES: &[&str] = &[
    "hld-current-interface-nodes",
    "hld-current-dependency-edges",
    "hld-edge-evidence-paths",
    "hld-file-level-rollup",
    "hld-folder-level-rollup",
];

const HLD_STAGE_MARKER_PREFIX: &str = "__marker__/hld-current-materialization-stage/";

#[cfg(test)]
#[path = "hld-publication-contract-tests.rs"]
mod hld_publication_contract_tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HldPublicationCheckpointValue {
    OutputTableWritten(&'static str),
}

impl TursoWorkspaceStoreAdapter {
    fn publish_hld_materialization_checkpointed(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        destination_scope: &StageRunScopeValue,
        output_record: &HldMaterializationOutputRecord,
        cancel_flag: &RunCancelFlagValue,
        mut checkpoint: impl FnMut(HldPublicationCheckpointValue),
    ) -> Result<StageRunScopeValue, HldPublicationPortError> {
        check_hld_stage_work_cancelled(cancel_flag)?;
        validate_hld_output_generations(output_record)?;
        let output_rows = encode_hld_output_rows(output_record, cancel_flag)?;
        retry_hld_materialization_publication(cancel_flag, || {
            self.publish_hld_transaction_once(
                source_snapshot_scope,
                destination_scope,
                output_record,
                &output_rows,
                cancel_flag,
                &mut checkpoint,
            )
        })
    }

    fn publish_hld_transaction_once(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        destination_scope: &StageRunScopeValue,
        output_record: &HldMaterializationOutputRecord,
        output_rows: &[StageTableRowRecord],
        cancel_flag: &RunCancelFlagValue,
        checkpoint: &mut impl FnMut(HldPublicationCheckpointValue),
    ) -> Result<StageRunScopeValue, HldPublicationPortError> {
        check_hld_stage_work_cancelled(cancel_flag)?;
        crate::check_sync_runtime_context().map_err(|source| {
            build_hld_backend_error(HldPublicationOperationKind::ValidateInputSnapshot, source)
        })?;
        self.store_runtime_handle_value.block_on(async {
            let mut connection = self.store_database_handle_value.connect().map_err(|source| {
                build_hld_backend_error(
                    HldPublicationOperationKind::ValidateInputSnapshot,
                    source,
                )
            })?;
            let transaction = connection
                .transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent)
                .await
                .map_err(|source| {
                    build_hld_backend_error(
                        HldPublicationOperationKind::ValidateInputSnapshot,
                        source,
                    )
                })?;
            let result = async {
                check_hld_stage_work_cancelled(cancel_flag)?;
                let (base_scope, revision) =
                    resolve_snapshot_read_scope(&transaction, source_snapshot_scope)
                        .await
                        .map_err(|source| {
                            build_hld_backend_error(
                                HldPublicationOperationKind::ValidateInputSnapshot,
                                source,
                            )
                        })?;
                let Some(revision) = revision else {
                    return Err(HldPublicationPortError::UnfrozenSourceSnapshot);
                };
                if destination_scope.run_generation_digest_value
                    != source_snapshot_scope.run_generation_digest_value
                    || destination_scope.run_generation_digest_value
                        != base_scope.run_generation_digest_value
                    || destination_scope.stage_attempt_sequence_value
                        != base_scope.stage_attempt_sequence_value
                {
                    return Err(HldPublicationPortError::IncompatibleDestinationScope);
                }
                if encode_generation_digest_text(output_record.run_generation_digest_value)
                    != destination_scope.run_generation_digest_value
                {
                    return Err(HldPublicationPortError::GenerationMismatch);
                }
                let frozen_coverage =
                    read_frozen_source_coverage(&transaction, &base_scope, revision).await?;
                if frozen_coverage != output_record.source_coverage_kind {
                    return Err(HldPublicationPortError::CoverageScopeMismatch);
                }
                let required_stages = match frozen_coverage {
                    HldSourceCoverageKind::FullSemantic => HLD_INPUT_STAGE_NAMES,
                    HldSourceCoverageKind::OriginalTopLevelContainment => {
                        HLD_STRUCTURAL_INPUT_STAGE_NAMES
                    }
                };
                check_frozen_input_markers(
                    &transaction,
                    &base_scope,
                    revision,
                    required_stages,
                    frozen_coverage,
                )
                .await?;
                let input_completeness = validate_graph_source_provenance(
                    &transaction,
                    &base_scope,
                    revision,
                    frozen_coverage,
                )
                .await?;
                if input_completeness != HldInputCompletenessKind::CompleteTypedInput {
                    return Err(HldPublicationPortError::IncompleteGraphInput);
                }
                check_hld_stage_work_cancelled(cancel_flag)?;
                check_sealed_hld_unit(&transaction, destination_scope).await?;
                clear_unpublished_hld_staging(&transaction, destination_scope, cancel_flag).await?;
                persist_hld_output_rows(
                    &transaction,
                    destination_scope,
                    output_rows,
                    cancel_flag,
                    checkpoint,
                )
                .await?;
                write_hld_completion_markers(
                    &transaction,
                    HldMarkerPublicationContext {
                        source_snapshot_scope,
                        base_scope: &base_scope,
                        revision,
                        destination_scope,
                        source_coverage_kind: output_record.source_coverage_kind,
                        unknown_reachability_diagnostic_count:
                            count_unknown_local_reachability(output_record),
                        cancel_flag,
                    },
                )
                .await?;
                validate_workspace_publication_gates(&transaction, destination_scope)
                    .await
                    .map_err(|source| {
                        build_hld_backend_error(
                            HldPublicationOperationKind::ValidateInputSnapshot,
                            source,
                        )
                    })?;
                let published_scope = seal_snapshot_publication(&transaction, destination_scope)
                    .await
                    .map_err(|source| {
                        build_hld_backend_error(
                            HldPublicationOperationKind::SealOutputSnapshot,
                            source,
                        )
                    })?;
                check_hld_stage_work_cancelled(cancel_flag)?;
                transaction
                    .execute(
                        "INSERT OR REPLACE INTO \"workspace-generation-publication-manifest\" VALUES (1,?,?)",
                        turso::params![
                            published_scope.run_generation_digest_value.clone(),
                            published_scope.stage_attempt_sequence_value.clone()
                        ],
                    )
                    .await
                    .map_err(|source| {
                        build_hld_backend_error(
                            HldPublicationOperationKind::SwitchPublicationManifest,
                            source,
                        )
                    })?;
                check_hld_stage_work_cancelled(cancel_flag)?;
                Ok(published_scope)
            }
            .await;
            match result {
                Ok(published_scope) => {
                    if cancel_flag.check_stage_work_cancelled() {
                        let _ = transaction.rollback().await;
                        return Err(HldPublicationPortError::Cancelled);
                    }
                    transaction.commit().await.map_err(|source| {
                        build_hld_backend_error(
                            HldPublicationOperationKind::SwitchPublicationManifest,
                            source,
                        )
                    })?;
                    Ok(published_scope)
                }
                Err(error) => {
                    let _ = transaction.rollback().await;
                    Err(error)
                }
            }
        })
    }
}

impl HldGraphPublicationPort for TursoWorkspaceStoreAdapter {
    fn publish_hld_graph_materialization(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        destination_scope: &StageRunScopeValue,
        output_record: &HldMaterializationOutputRecord,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, HldPublicationPortError> {
        self.publish_hld_materialization_checkpointed(
            source_snapshot_scope,
            destination_scope,
            output_record,
            cancel_flag,
            |_| {},
        )
    }
}

fn validate_hld_output_generations(
    output_record: &HldMaterializationOutputRecord,
) -> Result<(), HldPublicationPortError> {
    if output_record.output_completeness_kind != HldOutputCompletenessKind::CompleteGraph
        || output_record.file_rollup_records.is_none()
        || output_record.folder_rollup_records.is_none()
    {
        return Err(HldPublicationPortError::IncompleteOutput);
    }
    let generation = output_record.run_generation_digest_value;
    if output_record
        .selected_node_records
        .iter()
        .any(|node| node.run_generation_digest_value != generation)
        || output_record
            .materialized_edge_records
            .iter()
            .any(|edge| edge.edge_key.run_generation_digest_value != generation)
        || output_record
            .evidence_path_records
            .iter()
            .any(|path| path.edge_key.run_generation_digest_value != generation)
    {
        return Err(HldPublicationPortError::GenerationMismatch);
    }
    Ok(())
}

fn encode_hld_output_rows(
    output_record: &HldMaterializationOutputRecord,
    cancel_flag: &RunCancelFlagValue,
) -> Result<Vec<StageTableRowRecord>, HldPublicationPortError> {
    let generation = output_record.run_generation_digest_value.0;
    let mut rows = Vec::new();
    for node in &output_record.selected_node_records {
        check_hld_stage_work_cancelled(cancel_flag)?;
        rows.push(build_hld_stage_record(
            HLD_OUTPUT_TABLE_NAMES[0],
            json!({
                "generation_digest_full": generation,
                "node_target_identity": encode_target_identity_json(&node.node_target_reference)
            }),
            json!({
                "generation_digest_full": generation,
                "node_target_identity": encode_target_identity_json(&node.node_target_reference),
                "node_qualified_path_text": node.node_qualified_path_text,
                "node_source_relative_path": node.node_source_relative_path,
                "node_effective_visibility_kind": node.node_effective_visibility_kind.map(encode_visibility_kind_json),
                "node_reachable_outside_crate_flag": node.node_reachable_outside_crate_flag,
                "node_signature_text": node.node_signature_text
            }),
        )?);
    }
    for edge in &output_record.materialized_edge_records {
        check_hld_stage_work_cancelled(cancel_flag)?;
        rows.push(build_hld_stage_record(
            HLD_OUTPUT_TABLE_NAMES[1],
            encode_edge_identity_json(&edge.edge_key),
            json!({
                "edge_key": encode_edge_identity_json(&edge.edge_key),
                "edge_source_qualified_path": edge.edge_source_qualified_path,
                "edge_target_qualified_path": edge.edge_target_qualified_path,
                "edge_source_signature_text": edge.edge_source_signature_text,
                "edge_target_signature_text": edge.edge_target_signature_text,
                "edge_resolution_status_kind": encode_output_status_json(edge.edge_resolution_status_kind),
                "edge_evidence_row_count": edge.edge_evidence_row_count,
                "edge_dependency_origin_kind": encode_origin_kind_json(edge.edge_dependency_origin_kind)
            }),
        )?);
    }
    for path in &output_record.evidence_path_records {
        check_hld_stage_work_cancelled(cancel_flag)?;
        let ordered_endpoints: Vec<_> = path
            .ordered_endpoint_path
            .iter()
            .map(encode_target_identity_json)
            .collect();
        let ordered_proofs: Vec<_> = path
            .ordered_evidence_row_keys
            .iter()
            .map(encode_evidence_identity_json)
            .collect();
        let ordered_sites: Vec<_> = path
            .ordered_site_occurrence_keys
            .iter()
            .map(encode_occurrence_identity_json)
            .collect();
        rows.push(build_hld_stage_record(
            HLD_OUTPUT_TABLE_NAMES[2],
            json!({
                "edge_key": encode_edge_identity_json(&path.edge_key),
                "ordered_endpoint_identities": ordered_endpoints,
                "ordered_proof_identities": ordered_proofs
            }),
            json!({
                "edge_key": encode_edge_identity_json(&path.edge_key),
                "ordered_endpoint_path": path.ordered_endpoint_path.iter().map(encode_target_identity_json).collect::<Vec<_>>(),
                "ordered_evidence_row_keys": path.ordered_evidence_row_keys.iter().map(encode_evidence_identity_json).collect::<Vec<_>>(),
                "ordered_site_occurrence_keys": ordered_sites
            }),
        )?);
    }
    for rollup in output_record
        .file_rollup_records
        .as_deref()
        .unwrap_or_default()
    {
        check_hld_stage_work_cancelled(cancel_flag)?;
        rows.push(build_hld_stage_record(
            HLD_OUTPUT_TABLE_NAMES[3],
            json!({
                "generation_digest_full": generation,
                "source_endpoint_bucket": encode_file_bucket_json(&rollup.source_endpoint_bucket),
                "target_endpoint_bucket": encode_file_bucket_json(&rollup.target_endpoint_bucket),
                "relation_kind": encode_relation_kind_json(rollup.edge_relation_kind_value),
                "via_kind": encode_via_kind_json(rollup.edge_collapse_via_kind)
            }),
            json!({
                "generation_digest_full": generation,
                "source_endpoint_bucket": encode_file_bucket_json(&rollup.source_endpoint_bucket),
                "target_endpoint_bucket": encode_file_bucket_json(&rollup.target_endpoint_bucket),
                "relation_kind": encode_relation_kind_json(rollup.edge_relation_kind_value),
                "via_kind": encode_via_kind_json(rollup.edge_collapse_via_kind),
                "edge_row_count": rollup.edge_row_count
            }),
        )?);
    }
    for rollup in output_record
        .folder_rollup_records
        .as_deref()
        .unwrap_or_default()
    {
        check_hld_stage_work_cancelled(cancel_flag)?;
        rows.push(build_hld_stage_record(
            HLD_OUTPUT_TABLE_NAMES[4],
            json!({
                "generation_digest_full": generation,
                "source_folder_bucket": encode_folder_bucket_json(&rollup.source_folder_bucket),
                "target_folder_bucket": encode_folder_bucket_json(&rollup.target_folder_bucket),
                "relation_kind": encode_relation_kind_json(rollup.edge_relation_kind_value),
                "via_kind": encode_via_kind_json(rollup.edge_collapse_via_kind)
            }),
            json!({
                "generation_digest_full": generation,
                "source_folder_bucket": encode_folder_bucket_json(&rollup.source_folder_bucket),
                "target_folder_bucket": encode_folder_bucket_json(&rollup.target_folder_bucket),
                "relation_kind": encode_relation_kind_json(rollup.edge_relation_kind_value),
                "via_kind": encode_via_kind_json(rollup.edge_collapse_via_kind),
                "edge_row_count": rollup.edge_row_count
            }),
        )?);
    }
    Ok(rows)
}

fn build_hld_stage_record(
    table: &str,
    key_value: Value,
    payload_value: Value,
) -> Result<StageTableRowRecord, HldPublicationPortError> {
    let serialize = |value: &Value| {
        serde_json::to_string(value).map_err(|source| {
            build_hld_backend_error(HldPublicationOperationKind::WriteOutputRows, source)
        })
    };
    Ok(StageTableRowRecord {
        stage_owned_table_name: table.into(),
        stage_row_record_key: serialize(&key_value)?,
        stage_row_payload_text: serialize(&payload_value)?,
    })
}

pub(super) fn encode_generation_digest_text(generation: RunGenerationDigestValue) -> String {
    generation
        .0
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(super) fn encode_unit_identity_json(unit: &CrateCompilationUnitKey) -> Value {
    json!({
        "cargo_package_identifier": unit.unit_cargo_package_identifier,
        "cargo_target_name": unit.unit_cargo_target_name,
        "cargo_target_kind": unit.unit_cargo_target_kind,
        "compilation_role_kind": match unit.unit_compilation_role_kind {
            parseltongue_core_row_types::CompilationUnitRoleKind::Host => "Host",
            parseltongue_core_row_types::CompilationUnitRoleKind::Target => "Target",
        },
        "configuration_digest_full": unit.unit_configuration_digest_full
    })
}

fn encode_occurrence_identity_json(occurrence: &SemanticOccurrenceKeyValue) -> Value {
    let parts = occurrence.read_semantic_occurrence_parts();
    let (start, end) = parts.span_byte_range_value.read_source_range_pair();
    json!({
        "unit_compilation_identity": encode_unit_identity_json(parts.unit_compilation_identity_key),
        "source_file_path": parts.span_source_file_path,
        "byte_range": [start, end],
        "content_digest_full": parts.span_content_digest_full.read_full_digest_bytes(),
        "expansion_provenance_chain": parts.span_expansion_provenance_chain
    })
}

pub(super) fn encode_external_name_kind(kind: HldExternalNameKind) -> &'static str {
    match kind {
        HldExternalNameKind::CrateRootedPath => "crate_rooted_path",
        HldExternalNameKind::PreludeName => "prelude_name",
        HldExternalNameKind::BuiltinMember => "builtin_member",
        HldExternalNameKind::CoreDeclaredMember => "core_declared_member",
    }
}

pub(super) fn encode_target_identity_json(target: &HldTargetIdentityValue) -> Value {
    match target {
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(occurrence)) => json!({
            "kind": "Definition",
            "occurrence_identity": encode_occurrence_identity_json(occurrence)
        }),
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::GenericParameter(owner, name)) => {
            json!({
                "kind": "GenericParameter",
                "owner_occurrence_identity": encode_occurrence_identity_json(owner),
                "parameter_name": name
            })
        }
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(name)) => json!({
            "kind": "Builtin",
            "builtin_name": name
        }),
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::CrateRoot(unit)) => json!({
            "kind": "CrateRoot",
            "unit_compilation_identity": encode_unit_identity_json(unit)
        }),
        // REQ-RUST-053.0 N1: the name as the read source writes it; its own source was not read.
        HldTargetIdentityValue::ExternalNamed(external) => json!({
            "kind": "ExternalNamed",
            "external_name_kind": encode_external_name_kind(external.external_name_kind_value),
            "external_crate_name": external.external_crate_name_value,
            "external_item_path_text": external.external_item_path_text,
            "external_source_read_flag": false
        }),
        HldTargetIdentityValue::FilePublicArea(file_area) => json!({
            "kind": "FilePublicArea",
            "unit_compilation_identity": encode_unit_identity_json(file_area.read_unit_identity_value()),
            "file_source_relative_path": file_area.read_source_relative_path()
        }),
    }
}

pub(super) fn encode_evidence_identity_json(evidence: &HldEvidenceRowKey) -> Value {
    json!({
        "site_occurrence_identity": encode_occurrence_identity_json(&evidence.evidence_site_occurrence_key),
        "source_target_identity": encode_target_identity_json(&evidence.evidence_source_occurrence_key),
        "relation_kind": encode_relation_kind_json(evidence.evidence_relation_kind_value),
        "target_identity": evidence.evidence_target_reference_value.as_ref().map(encode_target_identity_json)
    })
}

fn encode_edge_identity_json(edge: &HldGraphEdgeKey) -> Value {
    json!({
        "generation_digest_full": edge.run_generation_digest_value.0,
        "source_target_identity": encode_target_identity_json(&edge.edge_source_occurrence_key),
        "target_identity": encode_target_identity_json(&edge.edge_target_reference_value),
        "relation_kind": encode_relation_kind_json(edge.edge_relation_kind_value),
        "via_kind": encode_via_kind_json(edge.edge_collapse_via_kind)
    })
}

fn encode_file_bucket_json(bucket: &HldEndpointRollupBucket) -> Value {
    match bucket {
        HldEndpointRollupBucket::SourceFile {
            unit_compilation_identity_key,
            file_source_relative_path,
        } => json!({
            "kind": "SourceFile",
            "unit_compilation_identity": encode_unit_identity_json(unit_compilation_identity_key),
            "file_source_relative_path": file_source_relative_path
        }),
        HldEndpointRollupBucket::CrateRoot {
            unit_compilation_identity_key,
        } => json!({
            "kind": "CrateRoot",
            "unit_compilation_identity": encode_unit_identity_json(unit_compilation_identity_key)
        }),
        HldEndpointRollupBucket::CanonicalExternalIdentity {
            endpoint_target_reference,
        } => json!({
            "kind": "CanonicalExternalIdentity",
            "endpoint_target_identity": encode_target_identity_json(endpoint_target_reference)
        }),
    }
}

fn encode_folder_bucket_json(bucket: &HldFolderEndpointBucket) -> Value {
    match bucket {
        HldFolderEndpointBucket::SourceFolder {
            unit_compilation_identity_key,
            folder_source_relative_path,
        } => json!({
            "kind": "SourceFolder",
            "unit_compilation_identity": encode_unit_identity_json(unit_compilation_identity_key),
            "folder_source_relative_path": folder_source_relative_path
        }),
        HldFolderEndpointBucket::CrateRoot {
            unit_compilation_identity_key,
        } => json!({
            "kind": "CrateRoot",
            "unit_compilation_identity": encode_unit_identity_json(unit_compilation_identity_key)
        }),
        HldFolderEndpointBucket::CanonicalExternalIdentity {
            endpoint_target_reference,
        } => json!({
            "kind": "CanonicalExternalIdentity",
            "endpoint_target_identity": encode_target_identity_json(endpoint_target_reference)
        }),
    }
}

fn encode_relation_kind_json(relation: HldRelationKindValue) -> &'static str {
    match relation {
        HldRelationKindValue::Calls => "Calls",
        HldRelationKindValue::UsesType => "UsesType",
        HldRelationKindValue::Imports => "Imports",
        HldRelationKindValue::Reads => "Reads",
        HldRelationKindValue::Implements => "Implements",
        HldRelationKindValue::Reexports => "Reexports",
        HldRelationKindValue::Contains => "Contains",
        HldRelationKindValue::Derives => "Derives",
        HldRelationKindValue::Generates => "Generates",
        HldRelationKindValue::Expands => "Expands",
    }
}

fn encode_via_kind_json(via: HldViaKindValue) -> &'static str {
    match via {
        HldViaKindValue::Direct => "Direct",
        HldViaKindValue::Collapsed => "Collapsed",
    }
}

fn encode_visibility_kind_json(
    visibility: parseltongue_core_row_types::HldEffectiveVisibilityKind,
) -> &'static str {
    match visibility {
        parseltongue_core_row_types::HldEffectiveVisibilityKind::Public => "Public",
        parseltongue_core_row_types::HldEffectiveVisibilityKind::RestrictedNonPrivate => {
            "RestrictedNonPrivate"
        }
        parseltongue_core_row_types::HldEffectiveVisibilityKind::PrivateEquivalent => {
            "PrivateEquivalent"
        }
        parseltongue_core_row_types::HldEffectiveVisibilityKind::NotApplicable => "NotApplicable",
        parseltongue_core_row_types::HldEffectiveVisibilityKind::Unknown => "Unknown",
    }
}

fn encode_output_status_json(status: HldOutputEdgeStatus) -> &'static str {
    match status {
        HldOutputEdgeStatus::Extracted => "Extracted",
    }
}

fn encode_origin_kind_json(origin: HldDependencyOriginKind) -> &'static str {
    match origin {
        HldDependencyOriginKind::SameCrate => "SameCrate",
        HldDependencyOriginKind::WorkspaceCrate => "WorkspaceCrate",
        HldDependencyOriginKind::StandardLibrary => "StandardLibrary",
        HldDependencyOriginKind::ExternalCrate => "ExternalCrate",
    }
}

pub(super) async fn read_frozen_source_coverage(
    connection: &turso::Connection,
    base_scope: &StageRunScopeValue,
    revision: i64,
) -> Result<HldSourceCoverageKind, HldPublicationPortError> {
    let mut rows = connection
        .query(
            "SELECT config.stage_row_payload_text FROM \"workspace-config-state-records\" AS config WHERE config.run_generation_digest_value=? AND config.stage_attempt_sequence_value=? AND config.stage_row_record_key='workspace' AND EXISTS (SELECT 1 FROM \"frozen-stage-unit-memberships\" AS frozen WHERE frozen.generation_key=config.run_generation_digest_value AND frozen.source_attempt_key=config.stage_attempt_sequence_value AND frozen.stage_marker_key='__marker__/crate-graph/' || config.stage_unit_work_key AND frozen.publication_revision<=?) LIMIT 1",
            turso::params![
                base_scope.run_generation_digest_value.clone(),
                base_scope.stage_attempt_sequence_value.clone(),
                revision
            ],
        )
        .await
        .map_err(|source| {
            build_hld_backend_error(HldPublicationOperationKind::ValidateInputSnapshot, source)
        })?;
    let payload_text = rows
        .next()
        .await
        .map_err(|source| {
            build_hld_backend_error(HldPublicationOperationKind::ValidateInputSnapshot, source)
        })?
        .and_then(|row| row.get::<String>(0).ok());
    let Some(payload_text) = payload_text else {
        return Ok(HldSourceCoverageKind::FullSemantic);
    };
    let payload: Value = serde_json::from_str(&payload_text)
        .map_err(|_| HldPublicationPortError::InvalidFrozenWorkspaceConfig)?;
    match payload.get("workspace_mapping_scope_kind") {
        None => Ok(HldSourceCoverageKind::FullSemantic),
        Some(Value::String(scope)) if scope == "full-semantic" || scope == "FullSemantic" => {
            Ok(HldSourceCoverageKind::FullSemantic)
        }
        Some(Value::String(scope)) if scope == "original-top-level-containment" => {
            Ok(HldSourceCoverageKind::OriginalTopLevelContainment)
        }
        Some(_) => Err(HldPublicationPortError::InvalidFrozenWorkspaceConfig),
    }
}

pub(super) async fn check_frozen_input_markers(
    connection: &turso::Connection,
    base_scope: &StageRunScopeValue,
    revision: i64,
    required_stages: &[&str],
    source_coverage_kind: HldSourceCoverageKind,
) -> Result<(), HldPublicationPortError> {
    for stage in required_stages {
        let graph_owner_stage = matches!(
            *stage,
            "item-effective-visibility-stage" | "semantic-edge-evidence-stage"
        );
        if graph_owner_stage
            && query_unfrozen_stage_marker(connection, base_scope, revision, stage).await?
        {
            return Err(HldPublicationPortError::StaleInputSnapshot {
                stage_name: (*stage).to_owned(),
            });
        }
        let mut rows = connection
            .query(
                "SELECT marker.stage_row_payload_text FROM \"stage-run-generation-records\" AS marker JOIN \"frozen-stage-unit-memberships\" AS frozen ON frozen.generation_key=marker.run_generation_digest_value AND frozen.source_attempt_key=marker.stage_attempt_sequence_value AND frozen.stage_marker_key=marker.stage_row_record_key WHERE marker.run_generation_digest_value=? AND marker.stage_attempt_sequence_value=? AND marker.stage_row_record_key=? AND frozen.publication_revision<=? LIMIT 1",
                turso::params![
                    base_scope.run_generation_digest_value.clone(),
                    base_scope.stage_attempt_sequence_value.clone(),
                    format!("__marker__/{stage}/__complete__"),
                    revision
                ],
            )
            .await
            .map_err(|source| {
                build_hld_backend_error(
                    HldPublicationOperationKind::ValidateInputSnapshot,
                    source,
                )
            })?;
        let marker_payload = rows
            .next()
            .await
            .map_err(|source| {
                build_hld_backend_error(HldPublicationOperationKind::ValidateInputSnapshot, source)
            })?
            .and_then(|row| row.get::<String>(0).ok());
        let completed = marker_payload
            .as_deref()
            .and_then(|payload| serde_json::from_str::<Value>(payload).ok())
            .and_then(|payload| {
                payload
                    .get("status")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .is_some_and(|status| status == "complete");
        if !completed {
            return Err(HldPublicationPortError::MissingInputStageMarker {
                stage_name: (*stage).to_owned(),
            });
        }
        if matches!(
            *stage,
            "item-effective-visibility-stage" | "semantic-edge-evidence-stage"
        ) {
            let payload = marker_payload
                .as_deref()
                .and_then(|payload| serde_json::from_str::<Value>(payload).ok());
            let marker_scope = payload
                .as_ref()
                .and_then(|value| value.get("mapping_scope_kind"))
                .and_then(Value::as_str);
            let marker_coverage = payload
                .as_ref()
                .and_then(|value| value.get("source_coverage_kind"))
                .and_then(Value::as_str);
            match source_coverage_kind {
                HldSourceCoverageKind::OriginalTopLevelContainment
                    if marker_scope != Some("original-top-level-containment")
                        || marker_coverage != Some("original-file-top-level-only") =>
                {
                    return Err(HldPublicationPortError::CoverageScopeMismatch);
                }
                HldSourceCoverageKind::FullSemantic
                    if (marker_scope.is_some() || marker_coverage.is_some())
                        && !matches!(
                            (marker_scope, marker_coverage),
                            (Some("full-semantic" | "FullSemantic"), Some("FullSemantic"))
                        ) =>
                {
                    return Err(HldPublicationPortError::CoverageScopeMismatch);
                }
                _ => {}
            }
        }
        if !graph_owner_stage
            && query_unfrozen_stage_marker(connection, base_scope, revision, stage).await?
        {
            return Err(HldPublicationPortError::StaleInputSnapshot {
                stage_name: (*stage).to_owned(),
            });
        }
    }
    // Impl-header rows may exist before signature lowering is globally complete.
    // Their work markers still belong to the exact frozen source closure.
    if !required_stages.contains(&"signature-type-lowering-stage")
        && query_unfrozen_stage_marker(
            connection,
            base_scope,
            revision,
            "signature-type-lowering-stage",
        )
        .await?
    {
        return Err(HldPublicationPortError::StaleInputSnapshot {
            stage_name: "signature-type-lowering-stage".into(),
        });
    }
    Ok(())
}

pub(super) async fn validate_graph_source_provenance(
    connection: &turso::Connection,
    graph_base_scope: &StageRunScopeValue,
    graph_revision: i64,
    coverage: HldSourceCoverageKind,
) -> Result<HldInputCompletenessKind, HldPublicationPortError> {
    let node = read_graph_source_marker(
        connection,
        graph_base_scope,
        graph_revision,
        "item-effective-visibility-stage",
    )
    .await?;
    let edge = read_graph_source_marker(
        connection,
        graph_base_scope,
        graph_revision,
        "semantic-edge-evidence-stage",
    )
    .await?;
    if node.input_completeness_kind != edge.input_completeness_kind {
        return Err(HldPublicationPortError::InvalidGraphSourceProvenance);
    }
    let completeness = node
        .input_completeness_kind
        .unwrap_or(HldInputCompletenessKind::CompleteTypedInput);
    let source_marker = graph_source_marker_pair(
        node.source_marker_metadata,
        edge.source_marker_metadata,
        coverage,
    )?;
    let Some((
        snapshot_generation,
        snapshot_attempt,
        base_generation,
        base_attempt,
        source_revision,
    )) = source_marker
    else {
        return Ok(completeness);
    };
    let source_snapshot_scope = StageRunScopeValue {
        run_generation_digest_value: snapshot_generation,
        stage_attempt_sequence_value: snapshot_attempt,
    };
    let (source_base_scope, resolved_revision) =
        resolve_snapshot_read_scope(connection, &source_snapshot_scope)
            .await
            .map_err(|source| {
                build_hld_backend_error(HldPublicationOperationKind::ValidateInputSnapshot, source)
            })?;
    if resolved_revision != Some(source_revision)
        || source_base_scope.run_generation_digest_value != base_generation
        || source_base_scope.stage_attempt_sequence_value != base_attempt
        || source_base_scope.run_generation_digest_value
            != graph_base_scope.run_generation_digest_value
        || source_base_scope.stage_attempt_sequence_value
            != graph_base_scope.stage_attempt_sequence_value
    {
        return Err(HldPublicationPortError::InvalidGraphSourceProvenance);
    }
    let source_coverage =
        read_frozen_source_coverage(connection, &source_base_scope, source_revision).await?;
    if source_coverage != coverage {
        return Err(HldPublicationPortError::InvalidGraphSourceProvenance);
    }
    check_frozen_input_markers(
        connection,
        &source_base_scope,
        source_revision,
        &[
            "crate-graph",
            "source-ingest",
            "syntax-spans",
            "syntax-namespace-input-records",
            "name-resolution",
        ],
        source_coverage,
    )
    .await?;
    Ok(completeness)
}

pub(super) type GraphSourceMarkerMetadata = Option<(String, String, String, String, i64)>;

struct GraphInputMarkerRecord {
    source_marker_metadata: GraphSourceMarkerMetadata,
    input_completeness_kind: Option<HldInputCompletenessKind>,
}

pub(super) fn graph_source_marker_pair(
    node: GraphSourceMarkerMetadata,
    edge: GraphSourceMarkerMetadata,
    coverage: HldSourceCoverageKind,
) -> Result<GraphSourceMarkerMetadata, HldPublicationPortError> {
    if node != edge || (node.is_none() && coverage != HldSourceCoverageKind::FullSemantic) {
        return Err(HldPublicationPortError::InvalidGraphSourceProvenance);
    }
    Ok(node)
}

async fn read_graph_source_marker(
    connection: &turso::Connection,
    graph_base_scope: &StageRunScopeValue,
    graph_revision: i64,
    owner: &str,
) -> Result<GraphInputMarkerRecord, HldPublicationPortError> {
    let key = format!("__marker__/{owner}/__complete__");
    let mut rows = connection.query(
        "SELECT marker.stage_row_payload_text FROM \"stage-run-generation-records\" AS marker JOIN \"frozen-stage-unit-memberships\" AS frozen ON frozen.generation_key=marker.run_generation_digest_value AND frozen.source_attempt_key=marker.stage_attempt_sequence_value AND frozen.stage_marker_key=marker.stage_row_record_key WHERE marker.run_generation_digest_value=? AND marker.stage_attempt_sequence_value=? AND marker.stage_row_record_key=? AND frozen.publication_revision<=? LIMIT 1",
        turso::params![graph_base_scope.run_generation_digest_value.clone(), graph_base_scope.stage_attempt_sequence_value.clone(), key, graph_revision],
    ).await.map_err(|source| build_hld_backend_error(HldPublicationOperationKind::ValidateInputSnapshot, source))?;
    let text = rows
        .next()
        .await
        .map_err(|source| {
            build_hld_backend_error(HldPublicationOperationKind::ValidateInputSnapshot, source)
        })?
        .and_then(|row| row.get::<String>(0).ok())
        .ok_or(HldPublicationPortError::InvalidGraphSourceProvenance)?;
    let payload: Value = serde_json::from_str(&text)
        .map_err(|_| HldPublicationPortError::InvalidGraphSourceProvenance)?;
    let completeness = match payload.get("semantic_input_completeness_kind") {
        None => None, // Legacy HLD markers predate explicit graph completeness.
        Some(Value::String(value)) if value == "CompleteTypedInput" => {
            Some(HldInputCompletenessKind::CompleteTypedInput)
        }
        Some(Value::String(value)) if value == "IncompleteTypedInput" => {
            Some(HldInputCompletenessKind::IncompleteTypedInput)
        }
        Some(_) => return Err(HldPublicationPortError::InvalidGraphSourceProvenance),
    };
    let keys = [
        "input_snapshot_generation_key",
        "input_snapshot_attempt_key",
        "input_snapshot_base_generation_key",
        "input_snapshot_base_attempt_key",
        "input_snapshot_revision",
    ];
    if keys.iter().all(|key| payload.get(*key).is_none()) {
        if completeness.is_some() {
            return Err(HldPublicationPortError::InvalidGraphSourceProvenance);
        }
        return Ok(GraphInputMarkerRecord {
            source_marker_metadata: None,
            input_completeness_kind: None,
        });
    }
    let strings = keys[..4]
        .iter()
        .map(|key| payload.get(*key).and_then(Value::as_str).map(str::to_owned))
        .collect::<Option<Vec<_>>>()
        .ok_or(HldPublicationPortError::InvalidGraphSourceProvenance)?;
    let revision = payload
        .get(keys[4])
        .and_then(Value::as_i64)
        .ok_or(HldPublicationPortError::InvalidGraphSourceProvenance)?;
    Ok(GraphInputMarkerRecord {
        source_marker_metadata: Some((
            strings[0].clone(),
            strings[1].clone(),
            strings[2].clone(),
            strings[3].clone(),
            revision,
        )),
        input_completeness_kind: completeness,
    })
}

async fn query_unfrozen_stage_marker(
    connection: &turso::Connection,
    base_scope: &StageRunScopeValue,
    revision: i64,
    stage_name: &str,
) -> Result<bool, HldPublicationPortError> {
    let marker_prefix = format!("__marker__/{stage_name}/");
    let mut rows = connection
        .query(
            "SELECT 1 FROM \"stage-run-generation-records\" AS marker WHERE marker.run_generation_digest_value=? AND marker.stage_attempt_sequence_value=? AND substr(marker.stage_row_record_key,1,?)=? AND NOT EXISTS (SELECT 1 FROM \"frozen-stage-unit-memberships\" AS frozen WHERE frozen.generation_key=marker.run_generation_digest_value AND frozen.source_attempt_key=marker.stage_attempt_sequence_value AND frozen.stage_marker_key=marker.stage_row_record_key AND frozen.publication_revision<=?) LIMIT 1",
            turso::params![
                base_scope.run_generation_digest_value.clone(),
                base_scope.stage_attempt_sequence_value.clone(),
                marker_prefix.len() as i64,
                marker_prefix,
                revision
            ],
        )
        .await
        .map_err(|source| {
            build_hld_backend_error(HldPublicationOperationKind::ValidateInputSnapshot, source)
        })?;
    rows.next()
        .await
        .map(|row| row.is_some())
        .map_err(|source| {
            build_hld_backend_error(HldPublicationOperationKind::ValidateInputSnapshot, source)
        })
}

async fn check_sealed_hld_unit(
    connection: &turso::Connection,
    destination_scope: &StageRunScopeValue,
) -> Result<(), HldPublicationPortError> {
    let mut rows = connection
        .query(
            "SELECT 1 FROM \"frozen-stage-unit-memberships\" WHERE generation_key=? AND source_attempt_key=? AND (substr(stage_marker_key,1,?)=? OR stage_marker_key='__marker__/hld-current-materialization-stage/__complete__') LIMIT 1",
            turso::params![
                destination_scope.run_generation_digest_value.clone(),
                destination_scope.stage_attempt_sequence_value.clone(),
                HLD_STAGE_MARKER_PREFIX.len() as i64,
                HLD_STAGE_MARKER_PREFIX
            ],
        )
        .await
        .map_err(|source| {
            build_hld_backend_error(HldPublicationOperationKind::ValidateInputSnapshot, source)
        })?;
    if rows
        .next()
        .await
        .map_err(|source| {
            build_hld_backend_error(HldPublicationOperationKind::ValidateInputSnapshot, source)
        })?
        .is_some()
    {
        return Err(HldPublicationPortError::SealedOutputUnit);
    }
    Ok(())
}

async fn clear_unpublished_hld_staging(
    connection: &turso::Connection,
    destination_scope: &StageRunScopeValue,
    cancel_flag: &RunCancelFlagValue,
) -> Result<(), HldPublicationPortError> {
    for table in HLD_OUTPUT_TABLE_NAMES {
        check_hld_stage_work_cancelled(cancel_flag)?;
        connection
            .execute(
                format!("DELETE FROM \"{table}\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=?"),
                turso::params![
                    destination_scope.run_generation_digest_value.clone(),
                    destination_scope.stage_attempt_sequence_value.clone()
                ],
            )
            .await
            .map_err(|source| {
                build_hld_backend_error(HldPublicationOperationKind::WriteOutputRows, source)
            })?;
    }
    connection
        .execute(
            "DELETE FROM \"stage-run-generation-records\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND (substr(stage_row_record_key,1,?)=? OR stage_row_record_key='__marker__/hld-current-materialization-stage/__complete__')",
            turso::params![
                destination_scope.run_generation_digest_value.clone(),
                destination_scope.stage_attempt_sequence_value.clone(),
                HLD_STAGE_MARKER_PREFIX.len() as i64,
                HLD_STAGE_MARKER_PREFIX
            ],
        )
        .await
        .map_err(|source| {
            build_hld_backend_error(HldPublicationOperationKind::WriteOutputRows, source)
        })?;
    Ok(())
}

async fn persist_hld_output_rows(
    connection: &turso::Connection,
    destination_scope: &StageRunScopeValue,
    rows: &[StageTableRowRecord],
    cancel_flag: &RunCancelFlagValue,
    checkpoint: &mut impl FnMut(HldPublicationCheckpointValue),
) -> Result<(), HldPublicationPortError> {
    for table in HLD_OUTPUT_TABLE_NAMES {
        for row in rows
            .iter()
            .filter(|row| row.stage_owned_table_name == *table)
        {
            check_hld_stage_work_cancelled(cancel_flag)?;
            connection
                .execute(
                    format!("INSERT INTO \"{table}\" VALUES (?,?,?,?,?)"),
                    turso::params![
                        destination_scope.run_generation_digest_value.clone(),
                        destination_scope.stage_attempt_sequence_value.clone(),
                        row.stage_row_record_key.clone(),
                        "hld/current".to_owned(),
                        row.stage_row_payload_text.clone()
                    ],
                )
                .await
                .map_err(|source| {
                    build_hld_backend_error(HldPublicationOperationKind::WriteOutputRows, source)
                })?;
            check_hld_stage_work_cancelled(cancel_flag)?;
        }
        checkpoint(HldPublicationCheckpointValue::OutputTableWritten(table));
        check_hld_stage_work_cancelled(cancel_flag)?;
    }
    Ok(())
}

struct HldMarkerPublicationContext<'a> {
    source_snapshot_scope: &'a StageRunScopeValue,
    base_scope: &'a StageRunScopeValue,
    revision: i64,
    destination_scope: &'a StageRunScopeValue,
    source_coverage_kind: HldSourceCoverageKind,
    unknown_reachability_diagnostic_count: usize,
    cancel_flag: &'a RunCancelFlagValue,
}

async fn write_hld_completion_markers(
    connection: &turso::Connection,
    context: HldMarkerPublicationContext<'_>,
) -> Result<(), HldPublicationPortError> {
    let HldMarkerPublicationContext {
        source_snapshot_scope,
        base_scope,
        revision,
        destination_scope,
        source_coverage_kind,
        unknown_reachability_diagnostic_count,
        cancel_flag,
    } = context;
    check_hld_stage_work_cancelled(cancel_flag)?;
    let mapping_scope_kind = match source_coverage_kind {
        HldSourceCoverageKind::FullSemantic => "full-semantic",
        HldSourceCoverageKind::OriginalTopLevelContainment => "original-top-level-containment",
    };
    let unit_marker_payload = serde_json::to_string(&json!({
        "status":"complete",
        "mapping_scope_kind":mapping_scope_kind,
        "source_coverage_kind":match source_coverage_kind {
            HldSourceCoverageKind::FullSemantic => "FullSemantic",
            HldSourceCoverageKind::OriginalTopLevelContainment => "original-file-top-level-only"
        }
    }))
    .map_err(|source| {
        build_hld_backend_error(HldPublicationOperationKind::WriteOutputRows, source)
    })?;
    connection
        .execute(
            "INSERT INTO \"stage-run-generation-records\" VALUES (?,?,?,?,?)",
            turso::params![
                destination_scope.run_generation_digest_value.clone(),
                destination_scope.stage_attempt_sequence_value.clone(),
                "__marker__/hld-current-materialization-stage/hld/current".to_owned(),
                "hld/current".to_owned(),
                unit_marker_payload
            ],
        )
        .await
        .map_err(|source| {
            build_hld_backend_error(HldPublicationOperationKind::WriteOutputRows, source)
        })?;
    check_hld_stage_work_cancelled(cancel_flag)?;
    let global_marker_payload = serde_json::to_string(&json!({
        "status": "complete",
        "input_snapshot_generation_key": source_snapshot_scope.run_generation_digest_value,
        "input_snapshot_attempt_key": source_snapshot_scope.stage_attempt_sequence_value,
        "input_snapshot_base_generation_key": base_scope.run_generation_digest_value,
        "input_snapshot_base_attempt_key": base_scope.stage_attempt_sequence_value,
        "input_snapshot_revision": revision,
        "source_coverage_kind": match source_coverage_kind {
            HldSourceCoverageKind::FullSemantic => "FullSemantic",
            HldSourceCoverageKind::OriginalTopLevelContainment => "original-file-top-level-only"
        },
        "mapping_scope_kind": mapping_scope_kind,
        "unknown_reachability_diagnostic_count": unknown_reachability_diagnostic_count
    }))
    .map_err(|source| {
        build_hld_backend_error(HldPublicationOperationKind::WriteOutputRows, source)
    })?;
    connection
        .execute(
            "INSERT INTO \"stage-run-generation-records\" VALUES (?,?,?,?,?)",
            turso::params![
                destination_scope.run_generation_digest_value.clone(),
                destination_scope.stage_attempt_sequence_value.clone(),
                "__marker__/hld-current-materialization-stage/__complete__".to_owned(),
                "__complete__".to_owned(),
                global_marker_payload
            ],
        )
        .await
        .map_err(|source| {
            build_hld_backend_error(HldPublicationOperationKind::WriteOutputRows, source)
        })?;
    check_hld_stage_work_cancelled(cancel_flag)?;
    Ok(())
}

fn count_unknown_local_reachability(output_record: &HldMaterializationOutputRecord) -> usize {
    output_record
        .selected_node_records
        .iter()
        .filter(|node| {
            node.node_reachable_outside_crate_flag.is_none()
                && matches!(
                    &node.node_target_reference,
                    HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(_))
                )
        })
        .count()
}

fn check_hld_stage_work_cancelled(
    cancel_flag: &RunCancelFlagValue,
) -> Result<(), HldPublicationPortError> {
    if cancel_flag.check_stage_work_cancelled() {
        Err(HldPublicationPortError::Cancelled)
    } else {
        Ok(())
    }
}

fn build_hld_backend_error(
    operation: HldPublicationOperationKind,
    source: impl std::error::Error + Send + Sync + 'static,
) -> HldPublicationPortError {
    HldPublicationPortError::Backend {
        operation,
        source: Box::new(source),
    }
}

fn retry_hld_materialization_publication<T>(
    cancel_flag: &RunCancelFlagValue,
    mut attempt: impl FnMut() -> Result<T, HldPublicationPortError>,
) -> Result<T, HldPublicationPortError> {
    for retry in 0..6 {
        check_hld_stage_work_cancelled(cancel_flag)?;
        match attempt() {
            Ok(value) => return Ok(value),
            Err(error) => {
                let is_busy = matches!(
                    &error,
                    HldPublicationPortError::Backend { source, .. }
                        if source.downcast_ref::<turso::Error>().is_some_and(|source| {
                            matches!(source, turso::Error::Busy(_) | turso::Error::BusySnapshot(_))
                        })
                );
                if !is_busy || retry == 5 {
                    return Err(error);
                }
                for _ in 0..(10_u64 << retry) {
                    check_hld_stage_work_cancelled(cancel_flag)?;
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        }
    }
    unreachable!()
}
