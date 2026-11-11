use parseltongue_core_row_types::{RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StageUnitWriteRequest, StoreBackendFailureError};
use parseltongue_table_store_traits::StageTableStorePort;
use super::TursoWorkspaceStoreAdapter;

pub fn retry_bounded_store_write<T>(_stage_cancel_flag_value: &RunCancelFlagValue, _store_write_attempt_function: impl FnMut() -> Result<T, turso::Error>) -> Result<T, StoreBackendFailureError> { todo!() }

impl StageTableStorePort for TursoWorkspaceStoreAdapter {
    fn initialize_workspace_stage_schema(&self) -> Result<(), StoreBackendFailureError> { todo!() }
    fn write_stage_table_unit(&self, _stage_unit_write_request: &StageUnitWriteRequest<'_>, _stage_cancel_flag_value: &RunCancelFlagValue) -> Result<usize, StoreBackendFailureError> { todo!() }
    fn read_stage_table_rows(&self, _stage_run_scope_value: &StageRunScopeValue, _stage_owned_table_name: &str) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> { todo!() }
    fn write_file_content_bytes(&self, _file_content_digest_full: &str, _file_source_content_bytes: &[u8]) -> Result<bool, StoreBackendFailureError> { todo!() }
    fn read_file_content_bytes(&self, _file_content_digest_full: &str) -> Result<Vec<u8>, StoreBackendFailureError> { todo!() }
    fn publish_workspace_generation_manifest(&self, _stage_run_scope_value: &StageRunScopeValue, _run_generation_digest_value: &str, _stage_cancel_flag_value: &RunCancelFlagValue) -> Result<StageRunScopeValue, StoreBackendFailureError> { todo!() }
    fn read_workspace_publication_scope(&self) -> Result<Option<StageRunScopeValue>, StoreBackendFailureError> { todo!() }
}
