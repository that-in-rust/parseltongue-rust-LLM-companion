#![allow(clippy::unwrap_used)]

use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter;

fn make_fixture_store_adapter() -> TursoWorkspaceStoreAdapter {
    let store = TursoWorkspaceStoreAdapter::open_turso_workspace_store(":memory:").unwrap();
    store.initialize_workspace_stage_schema().unwrap();
    store
}
fn make_fixture_stage_scope(attempt: &str) -> StageRunScopeValue {
    StageRunScopeValue {
        run_generation_digest_value: "staging".into(),
        stage_attempt_sequence_value: attempt.into(),
    }
}
fn write_fixture_stage_rows(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    stage: &str,
    unit: &str,
    inputs: &[&str],
    rows: &[StageTableRowRecord],
) -> Result<usize, StoreBackendFailureError> {
    store.write_stage_table_unit(
        &StageUnitWriteRequest {
            stage_run_scope_value: scope,
            stage_owner_name_value: stage,
            stage_unit_work_key: unit,
            stage_input_stage_names: inputs,
            stage_unit_output_rows: rows,
        },
        &RunCancelFlagValue::default(),
    )
}
fn make_fixture_graph_rows(key: &str) -> Vec<StageTableRowRecord> {
    vec![StageTableRowRecord {
        stage_owned_table_name: "crate-dependency-graph-table".into(),
        stage_row_record_key: key.into(),
        stage_row_payload_text: "{}".into(),
    }]
}
fn complete_fixture_stage_set(store: &dyn StageTableStorePort, scope: &StageRunScopeValue) {
    for stage in ["crate-graph", "source-ingest", "syntax-spans"] {
        write_fixture_stage_rows(store, scope, stage, "__complete__", &[], &[]).unwrap();
    }
}

#[test]
fn test_stage_row_visits_bounded() {
    let store = make_fixture_store_adapter();
    let scope = make_fixture_stage_scope("visit");
    let rows: Vec<_> = (0..3)
        .flat_map(|n| make_fixture_graph_rows(&n.to_string()))
        .collect();
    write_fixture_stage_rows(&store, &scope, "crate-graph", "unit", &[], &rows).unwrap();
    assert_eq!(
        store
            .count_stage_table_rows(&scope, "crate-dependency-graph-table")
            .unwrap(),
        3
    );
    let mut keys = Vec::new();
    assert_eq!(
        store
            .visit_stage_table_rows(&scope, "crate-dependency-graph-table", &mut |row| {
                keys.push(row.stage_row_record_key);
                Ok(())
            })
            .unwrap(),
        3
    );
    assert_eq!(keys, vec!["0", "1", "2"]);
    assert_eq!(
        store
            .count_stage_table_rows(&scope, "body-site-reference-table")
            .unwrap(),
        0
    );
    assert_eq!(
        store
            .visit_stage_table_rows(&scope, "body-site-reference-table", &mut |_| panic!(
                "empty"
            ))
            .unwrap(),
        0
    );
    assert!(
        store
            .visit_stage_table_rows(&scope, "crate-dependency-graph-table", &mut |_| Err(
                StoreBackendFailureError {
                    error_context_detail_text: "stop".into()
                }
            ))
            .unwrap_err()
            .to_string()
            .contains("stop")
    );
    assert_eq!(
        store
            .count_stage_table_rows(&scope, "crate-dependency-graph-table")
            .unwrap(),
        3
    );
    assert!(store.count_stage_table_rows(&scope, "unknown").is_err());
    assert!(
        store
            .visit_stage_table_rows(&scope, "unknown", &mut |_| Ok(()))
            .is_err()
    );
}

#[test]
fn test_stage_scope_preparation_atomic() {
    let store = make_fixture_store_adapter();
    let old = make_fixture_stage_scope("old");
    complete_fixture_stage_set(&store, &old);
    let published = store
        .publish_workspace_generation_manifest(
            &old,
            &"a".repeat(64),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let scope = make_fixture_stage_scope("next");
    let rows = make_fixture_graph_rows("next");
    write_fixture_stage_rows(&store, &scope, "crate-graph", "unit", &[], &rows).unwrap();
    let prepared = store
        .prepare_workspace_generation_scope(&scope, &"b".repeat(64), &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        Some(published.clone())
    );
    assert_eq!(
        store
            .read_stage_table_rows(&prepared, "crate-dependency-graph-table")
            .unwrap(),
        rows
    );
    assert!(
        store
            .read_stage_table_rows(&scope, "crate-dependency-graph-table")
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .publish_workspace_generation_manifest(
                &prepared,
                &prepared.run_generation_digest_value,
                &RunCancelFlagValue::default()
            )
            .is_err()
    );
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        Some(published.clone())
    );
    complete_fixture_stage_set(&store, &prepared);
    assert_eq!(
        store
            .publish_workspace_generation_manifest(
                &prepared,
                &prepared.run_generation_digest_value,
                &RunCancelFlagValue::default()
            )
            .unwrap(),
        prepared
    );
    let empty = make_fixture_stage_scope("empty");
    assert!(
        store
            .prepare_workspace_generation_scope(
                &empty,
                &"c".repeat(64),
                &RunCancelFlagValue::default()
            )
            .is_ok()
    );
    let cancelled = RunCancelFlagValue::default();
    cancelled.request_stage_work_cancellation();
    assert!(
        store
            .prepare_workspace_generation_scope(&prepared, &"d".repeat(64), &cancelled)
            .is_err()
    );
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        Some(prepared)
    );
}

#[test]
fn test_scope_preparation_rejects_outputs() {
    let store = make_fixture_store_adapter();
    let scope = make_fixture_stage_scope("late");
    let rows = vec![StageTableRowRecord {
        stage_owned_table_name: "span-reference-text-table".into(),
        stage_row_record_key: "span".into(),
        stage_row_payload_text: "{}".into(),
    }];
    write_fixture_stage_rows(&store, &scope, "syntax-spans", "file", &[], &rows).unwrap();
    assert!(
        store
            .prepare_workspace_generation_scope(
                &scope,
                &"a".repeat(64),
                &RunCancelFlagValue::default()
            )
            .unwrap_err()
            .to_string()
            .contains("StageScopePreparationError")
    );
    assert_eq!(
        store
            .read_stage_table_rows(&scope, "span-reference-text-table")
            .unwrap(),
        rows
    );
}

#[test]
fn test_ingest_bytes_stay_identical() {
    let store = make_fixture_store_adapter();
    let bytes = [0, 255, b'\r', b'\n'];
    assert!(store.write_file_content_bytes("digest", &bytes).unwrap());
    assert_eq!(store.read_file_content_bytes("digest").unwrap(), bytes);
    assert!(store.read_file_content_bytes("absent").is_err());
}
#[test]
fn test_unchanged_digest_skips_write() {
    let store = make_fixture_store_adapter();
    assert!(store.write_file_content_bytes("digest", b"same").unwrap());
    assert!(!store.write_file_content_bytes("digest", b"same").unwrap());
    assert!(
        store
            .write_file_content_bytes("digest", b"collision")
            .is_err()
    );
}

#[test]
fn test_conflict_retry_is_bounded() {
    let mut calls = 0;
    let error = parseltongue_turso_store_adapter::retry_bounded_store_write(
        &RunCancelFlagValue::default(),
        || {
            calls += 1;
            Err::<(), _>(turso::Error::BusySnapshot("injected conflict".into()))
        },
    )
    .unwrap_err();
    assert_eq!(calls, 6);
    assert!(error.to_string().contains("StoreWriteConflictError"));
    calls = 0;
    assert!(
        parseltongue_turso_store_adapter::retry_bounded_store_write(
            &RunCancelFlagValue::default(),
            || {
                calls += 1;
                Err::<(), _>(turso::Error::Constraint("not a conflict".into()))
            }
        )
        .is_err()
    );
    assert_eq!(calls, 1);
}
#[test]
fn test_stage_rerun_is_identical() {
    let store = make_fixture_store_adapter();
    let scope = make_fixture_stage_scope("one");
    let rows = make_fixture_graph_rows("unit");
    write_fixture_stage_rows(&store, &scope, "crate-graph", "unit", &[], &rows).unwrap();
    let first = store
        .read_stage_table_rows(&scope, "crate-dependency-graph-table")
        .unwrap();
    write_fixture_stage_rows(&store, &scope, "crate-graph", "unit", &[], &rows).unwrap();
    assert_eq!(
        first,
        store
            .read_stage_table_rows(&scope, "crate-dependency-graph-table")
            .unwrap()
    );
}

#[test]
fn test_bulk_failure_restores_unit() {
    let store = make_fixture_store_adapter();
    let scope = make_fixture_stage_scope("batch");
    let original = make_fixture_graph_rows("original");
    write_fixture_stage_rows(&store, &scope, "crate-graph", "unit", &[], &original).unwrap();
    let mut rows: Vec<_> = (0..260)
        .map(|index| make_fixture_graph_rows(&format!("row-{index}")).remove(0))
        .collect();
    rows.push(rows[0].clone());
    assert!(write_fixture_stage_rows(&store, &scope, "crate-graph", "unit", &[], &rows).is_err());
    assert_eq!(
        store
            .read_stage_table_rows(&scope, "crate-dependency-graph-table")
            .unwrap(),
        original
    );
    rows.pop();
    assert_eq!(
        write_fixture_stage_rows(&store, &scope, "crate-graph", "unit", &[], &rows).unwrap(),
        260
    );
    assert_eq!(
        store
            .read_stage_table_rows(&scope, "crate-dependency-graph-table")
            .unwrap()
            .len(),
        260
    );
    write_fixture_stage_rows(&store, &scope, "crate-graph", "unit", &[], &[]).unwrap();
    assert!(
        store
            .read_stage_table_rows(&scope, "crate-dependency-graph-table")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn test_readers_survive_concurrent_units() {
    let store = make_fixture_store_adapter();
    let scope = make_fixture_stage_scope("readers");
    store
        .write_file_content_bytes("source", b"source bytes")
        .unwrap();
    std::thread::scope(|threads| {
        let jobs: Vec<_> = (0..4)
            .map(|worker| {
                let store = &store;
                let scope = &scope;
                threads.spawn(move || {
                    for unit in 0..40 {
                        assert_eq!(
                            store.read_file_content_bytes("source").unwrap(),
                            b"source bytes"
                        );
                        let rows: Vec<_> = (0..128)
                            .map(|row| {
                                make_fixture_graph_rows(&format!("{worker}/{unit}/{row}")).remove(0)
                            })
                            .collect();
                        write_fixture_stage_rows(
                            store,
                            scope,
                            "crate-graph",
                            &format!("{worker}/{unit}"),
                            &[],
                            &rows,
                        )
                        .unwrap();
                    }
                })
            })
            .collect();
        for job in jobs {
            job.join().unwrap();
        }
    });
    assert_eq!(
        store
            .read_stage_table_rows(&scope, "crate-dependency-graph-table")
            .unwrap()
            .len(),
        4 * 40 * 128
    );
}
#[test]
fn test_stage_refuses_missing_input() {
    let store = make_fixture_store_adapter();
    let scope = make_fixture_stage_scope("one");
    let error = write_fixture_stage_rows(
        &store,
        &scope,
        "syntax-spans",
        "unit",
        &["source-ingest"],
        &[],
    )
    .unwrap_err();
    assert!(error.to_string().contains("StageInputMissingError"));
}
#[test]
fn test_stage_table_access_restricted() {
    let store = make_fixture_store_adapter();
    let scope = make_fixture_stage_scope("one");
    assert!(
        write_fixture_stage_rows(
            &store,
            &scope,
            "syntax-spans",
            "unit",
            &[],
            &make_fixture_graph_rows("unit")
        )
        .is_err()
    );
    assert!(
        store
            .read_stage_table_rows(&scope, "hld-future-dependency-edges")
            .is_err()
    );
}
#[test]
fn test_generation_publication_is_atomic() {
    let store = make_fixture_store_adapter();
    let scope = make_fixture_stage_scope("one");
    assert!(store.read_workspace_publication_scope().unwrap().is_none());
    assert!(
        store
            .publish_workspace_generation_manifest(
                &scope,
                &"a".repeat(64),
                &RunCancelFlagValue::default()
            )
            .is_err()
    );
    complete_fixture_stage_set(&store, &scope);
    let published = store
        .publish_workspace_generation_manifest(
            &scope,
            &"a".repeat(64),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        Some(published)
    );
}
#[test]
fn test_concurrent_writers_lose_nothing() {
    let store = make_fixture_store_adapter();
    let scope = make_fixture_stage_scope("one");
    std::thread::scope(|threads| {
        let handles: Vec<_> = (0..4)
            .map(|n| {
                let store = &store;
                let scope = &scope;
                threads.spawn(move || {
                    write_fixture_stage_rows(
                        store,
                        scope,
                        "crate-graph",
                        &format!("unit{n}"),
                        &[],
                        &make_fixture_graph_rows(&format!("unit{n}")),
                    )
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap().unwrap();
        }
    });
    assert_eq!(
        store
            .read_stage_table_rows(&scope, "crate-dependency-graph-table")
            .unwrap()
            .len(),
        4
    );
}
#[test]
fn test_cancel_publication_stays_consistent() {
    let store = make_fixture_store_adapter();
    let first = make_fixture_stage_scope("first");
    complete_fixture_stage_set(&store, &first);
    let published = store
        .publish_workspace_generation_manifest(
            &first,
            &"a".repeat(64),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let second = make_fixture_stage_scope("second");
    complete_fixture_stage_set(&store, &second);
    let cancel = RunCancelFlagValue::default();
    cancel.request_stage_work_cancellation();
    assert!(
        store
            .publish_workspace_generation_manifest(&second, &"b".repeat(64), &cancel)
            .is_err()
    );
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        Some(published)
    );
    let request = StageUnitWriteRequest {
        stage_run_scope_value: &second,
        stage_owner_name_value: "crate-graph",
        stage_unit_work_key: "cancelled",
        stage_input_stage_names: &[],
        stage_unit_output_rows: &[],
    };
    assert!(store.write_stage_table_unit(&request, &cancel).is_err());
    assert!(
        !store
            .read_stage_table_rows(&second, "stage-run-generation-records")
            .unwrap()
            .iter()
            .any(|r| r.stage_row_record_key.contains("cancelled"))
    );
}
