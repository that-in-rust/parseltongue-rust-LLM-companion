use super::build_pipeline_stage_error;
use parseltongue_core_row_types::StoreBackendFailureError;
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::Value;
use std::io::Write;

pub(crate) fn write_published_stage_rows(
    _store: &dyn StageTableStorePort,
    _table: &str,
    _filter: Option<&str>,
    _output: &mut dyn Write,
) -> Result<usize, StoreBackendFailureError> {
    Err(build_pipeline_stage_error(
        "published-stage export not implemented",
    ))
}

pub(crate) fn read_published_unit_manifest(
    _store: &dyn StageTableStorePort,
) -> Result<Value, StoreBackendFailureError> {
    Err(build_pipeline_stage_error(
        "mapped-unit manifest not implemented",
    ))
}
