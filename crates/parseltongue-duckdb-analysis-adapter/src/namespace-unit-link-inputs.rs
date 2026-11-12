use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};

impl DuckdbAnalysisStoreAdapter {
    pub fn link_frozen_namespace_units(
        &self,
        _scope: &StageRunScopeValue,
        _cancel: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        Err(build_analysis_error("frozen unit links not implemented"))
    }
}
