use super::DuckdbAnalysisStoreAdapter;
use parseltongue_core_row_types::StoreBackendFailureError;
use serde_json::{Value, json};
use std::collections::BTreeMap;

impl DuckdbAnalysisStoreAdapter {
    pub fn checkpoint_namespace_analysis_store(&self) -> Result<(), StoreBackendFailureError> {
        self._connection
            .lock()
            .map_err(super::build_analysis_error)?
            .execute_batch("CHECKPOINT")
            .map_err(super::build_analysis_error)
    }

    pub fn read_namespace_storage_usage(
        &self,
    ) -> Result<serde_json::Value, StoreBackendFailureError> {
        let connection = self
            ._connection
            .lock()
            .map_err(super::build_analysis_error)?;
        let memory_tags = read_namespace_memory_tags(&connection)?;
        let mut statement=connection.prepare("SELECT s.database_name,d.path,s.database_size,s.block_size,s.total_blocks,s.used_blocks,s.free_blocks,s.wal_size,s.memory_usage,s.memory_limit FROM pragma_database_size() s JOIN duckdb_databases() d USING(database_name) ORDER BY s.database_name").map_err(super::build_analysis_error)?;
        let databases=statement.query_map([],|row|Ok(json!({"database_name":row.get::<_,String>(0)?,"path":row.get::<_,Option<String>>(1)?,"database_size":row.get::<_,String>(2)?,"block_size":row.get::<_,u64>(3)?,"total_blocks":row.get::<_,u64>(4)?,"used_blocks":row.get::<_,u64>(5)?,"free_blocks":row.get::<_,u64>(6)?,"wal_size":row.get::<_,String>(7)?,"memory_usage":row.get::<_,String>(8)?,"memory_limit":row.get::<_,String>(9)?}))).map_err(super::build_analysis_error)?.collect::<Result<Vec<_>,_>>().map_err(super::build_analysis_error)?;
        let mut statement=connection.prepare("SELECT table_name,estimated_size FROM duckdb_tables() WHERE database_name=current_database() AND schema_name='main' AND NOT internal ORDER BY table_name").map_err(super::build_analysis_error)?;
        let tables = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
            })
            .map_err(super::build_analysis_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(super::build_analysis_error)?;
        let mut column_widths = Vec::new();
        let mut storage_segments = Vec::new();
        for (table, estimated_rows) in &tables {
            let mut statement = connection
                .prepare("SELECT name,type FROM pragma_table_info(?) ORDER BY cid")
                .map_err(super::build_analysis_error)?;
            let columns = statement
                .query_map([table], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(super::build_analysis_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(super::build_analysis_error)?;
            for (column, data_type) in columns {
                let sql = format!(
                    "SELECT coalesce(sum(octet_length(encode(CAST(\"{}\" AS VARCHAR)))),0)::UBIGINT,coalesce(max(octet_length(encode(CAST(\"{}\" AS VARCHAR)))),0)::UBIGINT FROM \"{}\"",
                    column.replace('"', "\"\""),
                    column.replace('"', "\"\""),
                    table.replace('"', "\"\"")
                );
                let (bytes, maximum): (u64, u64) = connection
                    .query_row(&sql, [], |row| Ok((row.get(0)?, row.get(1)?)))
                    .map_err(super::build_analysis_error)?;
                column_widths.push(json!({"table_name":table,"estimated_rows":estimated_rows,"column_name":column,"data_type":data_type,"logical_bytes":bytes,"maximum_value_bytes":maximum}));
            }
            let mut statement=connection.prepare("SELECT column_name,segment_type,compression,persistent,count(*)::UBIGINT,sum(count)::UBIGINT,count(DISTINCT CASE WHEN block_id>=0 THEN block_id END)::UBIGINT FROM pragma_storage_info(?) GROUP BY column_name,segment_type,compression,persistent ORDER BY column_name,segment_type,compression,persistent").map_err(super::build_analysis_error)?;
            storage_segments.extend(statement.query_map([table],|row|Ok(json!({"table_name":table,"column_name":row.get::<_,String>(0)?,"segment_type":row.get::<_,String>(1)?,"compression":row.get::<_,String>(2)?,"persistent":row.get::<_,bool>(3)?,"segment_count":row.get::<_,u64>(4)?,"value_count":row.get::<_,u64>(5)?,"distinct_blocks":row.get::<_,u64>(6)?}))).map_err(super::build_analysis_error)?.collect::<Result<Vec<_>,_>>().map_err(super::build_analysis_error)?);
        }
        let (source_tables, source_fields) = if tables
            .iter()
            .any(|(name, _)| name == "frozen-generation-source-records")
        {
            read_namespace_source_widths(&connection)?
        } else {
            (Vec::new(), Vec::new())
        };
        Ok(
            json!({"memory_tags":memory_tags,"databases":databases,"column_widths":column_widths,"storage_segments":storage_segments,"source_tables":source_tables,"source_fields":source_fields,"logical_width_rule":"UTF-8 bytes after VARCHAR cast; estimated_size is row count, not physical bytes; physical blocks may be shared across columns"}),
        )
    }
}

pub(super) fn read_namespace_memory_tags(
    connection: &duckdb::Connection,
) -> Result<Vec<Value>, StoreBackendFailureError> {
    let mut statement=connection.prepare("SELECT tag,memory_usage_bytes,temporary_storage_bytes FROM duckdb_memory() ORDER BY tag").map_err(super::build_analysis_error)?;
    statement.query_map([],|row|Ok(json!({"tag":row.get::<_,String>(0)?,"memory_usage_bytes":row.get::<_,u64>(1)?,"temporary_storage_bytes":row.get::<_,u64>(2)?}))).map_err(super::build_analysis_error)?.collect::<Result<Vec<_>,_>>().map_err(super::build_analysis_error)
}

fn read_namespace_source_widths(
    connection: &duckdb::Connection,
) -> Result<(Vec<Value>, Vec<Value>), StoreBackendFailureError> {
    let mut statement=connection.prepare("SELECT generation_key,attempt_key,table_name,count(*)::UBIGINT,coalesce(sum(octet_length(encode(payload))),0)::UBIGINT,coalesce(max(octet_length(encode(payload))),0)::UBIGINT FROM \"frozen-generation-source-records\" GROUP BY generation_key,attempt_key,table_name ORDER BY generation_key,attempt_key,table_name").map_err(super::build_analysis_error)?;
    let tables = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, u64>(3)?,
                row.get::<_, u64>(4)?,
                row.get::<_, u64>(5)?,
            ))
        })
        .map_err(super::build_analysis_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(super::build_analysis_error)?;
    let mut source_tables = Vec::new();
    let mut source_fields = Vec::new();
    for (generation, attempt, table, count, bytes, maximum) in tables {
        source_tables.push(json!({"generation_key":generation,"attempt_key":attempt,"table_name":table,"row_count":count,"payload_bytes":bytes,"maximum_payload_bytes":maximum}));
        let mut widths: BTreeMap<String, (u64, u64, u64)> = BTreeMap::new();
        let mut previous: Option<String> = None;
        loop {
            let mut statement=connection.prepare("SELECT row_key,payload FROM \"frozen-generation-source-records\" WHERE generation_key=? AND attempt_key=? AND table_name=? AND (? IS NULL OR row_key>?) ORDER BY generation_key,attempt_key,table_name,row_key LIMIT 128").map_err(super::build_analysis_error)?;
            let mut rows = statement
                .query(duckdb::params![
                    generation, attempt, table, previous, previous
                ])
                .map_err(super::build_analysis_error)?;
            let mut page = 0;
            while let Some(row) = rows.next().map_err(super::build_analysis_error)? {
                previous = Some(
                    row.get::<_, String>(0)
                        .map_err(super::build_analysis_error)?,
                );
                let payload: String = row.get(1).map_err(super::build_analysis_error)?;
                if let Ok(Value::Object(fields)) = serde_json::from_str::<Value>(&payload) {
                    for (field, value) in fields {
                        let bytes = value.to_string().len() as u64;
                        let entry = widths.entry(field).or_default();
                        entry.0 += bytes;
                        entry.1 = entry.1.max(bytes);
                        entry.2 += 1;
                    }
                }
                page += 1;
            }
            if page < 128 {
                break;
            }
        }
        source_fields.extend(widths.into_iter().map(|(field,(bytes,maximum,rows))|json!({"generation_key":generation,"attempt_key":attempt,"table_name":table,"field_name":field,"json_value_bytes":bytes,"maximum_json_value_bytes":maximum,"row_count":rows})));
    }
    Ok((source_tables, source_fields))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// REQ-RUST-024.8
    #[test]
    fn test_namespace_storage_snapshot_contents() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("storage.duckdb");
        let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
            &path,
            directory.path(),
            1288490188,
        )
        .unwrap();
        store._connection.lock().unwrap().execute_batch("CREATE TABLE \"frozen-generation-source-records\" (generation_key VARCHAR,attempt_key VARCHAR,table_name VARCHAR,row_key VARCHAR,payload VARCHAR); INSERT INTO \"frozen-generation-source-records\" VALUES ('g','a','fixture','k','{\"x\":\"λ\"}');").unwrap();
        store.checkpoint_namespace_analysis_store().unwrap();
        let snapshot = store.read_namespace_storage_usage().unwrap();
        assert_eq!(snapshot["databases"][0]["path"], path.to_str().unwrap());
        assert_eq!(snapshot["source_tables"][0]["payload_bytes"], 10);
        assert!(
            snapshot["storage_segments"]
                .as_array()
                .unwrap()
                .iter()
                .all(|value| value["persistent"] == true)
        );
        assert!(
            snapshot["memory_tags"]
                .as_array()
                .unwrap()
                .iter()
                .all(|value| value["memory_usage_bytes"].is_u64())
        );
        assert!(
            snapshot["column_widths"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value["column_name"] == "payload" && value["logical_bytes"] == 10)
        );
        store
            ._connection
            .lock()
            .unwrap()
            .execute_batch("DROP TABLE \"frozen-generation-source-records\"")
            .unwrap();
        assert_eq!(
            store.read_namespace_storage_usage().unwrap()["source_tables"],
            serde_json::json!([])
        );
    }
}
