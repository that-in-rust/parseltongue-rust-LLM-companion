use super::build_analysis_error;
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};

pub fn resolve_import_endpoint_rounds(
    _connection: &duckdb::Connection,
    _scope: &StageRunScopeValue,
    _cancel: &RunCancelFlagValue,
) -> Result<usize, StoreBackendFailureError> {
    Err(build_analysis_error(
        "import endpoint fixed point not implemented",
    ))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn scope() -> StageRunScopeValue {
        StageRunScopeValue {
            run_generation_digest_value: "generation".into(),
            stage_attempt_sequence_value: "attempt".into(),
        }
    }

    fn connection() -> duckdb::Connection {
        let connection = duckdb::Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE \"namespace-binding-declaration-records\" (
            generation_key VARCHAR,attempt_key VARCHAR,scope_key VARCHAR,name VARCHAR,
            namespace VARCHAR,target_key VARCHAR,priority INTEGER,evidence_key VARCHAR);
            CREATE TABLE \"namespace-import-endpoint-records\" (
            generation_key VARCHAR,attempt_key VARCHAR,import_key VARCHAR,importer_scope VARCHAR,
            source_scope VARCHAR,source_name VARCHAR,alias_name VARCHAR,glob BOOLEAN);",
            )
            .unwrap();
        connection
    }

    fn declaration(connection: &duckdb::Connection, scope: &str, name: &str, target: &str) {
        connection.execute("INSERT INTO \"namespace-binding-declaration-records\" VALUES ('generation','attempt',?,?,'type',?,4,'')",duckdb::params![scope,name,target]).unwrap();
    }

    fn import(
        connection: &duckdb::Connection,
        importer: &str,
        source: &str,
        name: &str,
        alias: &str,
        glob: bool,
    ) {
        connection.execute("INSERT INTO \"namespace-import-endpoint-records\" VALUES ('generation','attempt',?,?,?,?,?,?)",duckdb::params![format!("{importer}/{source}/{name}/{alias}"),importer,source,name,alias,glob]).unwrap();
    }

    fn targets(connection: &duckdb::Connection, scope: &str, name: &str) -> Vec<(String, String)> {
        connection.prepare("SELECT DISTINCT target_key,state FROM \"namespace-binding-round-results\" WHERE scope_key=? AND name=? ORDER BY target_key,state").unwrap()
            .query_map(duckdb::params![scope,name],|row|Ok((row.get(0)?,row.get(1)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap()
    }

    #[test]
    fn cyclic_glob_imports_terminate() {
        let connection = connection();
        declaration(&connection, "a", "Thing", "target");
        import(&connection, "a", "b", "", "", true);
        import(&connection, "b", "a", "", "", true);
        let rounds =
            resolve_import_endpoint_rounds(&connection, &scope(), &RunCancelFlagValue::default())
                .unwrap();
        assert!(rounds < 8);
        assert_eq!(
            targets(&connection, "b", "Thing"),
            vec![("target".into(), "resolved".into())]
        );
    }

    #[test]
    fn later_named_import_replaces_earlier_glob() {
        let connection = connection();
        declaration(&connection, "early", "Chosen", "glob-target");
        declaration(&connection, "definition", "Original", "named-target");
        import(&connection, "root", "early", "", "", true);
        import(
            &connection,
            "bridge",
            "definition",
            "Original",
            "Late",
            false,
        );
        import(&connection, "root", "bridge", "Late", "Chosen", false);
        resolve_import_endpoint_rounds(&connection, &scope(), &RunCancelFlagValue::default())
            .unwrap();
        assert_eq!(
            targets(&connection, "root", "Chosen"),
            vec![("named-target".into(), "resolved".into())]
        );
    }

    #[test]
    fn conflicting_globs_remain_ambiguous() {
        let connection = connection();
        declaration(&connection, "left", "Chosen", "left-target");
        declaration(&connection, "right", "Chosen", "right-target");
        import(&connection, "root", "left", "", "", true);
        import(&connection, "root", "right", "", "", true);
        resolve_import_endpoint_rounds(&connection, &scope(), &RunCancelFlagValue::default())
            .unwrap();
        assert_eq!(
            targets(&connection, "root", "Chosen"),
            vec![
                ("left-target".into(), "ambiguous".into()),
                ("right-target".into(), "ambiguous".into())
            ]
        );
    }

    #[test]
    fn empty_and_cancelled_fixed_points() {
        let connection = connection();
        assert!(
            resolve_import_endpoint_rounds(&connection, &scope(), &RunCancelFlagValue::default())
                .unwrap()
                < 3
        );
        let cancel = RunCancelFlagValue::default();
        cancel.request_stage_work_cancellation();
        assert!(resolve_import_endpoint_rounds(&connection, &scope(), &cancel).is_err());
    }
}
