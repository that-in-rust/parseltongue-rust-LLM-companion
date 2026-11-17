//! `parseltongue-graph-dataset-loader`: the exported graph JSON of one run into a DuckDB file.

use parseltongue_graph_dataset_loader::load_graph_export_files;
use std::path::PathBuf;
use std::process::ExitCode;

const LOADER_USAGE_HELP_TEXT: &str = "usage: parseltongue-graph-dataset-loader \
--node-export-file-path <nodes.json> --edge-export-file-path <edges.json> \
--dataset-output-file-path <graph.duckdb>";

/// The value that follows the flag `name`, when the flag is given once with a value.
fn read_loader_flag_value(arguments: &[String], name: &str) -> Option<PathBuf> {
    let mut values = arguments
        .windows(2)
        .filter(|pair| pair[0] == name)
        .map(|pair| PathBuf::from(&pair[1]));
    values.next().filter(|_| values.next().is_none())
}

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let flag = |name| read_loader_flag_value(&arguments, name);
    let (Some(nodes), Some(edges), Some(output), 6) = (
        flag("--node-export-file-path"),
        flag("--edge-export-file-path"),
        flag("--dataset-output-file-path"),
        arguments.len(),
    ) else {
        eprintln!("{LOADER_USAGE_HELP_TEXT}");
        return ExitCode::from(2);
    };
    match load_graph_export_files(&nodes, &edges, &output) {
        Ok(info) => {
            println!(
                "{}",
                serde_json::json!({
                    "dataset_output_file_path": output,
                    "run_generation_digest_value": info.run_generation_digest_value,
                    "node_row_count_value": info.node_row_count_value,
                    "edge_row_count_value": info.edge_row_count_value,
                    "node_file_digest_sha256": info.node_file_digest_sha256,
                    "edge_file_digest_sha256": info.edge_file_digest_sha256,
                })
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("graph dataset not written: {error}");
            ExitCode::FAILURE
        }
    }
}
