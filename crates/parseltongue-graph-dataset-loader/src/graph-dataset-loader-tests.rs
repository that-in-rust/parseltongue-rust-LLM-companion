//! Unit tests of REQ-RUST-067.0 on a small export written here: three nodes and three edges.

use crate::{
    GraphDatasetLoadError, GraphExportFileKind, build_graph_dataset_tables,
    load_graph_export_files, read_graph_dataset_file, write_canonical_json_text,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const RUN_GENERATION_SAMPLE_TEXT: &str = "generation-one";

fn build_export_header_line(table: &str, generation: &str) -> Value {
    json!({
        "export_owned_table_name": table,
        "export_record_kind_value": "header",
        "export_scope_identity_record": {"run_generation_digest_value": generation},
    })
}

fn build_node_export_row(identity: Value, path: &str) -> Value {
    json!({
        "export_record_kind_value": "row",
        "export_row_identity_value": write_canonical_json_text(&identity),
        "export_row_payload_value": {
            "node_target_identity": identity,
            "node_qualified_path_text": path,
            "node_input_category": "Definition",
            "node_item_kind_value": null,
            "node_function_body_ancestry_flag": false,
            "node_reachable_outside_crate_flag": null,
            "run_generation_digest_full": RUN_GENERATION_SAMPLE_TEXT,
        },
    })
}

fn build_edge_export_row(site: u64, source: Value, resolution: Value) -> Value {
    let evidence = json!({
        "relation_kind": "Calls",
        "site_occurrence_identity": {"byte_range": [site, site + 5], "source_file_path": "src/lib.rs"},
        "source_target_identity": source,
    });
    json!({
        "export_record_kind_value": "row",
        "export_row_identity_value": write_canonical_json_text(&json!({"evidence_identity": evidence})),
        "export_row_payload_value": {
            "edge_dispatch_metadata_text": null,
            "edge_evidence_identity": evidence,
            "edge_relation_kind": "Calls",
            "edge_resolution": resolution,
            "edge_source_target_identity": source,
            "run_generation_digest_full": RUN_GENERATION_SAMPLE_TEXT,
        },
    })
}

fn build_sample_node_identity(name: &str) -> Value {
    json!({"kind": "Definition", "occurrence_identity": {"name": name, "byte_range": [1, 2]}})
}

fn build_sample_node_lines() -> Vec<Value> {
    [
        build_export_header_line(
            GraphExportFileKind::NodeRows.read_export_table_name(),
            RUN_GENERATION_SAMPLE_TEXT,
        ),
        build_node_export_row(build_sample_node_identity("alpha"), "sample::alpha"),
        build_node_export_row(build_sample_node_identity("beta"), "sample::beta"),
        build_node_export_row(json!({"builtin_name": "u8", "kind": "Builtin"}), "u8"),
    ]
    .into()
}

/// alpha calls beta, alpha calls u8's item, and one call of beta has no target.
fn build_sample_edge_lines() -> Vec<Value> {
    let extracted = |target: Value| json!({"kind": "Extracted", "target": target});
    [
        build_export_header_line(
            GraphExportFileKind::EdgeRows.read_export_table_name(),
            RUN_GENERATION_SAMPLE_TEXT,
        ),
        build_edge_export_row(
            10,
            build_sample_node_identity("alpha"),
            extracted(build_sample_node_identity("beta")),
        ),
        build_edge_export_row(
            20,
            build_sample_node_identity("alpha"),
            extracted(json!({"kind": "Builtin", "builtin_name": "u8"})),
        ),
        build_edge_export_row(
            30,
            build_sample_node_identity("beta"),
            json!({"kind": "Unresolved", "reason": "TraitSolverNeeded"}),
        ),
    ]
    .into()
}

fn write_export_line_text(lines: &[Value]) -> String {
    lines.iter().map(|line| format!("{line}\n")).collect()
}

fn check_sample_export_lines(
    nodes: &[Value],
    edges: &[Value],
) -> Result<crate::GraphDatasetRowTables, GraphDatasetLoadError> {
    build_graph_dataset_tables(
        &write_export_line_text(nodes),
        &write_export_line_text(edges),
    )
}

/// The sample with one line of one file replaced.
fn replace_sample_export_line(lines: Vec<Value>, index: usize, line: Value) -> Vec<Value> {
    lines
        .into_iter()
        .enumerate()
        .map(|(position, old)| if position == index { line.clone() } else { old })
        .collect()
}

fn write_sample_export_files(directory: &Path, nodes: &[Value], edges: &[Value]) -> [PathBuf; 3] {
    let paths = ["nodes.json", "edges.json", "graph.duckdb"].map(|name| directory.join(name));
    std::fs::write(&paths[0], write_export_line_text(nodes)).unwrap();
    std::fs::write(&paths[1], write_export_line_text(edges)).unwrap();
    paths
}

/// REQ-RUST-067.0 C3 support: equal JSON values give equal key text whatever the key order.
#[test]
fn test_canonical_json_text_sorted() {
    let left: Value = serde_json::from_str(r#"{"b":[1,{"z":null,"a":"x\"y"}],"a":true}"#).unwrap();
    let right: Value = serde_json::from_str(r#"{"a":true,"b":[1,{"a":"x\"y","z":null}]}"#).unwrap();
    assert_eq!(
        write_canonical_json_text(&left),
        r#"{"a":true,"b":[1,{"a":"x\"y","z":null}]}"#
    );
    assert_eq!(
        write_canonical_json_text(&left),
        write_canonical_json_text(&right)
    );
    assert_eq!(
        serde_json::from_str::<Value>(&write_canonical_json_text(&left)).unwrap(),
        left
    );
}

/// REQ-RUST-067.0 C1: the first line is a header that names the expected table.
#[test]
fn test_export_header_line_required() {
    let nodes = build_sample_node_lines();
    let edges = build_sample_edge_lines();
    assert_eq!(
        check_sample_export_lines(&nodes[1..], &edges),
        Err(GraphDatasetLoadError::ExportHeaderMissing {
            export_file_kind_value: GraphExportFileKind::NodeRows
        })
    );
    assert_eq!(
        check_sample_export_lines(&nodes, &nodes),
        Err(GraphDatasetLoadError::ExportTableNameMismatch {
            export_file_kind_value: GraphExportFileKind::EdgeRows,
            export_owned_table_name: "item-effective-visibility-records".into(),
        })
    );
    let twice = replace_sample_export_line(edges.clone(), 2, edges[0].clone());
    assert_eq!(
        check_sample_export_lines(&nodes, &twice),
        Err(GraphDatasetLoadError::ExportRowShapeInvalid {
            export_file_kind_value: GraphExportFileKind::EdgeRows,
            export_line_number_value: 3,
        })
    );
    assert!(matches!(
        build_graph_dataset_tables("not json\n", &write_export_line_text(&edges)),
        Err(GraphDatasetLoadError::ExportLineNotJson {
            export_file_kind_value: GraphExportFileKind::NodeRows,
            export_line_number_value: 1,
            ..
        })
    ));
}

/// REQ-RUST-067.0 C2: both headers and every row name one run generation.
#[test]
fn test_run_generation_rows_agree() {
    let nodes = build_sample_node_lines();
    let edges = build_sample_edge_lines();
    let mut row = nodes[2].clone();
    row["export_row_payload_value"]["run_generation_digest_full"] = json!("generation-two");
    assert_eq!(
        check_sample_export_lines(&replace_sample_export_line(nodes.clone(), 2, row), &edges),
        Err(GraphDatasetLoadError::RunGenerationMismatch {
            export_file_kind_value: GraphExportFileKind::NodeRows,
            export_line_number_value: 3,
            run_generation_digest_value: "generation-two".into(),
        })
    );
    let other = edges
        .iter()
        .map(|line| {
            serde_json::from_str::<Value>(
                &line.to_string().replace("generation-one", "generation-two"),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        check_sample_export_lines(&nodes, &other),
        Err(GraphDatasetLoadError::RunGenerationMismatch {
            export_file_kind_value: GraphExportFileKind::EdgeRows,
            export_line_number_value: 1,
            run_generation_digest_value: "generation-two".into(),
        })
    );
}

/// REQ-RUST-067.0 C3: a node's identity text is JSON equal to its `node_target_identity`.
#[test]
fn test_node_identity_text_checked() {
    let nodes = build_sample_node_lines();
    let mut row = nodes[1].clone();
    row["export_row_identity_value"] = nodes[2]["export_row_identity_value"].clone();
    let text = row["export_row_identity_value"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        check_sample_export_lines(
            &replace_sample_export_line(nodes.clone(), 1, row),
            &build_sample_edge_lines()
        ),
        Err(GraphDatasetLoadError::NodeIdentityTextMismatch {
            export_line_number_value: 2,
            export_row_identity_text: text,
        })
    );
    let mut reordered = nodes[3].clone();
    reordered["export_row_identity_value"] =
        json!(r#"{ "kind": "Builtin", "builtin_name": "u8" }"#);
    assert!(
        check_sample_export_lines(
            &replace_sample_export_line(nodes, 3, reordered),
            &build_sample_edge_lines()
        )
        .is_ok(),
        "key order and spacing of the identity text do not matter"
    );
}

/// REQ-RUST-067.0 C4: no two nodes and no two edges share an identity.
#[test]
fn test_duplicate_row_identity_rejected() {
    let nodes = build_sample_node_lines();
    let edges = build_sample_edge_lines();
    assert_eq!(
        check_sample_export_lines(
            &replace_sample_export_line(nodes.clone(), 3, nodes[1].clone()),
            &edges[..1]
                .iter()
                .chain(&edges[3..])
                .cloned()
                .collect::<Vec<_>>()
        ),
        Err(GraphDatasetLoadError::RowIdentityDuplicated {
            export_file_kind_value: GraphExportFileKind::NodeRows,
            export_line_number_value: 4,
            export_row_identity_text: write_canonical_json_text(&build_sample_node_identity(
                "alpha"
            )),
        })
    );
    assert_eq!(
        check_sample_export_lines(
            &nodes,
            &replace_sample_export_line(edges.clone(), 3, edges[2].clone())
        ),
        Err(GraphDatasetLoadError::RowIdentityDuplicated {
            export_file_kind_value: GraphExportFileKind::EdgeRows,
            export_line_number_value: 4,
            export_row_identity_text: edges[2]["export_row_identity_value"]
                .as_str()
                .unwrap()
                .into(),
        })
    );
}

/// REQ-RUST-067.0 C5: every edge's source is a node.
#[test]
fn test_edge_source_node_required() {
    let edges = replace_sample_export_line(
        build_sample_edge_lines(),
        3,
        build_edge_export_row(
            30,
            build_sample_node_identity("gamma"),
            json!({"kind": "Unresolved", "reason": "TraitSolverNeeded"}),
        ),
    );
    assert_eq!(
        check_sample_export_lines(&build_sample_node_lines(), &edges),
        Err(GraphDatasetLoadError::EdgeSourceNotNode {
            export_line_number_value: 4,
            node_identity_key_text: write_canonical_json_text(&build_sample_node_identity("gamma")),
        })
    );
}

/// REQ-RUST-067.0 C6: an edge has a target that is a node, or no target and a reason.
#[test]
fn test_edge_target_node_required() {
    let check = |resolution: Value| {
        check_sample_export_lines(
            &build_sample_node_lines(),
            &replace_sample_export_line(
                build_sample_edge_lines(),
                1,
                build_edge_export_row(10, build_sample_node_identity("alpha"), resolution),
            ),
        )
    };
    assert_eq!(
        check(json!({"kind": "Extracted", "target": build_sample_node_identity("gamma")})),
        Err(GraphDatasetLoadError::EdgeTargetNotNode {
            export_line_number_value: 2,
            node_identity_key_text: write_canonical_json_text(&build_sample_node_identity("gamma")),
        })
    );
    let invalid = Err(GraphDatasetLoadError::EdgeResolutionShapeInvalid {
        export_line_number_value: 2,
    });
    assert_eq!(check(json!({"kind": "Unresolved"})), invalid);
    assert_eq!(
        check(
            json!({"kind": "Extracted", "reason": "X", "target": build_sample_node_identity("beta")})
        ),
        invalid
    );
    assert_eq!(
        check(json!({"kind": "Extracted", "target": "beta"})),
        invalid
    );
}

/// REQ-RUST-067.0 C7: a file with a header and no row is refused.
#[test]
fn test_export_file_rows_required() {
    assert_eq!(
        check_sample_export_lines(&build_sample_node_lines(), &build_sample_edge_lines()[..1]),
        Err(GraphDatasetLoadError::ExportFileEmpty {
            export_file_kind_value: GraphExportFileKind::EdgeRows
        })
    );
}

/// REQ-RUST-067.0 A2 and A3: the rows read back from the file equal the rows checked, every
/// column is the payload field it copies, and two loads of one export store equal rows.
#[test]
fn test_small_export_dataset_roundtrip() {
    let (nodes, edges) = (build_sample_node_lines(), build_sample_edge_lines());
    let directory = tempfile::tempdir().unwrap();
    let [node_path, edge_path, output] =
        write_sample_export_files(directory.path(), &nodes, &edges);
    let info = load_graph_export_files(&node_path, &edge_path, &output).unwrap();
    assert_eq!(
        (info.node_row_count_value, info.edge_row_count_value),
        (3, 3)
    );
    assert_eq!(info.run_generation_digest_value, RUN_GENERATION_SAMPLE_TEXT);
    let stored = read_graph_dataset_file(&output).unwrap();
    assert_eq!(stored, check_sample_export_lines(&nodes, &edges).unwrap());
    assert_eq!(stored.dataset_info_record_value, info);

    let payloads = |lines: &[Value]| {
        let mut values = lines[1..]
            .iter()
            .map(|line| line["export_row_payload_value"].clone())
            .collect::<Vec<_>>();
        values.sort_by_key(write_canonical_json_text);
        values
    };
    let parse = |text: &String| serde_json::from_str::<Value>(text).unwrap();
    let mut stored_nodes = stored
        .dataset_node_record_values
        .iter()
        .map(|row| parse(&row.node_payload_json_text))
        .collect::<Vec<_>>();
    stored_nodes.sort_by_key(write_canonical_json_text);
    assert_eq!(stored_nodes, payloads(&nodes));
    let mut stored_edges = stored
        .dataset_edge_record_values
        .iter()
        .map(|row| parse(&row.edge_payload_json_text))
        .collect::<Vec<_>>();
    stored_edges.sort_by_key(write_canonical_json_text);
    assert_eq!(stored_edges, payloads(&edges));

    let beta = write_canonical_json_text(&build_sample_node_identity("beta"));
    let unresolved = stored
        .dataset_edge_record_values
        .iter()
        .find(|row| row.target_node_identity_key.is_none())
        .unwrap();
    assert_eq!(unresolved.source_node_identity_key, beta);
    assert_eq!(unresolved.edge_resolution_kind_value, "Unresolved");
    assert_eq!(
        unresolved.edge_unresolved_reason_value.as_deref(),
        Some("TraitSolverNeeded")
    );
    assert_eq!(
        (
            unresolved.site_byte_start_value,
            unresolved.site_byte_end_value
        ),
        (Some(30), Some(35))
    );
    assert_eq!(
        unresolved.site_source_file_path.as_deref(),
        Some("src/lib.rs")
    );
    let node = stored
        .dataset_node_record_values
        .iter()
        .find(|row| row.node_identity_key_text == beta)
        .unwrap();
    assert_eq!(
        node.node_qualified_path_text.as_deref(),
        Some("sample::beta")
    );
    assert_eq!(node.node_function_body_ancestry_flag, Some(false));
    assert_eq!(node.node_reachable_outside_crate_flag, None);
    assert_eq!(node.node_item_kind_value, None);

    let second = directory.path().join("second.duckdb");
    load_graph_export_files(&node_path, &edge_path, &second).unwrap();
    assert_eq!(read_graph_dataset_file(&second).unwrap(), stored);
    let names = std::fs::read_dir(directory.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        names.iter().map(String::as_str).collect::<Vec<_>>(),
        ["edges.json", "graph.duckdb", "nodes.json", "second.duckdb"],
        "no work file is left beside the output"
    );
}

/// REQ-RUST-067.0 A4: a failed check leaves nothing at the output path or beside it.
#[test]
fn test_failed_check_writes_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let [node_path, edge_path, output] = write_sample_export_files(
        directory.path(),
        &build_sample_node_lines()[..3],
        &build_sample_edge_lines(),
    );
    assert!(matches!(
        load_graph_export_files(&node_path, &edge_path, &output),
        Err(GraphDatasetLoadError::EdgeTargetNotNode {
            export_line_number_value: 3,
            ..
        })
    ));
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2);
    assert!(matches!(
        load_graph_export_files(&directory.path().join("absent.json"), &edge_path, &output),
        Err(GraphDatasetLoadError::DatasetFileAccessFailed { .. })
    ));
    assert!(!output.exists());
}

/// REQ-RUST-067.0 A5 and A7: an output path that exists is refused and keeps its bytes, and
/// a load leaves the bytes of both input files as they were.
#[test]
fn test_existing_output_path_kept() {
    let directory = tempfile::tempdir().unwrap();
    let [node_path, edge_path, output] = write_sample_export_files(
        directory.path(),
        &build_sample_node_lines(),
        &build_sample_edge_lines(),
    );
    let inputs = [
        std::fs::read(&node_path).unwrap(),
        std::fs::read(&edge_path).unwrap(),
    ];
    std::fs::write(&output, b"kept").unwrap();
    assert_eq!(
        load_graph_export_files(&node_path, &edge_path, &output),
        Err(GraphDatasetLoadError::DatasetOutputPathExists {
            dataset_output_file_path: output.clone()
        })
    );
    assert_eq!(std::fs::read(&output).unwrap(), b"kept");
    load_graph_export_files(&node_path, &edge_path, &directory.path().join("new.duckdb")).unwrap();
    assert_eq!(
        [
            std::fs::read(&node_path).unwrap(),
            std::fs::read(&edge_path).unwrap()
        ],
        inputs
    );
}
