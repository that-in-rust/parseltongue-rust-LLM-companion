//! The two exported tables the loader reads, and every way a load can fail (REQ-RUST-067.0).

use std::path::PathBuf;

/// Which exported table a file holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GraphExportFileKind {
    NodeRows,
    EdgeRows,
}

impl GraphExportFileKind {
    /// The stage table `export-published-stage-rows` names in the file's header.
    pub fn read_export_table_name(self) -> &'static str {
        match self {
            Self::NodeRows => "item-effective-visibility-records",
            Self::EdgeRows => "edge-evidence-record-table",
        }
    }
}

impl std::fmt::Display for GraphExportFileKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::NodeRows => "node export",
            Self::EdgeRows => "edge export",
        })
    }
}

/// Why no DuckDB file was written. One variant per check of REQ-RUST-067.0.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphDatasetLoadError {
    #[error("{export_file_kind_value}: the first line is not an export header")]
    ExportHeaderMissing {
        export_file_kind_value: GraphExportFileKind,
    },
    #[error("{export_file_kind_value}: the header names table {export_owned_table_name}")]
    ExportTableNameMismatch {
        export_file_kind_value: GraphExportFileKind,
        export_owned_table_name: String,
    },
    #[error(
        "{export_file_kind_value} line {export_line_number_value}: not JSON: {error_context_detail_text}"
    )]
    ExportLineNotJson {
        export_file_kind_value: GraphExportFileKind,
        export_line_number_value: usize,
        error_context_detail_text: String,
    },
    #[error(
        "{export_file_kind_value} line {export_line_number_value}: not a row with an identity text and a payload object"
    )]
    ExportRowShapeInvalid {
        export_file_kind_value: GraphExportFileKind,
        export_line_number_value: usize,
    },
    #[error(
        "{export_file_kind_value} line {export_line_number_value}: payload field {payload_field_name_text} is missing or has the wrong type"
    )]
    PayloadFieldShapeInvalid {
        export_file_kind_value: GraphExportFileKind,
        export_line_number_value: usize,
        payload_field_name_text: &'static str,
    },
    #[error("{export_file_kind_value}: the file has a header and no row")]
    ExportFileEmpty {
        export_file_kind_value: GraphExportFileKind,
    },
    #[error(
        "{export_file_kind_value} line {export_line_number_value}: run generation {run_generation_digest_value} is not the node header's"
    )]
    RunGenerationMismatch {
        export_file_kind_value: GraphExportFileKind,
        export_line_number_value: usize,
        run_generation_digest_value: String,
    },
    #[error(
        "node export line {export_line_number_value}: identity text {export_row_identity_text} is not the row's node_target_identity"
    )]
    NodeIdentityTextMismatch {
        export_line_number_value: usize,
        export_row_identity_text: String,
    },
    #[error(
        "{export_file_kind_value} line {export_line_number_value}: identity {export_row_identity_text} is on an earlier row too"
    )]
    RowIdentityDuplicated {
        export_file_kind_value: GraphExportFileKind,
        export_line_number_value: usize,
        export_row_identity_text: String,
    },
    #[error(
        "edge export line {export_line_number_value}: source {node_identity_key_text} is not a node"
    )]
    EdgeSourceNotNode {
        export_line_number_value: usize,
        node_identity_key_text: String,
    },
    #[error(
        "edge export line {export_line_number_value}: target {node_identity_key_text} is not a node"
    )]
    EdgeTargetNotNode {
        export_line_number_value: usize,
        node_identity_key_text: String,
    },
    #[error(
        "edge export line {export_line_number_value}: the resolution has neither one target nor a reason alone"
    )]
    EdgeResolutionShapeInvalid { export_line_number_value: usize },
    #[error("{}: the output path exists", dataset_output_file_path.display())]
    DatasetOutputPathExists { dataset_output_file_path: PathBuf },
    #[error("{}: {error_context_detail_text}", dataset_file_path_value.display())]
    DatasetFileAccessFailed {
        dataset_file_path_value: PathBuf,
        error_context_detail_text: String,
    },
    #[error("DuckDB step {dataset_store_step_name}: {error_context_detail_text}")]
    DatasetStoreStepFailed {
        dataset_store_step_name: &'static str,
        error_context_detail_text: String,
    },
    #[error("the rows read back from the written file differ from the rows checked")]
    DatasetReadbackRowsDiffer,
}
