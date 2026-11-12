use super::{
    crate_syntax_namespace_projection::CrateSyntaxProjectionState,
    name_resolution_record_codec::build_name_projection_error,
};
use parseltongue_core_row_types::*;
use parseltongue_port_name_resolver::ItemMacroExpansionPort;
use std::sync::Mutex;

pub(super) struct PersistedItemMacroPort<'a> {
    projection_source_state_lock: Mutex<CrateSyntaxProjectionState<'a>>,
}

impl<'a> PersistedItemMacroPort<'a> {
    pub(super) fn build_persisted_macro_port(projection: CrateSyntaxProjectionState<'a>) -> Self {
        Self {
            projection_source_state_lock: Mutex::new(projection),
        }
    }
    pub(super) fn finish_persisted_macro_port(
        self,
    ) -> Result<CrateSyntaxProjectionState<'a>, StoreBackendFailureError> {
        self.projection_source_state_lock
            .into_inner()
            .map_err(build_name_projection_error)
    }
}

impl ItemMacroExpansionPort for PersistedItemMacroPort<'_> {
    fn expand_resolved_macro_items(
        &self,
        _query: &NameMacroExpansionQuery<'_>,
    ) -> Result<NameExpansionRoundRows, NameResolutionStageError> {
        todo!()
    }
}
