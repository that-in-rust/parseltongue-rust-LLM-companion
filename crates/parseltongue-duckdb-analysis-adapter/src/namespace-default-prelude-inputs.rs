use super::build_analysis_error;
use parseltongue_core_row_types::{StageRunScopeValue, StoreBackendFailureError};

pub(super) fn prepare_default_prelude_inputs(
    connection: &duckdb::Connection,
    scope: &StageRunScopeValue,
) -> Result<usize, StoreBackendFailureError> {
    connection.execute_batch("CREATE OR REPLACE TEMP TABLE \"namespace-default-prelude-records\" AS
        SELECT u.generation_key,u.attempt_key,u.unit_key,u.root_scope,u.extern_scope,u.prelude_scope,u.edition,d.root_scope AS source_scope
        FROM \"namespace-active-unit-contexts\" u JOIN \"namespace-dependency-target-records\" d ON d.unit_key=u.unit_key
        WHERE u.implicit_prelude AND NOT u.no_core AND d.sysroot_context AND d.target_key IS NOT NULL
            AND d.name=CASE WHEN u.no_std THEN 'core' ELSE 'std' END;").map_err(build_analysis_error)?;
    let count: usize = connection
        .query_row(
            "SELECT count(*) FROM \"namespace-default-prelude-records\"",
            [],
            |row| row.get(0),
        )
        .map_err(build_analysis_error)?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(build_analysis_error)?;
    ["namespace-path-source-requests","namespace-path-segment-records","namespace-import-request-records","namespace-import-visibility-records"].iter().try_for_each(|table|transaction.execute(&format!("DELETE FROM \"{table}\" WHERE generation_key=? AND attempt_key=? AND starts_with(request_key,'namespace-default-prelude-import/')"),duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map(|_|()).map_err(build_analysis_error))?;
    transaction.execute("DELETE FROM \"namespace-macro-prelude-records\" WHERE generation_key=? AND attempt_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(build_analysis_error)?;
    transaction.execute_batch("INSERT INTO \"namespace-path-source-requests\"
        SELECT generation_key,attempt_key,'namespace-default-prelude-import/'||unit_key,prelude_scope,root_scope,extern_scope,edition,'prelude','any',3,NULL FROM \"namespace-default-prelude-records\";
        INSERT INTO \"namespace-path-segment-records\"
        SELECT generation_key,attempt_key,'namespace-default-prelude-import/'||unit_key,0,'defining',source_scope FROM \"namespace-default-prelude-records\"
        UNION ALL SELECT generation_key,attempt_key,'namespace-default-prelude-import/'||unit_key,1,'name','prelude' FROM \"namespace-default-prelude-records\"
        UNION ALL SELECT generation_key,attempt_key,'namespace-default-prelude-import/'||unit_key,2,'name','rust_'||edition::VARCHAR FROM \"namespace-default-prelude-records\";
        INSERT INTO \"namespace-import-request-records\"
        SELECT generation_key,attempt_key,'namespace-default-prelude-import/'||unit_key,prelude_scope,NULL,true FROM \"namespace-default-prelude-records\";
        INSERT INTO \"namespace-import-visibility-records\"
        SELECT generation_key,attempt_key,'namespace-default-prelude-import/'||unit_key,NULL FROM \"namespace-default-prelude-records\";
        INSERT INTO \"namespace-macro-prelude-records\"
        SELECT generation_key,attempt_key,prelude_scope,source_scope,'namespace-default-macro-prelude/'||unit_key FROM \"namespace-default-prelude-records\";").map_err(build_analysis_error)?;
    transaction.commit().map_err(build_analysis_error)?;
    Ok(count)
}
