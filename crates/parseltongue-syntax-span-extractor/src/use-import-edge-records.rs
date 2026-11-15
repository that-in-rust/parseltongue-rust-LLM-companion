use crate::impl_header_source_records::ImplHeaderSourceError;
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;

#[cfg(test)]
#[path = "use-import-edge-tests.rs"]
mod use_import_edge_tests;

#[derive(Debug, thiserror::Error)]
pub enum UseImportEdgeError {
    #[error("use import edge resolution cancelled")]
    Cancelled,
    #[error("use import edge store: {0}")]
    Store(#[from] StoreBackendFailureError),
    #[error("use import edge identity: {0}")]
    Identity(#[from] ImplHeaderSourceError),
    #[error("use import edge evidence field is invalid: {field}")]
    InvalidEvidence { field: &'static str },
    #[error("use import edge input scope does not match its graph")]
    InvalidScope,
}

/// The `imports` rows of one graph, and how many import sites gave no row:
/// a glob, a site in macro-expanded text, a site inside no `use` node.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UseImportEdgeSet {
    pub import_edge_input_records: Vec<HldEdgeInputRecord>,
    pub imports_glob_skipped: usize,
    pub imports_in_expanded_items: usize,
    pub imports_without_source: usize,
}

/// One `imports` edge per path of an original `use` item, from the `use`
/// node to what the path names. It joins the import requests the syntax stage
/// stored with the outcomes name resolution stored and never looks a name up
/// itself. The caller supplies the frozen source scope of `input`.
pub fn resolve_use_import_edges(
    _store: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    input: &HldMaterializationInputRecord,
    cancel: &RunCancelFlagValue,
) -> Result<UseImportEdgeSet, UseImportEdgeError> {
    if cancel.check_stage_work_cancelled() {
        return Err(UseImportEdgeError::Cancelled);
    }
    if source_scope.run_generation_digest_value
        != blake3::Hash::from(input.run_generation_digest_value.0)
            .to_hex()
            .as_str()
        || input.source_coverage_kind != HldSourceCoverageKind::FullSemantic
    {
        return Err(UseImportEdgeError::InvalidScope);
    }
    // STUB (part 4d step 3): no row is produced yet.
    Ok(UseImportEdgeSet::default())
}
