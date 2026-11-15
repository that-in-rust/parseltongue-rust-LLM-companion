#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::original_call_resolution_records::original_call_resolution_tests::{
    OriginalCallFixtureStore, make_call_fixture_graph, make_call_fixture_occurrence,
    make_call_fixture_row, make_call_fixture_scope, make_call_fixture_unit,
};
use crate::syntax_namespace_record_codec::{
    encode_compilation_unit_value, encode_symbol_target_value,
};
use serde_json::{Value, json};

/// A call fixture graph of `source` (which must declare `struct Buffer`) with a
/// node for each `static` and `const`, and one stored value-path resolution per
/// entry of `paths`: (path text, which occurrence of that text, declared name
/// it resolves to, symbol kind). An empty name is a path left unresolved, and
/// its kind is then the stored reason, `ambiguous`, or empty for no reason.
/// Name resolution outcomes are supplied here, at the stage seam; the resolver
/// under test only joins them.
fn make_read_fixture_graph(
    source: &str,
    paths: &[(&str, usize, &str, &str)],
) -> (OriginalCallFixtureStore, HldMaterializationInputRecord) {
    let (store, mut input) = make_call_fixture_graph(source);
    let unit = encode_compilation_unit_value(&make_call_fixture_unit());
    let spans = store
        .rows
        .lock()
        .unwrap()
        .iter()
        .filter(|row| row.stage_owned_table_name == "span-reference-text-table")
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .filter(|span| {
            matches!(
                span["span_syntax_kind_value"].as_str(),
                Some("STATIC" | "CONST" | "FN")
            )
        })
        .collect::<Vec<_>>();
    let metadata = |name: &str, kind: &str| {
        let span = spans
            .iter()
            .find(|span| span["span_declared_name_value"] == name)
            .unwrap();
        json!({"target_symbol_kind_value":kind,
            "target_source_file_path":"/fixture/src/lib.rs",
            "target_source_range_start":span["span_byte_start_offset"],
            "target_source_range_end":span["span_byte_end_offset"],
            "target_compilation_scope_record":{"unit_semantic_identity_record":unit}})
    };
    let occurrence = |span: &Value| {
        make_call_fixture_occurrence(
            span["span_byte_start_offset"].as_u64().unwrap(),
            span["span_byte_end_offset"].as_u64().unwrap(),
            span["span_source_code_text"].as_str().unwrap(),
        )
    };
    let mut extra = Vec::new();
    for span in &spans {
        let name = span["span_declared_name_value"].as_str().unwrap();
        let kind = match span["span_syntax_kind_value"].as_str().unwrap() {
            "STATIC" => "Static",
            "CONST" => "Const",
            _ => "Function",
        };
        let target = NameSymbolTargetKey::Definition(occurrence(span));
        extra.push(make_call_fixture_row(
            "syntax-namespace-input-items",
            json!({"binding_target_identity_key":encode_symbol_target_value(&target),
                "binding_target_metadata_value":metadata(name, kind)}),
        ));
        if kind != "Function" && !name.starts_with("LOCAL") {
            input.input_node_records.push(HldNodeInputRecord {
                node_target_reference: target.into(),
                node_item_kind_value: Some(HldItemKindValue::Static),
                node_owner_kind_value: None,
                node_qualified_path_text: name.into(),
                ..input.input_node_records[0].clone()
            });
        }
    }
    for (text, nth, name, kind) in paths {
        let start = source.match_indices(text).nth(*nth).unwrap().0;
        let targets = if name.is_empty() {
            json!([])
        } else if *kind == "ConstParam" {
            json!([{"target_symbol_kind_value":"ConstParam"}])
        } else {
            json!([metadata(name, kind)])
        };
        extra.push(make_call_fixture_row(
            "path-resolution-site-records",
            json!({"site_compilation_scope_record":{"unit_semantic_identity_record":unit},
                "site_source_absolute_path":"/fixture/src/lib.rs",
                "site_source_range_start":start, "site_source_range_end":start + text.len(),
                "site_expansion_provenance_chain":[],
                "site_path_context_kind":"body_value_path",
                "path_site_source_text":text,
                "path_resolution_state_kind":match (name.is_empty(), *kind) {
                    (false, _) => "resolved",
                    (true, "ambiguous") => "ambiguous",
                    (true, _) => "unresolved",
                },
                "path_unresolved_reason_kind":if name.is_empty() && !kind.is_empty() {
                    json!(kind)
                } else {
                    Value::Null
                },
                "path_target_symbol_rows":targets}),
        ));
    }
    store.rows.lock().unwrap().extend(extra);
    (store, input)
}

/// `reads` edges of a fixture as (source name, target name, "start:site text"),
/// in source order. An unresolved edge has `unresolved:<reason>` as its target.
fn read_body_fixture_edges(
    source: &str,
    store: &OriginalCallFixtureStore,
    input: &HldMaterializationInputRecord,
) -> Vec<(String, String, String)> {
    let run = || {
        resolve_body_read_edges(
            store,
            &make_call_fixture_scope(),
            input,
            &RunCancelFlagValue::default(),
        )
        .unwrap()
    };
    let edges = run();
    assert_eq!(edges, run(), "the same rows give the same edges");
    let name = |reference: &HldTargetIdentityValue| {
        input
            .input_node_records
            .iter()
            .find(|node| &node.node_target_reference == reference)
            .unwrap()
            .node_qualified_path_text
            .clone()
    };
    let mut rows = edges
        .read_edge_input_records
        .iter()
        .map(|edge| {
            assert_eq!(edge.edge_relation_kind_value, HldRelationKindValue::Reads);
            let target = match &edge.edge_resolution_value {
                HldEdgeResolutionValue::Extracted { target } => name(target),
                HldEdgeResolutionValue::Unresolved { reason } => format!("unresolved:{reason:?}"),
                other => panic!("neither extracted nor unresolved: {other:?}"),
            };
            let (start, end) = edge
                .edge_evidence_row_key
                .evidence_site_occurrence_key
                .read_semantic_occurrence_parts()
                .span_byte_range_value
                .read_source_range_pair();
            (
                start,
                (
                    name(&edge.edge_source_reference),
                    target,
                    format!("{start}:{}", &source[start as usize..end as usize]),
                ),
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows.into_iter().map(|(_, row)| row).collect()
}

fn edge(source: &str, target: &str, site: &str) -> (String, String, String) {
    (source.to_owned(), target.to_owned(), site.to_owned())
}

/// REQ-RUST-048.0 R1, R6
#[test]
fn test_static_read_row_edge() {
    let source = "struct Buffer; static TABLE: u8 = 1; const LIMIT: u8 = 2; \
        fn pick() -> u8 { TABLE + LIMIT + TABLE }";
    let (store, input) = make_read_fixture_graph(
        source,
        &[
            ("TABLE", 1, "TABLE", "Static"),
            ("LIMIT", 1, "LIMIT", "Const"),
            ("TABLE", 2, "TABLE", "Static"),
        ],
    );
    let site = |text: &str, nth: usize| {
        format!("{}:{text}", source.match_indices(text).nth(nth).unwrap().0)
    };
    assert_eq!(
        read_body_fixture_edges(source, &store, &input),
        vec![
            edge("pick", "TABLE", &site("TABLE", 1)),
            edge("pick", "LIMIT", &site("LIMIT", 1)),
            edge("pick", "TABLE", &site("TABLE", 2)),
        ],
        "one row per site, in source order, each with the path's own range"
    );
}

/// REQ-RUST-048.0 R2
#[test]
fn test_local_binding_no_read() {
    let source = "struct Buffer; static TABLE: u8 = 1; \
        fn pick<const WIDTH: usize>(seed: u8) -> u8 { const LOCAL_STEP: u8 = 3; \
        let held = seed; held + LOCAL_STEP + WIDTH as u8 }";
    let (store, input) = make_read_fixture_graph(
        source,
        &[
            ("LOCAL_STEP", 1, "LOCAL_STEP", "Const"),
            ("WIDTH", 1, "WIDTH", "ConstParam"),
        ],
    );
    assert_eq!(
        read_body_fixture_edges(source, &store, &input),
        vec![],
        "a parameter and a let have no request; a const declared in the body and a \
         const generic parameter resolve to something that is not a graph node"
    );
}

/// REQ-RUST-048.0 R3
#[test]
fn test_function_value_no_read() {
    let source = "struct Buffer; static TABLE: u8 = 1; fn helper() -> u8 { 4 } \
        fn pick() -> u8 { let call = helper; let unit = Buffer; call() + TABLE }";
    let (store, input) = make_read_fixture_graph(
        source,
        &[
            ("helper", 1, "helper", "Function"),
            ("TABLE", 1, "TABLE", "Static"),
        ],
    );
    assert_eq!(
        read_body_fixture_edges(source, &store, &input),
        vec![edge(
            "pick",
            "TABLE",
            &format!("{}:TABLE", source.rfind("TABLE").unwrap())
        )],
        "a path to a function or a unit struct is not a read"
    );
}

/// REQ-RUST-048.0 R7
#[test]
fn test_expanded_body_read_counted() {
    let source = "struct Buffer; static TABLE: u8 = 1; fn pick() -> u8 { TABLE }";
    let (store, input) = make_read_fixture_graph(source, &[("TABLE", 1, "TABLE", "Static")]);
    store.rows.lock().unwrap().iter_mut().for_each(|row| {
        let mut value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
        let chain = match row.stage_owned_table_name.as_str() {
            "body-site-reference-table" => "span_expansion_provenance_chain",
            "path-resolution-site-records" => "site_expansion_provenance_chain",
            _ => return,
        };
        value[chain] = json!(["ab".repeat(32)]);
        row.stage_row_payload_text = value.to_string();
    });
    let edges = resolve_body_read_edges(
        &store,
        &make_call_fixture_scope(),
        &input,
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    assert_eq!(edges.read_edge_input_records, vec![]);
    assert_eq!(
        edges.read_expanded_site_count, 1,
        "a value path in a macro-generated body is counted and left to part 5"
    );
}

/// REQ-RUST-048.0
#[test]
fn test_reject_read_edge_scope() {
    let source = "struct Buffer; static TABLE: u8 = 1; fn pick() -> u8 { TABLE }";
    let (store, mut input) = make_read_fixture_graph(source, &[]);
    let cancel = RunCancelFlagValue::default();
    cancel.request_stage_work_cancellation();
    assert!(matches!(
        resolve_body_read_edges(&store, &make_call_fixture_scope(), &input, &cancel),
        Err(BodyReadEdgeError::Cancelled)
    ));
    input.source_coverage_kind = HldSourceCoverageKind::OriginalTopLevelContainment;
    assert!(matches!(
        resolve_body_read_edges(
            &store,
            &make_call_fixture_scope(),
            &input,
            &RunCancelFlagValue::default()
        ),
        Err(BodyReadEdgeError::InvalidScope)
    ));
}

/// REQ-RUST-048.0 R5
#[test]
fn test_unresolved_read_keeps_reason() {
    let source = "struct Buffer; static TABLE: u8 = 1; \
        fn pick() -> u8 { MISSING + OUTSIDE + TWICE + TABLE }";
    let (store, input) = make_read_fixture_graph(
        source,
        &[
            ("MISSING", 0, "", ""),
            ("OUTSIDE", 0, "", "external_source_missing"),
            ("TWICE", 0, "", "ambiguous"),
            ("TABLE", 1, "TABLE", "Static"),
        ],
    );
    let site = |name: &str| format!("{}:{name}", source.rfind(name).unwrap());
    assert_eq!(
        read_body_fixture_edges(source, &store, &input),
        vec![
            edge("pick", "unresolved:TraitSolverNeeded", &site("MISSING")),
            edge("pick", "unresolved:ExternalSourceMissing", &site("OUTSIDE")),
            edge("pick", "unresolved:AmbiguousTarget", &site("TWICE")),
            edge("pick", "TABLE", &site("TABLE")),
        ],
        "a path that names no one value keeps one row and one reason"
    );
}
