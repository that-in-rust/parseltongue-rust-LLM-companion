use parseltongue_core_row_types::{
    HldMaterializationOutputRecord, RunCancelFlagValue, StageRunScopeValue,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HldPublicationOperationKind {
    ValidateInputSnapshot,
    WriteOutputRows,
    SealOutputSnapshot,
    SwitchPublicationManifest,
}

#[derive(Debug, thiserror::Error)]
pub enum HldPublicationPortError {
    #[error("HLD source scope is not a frozen snapshot")]
    UnfrozenSourceSnapshot,
    #[error("HLD source and destination scopes are incompatible")]
    IncompatibleDestinationScope,
    #[error("HLD output generation does not match publication scope")]
    GenerationMismatch,
    #[error("HLD typed coverage does not match the frozen workspace configuration")]
    CoverageScopeMismatch,
    #[error("frozen workspace coverage configuration is malformed or unsupported")]
    InvalidFrozenWorkspaceConfig,
    #[error("graph source snapshot provenance is missing or inconsistent")]
    InvalidGraphSourceProvenance,
    #[error("HLD output must be a CompleteGraph with both roll-ups present")]
    IncompleteOutput,
    #[error("HLD publication requires complete typed graph input")]
    IncompleteGraphInput,
    #[error("required input stage marker is not visible: {stage_name}")]
    MissingInputStageMarker { stage_name: String },
    #[error("input stage changed after the source snapshot: {stage_name}")]
    StaleInputSnapshot { stage_name: String },
    #[error("HLD output unit is already sealed")]
    SealedOutputUnit,
    #[error("HLD publication was cancelled")]
    Cancelled,
    #[error("HLD publication {operation:?} failed")]
    Backend {
        operation: HldPublicationOperationKind,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

pub trait HldGraphPublicationPort: Send + Sync {
    fn publish_hld_graph_materialization(
        &self,
        source_snapshot_scope: &StageRunScopeValue,
        destination_scope: &StageRunScopeValue,
        output_record: &HldMaterializationOutputRecord,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, HldPublicationPortError>;
}
