//! M0 database/runtime foundation; stage schemas and persistence arrive at M1.

use parseltongue_core_row_types::StoreBackendFailureError;

pub struct TursoWorkspaceStoreAdapter {
    store_database_handle_value: turso::Database,
    store_runtime_handle_value: tokio::runtime::Runtime,
}

impl TursoWorkspaceStoreAdapter {
    /// Owns a synchronous L3 runtime; construct, call and drop outside async executors.
    pub fn open_turso_workspace_store(
        workspace_store_file_path: &str,
    ) -> Result<Self, StoreBackendFailureError> {
        if workspace_store_file_path.is_empty() || workspace_store_file_path.contains('\0') {
            return Err(build_store_backend_error(
                "workspace store path must be nonempty without NUL",
            ));
        }
        check_sync_runtime_context()?;
        let store_runtime_handle_value = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(build_store_backend_error)?;
        let store_database_handle_value = store_runtime_handle_value
            .block_on(async {
                let store_database_handle_value =
                    turso::Builder::new_local(workspace_store_file_path)
                        .build()
                        .await?;
                store_database_handle_value.connect()?;
                Ok::<_, turso::Error>(store_database_handle_value)
            })
            .map_err(build_store_backend_error)?;
        Ok(Self {
            store_runtime_handle_value,
            store_database_handle_value,
        })
    }

    pub fn check_turso_concurrent_api(&self) -> Result<(), StoreBackendFailureError> {
        check_sync_runtime_context()?;
        self.store_runtime_handle_value
            .block_on(async {
                let mut store_database_connection_handle =
                    self.store_database_handle_value.connect()?;
                store_database_connection_handle
                    .execute("PRAGMA journal_mode='mvcc'", ())
                    .await?;
                store_database_connection_handle
                    .execute("PRAGMA busy_timeout=10", ())
                    .await?;
                let _ = store_database_connection_handle.prepare("SELECT 1").await?;
                let mut store_database_query_rows = store_database_connection_handle
                    .query("SELECT 1", ())
                    .await?;
                let _ = store_database_query_rows.next().await?;
                store_database_connection_handle
                    .transaction_with_behavior(turso::transaction::TransactionBehavior::Concurrent)
                    .await?
                    .rollback()
                    .await?;
                Ok::<_, turso::Error>(())
            })
            .map_err(build_store_backend_error)
    }
}

fn build_store_backend_error(
    store_backend_error_cause: impl std::fmt::Display,
) -> StoreBackendFailureError {
    StoreBackendFailureError {
        error_context_detail_text: format!("Turso store: {store_backend_error_cause}"),
    }
}

fn check_sync_runtime_context() -> Result<(), StoreBackendFailureError> {
    if tokio::runtime::Handle::try_current().is_ok() {
        Err(build_store_backend_error(
            "synchronous store must be called outside an async runtime",
        ))
    } else {
        Ok(())
    }
}
