use super::{
    crate_syntax_namespace_projection::{CrateSyntaxProjectionState, build_unit_compilation_key},
    name_resolution_record_codec::*,
    persisted_macro_expansion_adapter::PersistedItemMacroPort,
};
use parseltongue_core_row_types::*;
use parseltongue_port_name_resolver::resolve_crate_name_bindings;
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn read_name_stage_records(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    table: &str,
) -> Result<Vec<Value>, StoreBackendFailureError> {
    store
        .read_stage_table_rows(scope, table)?
        .into_iter()
        .map(|row| {
            serde_json::from_str(&row.stage_row_payload_text).map_err(build_name_projection_error)
        })
        .collect()
}

fn check_unit_library_dependency(unit: &Value, other: &Value) -> bool {
    if other["unit_cargo_target_kind"] == "bin" || other["unit_compilation_role_kind"] != "target" {
        return false;
    }
    (unit["unit_cargo_target_kind"] == "bin"
        && unit["unit_cargo_package_identifier"] == other["unit_cargo_package_identifier"])
        || unit["unit_normal_dependency_rows"]
            .as_array()
            .is_some_and(|dependencies| {
                dependencies.iter().any(|dependency| {
                    dependency["dependency_cargo_package_identifier"]
                        == other["unit_cargo_package_identifier"]
                        && dependency["dependency_compilation_role_kind"]
                            == other["unit_compilation_role_kind"]
                })
            })
}

fn append_library_namespace_inputs(
    projector: &mut CrateSyntaxProjectionState<'_>,
    unit: &Value,
    units: &BTreeMap<String, Value>,
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
) -> Result<(), StoreBackendFailureError> {
    let own = &projector
        .projection_resolution_input_rows
        .unit_compilation_identity_key;
    let scopes = read_name_stage_records(store, scope, "name-resolution-scope-records")?
        .iter()
        .map(decode_scope_record_value)
        .collect::<Result<Vec<_>, _>>()?;
    projector
        .projection_resolution_input_rows
        .name_module_scope_rows
        .extend(
            scopes
                .into_iter()
                .filter(|row| &row.scope_module_identity_key.scope_compilation_unit_key != own),
        );
    read_name_stage_records(store, scope, "module-scope-binding-table")?
        .into_iter()
        .try_for_each(|value| {
            let binding = decode_binding_record_value(&value)?;
            if &binding.binding_lexical_scope_key.scope_compilation_unit_key == own {
                return Ok(());
            }
            projector.projection_target_metadata_rows.insert(
                binding.binding_target_identity_key.clone(),
                value["binding_target_metadata_value"].clone(),
            );
            projector
                .projection_resolution_input_rows
                .name_symbol_binding_rows
                .push(binding);
            Ok::<_, StoreBackendFailureError>(())
        })?;
    let external = projector
        .projection_resolution_input_rows
        .unit_extern_prelude_key
        .clone()
        .ok_or_else(|| build_name_projection_error("extern prelude missing"))?;
    for dependency in units
        .values()
        .filter(|other| check_unit_library_dependency(unit, other))
    {
        let dependency_key = build_unit_compilation_key(dependency)?;
        let root = NameResolutionScopeKey {
            scope_compilation_unit_key: dependency_key.clone(),
            scope_semantic_path_parts: vec![],
        };
        let alias = unit["unit_normal_dependency_rows"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|link| {
                link["dependency_cargo_package_identifier"]
                    == dependency["unit_cargo_package_identifier"]
            })
            .and_then(|link| link["dependency_alias_name_value"].as_str())
            .or_else(|| dependency["unit_cargo_target_name"].as_str())
            .ok_or_else(|| build_name_projection_error("dependency alias missing"))?
            .replace('-', "_");
        let target = NameSymbolTargetKey::CrateRoot(dependency_key);
        projector.projection_target_metadata_rows.insert(target.clone(),json!({
            "target_source_file_path":dependency["unit_source_root_path"],
            "target_source_range_start":0,"target_source_range_end":0,"target_symbol_kind_value":"Crate",
            "target_compilation_scope_record":{
                "unit_source_root_path":dependency["unit_source_root_path"],"unit_cargo_package_identifier":dependency["unit_cargo_package_identifier"],
                "unit_cargo_target_name":dependency["unit_cargo_target_name"],"unit_cargo_target_kind":dependency["unit_cargo_target_kind"],
                "unit_rust_edition_value":dependency["unit_rust_edition_value"],"unit_module_path_values":[],
                "unit_semantic_identity_record":encode_compilation_unit_value(&root.scope_compilation_unit_key),
            }
        }));
        let binding = NameResolvedBindingRow {
            binding_lexical_scope_key: external.clone(),
            binding_resolved_name_text: alias.clone(),
            binding_namespace_kind_value: NamespaceBindingKindValue::Type,
            binding_target_identity_key: target,
            binding_target_child_scope: Some(root),
            binding_visibility_scope_kind: NameBindingVisibilityKind::Public,
            binding_selection_priority_kind: NameBindingPriorityKind::Prelude,
            binding_import_evidence_keys: vec![],
        };
        projector
            .projection_available_extern_rows
            .insert(alias.clone(), binding.clone());
        let system = dependency["unit_sysroot_context_flag"] == true;
        let implicit = unit["unit_implicit_prelude_flag"] != false
            && (!system
                || (alias == "core" && unit["unit_no_core_flag"] != true)
                || (alias == "std" && unit["unit_no_std_flag"] != true));
        if implicit {
            projector
                .projection_resolution_input_rows
                .name_symbol_binding_rows
                .push(binding.clone());
            if system
                && alias == "std"
                && projector
                    .projection_resolution_input_rows
                    .unit_rust_edition_kind
                    == RustSourceEditionKind::Edition2015
            {
                let mut local = binding;
                local
                    .binding_lexical_scope_key
                    .scope_semantic_path_parts
                    .clear();
                projector
                    .projection_resolution_input_rows
                    .name_symbol_binding_rows
                    .push(local);
            }
        }
        let selected = if unit["unit_no_std_flag"] == true {
            "core"
        } else {
            "std"
        };
        if system
            && alias == selected
            && unit["unit_no_core_flag"] != true
            && unit["unit_implicit_prelude_flag"] != false
        {
            let name = format!(
                "rust_{}",
                unit["unit_rust_edition_value"]
                    .as_str()
                    .ok_or_else(|| build_name_projection_error("edition missing"))?
            );
            let dependency_unit_key = build_unit_compilation_key(dependency)?;
            let source_scope = projector
                .projection_resolution_input_rows
                .name_symbol_binding_rows
                .iter()
                .find(|row| {
                    row.binding_lexical_scope_key.scope_compilation_unit_key == dependency_unit_key
                        && row.binding_lexical_scope_key.scope_semantic_path_parts == ["prelude"]
                        && row.binding_resolved_name_text == name
                })
                .and_then(|row| row.binding_target_child_scope.clone())
                .ok_or_else(|| {
                    build_name_projection_error(format!(
                        "sysroot edition prelude unavailable: {alias}::{name}"
                    ))
                })?;
            let standard = projector
                .projection_resolution_input_rows
                .name_symbol_binding_rows
                .iter()
                .filter(|row| {
                    row.binding_lexical_scope_key == source_scope
                        && row.binding_visibility_scope_kind == NameBindingVisibilityKind::Public
                })
                .cloned()
                .map(|mut row| {
                    row.binding_lexical_scope_key = NameResolutionScopeKey {
                        scope_compilation_unit_key: own.clone(),
                        scope_semantic_path_parts: vec!["@prelude".into()],
                    };
                    row.binding_selection_priority_kind = NameBindingPriorityKind::Prelude;
                    row
                })
                .collect::<Vec<_>>();
            projector
                .projection_resolution_input_rows
                .name_symbol_binding_rows
                .extend(standard);
        }
    }
    if unit["unit_implicit_prelude_flag"] == false {
        projector
            .projection_resolution_input_rows
            .unit_extern_prelude_key = None;
    }
    Ok(())
}

fn build_name_stage_record(table: &str, prefix: &str, value: Value) -> StageTableRowRecord {
    StageTableRowRecord {
        stage_owned_table_name: table.into(),
        stage_row_record_key: format!(
            "{prefix}/{}",
            blake3::hash(value.to_string().as_bytes()).to_hex()
        ),
        stage_row_payload_text: value.to_string(),
    }
}

fn build_namespaced_target_records(
    bindings: &[NameResolvedBindingRow],
    metadata: &BTreeMap<NameSymbolTargetKey, Value>,
) -> Result<Vec<Value>, StoreBackendFailureError> {
    bindings
        .iter()
        .try_fold(
            BTreeMap::<String, (Value, BTreeSet<&str>)>::new(),
            |mut groups, row| {
                let target = metadata
                    .get(&row.binding_target_identity_key)
                    .ok_or_else(|| {
                        build_name_projection_error("resolved target has no source metadata")
                    })?
                    .clone();
                groups
                    .entry(target.to_string())
                    .or_insert_with(|| (target, BTreeSet::new()))
                    .1
                    .insert(encode_namespace_kind_value(
                        row.binding_namespace_kind_value,
                    ));
                Ok(groups)
            },
        )
        .map(|groups| {
            groups
                .into_values()
                .map(|(mut value, namespaces)| {
                    value["target_namespace_kind_values"] = json!(namespaces);
                    value
                })
                .collect()
        })
}

fn build_resolved_crate_records(
    projector: &CrateSyntaxProjectionState<'_>,
    output: NameResolutionOutputRows,
    prefix: &str,
) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
    let own = &projector
        .projection_resolution_input_rows
        .unit_compilation_identity_key;
    let mut rows = output
        .resolved_module_scope_rows
        .iter()
        .filter(|row| &row.scope_module_identity_key.scope_compilation_unit_key == own)
        .map(|row| {
            build_name_stage_record(
                "name-resolution-scope-records",
                prefix,
                encode_scope_record_value(row),
            )
        })
        .collect::<Vec<_>>();
    for binding in output
        .resolved_symbol_binding_rows
        .iter()
        .filter(|row| &row.binding_lexical_scope_key.scope_compilation_unit_key == own)
    {
        let metadata = projector
            .projection_target_metadata_rows
            .get(&binding.binding_target_identity_key)
            .ok_or_else(|| build_name_projection_error("binding target metadata absent"))?
            .clone();
        rows.push(build_name_stage_record(
            "module-scope-binding-table",
            prefix,
            encode_binding_record_value(binding, metadata),
        ));
    }
    for answer in output.resolved_signature_path_rows {
        let mut value = projector
            .projection_path_metadata_rows
            .get(&answer.path_site_occurrence_key)
            .ok_or_else(|| build_name_projection_error("path site metadata absent"))?
            .clone();
        value["path_target_symbol_rows"] = json!(build_namespaced_target_records(
            &answer.path_resolved_target_rows,
            &projector.projection_target_metadata_rows
        )?);
        value["path_candidate_target_rows"] = json!(build_namespaced_target_records(
            &answer.path_candidate_target_rows,
            &projector.projection_target_metadata_rows
        )?);
        value["path_resolution_state_kind"] = json!(match answer.path_resolution_state_kind {
            NamePathAnswerKind::Resolved => "resolved",
            NamePathAnswerKind::Unresolved => "unresolved",
            NamePathAnswerKind::Ambiguous => "ambiguous",
        });
        value["path_unresolved_reason_kind"] = json!(answer.path_unresolved_reason_kind.map(
            |reason| match reason {
                SiteUnresolvedReasonKind::CfgInactive => "cfg_inactive",
                SiteUnresolvedReasonKind::BuildScript => "build_script",
                SiteUnresolvedReasonKind::ProcMacro => "proc_macro",
                SiteUnresolvedReasonKind::ExternalSourceMissing => "external_source_missing",
                SiteUnresolvedReasonKind::UnknownReceiver => "unknown_receiver",
                SiteUnresolvedReasonKind::TraitSolverNeeded => "trait_solver_needed",
            }
        ));
        rows.push(build_name_stage_record(
            "path-resolution-site-records",
            prefix,
            value,
        ));
    }
    rows.extend(
        output
            .resolved_unavailable_reason_rows
            .iter()
            .map(|(key, reason)| {
                build_name_stage_record(
                    "macro-expanded-item-table",
                    prefix,
                    projector.build_unresolved_macro_value(key, *reason),
                )
            }),
    );
    rows.extend(output.resolved_semantic_output_rows);
    rows.extend(projector.projection_lang_definition_rows.clone());
    Ok(rows)
}

pub fn run_persisted_name_resolution(
    stage_run_scope_value: &StageRunScopeValue,
    stage_table_store_port: &dyn StageTableStorePort,
    stage_cancel_flag_value: &RunCancelFlagValue,
) -> Result<usize, StoreBackendFailureError> {
    if stage_cancel_flag_value.check_stage_work_cancelled() {
        return Err(build_name_projection_error("NameResolutionCancelledError"));
    }
    let store = stage_table_store_port;
    let scope = stage_run_scope_value;
    let configuration = read_name_stage_records(store, scope, "workspace-config-state-records")?
        .into_iter()
        .next()
        .ok_or_else(|| build_name_projection_error("workspace configuration missing"))?;
    let units = store
        .read_stage_table_rows(scope, "crate-dependency-graph-table")?
        .into_iter()
        .map(|row| {
            let unit: Value = serde_json::from_str(&row.stage_row_payload_text)
                .map_err(build_name_projection_error)?;
            Ok((row.stage_row_record_key, unit))
        })
        .collect::<Result<BTreeMap<_, _>, StoreBackendFailureError>>()?;
    let mut pending = units
        .iter()
        .filter(|(_, unit)| {
            unit["unit_mapped_target_flag"] == true || unit["unit_sysroot_context_flag"] == true
        })
        .map(|(key, _)| key.clone())
        .collect::<BTreeSet<_>>();
    let mut completed = BTreeSet::new();
    let mut total = 0;
    while !pending.is_empty() {
        if stage_cancel_flag_value.check_stage_work_cancelled() {
            return Err(build_name_projection_error("NameResolutionCancelledError"));
        }
        let ready = pending
            .iter()
            .find(|key| {
                units
                    .iter()
                    .filter(|(_, dependency)| {
                        check_unit_library_dependency(&units[*key], dependency)
                    })
                    .all(|(dependency, _)| completed.contains(dependency))
            })
            .cloned()
            .ok_or_else(|| {
                build_name_projection_error("cyclic or unmapped crate dependency context")
            })?;
        let unit = &units[&ready];
        let mut projector = CrateSyntaxProjectionState::build_crate_projection_state(
            unit.clone(),
            &configuration,
            store,
            scope,
            stage_cancel_flag_value,
        )?;
        append_library_namespace_inputs(&mut projector, unit, &units, store, scope)?;
        projector.project_unit_source_root()?;
        let input = projector.projection_resolution_input_rows.clone();
        if std::env::var_os("PARSELTONGUE_PROFILE_MACRO_ROUNDS").is_some() {
            eprintln!(
                "stage=name-projection-complete unit={ready} scopes={} bindings={} imports={} paths={} macros={}",
                input.name_module_scope_rows.len(),
                input.name_symbol_binding_rows.len(),
                input.name_import_request_rows.len(),
                input.name_signature_path_rows.len(),
                input.name_macro_invocation_rows.len()
            );
        }
        let macro_port = PersistedItemMacroPort::build_persisted_macro_port(projector);
        let output = resolve_crate_name_bindings(&input, &macro_port, stage_cancel_flag_value)
            .map_err(build_name_projection_error)?;
        let projector = macro_port.finish_persisted_macro_port()?;
        let prefix = format!("crate/{ready}");
        let rows = build_resolved_crate_records(&projector, output, &prefix)?;
        total += store.write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: "name-resolution",
                stage_unit_work_key: &prefix,
                stage_input_stage_names: &["syntax-spans"],
                stage_unit_output_rows: &rows,
            },
            stage_cancel_flag_value,
        )?;
        pending.remove(&ready);
        completed.insert(ready);
    }
    store.write_stage_table_unit(
        &StageUnitWriteRequest {
            stage_run_scope_value: scope,
            stage_owner_name_value: "name-resolution",
            stage_unit_work_key: "__complete__",
            stage_input_stage_names: &["syntax-spans"],
            stage_unit_output_rows: &[],
        },
        stage_cancel_flag_value,
    )?;
    Ok(total)
}
