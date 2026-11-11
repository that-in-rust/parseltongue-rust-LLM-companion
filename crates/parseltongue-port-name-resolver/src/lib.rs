//! Pure crate-local name resolution; parsing and expansion are explicit ports.

use parseltongue_core_row_types::{
    NameExpansionRoundRows, NameMacroExpansionQuery, NameResolutionInputRows,
    NameResolutionOutputRows, NameResolutionStageError, RunCancelFlagValue,
};

#[path = "crate-resolution-round-engine.rs"]
mod crate_resolution_round_engine;
#[path = "namespace-binding-normalization-rules.rs"]
mod namespace_binding_normalization_rules;
#[path = "scope-path-resolution-rules.rs"]
mod scope_path_resolution_rules;

pub const FIXED_POINT_LIMIT: usize = 8192;

pub trait ItemMacroExpansionPort: Send + Sync {
    fn expand_resolved_macro_items(
        &self,
        expansion_round_query_rows: &NameMacroExpansionQuery<'_>,
    ) -> Result<NameExpansionRoundRows, NameResolutionStageError>;
}

pub fn resolve_crate_name_bindings(
    name_resolution_input_rows: &NameResolutionInputRows,
    item_macro_expansion_port: &dyn ItemMacroExpansionPort,
    stage_cancel_flag_value: &RunCancelFlagValue,
) -> Result<NameResolutionOutputRows, NameResolutionStageError> {
    crate_resolution_round_engine::run_crate_resolution_rounds(
        name_resolution_input_rows,
        item_macro_expansion_port,
        stage_cancel_flag_value,
    )
}

#[cfg(test)]
#[path = "row-resolution-contract-tests.rs"]
mod row_resolution_contract_tests;
