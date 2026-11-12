use crate::primary_path_export_rows::{
    build_oracle_export_error, export_configured_semantic_paths,
};
use crate::{OracleSemanticExportError, OracleWorkspaceExportConfig};
use ide_db::{
    RootDatabase,
    base_db::{all_crates, salsa::Setter},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleConfigurationKindValue {
    Primary,
    Secondary,
}

pub fn export_mapped_semantic_paths(
    request: &OracleWorkspaceExportConfig,
    manifest: &Value,
    configuration: OracleConfigurationKindValue,
) -> Result<Value, OracleSemanticExportError> {
    validate_mapped_unit_manifest(request, manifest)?;
    export_configured_semantic_paths(request, Some(manifest), configuration)
}

fn read_manifest_string_value<'a>(
    value: &'a Value,
    key: &str,
) -> Result<&'a str, OracleSemanticExportError> {
    value[key]
        .as_str()
        .ok_or_else(|| build_oracle_export_error(format!("invalid manifest string {key}")))
}

pub(super) fn read_manifest_unit_records(
    manifest: &Value,
) -> Result<&[Value], OracleSemanticExportError> {
    manifest["manifest_unit_context_rows"]
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| build_oracle_export_error("invalid manifest unit records"))
}

pub(super) fn validate_mapped_unit_manifest(
    request: &OracleWorkspaceExportConfig,
    manifest: &Value,
) -> Result<(), OracleSemanticExportError> {
    if !request.oracle_default_features_flag || !request.oracle_enabled_feature_values.is_empty() {
        return Err(build_oracle_export_error(
            "manifest Cargo options differ from fixed mapping policy",
        ));
    }
    if manifest["manifest_format_version_value"] != 1 {
        return Err(build_oracle_export_error("unsupported manifest version"));
    }
    if manifest["manifest_mapped_target_policy"]
        != json!({
            "policy_target_kind_values":["lib","bin"],"policy_test_targets_flag":false,
            "policy_dependency_kind_values":["normal"],
            "policy_excluded_target_kinds":["test","bench","example","custom-build","proc-macro"],
            "policy_support_mapping_flag":false,
        })
    {
        return Err(build_oracle_export_error(
            "unsupported mapped-target policy",
        ));
    }
    if manifest["manifest_target_context_kind"] != "compiler_default"
        || request.oracle_target_triple_value.is_some()
    {
        return Err(build_oracle_export_error(
            "manifest target context differs from compiler default",
        ));
    }
    if manifest["manifest_compiler_version_text"] != env!("ORACLE_RUSTC_VERSION_TEXT") {
        return Err(build_oracle_export_error(
            "manifest compiler identity differs from oracle",
        ));
    }
    if manifest["manifest_parser_source_record"]["parser_source_commit_hash"]
        != env!("ORACLE_SOURCE_COMMIT_HASH")
    {
        return Err(build_oracle_export_error(
            "manifest parser identity differs from oracle",
        ));
    }
    let root = request
        .workspace_source_root_path
        .canonicalize()
        .map_err(build_oracle_export_error)?;
    let declared = Path::new(read_manifest_string_value(
        &manifest["manifest_configuration_state_record"],
        "workspace_source_root_path",
    )?)
    .canonicalize()
    .map_err(build_oracle_export_error)?;
    if root != declared {
        return Err(build_oracle_export_error(
            "manifest workspace root differs from request",
        ));
    }
    for field in [
        "run_generation_digest_value",
        "stage_attempt_sequence_value",
    ] {
        if read_manifest_string_value(&manifest["manifest_scope_identity_record"], field)?
            .is_empty()
        {
            return Err(build_oracle_export_error(
                "manifest generation identity is empty",
            ));
        }
    }
    let mut identities = BTreeSet::new();
    let mut semantic_identities = BTreeSet::new();
    for unit in read_manifest_unit_records(manifest)? {
        let identity = read_manifest_string_value(unit, "unit_compilation_identity_key")?;
        if !identities.insert(identity) {
            return Err(build_oracle_export_error(
                "duplicate compilation unit in manifest",
            ));
        }
        if !matches!(
            unit["unit_compilation_role_kind"].as_str(),
            Some("host" | "target")
        ) {
            return Err(build_oracle_export_error("invalid compilation role"));
        }
        let _: syntax::Edition = read_manifest_string_value(unit, "unit_rust_edition_value")?
            .parse()
            .map_err(build_oracle_export_error)?;
        if (unit["unit_mapped_target_flag"] == true || unit["unit_sysroot_context_flag"] == true)
            && (unit["unit_compilation_role_kind"] != "target"
                || (unit["unit_mapped_target_flag"] == true
                    && !matches!(unit["unit_cargo_target_kind"].as_str(), Some("lib" | "bin"))))
        {
            return Err(build_oracle_export_error(
                "mapped compilation unit violates target policy",
            ));
        }
        let expected = blake3::hash(
            json!([
                unit["unit_target_cfg_values"],
                unit["unit_enabled_feature_values"],
                unit["unit_rust_edition_value"],
                unit["unit_compilation_role_kind"],
            ])
            .to_string()
            .as_bytes(),
        )
        .to_hex()
        .to_string();
        if unit["unit_semantic_identity_record"]["unit_configuration_digest_full"] != expected {
            return Err(build_oracle_export_error(
                "manifest compilation cfg identity differs",
            ));
        }
        if [
            "unit_cargo_package_identifier",
            "unit_cargo_target_name",
            "unit_cargo_target_kind",
            "unit_compilation_role_kind",
        ]
        .iter()
        .any(|key| unit["unit_semantic_identity_record"][*key] != unit[*key])
        {
            return Err(build_oracle_export_error(
                "manifest semantic compilation identity differs",
            ));
        }
        if !semantic_identities.insert(unit["unit_semantic_identity_record"].to_string()) {
            return Err(build_oracle_export_error(
                "duplicate semantic compilation identity",
            ));
        }
        build_manifest_cfg_options(unit)?;
        let closure = unit["unit_source_closure_rows"]
            .as_array()
            .ok_or_else(|| build_oracle_export_error("invalid source closure records"))?;
        let source = read_manifest_string_value(unit, "unit_source_root_path")?;
        if (unit["unit_mapped_target_flag"] == true
            || unit["unit_sysroot_context_flag"] == true
            || !closure.is_empty())
            && !closure
                .iter()
                .any(|file| file["file_source_absolute_path"] == source)
        {
            return Err(build_oracle_export_error(
                "source closure omits compilation root",
            ));
        }
        for file in closure {
            let path = read_manifest_string_value(file, "file_source_absolute_path")?;
            let bytes = std::fs::read(path).map_err(|error| {
                build_oracle_export_error(format!("source closure {path}: {error}"))
            })?;
            if file["file_content_digest_full"] != blake3::hash(&bytes).to_hex().to_string() {
                return Err(build_oracle_export_error(format!(
                    "source closure digest changed: {path}"
                )));
            }
            std::str::from_utf8(&bytes).map_err(|error| {
                build_oracle_export_error(format!("source closure {path}: {error}"))
            })?;
        }
    }
    Ok(())
}

fn build_manifest_cfg_options(unit: &Value) -> Result<cfg::CfgOptions, OracleSemanticExportError> {
    let mut options = cfg::CfgOptions::default();
    for flag in unit["unit_target_cfg_values"]
        .as_array()
        .ok_or_else(|| build_oracle_export_error("manifest cfg values missing"))?
    {
        let flag = flag
            .as_str()
            .ok_or_else(|| build_oracle_export_error("invalid cfg value"))?;
        if let Some((key, value)) = flag.split_once('=') {
            let value: String =
                serde_json::from_str(value.trim()).map_err(build_oracle_export_error)?;
            options.insert_key_value(hir::Symbol::intern(key.trim()), hir::Symbol::intern(&value));
        } else {
            if flag == "test" || flag == "rust_analyzer" {
                return Err(build_oracle_export_error(
                    "manifest enables excluded test/oracle cfg",
                ));
            }
            options.insert_atom(hir::Symbol::intern(flag));
        }
    }
    for feature in unit["unit_enabled_feature_values"]
        .as_array()
        .ok_or_else(|| build_oracle_export_error("manifest feature values missing"))?
    {
        options.insert_key_value(
            hir::Symbol::intern("feature"),
            hir::Symbol::intern(
                feature
                    .as_str()
                    .ok_or_else(|| build_oracle_export_error("invalid feature value"))?,
            ),
        );
    }
    Ok(options)
}

pub(super) fn apply_mapped_unit_configuration(
    database: &mut RootDatabase,
    files: &vfs::Vfs,
    packages: &BTreeMap<(String, String), String>,
    manifest: &Value,
    scripts: Option<&[Value]>,
) -> Result<Vec<Value>, OracleSemanticExportError> {
    let crates = all_crates(database);
    let mut deltas = Vec::new();
    let mut matched = BTreeSet::new();
    for unit in read_manifest_unit_records(manifest)?.iter().filter(|unit| {
        unit["unit_mapped_target_flag"] == true || unit["unit_sysroot_context_flag"] == true
    }) {
        let source = read_manifest_string_value(unit, "unit_source_root_path")?;
        let candidates = crates
            .iter()
            .copied()
            .filter(|krate| {
                let data = krate.data(database);
                let name = krate
                    .extra_data(database)
                    .display_name
                    .as_ref()
                    .map(|name| name.canonical_name().as_str());
                let package_name = match &data.origin {
                    ide_db::base_db::CrateOrigin::Local {
                        name: Some(name), ..
                    }
                    | ide_db::base_db::CrateOrigin::Library { name, .. }
                    | ide_db::base_db::CrateOrigin::Rustc { name } => name.as_str(),
                    _ => name.unwrap_or(""),
                };
                files.file_path(data.root_file_id).to_string() == source
                    && name == unit["unit_cargo_target_name"].as_str()
                    && data.target_kind.as_cargo_target().unwrap_or("lib")
                        == unit["unit_cargo_target_kind"].as_str().unwrap_or("")
                    && packages
                        .get(&(source.into(), package_name.into()))
                        .is_some_and(|package| {
                            Some(package.as_str()) == unit["unit_cargo_package_identifier"].as_str()
                        })
            })
            .collect::<Vec<_>>();
        let [krate] = candidates.as_slice() else {
            return Err(build_oracle_export_error(format!(
                "mapped compilation unit match count {}: {source}",
                candidates.len()
            )));
        };
        if !matched.insert(*krate) {
            return Err(build_oracle_export_error(
                "multiple manifest contexts map to one oracle compilation unit",
            ));
        }
        if krate.data(database).edition.to_string()
            != unit["unit_rust_edition_value"].as_str().unwrap_or("")
        {
            return Err(build_oracle_export_error(
                "mapped compilation unit edition differs",
            ));
        }
        let options = build_manifest_cfg_options(unit)?;
        let (options, delta) = match scripts {
            Some(rows) => crate::oracle_build_script_deltas::apply_recorded_script_options(
                unit,
                rows,
                krate.cfg_options(database),
                krate.env(database),
                options,
            )?,
            None => (options, None),
        };
        deltas.extend(delta);
        krate.set_cfg_options(database).to(options);
    }
    Ok(deltas)
}

pub(super) fn check_mapped_compilation_record(record: &Value, unit: &Value) -> bool {
    [
        "unit_source_root_path",
        "unit_cargo_package_identifier",
        "unit_cargo_target_name",
        "unit_cargo_target_kind",
        "unit_rust_edition_value",
    ]
    .iter()
    .all(|key| record[*key] == unit[*key])
}

pub(super) fn append_mapped_context_records(value: &mut Value, units: &[Value]) {
    match value {
        Value::Object(fields) => {
            if fields.contains_key("unit_cargo_package_identifier")
                && let Some(unit) = units.iter().find(|unit| {
                    (unit["unit_mapped_target_flag"] == true
                        || unit["unit_sysroot_context_flag"] == true)
                        && [
                            "unit_source_root_path",
                            "unit_cargo_package_identifier",
                            "unit_cargo_target_name",
                            "unit_cargo_target_kind",
                        ]
                        .iter()
                        .all(|key| fields.get(*key) == unit.get(*key))
                })
            {
                fields.insert(
                    "unit_semantic_identity_record".into(),
                    unit["unit_semantic_identity_record"].clone(),
                );
            }
            fields
                .values_mut()
                .for_each(|value| append_mapped_context_records(value, units));
        }
        Value::Array(values) => values
            .iter_mut()
            .for_each(|value| append_mapped_context_records(value, units)),
        _ => {}
    }
}
