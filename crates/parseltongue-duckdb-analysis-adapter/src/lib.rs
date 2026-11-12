#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

//! Bounded, extension-free analysis of frozen Turso generations.

use parseltongue_core_row_types::{
    NameResolutionInputRows, NameResolutionOutputRows, NameResolutionStageError,
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};
use parseltongue_table_store_traits::{
    AnalysisTableStorePort, ItemMacroExpansionPort, StageTableStorePort,
};
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

pub const FIXED_POINT_LIMIT: usize = 8192;

pub fn resolve_crate_name_bindings(
    _input: &NameResolutionInputRows,
    _macros: &dyn ItemMacroExpansionPort,
    _cancel: &RunCancelFlagValue,
) -> Result<NameResolutionOutputRows, NameResolutionStageError> {
    todo!("DuckDB non-monotone SQL resolution")
}

pub fn resolve_owned_crate_bindings(
    input: NameResolutionInputRows,
    macros: &dyn ItemMacroExpansionPort,
    cancel: &RunCancelFlagValue,
) -> Result<NameResolutionOutputRows, NameResolutionStageError> {
    resolve_crate_name_bindings(&input, macros, cancel)
}

#[cfg(test)]
#[path = "row-resolution-contract-tests.rs"]
mod row_resolution_contract_tests;
