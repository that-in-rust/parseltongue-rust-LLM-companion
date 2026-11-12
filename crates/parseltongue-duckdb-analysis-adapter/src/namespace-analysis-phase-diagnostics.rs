use super::DuckdbAnalysisStoreAdapter;
use parseltongue_core_row_types::StoreBackendFailureError;

#[derive(Debug, thiserror::Error)]
pub enum NamespaceAnalysisPhaseError {
    #[error("namespace copy failed: {source}")]
    Copy {
        #[source]
        source: StoreBackendFailureError,
    },
    #[error("namespace normalization failed: {source}")]
    Normalization {
        #[source]
        source: StoreBackendFailureError,
    },
    #[error("namespace resolution failed: {source}")]
    Resolution {
        #[source]
        source: StoreBackendFailureError,
    },
    #[error(transparent)]
    Store(#[from] StoreBackendFailureError),
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use std::error::Error;

    /// REQ-RUST-024.7
    #[test]
    fn test_namespace_probe_settings_limits() {
        let directory = tempfile::tempdir().unwrap();
        let analysis = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
            &directory.path().join("probe.duckdb"),
            directory.path(),
            1288490188,
        )
        .unwrap();
        for threads in [1, 2] {
            analysis
                .configure_namespace_probe_settings(threads)
                .unwrap();
            let memory = analysis.read_namespace_memory_usage().unwrap();
            assert_eq!(memory["settings"]["threads"], threads);
            assert_eq!(memory["settings"]["preserve_insertion_order"], false);
            assert_eq!(memory["settings"]["memory_limit"], "1.1 GiB");
            assert!(memory["settings"]["version"].is_string());
        }
        assert!(analysis.configure_namespace_probe_settings(0).is_err());
        assert!(analysis.configure_namespace_probe_settings(3).is_err());
    }

    /// REQ-RUST-024.6
    #[test]
    fn test_namespace_phase_failure_context() {
        let directory = tempfile::tempdir().unwrap();
        let analysis = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
            &directory.path().join("phases.duckdb"),
            directory.path(),
            1288490188,
        )
        .unwrap();
        [
            NamespaceAnalysisPhaseKind::Copy,
            NamespaceAnalysisPhaseKind::Normalization,
            NamespaceAnalysisPhaseKind::Resolution,
        ]
        .into_iter()
        .for_each(|phase| {
            let error = analysis
                .measure_namespace_phase_work::<()>(phase, || {
                    Err(super::super::build_analysis_error("forced phase failure"))
                })
                .unwrap_err();
            assert!(matches!(
                (phase, &error),
                (
                    NamespaceAnalysisPhaseKind::Copy,
                    NamespaceAnalysisPhaseError::Copy { .. }
                ) | (
                    NamespaceAnalysisPhaseKind::Normalization,
                    NamespaceAnalysisPhaseError::Normalization { .. }
                ) | (
                    NamespaceAnalysisPhaseKind::Resolution,
                    NamespaceAnalysisPhaseError::Resolution { .. }
                )
            ));
            assert!(
                error
                    .source()
                    .unwrap()
                    .to_string()
                    .contains("forced phase failure")
            );
        });
    }

    /// REQ-RUST-024.6
    #[test]
    fn test_namespace_phase_empty_success() {
        let directory = tempfile::tempdir().unwrap();
        let analysis = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
            &directory.path().join("phases.duckdb"),
            directory.path(),
            1288490188,
        )
        .unwrap();
        assert_eq!(
            analysis
                .measure_namespace_phase_work(NamespaceAnalysisPhaseKind::Copy, || Ok(0))
                .unwrap(),
            0
        );
        let memory = analysis.read_namespace_memory_usage().unwrap();
        assert!(memory["memory_usage_bytes"].is_u64());
        assert!(memory["temporary_storage_bytes"].is_u64());
        assert!(!memory["databases"].as_array().unwrap().is_empty());
    }

    /// REQ-RUST-024.6
    #[test]
    fn test_namespace_telemetry_failure_preserves() {
        let directory = tempfile::tempdir().unwrap();
        let analysis = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
            &directory.path().join("phases.duckdb"),
            directory.path(),
            1288490188,
        )
        .unwrap();
        let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = analysis._connection.lock().unwrap();
            panic!("forced telemetry failure");
        }));
        assert!(poisoned.is_err());
        let error = analysis
            .measure_namespace_phase_work::<()>(NamespaceAnalysisPhaseKind::Resolution, || {
                Err(super::super::build_analysis_error("original failure"))
            })
            .unwrap_err();
        assert!(matches!(
            error,
            NamespaceAnalysisPhaseError::Resolution { .. }
        ));
        assert!(
            error
                .source()
                .unwrap()
                .to_string()
                .contains("original failure")
        );
    }
    /// REQ-RUST-024.7
    #[test]
    fn test_namespace_materialization_probe_rollback() {
        let directory = tempfile::tempdir().unwrap();
        let analysis = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
            &directory.path().join("probe.duckdb"),
            directory.path(),
            1288490188,
        )
        .unwrap();
        analysis._connection.lock().unwrap().execute_batch("CREATE TABLE \"namespace-path-candidate-records\" (generation_key VARCHAR,attempt_key VARCHAR,target_key VARCHAR PRIMARY KEY); INSERT INTO \"namespace-path-candidate-records\" VALUES ('same','same','original');").unwrap();
        let scope = parseltongue_core_row_types::StageRunScopeValue {
            run_generation_digest_value: "same".into(),
            stage_attempt_sequence_value: "same".into(),
        };
        for query in [
            "SELECT ? AS generation_key,? AS attempt_key,'replacement' AS target_key",
            "SELECT ? AS generation_key,? AS attempt_key,'replacement' AS target_key WHERE false",
            "SELECT ? AS generation_key,? AS attempt_key,CASE WHEN EXISTS(SELECT 1 FROM \"namespace-path-candidate-records\" WHERE target_key='original') THEN 'replacement' ELSE error('source deleted before materialization') END AS target_key",
        ] {
            assert_eq!(
                analysis
                    .explain_namespace_materialized_insert(query, &scope)
                    .unwrap()
                    .len(),
                2
            );
        }
        assert!(analysis.explain_namespace_materialized_insert("SELECT ? AS generation_key,? AS attempt_key,'duplicate' AS target_key FROM range(2)",&scope).is_err());
        let targets:String=analysis._connection.lock().unwrap().query_row("SELECT string_agg(target_key,',' ORDER BY target_key) FROM \"namespace-path-candidate-records\"",[],|row|row.get(0)).unwrap();
        assert_eq!(targets, "original");
    }

    /// REQ-RUST-024.6
    #[test]
    fn test_namespace_insert_plan_rollback() {
        let directory = tempfile::tempdir().unwrap();
        let analysis = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
            &directory.path().join("phases.duckdb"),
            directory.path(),
            1288490188,
        )
        .unwrap();
        analysis._connection.lock().unwrap().execute_batch("CREATE TABLE \"namespace-path-candidate-records\" (generation_key VARCHAR,attempt_key VARCHAR,target_key VARCHAR); INSERT INTO \"namespace-path-candidate-records\" VALUES ('same','same','original');").unwrap();
        let scope = parseltongue_core_row_types::StageRunScopeValue {
            run_generation_digest_value: "same".into(),
            stage_attempt_sequence_value: "same".into(),
        };
        let plan = analysis
            .explain_namespace_insert_plan(
                "INSERT INTO \"namespace-path-candidate-records\" SELECT ?,?,'replacement'",
                &scope,
            )
            .unwrap()
            .join("\n");
        assert!(plan.to_ascii_uppercase().contains("INSERT"));
        assert!(
            analysis
                .explain_namespace_insert_plan("INSERT INTO absent_source SELECT ?,?", &scope)
                .is_err()
        );
        let targets: String = analysis._connection.lock().unwrap().query_row("SELECT string_agg(target_key,',' ORDER BY target_key) FROM \"namespace-path-candidate-records\"",[],|row|row.get(0)).unwrap();
        assert_eq!(targets, "original");
    }

    /// REQ-RUST-024.6
    #[test]
    fn test_namespace_query_explain_rows() {
        let directory = tempfile::tempdir().unwrap();
        let analysis = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
            &directory.path().join("phases.duckdb"),
            directory.path(),
            1288490188,
        )
        .unwrap();
        let scope = parseltongue_core_row_types::StageRunScopeValue {
            run_generation_digest_value: "same".into(),
            stage_attempt_sequence_value: "same".into(),
        };
        let plan = analysis
            .explain_namespace_query_plan("SELECT * FROM range(3) WHERE ?=?", &scope, true)
            .unwrap()
            .join("\n");
        assert!(plan.contains("3 rows"), "{plan}");
        let empty = analysis
            .explain_namespace_query_plan("SELECT * FROM range(3) WHERE ?<>?", &scope, true)
            .unwrap()
            .join("\n");
        assert!(empty.contains("0 rows"), "{empty}");
    }

    /// REQ-RUST-024.6
    #[test]
    fn test_namespace_query_explain_error() {
        let directory = tempfile::tempdir().unwrap();
        let analysis = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
            &directory.path().join("phases.duckdb"),
            directory.path(),
            1288490188,
        )
        .unwrap();
        let scope = parseltongue_core_row_types::StageRunScopeValue {
            run_generation_digest_value: "same".into(),
            stage_attempt_sequence_value: "same".into(),
        };
        assert!(
            analysis
                .explain_namespace_query_plan(
                    "SELECT absent FROM missing_source WHERE ?=?",
                    &scope,
                    true
                )
                .is_err()
        );
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NamespaceAnalysisPhaseKind {
    Copy,
    Normalization,
    Resolution,
}

impl DuckdbAnalysisStoreAdapter {
    pub fn explain_namespace_materialized_insert(
        &self,
        query: &str,
        scope: &parseltongue_core_row_types::StageRunScopeValue,
    ) -> Result<Vec<String>, StoreBackendFailureError> {
        let connection = self
            ._connection
            .lock()
            .map_err(super::build_analysis_error)?;
        let transaction = connection
            .unchecked_transaction()
            .map_err(super::build_analysis_error)?;
        let queries=[
            format!("CREATE OR REPLACE TEMP TABLE \"namespace-materialized-path-candidates\" AS {query}"),
            "INSERT INTO \"namespace-path-candidate-records\" SELECT * FROM \"namespace-materialized-path-candidates\" WHERE generation_key=? AND attempt_key=?".into(),
        ];
        let plans=queries.iter().enumerate().try_fold(Vec::new(),|mut plans,(index,query)|{
            if index==1 {
                transaction.execute("DELETE FROM \"namespace-path-candidate-records\" WHERE generation_key=? AND attempt_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(super::build_analysis_error)?;
            }
            eprintln!("stage=namespace-materialization-probe operation={index} event=started");
            let started=std::time::Instant::now();
            let result=read_namespace_explain_rows(&transaction,query,scope,true);
            eprintln!("stage=namespace-materialization-probe operation={index} event=finished status={} elapsed_seconds={:.3}",if result.is_ok(){"complete"}else{"failed"},started.elapsed().as_secs_f64());
            plans.extend(result?);
            Ok::<_,StoreBackendFailureError>(plans)
        })?;
        transaction
            .rollback()
            .map_err(super::build_analysis_error)?;
        Ok(plans)
    }

    pub fn configure_namespace_probe_settings(
        &self,
        threads: u32,
    ) -> Result<(), StoreBackendFailureError> {
        if !(1..=2).contains(&threads) {
            return Err(super::build_analysis_error(
                "namespace probe threads must be one or two",
            ));
        }
        let connection = self
            ._connection
            .lock()
            .map_err(super::build_analysis_error)?;
        connection
            .execute_batch("SET preserve_insertion_order=false")
            .map_err(super::build_analysis_error)?;
        connection
            .execute("SET threads = ?", [threads])
            .map(|_| ())
            .map_err(super::build_analysis_error)
    }

    pub fn explain_namespace_insert_plan(
        &self,
        query: &str,
        scope: &parseltongue_core_row_types::StageRunScopeValue,
    ) -> Result<Vec<String>, StoreBackendFailureError> {
        let connection = self
            ._connection
            .lock()
            .map_err(super::build_analysis_error)?;
        let transaction = connection
            .unchecked_transaction()
            .map_err(super::build_analysis_error)?;
        transaction.execute("DELETE FROM \"namespace-path-candidate-records\" WHERE generation_key=? AND attempt_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(super::build_analysis_error)?;
        let plan = read_namespace_explain_rows(&transaction, query, scope, true)?;
        transaction
            .rollback()
            .map_err(super::build_analysis_error)?;
        Ok(plan)
    }

    pub fn explain_namespace_query_plan(
        &self,
        query: &str,
        scope: &parseltongue_core_row_types::StageRunScopeValue,
        analyze: bool,
    ) -> Result<Vec<String>, StoreBackendFailureError> {
        let connection = self
            ._connection
            .lock()
            .map_err(super::build_analysis_error)?;
        read_namespace_explain_rows(&connection, query, scope, analyze)
    }

    pub fn measure_namespace_phase_work<T>(
        &self,
        phase: NamespaceAnalysisPhaseKind,
        action: impl FnOnce() -> Result<T, StoreBackendFailureError>,
    ) -> Result<T, NamespaceAnalysisPhaseError> {
        let label = match phase {
            NamespaceAnalysisPhaseKind::Copy => "copy",
            NamespaceAnalysisPhaseKind::Normalization => "normalization",
            NamespaceAnalysisPhaseKind::Resolution => "resolution",
        };
        let memory = self.read_namespace_memory_usage();
        eprintln!(
            "stage=namespace-{label} event=started memory={}",
            memory
                .unwrap_or_else(|error| serde_json::json!({"measurement_error":error.to_string()}))
        );
        let started = std::time::Instant::now();
        let result = action().map_err(|source| match phase {
            NamespaceAnalysisPhaseKind::Copy => NamespaceAnalysisPhaseError::Copy { source },
            NamespaceAnalysisPhaseKind::Normalization => {
                NamespaceAnalysisPhaseError::Normalization { source }
            }
            NamespaceAnalysisPhaseKind::Resolution => {
                NamespaceAnalysisPhaseError::Resolution { source }
            }
        });
        let elapsed = started.elapsed().as_secs_f64();
        let memory = self.read_namespace_memory_usage();
        eprintln!(
            "stage=namespace-{label} event=finished status={} elapsed_seconds={elapsed:.3} memory={}",
            if result.is_ok() { "complete" } else { "failed" },
            memory
                .unwrap_or_else(|error| serde_json::json!({"measurement_error":error.to_string()}))
        );
        result
    }

    pub fn read_namespace_memory_usage(
        &self,
    ) -> Result<serde_json::Value, StoreBackendFailureError> {
        let connection = self
            ._connection
            .lock()
            .map_err(super::build_analysis_error)?;
        let (memory, temporary): (u64,u64) = connection.query_row(
            "SELECT coalesce(sum(memory_usage_bytes),0)::UBIGINT,coalesce(sum(temporary_storage_bytes),0)::UBIGINT FROM duckdb_memory()",
            [], |row| Ok((row.get(0)?,row.get(1)?)),
        ).map_err(super::build_analysis_error)?;
        let (threads, insertion_order, limit, version):(u64,bool,String,String)=connection.query_row(
            "SELECT current_setting('threads')::UBIGINT,current_setting('preserve_insertion_order')::BOOLEAN,current_setting('memory_limit'),version()",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)),
        ).map_err(super::build_analysis_error)?;
        let mut statement = connection.prepare("SELECT database_name,database_size,memory_usage,memory_limit FROM pragma_database_size() ORDER BY database_name").map_err(super::build_analysis_error)?;
        let databases = statement
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "database_name":row.get::<_,String>(0)?,"database_size":row.get::<_,String>(1)?,
                    "memory_usage":row.get::<_,String>(2)?,"memory_limit":row.get::<_,String>(3)?
                }))
            })
            .map_err(super::build_analysis_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(super::build_analysis_error)?;
        let tags =
            super::namespace_storage_size_diagnostics::read_namespace_memory_tags(&connection)?;
        Ok(
            serde_json::json!({"memory_usage_bytes":memory,"temporary_storage_bytes":temporary,"databases":databases,"tags":tags,"settings":{"threads":threads,"preserve_insertion_order":insertion_order,"memory_limit":limit,"version":version}}),
        )
    }
}

fn read_namespace_explain_rows(
    connection: &duckdb::Connection,
    query: &str,
    scope: &parseltongue_core_row_types::StageRunScopeValue,
    analyze: bool,
) -> Result<Vec<String>, StoreBackendFailureError> {
    let command = if analyze {
        "EXPLAIN ANALYZE"
    } else {
        "EXPLAIN"
    };
    let mut statement = connection
        .prepare(&format!("{command} {query}"))
        .map_err(super::build_analysis_error)?;
    statement
        .query_map(
            duckdb::params![
                scope.run_generation_digest_value,
                scope.stage_attempt_sequence_value
            ],
            |row| row.get(1),
        )
        .map_err(super::build_analysis_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(super::build_analysis_error)
}
