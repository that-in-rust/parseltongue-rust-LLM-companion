use super::build_pipeline_stage_error;
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};
use parseltongue_table_store_traits::StageTableStorePort;
use std::path::Path;

pub(crate) fn run_namespace_analysis_pipeline(
    _scope: &StageRunScopeValue,
    _store: &dyn StageTableStorePort,
    _cancel: &RunCancelFlagValue,
    _directory: &Path,
) -> Result<(), StoreBackendFailureError> {
    Err(build_pipeline_stage_error(
        "DuckDB name-resolution stage not implemented",
    ))
}
