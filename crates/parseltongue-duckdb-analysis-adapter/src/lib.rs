#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

//! Measured-only, extension-free analysis of frozen Turso generations.

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

#[path = "namespace-analysis-phase-diagnostics.rs"]
mod namespace_analysis_phase_diagnostics;
#[path = "namespace-memory-limit-contract.rs"]
mod namespace_memory_limit_contract;
#[path = "namespace-storage-size-diagnostics.rs"]
mod namespace_storage_size_diagnostics;
pub use namespace_analysis_phase_diagnostics::{
    NamespaceAnalysisPhaseError, NamespaceAnalysisPhaseKind,
};

impl DuckdbAnalysisStoreAdapter {
    pub fn open_bounded_analysis_store(
        database: &Path,
        run_directory: &Path,
        _memory_limit_bytes: usize,
    ) -> Result<Self, StoreBackendFailureError> {
        Self::open_measured_analysis_store(database, run_directory)
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
        source: &dyn StageTableStorePort,
        scope: &StageRunScopeValue,
        tables: &[&str],
        cancel: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        frozen_generation_appender_copy::copy_generation(self, source, scope, tables, cancel)
    }

    fn count_analysis_stage_rows(
        &self,
        scope: &StageRunScopeValue,
        table: &str,
    ) -> Result<usize, StoreBackendFailureError> {
        let connection = self._connection.lock().map_err(build_analysis_error)?;
        connection.query_row(
            "SELECT row_count FROM \"frozen-generation-table-manifests\" WHERE generation_key=? AND attempt_key=? AND table_name=? ORDER BY generation_key,attempt_key,table_name",
            duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value,table],
            |row| row.get::<_, usize>(0),
        ).map_err(build_analysis_error)
    }
}

#[path = "frozen-generation-appender-copy.rs"]
mod frozen_generation_appender_copy;
#[path = "namespace-appender-batch-budget.rs"]
mod namespace_appender_batch_budget;
#[path = "namespace-source-column-projection.rs"]
mod namespace_source_column_projection;

#[path = "frozen-namespace-source-codec.rs"]
mod frozen_namespace_source_codec;
pub use frozen_namespace_source_codec::NAMESPACE_SOURCE_TABLE_NAMES;
#[path = "namespace-frozen-resolution-rounds.rs"]
mod namespace_frozen_resolution_rounds;
pub use namespace_frozen_resolution_rounds::NamespaceResolutionRoundReport;
#[path = "namespace-default-prelude-inputs.rs"]
mod namespace_default_prelude_inputs;
#[path = "namespace-frozen-macro-lookups.rs"]
mod namespace_frozen_macro_lookups;
pub use namespace_frozen_macro_lookups::FrozenNamespaceMacroLookup;
#[path = "namespace-final-output-records.rs"]
mod namespace_final_output_records;
#[path = "namespace-frozen-identity-codec.rs"]
mod namespace_frozen_identity_codec;
#[path = "namespace-unit-link-inputs.rs"]
mod namespace_unit_link_inputs;
#[path = "namespace-working-cache-reset.rs"]
mod namespace_working_cache_reset;

#[path = "namespace-binding-round-strata.rs"]
mod namespace_binding_round_strata;
pub use namespace_binding_round_strata::recompute_binding_round;

#[path = "namespace-import-endpoint-rounds.rs"]
mod namespace_import_endpoint_rounds;
pub use namespace_import_endpoint_rounds::resolve_import_endpoint_rounds;

#[path = "namespace-scope-relation-records.rs"]
mod namespace_scope_relation_records;
pub use namespace_scope_relation_records::materialize_scope_relations;

#[path = "namespace-path-candidate-records.rs"]
mod namespace_path_candidate_records;
pub use namespace_path_candidate_records::recompute_path_candidates;

#[path = "namespace-contract-input-codec.rs"]
mod namespace_contract_input_codec;
#[path = "namespace-contract-resolution-rounds.rs"]
mod namespace_contract_resolution_rounds;
#[path = "namespace-macro-alternation-rounds.rs"]
mod namespace_macro_alternation_rounds;

pub const FIXED_POINT_LIMIT: usize = 8192;

pub fn resolve_crate_name_bindings(
    input: &NameResolutionInputRows,
    macros: &dyn ItemMacroExpansionPort,
    cancel: &RunCancelFlagValue,
) -> Result<NameResolutionOutputRows, NameResolutionStageError> {
    namespace_macro_alternation_rounds::resolve_with_macro_rounds(input.clone(), macros, cancel)
}

pub fn resolve_owned_crate_bindings(
    input: NameResolutionInputRows,
    macros: &dyn ItemMacroExpansionPort,
    cancel: &RunCancelFlagValue,
) -> Result<NameResolutionOutputRows, NameResolutionStageError> {
    namespace_macro_alternation_rounds::resolve_with_macro_rounds(input, macros, cancel)
}

#[cfg(test)]
#[path = "row-resolution-contract-tests.rs"]
mod row_resolution_contract_tests;

#[cfg(test)]
#[path = "bounded-constructor-policy-tests.rs"]
mod bounded_constructor_policy_tests;

#[cfg(test)]
#[path = "published-generation-copy-tests.rs"]
mod published_generation_copy_tests;
