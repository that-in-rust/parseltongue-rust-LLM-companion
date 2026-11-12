use super::{
    crate_syntax_namespace_projection::CrateSyntaxProjectionState,
    name_resolution_record_codec::build_name_projection_error,
};
use parseltongue_core_row_types::*;
use parseltongue_port_macro_expander::expand_declarative_macro_items;
use parseltongue_port_name_resolver::ItemMacroExpansionPort;
use std::sync::Mutex;

pub(super) struct PersistedItemMacroPort<'a> {
    projection_source_state_lock: Mutex<CrateSyntaxProjectionState<'a>>,
}

impl<'a> PersistedItemMacroPort<'a> {
    pub(super) fn build_persisted_macro_port(projection: CrateSyntaxProjectionState<'a>) -> Self {
        Self {
            projection_source_state_lock: Mutex::new(projection),
        }
    }
    pub(super) fn finish_persisted_macro_port(
        self,
    ) -> Result<CrateSyntaxProjectionState<'a>, StoreBackendFailureError> {
        self.projection_source_state_lock
            .into_inner()
            .map_err(build_name_projection_error)
    }
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
        query.expansion_resolved_macro_rows.iter().try_fold(
            NameExpansionRoundRows::default(),
            |mut round, (key, binding)| {
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
                        macro_definition_source_text:
                            super::name_resolution_record_codec::read_record_string_value(
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
                        build_macro_expansion_error(error)
                    }
                })?;
                let delta = state
                    .project_macro_expansion_rows(invocation, &definition, &expanded)
                    .map_err(build_macro_expansion_error)?;
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
                Ok(round)
            },
        )
    }
}

fn build_macro_expansion_error(error: impl std::fmt::Display) -> NameResolutionStageError {
    NameResolutionStageError::Expansion(WorkspaceNotCompilingError {
        error_context_detail_text: format!("WorkspaceNotCompilingError: {error}"),
    })
}
