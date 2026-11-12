use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StoreBackendFailureError,
};
use parseltongue_table_store_traits::StageTableStorePort;

fn check_cancel(cancel: &RunCancelFlagValue) -> Result<(), StoreBackendFailureError> {
    if cancel.check_stage_work_cancelled() {
        Err(build_analysis_error("analysis copy cancelled"))
    } else {
        Ok(())
    }
}

fn hash_record(hasher: &mut blake3::Hasher, row: &StageTableRowRecord) {
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
    if source.read_workspace_publication_scope()?.as_ref() != Some(scope) {
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
        let mut hasher = blake3::Hasher::new();
        let mut previous_key: Option<String> = None;
        let mut count = 0;
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
                appender
                    .append_row(duckdb::params![
                        scope.run_generation_digest_value,
                        scope.stage_attempt_sequence_value,
                        table,
                        row.stage_row_record_key,
                        row.stage_row_payload_text
                    ])
                    .map_err(build_analysis_error)?;
                hash_record(&mut hasher, &row);
                previous_key = Some(row.stage_row_record_key);
                count += 1;
                Ok(())
            })?;
            appender.flush().map_err(build_analysis_error)?;
        }
        let digest = hasher.finalize().to_hex().to_string();
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
        Ok(total + count)
    })?;
    check_cancel(cancel)?;
    if source.read_workspace_publication_scope()?.as_ref() != Some(scope) {
        return Err(build_analysis_error(
            "source publication changed during copy",
        ));
    }
    transaction.commit().map_err(build_analysis_error)?;
    Ok(count)
}
