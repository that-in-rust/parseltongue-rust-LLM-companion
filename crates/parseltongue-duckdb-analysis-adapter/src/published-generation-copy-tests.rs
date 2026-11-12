#![allow(clippy::unwrap_used)]

use super::*;
use parseltongue_core_row_types::{StageTableRowRecord, StageUnitWriteRequest};
use serde_json::json;
use std::collections::BTreeMap;

fn namespace_source_fixture() -> Source {
    let unit = json!({"unit_cargo_package_identifier":"fixture","unit_cargo_target_name":"fixture","unit_cargo_target_kind":"lib","unit_compilation_role_kind":"target","unit_configuration_digest_full":"0".repeat(64)});
    let root = json!({"scope_compilation_unit_key":unit,"scope_semantic_path_parts":[]});
    let item = json!({"target_identity_variant_kind":"builtin","target_builtin_name_value":"u8"});
    let scope_row = json!({"scope_module_identity_key":root,"scope_parent_identity_key":null,"scope_parent_lookup_flag":false,"scope_module_boundary_flag":true,"scope_implicit_prelude_key":null,"scope_cargo_unit_key":"fixture-unit"});
    let binding = json!({"binding_lexical_scope_key":root,"binding_resolved_name_text":"r#Item","binding_namespace_kind_value":"type","binding_target_identity_key":item,"binding_target_child_scope":null,"binding_visibility_scope_kind":{"visibility_scope_kind_value":"public"},"binding_selection_priority_kind":"declaration","binding_import_evidence_keys":[],"binding_target_metadata_value":{"target_symbol_kind_value":"Struct"}});
    let path = json!({"path_site_occurrence_key":{"site":"path"},"path_lexical_scope_key":root,"path_source_segment_rows":[{"kind":"name","name":"r#Item"}],"path_requested_namespace_kind":"Type","path_resolution_source_kind":"Signature","path_unavailable_reason_kind":null,"path_source_metadata_value":{"path_source_absolute_path":"/fixture/lib.rs"}});
    let mut records = NAMESPACE_SOURCE_TABLE_NAMES
        .iter()
        .map(|table| (table.to_string(), vec![]))
        .collect::<BTreeMap<_, _>>();
    for (table, value) in [
        ("syntax-namespace-input-scopes", scope_row),
        ("syntax-namespace-input-items", binding),
        ("syntax-namespace-input-paths", path),
        (
            "crate-dependency-graph-table",
            json!({"unit_compilation_identity_key":"fixture-unit","unit_rust_edition_value":"2024","unit_normal_dependency_rows":[]}),
        ),
    ] {
        let text = value.to_string();
        records
            .entry(table.into())
            .or_default()
            .push(StageTableRowRecord {
                stage_owned_table_name: table.into(),
                stage_row_record_key: blake3::hash(text.as_bytes()).to_hex().to_string(),
                stage_row_payload_text: text,
            });
    }
    Source {
        published: Some(scope()),
        records,
        fail: false,
        wrong_count: false,
    }
}

#[test]
fn frozen_namespace_normalization_contract() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.duckdb"),
        directory.path(),
        64 * 1024 * 1024,
    )
    .unwrap();
    store
        .copy_published_stage_rows(
            &namespace_source_fixture(),
            &scope(),
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    assert_eq!(
        store
            .normalize_frozen_namespace_rows(&scope(), &RunCancelFlagValue::default())
            .unwrap(),
        4
    );
    let connection = store._connection.lock().unwrap();
    let name:String=connection.query_row("SELECT name FROM \"namespace-binding-declaration-records\" ORDER BY generation_key,attempt_key,scope_key,name,namespace,target_key,priority,evidence_key",[],|row|row.get(0)).unwrap();
    assert_eq!(name, "Item");
    let segments: usize = connection
        .query_row(
            "SELECT count(*) FROM \"namespace-path-segment-records\"",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(segments, 1);
    let generation:String=connection.query_row("SELECT generation_key FROM \"namespace-normalized-source-manifests\" ORDER BY generation_key,attempt_key",[],|row|row.get(0)).unwrap();
    assert_eq!(generation, scope().run_generation_digest_value);
}

#[test]
fn frozen_namespace_generation_integrity() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.duckdb"),
        directory.path(),
        64 * 1024 * 1024,
    )
    .unwrap();
    let first = scope();
    let second = StageRunScopeValue {
        run_generation_digest_value: "second".into(),
        ..scope()
    };
    let mut source = namespace_source_fixture();
    [&first, &second].into_iter().for_each(|scope| {
        source.published = Some(scope.clone());
        store
            .copy_published_stage_rows(
                &source,
                scope,
                NAMESPACE_SOURCE_TABLE_NAMES,
                &RunCancelFlagValue::default(),
            )
            .unwrap();
        assert_eq!(
            store
                .normalize_frozen_namespace_rows(scope, &RunCancelFlagValue::default())
                .unwrap(),
            4
        );
    });
    let connection = store._connection.lock().unwrap();
    let count:usize=connection.query_row("SELECT count(*) FROM \"namespace-binding-declaration-records\" WHERE generation_key='generation'",[],|row|row.get(0)).unwrap();
    assert_eq!(count, 1);
    drop(connection);
    let invalid = StageRunScopeValue {
        run_generation_digest_value: "corrupt".into(),
        ..scope()
    };
    source.published = Some(invalid.clone());
    store
        .copy_published_stage_rows(
            &source,
            &invalid,
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    store._connection.lock().unwrap().execute("UPDATE \"frozen-generation-source-records\" SET payload=replace(payload,'\"scope_parent_lookup_flag\":false','\"scope_parent_lookup_flag\":true') WHERE generation_key='corrupt' AND table_name='syntax-namespace-input-scopes'",[]).unwrap();
    let error = store
        .normalize_frozen_namespace_rows(&invalid, &RunCancelFlagValue::default())
        .unwrap_err();
    assert!(error.to_string().contains("digest changed"), "{error}");
    let count:usize=store._connection.lock().unwrap().query_row("SELECT count(*) FROM \"namespace-normalized-source-manifests\" WHERE generation_key='corrupt'",[],|row|row.get(0)).unwrap();
    assert_eq!(count, 0);
}

fn append_namespace_fixture_value(source: &mut Source, table: &str, value: serde_json::Value) {
    let payload = value.to_string();
    let rows = source.records.get_mut(table).unwrap();
    rows.push(StageTableRowRecord {
        stage_owned_table_name: table.into(),
        stage_row_record_key: blake3::hash(payload.as_bytes()).to_hex().to_string(),
        stage_row_payload_text: payload,
    });
    rows.sort_by(|left, right| left.stage_row_record_key.cmp(&right.stage_row_record_key));
}

#[test]
fn frozen_namespace_sql_resolution_contract() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.duckdb"),
        directory.path(),
        64 * 1024 * 1024,
    )
    .unwrap();
    assert!(
        store
            .resolve_frozen_namespace_rows(&scope(), &RunCancelFlagValue::default())
            .is_err()
    );
    let mut source = namespace_source_fixture();
    store
        .copy_published_stage_rows(
            &source,
            &scope(),
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    store
        .normalize_frozen_namespace_rows(&scope(), &RunCancelFlagValue::default())
        .unwrap();
    let report = store
        .resolve_frozen_namespace_rows(&scope(), &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(report.binding_set_digest_value.len(), 64);
    assert!(report.fixed_point_round_count > 0);
    let count:usize=store._connection.lock().unwrap().query_row("SELECT count(*) FROM \"namespace-path-candidate-records\" WHERE namespace='type' AND state='resolved'",[],|row|row.get(0)).unwrap();
    assert_eq!(count, 1);
    let cancelled = RunCancelFlagValue::default();
    cancelled.request_stage_work_cancellation();
    assert!(
        store
            .resolve_frozen_namespace_rows(&scope(), &cancelled)
            .is_err()
    );
    let empty = StageRunScopeValue {
        run_generation_digest_value: "empty".into(),
        ..scope()
    };
    source.published = Some(empty.clone());
    source.records.values_mut().for_each(Vec::clear);
    store
        .copy_published_stage_rows(
            &source,
            &empty,
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    store
        .normalize_frozen_namespace_rows(&empty, &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(
        store
            .resolve_frozen_namespace_rows(&empty, &RunCancelFlagValue::default())
            .unwrap()
            .fixed_point_round_count,
        1
    );
}

#[test]
fn frozen_namespace_latest_textual_lookup() {
    [false,true].into_iter().for_each(|null_latest|{
        let directory=tempfile::tempdir().unwrap();
        let store=DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(&directory.path().join("analysis.duckdb"),directory.path(),64*1024*1024).unwrap();
        let mut source=namespace_source_fixture();
        let scope_row:serde_json::Value=serde_json::from_str(&source.records["syntax-namespace-input-scopes"][0].stage_row_payload_text).unwrap();
        let root=&scope_row["scope_module_identity_key"];
        let target=|name:&str|json!({"target_identity_variant_kind":"builtin","target_builtin_name_value":name});
        ["old","new","module"].into_iter().for_each(|name|{
            append_namespace_fixture_value(&mut source,"syntax-namespace-input-items",json!({"binding_lexical_scope_key":root,"binding_resolved_name_text":"m","binding_namespace_kind_value":"macro","binding_target_identity_key":target(name),"binding_target_child_scope":null,"binding_visibility_scope_kind":{"visibility_scope_kind_value":"restricted","visibility_restricted_scope_key":root},"binding_selection_priority_kind":"declaration","binding_import_evidence_keys":[],"binding_target_metadata_value":{"target_symbol_kind_value":"Macro","target_macro_definition_record":{"macro_declaration_style_flag":name=="module"}}}));
        });
        append_namespace_fixture_value(&mut source,"syntax-namespace-input-macros",json!({"invocation_site_occurrence_key":{"site":"macro"},"invocation_lexical_scope_key":root,"invocation_macro_path_segments":[{"kind":"name","name":"m"}],"invocation_macro_input_text":""}));
        append_namespace_fixture_value(&mut source,"syntax-namespace-input-imports",json!({"import_site_occurrence_key":{"site":"import"},"import_lexical_scope_key":root,"import_path_segment_rows":[{"kind":"name","name":"m"}],"import_binding_alias_text":"Alias","import_glob_expansion_flag":false,"import_visibility_scope_kind":{"visibility_scope_kind_value":"public"}}));
        ["macro","import"].into_iter().for_each(|site|{
            [(1,target("old")),(2,if null_latest {serde_json::Value::Null}else{target("new")})].into_iter().for_each(|(revision,target)|{
                append_namespace_fixture_value(&mut source,"legacy-macro-scope-records",json!({"legacy_lookup_identity_key":format!("lookup-{site}"),"legacy_source_site_key":{"site":site},"legacy_macro_name_text":"m","legacy_target_identity_key":target,"legacy_scope_revision_value":revision,"legacy_source_order_parts":[10,4]}));
            });
        });
        store.copy_published_stage_rows(&source,&scope(),NAMESPACE_SOURCE_TABLE_NAMES,&RunCancelFlagValue::default()).unwrap();
        store.normalize_frozen_namespace_rows(&scope(),&RunCancelFlagValue::default()).unwrap();
        store.resolve_frozen_namespace_rows(&scope(),&RunCancelFlagValue::default()).unwrap();
        let connection=store._connection.lock().unwrap();
        let resolved:String=connection.query_row("SELECT target_key FROM \"namespace-path-candidate-records\" WHERE request_key=? AND namespace='macro' ORDER BY generation_key,attempt_key,request_key,target_key,namespace,binding_scope,binding_name",[json!({"site":"macro"}).to_string()],|row|row.get(0)).unwrap();
        assert_eq!(resolved,target(if null_latest {"module"}else{"new"}).to_string());
        let imported:String=connection.query_row("SELECT target_key FROM \"namespace-binding-round-results\" WHERE name='Alias' ORDER BY generation_key,attempt_key,scope_key,name,namespace,target_key,evidence_key",[],|row|row.get(0)).unwrap();
        assert_eq!(imported,resolved);
        let private:usize=connection.query_row("SELECT count(*) FROM \"namespace-binding-declaration-records\" WHERE target_key IN (?,?)",duckdb::params![target("old").to_string(),target("new").to_string()],|row|row.get(0)).unwrap();
        assert_eq!(private,0);
    });
}

#[test]
fn frozen_namespace_normalization_empty_and_errors() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.duckdb"),
        directory.path(),
        64 * 1024 * 1024,
    )
    .unwrap();
    assert!(
        store
            .normalize_frozen_namespace_rows(&scope(), &RunCancelFlagValue::default())
            .is_err()
    );
    let mut source = namespace_source_fixture();
    source.records.values_mut().for_each(Vec::clear);
    store
        .copy_published_stage_rows(
            &source,
            &scope(),
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    assert_eq!(
        store
            .normalize_frozen_namespace_rows(&scope(), &RunCancelFlagValue::default())
            .unwrap(),
        0
    );
    let cancelled = RunCancelFlagValue::default();
    cancelled.request_stage_work_cancellation();
    assert!(
        store
            .normalize_frozen_namespace_rows(&scope(), &cancelled)
            .is_err()
    );
    let invalid = StageRunScopeValue {
        stage_attempt_sequence_value: "invalid".into(),
        ..scope()
    };
    source.published = Some(invalid.clone());
    source
        .records
        .get_mut("syntax-namespace-input-scopes")
        .unwrap()
        .push(StageTableRowRecord {
            stage_owned_table_name: "syntax-namespace-input-scopes".into(),
            stage_row_record_key: "bad".into(),
            stage_row_payload_text: "null".into(),
        });
    store
        .copy_published_stage_rows(
            &source,
            &invalid,
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    assert!(
        store
            .normalize_frozen_namespace_rows(&invalid, &RunCancelFlagValue::default())
            .is_err()
    );
    let connection = store._connection.lock().unwrap();
    let count:usize=connection.query_row("SELECT count(*) FROM \"namespace-normalized-source-manifests\" WHERE attempt_key='invalid'",[],|row|row.get(0)).unwrap();
    assert_eq!(count, 0);
}

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

#[test]
fn test_turso_published_appender_handoff() {
    let directory = tempfile::tempdir().unwrap();
    let source =
        parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter::open_turso_workspace_store(
            ":memory:",
        )
        .unwrap();
    source.initialize_workspace_stage_schema().unwrap();
    let scope = scope();
    let cancel = RunCancelFlagValue::default();
    let records = [StageTableRowRecord {
        stage_owned_table_name: "span-reference-text-table".into(),
        stage_row_record_key: "source/λ".into(),
        stage_row_payload_text: "{\"source\":\"identity\"}".into(),
    }];
    for stage in ["crate-graph", "source-ingest", "syntax-spans"] {
        source
            .write_stage_table_unit(
                &StageUnitWriteRequest {
                    stage_run_scope_value: &scope,
                    stage_owner_name_value: stage,
                    stage_unit_work_key: "__complete__",
                    stage_input_stage_names: &[],
                    stage_unit_output_rows: if stage == "syntax-spans" {
                        &records
                    } else {
                        &[]
                    },
                },
                &cancel,
            )
            .unwrap();
    }
    let published = source
        .publish_workspace_generation_manifest(&scope, &scope.run_generation_digest_value, &cancel)
        .unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.db"),
        directory.path(),
        1_288_490_188,
    )
    .unwrap();
    assert_eq!(
        store
            .copy_published_stage_rows(&source, &published, &["span-reference-text-table"], &cancel)
            .unwrap(),
        1
    );
    assert_eq!(
        store
            .count_analysis_stage_rows(&published, "span-reference-text-table")
            .unwrap(),
        source
            .count_stage_table_rows(&published, "span-reference-text-table")
            .unwrap()
    );
    let connection = store._connection.lock().unwrap();
    let payload: String = connection.query_row(
        "SELECT payload FROM \"frozen-generation-source-records\" ORDER BY generation_key,attempt_key,table_name,row_key",[],|row|row.get(0),
    ).unwrap();
    assert_eq!(payload, records[0].stage_row_payload_text);
}
