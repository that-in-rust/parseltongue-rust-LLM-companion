#![allow(clippy::unwrap_used)]

use super::*;

fn read_stage_query_plan(query: String, parameters: Vec<String>) -> Vec<String> {
    let store = TursoWorkspaceStoreAdapter::open_turso_workspace_store(":memory:").unwrap();
    store.initialize_workspace_stage_schema().unwrap();
    store.store_runtime_handle_value.block_on(async {
        let connection = store.store_database_handle_value.connect().unwrap();
        let mut rows = connection
            .query(format!("EXPLAIN QUERY PLAN {query}"), parameters)
            .await
            .unwrap();
        let mut plan = Vec::new();
        while let Some(row) = rows.next().await.unwrap() {
            plan.push(row.get::<String>(3).unwrap());
        }
        plan
    })
}

fn read_visibility_query_plan(table: &str, revision: Option<i64>) -> Vec<String> {
    read_stage_query_plan(
        format!(
            "SELECT COUNT(*) FROM \"{table}\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND {}",
            build_crate_visibility_clause(table, revision)
        ),
        vec!["generation".into(), "attempt".into()],
    )
}

/// REQ-RUST-020.1
/// REQ-RUST-021.1
#[test]
fn test_namespace_cleanup_index_coverage() {
    STAGE_OWNED_TABLE_NAMES
        .iter()
        .filter(|(_, owner)| *owner == "syntax-namespace-input-records")
        .for_each(|(table, _)| {
            let plan = read_stage_query_plan(
                format!("DELETE FROM \"{table}\" WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=? AND stage_unit_work_key=?"),
                vec!["generation".into(), "attempt".into(), "crate/unit".into()],
            );
            eprintln!("{table}: {plan:#?}");
            assert!(
                plan.iter().any(|line| line.contains("stage_unit_work_key=?")),
                "namespace cleanup must probe the complete work-unit key: {plan:#?}"
            );
        });
}

/// REQ-RUST-024.1
#[test]
fn test_frozen_visibility_marker_probes() {
    [
        "syntax-namespace-input-items",
        "file-source-text-table",
        "span-reference-text-table",
    ]
    .into_iter()
    .for_each(|table| {
        let plan = read_visibility_query_plan(table, Some(1));
        eprintln!("{table}: {plan:#?}");
        let probes = plan.iter().filter(|line| line.contains("SEARCH frozen "));
        assert!(probes.clone().next().is_some(), "{plan:#?}");
        assert!(
            probes
                .clone()
                .all(|line| line.contains("stage_marker_key=?")),
            "frozen visibility must probe the complete marker key: {plan:#?}"
        );
    });
}

/// REQ-RUST-020.1
#[test]
fn test_namespace_completion_marker_probes() {
    let plan = read_visibility_query_plan("syntax-namespace-input-items", None);
    eprintln!("namespace completion: {plan:#?}");
    let probes = plan
        .iter()
        .filter(|line| line.contains("SEARCH completed "));
    assert!(probes.clone().next().is_some(), "{plan:#?}");
    assert!(
        probes
            .clone()
            .all(|line| line.contains("stage_row_record_key=?")),
        "namespace completion must probe the complete marker key: {plan:#?}"
    );
}
