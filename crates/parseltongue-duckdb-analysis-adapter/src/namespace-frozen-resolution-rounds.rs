use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceResolutionRoundReport {
    pub binding_set_digest_value: String,
    pub fixed_point_round_count: usize,
}

impl DuckdbAnalysisStoreAdapter {
    pub fn resolve_frozen_namespace_rows(
        &self,
        _scope: &StageRunScopeValue,
        _cancel: &RunCancelFlagValue,
    ) -> Result<NamespaceResolutionRoundReport, StoreBackendFailureError> {
        Err(build_analysis_error(
            "frozen namespace resolution not implemented",
        ))
    }
}
