#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::extract_file_syntax_rows;
use crate::original_call_resolution_records::original_call_resolution_tests::{
    OriginalCallFixtureStore, make_call_fixture_graph, make_call_fixture_row,
    make_call_fixture_scope, make_call_fixture_unit, make_call_syntax_input,
};
use crate::syntax_namespace_record_codec::encode_compilation_unit_value;
use serde_json::{Value, json};

/// Every signature row of `source` as (row kind, row text, listed type paths).
/// A row that carries no list at all reports `None`.
fn read_signature_path_lists(source: &str) -> Vec<(String, String, Option<Vec<String>>)> {
    extract_file_syntax_rows(&make_call_syntax_input(source))
        .unwrap()
        .stage_unit_output_rows
        .iter()
        .filter(|row| row.stage_owned_table_name == "field-signature-detail-table")
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .filter(|row| {
            !matches!(
                row["site_syntax_kind_value"].as_str(),
                Some("ATTR" | "IMPL")
            )
        })
        .map(|row| {
            let paths = row["type_path_source_rows"].as_array().map(|paths| {
                paths
                    .iter()
                    .map(|path| {
                        let start = path["span_byte_start_offset"].as_u64().unwrap() as usize;
                        let end = path["span_byte_end_offset"].as_u64().unwrap() as usize;
                        assert_eq!(path["type_source_text"], source[start..end]);
                        assert_eq!(
                            path["type_path_self_flag"],
                            &source[start..end] == "Self",
                            "only a bare Self is flagged"
                        );
                        source[start..end].to_owned()
                    })
                    .collect()
            });
            (
                row["site_syntax_kind_value"].as_str().unwrap().to_owned(),
                row["site_syntax_source_text"].as_str().unwrap().to_owned(),
                paths,
            )
        })
        .collect()
}

fn row(kind: &str, text: &str, paths: &[&str]) -> (String, String, Option<Vec<String>>) {
    (
        kind.to_owned(),
        text.to_owned(),
        Some(paths.iter().map(|path| (*path).to_owned()).collect()),
    )
}

/// REQ-RUST-018.2
#[test]
fn test_signature_paths_listed_once() {
    let source = "struct S { a: m::A<B>, b: &'static [C] }\nstruct T(D);\n\
        impl S { fn run<I: E>(&self, x: Self, y: (F, F)) -> Option<Self::G> where I: H { loop {} } }\n";
    assert_eq!(
        read_signature_path_lists(source),
        vec![
            row("RECORD_FIELD", "a: m::A<B>", &["m::A<B>", "B"]),
            row("RECORD_FIELD", "b: &'static [C]", &["C"]),
            row("TUPLE_FIELD", "D", &["D"]),
            row("TYPE_PARAM", "I: E", &[]),
            row("TYPE_BOUND", "E", &["E"]),
            row("SELF_PARAM", "&self", &[]),
            row("PARAM", "x: Self", &["Self"]),
            row("PARAM", "y: (F, F)", &["F", "F"]),
            row(
                "RET_TYPE",
                "-> Option<Self::G>",
                &["Option<Self::G>", "Self::G"]
            ),
            row("WHERE_PRED", "I: H", &["I"]),
            row("TYPE_BOUND", "H", &["H"]),
        ],
        "each type path is listed once, on the nearest signature row that holds it"
    );
}

/// REQ-RUST-018.2
#[test]
fn test_expression_paths_not_types() {
    let source = "struct S { a: [u8; N::LEN], b: W<{ LEN }>, c: [u8; size::<X>()] }\n";
    assert_eq!(
        read_signature_path_lists(source),
        vec![
            row("RECORD_FIELD", "a: [u8; N::LEN]", &["u8"]),
            row("RECORD_FIELD", "b: W<{ LEN }>", &["W<{ LEN }>"]),
            row("RECORD_FIELD", "c: [u8; size::<X>()]", &["u8"]),
        ],
        "a path under an expression is not a signature type path"
    );
}

/// REQ-RUST-018.2
#[test]
fn test_body_position_rows_unlisted() {
    let source = "fn run(a: A) { let f = |b: B| -> C { loop {} }; }\n";
    let rows = read_signature_path_lists(source);
    assert_eq!(rows[0], row("PARAM", "a: A", &["A"]));
    assert!(
        rows[1..].iter().all(|(_, _, paths)| paths.is_none()),
        "closure parameters and return types are body sites: {rows:?}"
    );
    assert_eq!(rows.len(), 3);
}

/// `uses_type` edges of a fixture as (source name, resolution, site text).
fn read_type_fixture_edges(
    source: &str,
    store: &OriginalCallFixtureStore,
    input: &HldMaterializationInputRecord,
) -> Vec<(String, HldEdgeResolutionValue, String)> {
    let run = || {
        resolve_signature_type_edges(
            store,
            &make_call_fixture_scope(),
            input,
            &RunCancelFlagValue::default(),
        )
        .unwrap()
    };
    let edges = run();
    assert_eq!(edges, run(), "the same rows give the same edges");
    edges
        .iter()
        .map(|edge| {
            assert_eq!(
                edge.edge_relation_kind_value,
                HldRelationKindValue::UsesType
            );
            let owner = input
                .input_node_records
                .iter()
                .find(|node| node.node_target_reference == edge.edge_source_reference)
                .unwrap();
            let (start, end) = edge
                .edge_evidence_row_key
                .evidence_site_occurrence_key
                .read_semantic_occurrence_parts()
                .span_byte_range_value
                .read_source_range_pair();
            (
                owner.node_qualified_path_text.clone(),
                edge.edge_resolution_value.clone(),
                format!("{start}:{}", &source[start as usize..end as usize]),
            )
        })
        .collect()
}

fn read_fixture_node_target(
    input: &HldMaterializationInputRecord,
    name: &str,
) -> HldEdgeResolutionValue {
    HldEdgeResolutionValue::Extracted {
        target: input
            .input_node_records
            .iter()
            .find(|node| node.node_qualified_path_text == name)
            .unwrap()
            .node_target_reference
            .clone(),
    }
}

/// REQ-RUST-018.2
#[test]
fn test_repeated_type_two_edges() {
    let source = "struct Buffer; fn make(a: Buffer, b: &[Buffer]) -> Buffer { loop {} }";
    let (store, input) = make_call_fixture_graph(source);
    let buffer = read_fixture_node_target(&input, "Buffer");
    let mut edges = read_type_fixture_edges(source, &store, &input);
    edges.sort_by_key(|(_, _, site)| site.clone());
    assert_eq!(
        edges,
        vec![
            ("make".to_owned(), buffer.clone(), "26:Buffer".to_owned()),
            ("make".to_owned(), buffer.clone(), "39:Buffer".to_owned()),
            ("make".to_owned(), buffer, "51:Buffer".to_owned()),
        ],
        "one edge per site, each with the path's own range"
    );
}

/// REQ-RUST-018.2
#[test]
fn test_type_parameter_no_edge() {
    let source = "struct Buffer; fn make<T>(t: T, b: Buffer) { loop {} }";
    let (store, input) = make_call_fixture_graph(source);
    let start = source.find("t: T").unwrap() + 3;
    store.rows.lock().unwrap().push(make_call_fixture_row(
        "path-resolution-site-records",
        json!({
            "site_compilation_scope_record":{"unit_semantic_identity_record":
                encode_compilation_unit_value(&make_call_fixture_unit())},
            "site_source_absolute_path":"/fixture/src/lib.rs",
            "site_source_range_start":start, "site_source_range_end":start + 1,
            "site_expansion_provenance_chain":[], "path_resolution_state_kind":"resolved",
            "path_target_symbol_rows":[{"target_symbol_kind_value":"TypeParam",
                "target_source_file_path":"/fixture/src/lib.rs"}]
        }),
    ));
    let edges = read_type_fixture_edges(source, &store, &input);
    assert_eq!(
        edges,
        vec![(
            "make".to_owned(),
            read_fixture_node_target(&input, "Buffer"),
            "35:Buffer".to_owned()
        )],
        "a type parameter names no item"
    );
}

/// REQ-RUST-018.2
#[test]
fn test_body_and_macro_rows_skipped() {
    let source = "struct Buffer; fn outer() { fn inner(a: Buffer) {} }";
    let (store, mut input) = make_call_fixture_graph(source);
    assert_eq!(read_type_fixture_edges(source, &store, &input).len(), 1);
    input
        .input_node_records
        .iter_mut()
        .filter(|node| node.node_qualified_path_text == "inner")
        .for_each(|node| node.node_function_body_ancestry_flag = true);
    assert_eq!(
        read_type_fixture_edges(source, &store, &input),
        vec![],
        "an owner inside a function body is not a source"
    );

    let source = "struct Buffer; fn make(a: Buffer) {}";
    let (store, input) = make_call_fixture_graph(source);
    assert_eq!(read_type_fixture_edges(source, &store, &input).len(), 1);
    store.rows.lock().unwrap().iter_mut().for_each(|row| {
        if row.stage_owned_table_name == "field-signature-detail-table" {
            let mut value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            value["span_expansion_provenance_chain"] = json!(["ab".repeat(32)]);
            row.stage_row_payload_text = value.to_string();
        }
    });
    assert_eq!(
        read_type_fixture_edges(source, &store, &input),
        vec![],
        "a row from a macro expansion is left to part 5"
    );
}

/// REQ-RUST-018.2
#[test]
fn test_reject_type_edge_scope() {
    let (store, mut input) = make_call_fixture_graph("struct Buffer; fn make(a: Buffer) {}");
    let cancel = RunCancelFlagValue::default();
    cancel.request_stage_work_cancellation();
    assert!(matches!(
        resolve_signature_type_edges(&store, &make_call_fixture_scope(), &input, &cancel),
        Err(SignatureTypeEdgeError::Cancelled)
    ));
    input.source_coverage_kind = HldSourceCoverageKind::OriginalTopLevelContainment;
    assert!(matches!(
        resolve_signature_type_edges(
            &store,
            &make_call_fixture_scope(),
            &input,
            &RunCancelFlagValue::default()
        ),
        Err(SignatureTypeEdgeError::InvalidScope)
    ));
}

/// REQ-RUST-018.2
#[test]
fn test_impl_self_type_edge() {
    let source = "struct Buffer; trait Make { fn make() -> Self; } \
        impl Make for Buffer { fn make() -> Self { loop {} } } \
        impl Buffer { fn pair(self, other: &Self) -> Option<Self> { loop {} } }";
    let (store, input) = make_call_fixture_graph(source);
    let buffer = read_fixture_node_target(&input, "Buffer");
    let site = |nth: usize| {
        let start = source.match_indices("Self").nth(nth).unwrap().0;
        format!("{start}:Self")
    };
    let mut edges = read_type_fixture_edges(source, &store, &input);
    edges.sort_by_key(|(_, _, site)| site.split(':').next().unwrap().parse::<usize>().unwrap());
    assert_eq!(
        edges,
        vec![
            ("make".to_owned(), buffer.clone(), site(1)),
            ("pair".to_owned(), buffer.clone(), site(2)),
            ("pair".to_owned(), buffer, site(3)),
        ],
        "Self in an impl is the impl's self type; Self in a trait declaration names no item"
    );
}
