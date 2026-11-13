use parseltongue_core_row_types::{
    HldMaterializationInputRecord, HldMaterializationOutputRecord,
    HldMaterializationValidationError, RunCancelFlagValue,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HldAnalysisOperationKind {
    PrepareTransaction,
    CreateWorkingRelations,
    NormalizeNodeInputs,
    NormalizeEdgeInputs,
    ProjectRecursivePaths,
    ReadAnalysisOutputs,
    CommitTransaction,
}

#[derive(Debug, thiserror::Error)]
pub enum HldAnalysisPortError {
    #[error("HLD semantic input is invalid: {0}")]
    InvalidInput(#[from] HldMaterializationValidationError),
    #[error("HLD analysis was cancelled")]
    Cancelled,
    #[error("HLD analysis connection lock is poisoned")]
    LockPoisoned,
    #[error("HLD analysis operation {operation:?} failed")]
    Backend {
        operation: HldAnalysisOperationKind,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

pub trait HldGraphAnalysisPort: Send + Sync {
    fn materialize_hld_graph_output(
        &self,
        input_record: &HldMaterializationInputRecord,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<HldMaterializationOutputRecord, HldAnalysisPortError>;
}
