use std::path::Path;
use parseltongue_core_row_types::{RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StoreBackendFailureError};
use parseltongue_table_store_traits::StageTableStorePort;

pub trait WorkspaceCommandRunPort {
    fn run_workspace_command_output(&self, workspace_root_directory_path: &Path, workspace_command_argument_values: &[&str])
        -> Result<Vec<u8>, StoreBackendFailureError>;
}

pub struct OfflineWorkspaceCommandRunner;

impl WorkspaceCommandRunPort for OfflineWorkspaceCommandRunner {
    fn run_workspace_command_output(&self, _workspace_root_directory_path: &Path, _workspace_command_argument_values: &[&str])
        -> Result<Vec<u8>, StoreBackendFailureError> { todo!() }
}

pub fn load_workspace_crate_graph(
    _workspace_root_directory_path: &Path,
    _workspace_command_runner_port: &dyn WorkspaceCommandRunPort,
) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> { todo!() }

pub fn ingest_workspace_source_files(
    _stage_run_scope_value: &StageRunScopeValue,
    _stage_table_store_port: &dyn StageTableStorePort,
    _stage_cancel_flag_value: &RunCancelFlagValue,
) -> Result<usize, StoreBackendFailureError> { todo!() }
