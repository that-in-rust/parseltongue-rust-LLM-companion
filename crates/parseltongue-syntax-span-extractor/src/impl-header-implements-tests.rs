#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::original_call_resolution_records::original_call_resolution_tests::{
    OriginalCallFixtureStore, make_call_fixture_graph, make_call_fixture_row,
    make_call_fixture_scope, make_call_fixture_unit,
};
use crate::syntax_namespace_record_codec::encode_symbol_target_value;
use serde_json::{Value, json};

const BUILTIN_NAMES: [&str; 2] = ["u8", "u16"];

/// A fixture graph of `source` with one extra node for `trait Make`. The
/// fixture resolves only `Buffer`, so each stored impl header is completed
/// here: a `Make` trait path names that node and a builtin self type names
/// its builtin. `patch` then rewrites each header row.
fn make_header_fixture(
    source: &str,
    patch: impl Fn(&mut Value),
) -> (OriginalCallFixtureStore, HldMaterializationInputRecord) {
    let (store, mut input) = make_call_fixture_graph(source);
    let start = source.find("trait Make").unwrap() as u64;
    let make = NameSymbolTargetKey::Definition(
        SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
            &make_call_fixture_unit(),
            "src/lib.rs",
            SpanByteRangeValue::try_from((start, start + 10)).unwrap(),
            SpanDigestIdValue::from([7; 32]),
            &[],
        )
        .unwrap(),
    );
    let mut node = input.input_node_records[0].clone();
    node.node_target_reference = make.clone().into();
    node.node_item_kind_value = Some(HldItemKindValue::Trait);
    node.node_qualified_path_text = "Make".into();
    input.input_node_records.push(node);
    let scope = make_call_fixture_scope();
    let headers = read_impl_stage_values(&store, &scope, "impl-header-index-table").unwrap();
    let mut rows = store.rows.lock().unwrap();
    rows.retain(|row| row.stage_owned_table_name != "impl-header-index-table");
    rows.extend(headers.into_iter().map(|mut header| {
        if header["impl_trait_type_resolution"]["type_source_evidence"]["type_source_text"]
            == "Make"
        {
            header["impl_trait_type_resolution"]["resolution_state_kind"] = json!("resolved");
            header["impl_trait_type_resolution"]["unresolved_reason_kind"] = Value::Null;
            header["impl_trait_type_resolution"]["target_identity"] =
                encode_symbol_target_value(&make);
        }
        let name = header["impl_self_type_resolution"]["type_source_evidence"]["type_source_text"]
            .as_str()
            .filter(|name| BUILTIN_NAMES.contains(name))
            .map(str::to_owned);
        if let Some(name) = name {
            header["impl_self_type_resolution"]["resolution_state_kind"] = json!("resolved");
            header["impl_self_type_resolution"]["unresolved_reason_kind"] = Value::Null;
            header["impl_self_type_resolution"]["target_identity"] =
                encode_symbol_target_value(&NameSymbolTargetKey::Builtin(name));
        }
        patch(&mut header);
        make_call_fixture_row("impl-header-index-table", header)
    }));
    drop(rows);
    (store, input)
}

/// `implements` rows as (source name, resolution, site text), read twice.
fn read_header_fixture_edges(
    source: &str,
    store: &OriginalCallFixtureStore,
    input: &HldMaterializationInputRecord,
) -> (Vec<(String, HldEdgeResolutionValue, String)>, usize) {
    let run = || {
        resolve_impl_header_edges(
            store,
            &make_call_fixture_scope(),
            input,
            &RunCancelFlagValue::default(),
        )
        .unwrap()
    };
    let set = run();
    assert_eq!(set, run(), "the same rows give the same edges");
    let edges = set
        .implements_edge_records
        .iter()
        .map(|edge| {
            assert_eq!(
                edge.edge_relation_kind_value,
                HldRelationKindValue::Implements
            );
            let name = match &edge.edge_source_reference {
                HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(name)) => name.clone(),
                source => input
                    .input_node_records
                    .iter()
                    .find(|node| &node.node_target_reference == source)
                    .unwrap()
                    .node_qualified_path_text
                    .clone(),
            };
            let (start, end) = edge
                .edge_evidence_row_key
                .evidence_site_occurrence_key
                .read_semantic_occurrence_parts()
                .span_byte_range_value
                .read_source_range_pair();
            (
                name,
                edge.edge_resolution_value.clone(),
                format!("{start}:{}", &source[start as usize..end as usize]),
            )
        })
        .collect();
    (edges, set.headers_without_source)
}

fn read_make_node_target(input: &HldMaterializationInputRecord) -> HldEdgeResolutionValue {
    HldEdgeResolutionValue::Extracted {
        target: input
            .input_node_records
            .iter()
            .find(|node| node.node_qualified_path_text == "Make")
            .unwrap()
            .node_target_reference
            .clone(),
    }
}

/// REQ-RUST-047.0
#[test]
fn test_trait_impl_header_edge() {
    let source = "struct Buffer; trait Make {} impl Make for Buffer {}";
    let (store, input) = make_header_fixture(source, |_| {});
    let (edges, without_source) = read_header_fixture_edges(source, &store, &input);
    let site = format!("{}:Make", source.rfind("Make").unwrap());
    assert_eq!(
        edges,
        vec![("Buffer".to_owned(), read_make_node_target(&input), site)],
        "one row from the self type to the trait, sited at the trait path"
    );
    assert_eq!(without_source, 0);
}

/// REQ-RUST-047.0
#[test]
fn test_builtin_source_two_headers() {
    let source = "struct Buffer; trait Make {} impl Make for u8 {} impl Make for u16 {}";
    let (store, input) = make_header_fixture(source, |_| {});
    let (edges, _) = read_header_fixture_edges(source, &store, &input);
    let make = read_make_node_target(&input);
    let site = |nth: usize| format!("{}:Make", source.match_indices("Make").nth(nth).unwrap().0);
    assert_eq!(
        edges,
        vec![
            ("u16".to_owned(), make.clone(), site(2)),
            ("u8".to_owned(), make, site(1)),
        ],
        "a builtin self type sources its own row"
    );
}

/// REQ-RUST-047.0
#[test]
fn test_inherent_negative_no_edge() {
    let source = "struct Buffer; trait Make {} impl Buffer {} impl !Make for Buffer {}";
    let (store, input) = make_header_fixture(source, |_| {});
    assert_eq!(
        read_header_fixture_edges(source, &store, &input),
        (Vec::new(), 0),
        "an inherent impl and a negative impl implement nothing"
    );
}

/// REQ-RUST-047.0
#[test]
fn test_builtin_source_one_node() {
    let source = "struct Buffer; trait Make {} impl Make for u8 {} impl Make for u8 {}";
    let (store, input) = make_header_fixture(source, |_| {});
    let set = resolve_impl_header_edges(
        &store,
        &make_call_fixture_scope(),
        &input,
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    assert_eq!(set.implements_edge_records.len(), 2);
    let nodes = crate::collect_builtin_endpoint_nodes(&set.implements_edge_records);
    assert_eq!(nodes.len(), 1, "two headers on u8 share one u8 node");
    assert_eq!(nodes[0].node_input_category, HldNodeInputCategory::Builtin);
    assert_eq!(nodes[0].node_qualified_path_text, "u8");
    let mut graph = input.clone();
    graph.input_node_records.extend(nodes);
    graph.input_edge_records.extend(set.implements_edge_records);
    assert_eq!(validate_full_graph_input(&graph), Ok(()));
}

/// REQ-RUST-047.0
#[test]
fn test_unresolved_trait_keeps_reason() {
    let source = "struct Buffer; trait Make {} impl Clone for Buffer {} impl Gone for Buffer {} \
        impl Lost for Buffer {}";
    let outside = NameSymbolTargetKey::Definition(
        SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
            &make_call_fixture_unit(),
            "src/other.rs",
            SpanByteRangeValue::try_from((0, 4)).unwrap(),
            SpanDigestIdValue::from([9; 32]),
            &[],
        )
        .unwrap(),
    );
    let (store, input) = make_header_fixture(source, |header| {
        let resolution = &mut header["impl_trait_type_resolution"];
        if resolution["type_source_evidence"]["type_source_text"] == "Clone" {
            resolution["unresolved_reason_kind"] = json!("external_source_missing");
        }
        if resolution["type_source_evidence"]["type_source_text"] == "Lost" {
            resolution["resolution_state_kind"] = json!("resolved");
            resolution["unresolved_reason_kind"] = Value::Null;
            resolution["target_identity"] = encode_symbol_target_value(&outside);
        }
    });
    let (mut edges, without_source) = read_header_fixture_edges(source, &store, &input);
    edges.sort_by_key(|(_, _, site)| site.clone());
    let unresolved = |reason| HldEdgeResolutionValue::Unresolved { reason };
    assert_eq!(
        edges,
        vec![
            (
                "Buffer".to_owned(),
                unresolved(HldUnresolvedReasonKind::ExternalSourceMissing),
                format!("{}:Clone", source.find("Clone").unwrap())
            ),
            (
                "Buffer".to_owned(),
                unresolved(HldUnresolvedReasonKind::TraitSolverNeeded),
                format!("{}:Gone", source.find("Gone").unwrap())
            ),
            (
                "Buffer".to_owned(),
                unresolved(HldUnresolvedReasonKind::MissingTarget),
                format!("{}:Lost", source.find("Lost").unwrap())
            ),
        ],
        "a trait that is not a node keeps its row with one reason: the stored one, \
         or MissingTarget when it resolved to an item outside the graph"
    );
    assert_eq!(without_source, 0);
    let mut graph = input.clone();
    graph.input_edge_records.extend(
        resolve_impl_header_edges(
            &store,
            &make_call_fixture_scope(),
            &input,
            &RunCancelFlagValue::default(),
        )
        .unwrap()
        .implements_edge_records,
    );
    assert_eq!(validate_full_graph_input(&graph), Ok(()));
}

/// REQ-RUST-047.0
#[test]
fn test_unresolved_self_counted_only() {
    let source = "struct Buffer; trait Make {} impl Make for Gone {} impl<T> Make for T {} \
        impl Gone {}";
    let (store, input) = make_header_fixture(source, |_| {});
    assert_eq!(
        read_header_fixture_edges(source, &store, &input),
        (Vec::new(), 2),
        "a trait impl whose self type is not a node has no row and is counted; \
         an inherent impl is not counted"
    );
}
