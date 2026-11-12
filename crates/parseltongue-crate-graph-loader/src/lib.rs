#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use parseltongue_core_row_types::{
    MAPPED_TARGET_FLAG_FIELD, RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord,
    StageUnitWriteRequest, StoreBackendFailureError, evaluate_cfg_expression_truth,
};
use parseltongue_table_store_traits::StageTableStorePort;
#[path = "sysroot-compilation-context-rows.rs"]
mod sysroot_compilation_context_rows;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
pub use sysroot_compilation_context_rows::extend_sysroot_crate_graph;

pub trait WorkspaceCommandRunPort {
    fn run_workspace_command_output(
        &self,
        workspace_root_directory_path: &Path,
        workspace_command_argument_values: &[&str],
    ) -> Result<Vec<u8>, StoreBackendFailureError>;
}
pub struct OfflineWorkspaceCommandRunner;
pub fn build_graph_stage_failure(cause: impl std::fmt::Display) -> StoreBackendFailureError {
    StoreBackendFailureError {
        error_context_detail_text: format!("CrateGraphLoadError: {cause}"),
    }
}
impl WorkspaceCommandRunPort for OfflineWorkspaceCommandRunner {
    fn run_workspace_command_output(
        &self,
        root: &Path,
        args: &[&str],
    ) -> Result<Vec<u8>, StoreBackendFailureError> {
        if !matches!(
            args,
            ["cargo", "metadata", "--offline"]
                | ["rustc", "--print", "cfg"]
                | ["rustc", "--print", "sysroot"]
        ) {
            return Err(build_graph_stage_failure("executable request denied"));
        }
        let output = std::process::Command::new(args[0])
            .args(&args[1..])
            .current_dir(root)
            .env("CARGO_NET_OFFLINE", "true")
            .output()
            .map_err(build_graph_stage_failure)?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(build_graph_stage_failure(format!(
                "{}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }
}

fn read_json_string_field<'a>(
    value: &'a Value,
    key: &str,
) -> Result<&'a str, StoreBackendFailureError> {
    value[key]
        .as_str()
        .ok_or_else(|| build_graph_stage_failure(format!("missing metadata field {key}")))
}
fn read_json_array_field<'a>(
    value: &'a Value,
    key: &str,
) -> Result<&'a Vec<Value>, StoreBackendFailureError> {
    value[key]
        .as_array()
        .ok_or_else(|| build_graph_stage_failure(format!("missing metadata array {key}")))
}
fn check_library_target_kind(target: &Value) -> bool {
    target["kind"].as_array().is_some_and(|kinds| {
        kinds.iter().any(|kind| {
            matches!(
                kind.as_str(),
                Some("lib" | "rlib" | "dylib" | "cdylib" | "staticlib")
            )
        })
    })
}
fn check_proc_macro_package(package: &Value) -> bool {
    package["targets"].as_array().is_some_and(|targets| {
        targets.iter().any(|target| {
            target["kind"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|k| k == "proc-macro"))
        })
    })
}

pub fn load_workspace_crate_graph(
    root: &Path,
    commands: &dyn WorkspaceCommandRunPort,
) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
    let lock = root.join("Cargo.lock");
    let original = std::fs::read(&lock).map_err(build_graph_stage_failure)?;
    let metadata = commands.run_workspace_command_output(root, &["cargo", "metadata", "--offline"]);
    let after = std::fs::read(&lock).map_err(build_graph_stage_failure);
    if after.as_ref().ok() != Some(&original) {
        std::fs::write(&lock, &original).map_err(build_graph_stage_failure)?;
        return Err(build_graph_stage_failure("Cargo.lock mutation restored"));
    }
    let bytes = metadata?;
    let _: cargo_metadata::Metadata =
        serde_json::from_slice(&bytes).map_err(build_graph_stage_failure)?;
    let metadata: Value = serde_json::from_slice(&bytes).map_err(build_graph_stage_failure)?;
    let cfg_bytes = commands.run_workspace_command_output(root, &["rustc", "--print", "cfg"])?;
    let cfg: BTreeSet<String> = String::from_utf8(cfg_bytes)
        .map_err(build_graph_stage_failure)?
        .lines()
        .filter(|line| *line != "test")
        .map(str::to_owned)
        .collect();
    let sysroot = String::from_utf8(
        commands.run_workspace_command_output(root, &["rustc", "--print", "sysroot"])?,
    )
    .map_err(build_graph_stage_failure)?;
    let packages: BTreeMap<String, Value> = read_json_array_field(&metadata, "packages")?
        .iter()
        .map(|p| Ok((read_json_string_field(p, "id")?.to_owned(), p.clone())))
        .collect::<Result<_, StoreBackendFailureError>>()?;
    let members: BTreeSet<String> = read_json_array_field(&metadata, "workspace_members")?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    let nodes: BTreeMap<String, Value> = read_json_array_field(&metadata["resolve"], "nodes")?
        .iter()
        .map(|p| Ok((read_json_string_field(p, "id")?.to_owned(), p.clone())))
        .collect::<Result<_, StoreBackendFailureError>>()?;
    let mut contexts: BTreeMap<(String, bool), BTreeSet<String>> = members
        .iter()
        .filter_map(|id| {
            packages
                .get(id)
                .filter(|package| {
                    package["targets"].as_array().is_some_and(|targets| {
                        targets.iter().any(|target| {
                            check_library_target_kind(target)
                                || target["kind"][0] == "bin"
                                || target["kind"][0] == "proc-macro"
                        })
                    })
                })
                .map(|package| {
                    let defaults = if package["features"].get("default").is_some() {
                        BTreeSet::from(["default".into()])
                    } else {
                        BTreeSet::new()
                    };
                    ((id.clone(), check_proc_macro_package(package)), defaults)
                })
        })
        .collect();
    loop {
        let previous = contexts.clone();
        for ((id, host), enabled) in &previous {
            let package = packages
                .get(id)
                .ok_or_else(|| build_graph_stage_failure("package absent"))?;
            let mut features = enabled.clone();
            let mut optional = BTreeSet::new();
            let mut requested: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            loop {
                let before = features.clone();
                for feature in &before {
                    if let Some(expansion) = package["features"][feature].as_array() {
                        for value in expansion.iter().filter_map(Value::as_str) {
                            if let Some(name) = value.strip_prefix("dep:") {
                                optional.insert(name.to_owned());
                            } else if let Some((dependency, feature)) = value.split_once('/') {
                                if !dependency.ends_with('?') {
                                    optional.insert(dependency.to_owned());
                                }
                                requested
                                    .entry(dependency.trim_end_matches('?').into())
                                    .or_default()
                                    .insert(feature.into());
                            } else {
                                features.insert(value.into());
                            }
                        }
                    }
                }
                if before == features {
                    break;
                }
            }
            contexts
                .entry((id.clone(), *host))
                .or_default()
                .extend(features);
            for dependency in read_json_array_field(package, "dependencies")? {
                let kind = dependency["kind"].as_str().unwrap_or("normal");
                if kind == "dev" {
                    continue;
                }
                let alias = dependency["rename"]
                    .as_str()
                    .or_else(|| dependency["name"].as_str())
                    .ok_or_else(|| build_graph_stage_failure("dependency name absent"))?;
                if dependency["optional"] == true && !optional.contains(alias) {
                    continue;
                }
                if let Some(target) = dependency["target"].as_str() {
                    let active = if let Some(expression) = target
                        .strip_prefix("cfg(")
                        .and_then(|t| t.strip_suffix(')'))
                    {
                        evaluate_cfg_expression_truth(expression, &cfg)?
                    } else {
                        sysroot.contains(target)
                    };
                    if !active {
                        continue;
                    }
                }
                let target_id = nodes
                    .get(id)
                    .and_then(|node| node["deps"].as_array())
                    .and_then(|deps| {
                        deps.iter().find(|dep| {
                            dep["name"].as_str() == Some(&alias.replace('-', "_"))
                                && dep["dep_kinds"].as_array().is_some_and(|kinds| {
                                    kinds.iter().any(|k| {
                                        k["kind"].as_str().unwrap_or("normal") == kind
                                            && k["target"] == dependency["target"]
                                    })
                                })
                        })
                    })
                    .and_then(|dep| dep["pkg"].as_str());
                let Some(target_id) = target_id else {
                    continue;
                };
                let target_package = packages
                    .get(target_id)
                    .ok_or_else(|| build_graph_stage_failure("dependency package absent"))?;
                let target_host =
                    *host || kind == "build" || check_proc_macro_package(target_package);
                let target_features = contexts.entry((target_id.into(), target_host)).or_default();
                if dependency["uses_default_features"] != false
                    && target_package["features"].get("default").is_some()
                {
                    target_features.insert("default".into());
                }
                if let Some(features) = dependency["features"].as_array() {
                    target_features
                        .extend(features.iter().filter_map(Value::as_str).map(str::to_owned));
                }
                if let Some(features) = requested.get(alias) {
                    target_features.extend(features.iter().cloned());
                }
            }
        }
        if previous == contexts {
            break;
        }
    }
    let mut rows = Vec::new();
    for ((id, host), features) in &contexts {
        let package = &packages[id];
        let build_script = read_json_array_field(package, "targets")?.iter().any(|t| {
            t["kind"]
                .as_array()
                .is_some_and(|ks| ks.iter().any(|k| k == "custom-build"))
        });
        let cfg: BTreeSet<_> = cfg
            .iter()
            .cloned()
            .chain(features.iter().map(|f| format!("feature=\"{f}\"")))
            .collect();
        let manifest = Path::new(read_json_string_field(package, "manifest_path")?);
        let package_root = manifest
            .parent()
            .ok_or_else(|| build_graph_stage_failure("manifest parent absent"))?;
        let mut dependencies = Vec::new();
        for link in nodes
            .get(id)
            .and_then(|n| n["deps"].as_array())
            .into_iter()
            .flatten()
        {
            let Some(dependency_id) = link["pkg"].as_str() else {
                continue;
            };
            let dependency_package = packages
                .get(dependency_id)
                .ok_or_else(|| build_graph_stage_failure("dependency package absent"))?;
            for context in link["dep_kinds"].as_array().into_iter().flatten() {
                let kind = context["kind"].as_str().unwrap_or("normal");
                if kind == "dev" {
                    continue;
                }
                if let Some(target) = context["target"].as_str() {
                    let active = if let Some(expression) = target
                        .strip_prefix("cfg(")
                        .and_then(|v| v.strip_suffix(')'))
                    {
                        evaluate_cfg_expression_truth(expression, &cfg)?
                    } else {
                        sysroot.contains(target)
                    };
                    if !active {
                        continue;
                    }
                }
                let role_host =
                    *host || kind == "build" || check_proc_macro_package(dependency_package);
                if !contexts.contains_key(&(dependency_id.into(), role_host)) {
                    continue;
                }
                let definition =
                    read_json_array_field(package, "dependencies")?
                        .iter()
                        .find(|definition| {
                            definition["rename"]
                                .as_str()
                                .or_else(|| definition["name"].as_str())
                                .is_some_and(|alias| {
                                    Some(alias.replace('-', "_"))
                                        == link["name"].as_str().map(str::to_owned)
                                })
                                && definition["kind"].as_str().unwrap_or("normal") == kind
                                && definition["target"] == context["target"]
                        });
                if let Some(definition) = definition {
                    let alias = definition["rename"]
                        .as_str()
                        .or_else(|| definition["name"].as_str())
                        .unwrap_or("");
                    if definition["optional"] == true
                        && !features.iter().any(|feature| {
                            package["features"][feature]
                                .as_array()
                                .is_some_and(|values| {
                                    values.iter().filter_map(Value::as_str).any(|v| {
                                        v == format!("dep:{alias}")
                                            || v.starts_with(&format!("{alias}/"))
                                    })
                                })
                        })
                    {
                        continue;
                    }
                }
                dependencies.push(json!({"dependency_alias_name_value":link["name"],"dependency_cargo_package_identifier":dependency_id,
                    "dependency_compilation_role_kind":if role_host{"host"}else{"target"},
                    "dependency_context_kind_value":kind,"dependency_target_cfg_expression":context["target"]}));
            }
        }
        for target in read_json_array_field(package, "targets")? {
            let kind = target["kind"][0].as_str().unwrap_or("unknown");
            let selected =
                check_library_target_kind(target) || (members.contains(id) && kind == "bin");
            if !selected
                && !(*host && (kind == "proc-macro" || kind == "custom-build"))
                && !(build_script && kind == "custom-build")
            {
                continue;
            }
            let role = if *host || kind == "custom-build" || kind == "proc-macro" {
                "host"
            } else {
                "target"
            };
            let mapped = selected && role == "target";
            let version = read_json_string_field(package, "version")?;
            let without_build = version
                .split_once('+')
                .map(|(value, _)| value)
                .unwrap_or(version);
            let (release, prerelease) =
                without_build.split_once('-').unwrap_or((without_build, ""));
            let components: Vec<_> = release.split('.').collect();
            let mut unit = json!({
                "unit_cargo_package_identifier":id,"unit_cargo_package_source":package["source"],
                "unit_cargo_target_name":target["name"],"unit_cargo_target_kind":kind,
                "unit_compilation_role_kind":role,(MAPPED_TARGET_FLAG_FIELD):mapped,
                "unit_source_root_path":target["src_path"],"unit_package_root_path":package_root,
                "unit_rust_edition_value":target["edition"],"unit_enabled_feature_values":features,
                "unit_target_cfg_values":cfg,"unit_normal_dependency_rows":dependencies,
                "unit_build_script_present":build_script,
                "unit_metadata_environment_values":{
                    "CARGO_PKG_NAME":package["name"],"CARGO_PKG_VERSION":version,
                    "CARGO_PKG_VERSION_MAJOR":components.first().copied().unwrap_or(""),
                    "CARGO_PKG_VERSION_MINOR":components.get(1).copied().unwrap_or(""),
                    "CARGO_PKG_VERSION_PATCH":components.get(2).copied().unwrap_or(""),
                    "CARGO_PKG_VERSION_PRE":prerelease,
                    "CARGO_MANIFEST_DIR":package_root,"CARGO_CRATE_NAME":target["name"].as_str().unwrap_or("").replace('-',"_"),
                    "CARGO_PKG_DESCRIPTION":package["description"].as_str().unwrap_or(""),
                    "CARGO_PKG_LICENSE":package["license"].as_str().unwrap_or(""),
                    "CARGO_PKG_AUTHORS":package["authors"].as_array().map(|rows|rows.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(":")).unwrap_or_default(),
                    "CARGO_PKG_HOMEPAGE":package["homepage"].as_str().unwrap_or(""),
                    "CARGO_PKG_REPOSITORY":package["repository"].as_str().unwrap_or(""),
                    "CARGO_PKG_LICENSE_FILE":package["license_file"].as_str().unwrap_or(""),
                    "CARGO_PKG_README":package["readme"].as_str().unwrap_or(""),
                    "CARGO_PKG_RUST_VERSION":package["rust_version"].as_str().unwrap_or("")
                }
            });
            let key = blake3::hash(
                serde_json::to_string(&unit)
                    .map_err(build_graph_stage_failure)?
                    .as_bytes(),
            )
            .to_hex()
            .to_string();
            unit["unit_compilation_identity_key"] = json!(key);
            rows.push(StageTableRowRecord {
                stage_owned_table_name: "crate-dependency-graph-table".into(),
                stage_row_record_key: key,
                stage_row_payload_text: unit.to_string(),
            });
        }
    }
    rows.sort_by(|a, b| a.stage_row_record_key.cmp(&b.stage_row_record_key));
    rows.dedup_by(|a, b| a.stage_row_record_key == b.stage_row_record_key);
    rows.push(StageTableRowRecord {stage_owned_table_name:"workspace-config-state-records".into(),stage_row_record_key:"workspace".into(),
        stage_row_payload_text:json!({"workspace_source_root_path":root,"workspace_lock_digest_full":blake3::hash(&original).to_hex().to_string(),"workspace_sysroot_directory_path":sysroot.trim(),"workspace_target_cfg_values":cfg}).to_string()});
    Ok(rows)
}

pub fn read_safe_source_bytes(path: &Path) -> Result<(PathBuf, Vec<u8>), StoreBackendFailureError> {
    let canonical = path.canonicalize().map_err(build_graph_stage_failure)?;
    if canonical.components().any(|c| c.as_os_str() == "target") {
        return Err(build_graph_stage_failure(format!(
            "forbidden target source {}",
            canonical.display()
        )));
    }
    let bytes = std::fs::read(&canonical).map_err(build_graph_stage_failure)?;
    Ok((canonical, bytes))
}

pub fn ingest_requested_source_file(
    scope: &StageRunScopeValue,
    store: &dyn StageTableStorePort,
    cancel: &RunCancelFlagValue,
    path: &Path,
) -> Result<StageTableRowRecord, StoreBackendFailureError> {
    if cancel.check_stage_work_cancelled() {
        return Err(build_graph_stage_failure("stage cancelled"));
    }
    let (path, bytes) = read_safe_source_bytes(path)?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    store.write_file_content_bytes(&digest, &bytes)?;
    let key = path.to_string_lossy().into_owned();
    let row = StageTableRowRecord {stage_owned_table_name:"file-source-text-table".into(),stage_row_record_key:key.clone(),
        stage_row_payload_text:json!({"file_source_absolute_path":key,"file_content_digest_full":digest,"file_source_byte_count":bytes.len()}).to_string()};
    store.write_stage_table_unit(
        &StageUnitWriteRequest {
            stage_run_scope_value: scope,
            stage_owner_name_value: "source-ingest",
            stage_unit_work_key: &key,
            stage_input_stage_names: &["crate-graph"],
            stage_unit_output_rows: std::slice::from_ref(&row),
        },
        cancel,
    )?;
    Ok(row)
}

pub fn ingest_workspace_source_files(
    scope: &StageRunScopeValue,
    store: &dyn StageTableStorePort,
    cancel: &RunCancelFlagValue,
) -> Result<usize, StoreBackendFailureError> {
    let configuration = store.read_stage_table_rows(scope, "workspace-config-state-records")?;
    let configuration: Value = serde_json::from_str(
        &configuration
            .first()
            .ok_or_else(|| build_graph_stage_failure("StageInputMissingError: graph config"))?
            .stage_row_payload_text,
    )
    .map_err(build_graph_stage_failure)?;
    let graph = store.read_stage_table_rows(scope, "crate-dependency-graph-table")?;
    let roots = BTreeSet::from([PathBuf::from(read_json_string_field(
        &configuration,
        "workspace_source_root_path",
    )?)]);
    let roots = graph.into_iter().try_fold(roots, |mut roots, row| {
        let unit: Value =
            serde_json::from_str(&row.stage_row_payload_text).map_err(build_graph_stage_failure)?;
        if unit[MAPPED_TARGET_FLAG_FIELD] == true || unit["unit_sysroot_context_flag"] == true {
            let package = PathBuf::from(read_json_string_field(&unit, "unit_package_root_path")?);
            roots.insert(if unit["unit_sysroot_context_flag"] == true {
                package
                    .parent()
                    .ok_or_else(|| build_graph_stage_failure("sysroot library parent missing"))?
                    .to_owned()
            } else {
                package
            });
        }
        Ok::<_, StoreBackendFailureError>(roots)
    })?;
    let mut files = BTreeSet::new();
    fn walk_source_directory_tree(
        root: &Path,
        files: &mut BTreeSet<PathBuf>,
    ) -> Result<(), StoreBackendFailureError> {
        if root.components().any(|c| c.as_os_str() == "target") {
            return Ok(());
        }
        for entry in std::fs::read_dir(root).map_err(build_graph_stage_failure)? {
            let entry = entry.map_err(build_graph_stage_failure)?;
            let path = entry.path();
            if entry
                .file_type()
                .map_err(build_graph_stage_failure)?
                .is_dir()
            {
                if !matches!(
                    entry.file_name().to_str(),
                    Some(".git" | "target" | "node_modules" | ".codebase-memory" | "e2e-runs")
                ) {
                    walk_source_directory_tree(&path, files)?;
                }
            } else if path.extension().is_some_and(|e| e == "rs") {
                let canonical = path.canonicalize().map_err(build_graph_stage_failure)?;
                if !canonical.components().any(|c| c.as_os_str() == "target") {
                    files.insert(canonical);
                }
            }
        }
        Ok(())
    }
    roots
        .into_iter()
        .try_for_each(|root| walk_source_directory_tree(&root, &mut files))?;
    for path in &files {
        ingest_requested_source_file(scope, store, cancel, path)?;
    }
    Ok(files.len())
}
