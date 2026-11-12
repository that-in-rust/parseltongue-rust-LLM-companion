use crate::{OracleSemanticExportError, primary_path_export_rows::build_oracle_export_error};
use cfg::CfgOptions;
use ide_db::base_db::Env;
use serde_json::Value;

pub(super) fn apply_recorded_script_options(
    unit: &Value,
    rows: &[Value],
    _actual: &CfgOptions,
    _environment: &Env,
    options: CfgOptions,
) -> Result<(CfgOptions, Option<Value>), OracleSemanticExportError> {
    if rows
        .iter()
        .all(|row| row["script_package_identifier_text"] != unit["unit_cargo_package_identifier"])
    {
        return Ok((options, None));
    }
    Err(build_oracle_export_error(
        "SECONDARY script deltas not implemented",
    ))
}
