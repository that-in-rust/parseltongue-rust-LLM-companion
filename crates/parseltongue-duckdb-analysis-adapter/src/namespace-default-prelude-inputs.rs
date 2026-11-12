use super::build_analysis_error;
use parseltongue_core_row_types::{StageRunScopeValue, StoreBackendFailureError};

pub(super) fn prepare_default_prelude_inputs(
    connection: &duckdb::Connection,
    _scope: &StageRunScopeValue,
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
    if count == 0 {
        Ok(0)
    } else {
        Err(build_analysis_error(
            "default prelude projection not implemented",
        ))
    }
}
