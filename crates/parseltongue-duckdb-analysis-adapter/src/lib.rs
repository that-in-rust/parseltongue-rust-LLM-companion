#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

//! Bounded, extension-free analysis of frozen Turso generations.

use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};
use parseltongue_table_store_traits::{AnalysisTableStorePort, StageTableStorePort};
use std::{path::Path, sync::Mutex};

pub struct DuckdbAnalysisStoreAdapter {
    _connection: Mutex<duckdb::Connection>,
}

impl DuckdbAnalysisStoreAdapter {
    pub fn open_bounded_analysis_store(
        _database: &Path,
        _run_directory: &Path,
        _memory_limit_bytes: usize,
    ) -> Result<Self, StoreBackendFailureError> {
        todo!("bounded DuckDB analysis store")
    }
}

impl AnalysisTableStorePort for DuckdbAnalysisStoreAdapter {
    fn copy_published_stage_rows(
        &self,
        _source: &dyn StageTableStorePort,
        _scope: &StageRunScopeValue,
        _tables: &[&str],
        _cancel: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        todo!("frozen generation Appender copy")
    }

    fn count_analysis_stage_rows(
        &self,
        _scope: &StageRunScopeValue,
        _table: &str,
    ) -> Result<usize, StoreBackendFailureError> {
        todo!("frozen generation analysis count")
    }
}
