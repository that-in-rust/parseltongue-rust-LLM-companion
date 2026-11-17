//! REQ-RUST-067.0: the loader executable writes no file when it is called wrongly.

#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::process::{Command, Output};

fn run_loader_with_arguments(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_parseltongue-graph-dataset-loader"))
        .args(arguments)
        .output()
        .unwrap()
}

fn list_directory_entry_names(directory: &Path) -> Vec<String> {
    std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect()
}

/// REQ-RUST-067.0 A4: a missing flag, a flag given twice and a flag with no value each exit
/// with 2, print the usage text and create nothing.
#[test]
fn test_wrong_flag_usage_exit() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("graph.duckdb");
    let output_text = output.to_str().unwrap();
    [
        vec![],
        vec!["--node-export-file-path", "nodes.json"],
        vec![
            "--node-export-file-path",
            "nodes.json",
            "--node-export-file-path",
            "nodes.json",
            "--dataset-output-file-path",
            output_text,
        ],
        vec![
            "--node-export-file-path",
            "nodes.json",
            "--edge-export-file-path",
            "edges.json",
            "--dataset-output-file-path",
        ],
    ]
    .iter()
    .for_each(|arguments| {
        let result = run_loader_with_arguments(arguments);
        assert_eq!(result.status.code(), Some(2), "{arguments:?}");
        assert!(String::from_utf8_lossy(&result.stderr).starts_with("usage: "));
        assert!(result.stdout.is_empty());
    });
    assert!(list_directory_entry_names(directory.path()).is_empty());
}

/// REQ-RUST-067.0 A4: an export file that cannot be read exits with 1, names the file and
/// creates nothing.
#[test]
fn test_missing_export_file_exit() {
    let directory = tempfile::tempdir().unwrap();
    let path = |name: &str| directory.path().join(name);
    let (nodes, edges, output) = (path("nodes.json"), path("edges.json"), path("graph.duckdb"));
    let result = run_loader_with_arguments(&[
        "--node-export-file-path",
        nodes.to_str().unwrap(),
        "--edge-export-file-path",
        edges.to_str().unwrap(),
        "--dataset-output-file-path",
        output.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("nodes.json"));
    assert!(result.stdout.is_empty());
    assert!(list_directory_entry_names(directory.path()).is_empty());
}
