use std::{fmt, path::PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleWorkspaceExportConfig {
    pub workspace_source_root_path: PathBuf,
    pub oracle_macro_cache_manifest: Option<PathBuf>,
    pub oracle_target_triple_value: Option<String>,
    pub oracle_enabled_feature_values: Vec<String>,
    pub oracle_default_features_flag: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleSemanticExportError {
    pub error_context_detail_text: String,
}

impl fmt::Display for OracleSemanticExportError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str(&self.error_context_detail_text)
    }
}

impl std::error::Error for OracleSemanticExportError {}

pub fn export_primary_semantic_paths(
    _oracle_workspace_export_config: &OracleWorkspaceExportConfig,
) -> Result<Vec<serde_json::Value>, OracleSemanticExportError> {
    todo!()
}
