#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::syntax_namespace_record_codec::*;
use crate::{SyntaxFileInputData, extract_file_syntax_rows, persist_impl_header_rows};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path, sync::Mutex};

pub(crate) struct OriginalCallFixtureStore {
    pub(crate) rows: Mutex<Vec<StageTableRowRecord>>,
    frozen_read_only_flag: std::sync::atomic::AtomicBool,
}
impl StageTableStorePort for OriginalCallFixtureStore {
    fn initialize_workspace_stage_schema(&self) -> Result<(), StoreBackendFailureError> {
        Ok(())
    }
    fn publish_analysis_input_snapshot(
        &self,
        scope: &StageRunScopeValue,
        _: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        if self
            .frozen_read_only_flag
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err(StoreBackendFailureError {
                error_context_detail_text: "frozen source cannot be republished".into(),
            });
        }
        Ok(scope.clone())
    }
    fn write_stage_table_unit(
        &self,
        request: &StageUnitWriteRequest<'_>,
        _: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        self.rows
            .lock()
            .unwrap()
            .extend_from_slice(request.stage_unit_output_rows);
        Ok(request.stage_unit_output_rows.len())
    }
    fn read_stage_table_rows(
        &self,
        _: &StageRunScopeValue,
        table: &str,
    ) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|row| row.stage_owned_table_name == table)
            .cloned()
            .collect())
    }
    fn write_file_content_bytes(
        &self,
        _: &str,
        _: &[u8],
    ) -> Result<bool, StoreBackendFailureError> {
        unreachable!()
    }
    fn read_file_content_bytes(&self, _: &str) -> Result<Vec<u8>, StoreBackendFailureError> {
        unreachable!()
    }
    fn publish_workspace_generation_manifest(
        &self,
        _: &StageRunScopeValue,
        _: &str,
        _: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        unreachable!()
    }
    fn read_workspace_publication_scope(
        &self,
    ) -> Result<Option<StageRunScopeValue>, StoreBackendFailureError> {
        Ok(None)
    }
}

pub(crate) fn make_call_syntax_input(source: &str) -> SyntaxFileInputData {
    SyntaxFileInputData {
        file_source_absolute_path: "/fixture/src/lib.rs".into(),
        file_source_content_bytes: source.as_bytes().to_vec(),
        unit_compilation_identity_key: "raw-unit".into(),
        unit_rust_edition_value: "2024".into(),
        unit_target_cfg_values: BTreeSet::new(),
        file_module_directory_path: "/fixture/src".into(),
        unit_metadata_environment_values: Default::default(),
        file_parse_context_kind: "items".into(),
        file_expansion_provenance_chain: vec![],
    }
}
pub(crate) fn make_call_fixture_unit() -> CrateCompilationUnitKey {
    CrateCompilationUnitKey {
        unit_cargo_package_identifier: "fixture@1".into(),
        unit_cargo_target_name: "fixture".into(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [2; 32],
    }
}
pub(crate) fn make_call_fixture_scope() -> StageRunScopeValue {
    StageRunScopeValue {
        run_generation_digest_value: "01".repeat(32),
        stage_attempt_sequence_value: "attempt".into(),
    }
}
pub(crate) fn make_call_fixture_occurrence(
    start: u64,
    end: u64,
    text: &str,
) -> SemanticOccurrenceKeyValue {
    SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        &make_call_fixture_unit(),
        "src/lib.rs",
        SpanByteRangeValue::try_from((start, end)).unwrap(),
        (*blake3::hash(text.as_bytes()).as_bytes()).into(),
        &[],
    )
    .unwrap()
}
pub(crate) fn make_call_fixture_row(table: &str, value: Value) -> StageTableRowRecord {
    StageTableRowRecord {
        stage_owned_table_name: table.into(),
        stage_row_record_key: blake3::hash(value.to_string().as_bytes())
            .to_hex()
            .to_string(),
        stage_row_payload_text: value.to_string(),
    }
}
fn read_call_fixture_values(store: &OriginalCallFixtureStore, table: &str) -> Vec<Value> {
    store
        .read_stage_table_rows(&make_call_fixture_scope(), table)
        .unwrap()
        .iter()
        .map(|row| serde_json::from_str(&row.stage_row_payload_text).unwrap())
        .collect()
}
// Namespace outcomes are supplied at the existing stage seam. This is a component
// fixture, not the parent's real-source/DuckDB integration acceptance test.
pub(crate) fn make_call_fixture_graph(
    source: &str,
) -> (OriginalCallFixtureStore, HldMaterializationInputRecord) {
    let rows = extract_file_syntax_rows(&make_call_syntax_input(source))
        .unwrap()
        .stage_unit_output_rows;
    let store = OriginalCallFixtureStore {
        rows: Mutex::new(rows),
        frozen_read_only_flag: std::sync::atomic::AtomicBool::new(false),
    };
    let spans = read_call_fixture_values(&store, "span-reference-text-table");
    let unit = encode_compilation_unit_value(&make_call_fixture_unit());
    let mut extra = vec![
        make_call_fixture_row(
            "crate-dependency-graph-table",
            json!({"unit_compilation_identity_key":"raw-unit", "unit_package_root_path":"/fixture"}),
        ),
        make_call_fixture_row(
            "syntax-namespace-input-scopes",
            json!({"scope_cargo_unit_key":"raw-unit", "scope_module_identity_key":{"scope_semantic_path_parts":[], "scope_compilation_unit_key":unit}}),
        ),
    ];
    let buffer = spans
        .iter()
        .find(|span| {
            span["span_declared_name_value"] == "Buffer"
                && span["span_syntax_kind_value"] == "STRUCT"
        })
        .unwrap();
    let occurrence = make_call_fixture_occurrence(
        buffer["span_byte_start_offset"].as_u64().unwrap(),
        buffer["span_byte_end_offset"].as_u64().unwrap(),
        buffer["span_source_code_text"].as_str().unwrap(),
    );
    let metadata = json!({"target_symbol_kind_value":"Struct", "target_source_file_path":"/fixture/src/lib.rs", "target_source_range_start":buffer["span_byte_start_offset"], "target_source_range_end":buffer["span_byte_end_offset"], "target_compilation_scope_record":{"unit_semantic_identity_record":unit}});
    extra.push(make_call_fixture_row("syntax-namespace-input-items", json!({"binding_target_identity_key":encode_symbol_target_value(&NameSymbolTargetKey::Definition(occurrence)), "binding_target_metadata_value":metadata})));
    for (start, _) in source.match_indices("Buffer") {
        extra.push(make_call_fixture_row("path-resolution-site-records", json!({
            "site_compilation_scope_record":{"unit_semantic_identity_record":unit},
            "site_source_absolute_path":"/fixture/src/lib.rs", "site_source_range_start":start, "site_source_range_end":start+6,
            "site_expansion_provenance_chain":[], "path_resolution_state_kind":"resolved", "path_target_symbol_rows":[metadata]
        })));
    }
    store.rows.lock().unwrap().extend(extra);
    persist_impl_header_rows(
        &store,
        &make_call_fixture_scope(),
        Path::new("/fixture"),
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    let nodes = spans
        .iter()
        .filter(|span| {
            span["span_configuration_active_flag"] == true
                && matches!(
                    span["span_syntax_kind_value"].as_str(),
                    Some("FN" | "STRUCT")
                )
        })
        .map(|span| HldNodeInputRecord {
            run_generation_digest_value: RunGenerationDigestValue([1; 32]),
            node_target_reference: NameSymbolTargetKey::Definition(make_call_fixture_occurrence(
                span["span_byte_start_offset"].as_u64().unwrap(),
                span["span_byte_end_offset"].as_u64().unwrap(),
                span["span_source_code_text"].as_str().unwrap(),
            ))
            .into(),
            node_input_category: HldNodeInputCategory::LocalItem,
            node_item_kind_value: Some(if span["span_syntax_kind_value"] == "FN" {
                HldItemKindValue::AssociatedFunction
            } else {
                HldItemKindValue::Struct
            }),
            node_owner_kind_value: Some(HldOwnerKindValue::Impl),
            node_function_body_ancestry_flag: false,
            node_effective_visibility_kind: Some(HldEffectiveVisibilityKind::PrivateEquivalent),
            node_reachable_outside_crate_flag: Some(false),
            node_source_relative_path: Some("src/lib.rs".into()),
            node_qualified_path_text: span["span_declared_name_value"].as_str().unwrap().into(),
            node_signature_text: None,
        })
        .collect();
    (
        store,
        HldMaterializationInputRecord {
            run_generation_digest_value: RunGenerationDigestValue([1; 32]),
            stage_attempt_sequence_value: "attempt".into(),
            semantic_input_completeness_kind: HldInputCompletenessKind::IncompleteTypedInput,
            source_coverage_kind: HldSourceCoverageKind::FullSemantic,
            input_unit_provenance_records: vec![HldUnitProvenanceRecord {
                unit_compilation_identity_key: make_call_fixture_unit(),
                unit_origin_kind: HldUnitOriginKind::Workspace,
            }],
            input_node_records: nodes,
            input_edge_records: vec![],
        },
    )
}
const SOURCE: &str = "struct Buffer; impl Buffer { fn new() -> Self { loop {} } } impl Default for Buffer { fn default() -> Self { Buffer::new() } }";
fn read_call_fixture_edges(
    store: &OriginalCallFixtureStore,
    input: &HldMaterializationInputRecord,
) -> Vec<HldEdgeInputRecord> {
    resolve_original_call_edges(
        store,
        &make_call_fixture_scope(),
        input,
        &RunCancelFlagValue::default(),
    )
    .unwrap()
}

/// REQ-RUST-034.0 / REQ-RUST-035.0 / REQ-RUST-036.0
#[test]
fn test_resolve_original_member_call() {
    let (store, input) = make_call_fixture_graph(SOURCE);
    let edges = read_call_fixture_edges(&store, &input);
    assert_eq!(edges.len(), 1);
    let source = input
        .input_node_records
        .iter()
        .find(|n| n.node_qualified_path_text == "default")
        .unwrap();
    let target = input
        .input_node_records
        .iter()
        .find(|n| n.node_qualified_path_text == "new")
        .unwrap();
    assert_eq!(edges[0].edge_source_reference, source.node_target_reference);
    assert_eq!(
        edges[0].edge_resolution_value,
        HldEdgeResolutionValue::Extracted {
            target: target.node_target_reference.clone()
        }
    );
    let start = SOURCE.find("Buffer::new()").unwrap() as u64;
    assert_eq!(
        edges[0].edge_evidence_row_key.evidence_site_occurrence_key,
        make_call_fixture_occurrence(start, start + 13, "Buffer::new()")
    );
    assert_eq!(
        edges[0].edge_relation_kind_value,
        HldRelationKindValue::Calls
    );
    let members = read_call_fixture_values(&store, "impl-header-index-table");
    assert!(
        members
            .iter()
            .flat_map(|r| r["impl_member_occurrence_rows"].as_array().unwrap())
            .any(|m| m["member_semantic_occurrence_key"]
                == encode_occurrence_identity_value(match &source.node_target_reference {
                    HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key)) => key,
                    _ => unreachable!(),
                }))
    );
}
/// REQ-RUST-034.0
#[test]
fn test_preserve_crlf_call_evidence() {
    let text = SOURCE.replace("impl Default", "\r\nimpl Default");
    let (store, input) = make_call_fixture_graph(&text);
    let edges = read_call_fixture_edges(&store, &input);
    assert_eq!(edges.len(), 1);
    let start = text.find("Buffer::new()").unwrap() as u64;
    assert_eq!(
        edges[0].edge_evidence_row_key.evidence_site_occurrence_key,
        make_call_fixture_occurrence(start, start + 13, "Buffer::new()")
    );
}
/// REQ-RUST-036.0
#[test]
fn test_reject_ambiguous_inherent_members() {
    let text = format!("{SOURCE} impl Buffer {{ fn new() {{}} }}");
    let (store, input) = make_call_fixture_graph(&text);
    assert_eq!(
        read_call_fixture_edges(&store, &input)[0].edge_resolution_value,
        HldEdgeResolutionValue::Unresolved {
            reason: HldUnresolvedReasonKind::AmbiguousTarget
        }
    );
}
/// REQ-RUST-036.0
#[test]
fn test_reject_generic_inherent_members() {
    let text = SOURCE.replace("impl Buffer", "impl<T> Buffer");
    let (store, input) = make_call_fixture_graph(&text);
    assert_eq!(
        read_call_fixture_edges(&store, &input)[0].edge_resolution_value,
        HldEdgeResolutionValue::Unresolved {
            reason: HldUnresolvedReasonKind::TraitSolverNeeded
        }
    );
}
/// REQ-RUST-036.0
#[test]
fn test_preserve_missing_member_reason() {
    let text = SOURCE.replace("fn new()", "fn different()");
    let (store, input) = make_call_fixture_graph(&text);
    assert_eq!(
        read_call_fixture_edges(&store, &input)[0].edge_resolution_value,
        HldEdgeResolutionValue::Unresolved {
            reason: HldUnresolvedReasonKind::MissingTarget
        }
    );
}
/// REQ-RUST-036.0
#[test]
fn test_reject_trait_dispatch_guessing() {
    let text = SOURCE.replace("impl Buffer", "impl Factory for Buffer");
    let (store, input) = make_call_fixture_graph(&text);
    assert_eq!(
        read_call_fixture_edges(&store, &input)[0].edge_resolution_value,
        HldEdgeResolutionValue::Unresolved {
            reason: HldUnresolvedReasonKind::TraitSolverNeeded
        }
    );
}
/// REQ-RUST-032.0 / REQ-RUST-045.0
#[test]
fn test_preserve_absent_target_reason() {
    let (store, mut input) = make_call_fixture_graph(SOURCE);
    input
        .input_node_records
        .retain(|n| n.node_qualified_path_text != "new");
    assert_eq!(
        read_call_fixture_edges(&store, &input)[0].edge_resolution_value,
        HldEdgeResolutionValue::Unresolved {
            reason: HldUnresolvedReasonKind::MissingTarget
        }
    );
}
/// REQ-RUST-032.0 / REQ-RUST-043.0
#[test]
fn test_exclude_absent_source_edges() {
    let (store, mut input) = make_call_fixture_graph(SOURCE);
    input
        .input_node_records
        .retain(|n| n.node_qualified_path_text != "default");
    assert!(read_call_fixture_edges(&store, &input).is_empty());
    assert!(!read_call_fixture_values(&store, "body-site-reference-table").is_empty());
}
/// REQ-RUST-034.0
#[test]
fn test_reject_shadowed_local_type() {
    let text = SOURCE.replace("Buffer::new()", "struct Buffer; Buffer::new()");
    let (store, input) = make_call_fixture_graph(&text);
    // Deliberately stale outer-type path fact must not override lexical evidence.
    assert!(matches!(
        read_call_fixture_edges(&store, &input)[0].edge_resolution_value,
        HldEdgeResolutionValue::Unresolved { .. }
    ));
}
/// REQ-RUST-034.0
#[test]
fn test_preserve_unsupported_call_reason() {
    let text = SOURCE.replace("Buffer::new()", "<Buffer as Factory>::new()");
    let (store, input) = make_call_fixture_graph(&text);
    assert_eq!(
        read_call_fixture_edges(&store, &input)[0].edge_resolution_value,
        HldEdgeResolutionValue::Unresolved {
            reason: HldUnresolvedReasonKind::TraitSolverNeeded
        }
    );
}
/// REQ-RUST-045.0
#[test]
fn test_stop_cancelled_call_resolution() {
    let (store, input) = make_call_fixture_graph(SOURCE);
    let cancel = RunCancelFlagValue::default();
    cancel.request_stage_work_cancellation();
    assert!(matches!(
        resolve_original_call_edges(&store, &make_call_fixture_scope(), &input, &cancel),
        Err(OriginalCallResolutionError::Cancelled)
    ));
}

/// REQ-RUST-034.0
#[test]
fn test_capture_qualified_body_prefix() {
    let rows = extract_file_syntax_rows(&make_call_syntax_input(SOURCE))
        .unwrap()
        .stage_unit_output_rows;
    let call: Value = rows
        .iter()
        .filter(|r| r.stage_owned_table_name == "body-site-reference-table")
        .map(|r| serde_json::from_str::<Value>(&r.stage_row_payload_text).unwrap())
        .find(|r| r["site_syntax_kind_value"] == "CALL_EXPR")
        .unwrap();
    assert_eq!(
        call["call_syntax_evidence"]["member_declared_name_value"],
        "new"
    );
    assert_eq!(
        call["call_syntax_evidence"]["type_source_evidence"]["type_source_text"],
        "Buffer"
    );
    assert_eq!(
        call["call_syntax_evidence"]["call_lexical_shadowing_flag"],
        false
    );
}

/// REQ-RUST-034.0
#[test]
fn test_reject_shadowed_parameter_binding() {
    let text = SOURCE.replace("fn default()", "fn default(Buffer: usize)");
    let (store, input) = make_call_fixture_graph(&text);
    assert!(matches!(
        read_call_fixture_edges(&store, &input)[0].edge_resolution_value,
        HldEdgeResolutionValue::Unresolved { .. }
    ));
}

/// REQ-RUST-036.0
#[test]
fn test_reject_unproven_impl_predicates() {
    let text = SOURCE.replace("impl Buffer {", "impl Buffer where Buffer: SomeTrait {");
    let (store, input) = make_call_fixture_graph(&text);
    assert_eq!(
        read_call_fixture_edges(&store, &input)[0].edge_resolution_value,
        HldEdgeResolutionValue::Unresolved {
            reason: HldUnresolvedReasonKind::TraitSolverNeeded
        }
    );
}

/// REQ-RUST-034.0 / REQ-RUST-036.0
#[test]
fn test_preserve_ambiguous_prefix_reason() {
    let (store, input) = make_call_fixture_graph(SOURCE);
    for row in store
        .rows
        .lock()
        .unwrap()
        .iter_mut()
        .filter(|r| r.stage_owned_table_name == "path-resolution-site-records")
    {
        let mut value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
        if value["site_source_range_start"] == SOURCE.find("Buffer::new()").unwrap() {
            value["path_resolution_state_kind"] = json!("ambiguous");
            value["path_unresolved_reason_kind"] = json!("trait_solver_needed");
            value["path_target_symbol_rows"] = json!([]);
            row.stage_row_payload_text = value.to_string();
        }
    }
    assert_eq!(
        read_call_fixture_edges(&store, &input)[0].edge_resolution_value,
        HldEdgeResolutionValue::Unresolved {
            reason: HldUnresolvedReasonKind::AmbiguousTarget
        }
    );
}

/// REQ-RUST-045.0
#[test]
fn test_preserve_supplied_frozen_snapshot() {
    let (store, input) = make_call_fixture_graph(SOURCE);
    store
        .frozen_read_only_flag
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let mut scope = make_call_fixture_scope();
    scope.stage_attempt_sequence_value = "attempt/frozen/7".into();
    let edges = resolve_original_call_edges(&store, &scope, &input, &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(edges.len(), 1);
    assert!(matches!(
        edges[0].edge_resolution_value,
        HldEdgeResolutionValue::Extracted { .. }
    ));
}

fn read_method_receiver_evidence(source: &str, text: &str) -> Value {
    extract_file_syntax_rows(&make_call_syntax_input(source))
        .unwrap()
        .stage_unit_output_rows
        .iter()
        .filter(|row| row.stage_owned_table_name == "body-site-reference-table")
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .find(|row| {
            row["site_syntax_kind_value"] == "METHOD_CALL_EXPR"
                && row["site_syntax_source_text"] == text
        })
        .unwrap()["call_syntax_evidence"]["method_receiver_evidence"]
        .clone()
}

/// REQ-RUST-042.0
#[test]
fn test_capture_generic_receiver_evidence() {
    let evidence = read_method_receiver_evidence(
        "trait T { fn go(self); }\nfn run<I: T>(i: I) { i.go() }\n",
        "i.go()",
    );
    assert_eq!(
        evidence,
        json!({
            "receiver_shape_kind":"parameter_identifier",
            "receiver_declared_name_value":"i",
            "method_declared_name_value":"go",
            "method_generic_arguments_flag":false,
            "receiver_parameter_type_text":"I",
            "receiver_generic_owner_kind":"function",
            "receiver_rebinding_flag":false,
        })
    );
}

/// REQ-RUST-042.0
#[test]
fn test_classify_other_receiver_shapes() {
    let source = "struct S { f: u8 }\nfn make() -> u8 { 0 }\nimpl S { fn run(self, n: u8) {\n self.go(); self.f.go(); make().go(); (n).go(); n.go(); } }\n";
    for text in ["self.go()", "self.f.go()", "make().go()", "(n).go()"] {
        let evidence = read_method_receiver_evidence(source, text);
        assert_eq!(evidence["receiver_shape_kind"], "other", "{text}");
        assert_eq!(
            evidence["receiver_generic_owner_kind"],
            Value::Null,
            "{text}"
        );
    }
    let concrete = read_method_receiver_evidence(source, "n.go()");
    assert_eq!(concrete["receiver_shape_kind"], "parameter_identifier");
    assert_eq!(concrete["receiver_parameter_type_text"], "u8");
    assert_eq!(concrete["receiver_generic_owner_kind"], Value::Null);
}

/// REQ-RUST-042.0
#[test]
fn test_flag_rebound_receiver_names() {
    for body in [
        "let i = 0u8; i.go()",
        "let f = |i: u8| i.go(); f(0)",
        "if let Some(i) = Some(0u8) { i.go() }",
        "rebind!(i); i.go()",
    ] {
        let source = format!("fn run<I: T>(i: I) {{ {body} }}\n");
        let evidence = read_method_receiver_evidence(&source, "i.go()");
        assert_eq!(evidence["receiver_rebinding_flag"], true, "{body}");
    }
    let later =
        read_method_receiver_evidence("fn run<I: T>(i: I, j: I) { i.go(); show!(j) }\n", "i.go()");
    assert_eq!(later["receiver_rebinding_flag"], false);
}

/// REQ-RUST-042.0
#[test]
fn test_locate_enclosing_generic_owner() {
    let member = read_method_receiver_evidence(
        "struct S<I>(I);\nimpl<I: T> S<I> { fn run(i: I) { i.go() } }\n",
        "i.go()",
    );
    assert_eq!(member["receiver_generic_owner_kind"], "enclosing_item");
    let nested = read_method_receiver_evidence(
        "struct S<I>(I);\nimpl<I: T> S<I> { fn run() { fn inner(i: I) { i.go() } } }\n",
        "i.go()",
    );
    assert_eq!(nested["receiver_generic_owner_kind"], Value::Null);
}

/// REQ-RUST-042.0
#[test]
fn test_capture_type_bound_evidence() {
    let source = "trait A: B + ?Sized {}\nfn run<I: A + 'static>(i: I) where I: m::C<u8> {}\n";
    let bounds = extract_file_syntax_rows(&make_call_syntax_input(source))
        .unwrap()
        .stage_unit_output_rows
        .iter()
        .filter(|row| row.stage_owned_table_name == "field-signature-detail-table")
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .filter(|row| row["site_syntax_kind_value"] == "TYPE_BOUND")
        .map(|row| {
            let bound = &row["bound_syntax_evidence"];
            (
                row["site_syntax_source_text"].as_str().unwrap().to_owned(),
                bound["bound_subject_kind"].as_str().unwrap().to_owned(),
                bound["bound_subject_name_value"]
                    .as_str()
                    .map(str::to_owned),
                bound["bound_form_kind"].as_str().unwrap().to_owned(),
                bound["trait_type_source"]["type_simple_path_flag"].as_bool(),
            )
        })
        .collect::<Vec<_>>();
    let row = |text: &str, subject: &str, name: Option<&str>, form: &str, simple| {
        (
            text.to_owned(),
            subject.to_owned(),
            name.map(str::to_owned),
            form.to_owned(),
            simple,
        )
    };
    assert_eq!(
        bounds,
        vec![
            row("B", "supertrait", None, "trait", Some(true)),
            row("?Sized", "supertrait", None, "other", None),
            row("A", "type_parameter", Some("I"), "trait", Some(true)),
            row("'static", "type_parameter", Some("I"), "lifetime", None),
            row(
                "m::C<u8>",
                "where_predicate",
                Some("I"),
                "trait",
                Some(false)
            ),
        ]
    );
}

/// A fixture graph for bound-method calls: every active `fn` and `trait` is a
/// node, every trait contains the functions written inside it, and a bound's
/// trait path resolves to the trait of that name. A name with no trait in the
/// source stands for a trait whose source is absent.
fn make_bound_fixture_graph(
    source: &str,
) -> (OriginalCallFixtureStore, HldMaterializationInputRecord) {
    let rows = extract_file_syntax_rows(&make_call_syntax_input(source))
        .unwrap()
        .stage_unit_output_rows;
    let store = OriginalCallFixtureStore {
        rows: Mutex::new(rows),
        frozen_read_only_flag: std::sync::atomic::AtomicBool::new(false),
    };
    let unit = encode_compilation_unit_value(&make_call_fixture_unit());
    let range = |span: &Value| {
        (
            span["span_byte_start_offset"].as_u64().unwrap(),
            span["span_byte_end_offset"].as_u64().unwrap(),
        )
    };
    let identify = |span: &Value| -> HldTargetIdentityValue {
        let (start, end) = range(span);
        NameSymbolTargetKey::Definition(make_call_fixture_occurrence(
            start,
            end,
            span["span_source_code_text"].as_str().unwrap(),
        ))
        .into()
    };
    let items = read_call_fixture_values(&store, "span-reference-text-table")
        .into_iter()
        .filter(|span| {
            span["span_configuration_active_flag"] == true
                && matches!(
                    span["span_syntax_kind_value"].as_str(),
                    Some("FN" | "TRAIT")
                )
        })
        .collect::<Vec<_>>();
    let traits = items
        .iter()
        .filter(|span| span["span_syntax_kind_value"] == "TRAIT")
        .collect::<Vec<_>>();
    let metadata = |span: &Value| json!({"target_symbol_kind_value":"Trait", "target_source_file_path":"/fixture/src/lib.rs", "target_source_range_start":span["span_byte_start_offset"], "target_source_range_end":span["span_byte_end_offset"], "target_compilation_scope_record":{"unit_semantic_identity_record":unit}});
    let mut extra = vec![
        make_call_fixture_row(
            "crate-dependency-graph-table",
            json!({"unit_compilation_identity_key":"raw-unit", "unit_package_root_path":"/fixture"}),
        ),
        make_call_fixture_row(
            "syntax-namespace-input-scopes",
            json!({"scope_cargo_unit_key":"raw-unit", "scope_module_identity_key":{"scope_semantic_path_parts":[], "scope_compilation_unit_key":unit}}),
        ),
    ];
    for found in &traits {
        let HldTargetIdentityValue::Symbol(symbol) = identify(found) else {
            unreachable!()
        };
        extra.push(make_call_fixture_row(
            "syntax-namespace-input-items",
            json!({"binding_target_identity_key":encode_symbol_target_value(&symbol), "binding_target_metadata_value":metadata(found)}),
        ));
    }
    for bound in read_call_fixture_values(&store, "field-signature-detail-table") {
        let path = &bound["bound_syntax_evidence"]["trait_type_source"];
        let Some(name) = path["type_source_text"].as_str() else {
            continue;
        };
        let mut row = json!({
            "site_compilation_scope_record":{"unit_semantic_identity_record":unit},
            "site_source_absolute_path":"/fixture/src/lib.rs",
            "site_source_range_start":path["span_byte_start_offset"],
            "site_source_range_end":path["span_byte_end_offset"],
            "site_expansion_provenance_chain":[]
        });
        match traits
            .iter()
            .find(|found| found["span_declared_name_value"] == name)
        {
            Some(found) => {
                row["path_resolution_state_kind"] = json!("resolved");
                row["path_target_symbol_rows"] = json!([metadata(found)]);
            }
            None => {
                row["path_resolution_state_kind"] = json!("unresolved");
                row["path_unresolved_reason_kind"] = json!("external_source_missing");
                row["path_target_symbol_rows"] = json!([]);
            }
        }
        extra.push(make_call_fixture_row("path-resolution-site-records", row));
    }
    store.rows.lock().unwrap().extend(extra);
    let generation = RunGenerationDigestValue([1; 32]);
    let nodes = items
        .iter()
        .map(|span| HldNodeInputRecord {
            run_generation_digest_value: generation,
            node_target_reference: identify(span),
            node_input_category: HldNodeInputCategory::LocalItem,
            node_item_kind_value: Some(if span["span_syntax_kind_value"] == "FN" {
                HldItemKindValue::Function
            } else {
                HldItemKindValue::Trait
            }),
            node_owner_kind_value: None,
            node_function_body_ancestry_flag: false,
            node_effective_visibility_kind: Some(HldEffectiveVisibilityKind::PrivateEquivalent),
            node_reachable_outside_crate_flag: Some(false),
            node_source_relative_path: Some("src/lib.rs".into()),
            node_qualified_path_text: span["span_declared_name_value"].as_str().unwrap().into(),
            node_signature_text: None,
        })
        .collect();
    let edges = traits
        .iter()
        .flat_map(|owner| {
            let (start, end) = range(owner);
            let identify = &identify;
            items
                .iter()
                .filter(move |item| {
                    let (inner_start, inner_end) = range(item);
                    item["span_syntax_kind_value"] == "FN"
                        && start < inner_start
                        && inner_end <= end
                })
                .map(move |item| {
                    let (inner_start, inner_end) = range(item);
                    HldEdgeInputRecord {
                        run_generation_digest_value: generation,
                        edge_source_reference: identify(owner),
                        edge_relation_kind_value: HldRelationKindValue::Contains,
                        edge_resolution_value: HldEdgeResolutionValue::Extracted {
                            target: identify(item),
                        },
                        edge_evidence_row_key: HldEvidenceRowKey {
                            evidence_site_occurrence_key: make_call_fixture_occurrence(
                                inner_start,
                                inner_end,
                                item["span_source_code_text"].as_str().unwrap(),
                            ),
                            evidence_source_occurrence_key: identify(owner),
                            evidence_relation_kind_value: HldRelationKindValue::Contains,
                            evidence_target_reference_value: Some(identify(item)),
                        },
                        edge_dispatch_metadata_text: None,
                    }
                })
        })
        .collect();
    (
        store,
        HldMaterializationInputRecord {
            run_generation_digest_value: generation,
            stage_attempt_sequence_value: "attempt".into(),
            semantic_input_completeness_kind: HldInputCompletenessKind::IncompleteTypedInput,
            source_coverage_kind: HldSourceCoverageKind::FullSemantic,
            input_unit_provenance_records: vec![HldUnitProvenanceRecord {
                unit_compilation_identity_key: make_call_fixture_unit(),
                unit_origin_kind: HldUnitOriginKind::Workspace,
            }],
            input_node_records: nodes,
            input_edge_records: edges,
        },
    )
}

/// The one call edge of a bound fixture as (target node name or unresolved
/// reason, dispatch record).
fn read_bound_call_result(source: &str) -> (Result<String, HldUnresolvedReasonKind>, Value) {
    let (store, input) = make_bound_fixture_graph(source);
    let edges = read_call_fixture_edges(&store, &input);
    assert_eq!(edges.len(), 1, "one call site in the fixture");
    let outcome = match &edges[0].edge_resolution_value {
        HldEdgeResolutionValue::Extracted { target } => Ok(input
            .input_node_records
            .iter()
            .find(|node| node.node_target_reference == *target)
            .unwrap()
            .node_qualified_path_text
            .clone()),
        HldEdgeResolutionValue::Unresolved { reason } => Err(*reason),
        other => panic!("unexpected resolution {other:?}"),
    };
    let dispatch =
        serde_json::from_str(edges[0].edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
    (outcome, dispatch)
}

const BOUND_SOURCE: &str = "trait Sealed { fn write(self); } trait Integer: Sealed {} fn run<I: Integer>(i: I) { i.write(); }";

/// REQ-RUST-042.0
#[test]
fn test_resolve_bound_supertrait_method() {
    for source in [
        BOUND_SOURCE.to_owned(),
        BOUND_SOURCE.replace("<I: Integer>(i: I)", "<I>(i: I) where I: Integer"),
    ] {
        let (outcome, dispatch) = read_bound_call_result(&source);
        assert_eq!(outcome, Ok("write".to_owned()), "{source}");
        assert_eq!(dispatch["dispatch_kind"], "generic_bound_trait_method");
        assert_eq!(
            dispatch["bound_dispatch_evidence"],
            json!({"bound_chain":["I", "Integer", "Sealed"],"missing_source_traits":[]})
        );
    }
}

/// REQ-RUST-042.0
#[test]
fn test_bound_method_ambiguous_candidates() {
    let source = "trait A { fn write(self); } trait B { fn write(self); } fn run<I: A + B>(i: I) { i.write(); }";
    assert_eq!(
        read_bound_call_result(source).0,
        Err(HldUnresolvedReasonKind::AmbiguousTarget)
    );
}

/// REQ-RUST-042.0
#[test]
fn test_bound_method_name_distractor() {
    let source = format!(
        "trait Other {{ fn write(self); }} fn write() {{}} {BOUND_SOURCE} trait Late {{ fn write(self); }}"
    );
    let (store, input) = make_bound_fixture_graph(&source);
    let edges = read_call_fixture_edges(&store, &input);
    let HldEdgeResolutionValue::Extracted { target } = &edges[0].edge_resolution_value else {
        panic!("expected an edge, got {:?}", edges[0].edge_resolution_value);
    };
    // The target is the `write` that Sealed contains, not another one of that name.
    let sealed = input
        .input_node_records
        .iter()
        .find(|node| node.node_qualified_path_text == "Sealed")
        .unwrap();
    assert!(input.input_edge_records.iter().any(|edge| {
        edge.edge_source_reference == sealed.node_target_reference
            && edge.edge_resolution_value
                == HldEdgeResolutionValue::Extracted {
                    target: target.clone(),
                }
    }));
}

/// REQ-RUST-042.0
#[test]
fn test_local_receiver_shadow_unresolved() {
    let source = BOUND_SOURCE.replace("{ i.write(); }", "{ let i = 1u8; i.write(); }");
    let (outcome, dispatch) = read_bound_call_result(&source);
    assert_eq!(outcome, Err(HldUnresolvedReasonKind::TraitSolverNeeded));
    assert!(dispatch["bound_dispatch_evidence"].is_null());
}

/// REQ-RUST-042.0
#[test]
fn test_other_receiver_shapes_unresolved() {
    for source in [
        BOUND_SOURCE.replace("(i: I) { i.write(); }", "(i: (I,)) { i.0.write(); }"),
        BOUND_SOURCE.replace("(i: I) { i.write(); }", "(i: &I) { i.write(); }"),
        BOUND_SOURCE.replace("(i: I) { i.write(); }", "(i: I) { (i as I).write(); }"),
        BOUND_SOURCE.replace(
            "fn run<I: Integer>(i: I) { i.write(); }",
            "trait Host<I: Integer> { fn run(i: I) { i.write(); } }",
        ),
    ] {
        assert_eq!(
            read_bound_call_result(&source).0,
            Err(HldUnresolvedReasonKind::TraitSolverNeeded),
            "{source}"
        );
    }
}

/// REQ-RUST-042.0
#[test]
fn test_cyclic_supertrait_bounds_terminate() {
    let source = "trait A: B {} trait B: A { fn write(self); } fn run<I: A>(i: I) { i.write(); }";
    let (outcome, dispatch) = read_bound_call_result(source);
    assert_eq!(outcome, Ok("write".to_owned()));
    assert_eq!(
        dispatch["bound_dispatch_evidence"]["bound_chain"],
        json!(["I", "A", "B"]),
        "each trait is visited once"
    );
}

/// REQ-RUST-042.0
#[test]
fn test_missing_source_gap_recorded() {
    let source = BOUND_SOURCE.replace("trait Sealed {", "trait Sealed: Copy + ?Sized + 'static {");
    let (outcome, dispatch) = read_bound_call_result(&source);
    assert_eq!(outcome, Ok("write".to_owned()));
    assert_eq!(
        dispatch["bound_dispatch_evidence"],
        json!({"bound_chain":["I", "Integer", "Sealed"],"missing_source_traits":["Copy"]})
    );
}

/// REQ-RUST-042.0
#[test]
fn test_missing_source_only_unresolved() {
    let source = "fn run<I: Copy>(i: I) { i.write(); }";
    let (outcome, dispatch) = read_bound_call_result(source);
    assert_eq!(outcome, Err(HldUnresolvedReasonKind::ExternalSourceMissing));
    assert_eq!(
        dispatch["bound_dispatch_evidence"]["missing_source_traits"],
        json!(["Copy"])
    );
}

/// REQ-RUST-042.0
#[test]
fn test_bound_method_missing_target() {
    for source in [
        // No bound trait declares the method.
        BOUND_SOURCE.replace("fn write(self);", "fn other(self);"),
        // An associated function without `self` is not callable as a method.
        BOUND_SOURCE.replace("fn write(self);", "fn write();"),
        // No bound at all.
        BOUND_SOURCE.replace("<I: Integer>", "<I>"),
    ] {
        assert_eq!(
            read_bound_call_result(&source).0,
            Err(HldUnresolvedReasonKind::MissingTarget),
            "{source}"
        );
    }
}

/// REQ-RUST-042.0
#[test]
fn test_trait_rename_changes_identity() {
    // Nothing is keyed on itoa's names: renamed traits and method resolve the same way.
    let source = BOUND_SOURCE
        .replace("Sealed", "Hidden")
        .replace("Integer", "Number")
        .replace("write", "emit");
    let (outcome, dispatch) = read_bound_call_result(&source);
    assert_eq!(outcome, Ok("emit".to_owned()));
    assert_eq!(
        dispatch["bound_dispatch_evidence"]["bound_chain"],
        json!(["I", "Number", "Hidden"])
    );
}
