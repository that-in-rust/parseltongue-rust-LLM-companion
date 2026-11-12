use super::DuckdbAnalysisStoreAdapter;
use parseltongue_core_row_types::{
    StageRunScopeValue, StageTableRowRecord, StoreBackendFailureError,
};

impl DuckdbAnalysisStoreAdapter {
    pub fn visit_frozen_namespace_products(
        &self,
        _scope: &StageRunScopeValue,
        _visit: &mut dyn FnMut(&StageTableRowRecord) -> Result<(), StoreBackendFailureError>,
    ) -> Result<usize, StoreBackendFailureError> {
        Err(super::build_analysis_error(
            "final namespace products not implemented",
        ))
    }
}
