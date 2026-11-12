use super::build_pipeline_stage_error;
use parseltongue_core_row_types::{RunCancelFlagValue, StoreBackendFailureError};
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::Value;
use std::path::Path;

pub(crate) fn run_oracle_agreement_comparison(
    _source: &dyn StageTableStorePort,
    _primary: &Path,
    _secondary: &Path,
    _agreement: &dyn StageTableStorePort,
    _cancel: &RunCancelFlagValue,
) -> Result<Value, StoreBackendFailureError> {
    Err(build_pipeline_stage_error(
        "oracle path comparison not implemented",
    ))
}
