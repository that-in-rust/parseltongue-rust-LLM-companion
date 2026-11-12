use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};

impl DuckdbAnalysisStoreAdapter {
    pub fn normalize_frozen_namespace_rows(
        &self,
        _scope: &StageRunScopeValue,
        _cancel: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        Err(build_analysis_error(
            "frozen namespace normalization not implemented",
        ))
    }
}
