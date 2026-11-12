use crate::DuckdbAnalysisStoreAdapter;
use parseltongue_core_row_types::StoreBackendFailureError;
use std::path::Path;

impl DuckdbAnalysisStoreAdapter {
    pub fn open_measured_analysis_store(
        database: &Path,
        run_directory: &Path,
    ) -> Result<Self, StoreBackendFailureError> {
        if !run_directory.is_dir() {
            return Err(crate::build_analysis_error(
                "analysis run directory must exist",
            ));
        }
        let spill = run_directory
            .canonicalize()
            .map_err(crate::build_analysis_error)?
            .join("duckdb-analysis-spill-files");
        std::fs::create_dir_all(&spill).map_err(crate::build_analysis_error)?;
        let spill = spill
            .to_str()
            .ok_or_else(|| crate::build_analysis_error("analysis spill path is not UTF-8"))?;
        let config = duckdb::Config::default()
            .with("temp_directory", spill)
            .and_then(|config| config.enable_external_access(false))
            .and_then(|config| config.with("autoinstall_known_extensions", "false"))
            .and_then(|config| config.with("autoload_known_extensions", "false"))
            .and_then(|config| config.with("allow_community_extensions", "false"))
            .and_then(|config| config.with("threads", "2"))
            .and_then(|config| config.with("preserve_insertion_order", "false"))
            .map_err(crate::build_analysis_error)?;
        let connection = duckdb::Connection::open_with_flags(database, config)
            .map_err(crate::build_analysis_error)?;
        Ok(Self {
            _connection: std::sync::Mutex::new(connection),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// REQ-RUST-024.9
    #[test]
    fn test_namespace_constructor_uses_default() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("measured.duckdb");
        let store =
            DuckdbAnalysisStoreAdapter::open_measured_analysis_store(&database, directory.path())
                .unwrap();
        let connection = store._connection.lock().unwrap();
        for setting in [
            "enable_external_access",
            "autoinstall_known_extensions",
            "autoload_known_extensions",
            "allow_community_extensions",
        ] {
            let value: bool = connection
                .query_row(&format!("SELECT current_setting('{setting}')"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert!(!value, "{setting}");
        }
        let spill: String = connection
            .query_row("SELECT current_setting('temp_directory')", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(
            Path::new(&spill),
            directory
                .path()
                .canonicalize()
                .unwrap()
                .join("duckdb-analysis-spill-files")
        );
        let actual: String = connection
            .query_row("SELECT current_setting('memory_limit')", [], |row| {
                row.get(0)
            })
            .unwrap();
        drop(connection);
        let defaults = duckdb::Connection::open_in_memory().unwrap();
        let expected: String = defaults
            .query_row("SELECT current_setting('memory_limit')", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(actual, expected);
        assert!(database.is_file());
        let report = store.read_namespace_memory_usage().unwrap();
        assert_eq!(report["settings"]["memory_limit"], actual);
    }

    /// REQ-RUST-024.9
    #[test]
    fn test_namespace_constructor_rejects_missing() {
        let directory = tempfile::tempdir().unwrap();
        let missing = directory.path().join("missing");
        assert!(
            DuckdbAnalysisStoreAdapter::open_measured_analysis_store(
                &directory.path().join("missing-run.duckdb"),
                &missing,
            )
            .is_err()
        );
    }
}
