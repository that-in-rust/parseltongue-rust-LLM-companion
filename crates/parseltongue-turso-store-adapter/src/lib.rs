//! M0 database/runtime foundation; stage schemas and persistence arrive at M1.

use parseltongue_core_row_types::StoreBackendFailureError;

pub struct TursoWorkspaceStoreAdapter {
    store_runtime_handle_value: tokio::runtime::Runtime,
    store_database_handle_value: turso::Database,
}

impl TursoWorkspaceStoreAdapter {
    pub fn open_turso_workspace_store(
        _workspace_store_file_path: &str,
    ) -> Result<Self, StoreBackendFailureError> {
        todo!()
    }

    pub fn check_turso_concurrent_api(&self) -> Result<(), StoreBackendFailureError> {
        let _ = (&self.store_runtime_handle_value, &self.store_database_handle_value);
        todo!()
    }
}
