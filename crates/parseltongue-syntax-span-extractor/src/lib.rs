use std::collections::BTreeSet;
use parseltongue_core_row_types::{StageTableRowRecord, StoreBackendFailureError};

pub struct SyntaxFileInputData {
    pub file_source_absolute_path: String,
    pub file_source_content_bytes: Vec<u8>,
    pub unit_compilation_identity_key: String,
    pub unit_rust_edition_value: String,
    pub unit_target_cfg_values: BTreeSet<String>,
    pub file_module_directory_path: String,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SourceClosureFileRequest {
    pub source_request_path_candidates: Vec<String>,
    pub source_request_context_kind: String,
}

#[derive(Debug)]
pub struct SyntaxFileExtractionResult {
    pub stage_unit_output_rows: Vec<StageTableRowRecord>,
    pub source_closure_file_requests: Vec<SourceClosureFileRequest>,
}

pub fn extract_file_syntax_rows(_syntax_file_input_data: &SyntaxFileInputData)
    -> Result<SyntaxFileExtractionResult, StoreBackendFailureError> { todo!() }

pub fn evaluate_cfg_expression_truth(_cfg_expression_source_text: &str, _unit_target_cfg_values: &BTreeSet<String>)
    -> Result<bool, StoreBackendFailureError> { todo!() }
