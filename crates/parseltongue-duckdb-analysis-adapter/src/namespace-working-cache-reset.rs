use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::StoreBackendFailureError;

impl DuckdbAnalysisStoreAdapter {
    /// Discards a disposable working cache, never a published analysis file.
    pub fn discard_unpublished_analysis_inputs(&self) -> Result<(), StoreBackendFailureError> {
        Err(build_analysis_error(
            "working analysis cache reset not implemented",
        ))
    }
}
