#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::original_call_resolution_records::original_call_resolution_tests::{
    OriginalCallFixtureStore, make_call_fixture_graph, make_call_fixture_occurrence,
    make_call_fixture_row, make_call_fixture_scope, make_call_fixture_unit,
};
use crate::syntax_namespace_record_codec::{
    encode_compilation_unit_value, encode_occurrence_identity_value,
};
use serde_json::{Value, json};

/// One import request of a fixture and the outcome name resolution stored for
/// it. `site` is the text of the request's byte range, taken at its first
/// occurrence after `after`. `state` is `resolved` (the target is
/// `struct Buffer`), `resolved-elsewhere` (a target that is not a node),
/// `ambiguous`, `unresolved`, `no-outcome` (no outcome row is stored), or a
/// stored reason.
struct ImportFixtureRequest {
    after: &'static str,
    site: &'static str,
    segments: &'static [&'static str],
    alias: Option<&'static str>,
    glob: bool,
    kind: &'static str,
    state: &'static str,
    chain: &'static [u8],
}

const PLAIN: ImportFixtureRequest = ImportFixtureRequest {
    after: "",
    site: "",
    segments: &[],
    alias: None,
    glob: false,
    kind: "Import",
    state: "resolved",
    chain: &[],
};

/// A call fixture graph of `source` (which must declare `struct Buffer`) with
/// one `UseStatement` node per `use ...;` outside a function body, and the
/// stored request and outcome rows of `requests`. Name resolution outcomes are
/// supplied here, at the stage seam; the resolver under test only joins them.
fn make_import_fixture_graph(
    source: &str,
    requests: &[ImportFixtureRequest],
) -> (OriginalCallFixtureStore, HldMaterializationInputRecord) {
    let (store, mut input) = make_call_fixture_graph(source);
    let unit = encode_compilation_unit_value(&make_call_fixture_unit());
    let body = source.find("fn ").unwrap_or(source.len());
    for (start, _) in source.match_indices("use ") {
        if start > body {
            continue;
        }
        let end = start + source[start..].find(';').unwrap() + 1;
        input.input_node_records.push(HldNodeInputRecord {
            node_target_reference: NameSymbolTargetKey::Definition(make_call_fixture_occurrence(
                start as u64,
                end as u64,
                &source[start..end],
            ))
            .into(),
            node_item_kind_value: Some(HldItemKindValue::UseStatement),
            node_owner_kind_value: None,
            node_qualified_path_text: source[start..end].into(),
            ..input.input_node_records[0].clone()
        });
    }
    let buffer = source.find("struct Buffer;").unwrap();
    let target = |start: usize, end: usize| {
        json!({"target_symbol_kind_value":"Struct",
            "target_source_file_path":"/fixture/src/lib.rs",
            "target_source_range_start":start, "target_source_range_end":end,
            "target_compilation_scope_record":{"unit_semantic_identity_record":unit}})
    };
    let mut extra = Vec::new();
    for request in requests {
        let from = source.find(request.after).unwrap();
        let start = from + source[from..].find(request.site).unwrap();
        let mut site = encode_occurrence_identity_value(&make_call_fixture_occurrence(
            start as u64,
            (start + request.site.len()) as u64,
            request.site,
        ));
        if !request.chain.is_empty() {
            site["span_expansion_provenance_chain"] = json!(
                request
                    .chain
                    .iter()
                    .map(|byte| blake3::Hash::from([*byte; 32]).to_hex().to_string())
                    .collect::<Vec<_>>()
            );
        }
        extra.push(make_call_fixture_row(
            "syntax-namespace-input-imports",
            json!({"import_site_occurrence_key":site,
                "import_path_segment_rows":request.segments.iter().map(|name| match *name {
                    "self" | "crate" | "super" => json!({"kind":name}),
                    name => json!({"kind":"name", "name":name}),
                }).collect::<Vec<_>>(),
                "import_binding_alias_text":request.alias,
                "import_glob_expansion_flag":request.glob,
                "import_resolution_source_kind":request.kind}),
        ));
        let (state, reason, targets) = match request.state {
            "resolved" => (
                "resolved",
                Value::Null,
                json!([target(buffer, buffer + 14)]),
            ),
            "resolved-elsewhere" => ("resolved", Value::Null, json!([target(9000, 9001)])),
            "ambiguous" => ("ambiguous", Value::Null, json!([])),
            "unresolved" => ("unresolved", Value::Null, json!([])),
            reason => ("unresolved", json!(reason), json!([])),
        };
        if request.state == "no-outcome" {
            continue;
        }
        extra.push(make_call_fixture_row(
            "import-resolution-site-records",
            json!({"import_site_occurrence_key":site,
                "import_resolution_state_kind":state,
                "import_unresolved_reason_kind":reason,
                "import_target_symbol_rows":targets,
                "import_candidate_target_rows":[]}),
        ));
    }
    store.rows.lock().unwrap().extend(extra);
    (store, input)
}

/// `imports` rows of a fixture as (source `use` text, target name or
/// `unresolved:<reason>`, site text, path text, alias), in source order, and
/// the three skipped-site counts (glob, expanded, without source). Read twice.
#[allow(clippy::type_complexity)]
fn read_import_fixture_edges(
    source: &str,
    store: &OriginalCallFixtureStore,
    input: &HldMaterializationInputRecord,
) -> (
    Vec<(String, String, String, String, Option<String>)>,
    [usize; 3],
) {
    let run = || {
        resolve_use_import_edges(
            store,
            &make_call_fixture_scope(),
            input,
            &RunCancelFlagValue::default(),
        )
        .unwrap()
    };
    let set = run();
    assert_eq!(
        set,
        run(),
        "REQ-RUST-049.0 R12: the same rows give the same edges"
    );
    let name = |target: &HldTargetIdentityValue| {
        input
            .input_node_records
            .iter()
            .find(|node| &node.node_target_reference == target)
            .unwrap()
            .node_qualified_path_text
            .clone()
    };
    let mut edges = set
        .import_edge_input_records
        .iter()
        .map(|edge| {
            assert_eq!(edge.edge_relation_kind_value, HldRelationKindValue::Imports);
            assert_eq!(
                edge.edge_evidence_row_key.evidence_source_occurrence_key,
                edge.edge_source_reference
            );
            let (start, end) = edge
                .edge_evidence_row_key
                .evidence_site_occurrence_key
                .read_semantic_occurrence_parts()
                .span_byte_range_value
                .read_source_range_pair();
            let metadata: Value =
                serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
            let target = match &edge.edge_resolution_value {
                HldEdgeResolutionValue::Extracted { target } => name(target),
                HldEdgeResolutionValue::Unresolved { reason } => format!("unresolved:{reason:?}"),
                other => panic!("unexpected resolution {other:?}"),
            };
            (
                start,
                (
                    name(&edge.edge_source_reference),
                    target,
                    source[start as usize..end as usize].to_owned(),
                    metadata["import_path_text"].as_str().unwrap().to_owned(),
                    metadata["import_alias_text"].as_str().map(str::to_owned),
                ),
            )
        })
        .collect::<Vec<_>>();
    edges.sort();
    (
        edges.into_iter().map(|(_, edge)| edge).collect(),
        [
            set.imports_glob_skipped,
            set.imports_in_expanded_items,
            set.imports_without_source,
        ],
    )
}

fn row(
    source: &str,
    target: &str,
    site: &str,
    path: &str,
    alias: Option<&str>,
) -> (String, String, String, String, Option<String>) {
    (
        source.into(),
        target.into(),
        site.into(),
        path.into(),
        alias.map(str::to_owned),
    )
}

/// REQ-RUST-049.0 R2 R3 R6 R7
#[test]
fn test_resolved_import_names_node() {
    let source = "struct Buffer; use crate::Buffer as B; fn caller() { loop {} }";
    let (store, input) = make_import_fixture_graph(
        source,
        &[ImportFixtureRequest {
            site: "crate::Buffer as B",
            segments: &["crate", "Buffer"],
            alias: Some("B"),
            ..PLAIN
        }],
    );
    let (edges, skipped) = read_import_fixture_edges(source, &store, &input);
    assert_eq!(
        edges,
        [row(
            "use crate::Buffer as B;",
            "Buffer",
            "crate::Buffer as B",
            "crate::Buffer",
            Some("B")
        )]
    );
    assert_eq!(skipped, [0, 0, 0]);
}

/// REQ-RUST-049.0 R2 R6 R7: a group gives one row per member, from one node;
/// a trailing `self` is dropped from the path text.
#[test]
fn test_grouped_import_two_rows() {
    let source = "struct Buffer; use core::mem::{self, MaybeUninit}; fn caller() { loop {} }";
    let missing = ImportFixtureRequest {
        state: "external_source_missing",
        ..PLAIN
    };
    let (store, input) = make_import_fixture_graph(
        source,
        &[
            ImportFixtureRequest {
                site: "self",
                segments: &["core", "mem", "self"],
                ..missing
            },
            ImportFixtureRequest {
                site: "MaybeUninit",
                segments: &["core", "mem", "MaybeUninit"],
                ..missing
            },
        ],
    );
    let (edges, skipped) = read_import_fixture_edges(source, &store, &input);
    let node = "use core::mem::{self, MaybeUninit};";
    let reason = "unresolved:ExternalSourceMissing";
    assert_eq!(
        edges,
        [
            row(node, reason, "self", "core::mem", None),
            row(node, reason, "MaybeUninit", "core::mem::MaybeUninit", None),
        ]
    );
    assert_eq!(skipped, [0, 0, 0]);
}

/// REQ-RUST-049.0 R4 R5: one unresolved row and one reason per import that
/// has no node target.
#[test]
fn test_unresolved_import_keeps_reason() {
    let source = "struct Buffer; use a::One; use b::Two; use c::Three; use d::Four; use e::Five; fn caller() { loop {} }";
    let states = [
        ("a::One", "resolved-elsewhere", "MissingTarget"),
        ("b::Two", "ambiguous", "AmbiguousTarget"),
        ("c::Three", "unresolved", "MissingTarget"),
        ("d::Four", "cfg_inactive", "CfgInactive"),
        ("e::Five", "trait_solver_needed", "TraitSolverNeeded"),
    ];
    let requests = states.map(|(site, state, _)| ImportFixtureRequest {
        site,
        segments: match site {
            "a::One" => &["a", "One"],
            "b::Two" => &["b", "Two"],
            "c::Three" => &["c", "Three"],
            "d::Four" => &["d", "Four"],
            _ => &["e", "Five"],
        },
        state,
        ..PLAIN
    });
    let (store, input) = make_import_fixture_graph(source, &requests);
    let (edges, skipped) = read_import_fixture_edges(source, &store, &input);
    let expected = states.map(|(site, _, reason)| {
        row(
            &format!("use {site};"),
            &format!("unresolved:{reason}"),
            site,
            site,
            None,
        )
    });
    assert_eq!(edges, expected);
    assert_eq!(skipped, [0, 0, 0]);
}

/// REQ-RUST-049.0 R8 R9 R10: a signature request, a glob, a site in expanded
/// text and a `use` in a function body give no row; the last three are counted.
/// Two globs, so the glob count cannot be mistaken for another count.
#[test]
fn test_skipped_import_sites_counted() {
    let source = "struct Buffer; use crate::Buffer; use core::mem::*; use core::ptr::*; fn caller() { use crate::Buffer as Local; loop {} }";
    let (store, input) = make_import_fixture_graph(
        source,
        &[
            ImportFixtureRequest {
                site: "Buffer",
                segments: &["Buffer"],
                glob: true,
                kind: "Signature",
                ..PLAIN
            },
            ImportFixtureRequest {
                site: "core::mem::*",
                segments: &["core", "mem"],
                glob: true,
                state: "external_source_missing",
                ..PLAIN
            },
            ImportFixtureRequest {
                site: "core::ptr::*",
                segments: &["core", "ptr"],
                glob: true,
                state: "external_source_missing",
                ..PLAIN
            },
            ImportFixtureRequest {
                site: "crate::Buffer",
                segments: &["crate", "Buffer"],
                chain: &[3],
                ..PLAIN
            },
            ImportFixtureRequest {
                after: "fn caller",
                site: "crate::Buffer as Local",
                segments: &["crate", "Buffer"],
                alias: Some("Local"),
                ..PLAIN
            },
        ],
    );
    let (edges, skipped) = read_import_fixture_edges(source, &store, &input);
    assert_eq!(edges, []);
    assert_eq!(skipped, [2, 1, 1]);
}

/// REQ-RUST-049.0: cancellation, a foreign scope and an import whose outcome
/// row is absent are typed errors.
#[test]
fn test_import_scope_cancel_errors() {
    let source = "struct Buffer; use crate::Buffer; fn caller() { loop {} }";
    let (store, input) = make_import_fixture_graph(source, &[]);
    let cancel = RunCancelFlagValue::default();
    cancel.request_stage_work_cancellation();
    assert!(matches!(
        resolve_use_import_edges(&store, &make_call_fixture_scope(), &input, &cancel),
        Err(UseImportEdgeError::Cancelled)
    ));
    let mut scope = make_call_fixture_scope();
    scope.run_generation_digest_value = "02".repeat(32);
    assert!(matches!(
        resolve_use_import_edges(&store, &scope, &input, &RunCancelFlagValue::default()),
        Err(UseImportEdgeError::InvalidScope)
    ));
    let (store, input) = make_import_fixture_graph(
        source,
        &[ImportFixtureRequest {
            site: "crate::Buffer",
            segments: &["crate", "Buffer"],
            state: "no-outcome",
            ..PLAIN
        }],
    );
    assert!(matches!(
        resolve_use_import_edges(
            &store,
            &make_call_fixture_scope(),
            &input,
            &RunCancelFlagValue::default()
        ),
        Err(UseImportEdgeError::InvalidEvidence {
            field: "import_site_occurrence_key"
        })
    ));
}
