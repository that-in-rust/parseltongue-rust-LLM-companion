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
        database: &Path,
        run_directory: &Path,
        memory_limit_bytes: usize,
    ) -> Result<Self, StoreBackendFailureError> {
        if memory_limit_bytes == 0 || memory_limit_bytes > 1_288_490_188 {
            return Err(build_analysis_error(
                "analysis memory limit must be within 1.2 GiB",
            ));
        }
        if !run_directory.is_dir() {
            return Err(build_analysis_error("analysis run directory must exist"));
        }
        let spill = run_directory
            .canonicalize()
            .map_err(build_analysis_error)?
            .join("duckdb-analysis-spill-files");
        std::fs::create_dir_all(&spill).map_err(build_analysis_error)?;
        let spill = spill
            .to_str()
            .ok_or_else(|| build_analysis_error("analysis spill path is not UTF-8"))?;
        let config = duckdb::Config::default()
            .enable_external_access(true)
            .and_then(|config| config.with("autoinstall_known_extensions", "false"))
            .and_then(|config| config.with("autoload_known_extensions", "false"))
            .and_then(|config| config.with("allow_community_extensions", "false"))
            .and_then(|config| config.max_memory(&format!("{memory_limit_bytes}B")))
            .map_err(build_analysis_error)?;
        let connection =
            duckdb::Connection::open_with_flags(database, config).map_err(build_analysis_error)?;
        connection
            .execute("SET temp_directory = ?", [spill])
            .map_err(build_analysis_error)?;
        connection
            .execute_batch("SET enable_external_access = false")
            .map_err(build_analysis_error)?;
        Ok(Self {
            _connection: Mutex::new(connection),
        })
    }
}

fn build_analysis_error(message: impl std::fmt::Display) -> StoreBackendFailureError {
    StoreBackendFailureError {
        error_context_detail_text: message.to_string(),
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

#[cfg(test)]
#[path = "bounded-constructor-policy-tests.rs"]
mod bounded_constructor_policy_tests;
