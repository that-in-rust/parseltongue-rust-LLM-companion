#![allow(clippy::unwrap_used)]

use super::*;

/// REQ-RUST-024.9
#[test]
fn test_legacy_constructor_ignores_limit() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.db"),
        directory.path(),
        1_288_490_188,
    )
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
        directory.path().join("duckdb-analysis-spill-files")
    );
    assert!(directory.path().join("analysis.db").is_file());
    let memory: String = connection
        .query_row("SELECT current_setting('memory_limit')", [], |row| {
            row.get(0)
        })
        .unwrap();
    let defaults = duckdb::Connection::open_in_memory().unwrap();
    let expected: String = defaults
        .query_row("SELECT current_setting('memory_limit')", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(memory, expected);
    assert!(connection.execute_batch("INSTALL httpfs").is_err());
    assert!(connection.execute_batch("LOAD httpfs").is_err());
    assert!(
        connection
            .execute_batch("SET enable_external_access=true")
            .is_err()
    );
}

/// REQ-RUST-024.9
#[test]
fn test_constructor_rejects_invalid_inputs() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("analysis.db");
    for memory in [0, 2_147_483_648] {
        assert!(
            DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
                &database,
                directory.path(),
                memory
            )
            .is_ok()
        );
    }
    assert!(
        DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
            &database,
            &directory.path().join("missing"),
            1_288_490_188
        )
        .is_err()
    );
}
