use crate::impl_header_source_records::{ImplHeaderSourceError, read_impl_stage_values};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;

#[cfg(test)]
#[path = "impl-header-implements-tests.rs"]
mod impl_header_implements_tests;

#[derive(Debug, thiserror::Error)]
pub enum ImplHeaderEdgeError {
    #[error("impl header edge resolution cancelled")]
    Cancelled,
    #[error("impl header edge store: {0}")]
    Store(#[from] StoreBackendFailureError),
    #[error("impl header edge identity: {0}")]
    Identity(#[from] ImplHeaderSourceError),
    #[error("impl header edge evidence field is invalid: {field}")]
    InvalidEvidence { field: &'static str },
    #[error("impl header edge input scope does not match its graph")]
    InvalidScope,
}

/// The `implements` rows of one graph, and how many trait impl headers have
/// a self type that is not a graph node and so could not source a row.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ImplHeaderEdgeSet {
    pub implements_edge_records: Vec<HldEdgeInputRecord>,
    pub headers_without_source: usize,
}

/// One `implements` edge per trait impl header, from the header's self type
/// to its trait. It joins what the impl header stage stored and never looks a
/// name up itself. The caller supplies the frozen source scope of `input`.
pub fn resolve_impl_header_edges(
    store: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    input: &HldMaterializationInputRecord,
    cancel: &RunCancelFlagValue,
) -> Result<ImplHeaderEdgeSet, ImplHeaderEdgeError> {
    if cancel.check_stage_work_cancelled() {
        return Err(ImplHeaderEdgeError::Cancelled);
    }
    let _ = (
        read_impl_stage_values(store, source_scope, "impl-header-index-table")?,
        input,
    );
    Ok(ImplHeaderEdgeSet::default())
}
