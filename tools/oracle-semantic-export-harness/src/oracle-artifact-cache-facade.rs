use crate::mapped_oracle_export_facade::{
    read_manifest_unit_records, validate_mapped_unit_manifest,
};
use crate::primary_path_export_rows::build_oracle_export_error;
use crate::{OracleSemanticExportError, OracleWorkspaceExportConfig};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};

#[derive(Default)]
pub(super) struct OracleArtifactCacheState {
    pub oracle_cached_macro_paths: BTreeMap<String, vfs::AbsPathBuf>,
    pub oracle_recorded_script_rows: Vec<Value>,
}

fn read_artifact_string_value<'a>(
    value: &'a Value,
    key: &str,
) -> Result<&'a str, OracleSemanticExportError> {
    value[key]
        .as_str()
        .ok_or_else(|| build_oracle_export_error(format!("invalid artifact string {key}")))
}

fn read_artifact_feature_values(
    value: &Value,
) -> Result<BTreeSet<String>, OracleSemanticExportError> {
    value
        .as_array()
        .ok_or_else(|| build_oracle_export_error("invalid artifact feature array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| build_oracle_export_error("invalid artifact feature value"))
        })
        .collect()
}

fn read_required_macro_units(manifest: &Value) -> Result<Vec<&Value>, OracleSemanticExportError> {
    Ok(read_manifest_unit_records(manifest)?
        .iter()
        .filter(|unit| {
            unit["unit_cargo_target_kind"] == "proc-macro"
                && unit["unit_compilation_role_kind"] == "host"
        })
        .collect())
}

pub(super) fn build_recorded_env_digest(environment: &BTreeMap<String, String>) -> String {
    blake3::hash(json!(environment).to_string().as_bytes())
        .to_hex()
        .to_string()
}

pub fn build_oracle_artifact_cache(
    request: &OracleWorkspaceExportConfig,
    manifest: &Value,
) -> Result<Value, OracleSemanticExportError> {
    validate_mapped_unit_manifest(request, manifest)?;
    let compiler = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .arg("--version")
        .output()
        .map_err(build_oracle_export_error)?;
    if !compiler.status.success()
        || std::str::from_utf8(&compiler.stdout)
            .map_err(build_oracle_export_error)?
            .trim()
            != env!("ORACLE_RUSTC_VERSION_TEXT")
    {
        return Err(build_oracle_export_error(
            "artifact compiler differs from pinned oracle",
        ));
    }
    let arguments = [
        "check",
        "--offline",
        "--locked",
        "--workspace",
        "--message-format=json",
    ];
    let output = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .current_dir(&request.workspace_source_root_path)
        .args(arguments)
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .map_err(build_oracle_export_error)?;
    let messages = std::str::from_utf8(&output.stdout)
        .map_err(build_oracle_export_error)?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<Value>(line).map_err(build_oracle_export_error))
        .collect::<Result<Vec<_>, _>>()?;
    if !output.status.success()
        || messages
            .iter()
            .any(|message| message["reason"] == "build-finished" && message["success"] == false)
    {
        let errors = messages
            .iter()
            .filter(|message| message["reason"] == "compiler-message")
            .filter_map(|message| message["message"]["rendered"].as_str())
            .collect::<Vec<_>>()
            .join("\n");
        return Err(build_oracle_export_error(format!(
            "offline artifact preparation failed: {}\n{errors}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let artifacts=read_required_macro_units(manifest)?.into_iter().map(|unit| {
        let candidates=messages.iter().filter(|message|
            message["reason"]=="compiler-artifact"
            && message["package_id"]==unit["unit_cargo_package_identifier"]
            && message["target"]["src_path"]==unit["unit_source_root_path"]
            && message["target"]["name"]==unit["unit_cargo_target_name"]
            && message["profile"]["test"]==false
            && message["target"]["kind"].as_array().is_some_and(|kinds|kinds.iter().any(|kind|kind=="proc-macro")))
            .collect::<Vec<_>>();
        let [artifact]=candidates.as_slice() else {
            return Err(build_oracle_export_error(format!("proc-macro artifact match count {}: {}",candidates.len(),unit["unit_cargo_package_identifier"])));
        };
        if read_artifact_feature_values(&artifact["features"])?
            !=read_artifact_feature_values(&unit["unit_enabled_feature_values"])? {
            return Err(build_oracle_export_error(format!("proc-macro artifact features differ: {}",unit["unit_cargo_package_identifier"])));
        }
        let binaries=artifact["filenames"].as_array().ok_or_else(||build_oracle_export_error("artifact filenames missing"))?
            .iter().filter_map(Value::as_str).filter(|path|
                matches!(Path::new(path).extension().and_then(|extension|extension.to_str()),Some("so"|"dylib"|"dll"))).collect::<Vec<_>>();
        let [binary]=binaries.as_slice() else {
            return Err(build_oracle_export_error("expected exactly one proc-macro dylib"));
        };
        Ok(json!({
            "artifact_compilation_unit_key":unit["unit_compilation_identity_key"],
            "artifact_semantic_identity_record":unit["unit_semantic_identity_record"],
            "artifact_source_root_path":unit["unit_source_root_path"],
            "artifact_enabled_feature_values":artifact["features"],
            "artifact_binary_absolute_path":binary,
            "artifact_binary_digest_full":blake3::hash(&fs::read(binary).map_err(build_oracle_export_error)?).to_hex().to_string(),
        }))
    }).collect::<Result<Vec<_>,OracleSemanticExportError>>()?;
    let scripts = messages
        .iter()
        .filter(|message| message["reason"] == "build-script-executed")
        .map(|message| {
            let environment =
                serde_json::from_value::<Vec<(String, String)>>(message["env"].clone())
                    .map_err(build_oracle_export_error)?
                    .into_iter()
                    .collect::<BTreeMap<_, _>>();
            let cfgs = serde_json::from_value::<BTreeSet<String>>(message["cfgs"].clone())
                .map_err(build_oracle_export_error)?;
            Ok(json!({
                "script_package_identifier_text":message["package_id"],
                "script_emitted_cfg_values":cfgs,
                "script_emitted_env_names":environment.keys().collect::<Vec<_>>(),
                "script_emitted_env_digest":build_recorded_env_digest(&environment),
                "script_generated_output_path":message["out_dir"],
            }))
        })
        .collect::<Result<Vec<_>, OracleSemanticExportError>>()?;
    validate_mapped_unit_manifest(request, manifest)?;
    Ok(json!({
        "cache_manifest_version_value":1,
        "cache_scope_identity_record":manifest["manifest_scope_identity_record"],
        "cache_source_revision_hash":env!("ORACLE_SOURCE_COMMIT_HASH"),
        "cache_compiler_version_text":env!("ORACLE_RUSTC_VERSION_TEXT"),
        "cache_preparation_command_values":arguments,
        "cache_macro_artifact_rows":artifacts,"cache_build_script_rows":scripts,
    }))
}

pub(super) fn read_oracle_artifact_cache(
    request: &OracleWorkspaceExportConfig,
    manifest: Option<&Value>,
) -> Result<OracleArtifactCacheState, OracleSemanticExportError> {
    let Some(path) = &request.oracle_macro_cache_manifest else {
        if manifest.is_some_and(|manifest| {
            read_required_macro_units(manifest).is_ok_and(|units| !units.is_empty())
        }) {
            return Err(build_oracle_export_error(
                "mapped proc-macro units require an artifact cache",
            ));
        }
        return Ok(OracleArtifactCacheState::default());
    };
    let manifest = manifest.ok_or_else(|| {
        build_oracle_export_error("artifact cache requires mapped manifest scope")
    })?;
    let cache: Value = serde_json::from_slice(&fs::read(path).map_err(build_oracle_export_error)?)
        .map_err(build_oracle_export_error)?;
    if cache["cache_manifest_version_value"] != 1
        || cache["cache_scope_identity_record"] != manifest["manifest_scope_identity_record"]
        || cache["cache_source_revision_hash"] != env!("ORACLE_SOURCE_COMMIT_HASH")
        || cache["cache_compiler_version_text"] != env!("ORACLE_RUSTC_VERSION_TEXT")
    {
        return Err(build_oracle_export_error(
            "artifact cache scope/compiler/pin differs",
        ));
    }
    let rows = cache["cache_macro_artifact_rows"]
        .as_array()
        .ok_or_else(|| build_oracle_export_error("artifact cache rows missing"))?;
    let required = read_required_macro_units(manifest)?;
    if rows.len() != required.len() {
        return Err(build_oracle_export_error(
            "artifact cache unit count differs",
        ));
    }
    let paths = required
        .into_iter()
        .try_fold(BTreeMap::new(), |mut paths, unit| {
            let candidates = rows
                .iter()
                .filter(|row| {
                    row["artifact_compilation_unit_key"] == unit["unit_compilation_identity_key"]
                })
                .collect::<Vec<_>>();
            let [row] = candidates.as_slice() else {
                return Err(build_oracle_export_error(
                    "artifact cache unit missing or ambiguous",
                ));
            };
            if row["artifact_semantic_identity_record"] != unit["unit_semantic_identity_record"]
                || row["artifact_source_root_path"] != unit["unit_source_root_path"]
                || read_artifact_feature_values(&row["artifact_enabled_feature_values"])?
                    != read_artifact_feature_values(&unit["unit_enabled_feature_values"])?
            {
                return Err(build_oracle_export_error(
                    "artifact cache unit configuration differs",
                ));
            }
            let binary = read_artifact_string_value(row, "artifact_binary_absolute_path")?;
            let absolute = vfs::AbsPathBuf::try_from(binary)
                .map_err(|_| build_oracle_export_error("artifact path is not absolute UTF-8"))?;
            if row["artifact_binary_digest_full"]
                != blake3::hash(&fs::read(binary).map_err(build_oracle_export_error)?)
                    .to_hex()
                    .to_string()
            {
                return Err(build_oracle_export_error(
                    "artifact cache dylib digest differs",
                ));
            }
            if paths
                .insert(
                    read_artifact_string_value(unit, "unit_source_root_path")?.to_owned(),
                    absolute,
                )
                .is_some()
            {
                return Err(build_oracle_export_error(
                    "artifact cache source context ambiguous",
                ));
            }
            Ok(paths)
        })?;
    let scripts = cache["cache_build_script_rows"]
        .as_array()
        .ok_or_else(|| build_oracle_export_error("artifact cache build-script rows missing"))?
        .clone();
    Ok(OracleArtifactCacheState {
        oracle_cached_macro_paths: paths,
        oracle_recorded_script_rows: scripts,
    })
}
