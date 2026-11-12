#![allow(clippy::unwrap_used)]

use super::*;
use parseltongue_core_row_types::{StageTableRowRecord, StageUnitWriteRequest};
use std::collections::BTreeMap;

struct Source {
    published: Option<StageRunScopeValue>,
    records: BTreeMap<String, Vec<StageTableRowRecord>>,
    fail: bool,
    wrong_count: bool,
}

impl StageTableStorePort for Source {
    fn count_stage_table_rows(
        &self,
        scope: &StageRunScopeValue,
        table: &str,
    ) -> Result<usize, StoreBackendFailureError> {
        self.read_stage_table_rows(scope, table)
            .map(|rows| rows.len() + usize::from(self.wrong_count))
    }
    fn initialize_workspace_stage_schema(&self) -> Result<(), StoreBackendFailureError> {
        Ok(())
    }
    fn write_stage_table_unit(
        &self,
        _: &StageUnitWriteRequest<'_>,
        _: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        Err(build_analysis_error("read only"))
    }
    fn read_stage_table_rows(
        &self,
        _: &StageRunScopeValue,
        table: &str,
    ) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
        if self.fail {
            return Err(build_analysis_error("source failure"));
        }
        Ok(self.records.get(table).cloned().unwrap_or_default())
    }
    fn write_file_content_bytes(
        &self,
        _: &str,
        _: &[u8],
    ) -> Result<bool, StoreBackendFailureError> {
        Err(build_analysis_error("read only"))
    }
    fn read_file_content_bytes(&self, _: &str) -> Result<Vec<u8>, StoreBackendFailureError> {
        Err(build_analysis_error("read only"))
    }
    fn publish_workspace_generation_manifest(
        &self,
        _: &StageRunScopeValue,
        _: &str,
        _: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        Err(build_analysis_error("read only"))
    }
    fn read_workspace_publication_scope(
        &self,
    ) -> Result<Option<StageRunScopeValue>, StoreBackendFailureError> {
        Ok(self.published.clone())
    }
}

fn scope() -> StageRunScopeValue {
    StageRunScopeValue {
        run_generation_digest_value: "generation".into(),
        stage_attempt_sequence_value: "attempt".into(),
    }
}

fn source() -> Source {
    Source {
        published: Some(scope()),
        fail: false,
        wrong_count: false,
        records: BTreeMap::from([(
            "fixture-source-input-records".into(),
            vec![
                StageTableRowRecord {
                    stage_owned_table_name: "fixture-source-input-records".into(),
                    stage_row_record_key: "a".into(),
                    stage_row_payload_text: "unicode λ\0payload".into(),
                },
                StageTableRowRecord {
                    stage_owned_table_name: "fixture-source-input-records".into(),
                    stage_row_record_key: "b".into(),
                    stage_row_payload_text: "{}".into(),
                },
            ],
        )]),
    }
}

#[test]
fn test_published_generation_copy_equality() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.db"),
        directory.path(),
        1_288_490_188,
    )
    .unwrap();
    let source = source();
    let cancel = RunCancelFlagValue::default();
    assert_eq!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &[
                    "fixture-source-input-records",
                    "fixture-empty-input-records"
                ],
                &cancel
            )
            .unwrap(),
        2
    );
    assert_eq!(
        store
            .count_analysis_stage_rows(&scope(), "fixture-source-input-records")
            .unwrap(),
        2
    );
    assert_eq!(
        store
            .count_analysis_stage_rows(&scope(), "fixture-empty-input-records")
            .unwrap(),
        0
    );
    assert!(
        store
            .count_analysis_stage_rows(&scope(), "not-copied-input-records")
            .is_err()
    );
    assert!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &cancel
            )
            .is_err()
    );
}

#[test]
fn test_copy_rejects_partial_cancelled_and_failed_sources() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.db"),
        directory.path(),
        1_288_490_188,
    )
    .unwrap();
    let mut source = source();
    let cancel = RunCancelFlagValue::default();
    source.published = None;
    assert!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &cancel
            )
            .is_err()
    );
    source.published = Some(scope());
    source.fail = true;
    assert!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &cancel
            )
            .is_err()
    );
    assert!(
        store
            .count_analysis_stage_rows(&scope(), "fixture-source-input-records")
            .is_err()
    );
    source.fail = false;
    cancel.request_stage_work_cancellation();
    assert!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &cancel
            )
            .is_err()
    );
    assert!(
        store
            .count_analysis_stage_rows(&scope(), "fixture-source-input-records")
            .is_err()
    );
}

#[test]
fn test_count_mismatch_and_order_errors_rollback() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.db"),
        directory.path(),
        1_288_490_188,
    )
    .unwrap();
    let mut source = source();
    source.wrong_count = true;
    assert!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &RunCancelFlagValue::default()
            )
            .is_err()
    );
    assert!(
        store
            .count_analysis_stage_rows(&scope(), "fixture-source-input-records")
            .is_err()
    );
    source.wrong_count = false;
    source
        .records
        .get_mut("fixture-source-input-records")
        .unwrap()
        .reverse();
    assert!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &RunCancelFlagValue::default()
            )
            .is_err()
    );
    assert!(
        store
            .count_analysis_stage_rows(&scope(), "fixture-source-input-records")
            .is_err()
    );
    source
        .records
        .get_mut("fixture-source-input-records")
        .unwrap()
        .reverse();
    assert_eq!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &RunCancelFlagValue::default()
            )
            .unwrap(),
        2
    );
}

#[test]
fn test_duckdb_thread_export_determinism() {
    let exports = [1,4].map(|threads| {
        let directory = tempfile::tempdir().unwrap();
        let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(&directory.path().join("analysis.db"),directory.path(),1_288_490_188).unwrap();
        store._connection.lock().unwrap().execute("SET threads = ?", [threads]).unwrap();
        store.copy_published_stage_rows(&source(),&scope(),&["fixture-source-input-records"],&RunCancelFlagValue::default()).unwrap();
        let connection = store._connection.lock().unwrap();
        let mut statement = connection.prepare("SELECT generation_key,attempt_key,table_name,row_key,payload FROM \"frozen-generation-source-records\" ORDER BY generation_key,attempt_key,table_name,row_key").unwrap();
        statement.query_map([],|row| (0..5).map(|column|row.get::<_,String>(column)).collect::<Result<Vec<_>,_>>()).unwrap().collect::<Result<Vec<_>,_>>().unwrap()
    });
    assert_eq!(exports[0], exports[1]);
}
