use crate::{OracleSemanticExportError, OracleWorkspaceExportConfig};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleConfigurationKindValue {
    Primary,
    Secondary,
}

pub fn export_mapped_semantic_paths(
    _request: &OracleWorkspaceExportConfig,
    _manifest: &Value,
    _configuration: OracleConfigurationKindValue,
) -> Result<Value, OracleSemanticExportError> {
    Err(OracleSemanticExportError {
        error_context_detail_text: "exact mapped-unit exports not implemented".into(),
    })
}
