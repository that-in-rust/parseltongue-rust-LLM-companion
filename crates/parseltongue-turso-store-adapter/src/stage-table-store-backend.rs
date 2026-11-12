use super::{TursoWorkspaceStoreAdapter, build_store_backend_error};
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StageUnitWriteRequest,
    StoreBackendFailureError,
};
use parseltongue_table_store_traits::StageTableStorePort;

#[path = "frozen-stage-snapshot-rules.rs"]
mod frozen_stage_snapshot_rules;
use frozen_stage_snapshot_rules::{
    build_snapshot_unit_visibility, resolve_snapshot_read_scope, seal_snapshot_publication,
};

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
    ("module-scope-binding-table", "name-resolution"),
    ("name-resolution-scope-records", "name-resolution"),
    ("path-resolution-site-records", "name-resolution"),
    ("analysis-resolution-receipt-records", "name-resolution"),
    ("macro-expanded-item-table", "name-resolution"),
    (
        "lang-item-definition-table",
        "syntax-namespace-input-records",
    ),
    (
        "syntax-namespace-input-scopes",
        "syntax-namespace-input-records",
    ),
    (
        "syntax-namespace-input-items",
        "syntax-namespace-input-records",
    ),
    (
        "syntax-namespace-input-imports",
        "syntax-namespace-input-records",
    ),
    (
        "syntax-namespace-input-paths",
        "syntax-namespace-input-records",
    ),
    (
        "syntax-namespace-input-macros",
        "syntax-namespace-input-records",
    ),
    (
        "syntax-namespace-input-externs",
        "syntax-namespace-input-records",
    ),
    ("stage-run-generation-records", "markers"),
    (
        "legacy-macro-scope-records",
        "syntax-namespace-input-records",
    ),
    (
        "legacy-macro-source-events",
        "syntax-namespace-input-records",
    ),
];

fn check_table_stage_access(table: &str, stage: &str) -> bool {
    STAGE_OWNED_TABLE_NAMES
        .iter()
        .any(|(name, owner)| *name == table && *owner == stage)
        || (stage == "name-resolution"
            && STAGE_OWNED_TABLE_NAMES
                .iter()
                .any(|(name, owner)| *name == table && *owner == "syntax-namespace-input-records"))
        || (stage == "name-resolution"
            && matches!(
                table,
                "span-reference-text-table"
                    | "span-byte-ownership-ranges"
                    | "field-signature-detail-table"
                    | "body-site-reference-table"
                    | "file-source-text-table"
                    | "module-source-closure-requests"
                    | "syntax-file-work-requests"
            ))
}

fn check_stage_cancel_requested(cancel: &RunCancelFlagValue) -> Result<(), turso::Error> {
    if cancel.check_stage_work_cancelled() {
        Err(turso::Error::Interrupt("stage cancelled".into()))
    } else {
        Ok(())
    }
}

pub fn retry_bounded_store_write<T>(
    cancel: &RunCancelFlagValue,
    mut attempt: impl FnMut() -> Result<T, turso::Error>,
) -> Result<T, StoreBackendFailureError> {
    for retry in 0..6 {
        check_stage_cancel_requested(cancel).map_err(build_store_backend_error)?;
        match attempt() {
            Ok(value) => return Ok(value),
            Err(error)
                if matches!(error, turso::Error::Busy(_) | turso::Error::BusySnapshot(_)) =>
            {
                if retry == 5 {
                    return Err(build_store_backend_error(format!(
                        "StoreWriteConflictError: {error}; 6 attempts"
                    )));
                }
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

fn build_crate_visibility_clause(table: &str, revision: Option<i64>) -> String {
    let owner = STAGE_OWNED_TABLE_NAMES
        .iter()
        .find(|(name, _)| *name == table)
        .map(|(_, owner)| *owner)
        .filter(|owner| *owner == "syntax-namespace-input-records")
        .unwrap_or("name-resolution");
    let frozen_completion=revision.map(|revision|format!("AND EXISTS (SELECT 1 FROM \"frozen-stage-unit-memberships\" AS frozen_completion
        WHERE frozen_completion.generation_key=completed.run_generation_digest_value AND frozen_completion.source_attempt_key=completed.stage_attempt_sequence_value
            AND frozen_completion.stage_marker_key=completed.stage_row_record_key AND frozen_completion.publication_revision<={revision})")).unwrap_or_default();
    let snapshot_visibility = build_snapshot_unit_visibility(table, revision);
    format!(
        r#"("{table}".stage_unit_work_key NOT LIKE 'crate/%/pending/%' OR EXISTS (
        SELECT 1 FROM "stage-run-generation-records" AS completed
        WHERE completed.run_generation_digest_value="{table}".run_generation_digest_value
        AND completed.stage_attempt_sequence_value="{table}".stage_attempt_sequence_value
        AND completed.stage_row_record_key IN ('__marker__/{owner}/' ||
            substr("{table}".stage_unit_work_key,1,instr("{table}".stage_unit_work_key,'/pending/')-1),'__marker__/name-resolution/' ||
            substr("{table}".stage_unit_work_key,1,instr("{table}".stage_unit_work_key,'/pending/')-1)) {frozen_completion})) AND {snapshot_visibility}"#
    )
}

fn validate_stage_owned_table(table: &str) -> Result<(), StoreBackendFailureError> {
    if STAGE_OWNED_TABLE_NAMES
        .iter()
        .any(|(name, _)| *name == table)
    {
        Ok(())
    } else {
        Err(build_store_backend_error(format!(
            "undeclared table {table}"
        )))
    }
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
                let prefix = table.rsplit_once('-').map(|(prefix,_)|prefix).unwrap_or(table);
                connection.execute(format!("CREATE INDEX IF NOT EXISTS \"{prefix}-lookup\" ON \"{table}\" (run_generation_digest_value,stage_attempt_sequence_value,stage_unit_work_key)"), ()).await?;
            }
            connection.execute("CREATE TABLE IF NOT EXISTS \"file-source-content-records\" (file_content_digest_full TEXT PRIMARY KEY, file_source_content_bytes BLOB NOT NULL)", ()).await?;
            connection.execute("CREATE TABLE IF NOT EXISTS \"workspace-generation-publication-manifest\" (manifest_singleton_key_value INTEGER PRIMARY KEY, run_generation_digest_value TEXT NOT NULL, stage_attempt_sequence_value TEXT NOT NULL)", ()).await?;
            connection.execute("CREATE TABLE IF NOT EXISTS \"analysis-input-publication-manifest\" (manifest_singleton_key_value INTEGER PRIMARY KEY, run_generation_digest_value TEXT NOT NULL, stage_attempt_sequence_value TEXT NOT NULL)", ()).await?;
            connection.execute("CREATE TABLE IF NOT EXISTS \"frozen-stage-publication-records\" (generation_key TEXT NOT NULL,attempt_key TEXT NOT NULL,source_attempt_key TEXT NOT NULL,publication_revision INTEGER NOT NULL,PRIMARY KEY(generation_key,attempt_key))",()).await?;
            connection.execute("CREATE TABLE IF NOT EXISTS \"frozen-stage-unit-memberships\" (generation_key TEXT NOT NULL,source_attempt_key TEXT NOT NULL,stage_marker_key TEXT NOT NULL,publication_revision INTEGER NOT NULL,PRIMARY KEY(generation_key,source_attempt_key,stage_marker_key))",()).await?;
            Ok::<_,turso::Error>(())
        }).map_err(build_store_backend_error)
    }

    fn write_stage_table_unit(
        &self,
        request: &StageUnitWriteRequest<'_>,
        cancel: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        if request.stage_owner_name_value == "name-resolution"
            && !request.stage_unit_output_rows.is_empty()
            && !request.stage_unit_work_key.starts_with("crate/")
        {
            return Err(build_store_backend_error(
                "name-resolution work keys require the crate/ namespace",
            ));
        }
        request.stage_unit_output_rows.iter().try_for_each(|row| {
            if !check_table_stage_access(
                &row.stage_owned_table_name,
                request.stage_owner_name_value,
            ) || row.stage_row_record_key.starts_with("__marker__/")
            {
                return Err(build_store_backend_error(
                    "stage output table access denied",
                ));
            }
            Ok(())
        })?;
        retry_bounded_store_write(cancel, || {
            self.store_runtime_handle_value.block_on(async {
            let started=std::time::Instant::now();
            let profile=std::env::var_os("PARSELTONGUE_PROFILE_STAGE_WRITES").is_some();
            let mut connection = self.store_database_handle_value.connect()?;
            let transaction = connection.transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent).await?;
            let result = async {
                let scope = request.stage_run_scope_value;
                let completion=request.stage_unit_work_key=="__complete__";
                if completion && !request.stage_unit_output_rows.is_empty(){return Err(turso::Error::Misuse("stage completion markers cannot own output rows".into()));}
                let mut snapshot=transaction.query("SELECT 1 FROM \"frozen-stage-publication-records\" WHERE generation_key=? AND attempt_key=? AND source_attempt_key<>attempt_key",
                    turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
                if snapshot.next().await?.is_some() {return Err(turso::Error::Misuse("frozen snapshot scopes are read-only".into()));}
                drop(snapshot);
                let shared_owners=STAGE_OWNED_TABLE_NAMES.iter().map(|(_,owner)|*owner).filter(|owner|STAGE_OWNED_TABLE_NAMES.iter().any(|(table,_)|
                    check_table_stage_access(table,request.stage_owner_name_value) && check_table_stage_access(table,owner))).filter(|owner|!completion || *owner==request.stage_owner_name_value).collect::<std::collections::BTreeSet<_>>();
                let parameters=std::iter::once(scope.run_generation_digest_value.clone()).chain(std::iter::once(scope.stage_attempt_sequence_value.clone()))
                    .chain(shared_owners.iter().map(|owner|format!("__marker__/{owner}/{}",request.stage_unit_work_key))).collect::<Vec<_>>();
                let placeholders=std::iter::repeat_n("?",shared_owners.len()).collect::<Vec<_>>().join(",");
                let mut sealed=transaction.query(format!("SELECT 1 FROM \"frozen-stage-unit-memberships\" WHERE generation_key=? AND source_attempt_key=? AND stage_marker_key IN ({placeholders})"),parameters).await?;
                if sealed.next().await?.is_some() {return Err(turso::Error::Misuse("published stage unit is immutable".into()));}
                drop(sealed);
                for input in request.stage_input_stage_names {
                    let mut rows = transaction.query("SELECT 1 FROM \"stage-run-generation-records\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND stage_row_record_key=?",
                        turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone(),format!("__marker__/{input}/__complete__")]).await?;
                    if rows.next().await?.is_none() {
                        return Err(turso::Error::Misuse(format!("StageInputMissingError: {input}")));
                    }
                }
                let setup=started.elapsed().as_secs_f64();
                let cleanup_started=std::time::Instant::now();
                for (table,_) in STAGE_OWNED_TABLE_NAMES {
                    if !completion && check_table_stage_access(table,request.stage_owner_name_value) {
                        transaction.execute(format!("DELETE FROM \"{table}\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND stage_unit_work_key=?"),
                            turso::params![scope.run_generation_digest_value.clone(), scope.stage_attempt_sequence_value.clone(), request.stage_unit_work_key.to_owned()]).await?;
                    }
                }
                let cleanup=cleanup_started.elapsed().as_secs_f64();
                let insert_started=std::time::Instant::now();
                for (table,_) in STAGE_OWNED_TABLE_NAMES {
                    if !check_table_stage_access(table,request.stage_owner_name_value) { continue; }
                    let rows: Vec<_> = request.stage_unit_output_rows.iter().filter(|row|row.stage_owned_table_name==*table).collect();
                    for batch in rows.chunks(self.stage_insert_batch_limit.get()) {
                        check_stage_cancel_requested(cancel)?;
                        let placeholders = std::iter::repeat_n("(?,?,?,?,?)",batch.len()).collect::<Vec<_>>().join(",");
                        let parameters: Vec<_> = batch.iter().flat_map(|row|[
                            scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone(),
                            row.stage_row_record_key.clone(),request.stage_unit_work_key.to_owned(),row.stage_row_payload_text.clone()
                        ]).collect();
                        transaction.execute(format!("INSERT INTO \"{table}\" VALUES {placeholders}"),parameters).await?;
                    }
                }
                let insert=insert_started.elapsed().as_secs_f64();
                check_stage_cancel_requested(cancel)?;
                transaction.execute("INSERT OR REPLACE INTO \"stage-run-generation-records\" VALUES (?,?,?,?,?)",
                    turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone(),
                    format!("__marker__/{}/{}",request.stage_owner_name_value,request.stage_unit_work_key),request.stage_unit_work_key.to_owned(),"{\"status\":\"complete\"}".to_owned()]).await?;
                check_stage_cancel_requested(cancel)?;
                Ok::<_,turso::Error>((request.stage_unit_output_rows.len(),setup,cleanup,insert))
            }.await;
            match result { Ok((count,setup,cleanup,insert)) => {
                let commit_started=std::time::Instant::now();
                transaction.commit().await?;
                if profile { eprintln!("stage=store-unit owner={} unit={} rows={count} batch_limit={} setup_seconds={setup:.6} cleanup_seconds={cleanup:.6} insert_seconds={insert:.6} commit_seconds={:.6} total_seconds={:.6}",
                    request.stage_owner_name_value,request.stage_unit_work_key,self.stage_insert_batch_limit.get(),commit_started.elapsed().as_secs_f64(),started.elapsed().as_secs_f64()); }
                Ok(count)
            }, Err(error) => { let _ = transaction.rollback().await; Err(error) } }
        })
        })
    }

    fn read_stage_table_rows(
        &self,
        scope: &StageRunScopeValue,
        table: &str,
    ) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
        validate_stage_owned_table(table)?;
        self.store_runtime_handle_value.block_on(async {
            let connection = self.store_database_handle_value.connect()?;
            let (scope,revision)=resolve_snapshot_read_scope(&connection,scope).await?;
            let mut rows = connection.query(format!("SELECT stage_row_record_key,stage_row_payload_text FROM \"{table}\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND {} ORDER BY stage_row_record_key",build_crate_visibility_clause(table,revision)),
                turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
            let mut records = Vec::new();
            while let Some(row) = rows.next().await? {
                records.push(StageTableRowRecord { stage_owned_table_name: table.into(),stage_row_record_key: row.get(0)?, stage_row_payload_text: row.get(1)? });
            }
            Ok::<_,turso::Error>(records)
        }).map_err(build_store_backend_error)
    }

    fn count_stage_table_rows(
        &self,
        scope: &StageRunScopeValue,
        table: &str,
    ) -> Result<usize, StoreBackendFailureError> {
        validate_stage_owned_table(table)?;
        self.store_runtime_handle_value.block_on(async{
            let connection=self.store_database_handle_value.connect()?;
            let (scope,revision)=resolve_snapshot_read_scope(&connection,scope).await?;
            let mut rows=connection.query(format!(r#"SELECT COUNT(*) FROM "{table}" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND {}"#,build_crate_visibility_clause(table,revision)),
                turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
            let count:i64=rows.next().await?.ok_or(turso::Error::QueryReturnedNoRows)?.get(0)?;
            usize::try_from(count).map_err(|error|turso::Error::ConversionFailure(error.to_string()))
        }).map_err(build_store_backend_error)
    }

    fn visit_stage_table_rows(
        &self,
        scope: &StageRunScopeValue,
        table: &str,
        visitor: &mut dyn FnMut(StageTableRowRecord) -> Result<(), StoreBackendFailureError>,
    ) -> Result<usize, StoreBackendFailureError> {
        validate_stage_owned_table(table)?;
        self.store_runtime_handle_value.block_on(async{
            let connection=self.store_database_handle_value.connect().map_err(build_store_backend_error)?;
            let (scope,revision)=resolve_snapshot_read_scope(&connection,scope).await.map_err(build_store_backend_error)?;
            let mut rows=connection.query(format!(r#"SELECT stage_row_record_key,stage_row_payload_text FROM "{table}" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND {} ORDER BY stage_row_record_key"#,build_crate_visibility_clause(table,revision)),
                turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await.map_err(build_store_backend_error)?;
            let mut count=0;
            while let Some(row)=rows.next().await.map_err(build_store_backend_error)? {
                visitor(StageTableRowRecord{stage_owned_table_name:table.into(),stage_row_record_key:row.get(0).map_err(build_store_backend_error)?,stage_row_payload_text:row.get(1).map_err(build_store_backend_error)?})?;
                count+=1;
            }
            Ok(count)
        })
    }

    fn prepare_workspace_generation_scope(
        &self,
        scope: &StageRunScopeValue,
        generation: &str,
        cancel: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        retry_bounded_store_write(cancel, || {
            if scope.run_generation_digest_value == generation {
                return Ok(scope.clone());
            }
            self.store_runtime_handle_value.block_on(async{
                let mut connection=self.store_database_handle_value.connect()?;
                let transaction=connection.transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent).await?;
                let result=async{
                    let mut frozen=transaction.query("SELECT 1 FROM \"frozen-stage-publication-records\" WHERE generation_key=? AND attempt_key=?",
                        turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
                    if frozen.next().await?.is_some() {return Err(turso::Error::Misuse("published generations cannot be renamed".into()));}
                    drop(frozen);
                    for table in ["span-reference-text-table","span-byte-ownership-ranges","field-signature-detail-table","body-site-reference-table"]{
                        let mut rows=transaction.query(format!(r#"SELECT 1 FROM "{table}" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? LIMIT 1"#),
                            turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
                        if rows.next().await?.is_some(){return Err(turso::Error::Misuse("StageScopePreparationError: prepare before syntax outputs".into()));}
                    }
                    for (table,_) in STAGE_OWNED_TABLE_NAMES {
                        check_stage_cancel_requested(cancel)?;
                        transaction.execute(format!(r#"UPDATE "{table}" SET run_generation_digest_value=? WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=?"#),
                            turso::params![generation.to_owned(),scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
                    }
                    check_stage_cancel_requested(cancel)?;
                    Ok::<_,turso::Error>(StageRunScopeValue{run_generation_digest_value:generation.into(),stage_attempt_sequence_value:scope.stage_attempt_sequence_value.clone()})
                }.await;
                match result {Ok(scope)=>{transaction.commit().await?;Ok(scope)},Err(error)=>{let _=transaction.rollback().await;Err(error)}}
            })
        })
    }

    fn write_file_content_bytes(
        &self,
        digest: &str,
        bytes: &[u8],
    ) -> Result<bool, StoreBackendFailureError> {
        retry_bounded_store_write(&RunCancelFlagValue::default(), || {
            self.store_runtime_handle_value.block_on(async {
            let connection = self.store_database_handle_value.connect()?;
            let mut rows = connection.query("SELECT file_source_content_bytes FROM \"file-source-content-records\" WHERE file_content_digest_full=?", [digest.to_owned()]).await?;
            if let Some(row) = rows.next().await? {
                let existing: Vec<u8> = row.get(0)?;
                return if existing == bytes { Ok(false) } else { Err(turso::Error::Constraint("full digest collision".into())) };
            }
            connection.execute("INSERT INTO \"file-source-content-records\" VALUES (?,?)", turso::params![digest.to_owned(),bytes.to_vec()]).await?;
            Ok(true)
        })
        })
    }

    fn read_file_content_bytes(&self, digest: &str) -> Result<Vec<u8>, StoreBackendFailureError> {
        self.store_runtime_handle_value.block_on(async {
            let connection = self.store_database_handle_value.connect()?;
            let mut rows = connection.query("SELECT file_source_content_bytes FROM \"file-source-content-records\" WHERE file_content_digest_full=?", [digest.to_owned()]).await?;
            rows.next().await?.ok_or(turso::Error::QueryReturnedNoRows)?.get(0)
        }).map_err(build_store_backend_error)
    }

    fn publish_analysis_input_snapshot(
        &self,
        scope: &StageRunScopeValue,
        cancel: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        retry_bounded_store_write(cancel, || {
            self.store_runtime_handle_value.block_on(async{
            let mut connection=self.store_database_handle_value.connect()?;
            let transaction=connection.transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent).await?;
            let result=async{
                check_stage_cancel_requested(cancel)?;
                let mut rows=transaction.query("SELECT count(*) FROM \"stage-run-generation-records\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND stage_row_record_key IN ('__marker__/crate-graph/__complete__','__marker__/source-ingest/__complete__','__marker__/syntax-spans/__complete__','__marker__/syntax-namespace-input-records/__complete__')",turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
                let count:i64=rows.next().await?.ok_or(turso::Error::QueryReturnedNoRows)?.get(0)?;
                drop(rows);
                if count!=4{return Err(turso::Error::Misuse("StageInputMissingError: analysis source incomplete".into()));}
                let published=seal_snapshot_publication(&transaction,scope).await?;
                transaction.execute("INSERT OR REPLACE INTO \"analysis-input-publication-manifest\" VALUES (1,?,?)",turso::params![published.run_generation_digest_value.clone(),published.stage_attempt_sequence_value.clone()]).await?;
                check_stage_cancel_requested(cancel)?;
                Ok(published)
            }.await;
            match result {Ok(value)=>{transaction.commit().await?;Ok(value)},Err(error)=>{let _=transaction.rollback().await;Err(error)}}
        })
        })
    }

    fn read_analysis_input_publication_scope(
        &self,
    ) -> Result<Option<StageRunScopeValue>, StoreBackendFailureError> {
        self.store_runtime_handle_value.block_on(async{
            let connection=self.store_database_handle_value.connect()?;
            let mut rows=connection.query("SELECT run_generation_digest_value,stage_attempt_sequence_value FROM \"analysis-input-publication-manifest\" WHERE manifest_singleton_key_value=1 ORDER BY run_generation_digest_value,stage_attempt_sequence_value",()).await?;
            rows.next().await?.map(|row|Ok::<_,turso::Error>(StageRunScopeValue{run_generation_digest_value:row.get(0)?,stage_attempt_sequence_value:row.get(1)?})).transpose()
        }).map_err(build_store_backend_error)
    }

    fn publish_workspace_generation_manifest(
        &self,
        scope: &StageRunScopeValue,
        generation: &str,
        cancel: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        retry_bounded_store_write(cancel, || {
            self.store_runtime_handle_value.block_on(async {
            let mut connection = self.store_database_handle_value.connect()?;
            let transaction = connection.transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent).await?;
            let result = async {
                let mut required=std::collections::BTreeSet::from(["crate-graph".to_owned(),"source-ingest".to_owned(),"syntax-spans".to_owned()]);
                let mut configuration_rows=transaction.query("SELECT stage_row_payload_text FROM \"workspace-config-state-records\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND stage_row_record_key='workspace'",
                    turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
                if let Some(row)=configuration_rows.next().await? {
                    let configuration:serde_json::Value=serde_json::from_str(&row.get::<String>(0)?).map_err(|error|turso::Error::Misuse(error.to_string()))?;
                    if let Some(stages)=configuration.get("workspace_required_stage_names") {
                        for stage in stages.as_array().ok_or_else(||turso::Error::Misuse("invalid required stage list".into()))? {
                            let stage=stage.as_str().ok_or_else(||turso::Error::Misuse("invalid required stage name".into()))?;
                            if !STAGE_OWNED_TABLE_NAMES.iter().any(|(_,owner)|*owner==stage && stage!="markers") {
                                return Err(turso::Error::Misuse(format!("unsupported required stage {stage}")));
                            }
                            required.insert(stage.to_owned());
                        }
                    }
                }
                drop(configuration_rows);
                for stage in required {
                    let mut rows = transaction.query("SELECT 1 FROM \"stage-run-generation-records\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND stage_row_record_key=?",
                        turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone(),format!("__marker__/{stage}/__complete__")]).await?;
                    if rows.next().await?.is_none() { return Err(turso::Error::Misuse(format!("StageInputMissingError: {stage}"))); }
                }
                if scope.run_generation_digest_value!=generation {
                let mut frozen=transaction.query("SELECT 1 FROM \"frozen-stage-publication-records\" WHERE generation_key=? AND attempt_key=?",
                    turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
                if frozen.next().await?.is_some() {return Err(turso::Error::Misuse("published generations cannot be renamed".into()));}
                drop(frozen);
                for (table,_) in STAGE_OWNED_TABLE_NAMES {
                    transaction.execute(format!("UPDATE \"{table}\" SET run_generation_digest_value=? WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=?"),
                        turso::params![generation.to_owned(),scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
                }
                }
                check_stage_cancel_requested(cancel)?;
                let published=seal_snapshot_publication(&transaction,&StageRunScopeValue {run_generation_digest_value:generation.into(),stage_attempt_sequence_value:scope.stage_attempt_sequence_value.clone()}).await?;
                transaction.execute("INSERT OR REPLACE INTO \"workspace-generation-publication-manifest\" VALUES (1,?,?)",
                    turso::params![generation.to_owned(),published.stage_attempt_sequence_value.clone()]).await?;
                check_stage_cancel_requested(cancel)?;
                Ok(published)
            }.await;
            match result { Ok(value) => { transaction.commit().await?; Ok(value) }, Err(error) => { let _ = transaction.rollback().await; Err(error) } }
        })
        })
    }

    fn read_workspace_publication_scope(
        &self,
    ) -> Result<Option<StageRunScopeValue>, StoreBackendFailureError> {
        self.store_runtime_handle_value.block_on(async {
            let connection = self.store_database_handle_value.connect()?;
            let mut rows = connection.query("SELECT run_generation_digest_value,stage_attempt_sequence_value FROM \"workspace-generation-publication-manifest\" WHERE manifest_singleton_key_value=1", ()).await?;
            rows.next().await?.map(|row| Ok::<_,turso::Error>(StageRunScopeValue {run_generation_digest_value:row.get(0)?, stage_attempt_sequence_value:row.get(1)?})).transpose()
        }).map_err(build_store_backend_error)
    }
}

#[cfg(test)]
mod stage_unit_cleanup_tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use std::hash::{Hash, Hasher};

    fn write_fixture_json_units(
        store: &TursoWorkspaceStoreAdapter,
        scope: &StageRunScopeValue,
        unit: &str,
        rows: &[StageTableRowRecord],
        limit: usize,
    ) -> Result<(), turso::Error> {
        store.store_runtime_handle_value.block_on(async{
            let mut connection=store.store_database_handle_value.connect()?;
            let transaction=connection.transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent).await?;
            let result=async{
                for (table,owner) in STAGE_OWNED_TABLE_NAMES {
                    if *owner=="syntax-spans" {
                        transaction.execute(format!(r#"DELETE FROM "{table}" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND stage_unit_work_key=?"#),
                            turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone(),unit.to_owned()]).await?;
                    }
                }
                for batch in rows.chunks(limit) {
                    let records:Vec<_>=batch.iter().map(|row|(&row.stage_row_record_key,&row.stage_row_payload_text)).collect();
                    let records=serde_json::to_string(&records).map_err(|error|turso::Error::ConversionFailure(error.to_string()))?;
                    transaction.execute(r#"INSERT INTO "body-site-reference-table" SELECT ?1,?2,json_extract(value,'$[0]'),?3,json_extract(value,'$[1]') FROM json_each(?4)"#,
                        turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone(),unit.to_owned(),records]).await?;
                }
                transaction.execute(r#"INSERT OR REPLACE INTO "stage-run-generation-records" VALUES (?,?,?,?,?)"#,
                    turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone(),format!("__marker__/syntax-spans/{unit}"),unit.to_owned(),r#"{"status":"complete"}"#.to_owned()]).await?;
                Ok::<_,turso::Error>(())
            }.await;
            match result{Ok(())=>transaction.commit().await,Err(error)=>{let _=transaction.rollback().await;Err(error)}}
        })
    }

    #[test]
    fn test_json_bulk_payloads_roundtrip() {
        let store = TursoWorkspaceStoreAdapter::open_turso_workspace_store(":memory:").unwrap();
        store.initialize_workspace_stage_schema().unwrap();
        let scope = StageRunScopeValue {
            run_generation_digest_value: "g".repeat(64),
            stage_attempt_sequence_value: "a".repeat(64),
        };
        let rows = vec![
            StageTableRowRecord {
                stage_owned_table_name: "body-site-reference-table".into(),
                stage_row_record_key: "0\0\"雪".into(),
                stage_row_payload_text: "a\n\t\0\"😀".into(),
            },
            StageTableRowRecord {
                stage_owned_table_name: "body-site-reference-table".into(),
                stage_row_record_key: "1".into(),
                stage_row_payload_text: String::new(),
            },
        ];
        write_fixture_json_units(&store, &scope, "unit", &rows, 1024).unwrap();
        assert_eq!(
            store
                .read_stage_table_rows(&scope, "body-site-reference-table")
                .unwrap(),
            rows
        );
        assert!(
            write_fixture_json_units(
                &store,
                &scope,
                "unit",
                &[rows[0].clone(), rows[0].clone()],
                1024
            )
            .is_err()
        );
        assert_eq!(
            store
                .read_stage_table_rows(&scope, "body-site-reference-table")
                .unwrap(),
            rows
        );
        write_fixture_json_units(&store, &scope, "unit", &[], 1024).unwrap();
        assert_eq!(
            store
                .count_stage_table_rows(&scope, "body-site-reference-table")
                .unwrap(),
            0
        );
    }

    #[test]
    #[ignore = "manual JSON bulk insert profile"]
    fn test_profile_json_bulk_batches() {
        for limit in [128, 1024] {
            let store = TursoWorkspaceStoreAdapter::open_turso_workspace_store(":memory:").unwrap();
            store.initialize_workspace_stage_schema().unwrap();
            let scope = StageRunScopeValue {
                run_generation_digest_value: "g".repeat(64),
                stage_attempt_sequence_value: "a".repeat(64),
            };
            let started = std::time::Instant::now();
            for unit in 0..12u64 {
                let rows: Vec<_> = (0..1024u64)
                    .map(|row| {
                        let mut hash = std::collections::hash_map::DefaultHasher::new();
                        (unit, row).hash(&mut hash);
                        StageTableRowRecord {
                            stage_owned_table_name: "body-site-reference-table".into(),
                            stage_row_record_key: format!(
                                "{:016x}{unit:024x}{row:024x}",
                                hash.finish()
                            ),
                            stage_row_payload_text: "p".repeat(1024),
                        }
                    })
                    .collect();
                write_fixture_json_units(&store, &scope, &format!("{unit:064x}"), &rows, limit)
                    .unwrap();
            }
            eprintln!(
                "stage=json-bulk-profile batch_limit={limit} rows={} elapsed_seconds={:.6}",
                12 * 1024,
                started.elapsed().as_secs_f64()
            );
        }
    }

    #[test]
    #[ignore = "manual bounded bulk insert profile"]
    fn test_profile_bulk_batch_sizes() {
        for limit in [32, 64, 128, 256, 512, 1024] {
            let mut store =
                TursoWorkspaceStoreAdapter::open_turso_workspace_store(":memory:").unwrap();
            store.stage_insert_batch_limit = std::num::NonZeroUsize::new(limit).unwrap();
            store.initialize_workspace_stage_schema().unwrap();
            let scope = StageRunScopeValue {
                run_generation_digest_value: "g".repeat(64),
                stage_attempt_sequence_value: "a".repeat(64),
            };
            let started = std::time::Instant::now();
            for unit in 0..12u64 {
                let rows: Vec<_> = (0..1024u64)
                    .map(|row| {
                        let mut hash = std::collections::hash_map::DefaultHasher::new();
                        (unit, row).hash(&mut hash);
                        StageTableRowRecord {
                            stage_owned_table_name: "body-site-reference-table".into(),
                            stage_row_record_key: format!(
                                "{:016x}{unit:024x}{row:024x}",
                                hash.finish()
                            ),
                            stage_row_payload_text: "p".repeat(1024),
                        }
                    })
                    .collect();
                store
                    .write_stage_table_unit(
                        &StageUnitWriteRequest {
                            stage_run_scope_value: &scope,
                            stage_owner_name_value: "syntax-spans",
                            stage_unit_work_key: &format!("{unit:064x}"),
                            stage_input_stage_names: &[],
                            stage_unit_output_rows: &rows,
                        },
                        &RunCancelFlagValue::default(),
                    )
                    .unwrap();
            }
            eprintln!(
                "stage=bulk-profile batch_limit={limit} rows={} elapsed_seconds={:.6}",
                12 * 1024,
                started.elapsed().as_secs_f64()
            );
        }
    }

    #[test]
    #[ignore = "manual bounded transaction profile"]
    fn test_profile_bounded_unit_sizes() {
        for unit_size in [64, 256, 1024, 4096u64] {
            let store = TursoWorkspaceStoreAdapter::open_turso_workspace_store(":memory:").unwrap();
            store.initialize_workspace_stage_schema().unwrap();
            let scope = StageRunScopeValue {
                run_generation_digest_value: "g".repeat(64),
                stage_attempt_sequence_value: "a".repeat(64),
            };
            let started = std::time::Instant::now();
            for unit in 0..12288 / unit_size {
                let rows: Vec<_> = (0..unit_size)
                    .map(|row| {
                        let mut hash = std::collections::hash_map::DefaultHasher::new();
                        (unit, row).hash(&mut hash);
                        StageTableRowRecord {
                            stage_owned_table_name: "body-site-reference-table".into(),
                            stage_row_record_key: format!(
                                "{:016x}{unit:024x}{row:024x}",
                                hash.finish()
                            ),
                            stage_row_payload_text: "p".repeat(1024),
                        }
                    })
                    .collect();
                store
                    .write_stage_table_unit(
                        &StageUnitWriteRequest {
                            stage_run_scope_value: &scope,
                            stage_owner_name_value: "syntax-spans",
                            stage_unit_work_key: &format!("{unit:064x}"),
                            stage_input_stage_names: &[],
                            stage_unit_output_rows: &rows,
                        },
                        &RunCancelFlagValue::default(),
                    )
                    .unwrap();
            }
            eprintln!(
                "stage=unit-size-profile unit_size={unit_size} rows=12288 elapsed_seconds={:.6}",
                started.elapsed().as_secs_f64()
            );
        }
    }

    #[test]
    fn test_stage_unit_cleanup_indexed() {
        let store = TursoWorkspaceStoreAdapter::open_turso_workspace_store(":memory:").unwrap();
        store.initialize_workspace_stage_schema().unwrap();
        store.store_runtime_handle_value.block_on(async {
            let connection = store.store_database_handle_value.connect().unwrap();
            let mut rows = connection.query(
                "EXPLAIN QUERY PLAN DELETE FROM \"body-site-reference-table\" WHERE run_generation_digest_value='generation' AND stage_attempt_sequence_value='attempt' AND stage_unit_work_key='unit'", ()
            ).await.unwrap();
            let mut plan = String::new();
            while let Some(row) = rows.next().await.unwrap() {
                plan.push_str(&row.get::<String>(3).unwrap());
            }
            assert!(plan.contains("stage_unit_work_key"), "{plan}");
        });
    }
}
