//! Synchronous stage seams; concrete storage belongs to L3.

use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StageUnitWriteRequest,
};

pub trait StageTableStorePort: Send + Sync {
    fn initialize_workspace_stage_schema(&self) -> Result<(), StoreBackendFailureError>;
    fn write_stage_table_unit(
        &self,
        stage_unit_write_request: &StageUnitWriteRequest<'_>,
        stage_cancel_flag_value: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError>;
    fn read_stage_table_rows(
        &self,
        stage_run_scope_value: &StageRunScopeValue,
        stage_owned_table_name: &str,
    ) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError>;
    fn count_stage_table_rows(
        &self,
        stage_run_scope_value: &StageRunScopeValue,
        stage_owned_table_name: &str,
    ) -> Result<usize, StoreBackendFailureError> {
        self.read_stage_table_rows(stage_run_scope_value, stage_owned_table_name)
            .map(|rows| rows.len())
    }
    fn visit_stage_table_rows(
        &self,
        stage_run_scope_value: &StageRunScopeValue,
        stage_owned_table_name: &str,
        stage_row_visit_function: &mut dyn FnMut(
            StageTableRowRecord,
        ) -> Result<(), StoreBackendFailureError>,
    ) -> Result<usize, StoreBackendFailureError> {
        self.read_stage_table_rows(stage_run_scope_value, stage_owned_table_name)?
            .into_iter()
            .try_fold(0, |count, row| {
                stage_row_visit_function(row)?;
                Ok(count + 1)
            })
    }
    fn prepare_workspace_generation_scope(
        &self,
        _stage_run_scope_value: &StageRunScopeValue,
        _run_generation_digest_value: &str,
        _stage_cancel_flag_value: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        Err(StoreBackendFailureError {
            error_context_detail_text:
                "StageScopePreparationError: adapter does not support scope preparation".into(),
        })
    }
    fn write_file_content_bytes(
        &self,
        file_content_digest_full: &str,
        file_source_content_bytes: &[u8],
    ) -> Result<bool, StoreBackendFailureError>;
    fn read_file_content_bytes(
        &self,
        file_content_digest_full: &str,
    ) -> Result<Vec<u8>, StoreBackendFailureError>;
    fn publish_workspace_generation_manifest(
        &self,
        stage_run_scope_value: &StageRunScopeValue,
        run_generation_digest_value: &str,
        stage_cancel_flag_value: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError>;
    fn read_workspace_publication_scope(
        &self,
    ) -> Result<Option<StageRunScopeValue>, StoreBackendFailureError>;
}

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

#[cfg(test)]
mod stage_store_fixture_tests {
    use super::*;
    use parseltongue_core_row_types::{
        CompilationUnitRoleKind, CrateCompilationUnitKey, SpanByteRangeValue, SpanDigestIdValue,
    };
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct FakeStageRowStore {
        stage_unit_stored_rows:
            BTreeMap<(RunGenerationDigestValue, SemanticOccurrenceKeyValue), StageUnitRowRecord>,
    }

    impl StageRowWriteSink for FakeStageRowStore {
        fn write_stage_unit_rows(
            &mut self,
            stage_unit_output_rows: &[StageUnitRowRecord],
        ) -> Result<(), StoreBackendFailureError> {
            self.stage_unit_stored_rows
                .extend(stage_unit_output_rows.iter().map(|stage_unit_output_row| {
                    (
                        (
                            stage_unit_output_row.run_generation_digest_value,
                            stage_unit_output_row.span_semantic_occurrence_key.clone(),
                        ),
                        stage_unit_output_row.clone(),
                    )
                }));
            Ok(())
        }
    }

    impl SpanRowReadStore for FakeStageRowStore {
        fn read_span_by_digest(
            &self,
            run_generation_digest_value: RunGenerationDigestValue,
            span_semantic_occurrence_key: &SemanticOccurrenceKeyValue,
        ) -> Result<Option<StageUnitRowRecord>, StoreBackendFailureError> {
            Ok(self
                .stage_unit_stored_rows
                .get(&(
                    run_generation_digest_value,
                    span_semantic_occurrence_key.clone(),
                ))
                .cloned())
        }
    }

    #[test]
    fn test_fake_store_preserves_generations() -> Result<(), Box<dyn std::error::Error>> {
        let unit_compilation_identity_key = CrateCompilationUnitKey {
            unit_cargo_package_identifier: "sample@1".into(),
            unit_cargo_target_name: "sample".into(),
            unit_cargo_target_kind: "lib".into(),
            unit_compilation_role_kind: CompilationUnitRoleKind::Target,
            unit_configuration_digest_full: [0; 32],
        };
        let span_semantic_occurrence_key =
            SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
                &unit_compilation_identity_key,
                "lib.rs",
                SpanByteRangeValue::try_from((0, 0))?,
                SpanDigestIdValue::from([0; 32]),
                &[],
            )?;
        let mut fake_stage_row_store = FakeStageRowStore::default();
        assert_eq!(
            fake_stage_row_store.read_span_by_digest(
                RunGenerationDigestValue([0; 32]),
                &span_semantic_occurrence_key
            )?,
            None
        );
        fake_stage_row_store.write_stage_unit_rows(&[])?;
        let first_stage_unit_row = StageUnitRowRecord {
            run_generation_digest_value: RunGenerationDigestValue([0; 32]),
            span_semantic_occurrence_key,
            file_source_content_bytes: vec![0, 255],
        };
        let second_stage_unit_row = StageUnitRowRecord {
            run_generation_digest_value: RunGenerationDigestValue([1; 32]),
            file_source_content_bytes: vec![42],
            ..first_stage_unit_row.clone()
        };
        fake_stage_row_store.write_stage_unit_rows(&[
            first_stage_unit_row.clone(),
            second_stage_unit_row.clone(),
        ])?;
        fake_stage_row_store.write_stage_unit_rows(std::slice::from_ref(&first_stage_unit_row))?;
        assert_eq!(fake_stage_row_store.stage_unit_stored_rows.len(), 2);
        for expected_stage_unit_row in [first_stage_unit_row, second_stage_unit_row] {
            assert_eq!(
                fake_stage_row_store.read_span_by_digest(
                    expected_stage_unit_row.run_generation_digest_value,
                    &expected_stage_unit_row.span_semantic_occurrence_key
                )?,
                Some(expected_stage_unit_row)
            );
        }
        Ok(())
    }
}
