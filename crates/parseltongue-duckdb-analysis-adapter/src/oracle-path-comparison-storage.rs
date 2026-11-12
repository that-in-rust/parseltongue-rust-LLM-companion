use super::OracleComparisonFailureError;
use super::oracle_path_comparison_core::{
    CanonicalPathSiteValue, canonicalize_site_evidence_rows, read_required_text_value,
    validate_compilation_unit_rows,
};
use duckdb::{Connection, params};
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StageUnitWriteRequest,
};
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

const SOURCE_GRAPH_STAGE_NAME: &str = "crate-dependency-graph-table";
const SOURCE_PATH_STAGE_NAME: &str = "path-resolution-site-records";
const AGREEMENT_STAGE_TABLE_NAME: &str = "oracle-edge-agreement-table";

pub fn compare_oracle_path_sites(
    source: &dyn StageTableStorePort,
    primary: &Path,
    secondary: &Path,
    agreement: &dyn StageTableStorePort,
    cancel: &RunCancelFlagValue,
) -> Result<Value, OracleComparisonFailureError> {
    compare_oracle_path_sites_with_threads(source, primary, secondary, agreement, cancel, 1)
}

pub(super) fn compare_oracle_path_sites_with_threads(
    source: &dyn StageTableStorePort,
    primary: &Path,
    secondary: &Path,
    agreement: &dyn StageTableStorePort,
    cancel: &RunCancelFlagValue,
    threads: usize,
) -> Result<Value, OracleComparisonFailureError> {
    check_comparison_cancellation_flag(cancel)?;
    let source_scope = source
        .read_workspace_publication_scope()?
        .ok_or(OracleComparisonFailureError::SourceNotPublished)?;
    if agreement.read_workspace_publication_scope()?.as_ref() == Some(&source_scope) {
        return Err(OracleComparisonFailureError::CannotMutateSource);
    }
    let graph_rows = read_verified_source_table(source, &source_scope, SOURCE_GRAPH_STAGE_NAME)?;
    let path_rows = read_verified_source_table(source, &source_scope, SOURCE_PATH_STAGE_NAME)?;
    if source.read_workspace_publication_scope()?.as_ref() != Some(&source_scope) {
        return Err(OracleComparisonFailureError::SourceChanged);
    }
    let units = graph_rows
        .iter()
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text))
        .collect::<Result<Vec<_>, _>>()?;
    let mapped_unit_count = validate_compilation_unit_rows(&units)?;
    let (primary_envelope, primary_rows, primary_digest) =
        read_validated_oracle_envelope(primary, "PRIMARY", &source_scope, mapped_unit_count)?;
    let (secondary_envelope, secondary_rows, secondary_digest) =
        read_validated_oracle_envelope(secondary, "SECONDARY", &source_scope, mapped_unit_count)?;
    check_comparison_cancellation_flag(cancel)?;
    let production_oracle_site_rows = canonicalize_site_evidence_rows(
        &parse_persisted_source_payloads(&path_rows)?,
        &units,
        false,
    )?;
    let primary_oracle_site_rows = canonicalize_site_evidence_rows(&primary_rows, &units, true)?;
    let secondary_oracle_site_rows =
        canonicalize_site_evidence_rows(&secondary_rows, &units, true)?;
    let connection = build_isolated_comparison_connection(threads)?;
    let source_digest =
        verify_duckdb_source_copy(&connection, &source_scope, &graph_rows, &path_rows, cancel)?;
    load_canonical_comparison_rows(
        &connection,
        &source_scope,
        &production_oracle_site_rows,
        &primary_oracle_site_rows,
        &secondary_oracle_site_rows,
    )?;
    score_canonical_comparison_rows(&connection)?;
    check_comparison_cancellation_flag(cancel)?;
    if source.read_workspace_publication_scope()?.as_ref() != Some(&source_scope) {
        return Err(OracleComparisonFailureError::SourceChanged);
    }
    let (mut site_results, mut result) = read_canonical_comparison_results(
        &connection,
        &source_scope,
        &source_digest,
        &primary_digest,
        &secondary_digest,
    )?;
    if primary_envelope["oracle_configuration_kind_value"] != "PRIMARY"
        || secondary_envelope["oracle_configuration_kind_value"] != "SECONDARY"
    {
        return Err(OracleComparisonFailureError::InvalidEnvelope {
            configuration: "oracle".into(),
            field: "oracle_configuration_kind_value".into(),
        });
    }
    publish_completed_agreement_generation(
        agreement,
        &source_scope,
        &source_digest,
        &primary_digest,
        &secondary_digest,
        &mut site_results,
        &mut result,
        source,
        cancel,
    )?;
    Ok(result)
}

fn check_comparison_cancellation_flag(
    cancel: &RunCancelFlagValue,
) -> Result<(), OracleComparisonFailureError> {
    if cancel.check_stage_work_cancelled() {
        Err(OracleComparisonFailureError::Cancelled)
    } else {
        Ok(())
    }
}

fn read_validated_oracle_envelope(
    path: &Path,
    expected_kind: &str,
    source_scope: &StageRunScopeValue,
    mapped_unit_count: usize,
) -> Result<(Value, Vec<Value>, String), OracleComparisonFailureError> {
    let bytes = fs::read(path).map_err(|source| OracleComparisonFailureError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    let envelope: Value = serde_json::from_slice(&bytes)?;
    let configuration = read_required_text_value(&envelope, "oracle_configuration_kind_value")?;
    if configuration != expected_kind {
        return Err(OracleComparisonFailureError::InvalidEnvelope {
            configuration: expected_kind.into(),
            field: "oracle_configuration_kind_value".into(),
        });
    }
    let expected_build_scripts = expected_kind == "SECONDARY";
    for (field, expected) in [
        ("oracle_build_scripts_flag", expected_build_scripts),
        ("oracle_proc_macros_flag", true),
    ] {
        if let Some(value) = envelope.get(field)
            && value.as_bool() != Some(expected)
        {
            return Err(OracleComparisonFailureError::InvalidEnvelope {
                configuration: expected_kind.into(),
                field: field.into(),
            });
        }
    }
    let manifest_scope = envelope
        .get("oracle_manifest_scope_record")
        .ok_or_else(|| OracleComparisonFailureError::InvalidEnvelope {
            configuration: expected_kind.into(),
            field: "oracle_manifest_scope_record".into(),
        })?;
    if manifest_scope["run_generation_digest_value"] != source_scope.run_generation_digest_value
        || manifest_scope["stage_attempt_sequence_value"]
            != source_scope.stage_attempt_sequence_value
    {
        return Err(OracleComparisonFailureError::ScopeMismatch {
            configuration: expected_kind.into(),
        });
    }
    if envelope["oracle_mapped_unit_count"].as_u64() != Some(mapped_unit_count as u64) {
        return Err(OracleComparisonFailureError::InvalidEnvelope {
            configuration: expected_kind.into(),
            field: "oracle_mapped_unit_count".into(),
        });
    }
    let array_fields = [
        "oracle_path_export_rows",
        "oracle_proc_macro_rows",
        "oracle_source_omission_rows",
        "oracle_cached_script_rows",
        "oracle_build_script_deltas",
    ];
    for field in array_fields {
        if !envelope.get(field).is_some_and(Value::is_array) {
            return Err(OracleComparisonFailureError::InvalidEnvelope {
                configuration: expected_kind.into(),
                field: field.into(),
            });
        }
    }
    if !envelope["oracle_source_omission_rows"]
        .as_array()
        .is_some_and(Vec::is_empty)
    {
        return Err(OracleComparisonFailureError::InvalidEnvelope {
            configuration: expected_kind.into(),
            field: "oracle_source_omission_rows".into(),
        });
    }
    let macro_loading_rows = envelope["oracle_proc_macro_rows"]
        .as_array()
        .ok_or_else(|| OracleComparisonFailureError::InvalidEnvelope {
            configuration: expected_kind.into(),
            field: "oracle_proc_macro_rows".into(),
        })?;
    for row in macro_loading_rows {
        let malformed = |field: &str| OracleComparisonFailureError::MalformedMacroLoadingEvidence {
            field: field.into(),
        };
        let source_root = row
            .get("macro_source_root_path")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| malformed("macro_source_root_path"))?;
        let status = row
            .get("macro_loading_status_value")
            .and_then(Value::as_str)
            .ok_or_else(|| malformed("macro_loading_status_value"))?;
        let origin = row
            .get("macro_loading_origin_kind")
            .and_then(Value::as_str)
            .ok_or_else(|| malformed("macro_loading_origin_kind"))?;
        if !matches!(origin, "cached" | "workspace") {
            return Err(malformed("macro_loading_origin_kind"));
        }
        let error = row
            .get("macro_loading_error_text")
            .ok_or_else(|| malformed("macro_loading_error_text"))?;
        match status {
            "loaded" if error.is_null() => {}
            "loaded" => return Err(malformed("macro_loading_error_text")),
            "failed" => {
                let error_detail = error
                    .as_str()
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| malformed("macro_loading_error_text"))?;
                return Err(OracleComparisonFailureError::ProcMacroLoadingFailed {
                    source_root: source_root.into(),
                    error_detail: error_detail.into(),
                });
            }
            _ => return Err(malformed("macro_loading_status_value")),
        }
    }
    let rows = envelope["oracle_path_export_rows"]
        .as_array()
        .cloned()
        .ok_or_else(|| OracleComparisonFailureError::InvalidEnvelope {
            configuration: expected_kind.into(),
            field: "oracle_path_export_rows".into(),
        })?;
    Ok((envelope, rows, digest))
}

fn read_verified_source_table(
    source: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    table: &str,
) -> Result<Vec<StageTableRowRecord>, OracleComparisonFailureError> {
    let mut rows = source.read_stage_table_rows(scope, table)?;
    let count = source.count_stage_table_rows(scope, table)?;
    if rows.len() != count || rows.iter().any(|row| row.stage_owned_table_name != table) {
        return Err(OracleComparisonFailureError::InvalidSourceRows {
            table: table.to_owned(),
        });
    }
    rows.sort_by(|left, right| left.stage_row_record_key.cmp(&right.stage_row_record_key));
    if rows
        .windows(2)
        .any(|pair| pair[0].stage_row_record_key == pair[1].stage_row_record_key)
    {
        let key = rows
            .windows(2)
            .find(|pair| pair[0].stage_row_record_key == pair[1].stage_row_record_key)
            .map(|pair| pair[0].stage_row_record_key.clone())
            .unwrap_or_default();
        return Err(OracleComparisonFailureError::DuplicateSourceRow {
            table: table.to_owned(),
            key,
        });
    }
    Ok(rows)
}

fn parse_persisted_source_payloads(
    rows: &[StageTableRowRecord],
) -> Result<Vec<Value>, OracleComparisonFailureError> {
    rows.iter()
        .map(|row| serde_json::from_str(&row.stage_row_payload_text).map_err(Into::into))
        .collect()
}

fn hash_framed_source_value(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value.as_bytes());
}

fn digest_persisted_source_rows(
    source_scope: &StageRunScopeValue,
    rows: &[StageTableRowRecord],
) -> String {
    let mut hasher = blake3::Hasher::new();
    for row in rows {
        hash_framed_source_value(&mut hasher, &source_scope.run_generation_digest_value);
        hash_framed_source_value(&mut hasher, &source_scope.stage_attempt_sequence_value);
        hash_framed_source_value(&mut hasher, &row.stage_owned_table_name);
        hash_framed_source_value(&mut hasher, &row.stage_row_record_key);
        hash_framed_source_value(&mut hasher, &row.stage_row_payload_text);
    }
    hasher.finalize().to_hex().to_string()
}

fn verify_duckdb_source_copy(
    connection: &Connection,
    source_scope: &StageRunScopeValue,
    graph_rows: &[StageTableRowRecord],
    path_rows: &[StageTableRowRecord],
    cancel: &RunCancelFlagValue,
) -> Result<String, OracleComparisonFailureError> {
    connection
        .execute_batch(
            "CREATE TEMP TABLE oracle_source_copy_rows (source_scope_generation_value VARCHAR, source_scope_attempt_value VARCHAR, table_name VARCHAR, row_key VARCHAR, payload VARCHAR)",
        )
        .map_err(OracleComparisonFailureError::Duckdb)?;
    let expected_by_table = BTreeMap::from([
        (SOURCE_GRAPH_STAGE_NAME.to_owned(), graph_rows),
        (SOURCE_PATH_STAGE_NAME.to_owned(), path_rows),
    ]);
    let expected = expected_by_table
        .values()
        .flat_map(|rows| rows.iter().cloned())
        .collect::<Vec<_>>();
    let expected_digest = digest_persisted_source_rows(source_scope, &expected);
    {
        let mut appender = connection
            .appender("oracle_source_copy_rows")
            .map_err(OracleComparisonFailureError::Duckdb)?;
        for row in &expected {
            check_comparison_cancellation_flag(cancel)?;
            appender
                .append_row(params![
                    source_scope.run_generation_digest_value,
                    source_scope.stage_attempt_sequence_value,
                    row.stage_owned_table_name,
                    row.stage_row_record_key,
                    row.stage_row_payload_text,
                ])
                .map_err(OracleComparisonFailureError::Duckdb)?;
        }
        appender
            .flush()
            .map_err(OracleComparisonFailureError::Duckdb)?;
    }
    let total_count: usize = connection
        .query_row("SELECT count(*) FROM oracle_source_copy_rows", [], |row| {
            row.get(0)
        })
        .map_err(OracleComparisonFailureError::Duckdb)?;
    let scoped_count: usize = connection
        .query_row(
            "SELECT count(*) FROM oracle_source_copy_rows WHERE source_scope_generation_value=? AND source_scope_attempt_value=?",
            params![
                source_scope.run_generation_digest_value,
                source_scope.stage_attempt_sequence_value
            ],
            |row| row.get(0),
        )
        .map_err(OracleComparisonFailureError::Duckdb)?;
    if total_count != expected.len() || scoped_count != expected.len() {
        return Err(OracleComparisonFailureError::SourceCopyCountMismatch);
    }
    expected_by_table.iter().try_for_each(|(table, expected_rows)| {
        let table_count: usize = connection
            .query_row(
                "SELECT count(*) FROM oracle_source_copy_rows WHERE source_scope_generation_value=? AND source_scope_attempt_value=? AND table_name=?",
                params![
                    source_scope.run_generation_digest_value,
                    source_scope.stage_attempt_sequence_value,
                    table
                ],
                |row| row.get(0),
            )
            .map_err(OracleComparisonFailureError::Duckdb)?;
        if table_count != expected_rows.len() {
            return Err(OracleComparisonFailureError::SourceCopyCountMismatch);
        }
        let mut hasher = blake3::Hasher::new();
        let mut statement = connection
            .prepare(
                "SELECT source_scope_generation_value,source_scope_attempt_value,table_name,row_key,payload FROM oracle_source_copy_rows WHERE source_scope_generation_value=? AND source_scope_attempt_value=? AND table_name=? ORDER BY source_scope_generation_value,source_scope_attempt_value,table_name,row_key",
            )
            .map_err(OracleComparisonFailureError::Duckdb)?;
        let mut rows = statement
            .query(params![
                source_scope.run_generation_digest_value,
                source_scope.stage_attempt_sequence_value,
                table
            ])
            .map_err(OracleComparisonFailureError::Duckdb)?;
        let mut copied_count = 0usize;
        while let Some(row) = rows.next().map_err(OracleComparisonFailureError::Duckdb)? {
            for index in 0..5 {
                let value: String = row.get(index).map_err(OracleComparisonFailureError::Duckdb)?;
                hash_framed_source_value(&mut hasher, &value);
            }
            copied_count += 1;
        }
        let copied_digest = hasher.finalize().to_hex().to_string();
        if copied_count != expected_rows.len()
            || copied_digest != digest_persisted_source_rows(source_scope, expected_rows)
        {
            return Err(OracleComparisonFailureError::SourceCopyDigestMismatch);
        }
        Ok(())
    })?;
    let mut hasher = blake3::Hasher::new();
    let mut statement = connection
        .prepare("SELECT source_scope_generation_value,source_scope_attempt_value,table_name,row_key,payload FROM oracle_source_copy_rows ORDER BY source_scope_generation_value,source_scope_attempt_value,table_name,row_key")
        .map_err(OracleComparisonFailureError::Duckdb)?;
    let mut rows = statement
        .query([])
        .map_err(OracleComparisonFailureError::Duckdb)?;
    while let Some(row) = rows.next().map_err(OracleComparisonFailureError::Duckdb)? {
        for index in 0..5 {
            let value: String = row
                .get(index)
                .map_err(OracleComparisonFailureError::Duckdb)?;
            hash_framed_source_value(&mut hasher, &value);
        }
    }
    let copied_digest = hasher.finalize().to_hex().to_string();
    if copied_digest != expected_digest {
        return Err(OracleComparisonFailureError::SourceCopyDigestMismatch);
    }
    Ok(expected_digest)
}

fn build_isolated_comparison_connection(
    threads: usize,
) -> Result<Connection, OracleComparisonFailureError> {
    if !matches!(threads, 1 | 2) {
        return Err(OracleComparisonFailureError::UnsupportedThreadCount { threads });
    }
    let config = duckdb::Config::default()
        .enable_external_access(false)
        .and_then(|config| config.with("autoinstall_known_extensions", "false"))
        .and_then(|config| config.with("autoload_known_extensions", "false"))
        .map_err(OracleComparisonFailureError::Duckdb)?;
    let connection = Connection::open_in_memory_with_flags(config)
        .map_err(OracleComparisonFailureError::Duckdb)?;
    connection
        .execute_batch(&format!("SET threads = {threads};"))
        .map_err(OracleComparisonFailureError::Duckdb)?;
    Ok(connection)
}

fn append_canonical_site_rows(
    connection: &Connection,
    table: &str,
    source_scope: &StageRunScopeValue,
    sites: &[CanonicalPathSiteValue],
) -> Result<(), OracleComparisonFailureError> {
    let mut appender = connection
        .appender(table)
        .map_err(OracleComparisonFailureError::Duckdb)?;
    for site in sites {
        appender
            .append_row(params![
                source_scope.run_generation_digest_value,
                source_scope.stage_attempt_sequence_value,
                site.site_key,
                site.unit_identity,
                site.classification,
                site.classification_reason,
                site.unresolved_reason,
                site.targets_json,
                site.targets_empty,
                site.proc_macro_dependent,
                site.duplicate_count as u64,
                site.identity_json,
            ])
            .map_err(OracleComparisonFailureError::Duckdb)?;
    }
    appender
        .flush()
        .map_err(OracleComparisonFailureError::Duckdb)
}

fn load_canonical_comparison_rows(
    connection: &Connection,
    source_scope: &StageRunScopeValue,
    production: &[CanonicalPathSiteValue],
    primary: &[CanonicalPathSiteValue],
    secondary: &[CanonicalPathSiteValue],
) -> Result<(), OracleComparisonFailureError> {
    connection.execute_batch(
        "CREATE TEMP TABLE production_oracle_site_rows (source_scope_generation_value VARCHAR, source_scope_attempt_value VARCHAR, site_key VARCHAR, unit_identity VARCHAR, classification VARCHAR, classification_reason VARCHAR, unresolved_reason VARCHAR, targets_json VARCHAR, targets_empty BOOLEAN, proc_macro_dependent BOOLEAN, duplicate_count UBIGINT, identity_json VARCHAR);
         CREATE TEMP TABLE primary_oracle_site_rows AS SELECT * FROM production_oracle_site_rows WHERE false;
         CREATE TEMP TABLE secondary_oracle_site_rows AS SELECT * FROM production_oracle_site_rows WHERE false;
         CREATE TEMP TABLE oracle_comparison_evidence_rows (source_scope_generation_value VARCHAR, source_scope_attempt_value VARCHAR, site_key VARCHAR, classification VARCHAR, classification_reason VARCHAR, primary_reason VARCHAR, production_reason VARCHAR, secondary_reason VARCHAR, authoritative_reason VARCHAR, primary_targets VARCHAR, production_targets VARCHAR, secondary_targets VARCHAR, status VARCHAR, partition_kind VARCHAR, partition_reason VARCHAR, secondary_changed BOOLEAN, production_only BOOLEAN, secondary_only BOOLEAN, primary_duplicates UBIGINT, production_duplicates UBIGINT, secondary_duplicates UBIGINT, identity_json VARCHAR);",
    ).map_err(OracleComparisonFailureError::Duckdb)?;
    append_canonical_site_rows(
        connection,
        "production_oracle_site_rows",
        source_scope,
        production,
    )?;
    append_canonical_site_rows(
        connection,
        "primary_oracle_site_rows",
        source_scope,
        primary,
    )?;
    append_canonical_site_rows(
        connection,
        "secondary_oracle_site_rows",
        source_scope,
        secondary,
    )?;
    Ok(())
}

fn score_canonical_comparison_rows(
    connection: &Connection,
) -> Result<(), OracleComparisonFailureError> {
    connection.execute_batch(
        "INSERT INTO oracle_comparison_evidence_rows
         WITH joined AS (
             SELECT COALESCE(p.source_scope_generation_value,d.source_scope_generation_value,s.source_scope_generation_value) AS source_scope_generation_value,
                    COALESCE(p.source_scope_attempt_value,d.source_scope_attempt_value,s.source_scope_attempt_value) AS source_scope_attempt_value,
                    p.site_key AS primary_key, d.site_key AS production_key, s.site_key AS secondary_key,
                    COALESCE(p.site_key,d.site_key,s.site_key) AS site_key,
                    p.classification, p.classification_reason,
                    COALESCE(p.unresolved_reason,'') AS primary_reason,
                    COALESCE(d.unresolved_reason,'') AS production_reason,
                    COALESCE(s.unresolved_reason,'') AS secondary_reason,
                    p.targets_json AS primary_targets,
                    d.targets_json AS production_targets, s.targets_json AS secondary_targets,
                    p.targets_empty AS primary_empty, d.targets_empty AS production_empty,
                    COALESCE(p.proc_macro_dependent,FALSE) OR COALESCE(d.proc_macro_dependent,FALSE) AS proc_macro_dependent,
                    COALESCE(p.duplicate_count,0) AS primary_duplicates,
                    COALESCE(d.duplicate_count,0) AS production_duplicates,
                    COALESCE(s.duplicate_count,0) AS secondary_duplicates,
                    COALESCE(p.identity_json,d.identity_json,s.identity_json) AS identity_json
             FROM primary_oracle_site_rows p
             FULL OUTER JOIN production_oracle_site_rows d ON p.source_scope_generation_value=d.source_scope_generation_value AND p.source_scope_attempt_value=d.source_scope_attempt_value AND p.site_key=d.site_key
             FULL OUTER JOIN secondary_oracle_site_rows s ON COALESCE(p.source_scope_generation_value,d.source_scope_generation_value)=s.source_scope_generation_value AND COALESCE(p.source_scope_attempt_value,d.source_scope_attempt_value)=s.source_scope_attempt_value AND COALESCE(p.site_key,d.site_key)=s.site_key
         ), reasoned AS (
             SELECT *, CASE
                 WHEN 'cfg_inactive' IN (primary_reason,production_reason,secondary_reason) THEN 'cfg_inactive'
                 WHEN 'build_script' IN (primary_reason,production_reason,secondary_reason) THEN 'build_script'
                 WHEN 'proc_macro' IN (primary_reason,production_reason,secondary_reason) THEN 'proc_macro'
                 WHEN 'external_source_missing' IN (primary_reason,production_reason,secondary_reason) THEN 'external_source_missing'
                 WHEN 'unknown_receiver' IN (primary_reason,production_reason,secondary_reason) THEN 'unknown_receiver'
                 WHEN 'trait_solver_needed' IN (primary_reason,production_reason,secondary_reason) THEN 'trait_solver_needed'
                 ELSE '' END AS authoritative_reason
             FROM joined
         ), evaluated AS (
             SELECT *,
                 CASE
                   WHEN primary_key IS NULL AND production_key IS NOT NULL AND secondary_key IS NOT NULL THEN 'production-secondary-only'
                   WHEN primary_key IS NULL AND secondary_key IS NOT NULL THEN 'secondary-only'
                   WHEN primary_key IS NULL AND production_key IS NOT NULL THEN 'production-only'
                   WHEN classification='blocked' THEN 'blocked'
                   WHEN classification='type-relative' THEN 'deferred'
                   WHEN primary_empty AND production_key IS NOT NULL AND NOT production_empty THEN 'mismatch'
                   WHEN primary_targets <> production_targets AND production_empty AND proc_macro_dependent THEN 'proc-excluded'
                   WHEN primary_targets <> production_targets THEN 'mismatch'
                   WHEN primary_reason='cfg_inactive' THEN 'cfg-excluded'
                   WHEN primary_empty AND proc_macro_dependent THEN 'proc-excluded'
                   WHEN primary_empty THEN 'unresolved'
                   WHEN production_key IS NULL AND proc_macro_dependent THEN 'proc-excluded'
                   WHEN production_key IS NULL THEN 'missing-production'
                   WHEN proc_macro_dependent THEN 'proc-excluded'
                   ELSE 'match'
                 END AS status,
                 (primary_key IS NULL AND production_key IS NOT NULL) AS production_only,
                 (primary_key IS NULL AND secondary_key IS NOT NULL) AS secondary_only
             FROM reasoned
         ), partitioned AS (
             SELECT *, CASE status
                 WHEN 'deferred' THEN 'associated-deferred'
                 WHEN 'blocked' THEN 'unknown-blocked'
                 WHEN 'cfg-excluded' THEN 'cfg-excluded'
                 WHEN 'proc-excluded' THEN 'proc-excluded'
                 WHEN 'production-only' THEN 'non-primary-production-only'
                 WHEN 'secondary-only' THEN 'non-primary-secondary-only'
                 WHEN 'production-secondary-only' THEN 'non-primary-production-secondary-only'
                 ELSE 'module-scope-scored' END AS partition_kind,
                 CASE status
                 WHEN 'deferred' THEN COALESCE(classification_reason,'type_relative')
                 WHEN 'blocked' THEN COALESCE(classification_reason,'unknown_lookup_scope')
                 WHEN 'cfg-excluded' THEN 'cfg_inactive'
                 WHEN 'proc-excluded' THEN 'proc_macro'
                 ELSE COALESCE(NULLIF(authoritative_reason,''),classification_reason,'') END AS partition_reason
             FROM evaluated
         )
         SELECT source_scope_generation_value, source_scope_attempt_value, site_key, COALESCE(classification,'production'), COALESCE(classification_reason,'non_primary_site'),
                primary_reason, production_reason, secondary_reason, authoritative_reason,
                primary_targets, production_targets, secondary_targets, status, partition_kind, partition_reason,
                COALESCE(primary_key IS NOT NULL AND secondary_key IS NOT NULL AND primary_targets <> secondary_targets,FALSE),
                production_only, secondary_only, primary_duplicates, production_duplicates, secondary_duplicates, identity_json
         FROM partitioned ORDER BY source_scope_generation_value,source_scope_attempt_value,site_key",
    ).map_err(OracleComparisonFailureError::Duckdb)?;
    Ok(())
}

struct PathComparisonResultStats {
    eligible: u64,
    matched: u64,
    deferred: u64,
    blocked: u64,
    production_only: u64,
    secondary_only: u64,
    secondary_changed: u64,
    proc_excluded: u64,
    cfg_excluded: u64,
    primary_raw: u64,
    primary_unique: u64,
    primary_duplicates: u64,
    module_scope_raw: u64,
    type_relative_raw: u64,
    cfg_inactive_raw: u64,
    blocked_raw: u64,
    proc_macro_raw: u64,
    unresolved: u64,
    missing_production: u64,
    mismatched: u64,
}

fn read_canonical_comparison_results(
    connection: &Connection,
    source_scope: &StageRunScopeValue,
    source_digest: &str,
    primary_digest: &str,
    secondary_digest: &str,
) -> Result<(Vec<Value>, Value), OracleComparisonFailureError> {
    let stats = connection.query_row(
        "SELECT count(*) FILTER (WHERE primary_duplicates>0 AND partition_kind='module-scope-scored' AND status IN ('match','mismatch','unresolved','missing-production')),
                count(*) FILTER (WHERE primary_duplicates>0 AND status='match'),
                count(*) FILTER (WHERE primary_duplicates>0 AND status='deferred'),
                count(*) FILTER (WHERE status='blocked'),
                count(*) FILTER (WHERE production_only),
                count(*) FILTER (WHERE secondary_only),
                count(*) FILTER (WHERE secondary_changed),
                count(*) FILTER (WHERE primary_duplicates>0 AND status='proc-excluded'),
                count(*) FILTER (WHERE primary_duplicates>0 AND status='cfg-excluded'),
                COALESCE(sum(primary_duplicates),0),
                count(*) FILTER (WHERE primary_duplicates>0),
                COALESCE(sum(CASE WHEN primary_duplicates>0 THEN primary_duplicates-1 ELSE 0 END),0),
                COALESCE(sum(primary_duplicates) FILTER (WHERE classification='module-scope'),0),
                COALESCE(sum(primary_duplicates) FILTER (WHERE classification='type-relative'),0),
                COALESCE(sum(primary_duplicates) FILTER (WHERE partition_kind='cfg-excluded'),0),
                COALESCE(sum(primary_duplicates) FILTER (WHERE partition_kind='unknown-blocked'),0),
                COALESCE(sum(primary_duplicates) FILTER (WHERE partition_kind='proc-excluded'),0),
                count(*) FILTER (WHERE primary_duplicates>0 AND status='unresolved'),
                count(*) FILTER (WHERE primary_duplicates>0 AND status='missing-production'),
                count(*) FILTER (WHERE primary_duplicates>0 AND status='mismatch')
         FROM oracle_comparison_evidence_rows WHERE source_scope_generation_value=? AND source_scope_attempt_value=?",
        params![
            source_scope.run_generation_digest_value,
            source_scope.stage_attempt_sequence_value
        ],
        |row| {
            Ok(PathComparisonResultStats {
                eligible: row.get(0)?,
                matched: row.get(1)?,
                deferred: row.get(2)?,
                blocked: row.get(3)?,
                production_only: row.get(4)?,
                secondary_only: row.get(5)?,
                secondary_changed: row.get(6)?,
                proc_excluded: row.get(7)?,
                cfg_excluded: row.get(8)?,
                primary_raw: row.get(9)?,
                primary_unique: row.get(10)?,
                primary_duplicates: row.get(11)?,
                module_scope_raw: row.get(12)?,
                type_relative_raw: row.get(13)?,
                cfg_inactive_raw: row.get(14)?,
                blocked_raw: row.get(15)?,
                proc_macro_raw: row.get(16)?,
                unresolved: row.get(17)?,
                missing_production: row.get(18)?,
                mismatched: row.get(19)?,
            })
        },
    ).map_err(OracleComparisonFailureError::Duckdb)?;
    let rate = (stats.blocked == 0 && stats.eligible != 0)
        .then(|| stats.matched as f64 / stats.eligible as f64);
    let status = if stats.blocked != 0 {
        "blocked"
    } else if stats.eligible == 0 {
        "not-applicable"
    } else if stats.eligible == stats.matched {
        "match"
    } else {
        "mismatch"
    };
    let mut primary_reason_counts = BTreeMap::new();
    let mut production_reason_counts = BTreeMap::new();
    let mut authoritative_reason_counts = BTreeMap::new();
    for (field, counts) in [
        ("primary_reason", &mut primary_reason_counts),
        ("production_reason", &mut production_reason_counts),
        ("authoritative_reason", &mut authoritative_reason_counts),
    ] {
        let query = format!(
            "SELECT COALESCE(NULLIF({field},''),'none'), SUM({}) FROM oracle_comparison_evidence_rows WHERE source_scope_generation_value=? AND source_scope_attempt_value=? AND {}>0 GROUP BY 1 ORDER BY 1",
            if field == "production_reason" {
                "production_duplicates"
            } else {
                "primary_duplicates"
            },
            if field == "production_reason" {
                "production_duplicates"
            } else {
                "primary_duplicates"
            },
        );
        let mut statement = connection
            .prepare(&query)
            .map_err(OracleComparisonFailureError::Duckdb)?;
        let mut rows = statement
            .query(params![
                source_scope.run_generation_digest_value,
                source_scope.stage_attempt_sequence_value
            ])
            .map_err(OracleComparisonFailureError::Duckdb)?;
        while let Some(row) = rows.next().map_err(OracleComparisonFailureError::Duckdb)? {
            counts.insert(
                row.get::<_, String>(0)
                    .map_err(OracleComparisonFailureError::Duckdb)?,
                row.get::<_, u64>(1)
                    .map_err(OracleComparisonFailureError::Duckdb)?,
            );
        }
    }
    let mut partition_counts = BTreeMap::new();
    let mut partition_statement = connection.prepare(
        "SELECT partition_kind, SUM(primary_duplicates) FROM oracle_comparison_evidence_rows WHERE source_scope_generation_value=? AND source_scope_attempt_value=? AND primary_duplicates>0 GROUP BY partition_kind ORDER BY partition_kind",
    ).map_err(OracleComparisonFailureError::Duckdb)?;
    let mut partition_rows = partition_statement
        .query(params![
            source_scope.run_generation_digest_value,
            source_scope.stage_attempt_sequence_value
        ])
        .map_err(OracleComparisonFailureError::Duckdb)?;
    while let Some(row) = partition_rows
        .next()
        .map_err(OracleComparisonFailureError::Duckdb)?
    {
        partition_counts.insert(
            row.get::<_, String>(0)
                .map_err(OracleComparisonFailureError::Duckdb)?,
            row.get::<_, u64>(1)
                .map_err(OracleComparisonFailureError::Duckdb)?,
        );
    }
    let partition_raw_total = partition_counts.values().sum::<u64>();
    let partition_reconciled = partition_raw_total == stats.primary_raw;
    let mut statement = connection.prepare(
        "SELECT site_key,classification,classification_reason,primary_reason,production_reason,secondary_reason,authoritative_reason,primary_targets,production_targets,secondary_targets,status,partition_kind,partition_reason,secondary_changed,production_only,secondary_only,primary_duplicates,production_duplicates,secondary_duplicates,identity_json FROM oracle_comparison_evidence_rows WHERE source_scope_generation_value=? AND source_scope_attempt_value=? ORDER BY source_scope_generation_value,source_scope_attempt_value,site_key",
    ).map_err(OracleComparisonFailureError::Duckdb)?;
    let mut rows = statement
        .query(params![
            source_scope.run_generation_digest_value,
            source_scope.stage_attempt_sequence_value
        ])
        .map_err(OracleComparisonFailureError::Duckdb)?;
    let mut evidence = Vec::new();
    while let Some(row) = rows.next().map_err(OracleComparisonFailureError::Duckdb)? {
        evidence.push(json!({
            "agreement_site_key":row.get::<_,String>(0).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_classification_kind":row.get::<_,String>(1).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_classification_reason_kind":row.get::<_,String>(2).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_primary_unresolved_reason_kind":row.get::<_,String>(3).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_production_unresolved_reason_kind":row.get::<_,String>(4).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_secondary_unresolved_reason_kind":row.get::<_,String>(5).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_authoritative_causal_reason_kind":row.get::<_,String>(6).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_primary_targets_json":row.get::<_,Option<String>>(7).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_production_targets_json":row.get::<_,Option<String>>(8).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_secondary_targets_json":row.get::<_,Option<String>>(9).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_site_status_kind":row.get::<_,String>(10).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_partition_kind":row.get::<_,String>(11).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_partition_reason_kind":row.get::<_,String>(12).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_secondary_changed_flag":row.get::<_,bool>(13).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_production_only_flag":row.get::<_,bool>(14).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_secondary_only_flag":row.get::<_,bool>(15).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_primary_duplicate_count":row.get::<_,u64>(16).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_production_duplicate_count":row.get::<_,u64>(17).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_secondary_duplicate_count":row.get::<_,u64>(18).map_err(OracleComparisonFailureError::Duckdb)?,
            "agreement_site_identity_json":row.get::<_,String>(19).map_err(OracleComparisonFailureError::Duckdb)?,
        }));
    }
    let result = json!({
        "agreement_status_kind":status,
        "agreement_denominator_status_kind":if stats.blocked != 0 {"BLOCKED"} else if stats.eligible == 0 {"N/A"} else {"scored"},
        "agreement_eligible_path_count":stats.eligible,
        "agreement_match_count_value":stats.matched,
        "agreement_exact_match_rate":rate,
        "agreement_match_rate":rate,
        "agreement_deferred_path_count":stats.deferred,
        "agreement_blocked_path_count":stats.blocked,
        "agreement_production_only_path_count":stats.production_only,
        "agreement_secondary_only_path_count":stats.secondary_only,
        "agreement_secondary_only_count":stats.secondary_only,
        "agreement_secondary_changed_path_count":stats.secondary_changed,
        "agreement_proc_excluded_path_count":stats.proc_excluded,
        "agreement_proc_excluded_count":stats.proc_excluded,
        "agreement_cfg_excluded_path_count":stats.cfg_excluded,
        "agreement_primary_raw_site_count":stats.primary_raw,
        "agreement_primary_unique_site_count":stats.primary_unique,
        "agreement_primary_exact_duplicate_count":stats.primary_duplicates,
        "agreement_module_scope_raw_site_count":stats.module_scope_raw,
        "agreement_type_relative_raw_site_count":stats.type_relative_raw,
        "agreement_cfg_inactive_raw_site_count":stats.cfg_inactive_raw,
        "agreement_blocked_raw_site_count":stats.blocked_raw,
        "agreement_proc_macro_raw_site_count":stats.proc_macro_raw,
        "agreement_unresolved_path_count":stats.unresolved,
        "agreement_missing_production_path_count":stats.missing_production,
        "agreement_mismatched_path_count":stats.mismatched,
        "agreement_primary_reason_counts":primary_reason_counts,
        "agreement_production_reason_counts":production_reason_counts,
        "agreement_authoritative_reason_counts":authoritative_reason_counts,
        "agreement_reason_counts":primary_reason_counts,
        "agreement_primary_raw_partition_counts":partition_counts,
        "agreement_primary_partition_reconciled_flag":partition_reconciled,
        "agreement_source_generation_digest_value":source_scope.run_generation_digest_value,
        "agreement_source_stage_attempt_sequence_value":source_scope.stage_attempt_sequence_value,
        "agreement_source_framed_digest_full":source_digest,
        "agreement_primary_oracle_digest_full":primary_digest,
        "agreement_secondary_oracle_digest_full":secondary_digest,
    });
    Ok((evidence, result))
}

fn write_completed_agreement_unit(
    agreement: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    owner: &str,
    work_key: &str,
    inputs: &[&str],
    rows: &[StageTableRowRecord],
    cancel: &RunCancelFlagValue,
) -> Result<(), OracleComparisonFailureError> {
    check_comparison_cancellation_flag(cancel)?;
    agreement.write_stage_table_unit(
        &StageUnitWriteRequest {
            stage_run_scope_value: scope,
            stage_owner_name_value: owner,
            stage_unit_work_key: work_key,
            stage_input_stage_names: inputs,
            stage_unit_output_rows: rows,
        },
        cancel,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn publish_completed_agreement_generation(
    agreement: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    source_digest: &str,
    primary_digest: &str,
    secondary_digest: &str,
    evidence: &mut Vec<Value>,
    result: &mut Value,
    source: &dyn StageTableStorePort,
    cancel: &RunCancelFlagValue,
) -> Result<(), OracleComparisonFailureError> {
    check_comparison_cancellation_flag(cancel)?;
    let seed = format!(
        "{}:{}:{}:{}:{}",
        source_scope.run_generation_digest_value,
        source_scope.stage_attempt_sequence_value,
        source_digest,
        primary_digest,
        secondary_digest,
    );
    let generation = format!(
        "oracle-agreement/{}",
        blake3::hash(seed.as_bytes()).to_hex()
    );
    let previous = agreement.read_workspace_publication_scope()?;
    let previous_identity = previous.map_or_else(
        || "initial".to_owned(),
        |scope| {
            format!(
                "{}:{}",
                scope.run_generation_digest_value, scope.stage_attempt_sequence_value
            )
        },
    );
    let attempt = blake3::hash(format!("{previous_identity}:{generation}").as_bytes())
        .to_hex()
        .to_string();
    let requested_scope = StageRunScopeValue {
        run_generation_digest_value: generation.clone(),
        stage_attempt_sequence_value: format!("oracle-agreement/{attempt}"),
    };
    let scope =
        agreement.prepare_workspace_generation_scope(&requested_scope, &generation, cancel)?;
    result["agreement_generation_digest_value"] = json!(generation);
    result["agreement_operational_identity"] = json!({
        "agreement_stage_attempt_sequence_value":scope.stage_attempt_sequence_value,
    });
    for row in evidence.iter_mut() {
        row["agreement_source_generation_digest_value"] =
            json!(source_scope.run_generation_digest_value);
        row["agreement_source_stage_attempt_sequence_value"] =
            json!(source_scope.stage_attempt_sequence_value);
        row["agreement_source_framed_digest_full"] = json!(source_digest);
        row["agreement_primary_oracle_digest_full"] = json!(primary_digest);
        row["agreement_secondary_oracle_digest_full"] = json!(secondary_digest);
    }
    let mut semantic_result = result.clone();
    if let Some(result_fields) = semantic_result.as_object_mut() {
        result_fields.remove("agreement_operational_identity");
        result_fields.remove("agreement_semantic_export_digest_full");
    }
    let semantic_export =
        json!({"agreement_evidence_rows":evidence,"agreement_result":semantic_result});
    result["agreement_semantic_export_digest_full"] = json!(
        blake3::hash(semantic_export.to_string().as_bytes())
            .to_hex()
            .to_string()
    );
    for (owner, inputs) in [
        ("crate-graph", &[][..]),
        ("source-ingest", &["crate-graph"][..]),
        ("syntax-spans", &["source-ingest"][..]),
    ] {
        write_completed_agreement_unit(
            agreement,
            &scope,
            owner,
            "__complete__",
            inputs,
            &[],
            cancel,
        )?;
    }
    let mut rows = evidence
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let key = value["agreement_site_key"]
                .as_str()
                .map(|site| format!("site/{site}"))
                .unwrap_or_else(|| format!("site/{index:020}"));
            StageTableRowRecord {
                stage_owned_table_name: AGREEMENT_STAGE_TABLE_NAME.into(),
                stage_row_record_key: key,
                stage_row_payload_text: value.to_string(),
            }
        })
        .collect::<Vec<_>>();
    let summary_key = "summary".to_owned();
    rows.push(StageTableRowRecord {
        stage_owned_table_name: AGREEMENT_STAGE_TABLE_NAME.into(),
        stage_row_record_key: summary_key,
        stage_row_payload_text: result.to_string(),
    });
    write_completed_agreement_unit(
        agreement,
        &scope,
        "oracle-agreement",
        "comparison",
        &["crate-graph", "source-ingest", "syntax-spans"],
        &rows,
        cancel,
    )?;
    write_completed_agreement_unit(
        agreement,
        &scope,
        "oracle-agreement",
        "__complete__",
        &[],
        &[],
        cancel,
    )?;
    if source.read_workspace_publication_scope()?.as_ref() != Some(source_scope) {
        return Err(OracleComparisonFailureError::SourceChanged);
    }
    check_comparison_cancellation_flag(cancel)?;
    agreement.publish_workspace_generation_manifest(&scope, &generation, cancel)?;
    evidence.sort_by(|left, right| {
        left["agreement_site_key"]
            .as_str()
            .cmp(&right["agreement_site_key"].as_str())
    });
    Ok(())
}

#[cfg(test)]
mod oracle_path_comparison_sql_tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn build_test_site_value(
        key: &str,
        classification: &str,
        targets: &[&str],
        proc_macro_dependent: bool,
        unresolved_reason: &str,
    ) -> CanonicalPathSiteValue {
        CanonicalPathSiteValue {
            site_key: key.into(),
            unit_identity: "fixture-unit".into(),
            classification: classification.into(),
            classification_reason: classification.into(),
            unresolved_reason: unresolved_reason.into(),
            targets_json: json!(targets).to_string(),
            targets_empty: targets.is_empty(),
            proc_macro_dependent,
            duplicate_count: 1,
            identity_json: json!({"site":key}).to_string(),
        }
    }

    fn build_test_comparison_result(
        threads: usize,
        production: &[CanonicalPathSiteValue],
        primary: &[CanonicalPathSiteValue],
        secondary: &[CanonicalPathSiteValue],
    ) -> (Vec<Value>, Value) {
        let source_scope = StageRunScopeValue {
            run_generation_digest_value: "generation".into(),
            stage_attempt_sequence_value: "attempt".into(),
        };
        let connection = build_isolated_comparison_connection(threads).unwrap();
        load_canonical_comparison_rows(&connection, &source_scope, production, primary, secondary)
            .unwrap();
        score_canonical_comparison_rows(&connection).unwrap();
        read_canonical_comparison_results(
            &connection,
            &source_scope,
            "source-digest",
            "primary-digest",
            "secondary-digest",
        )
        .unwrap()
    }

    /// REQ-RUST-025.2
    #[test]
    fn test_proc_wrong_target_mismatch() {
        let production = vec![build_test_site_value(
            "site-a",
            "module-scope",
            &["right"],
            false,
            "",
        )];
        let primary = vec![build_test_site_value(
            "site-a",
            "module-scope",
            &["wrong"],
            true,
            "proc_macro",
        )];
        let secondary = primary.clone();
        let (_, result) = build_test_comparison_result(1, &production, &primary, &secondary);
        assert_eq!(result["agreement_eligible_path_count"], 1);
        assert_eq!(result["agreement_match_count_value"], 0);
        assert_eq!(result["agreement_proc_excluded_count"], 0);
        assert_eq!(result["agreement_mismatched_path_count"], 1);
    }

    /// REQ-RUST-025.2
    #[test]
    fn test_proc_empty_answer_excluded() {
        let production = vec![build_test_site_value(
            "site-a",
            "production",
            &[],
            true,
            "proc_macro",
        )];
        let primary = vec![build_test_site_value(
            "site-a",
            "module-scope",
            &["oracle-target"],
            false,
            "",
        )];
        let secondary = primary.clone();
        let (evidence, result) = build_test_comparison_result(1, &production, &primary, &secondary);
        assert_eq!(result["agreement_eligible_path_count"], 0);
        assert_eq!(result["agreement_proc_excluded_path_count"], 1);
        assert_eq!(
            evidence[0]["agreement_production_unresolved_reason_kind"],
            "proc_macro"
        );
        assert_eq!(evidence[0]["agreement_partition_kind"], "proc-excluded");
    }

    /// REQ-RUST-025.2
    #[test]
    fn test_proc_empty_primary_exclusions() {
        let primary = vec![build_test_site_value(
            "site-a",
            "module-scope",
            &[],
            true,
            "proc_macro",
        )];
        let empty_production = vec![build_test_site_value(
            "site-a",
            "production",
            &[],
            false,
            "",
        )];

        let (empty_evidence, empty_result) =
            build_test_comparison_result(1, &empty_production, &primary, &[]);
        assert_eq!(empty_result["agreement_proc_excluded_path_count"], 1);
        assert_eq!(empty_result["agreement_unresolved_path_count"], 0);
        assert_eq!(
            empty_evidence[0]["agreement_site_status_kind"],
            "proc-excluded"
        );

        let (missing_evidence, missing_result) =
            build_test_comparison_result(1, &[], &primary, &[]);
        assert_eq!(missing_result["agreement_proc_excluded_path_count"], 1);
        assert_eq!(missing_result["agreement_unresolved_path_count"], 0);
        assert_eq!(
            missing_evidence[0]["agreement_site_status_kind"],
            "proc-excluded"
        );
    }

    /// REQ-RUST-025.2
    #[test]
    fn test_extra_production_secondary_counts() {
        let production = vec![build_test_site_value(
            "site-a",
            "production",
            &["target"],
            false,
            "",
        )];
        let secondary = vec![build_test_site_value(
            "site-a",
            "module-scope",
            &["target"],
            false,
            "",
        )];
        let (evidence, result) = build_test_comparison_result(1, &production, &[], &secondary);
        assert_eq!(result["agreement_production_only_path_count"], 1);
        assert_eq!(result["agreement_secondary_only_path_count"], 1);
        assert_eq!(evidence[0]["agreement_production_only_flag"], true);
        assert_eq!(evidence[0]["agreement_secondary_only_flag"], true);
    }

    /// REQ-RUST-025.2, REQ-RUST-025.3
    #[test]
    fn test_blocked_partition_reconciles_duplicates() {
        let mut blocked =
            build_test_site_value("blocked", "blocked", &[], false, "unknown_receiver");
        blocked.duplicate_count = 2;
        let (evidence, result) = build_test_comparison_result(1, &[], &[blocked], &[]);
        assert_eq!(result["agreement_primary_raw_site_count"], 2);
        assert_eq!(
            result["agreement_primary_raw_partition_counts"]["unknown-blocked"],
            2
        );
        assert_eq!(result["agreement_primary_partition_reconciled_flag"], true);
        assert_eq!(result["agreement_exact_match_rate"], Value::Null);
        assert_eq!(
            evidence[0]["agreement_classification_reason_kind"],
            "blocked"
        );
    }

    /// REQ-RUST-025.2
    #[test]
    fn test_enum_variant_module_scored() {
        let production = vec![build_test_site_value(
            "site-a",
            "production",
            &["Choice::Variant"],
            false,
            "",
        )];
        let primary = vec![build_test_site_value(
            "site-a",
            "module-scope",
            &["Choice::Variant"],
            false,
            "",
        )];
        let (_, result) = build_test_comparison_result(1, &production, &primary, &primary);
        assert_eq!(result["agreement_eligible_path_count"], 1);
        assert_eq!(result["agreement_match_count_value"], 1);
    }

    /// REQ-RUST-025.2, REQ-RUST-025.3
    #[test]
    fn test_primary_classes_partition_once() {
        let primary = vec![
            build_test_site_value(
                "module",
                "module-scope",
                &[],
                false,
                "external_source_missing",
            ),
            build_test_site_value("type", "type-relative", &[], false, "trait_solver_needed"),
            build_test_site_value("cfg", "cfg-inactive", &[], false, "cfg_inactive"),
            build_test_site_value("blocked", "blocked", &[], false, "unknown_receiver"),
        ];
        let (_, result) = build_test_comparison_result(1, &[], &primary, &[]);
        assert_eq!(result["agreement_primary_raw_site_count"], 4);
        assert_eq!(result["agreement_primary_unique_site_count"], 4);
        assert_eq!(result["agreement_module_scope_raw_site_count"], 1);
        assert_eq!(result["agreement_type_relative_raw_site_count"], 1);
        assert_eq!(result["agreement_cfg_inactive_raw_site_count"], 1);
        assert_eq!(result["agreement_blocked_raw_site_count"], 1);
        assert_eq!(result["agreement_eligible_path_count"], 1);
        assert_eq!(result["agreement_deferred_path_count"], 1);
        assert_eq!(result["agreement_blocked_path_count"], 1);
        assert_eq!(
            result["agreement_reason_counts"]["external_source_missing"],
            1
        );
    }

    /// REQ-RUST-025.3
    #[test]
    fn test_empty_denominator_is_na() {
        let (_, result) = build_test_comparison_result(1, &[], &[], &[]);
        assert_eq!(result["agreement_status_kind"], "not-applicable");
        assert_eq!(result["agreement_denominator_status_kind"], "N/A");
        assert!(result["agreement_exact_match_rate"].is_null());
    }

    /// REQ-RUST-025.4
    #[test]
    fn test_thread_counts_export_identically() {
        let production = vec![build_test_site_value(
            "site-a",
            "module-scope",
            &["right"],
            false,
            "",
        )];
        let primary = production.clone();
        let secondary = vec![build_test_site_value(
            "site-a",
            "module-scope",
            &["other"],
            false,
            "",
        )];
        let single = build_test_comparison_result(1, &production, &primary, &secondary);
        let multiple = build_test_comparison_result(2, &production, &primary, &secondary);
        assert_eq!(single, multiple);
        assert_eq!(single.1["agreement_secondary_changed_path_count"], 1);
    }

    /// REQ-RUST-025.2
    #[test]
    fn test_production_only_rows_visible() {
        let production = vec![build_test_site_value(
            "site-a",
            "module-scope",
            &["right"],
            false,
            "",
        )];
        let (_, result) = build_test_comparison_result(1, &production, &[], &[]);
        assert_eq!(result["agreement_eligible_path_count"], 0);
        assert_eq!(result["agreement_production_only_path_count"], 1);
        assert_eq!(result["agreement_status_kind"], "not-applicable");
    }

    /// REQ-RUST-024.2
    #[test]
    fn test_duckdb_rows_preserve_scope() {
        let source_scope = StageRunScopeValue {
            run_generation_digest_value: "generation".into(),
            stage_attempt_sequence_value: "attempt".into(),
        };
        let source_rows = [
            StageTableRowRecord {
                stage_owned_table_name: SOURCE_GRAPH_STAGE_NAME.into(),
                stage_row_record_key: "graph-key".into(),
                stage_row_payload_text: "{}".into(),
            },
            StageTableRowRecord {
                stage_owned_table_name: SOURCE_PATH_STAGE_NAME.into(),
                stage_row_record_key: "path-key".into(),
                stage_row_payload_text: "{}".into(),
            },
        ];
        let connection = build_isolated_comparison_connection(1).unwrap();
        let source_digest = verify_duckdb_source_copy(
            &connection,
            &source_scope,
            std::slice::from_ref(&source_rows[0]),
            std::slice::from_ref(&source_rows[1]),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
        let mut ordered_source_rows = source_rows.to_vec();
        ordered_source_rows.sort_by(|left, right| {
            left.stage_owned_table_name
                .cmp(&right.stage_owned_table_name)
                .then_with(|| left.stage_row_record_key.cmp(&right.stage_row_record_key))
        });
        let mut expected_hasher = blake3::Hasher::new();
        for row in &ordered_source_rows {
            hash_framed_source_value(
                &mut expected_hasher,
                &source_scope.run_generation_digest_value,
            );
            hash_framed_source_value(
                &mut expected_hasher,
                &source_scope.stage_attempt_sequence_value,
            );
            hash_framed_source_value(&mut expected_hasher, &row.stage_owned_table_name);
            hash_framed_source_value(&mut expected_hasher, &row.stage_row_record_key);
            hash_framed_source_value(&mut expected_hasher, &row.stage_row_payload_text);
        }
        assert_eq!(
            source_digest,
            expected_hasher.finalize().to_hex().to_string()
        );
        for other_scope in [
            StageRunScopeValue {
                run_generation_digest_value: "other-generation".into(),
                stage_attempt_sequence_value: "attempt".into(),
            },
            StageRunScopeValue {
                run_generation_digest_value: "generation".into(),
                stage_attempt_sequence_value: "other-attempt".into(),
            },
        ] {
            assert_ne!(
                source_digest,
                digest_persisted_source_rows(&other_scope, &ordered_source_rows)
            );
        }
        let production = [build_test_site_value(
            "site-a",
            "production",
            &["target"],
            false,
            "",
        )];
        let primary = [build_test_site_value(
            "site-a",
            "module-scope",
            &["target"],
            false,
            "",
        )];
        load_canonical_comparison_rows(&connection, &source_scope, &production, &primary, &primary)
            .unwrap();
        score_canonical_comparison_rows(&connection).unwrap();
        for (table, expected_count) in [
            ("oracle_source_copy_rows", 2),
            ("production_oracle_site_rows", 1),
            ("primary_oracle_site_rows", 1),
            ("secondary_oracle_site_rows", 1),
            ("oracle_comparison_evidence_rows", 1),
        ] {
            let query = format!(
                "SELECT count(*) FROM {table} WHERE source_scope_generation_value=? AND source_scope_attempt_value=?"
            );
            let count: u64 = connection
                .query_row(
                    &query,
                    params![
                        source_scope.run_generation_digest_value,
                        source_scope.stage_attempt_sequence_value
                    ],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, expected_count, "{table}");
            let total_count: u64 = connection
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(total_count, expected_count, "{table} total scope");
        }
    }

    /// REQ-RUST-024.2
    #[test]
    fn test_wrong_scope_prevents_matching() {
        let source_scope = StageRunScopeValue {
            run_generation_digest_value: "generation".into(),
            stage_attempt_sequence_value: "attempt".into(),
        };
        let connection = build_isolated_comparison_connection(1).unwrap();
        let production = [build_test_site_value(
            "site-a",
            "production",
            &["target"],
            false,
            "",
        )];
        let primary = [build_test_site_value(
            "site-a",
            "module-scope",
            &["target"],
            false,
            "",
        )];
        load_canonical_comparison_rows(&connection, &source_scope, &production, &primary, &primary)
            .unwrap();
        connection
            .execute(
                "UPDATE production_oracle_site_rows SET source_scope_generation_value='other-generation'",
                [],
            )
            .unwrap();
        score_canonical_comparison_rows(&connection).unwrap();
        let (evidence, result) = read_canonical_comparison_results(
            &connection,
            &source_scope,
            "source-digest",
            "primary-digest",
            "secondary-digest",
        )
        .unwrap();
        assert_eq!(result["agreement_match_count_value"], 0);
        assert_eq!(result["agreement_missing_production_path_count"], 1);
        assert_eq!(evidence.len(), 1);
        assert_eq!(
            evidence[0]["agreement_site_status_kind"],
            "missing-production"
        );
    }
}
