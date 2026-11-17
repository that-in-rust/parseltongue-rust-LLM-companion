//! Loads the graph JSON a `parseltongue` run exports into one DuckDB file (REQ-RUST-067.0).
//!
//! The input is the two files `export-published-stage-rows` writes: the node table
//! (`item-effective-visibility-records`) and the edge table (`edge-evidence-record-table`).
//! The loader never writes to them. Parsing and checking are pure functions; only
//! `write_graph_dataset_file`, `read_graph_dataset_file` and `load_graph_export_files`
//! touch DuckDB and the file system.

#[path = "graph-dataset-load-error.rs"]
mod graph_dataset_load_error;
#[path = "graph-export-row-checks.rs"]
mod graph_export_row_checks;
#[path = "graph-export-row-parser.rs"]
mod graph_export_row_parser;

pub use graph_dataset_load_error::{GraphDatasetLoadError, GraphExportFileKind};
pub use graph_export_row_checks::{
    GraphDatasetEdgeRecord, GraphDatasetInfoRecord, GraphDatasetNodeRecord, GraphDatasetRowTables,
    build_graph_dataset_tables, check_graph_export_rows,
};
pub use graph_export_row_parser::{
    GraphExportFileRows, GraphExportRowRecord, parse_graph_export_text, write_canonical_json_text,
};
