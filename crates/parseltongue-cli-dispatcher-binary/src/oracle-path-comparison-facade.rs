use super::build_pipeline_stage_error;
use parseltongue_core_row_types::{RunCancelFlagValue, StoreBackendFailureError};
use parseltongue_duckdb_analysis_adapter::compare_oracle_path_sites;
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::Value;
use std::path::Path;

pub(crate) fn run_oracle_agreement_comparison(
    source: &dyn StageTableStorePort,
    primary: &Path,
    secondary: &Path,
    agreement: &dyn StageTableStorePort,
    cancel: &RunCancelFlagValue,
) -> Result<Value, StoreBackendFailureError> {
    compare_oracle_path_sites(source, primary, secondary, agreement, cancel)
        .map_err(|error| build_pipeline_stage_error(error.to_string()))
}
