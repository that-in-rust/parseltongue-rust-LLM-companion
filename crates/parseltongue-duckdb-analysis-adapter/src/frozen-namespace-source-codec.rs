use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};

pub const NAMESPACE_SOURCE_TABLE_NAMES: &[&str] = &[
    "crate-dependency-graph-table",
    "syntax-namespace-input-scopes",
    "syntax-namespace-input-items",
    "syntax-namespace-input-imports",
    "syntax-namespace-input-paths",
    "syntax-namespace-input-macros",
    "syntax-namespace-input-externs",
    "legacy-macro-source-events",
    "legacy-macro-scope-records",
    "lang-item-definition-table",
];

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
