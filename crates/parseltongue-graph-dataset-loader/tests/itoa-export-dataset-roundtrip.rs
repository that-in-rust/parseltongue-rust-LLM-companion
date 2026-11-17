//! REQ-RUST-067.0 on real source: pinned itoa goes through `parseltongue`, its two graph
//! tables are exported, and the loader's DuckDB file holds every exported row.

#![allow(clippy::unwrap_used)]

use parseltongue_graph_dataset_loader::{read_graph_dataset_file, write_canonical_json_text};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const WORKSPACE_ROOT_DIRECTORY_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const PINNED_ITOA_TREE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../parseltongue-cli-dispatcher-binary/tests/pinned-itoa-source-tree"
);
const PINNED_ITOA_FILE_NAMES: [(&str, &str); 5] = [
    ("upstream-cargo-manifest-file.toml", "Cargo.toml"),
    ("qualification-cargo-lock-file.toml", "Cargo.lock"),
    ("src/lib.rs", "src/lib.rs"),
    ("src/u128_ext.rs", "src/u128_ext.rs"),
    ("benches/bench.rs", "benches/bench.rs"),
];

/// `PARSELTONGUE_FROZEN_EXECUTABLE_PATH` when set, otherwise the release build of this
/// workspace. The loader crate does not build `parseltongue`, so a missing file is a failure.
fn read_mapper_executable_path() -> PathBuf {
    let path = std::env::var_os("PARSELTONGUE_FROZEN_EXECUTABLE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(WORKSPACE_ROOT_DIRECTORY_PATH).join("target/release/parseltongue")
        });
    assert!(
        path.is_file(),
        "BLOCKED: no parseltongue executable at {}; build it with `cargo build --release -p \
         parseltongue-cli-dispatcher-binary` or set PARSELTONGUE_FROZEN_EXECUTABLE_PATH",
        path.display()
    );
    path
}

/// A copy of the vendored pinned itoa 1.0.18 tree under `parent`.
fn prepare_pinned_itoa_workspace(parent: &Path) -> PathBuf {
    let root = parent.join("itoa");
    ["src", "benches"]
        .iter()
        .for_each(|name| std::fs::create_dir_all(root.join(name)).unwrap());
    PINNED_ITOA_FILE_NAMES.iter().for_each(|(from, to)| {
        std::fs::copy(Path::new(PINNED_ITOA_TREE_PATH).join(from), root.join(to)).unwrap();
    });
    root
}

fn check_command_exit_success(name: &str, output: Output) -> Output {
    assert!(
        output.status.success(),
        "{name} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// Exports one published table of `store` with the mapper and writes it to `file`.
fn export_published_table_file(mapper: &Path, store: &Path, table: &str, file: &Path) {
    let output = check_command_exit_success(
        "export-published-stage-rows",
        Command::new(mapper)
            .args(["export-published-stage-rows", "--workspace-store-file-path"])
            .arg(store)
            .args(["--stage-owned-table-name", table])
            .output()
            .unwrap(),
    );
    std::fs::write(file, output.stdout).unwrap();
}

/// The payload of every row line of an exported file, read here and not by the loader.
fn read_export_payload_values(file: &Path) -> Vec<Value> {
    std::fs::read_to_string(file)
        .unwrap()
        .lines()
        .skip(1)
        .map(|line| serde_json::from_str::<Value>(line).unwrap()["export_row_payload_value"].take())
        .collect()
}

fn sort_payload_value_texts(values: impl Iterator<Item = Value>) -> Vec<String> {
    let mut texts = values
        .map(|value| write_canonical_json_text(&value))
        .collect::<Vec<_>>();
    texts.sort();
    texts
}

fn read_optional_text_value(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}

/// REQ-RUST-067.0 A1 and A2: a `declarations` run of pinned itoa, exported and loaded, gives
/// one DuckDB file with 170 node rows and 682 edge rows, and every row equals its JSON row.
#[test]
fn test_itoa_export_dataset_roundtrip() {
    let mapper = read_mapper_executable_path();
    let directory = tempfile::tempdir().unwrap();
    let root = prepare_pinned_itoa_workspace(directory.path());
    let path = |name: &str| directory.path().join(name);
    let (store, nodes, edges, dataset) = (
        path("workspace-store.db"),
        path("full-graph-nodes.json"),
        path("full-graph-edges.json"),
        path("full-graph-dataset.duckdb"),
    );
    check_command_exit_success(
        "map-workspace-full-run",
        Command::new(&mapper)
            .args(["map-workspace-full-run", "--workspace-root-path-value"])
            .arg(&root)
            .arg("--workspace-store-file-path")
            .arg(&store)
            .args([
                "--worker-thread-count-value",
                "2",
                "--mapping-final-stage-kind",
                "full-item-dependency-graph",
                "--sysroot-source-scope-kind",
                "declarations",
            ])
            .output()
            .unwrap(),
    );
    export_published_table_file(&mapper, &store, "item-effective-visibility-records", &nodes);
    export_published_table_file(&mapper, &store, "edge-evidence-record-table", &edges);
    let export_bytes = [
        std::fs::read(&nodes).unwrap(),
        std::fs::read(&edges).unwrap(),
    ];

    let loaded = check_command_exit_success(
        "parseltongue-graph-dataset-loader",
        Command::new(env!("CARGO_BIN_EXE_parseltongue-graph-dataset-loader"))
            .arg("--node-export-file-path")
            .arg(&nodes)
            .arg("--edge-export-file-path")
            .arg(&edges)
            .arg("--dataset-output-file-path")
            .arg(&dataset)
            .output()
            .unwrap(),
    );
    let summary: Value = serde_json::from_slice(&loaded.stdout).unwrap();
    assert_eq!(summary["node_row_count_value"], 170);
    assert_eq!(summary["edge_row_count_value"], 682);
    assert_eq!(
        [
            std::fs::read(&nodes).unwrap(),
            std::fs::read(&edges).unwrap()
        ],
        export_bytes,
        "the loader changed an input file"
    );

    let stored = read_graph_dataset_file(&dataset).unwrap();
    let (node_payloads, edge_payloads) = (
        read_export_payload_values(&nodes),
        read_export_payload_values(&edges),
    );
    assert_eq!(stored.dataset_node_record_values.len(), 170);
    assert_eq!(stored.dataset_edge_record_values.len(), 682);
    let parse = |text: &String| serde_json::from_str::<Value>(text).unwrap();
    assert_eq!(
        sort_payload_value_texts(
            stored
                .dataset_node_record_values
                .iter()
                .map(|row| parse(&row.node_payload_json_text))
        ),
        sort_payload_value_texts(node_payloads.iter().cloned()),
        "node payloads differ from the export"
    );
    assert_eq!(
        sort_payload_value_texts(
            stored
                .dataset_edge_record_values
                .iter()
                .map(|row| parse(&row.edge_payload_json_text))
        ),
        sort_payload_value_texts(edge_payloads.iter().cloned()),
        "edge payloads differ from the export"
    );

    // Every other column is the payload field it copies.
    stored.dataset_node_record_values.iter().for_each(|row| {
        let payload = parse(&row.node_payload_json_text);
        assert_eq!(
            row.node_identity_key_text,
            write_canonical_json_text(&payload["node_target_identity"])
        );
        [
            (&row.node_qualified_path_text, "node_qualified_path_text"),
            (&row.node_input_category, "node_input_category"),
            (&row.node_item_kind_value, "node_item_kind_value"),
            (&row.node_owner_kind_value, "node_owner_kind_value"),
            (&row.node_source_relative_path, "node_source_relative_path"),
            (
                &row.node_effective_visibility_kind,
                "node_effective_visibility_kind",
            ),
            (&row.node_signature_text, "node_signature_text"),
        ]
        .iter()
        .for_each(|(column, name)| {
            assert_eq!(
                **column,
                read_optional_text_value(&payload[*name]),
                "{name}"
            );
        });
        assert_eq!(
            row.node_function_body_ancestry_flag,
            payload["node_function_body_ancestry_flag"].as_bool()
        );
        assert_eq!(
            row.node_reachable_outside_crate_flag,
            payload["node_reachable_outside_crate_flag"].as_bool()
        );
    });
    let node_keys = stored
        .dataset_node_record_values
        .iter()
        .map(|row| row.node_identity_key_text.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(node_keys.len(), 170);
    stored.dataset_edge_record_values.iter().for_each(|row| {
        let payload = parse(&row.edge_payload_json_text);
        let resolution = &payload["edge_resolution"];
        let site = &payload["edge_evidence_identity"]["site_occurrence_identity"];
        assert_eq!(
            Some(row.edge_relation_kind.as_str()),
            payload["edge_relation_kind"].as_str()
        );
        assert_eq!(
            row.source_node_identity_key,
            write_canonical_json_text(&payload["edge_source_target_identity"])
        );
        assert!(node_keys.contains(row.source_node_identity_key.as_str()));
        assert_eq!(
            row.target_node_identity_key,
            resolution.get("target").map(write_canonical_json_text)
        );
        assert!(
            row.target_node_identity_key
                .as_deref()
                .is_none_or(|key| node_keys.contains(key))
        );
        assert_eq!(
            Some(row.edge_resolution_kind_value.as_str()),
            resolution["kind"].as_str()
        );
        assert_eq!(
            row.edge_unresolved_reason_value,
            read_optional_text_value(&resolution["reason"])
        );
        assert_eq!(
            row.site_source_file_path,
            read_optional_text_value(&site["source_file_path"])
        );
        assert_eq!(row.site_byte_start_value, site["byte_range"][0].as_i64());
        assert_eq!(row.site_byte_end_value, site["byte_range"][1].as_i64());
        assert_eq!(
            row.edge_dispatch_metadata_text,
            read_optional_text_value(&payload["edge_dispatch_metadata_text"])
        );
    });
    let without_target = stored
        .dataset_edge_record_values
        .iter()
        .filter(|row| row.target_node_identity_key.is_none())
        .collect::<Vec<_>>();
    assert_eq!(without_target.len(), 1, "only `cast` has no target");
    assert!(
        without_target[0]
            .edge_dispatch_metadata_text
            .as_deref()
            .is_some_and(|text| text.contains("cast")),
        "the row without a target is the `cast` call"
    );
}
