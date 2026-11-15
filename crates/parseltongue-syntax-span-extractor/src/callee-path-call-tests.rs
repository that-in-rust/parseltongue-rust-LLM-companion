#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::original_call_resolution_tests::{
    OriginalCallFixtureStore, make_call_fixture_graph, make_call_fixture_occurrence,
    make_call_fixture_row, make_call_fixture_scope, make_call_fixture_unit,
};
use super::*;
use crate::syntax_namespace_record_codec::{
    encode_compilation_unit_value, encode_symbol_target_value,
};
use serde_json::{Value, json};

/// A call fixture graph of `source` (which must declare `struct Buffer` and
/// `fn helper`) with one stored path row per entry of `paths`: (path text at
/// its last occurrence, context kind, state, target symbol kind or stored
/// reason). A target kind of `Function` names `helper`; any other kind is a
/// target row of that kind only. Name lookup outcomes are supplied here, at the
/// stage seam; the resolver under test only joins them.
fn make_callee_fixture_graph(
    source: &str,
    paths: &[(&str, &str, &str, &str)],
) -> (OriginalCallFixtureStore, HldMaterializationInputRecord) {
    let (store, input) = make_call_fixture_graph(source);
    let unit = encode_compilation_unit_value(&make_call_fixture_unit());
    let helper = store
        .rows
        .lock()
        .unwrap()
        .iter()
        .filter(|row| row.stage_owned_table_name == "span-reference-text-table")
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .find(|span| span["span_declared_name_value"] == "helper")
        .unwrap();
    let function = json!({"target_symbol_kind_value":"Function",
        "target_source_file_path":"/fixture/src/lib.rs",
        "target_source_range_start":helper["span_byte_start_offset"],
        "target_source_range_end":helper["span_byte_end_offset"],
        "target_compilation_scope_record":{"unit_semantic_identity_record":unit}});
    let target = NameSymbolTargetKey::Definition(make_call_fixture_occurrence(
        helper["span_byte_start_offset"].as_u64().unwrap(),
        helper["span_byte_end_offset"].as_u64().unwrap(),
        helper["span_source_code_text"].as_str().unwrap(),
    ));
    let mut extra = vec![make_call_fixture_row(
        "syntax-namespace-input-items",
        json!({"binding_target_identity_key":encode_symbol_target_value(&target),
            "binding_target_metadata_value":function}),
    )];
    for (text, context, state, kind) in paths {
        let start = source.rfind(text).unwrap();
        let (targets, reason) = match (*state, *kind) {
            ("resolved", "Function") => (json!([function]), Value::Null),
            ("resolved", "Function+Function") => (json!([function, function]), Value::Null),
            ("resolved", "Builtin") => (
                json!([{"target_symbol_kind_value":"Builtin", "target_builtin_name_value":"str"}]),
                Value::Null,
            ),
            ("resolved", kind) => (json!([{"target_symbol_kind_value":kind}]), Value::Null),
            (_, "") => (json!([]), Value::Null),
            (_, reason) => (json!([]), json!(reason)),
        };
        extra.push(make_call_fixture_row(
            "path-resolution-site-records",
            json!({"site_compilation_scope_record":{"unit_semantic_identity_record":unit},
                "site_source_absolute_path":"/fixture/src/lib.rs",
                "site_source_range_start":start, "site_source_range_end":start + text.len(),
                "site_expansion_provenance_chain":[],
                "site_path_context_kind":context,
                "path_site_source_text":text,
                "path_resolution_state_kind":state,
                "path_unresolved_reason_kind":reason,
                "path_target_symbol_rows":targets}),
        ));
    }
    store.rows.lock().unwrap().extend(extra);
    (store, input)
}

/// The one call row of `caller`: its target's name or `unresolved:<reason>`,
/// its dispatch kind, and what stopped the lookup.
fn read_callee_fixture_row(
    store: &OriginalCallFixtureStore,
    input: &HldMaterializationInputRecord,
) -> (String, String, String) {
    let edges = resolve_original_call_edges(
        store,
        &make_call_fixture_scope(),
        input,
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    let [edge] = edges.as_slice() else {
        panic!("one call expected, found {}", edges.len());
    };
    let metadata: Value =
        serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
    let target = match &edge.edge_resolution_value {
        HldEdgeResolutionValue::Extracted { target } => input
            .input_node_records
            .iter()
            .find(|node| &node.node_target_reference == target)
            .unwrap()
            .node_qualified_path_text
            .clone(),
        HldEdgeResolutionValue::Unresolved { reason } => format!("unresolved:{reason:?}"),
        other => panic!("neither extracted nor unresolved: {other:?}"),
    };
    let outcome = metadata["callee_path_lookup_evidence"]["callee_path_outcome_kind"]
        .as_str()
        .unwrap_or("-")
        .to_owned();
    let detail = metadata["unresolved_detail_kind"]
        .as_str()
        .unwrap_or("-")
        .to_owned();
    // A row the callee path decided repeats the outcome as what stopped it.
    assert!(
        !detail.starts_with("callee_")
            || detail == outcome
            || [
                "callee_path_not_requested",
                "callee_prefix_lexically_shadowed"
            ]
            .contains(&detail.as_str()),
        "{metadata}"
    );
    (
        target,
        metadata["dispatch_kind"].as_str().unwrap().to_owned(),
        format!("{detail} {outcome}"),
    )
}

/// Expected row: target, dispatch kind, "<what stopped it> <callee path outcome>".
fn row(target: &str, kind: &str, detail: &str) -> (String, String, String) {
    (target.to_owned(), kind.to_owned(), detail.to_owned())
}

const BARE: &str = "struct Buffer; fn helper() {} fn caller() { helper(); }";
const CALLEE: &str = "body_call_callee_path";
const NONE: &str = "undetermined";

/// REQ-RUST-052.0 G9
#[test]
fn test_bare_callee_names_function() {
    let (store, input) =
        make_callee_fixture_graph(BARE, &[("helper", CALLEE, "resolved", "Function")]);
    assert_eq!(
        read_callee_fixture_row(&store, &input),
        row("helper", "callee_path_function", "- single_function_node")
    );
    let generic = BARE.replace("helper();", "helper::<u8>();");
    let (store, input) = make_callee_fixture_graph(
        &generic,
        &[("helper::<u8>", CALLEE, "resolved", "Function")],
    );
    assert_eq!(
        read_callee_fixture_row(&store, &input),
        row("helper", "callee_path_function", "- single_function_node"),
        "generic arguments on the callee do not change the function named"
    );
}

/// REQ-RUST-052.0 G9: a bare callee without a function target keeps one row and says why.
#[test]
fn test_bare_callee_untargeted_reasons() {
    let outcome = |paths: &[(&str, &str, &str, &str)]| {
        let (store, input) = make_callee_fixture_graph(BARE, paths);
        read_callee_fixture_row(&store, &input)
    };
    let cases = [
        // No stored lookup for the callee: the row is as it was before rule G9.
        (vec![], "TraitSolverNeeded", "callee_path_not_requested -"),
        (
            vec![("helper", CALLEE, "resolved", "Struct")],
            "TraitSolverNeeded",
            "callee_is_not_function callee_is_not_function",
        ),
        (
            vec![("helper", CALLEE, "unresolved", "external_source_missing")],
            "ExternalSourceMissing",
            "callee_path_unresolved callee_path_unresolved",
        ),
        (
            vec![("helper", CALLEE, "unresolved", "")],
            "TraitSolverNeeded",
            "callee_path_unresolved callee_path_unresolved",
        ),
        (
            vec![("helper", CALLEE, "ambiguous", "")],
            "AmbiguousTarget",
            "callee_path_ambiguous callee_path_ambiguous",
        ),
        // Stored as resolved, with two targets: never one of them picked.
        (
            vec![("helper", CALLEE, "resolved", "Function+Function")],
            "AmbiguousTarget",
            "callee_path_ambiguous callee_path_ambiguous",
        ),
    ];
    for (paths, reason, detail) in cases {
        assert_eq!(
            outcome(&paths),
            row(&format!("unresolved:{reason}"), NONE, detail),
            "{paths:?}"
        );
    }
    let (store, mut input) =
        make_callee_fixture_graph(BARE, &[("helper", CALLEE, "resolved", "Function")]);
    input
        .input_node_records
        .retain(|node| node.node_qualified_path_text != "helper");
    assert_eq!(
        read_callee_fixture_row(&store, &input),
        row(
            "unresolved:MissingTarget",
            NONE,
            "callee_target_not_node callee_target_not_node"
        ),
        "a function that is not a graph node, such as one declared in a block"
    );
}

/// REQ-RUST-052.0 G9: a qualified callee is decided by its whole path only when its
/// qualifier is stored as a module; otherwise the row keeps the record of the whole
/// path and goes to the type-prefix lookup.
#[test]
fn test_qualified_callee_needs_module() {
    let source = "struct Buffer; fn helper() {} fn caller() { m::helper(); }";
    let outcome = |prefix: (&str, &str), callee: (&str, &str)| {
        let (store, input) = make_callee_fixture_graph(
            source,
            &[
                ("m", "body_call_type_prefix", prefix.0, prefix.1),
                ("m::helper", CALLEE, callee.0, callee.1),
            ],
        );
        read_callee_fixture_row(&store, &input)
    };
    let missing = "external_source_missing";
    let cases = [
        (
            ("resolved", "Module"),
            ("resolved", "Function"),
            "helper",
            "callee_path_function",
            "- single_function_node",
        ),
        // A trait's method named through the trait is not this rule's.
        (
            ("resolved", "Trait"),
            ("resolved", "Function"),
            "unresolved:TraitSolverNeeded",
            NONE,
            "type_prefix_is_trait callee_qualifier_not_module",
        ),
        // A module qualifier whose whole path has no target: the older outcome stands.
        (
            ("resolved", "Module"),
            ("unresolved", ""),
            "unresolved:TraitSolverNeeded",
            NONE,
            "type_prefix_is_module callee_path_unresolved",
        ),
        (
            ("resolved", "Module"),
            ("resolved", "Struct"),
            "unresolved:TraitSolverNeeded",
            NONE,
            "type_prefix_is_module callee_is_not_function",
        ),
        (
            ("resolved", "Module"),
            ("ambiguous", ""),
            "unresolved:TraitSolverNeeded",
            NONE,
            "type_prefix_is_module callee_path_ambiguous",
        ),
        // The whole path's stored reason wins over a qualifier stored as a builtin type.
        (
            ("resolved", "Builtin"),
            ("unresolved", missing),
            "unresolved:ExternalSourceMissing",
            NONE,
            "callee_path_unresolved callee_path_unresolved",
        ),
        // A builtin qualifier whose whole path has no stored reason goes to member lookup.
        (
            ("resolved", "Builtin"),
            ("unresolved", ""),
            "unresolved:MissingTarget",
            NONE,
            "member_lookup_unproven callee_path_unresolved",
        ),
        // A qualifier without a target keeps its own outcome.
        (
            ("unresolved", missing),
            ("unresolved", missing),
            "unresolved:ExternalSourceMissing",
            NONE,
            "type_prefix_unresolved callee_path_unresolved",
        ),
    ];
    for (prefix, callee, target, kind, detail) in cases {
        assert_eq!(
            outcome(prefix, callee),
            row(target, kind, detail),
            "{prefix:?} {callee:?}"
        );
    }
}

/// REQ-RUST-052.0 G9: the whole path's stored reason does not displace the member
/// lookup of a struct qualifier, the only lookup that can give such a call a target.
#[test]
fn test_struct_qualifier_keeps_member() {
    let source = "struct Buffer; impl Buffer { fn new() {} } fn helper() {} \
        fn caller() { Buffer::new(); }";
    let (store, input) = make_callee_fixture_graph(
        source,
        &[(
            "Buffer::new",
            CALLEE,
            "unresolved",
            "external_source_missing",
        )],
    );
    assert_eq!(
        read_callee_fixture_row(&store, &input),
        row(
            "new",
            "type_path_associated_function",
            "- callee_path_unresolved"
        )
    );
}

/// REQ-RUST-052.0 G9: stored lookups for a qualifier that a name in the function
/// shadows are not used, as for a type prefix (REQ-RUST-034.0). No record is kept.
#[test]
fn test_shadowed_qualifier_blocks_callee() {
    let source = "struct Buffer; fn helper() {} fn caller(m: u8) { m::helper(); }";
    let (store, input) = make_callee_fixture_graph(
        source,
        &[
            ("m", "body_call_type_prefix", "resolved", "Module"),
            ("m::helper", CALLEE, "resolved", "Function"),
        ],
    );
    let (target, kind, detail) = read_callee_fixture_row(&store, &input);
    assert!(target.starts_with("unresolved:"), "{target}");
    assert_eq!(
        (kind.as_str(), detail.as_str()),
        (NONE, "callee_prefix_lexically_shadowed -")
    );
}
