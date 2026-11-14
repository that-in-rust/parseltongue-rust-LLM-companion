use crate::HldPublicationPortError;
use parseltongue_core_row_types::{
    HldMaterializationInputRecord, RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};

#[derive(Debug, thiserror::Error)]
pub enum HldGraphInputStoreError {
    #[error("HLD source scope is not a frozen snapshot")]
    UnfrozenSourceSnapshot,
    #[error("HLD source and destination base scopes are incompatible")]
    IncompatibleSourceScope,
    #[error("HLD input generation does not match its source scope")]
    GenerationMismatch,
    #[error("HLD input attempt does not match its destination base scope")]
    AttemptMismatch,
    #[error("HLD typed coverage does not match frozen workspace configuration")]
    CoverageScopeMismatch,
    #[error("typed unit provenance does not match frozen root namespace mappings")]
    UnitProvenanceMismatch,
    #[error("mapped raw crate unit has no frozen root namespace scope: {raw_unit_key}")]
    MissingRootUnitScope { raw_unit_key: String },
    #[error("frozen workspace coverage configuration is malformed")]
    InvalidFrozenWorkspaceConfig,
    #[error("frozen HLD source snapshot validation failed")]
    SourceSnapshotValidation(#[from] HldPublicationPortError),
    #[error("required graph input stage marker is not visible: {stage_name}")]
    MissingStageMarker { stage_name: String },
    #[error("graph input stage changed after its frozen source snapshot: {stage_name}")]
    StaleStageMarker { stage_name: String },
    #[error("persisted graph input row is malformed: {table}/{record_key}: {reason}")]
    MalformedPersistedRow {
        table: String,
        record_key: String,
        reason: String,
    },
    #[error("persisted graph input key is duplicated: {table}/{record_key}")]
    DuplicatePersistedKey { table: String, record_key: String },
    #[error("typed HLD graph input violates core invariants")]
    InvalidTypedInput(#[from] parseltongue_core_row_types::HldMaterializationValidationError),
    #[error("HLD graph input store backend failed")]
    Backend(#[from] StoreBackendFailureError),
    #[error("HLD graph input operation was cancelled")]
    Cancelled,
}

pub trait HldGraphInputStorePort: Send + Sync {
    fn write_hld_graph_input(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        destination_base_scope: &StageRunScopeValue,
        input_record: &HldMaterializationInputRecord,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, HldGraphInputStoreError>;

    fn assemble_hld_graph_input(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<HldMaterializationInputRecord, HldGraphInputStoreError>;
}

/// Persist a bounded full-map checkpoint without publishing workspace serving output.
/// Completeness is retained independently of source coverage and known visibility.
pub trait FullGraphInputStorePort: Send + Sync {
    fn write_full_graph_input(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        destination_base_scope: &StageRunScopeValue,
        input_record: &HldMaterializationInputRecord,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, HldGraphInputStoreError>;

    fn assemble_full_graph_input(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<HldMaterializationInputRecord, HldGraphInputStoreError>;
}
