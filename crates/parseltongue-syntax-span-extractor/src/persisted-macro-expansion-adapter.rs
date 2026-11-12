use super::syntax_namespace_input_extraction::CrateSyntaxProjectionState;
use parseltongue_core_row_types::*;
use parseltongue_port_macro_expander::{
    BuiltinMacroLookupPort, expand_builtin_macro_items, expand_declarative_macro_items,
};
use parseltongue_table_store_traits::{
    ItemMacroExpansionPort, ResolvedMacroLookupPort, StageTableStorePort,
};
use std::{cell::RefCell, collections::BTreeMap, sync::Mutex};

pub(super) struct PersistedItemMacroPort<'a> {
    projection_source_state_lock: Mutex<CrateSyntaxProjectionState<'a>>,
    resolved_macro_lookup_port: &'a dyn ResolvedMacroLookupPort,
}

struct ResolvedBuiltinMacroLookup<'a, 'b> {
    invocation_lexical_scope_key: &'a NameResolutionScopeKey,
    resolved_macro_lookup_port: &'b dyn ResolvedMacroLookupPort,
    lookup_failure: RefCell<Option<NameResolutionStageError>>,
}

impl BuiltinMacroLookupPort for ResolvedBuiltinMacroLookup<'_, '_> {
    fn read_builtin_macro_name(&self, path: &[NamePathSegmentKind]) -> Option<String> {
        match self
            .resolved_macro_lookup_port
            .read_resolved_builtin_macro(self.invocation_lexical_scope_key, path)
        {
            Ok(name) => name,
            Err(error) => {
                *self.lookup_failure.borrow_mut() = Some(error);
                None
            }
        }
    }
}

fn read_macro_builtin_name(
    binding: &NameResolvedBindingRow,
    metadata: &BTreeMap<NameSymbolTargetKey, serde_json::Value>,
) -> Option<String> {
    let value = metadata.get(&binding.binding_target_identity_key)?;
    if value["target_symbol_kind_value"] == "BuiltinMacro" {
        value["target_builtin_name_value"]
            .as_str()
            .map(str::to_owned)
    } else {
        value["target_macro_definition_record"]["macro_builtin_identifier_text"]
            .as_str()
            .map(str::to_owned)
    }
}

fn expand_builtin_stage_rows(
    state: &mut CrateSyntaxProjectionState<'_>,
    invocation: &NameMacroInvocationRow,
    name: &str,
    resolved_macro_lookup_port: &dyn ResolvedMacroLookupPort,
) -> Result<NameExpansionRoundRows, NameResolutionStageError> {
    let input = state
        .build_builtin_expansion_input(invocation, name)
        .map_err(build_macro_expansion_error)?;
    let lookup = ResolvedBuiltinMacroLookup {
        invocation_lexical_scope_key: &invocation.invocation_lexical_scope_key,
        resolved_macro_lookup_port,
        lookup_failure: RefCell::new(None),
    };
    let output = expand_builtin_macro_items(&input, &lookup, state.projection_cancel_flag_value);
    if let Some(error) = lookup.lookup_failure.into_inner() {
        return Err(error);
    }
    match output.map_err(|error| {
        if error == MacroExpansionFailureKind::Cancelled {
            NameResolutionStageError::Cancelled
        } else {
            build_macro_expansion_error(error)
        }
    })? {
        BuiltinMacroExpansionOutput::Include(path) => state
            .project_builtin_include_rows(invocation, &path)
            .map_err(build_macro_expansion_error),
        BuiltinMacroExpansionOutput::Unavailable(reason, detail) => {
            Ok(state.record_unavailable_macro_reason(invocation, reason, detail))
        }
        BuiltinMacroExpansionOutput::Expression(_) => Ok(state.record_unavailable_macro_reason(
            invocation,
            SiteUnresolvedReasonKind::ExternalSourceMissing,
            format!("builtin {name} produces an expression, not items"),
        )),
    }
}

fn merge_expansion_round_rows(
    mut round: NameExpansionRoundRows,
    delta: NameExpansionRoundRows,
) -> NameExpansionRoundRows {
    round
        .expansion_module_scope_rows
        .extend(delta.expansion_module_scope_rows);
    round
        .expansion_symbol_binding_rows
        .extend(delta.expansion_symbol_binding_rows);
    round
        .expansion_import_request_rows
        .extend(delta.expansion_import_request_rows);
    round
        .expansion_signature_path_rows
        .extend(delta.expansion_signature_path_rows);
    round
        .expansion_macro_invocation_rows
        .extend(delta.expansion_macro_invocation_rows);
    round
        .expansion_completed_invocation_keys
        .extend(delta.expansion_completed_invocation_keys);
    round
        .expansion_unavailable_reason_rows
        .extend(delta.expansion_unavailable_reason_rows);
    round
        .expansion_semantic_output_rows
        .extend(delta.expansion_semantic_output_rows);
    round
}

pub fn prepare_macro_expansion_port<'a>(
    unit: serde_json::Value,
    workspace: &serde_json::Value,
    store: &'a dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    cancel: &'a RunCancelFlagValue,
    lookup: &'a dyn ResolvedMacroLookupPort,
) -> Result<impl ItemMacroExpansionPort + 'a, StoreBackendFailureError> {
    let mut projection = CrateSyntaxProjectionState::build_crate_projection_state(
        unit, workspace, store, scope, cancel,
    )?;
    projection.project_unit_source_root()?;
    Ok(PersistedItemMacroPort {
        projection_source_state_lock: Mutex::new(projection),
        resolved_macro_lookup_port: lookup,
    })
}

impl ItemMacroExpansionPort for PersistedItemMacroPort<'_> {
    fn expand_resolved_macro_items(
        &self,
        query: &NameMacroExpansionQuery<'_>,
    ) -> Result<NameExpansionRoundRows, NameResolutionStageError> {
        let mut state = self
            .projection_source_state_lock
            .lock()
            .map_err(build_macro_expansion_error)?;
        if std::env::var_os("PARSELTONGUE_PROFILE_MACRO_ROUNDS").is_some() {
            eprintln!(
                "stage=macro-round pending={} resolved={} scopes={} bindings={} imports={} paths={}",
                query.expansion_pending_invocation_rows.len(),
                query.expansion_resolved_macro_rows.len(),
                query
                    .expansion_input_record_rows
                    .name_module_scope_rows
                    .len(),
                query.expansion_known_binding_rows.len(),
                query
                    .expansion_input_record_rows
                    .name_import_request_rows
                    .len(),
                query
                    .expansion_input_record_rows
                    .name_signature_path_rows
                    .len()
            );
        }
        query.expansion_resolved_macro_rows.iter().try_fold(
            NameExpansionRoundRows::default(),
            |round, (key, binding)| {
                if state
                    .projection_cancel_flag_value
                    .check_stage_work_cancelled()
                {
                    return Err(NameResolutionStageError::Cancelled);
                }
                let invocation = query
                    .expansion_pending_invocation_rows
                    .iter()
                    .find(|row| &row.invocation_site_occurrence_key == key)
                    .ok_or_else(|| {
                        build_macro_expansion_error("resolved macro invocation missing")
                    })?;
                let metadata = self
                    .resolved_macro_lookup_port
                    .read_macro_target_metadata(&binding.binding_target_identity_key)?
                    .ok_or_else(|| build_macro_expansion_error("macro target metadata absent"))?;
                state.projection_target_metadata_rows.insert(
                    binding.binding_target_identity_key.clone(),
                    serde_json::from_str(&metadata.stage_row_payload_text)
                        .map_err(build_macro_expansion_error)?,
                );
                if std::env::var_os("PARSELTONGUE_PROFILE_MACRO_ROUNDS").is_some()
                    && query.expansion_resolved_macro_rows.len() <= 16
                {
                    eprintln!(
                        "stage=macro-item name={} argument_digest={} argument_bytes={}",
                        binding.binding_resolved_name_text,
                        blake3::hash(invocation.invocation_argument_source_text.as_bytes())
                            .to_hex(),
                        invocation.invocation_argument_source_text.len()
                    );
                }
                if let Some(name) =
                    read_macro_builtin_name(binding, &state.projection_target_metadata_rows)
                {
                    let delta = expand_builtin_stage_rows(
                        &mut state,
                        invocation,
                        &name,
                        self.resolved_macro_lookup_port,
                    )?;
                    return Ok(merge_expansion_round_rows(round, delta));
                }
                let definition = state
                    .projection_target_metadata_rows
                    .get(&binding.binding_target_identity_key)
                    .and_then(|value| value.get("target_macro_definition_record"))
                    .cloned()
                    .ok_or_else(|| build_macro_expansion_error("macro definition source absent"))?;
                let edition = match definition["macro_definition_edition_kind"].as_str() {
                    Some("2015") => RustSourceEditionKind::Edition2015,
                    Some("2018") => RustSourceEditionKind::Edition2018,
                    Some("2021") => RustSourceEditionKind::Edition2021,
                    Some("2024") => RustSourceEditionKind::Edition2024,
                    _ => {
                        return Err(build_macro_expansion_error(
                            "macro definition edition absent",
                        ));
                    }
                };
                let expanded = expand_declarative_macro_items(
                    &DeclarativeMacroExpansionInput {
                        macro_declaration_style_flag: definition["macro_declaration_style_flag"]
                            == true,
                        macro_definition_source_text:
                            super::syntax_namespace_record_codec::read_record_string_value(
                                &definition,
                                "macro_definition_source_text",
                            )
                            .map_err(build_macro_expansion_error)?
                            .into(),
                        macro_pattern_source_text: definition["macro_pattern_source_text"]
                            .as_str()
                            .map(str::to_owned),
                        macro_argument_source_text: invocation
                            .invocation_argument_source_text
                            .clone(),
                        macro_definition_edition_kind: edition,
                        macro_invocation_edition_kind: query
                            .expansion_input_record_rows
                            .unit_rust_edition_kind,
                    },
                    state.projection_cancel_flag_value,
                )
                .map_err(|error| {
                    if error == MacroExpansionFailureKind::Cancelled {
                        NameResolutionStageError::Cancelled
                    } else {
                        build_macro_expansion_error(format!(
                            "{error}; source={} byte={}",
                            definition["macro_source_absolute_path"],
                            definition["macro_source_range_start"]
                        ))
                    }
                })?;
                let delta = state
                    .project_macro_expansion_rows(invocation, &definition, &expanded)
                    .map_err(build_macro_expansion_error)?;
                Ok(merge_expansion_round_rows(round, delta))
            },
        )
    }
}

fn build_macro_expansion_error(error: impl std::fmt::Display) -> NameResolutionStageError {
    NameResolutionStageError::Expansion(WorkspaceNotCompilingError {
        error_context_detail_text: format!("WorkspaceNotCompilingError: {error}"),
    })
}
