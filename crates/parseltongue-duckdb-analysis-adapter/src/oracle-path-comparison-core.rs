use parseltongue_core_row_types::StoreBackendFailureError;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OracleComparisonFailureError {
    #[error("no published source generation")]
    SourceNotPublished,
    #[error("source publication changed during comparison")]
    SourceChanged,
    #[error("invalid {configuration} oracle envelope field {field}")]
    InvalidEnvelope {
        configuration: String,
        field: String,
    },
    #[error("oracle scope does not match the published source for {configuration}")]
    ScopeMismatch { configuration: String },
    #[error("invalid comparison record field {field}")]
    InvalidRecord { field: String },
    #[error("conflicting duplicate path site {site_key}")]
    DuplicateConflict { site_key: String },
    #[error("malformed proc-macro loading evidence field {field}")]
    MalformedMacroLoadingEvidence { field: String },
    #[error("proc-macro loading failed for {source_root}: {error_detail}")]
    ProcMacroLoadingFailed {
        source_root: String,
        error_detail: String,
    },
    #[error("oracle path comparison was cancelled")]
    Cancelled,
    #[error("cannot mutate source store with agreement output")]
    CannotMutateSource,
    #[error("source row verification failed for table {table}")]
    InvalidSourceRows { table: String },
    #[error("duplicate source row key {key} in table {table}")]
    DuplicateSourceRow { table: String, key: String },
    #[error("DuckDB source copy row count mismatch")]
    SourceCopyCountMismatch,
    #[error("DuckDB source copy framed digest mismatch")]
    SourceCopyDigestMismatch,
    #[error("unsupported DuckDB thread count {threads}")]
    UnsupportedThreadCount { threads: usize },
    #[error("DuckDB comparison failed: {0}")]
    Duckdb(#[from] duckdb::Error),
    #[error("could not read oracle input {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Store(#[from] StoreBackendFailureError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug)]
pub(crate) struct CanonicalPathSiteValue {
    pub site_key: String,
    pub unit_identity: String,
    pub classification: String,
    pub classification_reason: String,
    pub unresolved_reason: String,
    pub targets_json: String,
    pub targets_empty: bool,
    pub proc_macro_dependent: bool,
    pub duplicate_count: usize,
    pub identity_json: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SitePathContextKind {
    Import,
    Signature,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PathLookupScopeClassification {
    ModuleScope,
    TypeRelative,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PathSiteUnresolvedReason {
    CfgInactive,
    BuildScript,
    ProcMacro,
    ExternalSourceMissing,
    UnknownReceiver,
    TraitSolverNeeded,
}

pub(crate) fn read_required_text_value<'a>(
    value: &'a Value,
    key: &str,
) -> Result<&'a str, OracleComparisonFailureError> {
    value[key]
        .as_str()
        .filter(|text| !text.is_empty())
        .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
            field: key.to_owned(),
        })
}

fn normalize_path_components_safely(
    path: &Path,
) -> Result<(bool, Vec<String>), OracleComparisonFailureError> {
    let mut absolute = false;
    let mut components = Vec::<String>::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) => {
                return Err(OracleComparisonFailureError::InvalidRecord {
                    field: "path_prefix".into(),
                });
            }
            Component::RootDir => absolute = true,
            Component::CurDir => {}
            Component::ParentDir => {
                if components.pop().is_none() {
                    return Err(OracleComparisonFailureError::InvalidRecord {
                        field: "path_escape".into(),
                    });
                }
            }
            Component::Normal(value) => {
                let value =
                    value
                        .to_str()
                        .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
                            field: "path_encoding".into(),
                        })?;
                if value.is_empty() || value.contains(['\\', ':', '\0']) {
                    return Err(OracleComparisonFailureError::InvalidRecord {
                        field: "path_component".into(),
                    });
                }
                components.push(value.to_owned());
            }
        }
    }
    Ok((absolute, components))
}

pub(crate) fn normalize_package_relative_path(
    path: &str,
    package_root: &str,
) -> Result<String, OracleComparisonFailureError> {
    let (path_absolute, path_components) = normalize_path_components_safely(Path::new(path))?;
    let (root_absolute, root_components) =
        normalize_path_components_safely(Path::new(package_root))?;
    let relative = if path_absolute {
        if !root_absolute || !path_components.starts_with(&root_components) {
            return Err(OracleComparisonFailureError::InvalidRecord {
                field: "path_outside_package".into(),
            });
        }
        &path_components[root_components.len()..]
    } else {
        &path_components[..]
    };
    if relative.is_empty() {
        return Err(OracleComparisonFailureError::InvalidRecord {
            field: "empty_source_path".into(),
        });
    }
    Ok(relative.join("/"))
}

fn read_nonempty_string_array(
    value: &Value,
    field: &str,
) -> Result<Vec<String>, OracleComparisonFailureError> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
            field: field.into(),
        })?
        .iter()
        .map(|item| {
            item.as_str()
                .filter(|text| !text.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
                    field: field.into(),
                })
        })
        .collect()
}

fn derive_semantic_unit_identity(unit: &Value) -> Result<String, OracleComparisonFailureError> {
    let package = read_required_text_value(unit, "unit_cargo_package_identifier")?;
    let target = read_required_text_value(unit, "unit_cargo_target_name")?;
    let kind = read_required_text_value(unit, "unit_cargo_target_kind")?;
    let role = read_required_text_value(unit, "unit_compilation_role_kind")?;
    let edition = read_required_text_value(unit, "unit_rust_edition_value")?;
    if !matches!(role, "host" | "target") {
        return Err(OracleComparisonFailureError::InvalidRecord {
            field: "unit_compilation_role_kind".into(),
        });
    }
    if !matches!(edition, "2015" | "2018" | "2021" | "2024") {
        return Err(OracleComparisonFailureError::InvalidRecord {
            field: "unit_rust_edition_value".into(),
        });
    }
    let cfg = read_nonempty_string_array(unit, "unit_target_cfg_values")?;
    let features = read_nonempty_string_array(unit, "unit_enabled_feature_values")?;
    unit.get("unit_mapped_target_flag")
        .and_then(Value::as_bool)
        .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
            field: "unit_mapped_target_flag".into(),
        })?;
    let configuration = json!([cfg, features, edition]);
    let configuration_digest = blake3::hash(configuration.to_string().as_bytes())
        .to_hex()
        .to_string();
    let identity = json!({
        "unit_cargo_package_identifier":package,
        "unit_cargo_target_name":target,
        "unit_cargo_target_kind":kind,
        "unit_compilation_role_kind":role,
        "unit_configuration_digest_full":configuration_digest,
    });
    Ok(identity.to_string())
}

pub(crate) fn validate_compilation_unit_rows(
    units: &[Value],
) -> Result<usize, OracleComparisonFailureError> {
    let mut mapped_count = 0;
    for unit in units {
        read_required_text_value(unit, "unit_source_root_path")?;
        read_required_text_value(unit, "unit_package_root_path")?;
        let mapped = unit["unit_mapped_target_flag"].as_bool().ok_or_else(|| {
            OracleComparisonFailureError::InvalidRecord {
                field: "unit_mapped_target_flag".into(),
            }
        })?;
        if mapped {
            mapped_count += 1;
        }
        derive_semantic_unit_identity(unit)?;
    }
    Ok(mapped_count)
}

fn resolve_compilation_context_identity(
    context: &Value,
    units: &[Value],
    require_mapped: bool,
) -> Result<(String, String), OracleComparisonFailureError> {
    let root = read_required_text_value(context, "unit_source_root_path")?;
    let package = read_required_text_value(context, "unit_cargo_package_identifier")?;
    let target = read_required_text_value(context, "unit_cargo_target_name")?;
    let kind = read_required_text_value(context, "unit_cargo_target_kind")?;
    let edition = read_required_text_value(context, "unit_rust_edition_value")?;
    let expected_semantic = context.get("unit_semantic_identity_record");
    if expected_semantic.is_some_and(|value| !value.is_object()) {
        return Err(OracleComparisonFailureError::InvalidRecord {
            field: "unit_semantic_identity_record".into(),
        });
    }
    let mut candidates = Vec::new();
    for unit in units {
        let mapped = unit["unit_mapped_target_flag"].as_bool() == Some(true);
        let matches = (!require_mapped || mapped)
            && unit.get("unit_source_root_path").and_then(Value::as_str) == Some(root)
            && unit
                .get("unit_cargo_package_identifier")
                .and_then(Value::as_str)
                == Some(package)
            && unit.get("unit_cargo_target_name").and_then(Value::as_str) == Some(target)
            && unit.get("unit_cargo_target_kind").and_then(Value::as_str) == Some(kind)
            && unit.get("unit_rust_edition_value").and_then(Value::as_str) == Some(edition);
        if !matches {
            continue;
        }
        let semantic = derive_semantic_unit_identity(unit)?;
        let semantic_value: Value = serde_json::from_str(&semantic)?;
        if expected_semantic
            .is_some_and(|expected| !expected.is_null() && expected != &semantic_value)
        {
            continue;
        }
        candidates.push((unit, semantic));
    }
    let [(unit, semantic)] = candidates.as_slice() else {
        return Err(OracleComparisonFailureError::InvalidRecord {
            field: if candidates.is_empty() {
                "compilation_context_missing"
            } else {
                "compilation_context_ambiguous"
            }
            .into(),
        });
    };
    let package_root = read_required_text_value(unit, "unit_package_root_path")?.to_owned();
    Ok((semantic.clone(), package_root))
}

fn read_site_compilation_context(value: &Value) -> Option<&Value> {
    value.get("site_compilation_scope_record").or_else(|| {
        value
            .get("path_source_metadata_value")
            .and_then(|metadata| metadata.get("site_compilation_scope_record"))
    })
}

fn canonicalize_target_symbol_rows(
    site: &Value,
    units: &[Value],
) -> Result<Vec<String>, OracleComparisonFailureError> {
    let targets = site
        .get("path_target_symbol_rows")
        .and_then(Value::as_array)
        .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
            field: "path_target_symbol_rows".into(),
        })?;
    let mut canonical = BTreeMap::<String, (Value, BTreeSet<String>)>::new();
    for raw in targets {
        let kind = read_required_text_value(raw, "target_symbol_kind_value")?;
        let namespaces = raw
            .get("target_namespace_kind_values")
            .and_then(Value::as_array)
            .filter(|values| !values.is_empty())
            .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
                field: "target_namespace_kind_values".into(),
            })?
            .iter()
            .map(|value| {
                let namespace = value
                    .as_str()
                    .filter(|namespace| matches!(*namespace, "type" | "value" | "macro"));
                namespace.map(str::to_owned).ok_or_else(|| {
                    OracleComparisonFailureError::InvalidRecord {
                        field: "target_namespace_kind_values".into(),
                    }
                })
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        if let Some(name_value) = raw.get("target_builtin_name_value") {
            let name = name_value
                .as_str()
                .filter(|name| !name.is_empty())
                .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
                    field: "target_builtin_name_value".into(),
                })?;
            if kind != "Builtin"
                || raw
                    .get("target_compilation_scope_record")
                    .is_some_and(|context| !context.is_null())
                || raw.get("target_source_file_path").is_some()
                || raw.get("target_source_range_start").is_some()
                || raw.get("target_source_range_end").is_some()
            {
                return Err(OracleComparisonFailureError::InvalidRecord {
                    field: "target_builtin_name_value".into(),
                });
            }
            let base = json!(["builtin", name, kind]);
            let key = base.to_string();
            canonical
                .entry(key)
                .and_modify(|entry| entry.1.extend(namespaces.clone()))
                .or_insert((base, namespaces));
            continue;
        }
        if kind == "Builtin" {
            return Err(OracleComparisonFailureError::InvalidRecord {
                field: "target_builtin_name_value".into(),
            });
        }
        let context = raw
            .get("target_compilation_scope_record")
            .filter(|context| context.is_object())
            .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
                field: "target_compilation_scope_record".into(),
            })?;
        let (unit_identity, package_root) =
            resolve_compilation_context_identity(context, units, false)?;
        let file = read_required_text_value(raw, "target_source_file_path")?;
        let start = raw["target_source_range_start"].as_u64().ok_or_else(|| {
            OracleComparisonFailureError::InvalidRecord {
                field: "target_source_range_start".into(),
            }
        })?;
        let end = raw["target_source_range_end"].as_u64().ok_or_else(|| {
            OracleComparisonFailureError::InvalidRecord {
                field: "target_source_range_end".into(),
            }
        })?;
        if end < start || (end == start && kind != "Crate") {
            return Err(OracleComparisonFailureError::InvalidRecord {
                field: "target_source_range".into(),
            });
        }
        let normalized_file = normalize_package_relative_path(file, &package_root)?;
        let module_path = context
            .get("unit_module_path_values")
            .cloned()
            .unwrap_or_else(|| json!([]));
        if !module_path.is_array()
            || module_path
                .as_array()
                .is_some_and(|values| values.iter().any(|value| value.as_str().is_none()))
        {
            return Err(OracleComparisonFailureError::InvalidRecord {
                field: "unit_module_path_values".into(),
            });
        }
        let base = json!([
            unit_identity,
            normalized_file,
            start,
            end,
            kind,
            module_path
        ]);
        let key = base.to_string();
        canonical
            .entry(key)
            .and_modify(|entry| entry.1.extend(namespaces.clone()))
            .or_insert((base, namespaces));
    }
    Ok(canonical
        .into_values()
        .map(|(base, namespaces)| json!([base, namespaces]).to_string())
        .collect())
}

pub(crate) fn canonicalize_site_evidence_rows(
    rows: &[Value],
    units: &[Value],
    classify: bool,
) -> Result<Vec<CanonicalPathSiteValue>, OracleComparisonFailureError> {
    let mut sites = BTreeMap::<String, CanonicalPathSiteValue>::new();
    for raw in rows {
        let metadata = raw
            .get("path_source_metadata_value")
            .filter(|value| value.is_object())
            .unwrap_or(raw);
        let context = read_site_compilation_context(raw).ok_or_else(|| {
            OracleComparisonFailureError::InvalidRecord {
                field: "site_compilation_scope_record".into(),
            }
        })?;
        let (unit_identity, package_root) =
            resolve_compilation_context_identity(context, units, true)?;
        let file = raw
            .get("site_source_absolute_path")
            .or_else(|| metadata.get("site_source_absolute_path"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
                field: "site_source_absolute_path".into(),
            })?;
        let start = raw
            .get("site_source_range_start")
            .and_then(Value::as_u64)
            .or_else(|| {
                metadata
                    .get("site_source_range_start")
                    .and_then(Value::as_u64)
            })
            .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
                field: "site_source_range_start".into(),
            })?;
        let end = raw
            .get("site_source_range_end")
            .and_then(Value::as_u64)
            .or_else(|| {
                metadata
                    .get("site_source_range_end")
                    .and_then(Value::as_u64)
            })
            .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
                field: "site_source_range_end".into(),
            })?;
        if end <= start {
            return Err(OracleComparisonFailureError::InvalidRecord {
                field: "site_source_range".into(),
            });
        }
        let normalized_file = normalize_package_relative_path(file, &package_root)?;
        let context_kind_value = raw
            .get("site_path_context_kind")
            .or_else(|| metadata.get("site_path_context_kind"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
                field: "site_path_context_kind".into(),
            })?;
        let context_kind = match context_kind_value {
            "import" => SitePathContextKind::Import,
            "signature" => SitePathContextKind::Signature,
            _ => {
                return Err(OracleComparisonFailureError::InvalidRecord {
                    field: "site_path_context_kind".into(),
                });
            }
        };
        let kind = match context_kind {
            SitePathContextKind::Import => "import",
            SitePathContextKind::Signature => "signature",
        };
        let provenance = raw
            .get("site_expansion_provenance_chain")
            .or_else(|| metadata.get("site_expansion_provenance_chain"));
        let provenance = match provenance {
            Some(Value::Array(values)) => {
                for value in values {
                    if value.as_str().is_none_or(str::is_empty) {
                        return Err(OracleComparisonFailureError::InvalidRecord {
                            field: "site_expansion_provenance_chain".into(),
                        });
                    }
                }
                Value::Array(values.clone())
            }
            Some(_) => {
                return Err(OracleComparisonFailureError::InvalidRecord {
                    field: "site_expansion_provenance_chain".into(),
                });
            }
            None if classify
                && raw["path_site_source_text"]
                    .as_str()
                    .is_some_and(|text| !text.is_empty()) =>
            {
                json!([])
            }
            None => {
                return Err(OracleComparisonFailureError::InvalidRecord {
                    field: "site_expansion_provenance_chain".into(),
                });
            }
        };
        let identity_json =
            json!([unit_identity, normalized_file, start, end, kind, provenance]).to_string();
        let site_key = blake3::hash(identity_json.as_bytes()).to_hex().to_string();
        let targets = canonicalize_target_symbol_rows(raw, units)?;
        let targets_json = serde_json::to_string(&targets)?;
        let lookup_value = raw
            .get("path_lookup_scope_kind")
            .or_else(|| metadata.get("path_lookup_scope_kind"));
        let (classification, classification_reason) = if classify {
            let lookup_scope = match lookup_value {
                None | Some(Value::Null) => PathLookupScopeClassification::Unknown,
                Some(Value::String(value)) => match value.as_str() {
                    "module-scope" => PathLookupScopeClassification::ModuleScope,
                    "type-relative" => PathLookupScopeClassification::TypeRelative,
                    "unknown" => PathLookupScopeClassification::Unknown,
                    _ => {
                        return Err(OracleComparisonFailureError::InvalidRecord {
                            field: "path_lookup_scope_kind".into(),
                        });
                    }
                },
                Some(_) => {
                    return Err(OracleComparisonFailureError::InvalidRecord {
                        field: "path_lookup_scope_kind".into(),
                    });
                }
            };
            match lookup_scope {
                PathLookupScopeClassification::ModuleScope => ("module-scope", "module_scope"),
                PathLookupScopeClassification::TypeRelative => ("type-relative", "type_relative"),
                PathLookupScopeClassification::Unknown => match lookup_value {
                    None | Some(Value::Null) => ("blocked", "missing_lookup_scope"),
                    _ => ("blocked", "unknown_lookup_scope"),
                },
            }
        } else {
            ("production", "production_source")
        };
        let reason_value = raw
            .get("path_unresolved_reason_kind")
            .or_else(|| metadata.get("path_unresolved_reason_kind"));
        let parsed_reason = match reason_value {
            None | Some(Value::Null) => None,
            Some(Value::String(reason)) => match reason.as_str() {
                "cfg_inactive" => Some(PathSiteUnresolvedReason::CfgInactive),
                "build_script" => Some(PathSiteUnresolvedReason::BuildScript),
                "proc_macro" => Some(PathSiteUnresolvedReason::ProcMacro),
                "external_source_missing" => Some(PathSiteUnresolvedReason::ExternalSourceMissing),
                "unknown_receiver" => Some(PathSiteUnresolvedReason::UnknownReceiver),
                "trait_solver_needed" => Some(PathSiteUnresolvedReason::TraitSolverNeeded),
                _ => {
                    return Err(OracleComparisonFailureError::InvalidRecord {
                        field: "path_unresolved_reason_kind".into(),
                    });
                }
            },
            Some(_) => {
                return Err(OracleComparisonFailureError::InvalidRecord {
                    field: "path_unresolved_reason_kind".into(),
                });
            }
        };
        let unresolved_reason = match parsed_reason {
            None => "",
            Some(PathSiteUnresolvedReason::CfgInactive) => "cfg_inactive",
            Some(PathSiteUnresolvedReason::BuildScript) => "build_script",
            Some(PathSiteUnresolvedReason::ProcMacro) => "proc_macro",
            Some(PathSiteUnresolvedReason::ExternalSourceMissing) => "external_source_missing",
            Some(PathSiteUnresolvedReason::UnknownReceiver) => "unknown_receiver",
            Some(PathSiteUnresolvedReason::TraitSolverNeeded) => "trait_solver_needed",
        };
        let proc_macro_flag = raw
            .get("site_proc_macro_dependent_flag")
            .or_else(|| metadata.get("site_proc_macro_dependent_flag"));
        let proc_macro_flag = match proc_macro_flag {
            None => false,
            Some(value) => {
                value
                    .as_bool()
                    .ok_or_else(|| OracleComparisonFailureError::InvalidRecord {
                        field: "site_proc_macro_dependent_flag".into(),
                    })?
            }
        };
        let normalized = CanonicalPathSiteValue {
            site_key: site_key.clone(),
            unit_identity,
            classification: classification.into(),
            classification_reason: classification_reason.into(),
            unresolved_reason: unresolved_reason.to_owned(),
            targets_empty: targets.is_empty(),
            targets_json,
            proc_macro_dependent: proc_macro_flag || unresolved_reason == "proc_macro",
            duplicate_count: 1,
            identity_json,
        };
        if let Some(existing) = sites.get_mut(&site_key) {
            if existing.classification != normalized.classification
                || existing.classification_reason != normalized.classification_reason
                || existing.unresolved_reason != normalized.unresolved_reason
                || existing.targets_json != normalized.targets_json
                || existing.proc_macro_dependent != normalized.proc_macro_dependent
            {
                return Err(OracleComparisonFailureError::DuplicateConflict { site_key });
            }
            existing.duplicate_count += 1;
        } else {
            sites.insert(site_key, normalized);
        }
    }
    Ok(sites.into_values().collect())
}

#[cfg(test)]
mod oracle_path_normalization_tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn build_test_unit_record(package_root: &str, target_name: &str, target_kind: &str) -> Value {
        json!({
            "unit_compilation_identity_key":"fixture-unit",
            "unit_source_root_path":format!("{package_root}/src/lib.rs"),
            "unit_package_root_path":package_root,
            "unit_cargo_package_identifier":format!("path+file://{package_root}#fixture@0.1.0"),
            "unit_cargo_target_name":target_name,
            "unit_cargo_target_kind":target_kind,
            "unit_compilation_role_kind":"target",
            "unit_rust_edition_value":"2024",
            "unit_target_cfg_values":[],
            "unit_enabled_feature_values":[],
            "unit_mapped_target_flag":true,
        })
    }

    fn build_test_path_site_row(unit: &Value) -> Value {
        let context = json!({
            "unit_source_root_path":unit["unit_source_root_path"],
            "unit_cargo_package_identifier":unit["unit_cargo_package_identifier"],
            "unit_cargo_target_name":unit["unit_cargo_target_name"],
            "unit_cargo_target_kind":unit["unit_cargo_target_kind"],
            "unit_rust_edition_value":unit["unit_rust_edition_value"],
        });
        json!({
            "site_compilation_scope_record":context,
            "site_source_absolute_path":unit["unit_source_root_path"],
            "site_source_range_start":10,
            "site_source_range_end":14,
            "site_path_context_kind":"signature",
            "path_site_source_text":"fixture::Item",
            "site_expansion_provenance_chain":[],
            "path_lookup_scope_kind":"module-scope",
            "path_unresolved_reason_kind":null,
            "site_proc_macro_dependent_flag":false,
            "path_target_symbol_rows":[{
                "target_compilation_scope_record":context,
                "target_source_file_path":unit["unit_source_root_path"],
                "target_source_range_start":0,
                "target_source_range_end":4,
                "target_symbol_kind_value":"Struct",
                "target_namespace_kind_values":["type"],
            }],
        })
    }

    /// REQ-RUST-025.1
    #[test]
    fn test_site_context_identity_distinct() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let second = build_test_unit_record("/fixture/pkg", "fixture-bin", "bin");
        let first_site = build_test_path_site_row(&unit);
        let second_site = build_test_path_site_row(&second);
        let sites =
            canonicalize_site_evidence_rows(&[first_site, second_site], &[unit, second], true)
                .unwrap();
        assert_eq!(sites.len(), 2);
        assert_ne!(sites[0].site_key, sites[1].site_key);
    }

    /// REQ-RUST-025.1
    #[test]
    fn test_exact_duplicates_collapse_rows() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let site = build_test_path_site_row(&unit);
        let sites = canonicalize_site_evidence_rows(&[site.clone(), site], &[unit], true).unwrap();
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].duplicate_count, 2);
    }

    /// REQ-RUST-025.1
    #[test]
    fn test_conflicting_duplicate_payloads_fail() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let site = build_test_path_site_row(&unit);
        let mut conflict = site.clone();
        conflict["path_target_symbol_rows"][0]["target_source_range_start"] = json!(2);
        conflict["path_target_symbol_rows"][0]["target_source_range_end"] = json!(6);
        assert!(matches!(
            canonicalize_site_evidence_rows(&[site, conflict], &[unit], true),
            Err(OracleComparisonFailureError::DuplicateConflict { .. })
        ));
    }

    /// REQ-RUST-025.1
    #[test]
    fn test_expansion_provenance_separates_sites() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let first = build_test_path_site_row(&unit);
        let mut second = first.clone();
        second["site_expansion_provenance_chain"] = json!(["invocation-a"]);
        let sites = canonicalize_site_evidence_rows(&[first, second], &[unit], true).unwrap();
        assert_eq!(sites.len(), 2);
    }

    /// REQ-RUST-025.0
    #[test]
    fn test_ambiguous_unit_contexts_fail() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let site = build_test_path_site_row(&unit);
        assert!(matches!(
            canonicalize_site_evidence_rows(&[site], &[unit.clone(), unit], true),
            Err(OracleComparisonFailureError::InvalidRecord { field }) if field == "compilation_context_ambiguous"
        ));
    }

    /// REQ-RUST-025.1
    #[test]
    fn test_package_paths_normalize_lexically() {
        assert_eq!(
            normalize_package_relative_path("/fixture/pkg/src/./nested/../lib.rs", "/fixture/pkg")
                .unwrap(),
            "src/lib.rs",
        );
        assert!(
            normalize_package_relative_path("/fixture/elsewhere/lib.rs", "/fixture/pkg").is_err()
        );
    }

    /// REQ-RUST-025.0
    #[test]
    fn test_malformed_target_ranges_fail() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let mut site = build_test_path_site_row(&unit);
        site["path_target_symbol_rows"][0]["target_source_range_start"] = json!(5);
        assert!(canonicalize_site_evidence_rows(&[site], &[unit], true).is_err());
    }

    /// REQ-RUST-025.2
    #[test]
    fn test_unknown_scope_is_blocked() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let mut site = build_test_path_site_row(&unit);
        site["path_lookup_scope_kind"] = json!("unknown");
        let sites = canonicalize_site_evidence_rows(&[site], &[unit], true).unwrap();
        assert_eq!(sites[0].classification, "blocked");
    }

    /// REQ-RUST-025.1
    #[test]
    fn test_target_namespaces_are_canonicalized() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let mut first = build_test_path_site_row(&unit);
        let mut second = first.clone();
        first["path_target_symbol_rows"][0]["target_namespace_kind_values"] =
            json!(["type", "value", "type"]);
        second["path_target_symbol_rows"][0]["target_namespace_kind_values"] =
            json!(["value", "type"]);
        let sites = canonicalize_site_evidence_rows(&[first, second], &[unit], true).unwrap();
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].duplicate_count, 2);
    }

    /// REQ-RUST-025.1
    #[test]
    fn test_builtin_target_names_canonicalize() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let mut site = build_test_path_site_row(&unit);
        site["path_target_symbol_rows"] = json!([{
            "target_builtin_name_value":"u32",
            "target_symbol_kind_value":"Builtin",
            "target_namespace_kind_values":["type"],
            "target_compilation_scope_record":null,
        }]);
        let sites = canonicalize_site_evidence_rows(&[site], &[unit], true).unwrap();
        assert!(sites[0].targets_json.contains("u32"));
    }

    /// REQ-RUST-025.1
    #[test]
    fn test_builtin_namespace_changes_identity() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let mut first = build_test_path_site_row(&unit);
        let mut second = first.clone();
        first["path_target_symbol_rows"] = json!([{
            "target_builtin_name_value":"u32",
            "target_symbol_kind_value":"Builtin",
            "target_namespace_kind_values":["type"],
            "target_compilation_scope_record":null,
        }]);
        second["path_target_symbol_rows"] = json!([{
            "target_builtin_name_value":"u32",
            "target_symbol_kind_value":"Builtin",
            "target_namespace_kind_values":["value"],
            "target_compilation_scope_record":null,
        }]);
        let sites = canonicalize_site_evidence_rows(&[first, second], &[unit], true);
        assert!(matches!(
            sites,
            Err(OracleComparisonFailureError::DuplicateConflict { .. })
        ));
    }

    /// REQ-RUST-025.1
    #[test]
    fn test_split_target_namespaces_merge() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let mut split = build_test_path_site_row(&unit);
        let mut grouped = split.clone();
        let base = split["path_target_symbol_rows"][0].clone();
        let mut value_target = base.clone();
        value_target["target_namespace_kind_values"] = json!(["value"]);
        split["path_target_symbol_rows"] = json!([base, value_target]);
        grouped["path_target_symbol_rows"][0]["target_namespace_kind_values"] =
            json!(["type", "value"]);
        let sites = canonicalize_site_evidence_rows(&[split, grouped], &[unit], true);
        assert!(matches!(sites, Ok(sites) if sites.len() == 1 && sites[0].duplicate_count == 2));
    }

    /// REQ-RUST-025.0
    #[test]
    fn test_invalid_context_kind_rejected() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let mut site = build_test_path_site_row(&unit);
        site["site_path_context_kind"] = json!("method");
        assert!(canonicalize_site_evidence_rows(&[site], &[unit], true).is_err());
    }

    /// REQ-RUST-025.0
    #[test]
    fn test_invalid_reason_enum_rejected() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let mut site = build_test_path_site_row(&unit);
        site["path_unresolved_reason_kind"] = json!("invented_reason");
        assert!(canonicalize_site_evidence_rows(&[site], &[unit], true).is_err());
    }

    /// REQ-RUST-025.0
    #[test]
    fn test_invalid_proc_flag_rejected() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let mut site = build_test_path_site_row(&unit);
        site["site_proc_macro_dependent_flag"] = json!("true");
        assert!(canonicalize_site_evidence_rows(&[site], &[unit], true).is_err());
    }

    /// REQ-RUST-025.0
    #[test]
    fn test_invalid_namespace_enum_rejected() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let mut site = build_test_path_site_row(&unit);
        site["path_target_symbol_rows"][0]["target_namespace_kind_values"] = json!(["constructor"]);
        assert!(canonicalize_site_evidence_rows(&[site], &[unit], true).is_err());
    }

    /// REQ-RUST-025.0
    #[test]
    fn test_unit_configuration_contracts_fail() {
        for (field, value) in [
            ("unit_target_cfg_values", json!("unix")),
            ("unit_enabled_feature_values", json!({})),
            ("unit_rust_edition_value", json!("future")),
            ("unit_compilation_role_kind", json!("support")),
            ("unit_mapped_target_flag", json!("true")),
            ("unit_package_root_path", json!("")),
        ] {
            let mut unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
            unit[field] = value;
            assert!(validate_compilation_unit_rows(&[unit]).is_err(), "{field}");
        }
    }

    /// REQ-RUST-025.0
    #[test]
    fn test_unmapped_site_units_rejected() {
        let mut unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        unit["unit_mapped_target_flag"] = json!(false);
        let site = build_test_path_site_row(&unit);
        assert!(canonicalize_site_evidence_rows(&[site], &[unit], true).is_err());
    }

    /// REQ-RUST-025.0
    #[test]
    fn test_site_provenance_entries_validated() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        let mut site = build_test_path_site_row(&unit);
        site["site_expansion_provenance_chain"] = json!(["invocation", 7]);
        assert!(canonicalize_site_evidence_rows(&[site], &[unit], true).is_err());
    }

    /// REQ-RUST-025.0
    #[test]
    fn test_site_evidence_required_values() {
        let unit = build_test_unit_record("/fixture/pkg", "fixture", "lib");
        for (field, value) in [
            ("site_source_absolute_path", json!("")),
            ("site_path_context_kind", json!("")),
        ] {
            let mut site = build_test_path_site_row(&unit);
            site[field] = value;
            assert!(
                canonicalize_site_evidence_rows(&[site], std::slice::from_ref(&unit), true)
                    .is_err()
            );
        }
    }
}
