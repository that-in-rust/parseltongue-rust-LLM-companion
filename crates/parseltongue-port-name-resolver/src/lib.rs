//! Pure crate-local name resolution; parsing and expansion are explicit ports.

use parseltongue_core_row_types::{
    NameExpansionRoundRows, NameMacroExpansionQuery, NameResolutionInputRows,
    NameResolutionOutputRows, NameResolutionStageError, RunCancelFlagValue,
};

pub const FIXED_POINT_LIMIT: usize = 8192;

pub trait ItemMacroExpansionPort: Send + Sync {
    fn expand_resolved_macro_items(
        &self,
        expansion_round_query_rows: &NameMacroExpansionQuery<'_>,
    ) -> Result<NameExpansionRoundRows, NameResolutionStageError>;
}

pub fn resolve_crate_name_bindings(
    _name_resolution_input_rows: &NameResolutionInputRows,
    _item_macro_expansion_port: &dyn ItemMacroExpansionPort,
    _stage_cancel_flag_value: &RunCancelFlagValue,
) -> Result<NameResolutionOutputRows, NameResolutionStageError> {
    todo!()
}

#[cfg(test)]
#[path = "row-resolution-contract-tests.rs"]
mod row_resolution_contract_tests;
