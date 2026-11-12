use super::{
    FIXED_POINT_LIMIT, build_analysis_error, namespace_binding_round_strata::recompute_binding_rows,
};
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};

pub fn resolve_import_endpoint_rounds(
    connection: &duckdb::Connection,
    scope: &StageRunScopeValue,
    cancel: &RunCancelFlagValue,
) -> Result<usize, StoreBackendFailureError> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(build_analysis_error)?;
    if cancel.check_stage_work_cancelled() {
        return Err(build_analysis_error("analysis resolution cancelled"));
    }
    transaction.execute_batch("CREATE TABLE IF NOT EXISTS \"namespace-binding-round-inputs\" AS SELECT * FROM \"namespace-binding-declaration-records\" WHERE false")
        .map_err(build_analysis_error)?;
    transaction.execute("DELETE FROM \"namespace-binding-round-inputs\" WHERE generation_key=? AND attempt_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(build_analysis_error)?;
    transaction.execute("INSERT INTO \"namespace-binding-round-inputs\" SELECT * FROM \"namespace-binding-declaration-records\" WHERE generation_key=? AND attempt_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(build_analysis_error)?;
    let mut previous = recompute_binding_rows(&transaction, scope)?;
    for round in 0..=FIXED_POINT_LIMIT {
        if cancel.check_stage_work_cancelled() {
            return Err(build_analysis_error("analysis resolution cancelled"));
        }
        transaction.execute("DELETE FROM \"namespace-binding-round-inputs\" WHERE generation_key=? AND attempt_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(build_analysis_error)?;
        transaction.execute("INSERT INTO \"namespace-binding-round-inputs\"
            SELECT * FROM \"namespace-binding-declaration-records\" WHERE generation_key=? AND attempt_key=?
            UNION ALL
            SELECT i.generation_key,i.attempt_key,i.importer_scope,
                CASE WHEN i.is_glob OR i.alias_name='' THEN b.name ELSE i.alias_name END,
                b.namespace,b.target_key,CASE WHEN i.is_glob THEN 2 ELSE 3 END,i.import_key
            FROM \"namespace-import-endpoint-records\" i
            JOIN \"namespace-binding-round-results\" b ON b.generation_key=i.generation_key AND b.attempt_key=i.attempt_key
                AND b.scope_key=i.source_scope AND (i.is_glob OR b.name=i.source_name)
            WHERE i.generation_key=? AND i.attempt_key=? AND (i.is_glob OR b.state='resolved')
                AND (i.is_glob OR i.alias_name<>'_')",
            duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value,scope.run_generation_digest_value,scope.stage_attempt_sequence_value])
            .map_err(build_analysis_error)?;
        let digest = recompute_binding_rows(&transaction, scope)?;
        if digest == previous {
            transaction.commit().map_err(build_analysis_error)?;
            return Ok(round);
        }
        previous = digest;
    }
    Err(build_analysis_error(
        "NameResolutionStuckError: import fixed point exceeded 8192 changing rounds",
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
            source_scope VARCHAR,source_name VARCHAR,alias_name VARCHAR,is_glob BOOLEAN);",
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
