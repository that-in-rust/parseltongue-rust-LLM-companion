use parseltongue_core_row_types::{RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StageUnitWriteRequest, StoreBackendFailureError};
use parseltongue_table_store_traits::StageTableStorePort;
use super::{TursoWorkspaceStoreAdapter, build_store_backend_error};

const STAGE_OWNED_TABLE_NAMES: &[(&str, &str)] = &[
    ("crate-dependency-graph-table", "crate-graph"),
    ("workspace-config-state-records", "crate-graph"),
    ("file-source-text-table", "source-ingest"),
    ("span-reference-text-table", "syntax-spans"),
    ("span-byte-ownership-ranges", "syntax-spans"),
    ("field-signature-detail-table", "syntax-spans"),
    ("body-site-reference-table", "syntax-spans"),
    ("module-source-closure-requests", "syntax-spans"),
    ("syntax-file-work-requests", "syntax-spans"),
    ("stage-run-generation-records", "markers"),
];

fn check_stage_cancel_requested(cancel: &RunCancelFlagValue) -> Result<(), turso::Error> {
    if cancel.check_stage_work_cancelled() { Err(turso::Error::Interrupt("stage cancelled".into())) } else { Ok(()) }
}

pub fn retry_bounded_store_write<T>(cancel: &RunCancelFlagValue, mut attempt: impl FnMut() -> Result<T, turso::Error>) -> Result<T, StoreBackendFailureError> {
    for retry in 0..6 {
        check_stage_cancel_requested(cancel).map_err(build_store_backend_error)?;
        match attempt() {
            Ok(value) => return Ok(value),
            Err(error) if matches!(error, turso::Error::Busy(_) | turso::Error::BusySnapshot(_)) => {
                if retry == 5 { return Err(build_store_backend_error(format!("StoreWriteConflictError: {error}; 6 attempts"))); }
                for _ in 0..(10_u64 << retry) {
                    check_stage_cancel_requested(cancel).map_err(build_store_backend_error)?;
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
            Err(error) => return Err(build_store_backend_error(error)),
        }
    }
    Err(build_store_backend_error("unreachable retry state"))
}

fn validate_stage_owned_table(table: &str) -> Result<(), StoreBackendFailureError> {
    if STAGE_OWNED_TABLE_NAMES.iter().any(|(name,_)| *name == table) { Ok(()) }
    else { Err(build_store_backend_error(format!("undeclared table {table}"))) }
}

impl StageTableStorePort for TursoWorkspaceStoreAdapter {
    fn initialize_workspace_stage_schema(&self) -> Result<(), StoreBackendFailureError> {
        self.store_runtime_handle_value.block_on(async {
            let connection = self.store_database_handle_value.connect()?;
            connection.execute("PRAGMA journal_mode='mvcc'", ()).await?;
            for (table,_) in STAGE_OWNED_TABLE_NAMES {
                connection.execute(format!("CREATE TABLE IF NOT EXISTS \"{table}\" (
                    run_generation_digest_value TEXT NOT NULL, stage_attempt_sequence_value TEXT NOT NULL,
                    stage_row_record_key TEXT NOT NULL, stage_unit_work_key TEXT NOT NULL, stage_row_payload_text TEXT NOT NULL,
                    PRIMARY KEY(run_generation_digest_value,stage_attempt_sequence_value,stage_row_record_key))"), ()).await?;
            }
            connection.execute("CREATE TABLE IF NOT EXISTS \"file-source-content-records\" (file_content_digest_full TEXT PRIMARY KEY, file_source_content_bytes BLOB NOT NULL)", ()).await?;
            connection.execute("CREATE TABLE IF NOT EXISTS \"workspace-generation-publication-manifest\" (manifest_singleton_key_value INTEGER PRIMARY KEY, run_generation_digest_value TEXT NOT NULL, stage_attempt_sequence_value TEXT NOT NULL)", ()).await?;
            Ok::<_,turso::Error>(())
        }).map_err(build_store_backend_error)
    }

    fn write_stage_table_unit(&self, request: &StageUnitWriteRequest<'_>, cancel: &RunCancelFlagValue) -> Result<usize, StoreBackendFailureError> {
        for row in request.stage_unit_output_rows {
            let owner = STAGE_OWNED_TABLE_NAMES.iter().find(|(name,_)| *name == row.stage_owned_table_name).map(|(_,owner)| *owner);
            if owner != Some(request.stage_owner_name_value) || row.stage_row_record_key.starts_with("__marker__/") {
                return Err(build_store_backend_error("stage output table access denied"));
            }
        }
        retry_bounded_store_write(cancel, || self.store_runtime_handle_value.block_on(async {
            let mut connection = self.store_database_handle_value.connect()?;
            let transaction = connection.transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent).await?;
            let result = async {
                let scope = request.stage_run_scope_value;
                for input in request.stage_input_stage_names {
                    let mut rows = transaction.query("SELECT 1 FROM \"stage-run-generation-records\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND stage_row_record_key=?",
                        turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone(),format!("__marker__/{input}/__complete__")]).await?;
                    if rows.next().await?.is_none() {
                        return Err(turso::Error::Misuse(format!("StageInputMissingError: {input}")));
                    }
                }
                for (table,owner) in STAGE_OWNED_TABLE_NAMES {
                    if *owner == request.stage_owner_name_value {
                        transaction.execute(format!("DELETE FROM \"{table}\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND stage_unit_work_key=?"),
                            turso::params![scope.run_generation_digest_value.clone(), scope.stage_attempt_sequence_value.clone(), request.stage_unit_work_key.to_owned()]).await?;
                    }
                }
                for row in request.stage_unit_output_rows {
                    check_stage_cancel_requested(cancel)?;
                    transaction.execute(format!("INSERT INTO \"{}\" VALUES (?,?,?,?,?)", row.stage_owned_table_name),
                        turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone(),row.stage_row_record_key.clone(),request.stage_unit_work_key.to_owned(),row.stage_row_payload_text.clone()]).await?;
                }
                check_stage_cancel_requested(cancel)?;
                transaction.execute("INSERT OR REPLACE INTO \"stage-run-generation-records\" VALUES (?,?,?,?,?)",
                    turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone(),
                    format!("__marker__/{}/{}",request.stage_owner_name_value,request.stage_unit_work_key),request.stage_unit_work_key.to_owned(),"{\"status\":\"complete\"}".to_owned()]).await?;
                check_stage_cancel_requested(cancel)?;
                Ok::<_,turso::Error>(request.stage_unit_output_rows.len())
            }.await;
            match result { Ok(count) => { transaction.commit().await?; Ok(count) }, Err(error) => { let _ = transaction.rollback().await; Err(error) } }
        }))
    }

    fn read_stage_table_rows(&self, scope: &StageRunScopeValue, table: &str) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
        validate_stage_owned_table(table)?;
        self.store_runtime_handle_value.block_on(async {
            let connection = self.store_database_handle_value.connect()?;
            let mut rows = connection.query(format!("SELECT stage_row_record_key,stage_row_payload_text FROM \"{table}\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? ORDER BY stage_row_record_key"),
                turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
            let mut records = Vec::new();
            while let Some(row) = rows.next().await? {
                records.push(StageTableRowRecord { stage_owned_table_name: table.into(),stage_row_record_key: row.get(0)?, stage_row_payload_text: row.get(1)? });
            }
            Ok::<_,turso::Error>(records)
        }).map_err(build_store_backend_error)
    }

    fn write_file_content_bytes(&self, digest: &str, bytes: &[u8]) -> Result<bool, StoreBackendFailureError> {
        retry_bounded_store_write(&RunCancelFlagValue::default(), || self.store_runtime_handle_value.block_on(async {
            let connection = self.store_database_handle_value.connect()?;
            let mut rows = connection.query("SELECT file_source_content_bytes FROM \"file-source-content-records\" WHERE file_content_digest_full=?", [digest.to_owned()]).await?;
            if let Some(row) = rows.next().await? {
                let existing: Vec<u8> = row.get(0)?;
                return if existing == bytes { Ok(false) } else { Err(turso::Error::Constraint("full digest collision".into())) };
            }
            connection.execute("INSERT INTO \"file-source-content-records\" VALUES (?,?)", turso::params![digest.to_owned(),bytes.to_vec()]).await?;
            Ok(true)
        }))
    }

    fn read_file_content_bytes(&self, digest: &str) -> Result<Vec<u8>, StoreBackendFailureError> {
        self.store_runtime_handle_value.block_on(async {
            let connection = self.store_database_handle_value.connect()?;
            let mut rows = connection.query("SELECT file_source_content_bytes FROM \"file-source-content-records\" WHERE file_content_digest_full=?", [digest.to_owned()]).await?;
            rows.next().await?.ok_or(turso::Error::QueryReturnedNoRows)?.get(0)
        }).map_err(build_store_backend_error)
    }

    fn publish_workspace_generation_manifest(&self, scope: &StageRunScopeValue, generation: &str, cancel: &RunCancelFlagValue) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        retry_bounded_store_write(cancel, || self.store_runtime_handle_value.block_on(async {
            let mut connection = self.store_database_handle_value.connect()?;
            let transaction = connection.transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent).await?;
            let result = async {
                for stage in ["crate-graph","source-ingest","syntax-spans"] {
                    let mut rows = transaction.query("SELECT 1 FROM \"stage-run-generation-records\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND stage_row_record_key=?",
                        turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone(),format!("__marker__/{stage}/__complete__")]).await?;
                    if rows.next().await?.is_none() { return Err(turso::Error::Misuse(format!("StageInputMissingError: {stage}"))); }
                }
                for (table,_) in STAGE_OWNED_TABLE_NAMES {
                    transaction.execute(format!("UPDATE \"{table}\" SET run_generation_digest_value=? WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=?"),
                        turso::params![generation.to_owned(),scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
                }
                check_stage_cancel_requested(cancel)?;
                transaction.execute("INSERT OR REPLACE INTO \"workspace-generation-publication-manifest\" VALUES (1,?,?)",
                    turso::params![generation.to_owned(),scope.stage_attempt_sequence_value.clone()]).await?;
                check_stage_cancel_requested(cancel)?;
                Ok(StageRunScopeValue { run_generation_digest_value:generation.into(),stage_attempt_sequence_value:scope.stage_attempt_sequence_value.clone() })
            }.await;
            match result { Ok(value) => { transaction.commit().await?; Ok(value) }, Err(error) => { let _ = transaction.rollback().await; Err(error) } }
        }))
    }

    fn read_workspace_publication_scope(&self) -> Result<Option<StageRunScopeValue>, StoreBackendFailureError> {
        self.store_runtime_handle_value.block_on(async {
            let connection = self.store_database_handle_value.connect()?;
            let mut rows = connection.query("SELECT run_generation_digest_value,stage_attempt_sequence_value FROM \"workspace-generation-publication-manifest\" WHERE manifest_singleton_key_value=1", ()).await?;
            rows.next().await?.map(|row| Ok::<_,turso::Error>(StageRunScopeValue {run_generation_digest_value:row.get(0)?, stage_attempt_sequence_value:row.get(1)?})).transpose()
        }).map_err(build_store_backend_error)
    }
}
