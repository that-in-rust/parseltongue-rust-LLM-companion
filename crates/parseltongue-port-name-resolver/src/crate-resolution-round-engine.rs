use crate::{
    FIXED_POINT_LIMIT, ItemMacroExpansionPort,
    namespace_binding_normalization_rules::{
        intersect_binding_visibility_scopes, normalize_scope_binding_rows,
    },
    scope_path_resolution_rules::{ResolvedPathLookupKind, ScopeBindingLookupState},
};
use parseltongue_core_row_types::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};

struct CrateResolutionRoundState {
    round_pending_input_rows: NameResolutionInputRows,
    round_completed_invocation_keys: BTreeSet<SemanticOccurrenceKeyValue>,
    round_unavailable_reason_rows: BTreeMap<SemanticOccurrenceKeyValue, SiteUnresolvedReasonKind>,
    round_semantic_output_rows: Vec<StageTableRowRecord>,
    round_changing_macro_count: usize,
}

fn build_resolution_input_error(message: impl Into<String>) -> NameResolutionStageError {
    NameResolutionStageError::Input(StageInputMissingError {
        error_context_detail_text: message.into(),
    })
}

fn check_resolution_work_cancelled(
    cancel: &RunCancelFlagValue,
) -> Result<(), NameResolutionStageError> {
    if cancel.check_stage_work_cancelled() {
        Err(NameResolutionStageError::Cancelled)
    } else {
        Ok(())
    }
}

fn merge_canonical_record_rows<T: Clone + Eq, K: Ord>(
    original: Vec<T>,
    added: Vec<T>,
    key: impl Fn(&T) -> K,
) -> Result<Vec<T>, NameResolutionStageError> {
    original
        .into_iter()
        .chain(added)
        .try_fold(BTreeMap::new(), |mut rows, row| {
            let id = key(&row);
            if rows.get(&id).is_some_and(|existing| existing != &row) {
                return Err(build_resolution_input_error(
                    "conflicting payloads for one canonical record key",
                ));
            }
            rows.insert(id, row);
            Ok(rows)
        })
        .map(|rows| rows.into_values().collect())
}

fn canonicalize_resolution_input_rows(
    mut input: NameResolutionInputRows,
) -> Result<NameResolutionInputRows, NameResolutionStageError> {
    input.name_module_scope_rows =
        merge_canonical_record_rows(input.name_module_scope_rows, vec![], |row| {
            row.scope_module_identity_key.clone()
        })?;
    input.name_symbol_binding_rows = input
        .name_symbol_binding_rows
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    input.name_import_request_rows =
        merge_canonical_record_rows(input.name_import_request_rows, vec![], |row| {
            row.import_site_occurrence_key.clone()
        })?;
    input.name_signature_path_rows =
        merge_canonical_record_rows(input.name_signature_path_rows, vec![], |row| {
            row.path_site_occurrence_key.clone()
        })?;
    input.name_macro_invocation_rows =
        merge_canonical_record_rows(input.name_macro_invocation_rows, vec![], |row| {
            row.invocation_site_occurrence_key.clone()
        })?;
    validate_resolution_input_scopes(&input)?;
    Ok(input)
}

fn validate_resolution_input_scopes(
    input: &NameResolutionInputRows,
) -> Result<(), NameResolutionStageError> {
    let scopes: BTreeMap<_, _> = input
        .name_module_scope_rows
        .iter()
        .map(|row| (&row.scope_module_identity_key, row))
        .collect();
    let referenced = input
        .name_module_scope_rows
        .iter()
        .flat_map(|row| {
            row.scope_parent_identity_key
                .iter()
                .chain(row.scope_implicit_prelude_key.iter())
        })
        .chain(input.unit_extern_prelude_key.iter())
        .chain(input.name_symbol_binding_rows.iter().flat_map(|row| {
            std::iter::once(&row.binding_lexical_scope_key)
                .chain(row.binding_target_child_scope.iter())
        }))
        .chain(
            input
                .name_import_request_rows
                .iter()
                .map(|row| &row.import_lexical_scope_key),
        )
        .chain(
            input
                .name_signature_path_rows
                .iter()
                .map(|row| &row.path_lexical_scope_key),
        )
        .chain(
            input
                .name_macro_invocation_rows
                .iter()
                .map(|row| &row.invocation_lexical_scope_key),
        );
    if referenced.into_iter().any(|key| !scopes.contains_key(key)) {
        return Err(build_resolution_input_error(
            "name-resolution row references a missing scope",
        ));
    }
    input.name_module_scope_rows.iter().try_for_each(|row| {
        std::iter::successors(Some(&row.scope_module_identity_key), |key| {
            scopes.get(key)?.scope_parent_identity_key.as_ref()
        })
        .take(scopes.len() + 1)
        .try_fold(BTreeSet::new(), |mut seen, key| {
            if !seen.insert(key) {
                return Err(build_resolution_input_error("cyclic scope parent chain"));
            }
            Ok(seen)
        })
        .map(|_| ())
    })
}

fn derive_import_binding_rows(
    input: &NameResolutionInputRows,
    bindings: &[NameResolvedBindingRow],
) -> Vec<NameResolvedBindingRow> {
    let lookup = ScopeBindingLookupState::build_namespace_lookup_rows(input, bindings);
    input
        .name_import_request_rows
        .iter()
        .flat_map(|request| {
            let resolved = lookup.resolve_scope_name_path(
                &request.import_lexical_scope_key,
                &request.import_path_segment_rows,
                NameNamespaceFilterKind::Any,
                request.import_resolution_source_kind,
            );
            let ResolvedPathLookupKind::Resolved(targets) = resolved else {
                return Vec::new();
            };
            let selected = if request.import_glob_expansion_flag {
                targets
                    .iter()
                    .filter_map(|row| row.binding_target_child_scope.as_ref())
                    .flat_map(|scope| {
                        bindings
                            .iter()
                            .filter(move |row| &row.binding_lexical_scope_key == scope)
                    })
                    .filter(|row| {
                        crate::namespace_binding_normalization_rules::check_binding_scope_visible(
                            &row.binding_visibility_scope_kind,
                            &request.import_lexical_scope_key,
                        )
                    })
                    .cloned()
                    .collect()
            } else {
                targets
            };
            selected
                .into_iter()
                .map(|mut row| {
                    row.binding_lexical_scope_key = request.import_lexical_scope_key.clone();
                    if !request.import_glob_expansion_flag {
                        row.binding_resolved_name_text =
                            request
                                .import_binding_alias_text
                                .clone()
                                .or_else(|| {
                                    request.import_path_segment_rows.iter().rev().find_map(
                                        |segment| match segment {
                                            NamePathSegmentKind::Name(name) => Some(name.clone()),
                                            _ => None,
                                        },
                                    )
                                })
                                .unwrap_or_else(|| row.binding_resolved_name_text.clone());
                    }
                    row.binding_selection_priority_kind = if request.import_glob_expansion_flag {
                        NameBindingPriorityKind::Glob
                    } else {
                        NameBindingPriorityKind::Explicit
                    };
                    row.binding_visibility_scope_kind = intersect_binding_visibility_scopes(
                        &row.binding_visibility_scope_kind,
                        &request.import_visibility_scope_kind,
                    );
                    row.binding_import_evidence_keys
                        .push(request.import_site_occurrence_key.clone());
                    row
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn settle_crate_import_bindings(
    input: &NameResolutionInputRows,
    cancel: &RunCancelFlagValue,
) -> Result<Vec<NameResolvedBindingRow>, NameResolutionStageError> {
    let seed = normalize_scope_binding_rows(input.name_symbol_binding_rows.clone());
    let result = (0usize..).try_fold(
        (seed.clone(), seed, 1usize, 0usize),
        |(current, anchor, power, steps), _| {
            if let Err(error) = check_resolution_work_cancelled(cancel) {
                return ControlFlow::Break(Err(error));
            }
            let next = normalize_scope_binding_rows(
                input
                    .name_symbol_binding_rows
                    .iter()
                    .cloned()
                    .chain(derive_import_binding_rows(input, &current)),
            );
            if next == current {
                return ControlFlow::Break(Ok(next));
            }
            if next == anchor {
                return ControlFlow::Break(Err(build_resolution_input_error(
                    "cyclic non-converging import binding states",
                )));
            }
            let steps = steps + 1;
            if steps == power {
                ControlFlow::Continue((next.clone(), next, power.saturating_mul(2), 0))
            } else {
                ControlFlow::Continue((next, anchor, power, steps))
            }
        },
    );
    match result {
        ControlFlow::Break(result) => result,
        ControlFlow::Continue(_) => Err(build_resolution_input_error("import iteration exhausted")),
    }
}

fn resolve_signature_path_answers(
    input: &NameResolutionInputRows,
    bindings: &[NameResolvedBindingRow],
) -> Vec<NamePathAnswerRow> {
    let lookup = ScopeBindingLookupState::build_namespace_lookup_rows(input, bindings);
    input
        .name_signature_path_rows
        .iter()
        .map(|request| {
            let resolved = if request.path_unavailable_reason_kind.is_some() {
                ResolvedPathLookupKind::Missing
            } else {
                lookup.resolve_scope_name_path(
                    &request.path_lexical_scope_key,
                    &request.path_source_segment_rows,
                    request.path_requested_namespace_kind,
                    request.path_resolution_source_kind,
                )
            };
            let (kind, targets, candidates, reason) = match resolved {
                ResolvedPathLookupKind::Resolved(rows) => {
                    (NamePathAnswerKind::Resolved, rows, vec![], None)
                }
                ResolvedPathLookupKind::Ambiguous(rows) => {
                    (NamePathAnswerKind::Ambiguous, vec![], rows, None)
                }
                ResolvedPathLookupKind::Missing => (
                    NamePathAnswerKind::Unresolved,
                    vec![],
                    vec![],
                    Some(
                        request
                            .path_unavailable_reason_kind
                            .unwrap_or(SiteUnresolvedReasonKind::ExternalSourceMissing),
                    ),
                ),
            };
            NamePathAnswerRow {
                path_site_occurrence_key: request.path_site_occurrence_key.clone(),
                path_resolution_state_kind: kind,
                path_resolved_target_rows: targets,
                path_candidate_target_rows: candidates,
                path_unresolved_reason_kind: reason,
            }
        })
        .collect()
}

fn finish_crate_resolution_output(
    state: CrateResolutionRoundState,
    bindings: Vec<NameResolvedBindingRow>,
) -> NameResolutionOutputRows {
    let mut reasons = state.round_unavailable_reason_rows;
    state
        .round_pending_input_rows
        .name_macro_invocation_rows
        .iter()
        .for_each(|row| {
            reasons
                .entry(row.invocation_site_occurrence_key.clone())
                .or_insert(SiteUnresolvedReasonKind::ExternalSourceMissing);
        });
    NameResolutionOutputRows {
        resolved_signature_path_rows: resolve_signature_path_answers(
            &state.round_pending_input_rows,
            &bindings,
        ),
        resolved_module_scope_rows: state.round_pending_input_rows.name_module_scope_rows,
        resolved_symbol_binding_rows: bindings,
        resolved_completed_invocation_keys: state
            .round_completed_invocation_keys
            .into_iter()
            .collect(),
        resolved_unavailable_reason_rows: reasons.into_iter().collect(),
        resolved_semantic_output_rows: state.round_semantic_output_rows,
        resolved_changing_macro_rounds: state.round_changing_macro_count,
    }
}

fn merge_expansion_round_state(
    mut state: CrateResolutionRoundState,
    delta: NameExpansionRoundRows,
) -> Result<(CrateResolutionRoundState, bool), NameResolutionStageError> {
    let pending: BTreeSet<_> = state
        .round_pending_input_rows
        .name_macro_invocation_rows
        .iter()
        .map(|row| row.invocation_site_occurrence_key.clone())
        .collect();
    if delta
        .expansion_completed_invocation_keys
        .iter()
        .chain(
            delta
                .expansion_unavailable_reason_rows
                .iter()
                .map(|row| &row.0),
        )
        .any(|key| !pending.contains(key))
    {
        return Err(build_resolution_input_error(
            "expansion refers to an unknown pending invocation",
        ));
    }
    let input = &mut state.round_pending_input_rows;
    let new_macros = delta.expansion_macro_invocation_rows.iter().any(|row| {
        !pending.contains(&row.invocation_site_occurrence_key)
            && !state
                .round_completed_invocation_keys
                .contains(&row.invocation_site_occurrence_key)
    });
    let before = input.name_module_scope_rows.len()
        + input.name_symbol_binding_rows.len()
        + input.name_import_request_rows.len()
        + input.name_signature_path_rows.len()
        + state.round_semantic_output_rows.len();
    input.name_module_scope_rows = merge_canonical_record_rows(
        std::mem::take(&mut input.name_module_scope_rows),
        delta.expansion_module_scope_rows,
        |row| row.scope_module_identity_key.clone(),
    )?;
    input.name_symbol_binding_rows = std::mem::take(&mut input.name_symbol_binding_rows)
        .into_iter()
        .chain(delta.expansion_symbol_binding_rows)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    input.name_import_request_rows = merge_canonical_record_rows(
        std::mem::take(&mut input.name_import_request_rows),
        delta.expansion_import_request_rows,
        |row| row.import_site_occurrence_key.clone(),
    )?;
    input.name_signature_path_rows = merge_canonical_record_rows(
        std::mem::take(&mut input.name_signature_path_rows),
        delta.expansion_signature_path_rows,
        |row| row.path_site_occurrence_key.clone(),
    )?;
    input.name_macro_invocation_rows = merge_canonical_record_rows(
        std::mem::take(&mut input.name_macro_invocation_rows),
        delta.expansion_macro_invocation_rows,
        |row| row.invocation_site_occurrence_key.clone(),
    )?;
    state.round_semantic_output_rows = merge_canonical_record_rows(
        state.round_semantic_output_rows,
        delta.expansion_semantic_output_rows,
        |row| {
            (
                row.stage_owned_table_name.clone(),
                row.stage_row_record_key.clone(),
            )
        },
    )?;
    state
        .round_completed_invocation_keys
        .extend(delta.expansion_completed_invocation_keys);
    input.name_macro_invocation_rows.retain(|row| {
        !state
            .round_completed_invocation_keys
            .contains(&row.invocation_site_occurrence_key)
    });
    state
        .round_unavailable_reason_rows
        .extend(delta.expansion_unavailable_reason_rows);
    state
        .round_unavailable_reason_rows
        .retain(|key, _| !state.round_completed_invocation_keys.contains(key));
    validate_resolution_input_scopes(input)?;
    let after = input.name_module_scope_rows.len()
        + input.name_symbol_binding_rows.len()
        + input.name_import_request_rows.len()
        + input.name_signature_path_rows.len()
        + state.round_semantic_output_rows.len();
    Ok((state, new_macros || after > before))
}

pub(crate) fn run_crate_resolution_rounds(
    input: &NameResolutionInputRows,
    port: &dyn ItemMacroExpansionPort,
    cancel: &RunCancelFlagValue,
) -> Result<NameResolutionOutputRows, NameResolutionStageError> {
    check_resolution_work_cancelled(cancel)?;
    let initial = CrateResolutionRoundState {
        round_pending_input_rows: canonicalize_resolution_input_rows(input.clone())?,
        round_completed_invocation_keys: BTreeSet::new(),
        round_unavailable_reason_rows: BTreeMap::new(),
        round_semantic_output_rows: Vec::new(),
        round_changing_macro_count: 0,
    };
    let result = (0..=FIXED_POINT_LIMIT).try_fold(initial, |state, _| {
        let run = || {
            check_resolution_work_cancelled(cancel)?;
            let bindings = settle_crate_import_bindings(&state.round_pending_input_rows, cancel)?;
            if state
                .round_pending_input_rows
                .name_macro_invocation_rows
                .is_empty()
            {
                return Ok(ControlFlow::Break(finish_crate_resolution_output(
                    state, bindings,
                )));
            }
            let lookup = ScopeBindingLookupState::build_namespace_lookup_rows(
                &state.round_pending_input_rows,
                &bindings,
            );
            let resolved = state
                .round_pending_input_rows
                .name_macro_invocation_rows
                .iter()
                .filter_map(|invocation| {
                    match lookup.resolve_scope_name_path(
                        &invocation.invocation_lexical_scope_key,
                        &invocation.invocation_macro_path_segments,
                        NameNamespaceFilterKind::Macro,
                        NameResolutionPathKind::Signature,
                    ) {
                        ResolvedPathLookupKind::Resolved(rows) => {
                            rows.into_iter().next().map(|binding| {
                                (invocation.invocation_site_occurrence_key.clone(), binding)
                            })
                        }
                        _ => None,
                    }
                })
                .collect();
            let delta = port.expand_resolved_macro_items(&NameMacroExpansionQuery {
                expansion_input_record_rows: &state.round_pending_input_rows,
                expansion_known_binding_rows: &bindings,
                expansion_pending_invocation_rows: &state
                    .round_pending_input_rows
                    .name_macro_invocation_rows,
                expansion_resolved_macro_rows: resolved,
            })?;
            check_resolution_work_cancelled(cancel)?;
            let (mut next, changed) = merge_expansion_round_state(state, delta)?;
            if !changed {
                return Ok(ControlFlow::Break(finish_crate_resolution_output(
                    next, bindings,
                )));
            }
            if next.round_changing_macro_count == FIXED_POINT_LIMIT {
                return Err(NameResolutionStageError::Stuck(NameResolutionStuckError {
                    error_context_detail_text:
                        "name resolution changed on attempted macro round 8193".into(),
                }));
            }
            next.round_changing_macro_count += 1;
            Ok(ControlFlow::Continue(next))
        };
        match run() {
            Ok(ControlFlow::Continue(state)) => ControlFlow::Continue(state),
            Ok(ControlFlow::Break(output)) => ControlFlow::Break(Ok(output)),
            Err(error) => ControlFlow::Break(Err(error)),
        }
    });
    match result {
        ControlFlow::Break(result) => result,
        ControlFlow::Continue(_) => Err(build_resolution_input_error("macro iteration exhausted")),
    }
}
