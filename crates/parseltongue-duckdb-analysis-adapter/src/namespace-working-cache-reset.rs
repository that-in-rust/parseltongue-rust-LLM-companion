use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::StoreBackendFailureError;

impl DuckdbAnalysisStoreAdapter {
    /// Discards a disposable working cache, never a published analysis file.
    pub fn discard_unpublished_analysis_inputs(&self) -> Result<(), StoreBackendFailureError> {
        let connection = self._connection.lock().map_err(build_analysis_error)?;
        let tables = {
            let mut statement=connection.prepare("SELECT database_name,schema_name,table_name FROM duckdb_tables() WHERE starts_with(table_name,'namespace-') OR starts_with(table_name,'frozen-generation-') ORDER BY database_name,schema_name,table_name").map_err(build_analysis_error)?;
            statement
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .map_err(build_analysis_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(build_analysis_error)?
        };
        tables
            .into_iter()
            .try_for_each(|(database, schema, table)| {
                let name = [database, schema, table]
                    .into_iter()
                    .map(|part| format!("\"{}\"", part.replace('"', "\"\"")))
                    .collect::<Vec<_>>()
                    .join(".");
                connection
                    .execute_batch(&format!("DROP TABLE {name}"))
                    .map_err(build_analysis_error)
            })?;
        connection
            .execute_batch("CHECKPOINT")
            .map_err(build_analysis_error)
    }
}
