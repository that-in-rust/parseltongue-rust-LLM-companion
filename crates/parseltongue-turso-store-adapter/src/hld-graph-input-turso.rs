use super::resolve_snapshot_read_scope;
use crate::{TursoWorkspaceStoreAdapter, build_store_backend_error, check_sync_runtime_context};
use parseltongue_core_row_types::{
    CompilationUnitRoleKind, CrateCompilationUnitKey, FileAreaIdentityKey, HldEdgeInputRecord,
    HldEdgeResolutionValue, HldEffectiveVisibilityKind, HldEvidenceRowKey,
    HldInputCompletenessKind, HldItemKindValue, HldMaterializationInputRecord,
    HldNodeInputCategory, HldNodeInputRecord, HldOwnerKindValue, HldRelationKindValue,
    HldSourceCoverageKind, HldTargetIdentityValue, HldUnitOriginKind, HldUnitProvenanceRecord,
    HldUnresolvedReasonKind, NameSymbolTargetKey, RunCancelFlagValue, RunGenerationDigestValue,
    SemanticOccurrenceKeyValue, SpanByteRangeValue, SpanDigestIdValue, StageRunScopeValue,
    StageTableRowRecord, StageUnitWriteRequest, validate_hld_materialization_input,
};
use parseltongue_table_store_traits::{
    FullGraphInputStorePort, HldGraphInputStoreError, HldGraphInputStorePort,
    HldPublicationPortError, StageTableStorePort,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const GRAPH_SOURCE_STAGE_NAMES: &[&str] = &[
    "crate-graph",
    "source-ingest",
    "syntax-spans",
    "syntax-namespace-input-records",
    "name-resolution",
];
const GRAPH_ASSEMBLY_STAGE_NAMES: &[&str] = &[
    "crate-graph",
    "source-ingest",
    "syntax-spans",
    "syntax-namespace-input-records",
    "name-resolution",
    "item-effective-visibility-stage",
    "semantic-edge-evidence-stage",
];
const GRAPH_UNIT_WORK_KEY: &str = "hld-graph-input";

impl FullGraphInputStorePort for TursoWorkspaceStoreAdapter {
    fn write_full_graph_input(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        destination_base_scope: &StageRunScopeValue,
        input_record: &HldMaterializationInputRecord,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, HldGraphInputStoreError> {
        self.write_graph_input_checkpoint(
            source_snapshot_scope,
            destination_base_scope,
            input_record,
            cancel_flag,
        )
    }

    fn assemble_full_graph_input(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<HldMaterializationInputRecord, HldGraphInputStoreError> {
        assemble_full_graph_checkpoint(self, source_snapshot_scope, cancel_flag, |_| {})
    }
}

impl HldGraphInputStorePort for TursoWorkspaceStoreAdapter {
    fn write_hld_graph_input(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        destination_base_scope: &StageRunScopeValue,
        input_record: &HldMaterializationInputRecord,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, HldGraphInputStoreError> {
        check_graph_input_cancelled(cancel_flag)?;
        validate_hld_materialization_input(input_record)?;
        self.write_graph_input_checkpoint(
            source_snapshot_scope,
            destination_base_scope,
            input_record,
            cancel_flag,
        )
    }

    fn assemble_hld_graph_input(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<HldMaterializationInputRecord, HldGraphInputStoreError> {
        assemble_graph_input_checkpoint(self, source_snapshot_scope, cancel_flag, |_| {})
    }
}

impl TursoWorkspaceStoreAdapter {
    fn write_graph_input_checkpoint(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        destination_base_scope: &StageRunScopeValue,
        input_record: &HldMaterializationInputRecord,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, HldGraphInputStoreError> {
        check_graph_input_cancelled(cancel_flag)?;
        parseltongue_core_row_types::validate_full_graph_input(input_record)?;
        let input_generation = super::hld_graph_publication_turso::encode_generation_digest_text(
            input_record.run_generation_digest_value,
        );
        if input_generation != destination_base_scope.run_generation_digest_value {
            return Err(HldGraphInputStoreError::GenerationMismatch);
        }
        if input_record.stage_attempt_sequence_value
            != destination_base_scope.stage_attempt_sequence_value
        {
            return Err(HldGraphInputStoreError::AttemptMismatch);
        }
        let (source_coverage, source_revision) = validate_graph_source_snapshot(
            self,
            source_snapshot_scope,
            destination_base_scope,
            input_record.source_coverage_kind,
        )?;
        let persisted_units = read_persisted_unit_provenance(self, source_snapshot_scope)?;
        let mut input_units = input_record.input_unit_provenance_records.clone();
        input_units.sort();
        if input_units != persisted_units {
            return Err(HldGraphInputStoreError::UnitProvenanceMismatch);
        }
        let node_rows = encode_graph_node_rows(input_record, source_coverage)?;
        let edge_rows = encode_graph_edge_rows(input_record, source_coverage)?;
        validate_graph_stages_writable(self, destination_base_scope, source_revision)?;
        let input_stage_names = GRAPH_SOURCE_STAGE_NAMES;
        self.write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: destination_base_scope,
                stage_owner_name_value: "item-effective-visibility-stage",
                stage_unit_work_key: GRAPH_UNIT_WORK_KEY,
                stage_input_stage_names: input_stage_names,
                stage_unit_output_rows: &node_rows,
            },
            cancel_flag,
        )?;
        check_graph_input_cancelled(cancel_flag)?;
        self.write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: destination_base_scope,
                stage_owner_name_value: "semantic-edge-evidence-stage",
                stage_unit_work_key: GRAPH_UNIT_WORK_KEY,
                stage_input_stage_names: input_stage_names,
                stage_unit_output_rows: &edge_rows,
            },
            cancel_flag,
        )?;
        check_graph_input_cancelled(cancel_flag)?;
        write_graph_source_marker_metadata(
            self,
            destination_base_scope,
            source_snapshot_scope,
            destination_base_scope,
            source_revision,
            input_record.semantic_input_completeness_kind,
            cancel_flag,
        )?;
        seal_graph_analysis_input_snapshot(
            self,
            source_snapshot_scope,
            destination_base_scope,
            source_revision,
            source_coverage,
            cancel_flag,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GraphAssemblyCheckpointKind {
    NodeRowsRead,
    EdgeRowsRead,
    NodeRowsDecoded,
    EdgeRowsDecoded,
    UnitProvenanceRead,
    SourceRevalidated,
}

pub(super) fn assemble_graph_input_checkpoint<F>(
    store: &TursoWorkspaceStoreAdapter,
    source_snapshot_scope: &StageRunScopeValue,
    cancel_flag: &RunCancelFlagValue,
    checkpoint: F,
) -> Result<HldMaterializationInputRecord, HldGraphInputStoreError>
where
    F: FnMut(GraphAssemblyCheckpointKind),
{
    let assembled =
        assemble_full_graph_checkpoint(store, source_snapshot_scope, cancel_flag, checkpoint)?;
    validate_hld_materialization_input(&assembled)?;
    Ok(assembled)
}

pub(super) fn assemble_full_graph_checkpoint<F>(
    store: &TursoWorkspaceStoreAdapter,
    source_snapshot_scope: &StageRunScopeValue,
    cancel_flag: &RunCancelFlagValue,
    mut checkpoint: F,
) -> Result<HldMaterializationInputRecord, HldGraphInputStoreError>
where
    F: FnMut(GraphAssemblyCheckpointKind),
{
    check_graph_input_cancelled(cancel_flag)?;
    let (base_scope, source_coverage, input_completeness) =
        resolve_graph_assembly_snapshot(store, source_snapshot_scope)?;
    let generation = decode_generation_digest_text(&base_scope.run_generation_digest_value)
        .map_err(|reason| {
            malformed_graph_row("frozen-stage-publication-records", "generation", reason)
        })?;
    let node_rows =
        store.read_stage_table_rows(source_snapshot_scope, "item-effective-visibility-records")?;
    checkpoint(GraphAssemblyCheckpointKind::NodeRowsRead);
    check_graph_input_cancelled(cancel_flag)?;
    let edge_rows =
        store.read_stage_table_rows(source_snapshot_scope, "edge-evidence-record-table")?;
    checkpoint(GraphAssemblyCheckpointKind::EdgeRowsRead);
    check_graph_input_cancelled(cancel_flag)?;
    let input_node_records = decode_graph_node_rows(&node_rows, generation, source_coverage)?;
    checkpoint(GraphAssemblyCheckpointKind::NodeRowsDecoded);
    check_graph_input_cancelled(cancel_flag)?;
    let input_edge_records = decode_graph_edge_rows(&edge_rows, generation, source_coverage)?;
    checkpoint(GraphAssemblyCheckpointKind::EdgeRowsDecoded);
    check_graph_input_cancelled(cancel_flag)?;
    let input_unit_provenance_records =
        read_persisted_unit_provenance(store, source_snapshot_scope)?;
    checkpoint(GraphAssemblyCheckpointKind::UnitProvenanceRead);
    check_graph_input_cancelled(cancel_flag)?;
    let assembled = HldMaterializationInputRecord {
        run_generation_digest_value: generation,
        stage_attempt_sequence_value: base_scope.stage_attempt_sequence_value,
        semantic_input_completeness_kind: input_completeness,
        source_coverage_kind: source_coverage,
        input_unit_provenance_records,
        input_node_records,
        input_edge_records,
    };
    parseltongue_core_row_types::validate_full_graph_input(&assembled)?;
    check_graph_input_cancelled(cancel_flag)?;
    resolve_graph_assembly_snapshot(store, source_snapshot_scope)?;
    checkpoint(GraphAssemblyCheckpointKind::SourceRevalidated);
    check_graph_input_cancelled(cancel_flag)?;
    Ok(assembled)
}

pub(super) fn write_graph_source_marker_metadata(
    store: &TursoWorkspaceStoreAdapter,
    marker_scope: &StageRunScopeValue,
    source_snapshot_scope: &StageRunScopeValue,
    source_base_scope: &StageRunScopeValue,
    source_revision: i64,
    input_completeness: HldInputCompletenessKind,
    cancel_flag: &RunCancelFlagValue,
) -> Result<(), HldGraphInputStoreError> {
    check_sync_runtime_context()
        .map_err(|source| HldGraphInputStoreError::Backend(build_store_backend_error(source)))?;
    let metadata = json!({
        "input_snapshot_generation_key": source_snapshot_scope.run_generation_digest_value,
        "input_snapshot_attempt_key": source_snapshot_scope.stage_attempt_sequence_value,
        "input_snapshot_base_generation_key": source_base_scope.run_generation_digest_value,
        "input_snapshot_base_attempt_key": source_base_scope.stage_attempt_sequence_value,
        "input_snapshot_revision": source_revision,
        "semantic_input_completeness_kind": match input_completeness {
            HldInputCompletenessKind::CompleteTypedInput => "CompleteTypedInput",
            HldInputCompletenessKind::IncompleteTypedInput => "IncompleteTypedInput",
        },
    });
    // The shared stage transaction persists metadata with the marker, and
    // applies the same cancellation and sealed-unit guards as ordinary writes.
    for owner in [
        "item-effective-visibility-stage",
        "semantic-edge-evidence-stage",
    ] {
        check_graph_input_cancelled(cancel_flag)?;
        store.write_stage_unit_metadata(
            &StageUnitWriteRequest {
                stage_run_scope_value: marker_scope,
                stage_owner_name_value: owner,
                stage_unit_work_key: "__complete__",
                stage_input_stage_names: GRAPH_SOURCE_STAGE_NAMES,
                stage_unit_output_rows: &[],
            },
            cancel_flag,
            Some(&metadata),
        )?;
    }
    Ok(())
}

pub(super) fn seal_graph_analysis_input_snapshot(
    store: &TursoWorkspaceStoreAdapter,
    source_snapshot_scope: &StageRunScopeValue,
    destination_base_scope: &StageRunScopeValue,
    expected_source_revision: i64,
    expected_coverage: HldSourceCoverageKind,
    cancel_flag: &RunCancelFlagValue,
) -> Result<StageRunScopeValue, HldGraphInputStoreError> {
    check_sync_runtime_context()
        .map_err(|source| HldGraphInputStoreError::Backend(build_store_backend_error(source)))?;
    store.store_runtime_handle_value.block_on(async {
        let mut connection = store
            .store_database_handle_value
            .connect()
            .map_err(|source| {
                HldGraphInputStoreError::Backend(build_store_backend_error(source))
            })?;
        let transaction = connection
            .transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent)
            .await
            .map_err(|source| {
                HldGraphInputStoreError::Backend(build_store_backend_error(source))
            })?;
        let result = async {
            check_graph_input_cancelled(cancel_flag)?;
            let (source_base, revision) =
                resolve_snapshot_read_scope(&transaction, source_snapshot_scope)
                    .await
                    .map_err(|source| {
                        HldGraphInputStoreError::Backend(build_store_backend_error(source))
                    })?;
            let Some(revision) = revision else {
                return Err(HldGraphInputStoreError::UnfrozenSourceSnapshot);
            };
            if revision != expected_source_revision
                || source_base.run_generation_digest_value
                    != destination_base_scope.run_generation_digest_value
                || source_base.stage_attempt_sequence_value
                    != destination_base_scope.stage_attempt_sequence_value
            {
                return Err(HldGraphInputStoreError::SourceSnapshotValidation(
                    HldPublicationPortError::InvalidGraphSourceProvenance,
                ));
            }
            let frozen_coverage = super::hld_graph_publication_turso::read_frozen_source_coverage(
                &transaction,
                &source_base,
                revision,
            )
            .await?;
            if frozen_coverage != expected_coverage {
                return Err(HldGraphInputStoreError::CoverageScopeMismatch);
            }
            super::hld_graph_publication_turso::check_frozen_input_markers(
                &transaction,
                &source_base,
                revision,
                GRAPH_SOURCE_STAGE_NAMES,
                frozen_coverage,
            )
            .await?;
            check_graph_input_cancelled(cancel_flag)?;
            let published = super::frozen_stage_snapshot_rules::seal_snapshot_publication(
                &transaction,
                destination_base_scope,
            )
            .await
            .map_err(|source| {
                HldGraphInputStoreError::Backend(build_store_backend_error(source))
            })?;
            transaction
                .execute(
                    "INSERT OR REPLACE INTO \"analysis-input-publication-manifest\" VALUES (1,?,?)",
                    turso::params![
                        published.run_generation_digest_value.clone(),
                        published.stage_attempt_sequence_value.clone()
                    ],
                )
                .await
                .map_err(|source| {
                    HldGraphInputStoreError::Backend(build_store_backend_error(source))
                })?;
            check_graph_input_cancelled(cancel_flag)?;
            Ok(published)
        }
        .await;
        match result {
            Ok(scope) => {
                transaction.commit().await.map_err(|source| {
                    HldGraphInputStoreError::Backend(build_store_backend_error(source))
                })?;
                Ok(scope)
            }
            Err(error) => {
                let _ = transaction.rollback().await;
                Err(error)
            }
        }
    })
}

fn validate_graph_source_snapshot(
    store: &TursoWorkspaceStoreAdapter,
    source_snapshot_scope: &StageRunScopeValue,
    destination_base_scope: &StageRunScopeValue,
    requested_coverage: HldSourceCoverageKind,
) -> Result<(HldSourceCoverageKind, i64), HldGraphInputStoreError> {
    check_sync_runtime_context()
        .map_err(|source| HldGraphInputStoreError::Backend(build_store_backend_error(source)))?;
    store.store_runtime_handle_value.block_on(async {
        let mut connection = store
            .store_database_handle_value
            .connect()
            .map_err(|source| {
                HldGraphInputStoreError::Backend(build_store_backend_error(source))
            })?;
        let transaction = connection
            .transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent)
            .await
            .map_err(|source| {
                HldGraphInputStoreError::Backend(build_store_backend_error(source))
            })?;
        let (base_scope, revision) =
            resolve_snapshot_read_scope(&transaction, source_snapshot_scope)
                .await
                .map_err(|source| {
                    HldGraphInputStoreError::Backend(build_store_backend_error(source))
                })?;
        let Some(revision) = revision else {
            return Err(HldGraphInputStoreError::UnfrozenSourceSnapshot);
        };
        if source_snapshot_scope.run_generation_digest_value
            != destination_base_scope.run_generation_digest_value
            || base_scope.run_generation_digest_value
                != destination_base_scope.run_generation_digest_value
            || base_scope.stage_attempt_sequence_value
                != destination_base_scope.stage_attempt_sequence_value
        {
            return Err(HldGraphInputStoreError::IncompatibleSourceScope);
        }
        let coverage = super::hld_graph_publication_turso::read_frozen_source_coverage(
            &transaction,
            &base_scope,
            revision,
        )
        .await
        .map_err(map_graph_snapshot_error)?;
        if coverage != requested_coverage {
            return Err(HldGraphInputStoreError::CoverageScopeMismatch);
        }
        super::hld_graph_publication_turso::check_frozen_input_markers(
            &transaction,
            &base_scope,
            revision,
            GRAPH_SOURCE_STAGE_NAMES,
            coverage,
        )
        .await
        .map_err(map_graph_snapshot_error)?;
        Ok((coverage, revision))
    })
}

fn resolve_graph_assembly_snapshot(
    store: &TursoWorkspaceStoreAdapter,
    source_snapshot_scope: &StageRunScopeValue,
) -> Result<
    (
        StageRunScopeValue,
        HldSourceCoverageKind,
        HldInputCompletenessKind,
    ),
    HldGraphInputStoreError,
> {
    check_sync_runtime_context()
        .map_err(|source| HldGraphInputStoreError::Backend(build_store_backend_error(source)))?;
    store.store_runtime_handle_value.block_on(async {
        let mut connection = store
            .store_database_handle_value
            .connect()
            .map_err(|source| {
                HldGraphInputStoreError::Backend(build_store_backend_error(source))
            })?;
        let transaction = connection
            .transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent)
            .await
            .map_err(|source| {
                HldGraphInputStoreError::Backend(build_store_backend_error(source))
            })?;
        let (base_scope, revision) =
            resolve_snapshot_read_scope(&transaction, source_snapshot_scope)
                .await
                .map_err(|source| {
                    HldGraphInputStoreError::Backend(build_store_backend_error(source))
                })?;
        let Some(revision) = revision else {
            return Err(HldGraphInputStoreError::UnfrozenSourceSnapshot);
        };
        if base_scope.run_generation_digest_value
            != source_snapshot_scope.run_generation_digest_value
        {
            return Err(HldGraphInputStoreError::IncompatibleSourceScope);
        }
        let coverage = super::hld_graph_publication_turso::read_frozen_source_coverage(
            &transaction,
            &base_scope,
            revision,
        )
        .await
        .map_err(map_graph_snapshot_error)?;
        super::hld_graph_publication_turso::check_frozen_input_markers(
            &transaction,
            &base_scope,
            revision,
            GRAPH_ASSEMBLY_STAGE_NAMES,
            coverage,
        )
        .await
        .map_err(map_graph_snapshot_error)?;
        let completeness = super::hld_graph_publication_turso::validate_graph_source_provenance(
            &transaction,
            &base_scope,
            revision,
            coverage,
        )
        .await
        .map_err(map_graph_snapshot_error)?;
        Ok((base_scope, coverage, completeness))
    })
}

fn validate_graph_stages_writable(
    store: &TursoWorkspaceStoreAdapter,
    base_scope: &StageRunScopeValue,
    revision: i64,
) -> Result<(), HldGraphInputStoreError> {
    store.store_runtime_handle_value.block_on(async {
        let connection = store.store_database_handle_value.connect().map_err(|source| {
            HldGraphInputStoreError::Backend(build_store_backend_error(source))
        })?;
        for stage in [
            "item-effective-visibility-stage",
            "semantic-edge-evidence-stage",
        ] {
            let prefix = format!("__marker__/{stage}/");
            let mut rows = connection
                .query(
                    "SELECT marker.stage_row_record_key,EXISTS(SELECT 1 FROM \"frozen-stage-unit-memberships\" AS frozen WHERE frozen.generation_key=marker.run_generation_digest_value AND frozen.source_attempt_key=marker.stage_attempt_sequence_value AND frozen.stage_marker_key=marker.stage_row_record_key AND frozen.publication_revision<=?) FROM \"stage-run-generation-records\" AS marker WHERE marker.run_generation_digest_value=? AND marker.stage_attempt_sequence_value=? AND substr(marker.stage_row_record_key,1,?)=? LIMIT 1",
                    turso::params![
                        revision,
                        base_scope.run_generation_digest_value.clone(),
                        base_scope.stage_attempt_sequence_value.clone(),
                        prefix.len() as i64,
                        prefix
                    ],
                )
                .await
                .map_err(|source| {
                    HldGraphInputStoreError::Backend(build_store_backend_error(source))
                })?;
            if let Some(row) = rows.next().await.map_err(|source| {
                HldGraphInputStoreError::Backend(build_store_backend_error(source))
            })? {
                let frozen: bool = row.get(1).map_err(|source| {
                    HldGraphInputStoreError::Backend(build_store_backend_error(source))
                })?;
                return Err(if frozen {
                    HldGraphInputStoreError::IncompatibleSourceScope
                } else {
                    HldGraphInputStoreError::StaleStageMarker {
                        stage_name: stage.to_owned(),
                    }
                });
            }
        }
        Ok(())
    })
}

fn read_persisted_unit_provenance(
    store: &TursoWorkspaceStoreAdapter,
    frozen_scope: &StageRunScopeValue,
) -> Result<Vec<HldUnitProvenanceRecord>, HldGraphInputStoreError> {
    let raw_rows = store.read_stage_table_rows(frozen_scope, "crate-dependency-graph-table")?;
    let scope_rows = store.read_stage_table_rows(frozen_scope, "syntax-namespace-input-scopes")?;
    let mut raw_units = BTreeMap::new();
    for row in &raw_rows {
        let payload = parse_graph_payload(row)?;
        let raw_key = required_string(&payload, "unit_compilation_identity_key")
            .map_err(|reason| {
                malformed_graph_row(
                    "crate-dependency-graph-table",
                    &row.stage_row_record_key,
                    reason,
                )
            })?
            .to_owned();
        if raw_units.insert(raw_key.clone(), payload).is_some() {
            return Err(HldGraphInputStoreError::DuplicatePersistedKey {
                table: "crate-dependency-graph-table".into(),
                record_key: raw_key,
            });
        }
    }
    let mut mapped_keys = BTreeSet::new();
    let mut mapped_units = BTreeSet::new();
    let mut records = Vec::new();
    for row in &scope_rows {
        let payload = parse_graph_payload(row)?;
        let scope_identity = payload.get("scope_module_identity_key").ok_or_else(|| {
            malformed_graph_row(
                "syntax-namespace-input-scopes",
                &row.stage_row_record_key,
                "scope identity is absent",
            )
        })?;
        let path_parts = scope_identity
            .get("scope_semantic_path_parts")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                malformed_graph_row(
                    "syntax-namespace-input-scopes",
                    &row.stage_row_record_key,
                    "scope path is malformed",
                )
            })?;
        if !path_parts.is_empty() {
            continue;
        }
        let raw_key = required_string(&payload, "scope_cargo_unit_key").map_err(|reason| {
            malformed_graph_row(
                "syntax-namespace-input-scopes",
                &row.stage_row_record_key,
                reason,
            )
        })?;
        if !mapped_keys.insert(raw_key.to_owned()) {
            return Err(HldGraphInputStoreError::DuplicatePersistedKey {
                table: "syntax-namespace-input-scopes".into(),
                record_key: raw_key.to_owned(),
            });
        }
        let unit_json = scope_identity
            .get("scope_compilation_unit_key")
            .ok_or_else(|| {
                malformed_graph_row(
                    "syntax-namespace-input-scopes",
                    &row.stage_row_record_key,
                    "canonical root unit is absent",
                )
            })?;
        let unit = decode_scope_unit_identity(unit_json).map_err(|reason| {
            malformed_graph_row(
                "syntax-namespace-input-scopes",
                &row.stage_row_record_key,
                reason,
            )
        })?;
        let raw_unit = raw_units.get(raw_key).ok_or_else(|| {
            malformed_graph_row(
                "syntax-namespace-input-scopes",
                &row.stage_row_record_key,
                "root cargo unit has no raw crate-graph row",
            )
        })?;
        validate_raw_unit_mapping(raw_unit, &unit).map_err(|reason| {
            malformed_graph_row("crate-dependency-graph-table", raw_key, reason)
        })?;
        if !mapped_units.insert(unit.clone()) {
            return Err(HldGraphInputStoreError::DuplicatePersistedKey {
                table: "syntax-namespace-input-scopes".into(),
                record_key: raw_key.to_owned(),
            });
        }
        let sysroot = bool_field_or_default(raw_unit, "unit_sysroot_context_flag", false).map_err(
            |reason| malformed_graph_row("crate-dependency-graph-table", raw_key, reason),
        )?;
        let mapped = required_bool(raw_unit, "unit_mapped_target_flag").map_err(|reason| {
            malformed_graph_row("crate-dependency-graph-table", raw_key, reason)
        })?;
        let origin = if sysroot {
            HldUnitOriginKind::StandardLibrary
        } else if mapped {
            HldUnitOriginKind::Workspace
        } else {
            HldUnitOriginKind::ExternalCrate
        };
        records.push(HldUnitProvenanceRecord {
            unit_compilation_identity_key: unit,
            unit_origin_kind: origin,
        });
    }
    for (raw_key, raw_unit) in &raw_units {
        let mapped = required_bool(raw_unit, "unit_mapped_target_flag").map_err(|reason| {
            malformed_graph_row("crate-dependency-graph-table", raw_key, reason)
        })?;
        let sysroot = bool_field_or_default(raw_unit, "unit_sysroot_context_flag", false).map_err(
            |reason| malformed_graph_row("crate-dependency-graph-table", raw_key, reason),
        )?;
        if (mapped || sysroot) && !mapped_keys.contains(raw_key) {
            return Err(HldGraphInputStoreError::MissingRootUnitScope {
                raw_unit_key: raw_key.clone(),
            });
        }
    }
    records.sort();
    Ok(records)
}

fn validate_raw_unit_mapping(
    raw_unit: &Value,
    canonical_unit: &CrateCompilationUnitKey,
) -> Result<(), String> {
    for (raw_field, canonical_value) in [
        (
            "unit_cargo_package_identifier",
            &canonical_unit.unit_cargo_package_identifier,
        ),
        (
            "unit_cargo_target_name",
            &canonical_unit.unit_cargo_target_name,
        ),
        (
            "unit_cargo_target_kind",
            &canonical_unit.unit_cargo_target_kind,
        ),
    ] {
        if required_string(raw_unit, raw_field)? != canonical_value {
            return Err(format!(
                "root namespace unit disagrees with raw field {raw_field}"
            ));
        }
    }
    let role = required_string(raw_unit, "unit_compilation_role_kind")?;
    let canonical_role = match canonical_unit.unit_compilation_role_kind {
        CompilationUnitRoleKind::Host => "host",
        CompilationUnitRoleKind::Target => "target",
    };
    if !role.eq_ignore_ascii_case(canonical_role) {
        return Err("root namespace unit role disagrees with raw crate-graph unit".into());
    }
    Ok(())
}

pub(super) fn encode_graph_node_rows(
    input: &HldMaterializationInputRecord,
    coverage: HldSourceCoverageKind,
) -> Result<Vec<StageTableRowRecord>, HldGraphInputStoreError> {
    let mut seen = BTreeSet::new();
    input
        .input_node_records
        .iter()
        .map(|node| {
            let identity = super::hld_graph_publication_turso::encode_target_identity_json(
                &node.node_target_reference,
            );
            let key = serde_json::to_string(&identity)
                .map_err(|source| HldGraphInputStoreError::Backend(build_store_backend_error(source)))?;
            if !seen.insert(key.clone()) {
                return Err(HldGraphInputStoreError::DuplicatePersistedKey {
                    table: "item-effective-visibility-records".into(),
                    record_key: key,
                });
            }
            let payload = json!({
                "run_generation_digest_full":super::hld_graph_publication_turso::encode_generation_digest_text(input.run_generation_digest_value),
                "mapping_scope_kind":mapping_scope_text(coverage),
                "source_coverage_kind":coverage_text(coverage),
                "node_target_identity":identity,
                "node_input_category":encode_node_category_value(node.node_input_category),
                "node_item_kind_value":node.node_item_kind_value.map(encode_item_kind_value),
                "node_owner_kind_value":node.node_owner_kind_value.map(encode_owner_kind_value),
                "node_function_body_ancestry_flag":node.node_function_body_ancestry_flag,
                "node_effective_visibility_kind":node.node_effective_visibility_kind.map(encode_visibility_value),
                "node_reachable_outside_crate_flag":node.node_reachable_outside_crate_flag,
                "node_source_relative_path":node.node_source_relative_path,
                "node_qualified_path_text":node.node_qualified_path_text,
                "node_signature_text":node.node_signature_text,
                "node_generating_call_identity":node.node_generating_call_reference.as_ref().map(super::hld_graph_publication_turso::encode_target_identity_json)
            });
            graph_table_row("item-effective-visibility-records", key, payload)
        })
        .collect()
}

pub(super) fn encode_graph_edge_rows(
    input: &HldMaterializationInputRecord,
    coverage: HldSourceCoverageKind,
) -> Result<Vec<StageTableRowRecord>, HldGraphInputStoreError> {
    let mut seen = BTreeSet::new();
    input
        .input_edge_records
        .iter()
        .map(|edge| {
            let source = super::hld_graph_publication_turso::encode_target_identity_json(
                &edge.edge_source_reference,
            );
            let evidence = super::hld_graph_publication_turso::encode_evidence_identity_json(
                &edge.edge_evidence_row_key,
            );
            let resolution = encode_edge_resolution_value(&edge.edge_resolution_value);
            let key_value = json!({
                "source_target_identity":source,
                "relation_kind":encode_relation_value(edge.edge_relation_kind_value),
                "resolution":resolution,
                "evidence_identity":evidence
            });
            let key = serde_json::to_string(&key_value)
                .map_err(|source| HldGraphInputStoreError::Backend(build_store_backend_error(source)))?;
            if !seen.insert(key.clone()) {
                return Err(HldGraphInputStoreError::DuplicatePersistedKey {
                    table: "edge-evidence-record-table".into(),
                    record_key: key,
                });
            }
            let payload = json!({
                "run_generation_digest_full":super::hld_graph_publication_turso::encode_generation_digest_text(input.run_generation_digest_value),
                "mapping_scope_kind":mapping_scope_text(coverage),
                "source_coverage_kind":coverage_text(coverage),
                "edge_source_target_identity":source,
                "edge_relation_kind":encode_relation_value(edge.edge_relation_kind_value),
                "edge_resolution":resolution,
                "edge_evidence_identity":evidence,
                "edge_dispatch_metadata_text":edge.edge_dispatch_metadata_text
            });
            graph_table_row("edge-evidence-record-table", key, payload)
        })
        .collect()
}

fn graph_table_row(
    table: &str,
    key: String,
    payload: Value,
) -> Result<StageTableRowRecord, HldGraphInputStoreError> {
    Ok(StageTableRowRecord {
        stage_owned_table_name: table.into(),
        stage_row_record_key: key,
        stage_row_payload_text: serde_json::to_string(&payload).map_err(|source| {
            HldGraphInputStoreError::Backend(build_store_backend_error(source))
        })?,
    })
}

fn decode_graph_node_rows(
    rows: &[StageTableRowRecord],
    generation: RunGenerationDigestValue,
    coverage: HldSourceCoverageKind,
) -> Result<Vec<HldNodeInputRecord>, HldGraphInputStoreError> {
    let mut seen = BTreeSet::new();
    rows.iter()
        .map(|row| {
            if !seen.insert(row.stage_row_record_key.clone()) {
                return Err(HldGraphInputStoreError::DuplicatePersistedKey {
                    table: row.stage_owned_table_name.clone(),
                    record_key: row.stage_row_record_key.clone(),
                });
            }
            let payload = parse_graph_payload(row)?;
            validate_graph_payload_scope(&payload, row, generation, coverage)?;
            let node = HldNodeInputRecord {
                run_generation_digest_value: generation,
                node_target_reference: decode_target_identity_value(
                    &payload["node_target_identity"],
                    row,
                )?,
                node_input_category: decode_node_category_value(&payload["node_input_category"])
                    .map_err(|reason| {
                        malformed_graph_row(
                            &row.stage_owned_table_name,
                            &row.stage_row_record_key,
                            reason,
                        )
                    })?,
                node_item_kind_value: decode_optional_item_kind_value(
                    &payload["node_item_kind_value"],
                )
                .map_err(|reason| {
                    malformed_graph_row(
                        &row.stage_owned_table_name,
                        &row.stage_row_record_key,
                        reason,
                    )
                })?,
                node_owner_kind_value: decode_optional_owner_kind_value(
                    &payload["node_owner_kind_value"],
                )
                .map_err(|reason| {
                    malformed_graph_row(
                        &row.stage_owned_table_name,
                        &row.stage_row_record_key,
                        reason,
                    )
                })?,
                node_function_body_ancestry_flag: required_bool(
                    &payload,
                    "node_function_body_ancestry_flag",
                )
                .map_err(|reason| {
                    malformed_graph_row(
                        &row.stage_owned_table_name,
                        &row.stage_row_record_key,
                        reason,
                    )
                })?,
                node_effective_visibility_kind: decode_optional_visibility_value(
                    &payload["node_effective_visibility_kind"],
                )
                .map_err(|reason| {
                    malformed_graph_row(
                        &row.stage_owned_table_name,
                        &row.stage_row_record_key,
                        reason,
                    )
                })?,
                node_reachable_outside_crate_flag: optional_bool(
                    &payload["node_reachable_outside_crate_flag"],
                )
                .map_err(|reason| {
                    malformed_graph_row(
                        &row.stage_owned_table_name,
                        &row.stage_row_record_key,
                        reason,
                    )
                })?,
                node_source_relative_path: optional_string(&payload["node_source_relative_path"])
                    .map_err(|reason| {
                    malformed_graph_row(
                        &row.stage_owned_table_name,
                        &row.stage_row_record_key,
                        reason,
                    )
                })?,
                node_qualified_path_text: required_string(&payload, "node_qualified_path_text")
                    .map_err(|reason| {
                        malformed_graph_row(
                            &row.stage_owned_table_name,
                            &row.stage_row_record_key,
                            reason,
                        )
                    })?
                    .into(),
                node_signature_text: optional_string(&payload["node_signature_text"]).map_err(
                    |reason| {
                        malformed_graph_row(
                            &row.stage_owned_table_name,
                            &row.stage_row_record_key,
                            reason,
                        )
                    },
                )?,
                node_generating_call_reference: Some(&payload["node_generating_call_identity"])
                    .filter(|call| !call.is_null())
                    .map(|call| decode_target_identity_value(call, row))
                    .transpose()?,
            };
            let canonical_key = serde_json::to_string(
                &super::hld_graph_publication_turso::encode_target_identity_json(
                    &node.node_target_reference,
                ),
            )
            .map_err(|source| {
                HldGraphInputStoreError::Backend(build_store_backend_error(source))
            })?;
            if canonical_key != row.stage_row_record_key {
                return Err(malformed_graph_row(
                    &row.stage_owned_table_name,
                    &row.stage_row_record_key,
                    "row key conflicts with persisted node identity",
                ));
            }
            Ok(node)
        })
        .collect()
}

fn decode_graph_edge_rows(
    rows: &[StageTableRowRecord],
    generation: RunGenerationDigestValue,
    coverage: HldSourceCoverageKind,
) -> Result<Vec<HldEdgeInputRecord>, HldGraphInputStoreError> {
    let mut seen = BTreeSet::new();
    rows.iter()
        .map(|row| {
            if !seen.insert(row.stage_row_record_key.clone()) {
                return Err(HldGraphInputStoreError::DuplicatePersistedKey {
                    table: row.stage_owned_table_name.clone(),
                    record_key: row.stage_row_record_key.clone(),
                });
            }
            let payload = parse_graph_payload(row)?;
            validate_graph_payload_scope(&payload, row, generation, coverage)?;
            let edge = HldEdgeInputRecord {
                run_generation_digest_value: generation,
                edge_source_reference: decode_target_identity_value(
                    &payload["edge_source_target_identity"],
                    row,
                )?,
                edge_relation_kind_value: decode_relation_value(&payload["edge_relation_kind"])
                    .map_err(|reason| {
                        malformed_graph_row(
                            &row.stage_owned_table_name,
                            &row.stage_row_record_key,
                            reason,
                        )
                    })?,
                edge_resolution_value: decode_edge_resolution_value(
                    &payload["edge_resolution"],
                    row,
                )?,
                edge_evidence_row_key: decode_evidence_identity_value(
                    &payload["edge_evidence_identity"],
                    row,
                )?,
                edge_dispatch_metadata_text: optional_string(
                    &payload["edge_dispatch_metadata_text"],
                )
                .map_err(|reason| {
                    malformed_graph_row(
                        &row.stage_owned_table_name,
                        &row.stage_row_record_key,
                        reason,
                    )
                })?,
            };
            let canonical_key = serde_json::to_string(&json!({
                "source_target_identity":super::hld_graph_publication_turso::encode_target_identity_json(&edge.edge_source_reference),
                "relation_kind":encode_relation_value(edge.edge_relation_kind_value),
                "resolution":encode_edge_resolution_value(&edge.edge_resolution_value),
                "evidence_identity":super::hld_graph_publication_turso::encode_evidence_identity_json(&edge.edge_evidence_row_key)
            }))
            .map_err(|source| {
                HldGraphInputStoreError::Backend(build_store_backend_error(source))
            })?;
            if canonical_key != row.stage_row_record_key {
                return Err(malformed_graph_row(
                    &row.stage_owned_table_name,
                    &row.stage_row_record_key,
                    "row key conflicts with persisted edge identity",
                ));
            }
            Ok(edge)
        })
        .collect()
}

fn validate_graph_payload_scope(
    payload: &Value,
    row: &StageTableRowRecord,
    generation: RunGenerationDigestValue,
    coverage: HldSourceCoverageKind,
) -> Result<(), HldGraphInputStoreError> {
    let expected_generation =
        super::hld_graph_publication_turso::encode_generation_digest_text(generation);
    let actual_generation =
        required_string(payload, "run_generation_digest_full").map_err(|reason| {
            malformed_graph_row(
                &row.stage_owned_table_name,
                &row.stage_row_record_key,
                reason,
            )
        })?;
    if actual_generation != expected_generation
        || payload["mapping_scope_kind"] != mapping_scope_text(coverage)
        || payload["source_coverage_kind"] != coverage_text(coverage)
    {
        return Err(HldGraphInputStoreError::CoverageScopeMismatch);
    }
    Ok(())
}

fn parse_graph_payload(row: &StageTableRowRecord) -> Result<Value, HldGraphInputStoreError> {
    serde_json::from_str(&row.stage_row_payload_text).map_err(|source| {
        malformed_graph_row(
            &row.stage_owned_table_name,
            &row.stage_row_record_key,
            source.to_string(),
        )
    })
}

fn decode_target_identity_value(
    value: &Value,
    row: &StageTableRowRecord,
) -> Result<HldTargetIdentityValue, HldGraphInputStoreError> {
    let malformed = |reason: String| {
        malformed_graph_row(
            &row.stage_owned_table_name,
            &row.stage_row_record_key,
            reason,
        )
    };
    match required_string(value, "kind").map_err(malformed)? {
        "Definition" => Ok(HldTargetIdentityValue::Symbol(
            NameSymbolTargetKey::Definition(
                decode_occurrence_identity_value(&value["occurrence_identity"])
                    .map_err(malformed)?,
            ),
        )),
        "GenericParameter" => Ok(HldTargetIdentityValue::Symbol(
            NameSymbolTargetKey::GenericParameter(
                decode_occurrence_identity_value(&value["owner_occurrence_identity"])
                    .map_err(malformed)?,
                required_string(value, "parameter_name")
                    .map_err(malformed)?
                    .into(),
            ),
        )),
        "Builtin" => Ok(HldTargetIdentityValue::Symbol(
            NameSymbolTargetKey::Builtin(
                required_string(value, "builtin_name")
                    .map_err(malformed)?
                    .into(),
            ),
        )),
        "CrateRoot" => Ok(HldTargetIdentityValue::Symbol(
            NameSymbolTargetKey::CrateRoot(
                decode_output_unit_identity(&value["unit_compilation_identity"])
                    .map_err(malformed)?,
            ),
        )),
        "FilePublicArea" => {
            let unit = decode_output_unit_identity(&value["unit_compilation_identity"])
                .map_err(malformed)?;
            let path = required_string(value, "file_source_relative_path").map_err(malformed)?;
            let identity = FileAreaIdentityKey::build_file_area_identity(&unit, path)
                .map_err(|error| malformed(error.to_string()))?;
            Ok(HldTargetIdentityValue::FilePublicArea(identity))
        }
        _ => Err(malformed("unknown target identity kind".into())),
    }
}

fn decode_output_unit_identity(value: &Value) -> Result<CrateCompilationUnitKey, String> {
    let role = match required_string(value, "compilation_role_kind")? {
        "Host" => CompilationUnitRoleKind::Host,
        "Target" => CompilationUnitRoleKind::Target,
        _ => return Err("invalid persisted compilation role".into()),
    };
    let digest = decode_json_byte_array(&value["configuration_digest_full"])?;
    Ok(CrateCompilationUnitKey {
        unit_cargo_package_identifier: required_string(value, "cargo_package_identifier")?.into(),
        unit_cargo_target_name: required_string(value, "cargo_target_name")?.into(),
        unit_cargo_target_kind: required_string(value, "cargo_target_kind")?.into(),
        unit_compilation_role_kind: role,
        unit_configuration_digest_full: digest,
    })
}

fn decode_scope_unit_identity(value: &Value) -> Result<CrateCompilationUnitKey, String> {
    let role = match required_string(value, "unit_compilation_role_kind")? {
        "host" | "Host" => CompilationUnitRoleKind::Host,
        "target" | "Target" => CompilationUnitRoleKind::Target,
        _ => return Err("invalid namespace compilation role".into()),
    };
    let digest_hex = required_string(value, "unit_configuration_digest_full")?;
    let digest = decode_hex_digest(digest_hex)?;
    Ok(CrateCompilationUnitKey {
        unit_cargo_package_identifier: required_string(value, "unit_cargo_package_identifier")?
            .into(),
        unit_cargo_target_name: required_string(value, "unit_cargo_target_name")?.into(),
        unit_cargo_target_kind: required_string(value, "unit_cargo_target_kind")?.into(),
        unit_compilation_role_kind: role,
        unit_configuration_digest_full: digest,
    })
}

fn decode_occurrence_identity_value(value: &Value) -> Result<SemanticOccurrenceKeyValue, String> {
    let unit = decode_output_unit_identity(&value["unit_compilation_identity"])?;
    let path = required_string(value, "source_file_path")?;
    let range = value["byte_range"]
        .as_array()
        .filter(|range| range.len() == 2)
        .ok_or_else(|| "invalid occurrence byte range".to_owned())?;
    let start = range[0]
        .as_u64()
        .ok_or_else(|| "invalid occurrence start".to_owned())?;
    let end = range[1]
        .as_u64()
        .ok_or_else(|| "invalid occurrence end".to_owned())?;
    let range = SpanByteRangeValue::try_from((start, end)).map_err(|error| error.to_string())?;
    let digest = SpanDigestIdValue::from(decode_json_byte_array(&value["content_digest_full"])?);
    let expansion: Vec<[u8; 32]> =
        serde_json::from_value(value["expansion_provenance_chain"].clone())
            .map_err(|error| error.to_string())?;
    SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        &unit, path, range, digest, &expansion,
    )
    .map_err(|error| error.to_string())
}

fn decode_evidence_identity_value(
    value: &Value,
    row: &StageTableRowRecord,
) -> Result<HldEvidenceRowKey, HldGraphInputStoreError> {
    let malformed = |reason: String| {
        malformed_graph_row(
            &row.stage_owned_table_name,
            &row.stage_row_record_key,
            reason,
        )
    };
    Ok(HldEvidenceRowKey {
        evidence_site_occurrence_key: decode_occurrence_identity_value(
            &value["site_occurrence_identity"],
        )
        .map_err(malformed)?,
        evidence_source_occurrence_key: decode_target_identity_value(
            &value["source_target_identity"],
            row,
        )?,
        evidence_relation_kind_value: decode_relation_value(&value["relation_kind"])
            .map_err(malformed)?,
        evidence_target_reference_value: if value["target_identity"].is_null() {
            None
        } else {
            Some(decode_target_identity_value(
                &value["target_identity"],
                row,
            )?)
        },
    })
}

fn encode_edge_resolution_value(value: &HldEdgeResolutionValue) -> Value {
    match value {
        HldEdgeResolutionValue::Extracted { target } => json!({
            "kind":"Extracted",
            "target":super::hld_graph_publication_turso::encode_target_identity_json(target)
        }),
        HldEdgeResolutionValue::Unresolved { reason } => json!({
            "kind":"Unresolved",
            "reason":encode_unresolved_reason_value(*reason)
        }),
        HldEdgeResolutionValue::Inferred { candidate_target } => json!({
            "kind":"Inferred",
            "candidate_target":candidate_target.as_ref().map(super::hld_graph_publication_turso::encode_target_identity_json)
        }),
        HldEdgeResolutionValue::Proposed { candidate_target } => json!({
            "kind":"Proposed",
            "candidate_target":candidate_target.as_ref().map(super::hld_graph_publication_turso::encode_target_identity_json)
        }),
    }
}

fn decode_edge_resolution_value(
    value: &Value,
    row: &StageTableRowRecord,
) -> Result<HldEdgeResolutionValue, HldGraphInputStoreError> {
    let kind = required_string(value, "kind").map_err(|reason| {
        malformed_graph_row(
            &row.stage_owned_table_name,
            &row.stage_row_record_key,
            reason,
        )
    })?;
    match kind {
        "Extracted" => Ok(HldEdgeResolutionValue::Extracted {
            target: decode_target_identity_value(&value["target"], row)?,
        }),
        "Unresolved" => Ok(HldEdgeResolutionValue::Unresolved {
            reason: decode_unresolved_reason_value(&value["reason"]).map_err(|reason| {
                malformed_graph_row(
                    &row.stage_owned_table_name,
                    &row.stage_row_record_key,
                    reason,
                )
            })?,
        }),
        "Inferred" | "Proposed" => {
            let candidate_target = if value["candidate_target"].is_null() {
                None
            } else {
                Some(decode_target_identity_value(
                    &value["candidate_target"],
                    row,
                )?)
            };
            if kind == "Inferred" {
                Ok(HldEdgeResolutionValue::Inferred { candidate_target })
            } else {
                Ok(HldEdgeResolutionValue::Proposed { candidate_target })
            }
        }
        _ => Err(malformed_graph_row(
            &row.stage_owned_table_name,
            &row.stage_row_record_key,
            "unknown edge resolution kind",
        )),
    }
}

fn decode_generation_digest_text(value: &str) -> Result<RunGenerationDigestValue, String> {
    decode_hex_digest(value).map(RunGenerationDigestValue)
}

fn decode_hex_digest(value: &str) -> Result<[u8; 32], String> {
    if value.len() != 64 || !value.is_ascii() {
        return Err("invalid full-length digest".into());
    }
    (0..32)
        .map(|index| {
            u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| "invalid digest length".into())
}

fn decode_json_byte_array(value: &Value) -> Result<[u8; 32], String> {
    let bytes =
        serde_json::from_value::<Vec<u8>>(value.clone()).map_err(|error| error.to_string())?;
    bytes
        .try_into()
        .map_err(|_| "invalid 32-byte digest array".into())
}

fn encode_node_category_value(value: HldNodeInputCategory) -> &'static str {
    match value {
        HldNodeInputCategory::LocalItem => "LocalItem",
        HldNodeInputCategory::FilePublicArea => "FilePublicArea",
        HldNodeInputCategory::CrateRoot => "CrateRoot",
        HldNodeInputCategory::ExternalDefinition => "ExternalDefinition",
        HldNodeInputCategory::Builtin => "Builtin",
    }
}

fn decode_node_category_value(value: &Value) -> Result<HldNodeInputCategory, String> {
    match value
        .as_str()
        .ok_or_else(|| "node category is not a string".to_owned())?
    {
        "LocalItem" => Ok(HldNodeInputCategory::LocalItem),
        "FilePublicArea" => Ok(HldNodeInputCategory::FilePublicArea),
        "CrateRoot" => Ok(HldNodeInputCategory::CrateRoot),
        "ExternalDefinition" => Ok(HldNodeInputCategory::ExternalDefinition),
        "Builtin" => Ok(HldNodeInputCategory::Builtin),
        _ => Err("unknown node category".into()),
    }
}

fn encode_item_kind_value(value: HldItemKindValue) -> &'static str {
    match value {
        HldItemKindValue::Union => "Union",
        HldItemKindValue::MacroDefinition => "MacroDefinition",
        HldItemKindValue::MacroInvocation => "MacroInvocation",
        HldItemKindValue::ExternBlock => "ExternBlock",
        HldItemKindValue::ExternCrate => "ExternCrate",
        HldItemKindValue::InlineAssembly => "InlineAssembly",
        HldItemKindValue::FilePublicArea => "FilePublicArea",
        HldItemKindValue::Module => "Module",
        HldItemKindValue::Function => "Function",
        HldItemKindValue::Struct => "Struct",
        HldItemKindValue::Enum => "Enum",
        HldItemKindValue::Trait => "Trait",
        HldItemKindValue::TypeAlias => "TypeAlias",
        HldItemKindValue::Constant => "Constant",
        HldItemKindValue::Static => "Static",
        HldItemKindValue::AssociatedFunction => "AssociatedFunction",
        HldItemKindValue::AssociatedConstant => "AssociatedConstant",
        HldItemKindValue::AssociatedType => "AssociatedType",
        HldItemKindValue::EnumVariant => "EnumVariant",
        HldItemKindValue::ImplContainer => "ImplContainer",
        HldItemKindValue::UseStatement => "UseStatement",
        HldItemKindValue::Field => "Field",
        HldItemKindValue::FunctionBodyItem => "FunctionBodyItem",
        HldItemKindValue::GenericParameter => "GenericParameter",
        HldItemKindValue::Unknown => "Unknown",
    }
}

fn decode_optional_item_kind_value(value: &Value) -> Result<Option<HldItemKindValue>, String> {
    if value.is_null() {
        return Ok(None);
    }
    let value = match value
        .as_str()
        .ok_or_else(|| "item kind is not a string".to_owned())?
    {
        "FilePublicArea" => HldItemKindValue::FilePublicArea,
        "Union" => HldItemKindValue::Union,
        "MacroDefinition" => HldItemKindValue::MacroDefinition,
        "MacroInvocation" => HldItemKindValue::MacroInvocation,
        "ExternBlock" => HldItemKindValue::ExternBlock,
        "ExternCrate" => HldItemKindValue::ExternCrate,
        "InlineAssembly" => HldItemKindValue::InlineAssembly,
        "Module" => HldItemKindValue::Module,
        "Function" => HldItemKindValue::Function,
        "Struct" => HldItemKindValue::Struct,
        "Enum" => HldItemKindValue::Enum,
        "Trait" => HldItemKindValue::Trait,
        "TypeAlias" => HldItemKindValue::TypeAlias,
        "Constant" => HldItemKindValue::Constant,
        "Static" => HldItemKindValue::Static,
        "AssociatedFunction" => HldItemKindValue::AssociatedFunction,
        "AssociatedConstant" => HldItemKindValue::AssociatedConstant,
        "AssociatedType" => HldItemKindValue::AssociatedType,
        "EnumVariant" => HldItemKindValue::EnumVariant,
        "ImplContainer" => HldItemKindValue::ImplContainer,
        "UseStatement" => HldItemKindValue::UseStatement,
        "Field" => HldItemKindValue::Field,
        "FunctionBodyItem" => HldItemKindValue::FunctionBodyItem,
        "GenericParameter" => HldItemKindValue::GenericParameter,
        "Unknown" => HldItemKindValue::Unknown,
        _ => return Err("unknown item kind".into()),
    };
    Ok(Some(value))
}

fn encode_owner_kind_value(value: HldOwnerKindValue) -> &'static str {
    match value {
        HldOwnerKindValue::ExternBlock => "ExternBlock",
        HldOwnerKindValue::CrateRoot => "CrateRoot",
        HldOwnerKindValue::Module => "Module",
        HldOwnerKindValue::Trait => "Trait",
        HldOwnerKindValue::Impl => "Impl",
        HldOwnerKindValue::FunctionBody => "FunctionBody",
        HldOwnerKindValue::Other => "Other",
    }
}

fn decode_optional_owner_kind_value(value: &Value) -> Result<Option<HldOwnerKindValue>, String> {
    if value.is_null() {
        return Ok(None);
    }
    match value
        .as_str()
        .ok_or_else(|| "owner kind is not a string".to_owned())?
    {
        "CrateRoot" => Ok(Some(HldOwnerKindValue::CrateRoot)),
        "ExternBlock" => Ok(Some(HldOwnerKindValue::ExternBlock)),
        "Module" => Ok(Some(HldOwnerKindValue::Module)),
        "Trait" => Ok(Some(HldOwnerKindValue::Trait)),
        "Impl" => Ok(Some(HldOwnerKindValue::Impl)),
        "FunctionBody" => Ok(Some(HldOwnerKindValue::FunctionBody)),
        "Other" => Ok(Some(HldOwnerKindValue::Other)),
        _ => Err("unknown owner kind".into()),
    }
}

fn encode_visibility_value(value: HldEffectiveVisibilityKind) -> &'static str {
    match value {
        HldEffectiveVisibilityKind::Public => "Public",
        HldEffectiveVisibilityKind::RestrictedNonPrivate => "RestrictedNonPrivate",
        HldEffectiveVisibilityKind::PrivateEquivalent => "PrivateEquivalent",
        HldEffectiveVisibilityKind::Unknown => "Unknown",
    }
}

fn decode_optional_visibility_value(
    value: &Value,
) -> Result<Option<HldEffectiveVisibilityKind>, String> {
    if value.is_null() {
        return Ok(None);
    }
    match value
        .as_str()
        .ok_or_else(|| "visibility is not a string".to_owned())?
    {
        "Public" => Ok(Some(HldEffectiveVisibilityKind::Public)),
        "RestrictedNonPrivate" => Ok(Some(HldEffectiveVisibilityKind::RestrictedNonPrivate)),
        "PrivateEquivalent" => Ok(Some(HldEffectiveVisibilityKind::PrivateEquivalent)),
        "Unknown" => Ok(Some(HldEffectiveVisibilityKind::Unknown)),
        _ => Err("unknown visibility kind".into()),
    }
}

fn encode_relation_value(value: HldRelationKindValue) -> &'static str {
    match value {
        HldRelationKindValue::Calls => "Calls",
        HldRelationKindValue::UsesType => "UsesType",
        HldRelationKindValue::Imports => "Imports",
        HldRelationKindValue::Reads => "Reads",
        HldRelationKindValue::Implements => "Implements",
        HldRelationKindValue::Reexports => "Reexports",
        HldRelationKindValue::Contains => "Contains",
        HldRelationKindValue::Derives => "Derives",
        HldRelationKindValue::Generates => "Generates",
    }
}

fn decode_relation_value(value: &Value) -> Result<HldRelationKindValue, String> {
    match value
        .as_str()
        .ok_or_else(|| "relation kind is not a string".to_owned())?
    {
        "Calls" => Ok(HldRelationKindValue::Calls),
        "UsesType" => Ok(HldRelationKindValue::UsesType),
        "Imports" => Ok(HldRelationKindValue::Imports),
        "Reads" => Ok(HldRelationKindValue::Reads),
        "Implements" => Ok(HldRelationKindValue::Implements),
        "Reexports" => Ok(HldRelationKindValue::Reexports),
        "Contains" => Ok(HldRelationKindValue::Contains),
        "Derives" => Ok(HldRelationKindValue::Derives),
        "Generates" => Ok(HldRelationKindValue::Generates),
        _ => Err("unknown HLD relation".into()),
    }
}

fn encode_unresolved_reason_value(value: HldUnresolvedReasonKind) -> &'static str {
    match value {
        HldUnresolvedReasonKind::MissingTarget => "MissingTarget",
        HldUnresolvedReasonKind::AmbiguousTarget => "AmbiguousTarget",
        HldUnresolvedReasonKind::CfgInactive => "CfgInactive",
        HldUnresolvedReasonKind::ExternalSourceMissing => "ExternalSourceMissing",
        HldUnresolvedReasonKind::TraitSolverNeeded => "TraitSolverNeeded",
        HldUnresolvedReasonKind::Other => "Other",
    }
}

fn decode_unresolved_reason_value(value: &Value) -> Result<HldUnresolvedReasonKind, String> {
    match value
        .as_str()
        .ok_or_else(|| "unresolved reason is not a string".to_owned())?
    {
        "MissingTarget" => Ok(HldUnresolvedReasonKind::MissingTarget),
        "AmbiguousTarget" => Ok(HldUnresolvedReasonKind::AmbiguousTarget),
        "CfgInactive" => Ok(HldUnresolvedReasonKind::CfgInactive),
        "ExternalSourceMissing" => Ok(HldUnresolvedReasonKind::ExternalSourceMissing),
        "TraitSolverNeeded" => Ok(HldUnresolvedReasonKind::TraitSolverNeeded),
        "Other" => Ok(HldUnresolvedReasonKind::Other),
        _ => Err("unknown unresolved reason".into()),
    }
}

fn mapping_scope_text(coverage: HldSourceCoverageKind) -> &'static str {
    match coverage {
        HldSourceCoverageKind::FullSemantic => "full-semantic",
        HldSourceCoverageKind::OriginalTopLevelContainment => "original-top-level-containment",
    }
}

fn coverage_text(coverage: HldSourceCoverageKind) -> &'static str {
    match coverage {
        HldSourceCoverageKind::FullSemantic => "FullSemantic",
        HldSourceCoverageKind::OriginalTopLevelContainment => "original-file-top-level-only",
    }
}

fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("required string field {field} is absent or malformed"))
}

fn required_bool(value: &Value, field: &str) -> Result<bool, String> {
    value
        .get(field)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("required boolean field {field} is absent or malformed"))
}

fn bool_field_or_default(value: &Value, field: &str, default: bool) -> Result<bool, String> {
    match value.get(field) {
        None => Ok(default),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(format!("{field} must be a boolean when present")),
    }
}

fn optional_bool(value: &Value) -> Result<Option<bool>, String> {
    if value.is_null() {
        Ok(None)
    } else {
        value
            .as_bool()
            .map(Some)
            .ok_or_else(|| "optional boolean field is malformed".into())
    }
}

fn optional_string(value: &Value) -> Result<Option<String>, String> {
    if value.is_null() {
        Ok(None)
    } else {
        value
            .as_str()
            .map(str::to_owned)
            .map(Some)
            .ok_or_else(|| "optional string field is malformed".into())
    }
}

fn malformed_graph_row(
    table: &str,
    record_key: &str,
    reason: impl Into<String>,
) -> HldGraphInputStoreError {
    HldGraphInputStoreError::MalformedPersistedRow {
        table: table.into(),
        record_key: record_key.into(),
        reason: reason.into(),
    }
}

fn map_graph_snapshot_error(error: HldPublicationPortError) -> HldGraphInputStoreError {
    match error {
        HldPublicationPortError::UnfrozenSourceSnapshot => {
            HldGraphInputStoreError::UnfrozenSourceSnapshot
        }
        HldPublicationPortError::MissingInputStageMarker { stage_name } => {
            HldGraphInputStoreError::MissingStageMarker { stage_name }
        }
        HldPublicationPortError::StaleInputSnapshot { stage_name } => {
            HldGraphInputStoreError::StaleStageMarker { stage_name }
        }
        HldPublicationPortError::CoverageScopeMismatch => {
            HldGraphInputStoreError::CoverageScopeMismatch
        }
        HldPublicationPortError::InvalidFrozenWorkspaceConfig => {
            HldGraphInputStoreError::InvalidFrozenWorkspaceConfig
        }
        other => HldGraphInputStoreError::SourceSnapshotValidation(other),
    }
}

fn check_graph_input_cancelled(
    cancel_flag: &RunCancelFlagValue,
) -> Result<(), HldGraphInputStoreError> {
    if cancel_flag.check_stage_work_cancelled() {
        Err(HldGraphInputStoreError::Cancelled)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod full_graph_codec_tests {
    use super::*;

    /// REQ-RUST-045.0
    #[test]
    fn test_full_item_codec_roundtrip() {
        for kind in [
            HldItemKindValue::Union,
            HldItemKindValue::MacroDefinition,
            HldItemKindValue::MacroInvocation,
            HldItemKindValue::ExternBlock,
            HldItemKindValue::ExternCrate,
            HldItemKindValue::InlineAssembly,
        ] {
            assert_eq!(
                decode_optional_item_kind_value(&json!(encode_item_kind_value(kind))),
                Ok(Some(kind))
            );
        }
        let owner = HldOwnerKindValue::ExternBlock;
        assert_eq!(
            decode_optional_owner_kind_value(&json!(encode_owner_kind_value(owner))),
            Ok(Some(owner))
        );
    }
}
