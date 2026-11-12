use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::ResolvedMacroLookupPort;
use std::sync::Mutex;

pub struct FrozenNamespaceMacroLookup<'a> {
    store: &'a DuckdbAnalysisStoreAdapter,
    scope: Mutex<StageRunScopeValue>,
}

impl DuckdbAnalysisStoreAdapter {
    pub fn frozen_namespace_macro_lookup(
        &self,
        scope: StageRunScopeValue,
    ) -> FrozenNamespaceMacroLookup<'_> {
        FrozenNamespaceMacroLookup {
            store: self,
            scope: Mutex::new(scope),
        }
    }
}

impl FrozenNamespaceMacroLookup<'_> {
    pub fn replace_frozen_lookup_scope(
        &self,
        scope: StageRunScopeValue,
    ) -> Result<(), StoreBackendFailureError> {
        *self.scope.lock().map_err(build_analysis_error)? = scope;
        Ok(())
    }
}

fn failure(message: impl std::fmt::Display) -> NameResolutionStageError {
    NameResolutionStageError::Input(StageInputMissingError {
        error_context_detail_text: message.to_string(),
    })
}

impl ResolvedMacroLookupPort for FrozenNamespaceMacroLookup<'_> {
    fn read_macro_invocation_binding_record(
        &self,
        _: &str,
    ) -> Result<Option<StageTableRowRecord>, NameResolutionStageError> {
        let _scope = self.scope.lock().map_err(failure)?;
        let _store = self.store;
        Err(failure("frozen macro candidate lookup not implemented"))
    }
    fn read_resolved_builtin_macro(
        &self,
        _: &NameResolutionScopeKey,
        _: &[NamePathSegmentKind],
    ) -> Result<Option<String>, NameResolutionStageError> {
        Err(failure("frozen nested builtin lookup not implemented"))
    }
    fn read_macro_target_metadata(
        &self,
        _: &NameSymbolTargetKey,
    ) -> Result<Option<StageTableRowRecord>, NameResolutionStageError> {
        Err(failure("frozen macro metadata lookup not implemented"))
    }
}
