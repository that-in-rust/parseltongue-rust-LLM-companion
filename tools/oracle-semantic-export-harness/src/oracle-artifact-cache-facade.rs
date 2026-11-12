use crate::{OracleSemanticExportError, OracleWorkspaceExportConfig};
use serde_json::Value;

pub fn build_oracle_artifact_cache(
    _request: &OracleWorkspaceExportConfig,
    _manifest: &Value,
) -> Result<Value, OracleSemanticExportError> {
    Err(OracleSemanticExportError {
        error_context_detail_text: "oracle artifact cache not implemented".into(),
    })
}
