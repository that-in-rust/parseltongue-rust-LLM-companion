use crate::{build_graph_stage_failure, read_safe_source_bytes};
use parseltongue_core_row_types::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::{collections::BTreeSet, path::Path};

pub fn extend_sysroot_crate_graph(
    graph: &[StageTableRowRecord],
    contexts: &BTreeMap<String, UnitPreludeConfigurationInput>,
) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
    if contexts.is_empty() {
        return Ok(graph.to_vec());
    }
    let config = graph
        .iter()
        .find(|row| row.stage_owned_table_name == "workspace-config-state-records")
        .ok_or_else(|| build_graph_stage_failure("sysroot configuration missing"))?;
    let config: Value =
        serde_json::from_str(&config.stage_row_payload_text).map_err(build_graph_stage_failure)?;
    let sysroot = config["workspace_sysroot_directory_path"]
        .as_str()
        .ok_or_else(|| build_graph_stage_failure("sysroot directory missing"))?;
    let library = Path::new(sysroot).join("lib/rustlib/src/rust/library");
    let cfg: BTreeSet<String> =
        serde_json::from_value::<BTreeSet<String>>(config["workspace_target_cfg_values"].clone())
            .map_err(build_graph_stage_failure)?
            .into_iter()
            .filter(|value| value != "test" && !value.starts_with("feature="))
            .collect();
    let system=["core","alloc","std"].into_iter().map(|name|{
        let package=library.join(name);
        let (_,bytes)=read_safe_source_bytes(&package.join("Cargo.toml"))
            .map_err(|error|build_graph_stage_failure(format!("matching sysroot sources unavailable: {error}")))?;
        let manifest:toml::Value=toml::from_str(std::str::from_utf8(&bytes).map_err(build_graph_stage_failure)?).map_err(build_graph_stage_failure)?;
        let edition=manifest["package"].get("edition").and_then(toml::Value::as_str).unwrap_or("2015");
        let source=package.join(manifest.get("lib").and_then(|value|value.get("path")).and_then(toml::Value::as_str).unwrap_or("src/lib.rs"));
        let (source,source_bytes)=read_safe_source_bytes(&source).map_err(|error|build_graph_stage_failure(format!("matching sysroot sources unavailable: {error}")))?;
        let package_id=format!("sysroot+{}#{name}@0.0.0",package.display());
        let digest=blake3::hash(&bytes).to_hex().to_string();
        let key=blake3::hash(json!([&package_id,edition,&cfg,blake3::hash(&source_bytes).to_hex().as_str()]).to_string().as_bytes()).to_hex().to_string();
        Ok((name, json!({
            "unit_cargo_package_identifier":package_id,"unit_cargo_target_name":name,"unit_cargo_target_kind":"lib",
            "unit_compilation_role_kind":"target","unit_mapped_target_flag":false,"unit_sysroot_context_flag":true,
            "unit_compilation_identity_key":key,"unit_rust_edition_value":edition,"unit_source_root_path":source,
            "unit_package_root_path":package,"unit_target_cfg_values":cfg,"unit_enabled_feature_values":[],
            "unit_sysroot_manifest_digest":digest,"unit_metadata_environment_values":{},"unit_normal_dependency_rows":[],
            "unit_no_std_flag":true,"unit_no_core_flag":name=="core","unit_implicit_prelude_flag":true
        })))
    }).collect::<Result<BTreeMap<_,_>,StoreBackendFailureError>>()?;
    let link = |name: &str| {
        json!({
            "dependency_alias_name_value":name,"dependency_cargo_package_identifier":system[name]["unit_cargo_package_identifier"],
            "dependency_compilation_role_kind":"target","dependency_sysroot_context_flag":true
        })
    };
    let mut rows = graph
        .iter()
        .map(|row| {
            let Some(flags) = contexts.get(&row.stage_row_record_key) else {
                return Ok(row.clone());
            };
            let mut value: Value = serde_json::from_str(&row.stage_row_payload_text)
                .map_err(build_graph_stage_failure)?;
            value["unit_no_std_flag"] = json!(flags.unit_no_std_flag);
            value["unit_no_core_flag"] = json!(flags.unit_no_core_flag);
            value["unit_implicit_prelude_flag"] = json!(flags.unit_implicit_prelude_flag);
            let dependencies = value["unit_normal_dependency_rows"]
                .as_array_mut()
                .ok_or_else(|| build_graph_stage_failure("unit dependencies missing"))?;
            dependencies.extend(["core", "alloc", "std"].into_iter().map(link));
            Ok(StageTableRowRecord {
                stage_row_payload_text: value.to_string(),
                ..row.clone()
            })
        })
        .collect::<Result<Vec<_>, StoreBackendFailureError>>()?;
    for (name, mut value) in system.clone() {
        value["unit_normal_dependency_rows"] = json!(match name {
            "core" => vec![],
            "alloc" => vec![link("core")],
            _ => vec![link("core"), link("alloc")],
        });
        rows.push(StageTableRowRecord {
            stage_owned_table_name: "crate-dependency-graph-table".into(),
            stage_row_record_key: value["unit_compilation_identity_key"]
                .as_str()
                .ok_or_else(|| build_graph_stage_failure("sysroot unit key missing"))?
                .into(),
            stage_row_payload_text: value.to_string(),
        });
    }
    rows.sort_by(|left, right| {
        (&left.stage_owned_table_name, &left.stage_row_record_key)
            .cmp(&(&right.stage_owned_table_name, &right.stage_row_record_key))
    });
    Ok(rows)
}
