use super::{
    FIXED_POINT_LIMIT,
    namespace_contract_input_codec::{error, key},
    namespace_contract_resolution_rounds::resolve_contract_input,
};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::ItemMacroExpansionPort;
use std::collections::BTreeSet;

pub(super) fn resolve_with_macro_rounds(
    mut input: NameResolutionInputRows,
    macros: &dyn ItemMacroExpansionPort,
    cancel: &RunCancelFlagValue,
) -> Result<NameResolutionOutputRows, NameResolutionStageError> {
    let mut completed = BTreeSet::new();
    let mut unavailable = Vec::new();
    let mut semantic = Vec::new();
    let mut changes = 0;
    canonicalize_source_rows(&mut input);
    let mut output = resolve_contract_input(&input, cancel)?;
    loop {
        if cancel.check_stage_work_cancelled() {
            return Err(NameResolutionStageError::Cancelled);
        }
        let pending = input
            .name_macro_invocation_rows
            .iter()
            .filter(|invocation| !completed.contains(&invocation.invocation_site_occurrence_key))
            .cloned()
            .collect::<Vec<_>>();
        if pending.is_empty() {
            break;
        }
        let resolved = resolve_pending_macros(&input, &output, &pending, cancel)?;
        let expansion = macros.expand_resolved_macro_items(&NameMacroExpansionQuery {
            expansion_input_record_rows: &input,
            expansion_known_binding_rows: &output.resolved_symbol_binding_rows,
            expansion_pending_invocation_rows: &pending,
            expansion_resolved_macro_rows: resolved,
        })?;
        let current_keys = pending
            .iter()
            .map(|invocation| &invocation.invocation_site_occurrence_key)
            .collect::<BTreeSet<_>>();
        if expansion
            .expansion_completed_invocation_keys
            .iter()
            .any(|occurrence| !current_keys.contains(occurrence))
        {
            return Err(error(
                "macro completed an invocation outside its pending query",
            ));
        }
        let proposed_source_change = !expansion.expansion_module_scope_rows.is_empty()
            || !expansion.expansion_symbol_binding_rows.is_empty()
            || !expansion.expansion_import_request_rows.is_empty()
            || !expansion.expansion_signature_path_rows.is_empty();
        let known = input
            .name_macro_invocation_rows
            .iter()
            .map(key)
            .collect::<BTreeSet<_>>();
        let fresh = expansion
            .expansion_macro_invocation_rows
            .iter()
            .any(|invocation| !known.contains(&key(invocation)));
        if (proposed_source_change || fresh) && changes == FIXED_POINT_LIMIT {
            return Err(NameResolutionStageError::Stuck(NameResolutionStuckError {
                error_context_detail_text: "macro fixed point exceeded 8192 changing rounds".into(),
            }));
        }
        let completed_before = completed.len();
        let source_count = source_row_count(&input);
        completed.extend(expansion.expansion_completed_invocation_keys);
        completed.extend(
            expansion
                .expansion_unavailable_reason_rows
                .iter()
                .map(|(occurrence, _)| occurrence.clone()),
        );
        unavailable.extend(expansion.expansion_unavailable_reason_rows);
        semantic.extend(expansion.expansion_semantic_output_rows);
        input
            .name_module_scope_rows
            .extend(expansion.expansion_module_scope_rows);
        input
            .name_symbol_binding_rows
            .extend(expansion.expansion_symbol_binding_rows);
        input
            .name_import_request_rows
            .extend(expansion.expansion_import_request_rows);
        input
            .name_signature_path_rows
            .extend(expansion.expansion_signature_path_rows);
        input
            .name_macro_invocation_rows
            .extend(expansion.expansion_macro_invocation_rows);
        input.name_module_scope_rows.sort();
        input.name_module_scope_rows.dedup();
        input.name_symbol_binding_rows.sort();
        input.name_symbol_binding_rows.dedup();
        input.name_import_request_rows.sort();
        input.name_import_request_rows.dedup();
        input.name_macro_invocation_rows.sort();
        input.name_macro_invocation_rows.dedup();
        canonicalize_source_rows(&mut input);
        let source_change = source_count != source_row_count(&input);
        input
            .name_macro_invocation_rows
            .retain(|invocation| !completed.contains(&invocation.invocation_site_occurrence_key));
        if source_change || fresh {
            changes += 1;
            if source_change {
                output = resolve_contract_input(&input, cancel)?;
            }
        } else if completed_before == completed.len() {
            break;
        }
    }
    output.resolved_completed_invocation_keys = completed.into_iter().collect();
    unavailable.sort_by(|left, right| left.0.cmp(&right.0));
    unavailable.dedup();
    semantic.sort_by(|left, right| {
        (
            &left.stage_owned_table_name,
            &left.stage_row_record_key,
            &left.stage_row_payload_text,
        )
            .cmp(&(
                &right.stage_owned_table_name,
                &right.stage_row_record_key,
                &right.stage_row_payload_text,
            ))
    });
    semantic.dedup();
    output.resolved_unavailable_reason_rows = unavailable;
    output.resolved_semantic_output_rows = semantic;
    output.resolved_changing_macro_rounds = changes;
    Ok(output)
}

fn source_row_count(input: &NameResolutionInputRows) -> usize {
    input.name_module_scope_rows.len()
        + input.name_symbol_binding_rows.len()
        + input.name_import_request_rows.len()
        + input.name_signature_path_rows.len()
}

fn canonicalize_source_rows(input: &mut NameResolutionInputRows) {
    input.name_module_scope_rows.sort();
    input.name_module_scope_rows.dedup();
    input.name_symbol_binding_rows.sort();
    input.name_symbol_binding_rows.dedup();
    input.name_import_request_rows.sort();
    input.name_import_request_rows.dedup();
    input.name_signature_path_rows.sort_by_key(key);
    input.name_signature_path_rows.dedup();
}

fn resolve_pending_macros(
    input: &NameResolutionInputRows,
    output: &NameResolutionOutputRows,
    pending: &[NameMacroInvocationRow],
    cancel: &RunCancelFlagValue,
) -> Result<Vec<(SemanticOccurrenceKeyValue, NameResolvedBindingRow)>, NameResolutionStageError> {
    if !input
        .all_name_binding_rows()
        .any(|binding| binding.binding_namespace_kind_value == NamespaceBindingKindValue::Macro)
    {
        return Ok(vec![]);
    }
    let mut lookup_input = input.clone();
    lookup_input.name_import_request_rows.clear();
    lookup_input.name_macro_invocation_rows.clear();
    lookup_input.name_signature_path_rows = pending
        .iter()
        .map(|invocation| NamePathRequestRow {
            path_site_occurrence_key: invocation.invocation_site_occurrence_key.clone(),
            path_lexical_scope_key: invocation.invocation_lexical_scope_key.clone(),
            path_source_segment_rows: invocation.invocation_macro_path_segments.clone(),
            path_requested_namespace_kind: NameNamespaceFilterKind::Macro,
            path_resolution_source_kind: NameResolutionPathKind::Signature,
            path_unavailable_reason_kind: None,
        })
        .collect();
    lookup_input.name_symbol_binding_rows = output.resolved_symbol_binding_rows.clone();
    let answers = resolve_contract_input(&lookup_input, cancel)?;
    Ok(answers
        .resolved_signature_path_rows
        .into_iter()
        .filter(|answer| answer.path_resolution_state_kind == NamePathAnswerKind::Resolved)
        .flat_map(|answer| {
            answer
                .path_resolved_target_rows
                .into_iter()
                .map(move |binding| (answer.path_site_occurrence_key.clone(), binding))
        })
        .collect())
}
