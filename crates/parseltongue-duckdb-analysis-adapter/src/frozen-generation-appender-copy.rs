use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StoreBackendFailureError,
};
use parseltongue_table_store_traits::StageTableStorePort;
use std::collections::{BTreeMap, BTreeSet};

fn check_cancel(cancel: &RunCancelFlagValue) -> Result<(), StoreBackendFailureError> {
    if cancel.check_stage_work_cancelled() {
        Err(build_analysis_error("analysis copy cancelled"))
    } else {
        Ok(())
    }
}

pub(super) fn hash_record(hasher: &mut blake3::Hasher, row: &StageTableRowRecord) {
    [
        &row.stage_owned_table_name,
        &row.stage_row_record_key,
        &row.stage_row_payload_text,
    ]
    .into_iter()
    .for_each(|value| {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    });
}

fn table_digest(
    connection: &duckdb::Connection,
    scope: &StageRunScopeValue,
    table: &str,
) -> Result<(usize, String), StoreBackendFailureError> {
    let mut statement = connection.prepare(
        "SELECT row_key,payload FROM \"frozen-generation-source-records\" WHERE generation_key=? AND attempt_key=? AND table_name=? ORDER BY generation_key,attempt_key,table_name,row_key",
    ).map_err(build_analysis_error)?;
    let mut rows = statement
        .query(duckdb::params![
            scope.run_generation_digest_value,
            scope.stage_attempt_sequence_value,
            table
        ])
        .map_err(build_analysis_error)?;
    let mut hasher = blake3::Hasher::new();
    let mut count = 0;
    while let Some(row) = rows.next().map_err(build_analysis_error)? {
        hash_record(
            &mut hasher,
            &StageTableRowRecord {
                stage_owned_table_name: table.into(),
                stage_row_record_key: row.get(0).map_err(build_analysis_error)?,
                stage_row_payload_text: row.get(1).map_err(build_analysis_error)?,
            },
        );
        count += 1;
    }
    Ok((count, hasher.finalize().to_hex().to_string()))
}

pub(super) fn copy_generation(
    store: &DuckdbAnalysisStoreAdapter,
    source: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    tables: &[&str],
    cancel: &RunCancelFlagValue,
) -> Result<usize, StoreBackendFailureError> {
    check_cancel(cancel)?;
    if source.read_workspace_publication_scope()?.as_ref() != Some(scope)
        && source.read_analysis_input_publication_scope()?.as_ref() != Some(scope)
    {
        return Err(build_analysis_error("source generation is not published"));
    }
    let mut tables = tables.to_vec();
    tables.sort_unstable();
    if tables.is_empty() || tables.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(build_analysis_error(
            "copy requires a nonempty unique table set",
        ));
    }
    let mut connection = store._connection.lock().map_err(build_analysis_error)?;
    let transaction = connection.transaction().map_err(build_analysis_error)?;
    transaction
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS \"frozen-generation-source-records\" (
          generation_key VARCHAR NOT NULL, attempt_key VARCHAR NOT NULL,
          table_name VARCHAR NOT NULL, row_key VARCHAR NOT NULL, payload VARCHAR NOT NULL,
          PRIMARY KEY(generation_key,attempt_key,table_name,row_key));
         CREATE TABLE IF NOT EXISTS \"frozen-generation-table-manifests\" (
          generation_key VARCHAR NOT NULL, attempt_key VARCHAR NOT NULL,
          table_name VARCHAR NOT NULL,row_count UBIGINT NOT NULL,content_digest VARCHAR NOT NULL,
          PRIMARY KEY(generation_key,attempt_key,table_name))",
        )
        .map_err(build_analysis_error)?;
    let existing: usize = transaction.query_row(
        "SELECT count(*) FROM \"frozen-generation-table-manifests\" WHERE generation_key=? AND attempt_key=?",
        duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value], |row| row.get(0),
    ).map_err(build_analysis_error)?;
    if existing != 0 {
        return Err(build_analysis_error("analysis generation is immutable"));
    }
    let count = tables.into_iter().try_fold(0, |total, table| {
        check_cancel(cancel)?;
        let expected_count = source.count_stage_table_rows(scope, table)?;
        eprintln!("stage=namespace-copy-table table={table} event=appending expected_rows={expected_count}");
        let mut hasher = blake3::Hasher::new();
        let mut previous_key: Option<String> = None;
        let mut count = 0;
        let mut budget=super::namespace_appender_batch_budget::NamespaceAppenderBatchBudget::default();
        {
            let mut appender = transaction
                .appender("frozen-generation-source-records")
                .map_err(build_analysis_error)?;
            source.visit_stage_table_rows(scope, table, &mut |row| {
                check_cancel(cancel)?;
                if row.stage_owned_table_name != table
                    || previous_key
                        .as_ref()
                        .is_some_and(|key| key >= &row.stage_row_record_key)
                {
                    return Err(build_analysis_error(
                        "source rows must have a unique total-key order",
                    ));
                }
                let row=super::namespace_source_column_projection::project_namespace_source_record(row)?;
                appender
                    .append_row(duckdb::params![
                        scope.run_generation_digest_value,
                        scope.stage_attempt_sequence_value,
                        table,
                        row.stage_row_record_key,
                        row.stage_row_payload_text
                    ])
                    .map_err(build_analysis_error)?;
                if budget.record_appender_row_budget(row.stage_row_payload_text.len()+row.stage_row_record_key.len()+table.len()+scope.run_generation_digest_value.len()+scope.stage_attempt_sequence_value.len()) {appender.flush().map_err(build_analysis_error)?;}
                hash_record(&mut hasher, &row);
                previous_key = Some(row.stage_row_record_key);
                count += 1;
                Ok(())
            })?;
            appender.flush().map_err(build_analysis_error)?;
        }
        let digest = hasher.finalize().to_hex().to_string();
        eprintln!("stage=namespace-copy-table table={table} event=verifying rows={count}");
        if expected_count != count
            || table_digest(&transaction, scope, table)? != (count, digest.clone())
        {
            return Err(build_analysis_error(
                "Turso/DuckDB row-count or content-digest mismatch",
            ));
        }
        transaction
            .execute(
                "INSERT INTO \"frozen-generation-table-manifests\" VALUES (?,?,?,?,?)",
                duckdb::params![
                    scope.run_generation_digest_value,
                    scope.stage_attempt_sequence_value,
                    table,
                    count as u64,
                    digest
                ],
            )
            .map_err(build_analysis_error)?;
        eprintln!("stage=namespace-copy-table table={table} event=verified rows={count}");
        Ok(total + count)
    })?;
    check_cancel(cancel)?;
    if source.read_workspace_publication_scope()?.as_ref() != Some(scope)
        && source.read_analysis_input_publication_scope()?.as_ref() != Some(scope)
    {
        return Err(build_analysis_error(
            "source publication changed during copy",
        ));
    }
    transaction.commit().map_err(build_analysis_error)?;
    connection
        .execute_batch("CHECKPOINT")
        .map_err(build_analysis_error)?;
    Ok(count)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VerifiedSourceTableManifest {
    source_table_manifest_name: String,
    source_table_row_count: u64,
    source_table_content_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FrozenSourceManifestSnapshot {
    run_generation_identity_key: String,
    frozen_attempt_identity_key: String,
    frozen_source_table_manifests: Vec<VerifiedSourceTableManifest>,
}

impl DuckdbAnalysisStoreAdapter {
    pub fn frozen_source_input_digest(
        &self,
        scope: &StageRunScopeValue,
    ) -> Result<String, StoreBackendFailureError> {
        read_source_manifest_digest(self, scope)
    }
}

fn read_source_manifest_digest(
    store: &DuckdbAnalysisStoreAdapter,
    scope: &StageRunScopeValue,
) -> Result<String, StoreBackendFailureError> {
    let connection = store._connection.lock().map_err(build_analysis_error)?;
    let mut statement = connection
        .prepare(
            "SELECT table_name,row_count,content_digest FROM \"frozen-generation-table-manifests\" WHERE generation_key=? AND attempt_key=? ORDER BY table_name",
        )
        .map_err(build_analysis_error)?;
    let verified_source_manifest_rows = statement
        .query_map(
            duckdb::params![
                scope.run_generation_digest_value,
                scope.stage_attempt_sequence_value
            ],
            |row| {
                Ok(VerifiedSourceTableManifest {
                    source_table_manifest_name: row.get(0)?,
                    source_table_row_count: row.get(1)?,
                    source_table_content_digest: row.get(2)?,
                })
            },
        )
        .map_err(build_analysis_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(build_analysis_error)?;
    let expected_source_table_names = super::NAMESPACE_SOURCE_TABLE_NAMES
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let actual_source_table_names = verified_source_manifest_rows
        .iter()
        .map(|manifest| manifest.source_table_manifest_name.as_str())
        .collect::<BTreeSet<_>>();
    if verified_source_manifest_rows.len() != expected_source_table_names.len()
        || actual_source_table_names != expected_source_table_names
    {
        return Err(build_analysis_error(
            "verified frozen namespace source manifests are incomplete",
        ));
    }
    hash_source_input_digest(&FrozenSourceManifestSnapshot {
        run_generation_identity_key: scope.run_generation_digest_value.clone(),
        frozen_attempt_identity_key: scope.stage_attempt_sequence_value.clone(),
        frozen_source_table_manifests: verified_source_manifest_rows,
    })
}

fn hash_source_input_digest(
    snapshot: &FrozenSourceManifestSnapshot,
) -> Result<String, StoreBackendFailureError> {
    let _snapshot_identity = (
        &snapshot.run_generation_identity_key,
        &snapshot.frozen_attempt_identity_key,
    );
    let source_manifest_name_map = snapshot
        .frozen_source_table_manifests
        .iter()
        .map(|manifest| (manifest.source_table_manifest_name.as_str(), manifest))
        .collect::<BTreeMap<_, _>>();
    if source_manifest_name_map.is_empty()
        || source_manifest_name_map.len() != snapshot.frozen_source_table_manifests.len()
    {
        return Err(build_analysis_error(
            "source input digest requires unique table manifests",
        ));
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"namespace-source-input-v1\0");
    source_manifest_name_map
        .into_iter()
        .for_each(|(table_name, manifest)| {
            hasher.update(&(table_name.len() as u64).to_le_bytes());
            hasher.update(table_name.as_bytes());
            hasher.update(&manifest.source_table_row_count.to_le_bytes());
            hasher.update(&(manifest.source_table_content_digest.len() as u64).to_le_bytes());
            hasher.update(manifest.source_table_content_digest.as_bytes());
        });
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod source_input_digest_tests {
    use super::{
        FrozenSourceManifestSnapshot, StoreBackendFailureError, VerifiedSourceTableManifest,
        hash_source_input_digest,
    };

    /// REQ-RUST-028.1
    #[test]
    fn test_source_digest_snapshot_stable() -> Result<(), StoreBackendFailureError> {
        let source_manifests = vec![
            VerifiedSourceTableManifest {
                source_table_manifest_name: "syntax-namespace-input-items".into(),
                source_table_row_count: 3,
                source_table_content_digest: "a".repeat(64),
            },
            VerifiedSourceTableManifest {
                source_table_manifest_name: "crate-dependency-graph-table".into(),
                source_table_row_count: 1,
                source_table_content_digest: "b".repeat(64),
            },
        ];
        let first = FrozenSourceManifestSnapshot {
            run_generation_identity_key: "generation-one".into(),
            frozen_attempt_identity_key: "attempt-one".into(),
            frozen_source_table_manifests: source_manifests.clone(),
        };
        let second = FrozenSourceManifestSnapshot {
            run_generation_identity_key: "generation-two".into(),
            frozen_attempt_identity_key: "attempt-two".into(),
            frozen_source_table_manifests: source_manifests.iter().cloned().rev().collect(),
        };
        let changed = FrozenSourceManifestSnapshot {
            run_generation_identity_key: "generation-two".into(),
            frozen_attempt_identity_key: "attempt-three".into(),
            frozen_source_table_manifests: source_manifests
                .iter()
                .cloned()
                .map(|mut manifest| {
                    if manifest.source_table_manifest_name == "syntax-namespace-input-items" {
                        manifest.source_table_content_digest = "c".repeat(64);
                    }
                    manifest
                })
                .collect(),
        };
        let first_digest = hash_source_input_digest(&first)?;
        let second_digest = hash_source_input_digest(&second)?;
        let changed_digest = hash_source_input_digest(&changed)?;
        assert_eq!(first_digest, second_digest);
        assert_ne!(first_digest, changed_digest);
        Ok(())
    }

    /// REQ-RUST-028.1
    #[test]
    fn test_source_digest_empty_rejected() {
        let snapshot = FrozenSourceManifestSnapshot {
            run_generation_identity_key: "generation-one".into(),
            frozen_attempt_identity_key: "attempt-one".into(),
            frozen_source_table_manifests: Vec::new(),
        };
        assert!(hash_source_input_digest(&snapshot).is_err());
    }

    /// REQ-RUST-028.1
    #[test]
    fn test_source_digest_duplicates_rejected() {
        let manifest = VerifiedSourceTableManifest {
            source_table_manifest_name: "syntax-namespace-input-items".into(),
            source_table_row_count: 3,
            source_table_content_digest: "a".repeat(64),
        };
        let snapshot = FrozenSourceManifestSnapshot {
            run_generation_identity_key: "generation-one".into(),
            frozen_attempt_identity_key: "attempt-one".into(),
            frozen_source_table_manifests: vec![manifest.clone(), manifest],
        };
        assert!(hash_source_input_digest(&snapshot).is_err());
    }
}
