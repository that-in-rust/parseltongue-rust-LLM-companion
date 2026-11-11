//! Synchronous stage seams; concrete storage belongs to L3.

use parseltongue_core_row_types::{
    RunGenerationDigestValue, SemanticOccurrenceKeyValue, StoreBackendFailureError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageUnitRowRecord {
    pub run_generation_digest_value: RunGenerationDigestValue,
    pub span_semantic_occurrence_key: SemanticOccurrenceKeyValue,
    pub file_source_content_bytes: Vec<u8>,
}

pub trait StageRowWriteSink {
    fn write_stage_unit_rows(
        &mut self,
        stage_unit_output_rows: &[StageUnitRowRecord],
    ) -> Result<(), StoreBackendFailureError>;
}

pub trait SpanRowReadStore {
    fn read_span_by_digest(
        &self,
        run_generation_digest_value: RunGenerationDigestValue,
        span_semantic_occurrence_key: &SemanticOccurrenceKeyValue,
    ) -> Result<Option<StageUnitRowRecord>, StoreBackendFailureError>;
}
