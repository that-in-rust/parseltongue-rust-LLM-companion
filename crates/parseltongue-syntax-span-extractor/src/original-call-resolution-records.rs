use crate::expanded_site_origin_records::{ExpandedTokenOriginIndex, read_expanded_token_origins};
use crate::external_named_target_records::decode_external_name_record;
use crate::impl_header_source_records::{
    ImplHeaderSourceContext, ImplHeaderSourceError, read_impl_source_context,
    read_impl_stage_values,
};
use crate::signature_type_edge_records::read_stored_reason_kind;
use crate::syntax_namespace_record_codec::{
    decode_occurrence_identity_value, decode_symbol_target_value,
};
use crate::type_relative_member_records::{
    BuiltinMemberNameOutcome, ImplMemberLookupIndex, read_impl_member_index,
};
use crate::{SyntaxFileInputData, restore_source_byte_offset};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use ra_ap_syntax::{
    AstNode, SyntaxKind, SyntaxNode,
    ast::{self, HasGenericArgs, HasGenericParams, HasName},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[cfg(test)]
#[path = "callee-path-call-tests.rs"]
mod callee_path_call_tests;
#[cfg(test)]
#[path = "cast-receiver-member-tests.rs"]
mod cast_receiver_member_tests;
#[cfg(test)]
#[path = "original-call-resolution-tests.rs"]
pub(crate) mod original_call_resolution_tests;
#[cfg(test)]
#[path = "self-argument-member-tests.rs"]
mod self_argument_member_tests;

#[derive(Debug, thiserror::Error)]
pub enum OriginalCallResolutionError {
    #[error("original call resolution cancelled")]
    Cancelled,
    #[error("original call store: {0}")]
    Store(#[from] StoreBackendFailureError),
    #[error("original call identity: {0}")]
    Identity(#[from] ImplHeaderSourceError),
    #[error("original call evidence field is invalid: {field}")]
    InvalidEvidence { field: &'static str },
    #[error("original call input scope does not match its graph")]
    InvalidScope,
}

/// Resolve only original, graph-owned calls. Namespace resolution remains a
/// prior DuckDB stage; this function consumes its persisted type-prefix facts.
/// The caller supplies the same frozen source scope used to assemble `input`;
/// this reader never publishes another snapshot or advances a serving pointer.
pub fn resolve_original_call_edges(
    store: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    input: &HldMaterializationInputRecord,
    cancel: &RunCancelFlagValue,
) -> Result<Vec<HldEdgeInputRecord>, OriginalCallResolutionError> {
    if cancel.check_stage_work_cancelled() {
        return Err(OriginalCallResolutionError::Cancelled);
    }
    if source_scope.run_generation_digest_value
        != blake3::Hash::from(input.run_generation_digest_value.0)
            .to_hex()
            .as_str()
        || input.source_coverage_kind != HldSourceCoverageKind::FullSemantic
    {
        return Err(OriginalCallResolutionError::InvalidScope);
    }
    let workspace = read_impl_stage_values(store, source_scope, "workspace-config-state-records")?;
    let workspace = workspace
        .first()
        .and_then(|row| row["workspace_source_root_path"].as_str())
        .unwrap_or("");
    let context = read_impl_source_context(store, source_scope, Path::new(workspace))?;
    let members = read_impl_member_index(store, source_scope, &input.input_node_records)?;
    let endpoints = input
        .input_node_records
        .iter()
        .filter(|node| {
            !node.node_function_body_ancestry_flag
                && node.node_input_category == HldNodeInputCategory::LocalItem
        })
        .map(|node| &node.node_target_reference)
        .collect::<BTreeSet<_>>();
    let lookup = build_bound_method_index(store, source_scope, &context, input)?;
    let origins = read_expanded_token_origins(store, source_scope)?;
    let mut edges = Vec::new();
    for site in read_impl_stage_values(store, source_scope, "body-site-reference-table")? {
        if cancel.check_stage_work_cancelled() {
            return Err(OriginalCallResolutionError::Cancelled);
        }
        if let Some(edge) = build_original_call_edge(
            &site,
            &context,
            &members,
            &endpoints,
            &lookup,
            &origins,
            input.run_generation_digest_value,
        )? {
            edges.push(edge);
        }
    }
    edges.sort();
    edges.dedup();
    Ok(edges)
}

fn build_original_call_edge(
    site: &Value,
    context: &ImplHeaderSourceContext<'_>,
    members: &ImplMemberLookupIndex,
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
    lookup: &BoundMethodLookupIndex,
    origins: &ExpandedTokenOriginIndex,
    generation: RunGenerationDigestValue,
) -> Result<Option<HldEdgeInputRecord>, OriginalCallResolutionError> {
    if !matches!(
        site["site_syntax_kind_value"].as_str(),
        Some("CALL_EXPR" | "METHOD_CALL_EXPR")
    ) {
        return Ok(None);
    }
    let owner = site["site_owner_occurrence_key"].as_str().ok_or(
        OriginalCallResolutionError::InvalidEvidence {
            field: "site_owner_occurrence_key",
        },
    )?;
    let source_span = context.span(owner)?;
    if source_span["span_configuration_active_flag"] != true {
        return Ok(None);
    }
    let source: HldTargetIdentityValue = NameSymbolTargetKey::Definition(
        decode_occurrence_identity_value(&context.occurrence(source_span)?)?,
    )
    .into();
    if !endpoints.contains(&source) {
        return Ok(None);
    }
    let text = site["site_syntax_source_text"].as_str().ok_or(
        OriginalCallResolutionError::InvalidEvidence {
            field: "site_syntax_source_text",
        },
    )?;
    let mut evidence = site.clone();
    evidence["span_source_code_text"] = json!(text);
    evidence["span_content_digest_full"] =
        json!(blake3::hash(text.as_bytes()).to_hex().to_string());
    let occurrence = decode_occurrence_identity_value(&context.occurrence(&evidence)?)?;
    // A source slice must actually be contained in its graph-owned item.
    let start = site["span_byte_start_offset"].as_u64().ok_or(
        OriginalCallResolutionError::InvalidEvidence {
            field: "span_byte_start_offset",
        },
    )?;
    let owner_start = source_span["span_byte_start_offset"].as_u64().ok_or(
        OriginalCallResolutionError::InvalidEvidence {
            field: "span_byte_start_offset",
        },
    )?;
    let offset = start
        .checked_sub(owner_start)
        .and_then(|n| usize::try_from(n).ok());
    // A site and its owner are in the same text: both original, or both in one expansion.
    if site["span_expansion_provenance_chain"] != source_span["span_expansion_provenance_chain"] {
        return Err(OriginalCallResolutionError::InvalidEvidence {
            field: "span_expansion_provenance_chain",
        });
    }
    if site["file_source_absolute_path"] != source_span["file_source_absolute_path"]
        || site["unit_compilation_identity_key"] != source_span["unit_compilation_identity_key"]
        || offset.and_then(|offset| {
            source_span["span_source_code_text"]
                .as_str()?
                .get(offset..offset.checked_add(text.len())?)
        }) != Some(text)
    {
        return Err(OriginalCallResolutionError::InvalidEvidence {
            field: "site_syntax_source_text",
        });
    }
    let headers = members.impl_header_value_rows.as_slice();
    let syntax = &site["call_syntax_evidence"];
    let receiver_evidence = &syntax["method_receiver_evidence"];
    let cast = (receiver_evidence["receiver_shape_kind"] == "cast_expression")
        .then(|| {
            resolve_cast_receiver_target(
                site,
                &evidence,
                receiver_evidence,
                context,
                members,
                endpoints,
            )
        })
        .transpose()?;
    let bound = if receiver_evidence.is_null() {
        None
    } else {
        Some(resolve_bound_method_target(
            owner,
            source_span,
            receiver_evidence,
            context,
            lookup,
            endpoints,
        )?)
    };
    let callee = if receiver_evidence.is_null() {
        resolve_callee_path_target(&evidence, syntax, context, endpoints, members)?
    } else {
        CalleePathLookupValue::default()
    };
    // The second value says what stopped the lookup of a row without a target.
    let (resolution, stopped) = if let Some((found, stopped)) = &callee.callee_path_row_decision {
        (found.clone(), *stopped)
    } else if let Some(cast) = &cast {
        cast.cast_receiver_row_decision.clone()
    } else if let Some((found, _)) = &bound {
        let stopped = match found {
            HldEdgeResolutionValue::Unresolved { reason } => match reason {
                HldUnresolvedReasonKind::AmbiguousTarget => "bound_method_ambiguous",
                HldUnresolvedReasonKind::ExternalSourceMissing => "bound_trait_source_missing",
                HldUnresolvedReasonKind::MissingTarget => "bound_method_not_found",
                _ => "receiver_type_not_determined",
            },
            _ => "",
        };
        (found.clone(), stopped)
    } else if syntax["call_supported_path_flag"] != true
        || syntax["call_lexical_shadowing_flag"] != false
    {
        // A callee shape the lookup does not handle. When its prefix was looked up and has
        // a stored reason for having no target, that reason is the row's reason.
        let stored = (syntax["call_lexical_shadowing_flag"] == false
            && !syntax["type_source_evidence"].is_null())
        .then(|| context.read_type_path_records(&evidence, &syntax["type_source_evidence"]))
        .transpose()?
        .and_then(|paths| match paths {
            [path] if path["path_resolution_state_kind"] == "unresolved" => {
                match path["path_unresolved_reason_kind"].as_str() {
                    Some("cfg_inactive") => Some(HldUnresolvedReasonKind::CfgInactive),
                    Some("external_source_missing") => {
                        Some(HldUnresolvedReasonKind::ExternalSourceMissing)
                    }
                    _ => None,
                }
            }
            _ => None,
        });
        match stored {
            Some(reason) => (
                unresolved_call_target_value(reason),
                "type_prefix_unresolved",
            ),
            None => (
                unresolved_call_target_value(HldUnresolvedReasonKind::TraitSolverNeeded),
                if syntax["call_lexical_shadowing_flag"] == true {
                    "callee_prefix_lexically_shadowed"
                } else if syntax["member_declared_name_value"].is_null() {
                    "callee_expression_not_path"
                } else {
                    "callee_path_not_requested"
                },
            ),
        }
    } else {
        let paths = context.read_type_path_records(&evidence, &syntax["type_source_evidence"])?;
        let receiver = context.resolve_type(&evidence, &syntax["type_source_evidence"], false)?;
        if paths.len() > 1
            || paths.iter().any(|path| {
                path["path_resolution_state_kind"] == "ambiguous"
                    || path["path_target_symbol_rows"]
                        .as_array()
                        .is_some_and(|targets| targets.len() > 1)
            })
        {
            (
                unresolved_call_target_value(HldUnresolvedReasonKind::AmbiguousTarget),
                "type_prefix_ambiguous",
            )
        } else if receiver["resolution_state_kind"] != "resolved" {
            (
                unresolved_call_target_value(match receiver["unresolved_reason_kind"].as_str() {
                    Some("cfg_inactive") => HldUnresolvedReasonKind::CfgInactive,
                    Some("external_source_missing") => {
                        HldUnresolvedReasonKind::ExternalSourceMissing
                    }
                    Some("trait_solver_needed") => HldUnresolvedReasonKind::TraitSolverNeeded,
                    _ => HldUnresolvedReasonKind::MissingTarget,
                }),
                // The prefix may have a target that is not a type: a trait or a module.
                match paths {
                    [path] => match path["path_target_symbol_rows"]
                        .as_array()
                        .map(Vec::as_slice)
                    {
                        Some([target]) => match target["target_symbol_kind_value"].as_str() {
                            Some("Trait") => "type_prefix_is_trait",
                            Some("Module") => "type_prefix_is_module",
                            _ => "type_prefix_unresolved",
                        },
                        _ => "type_prefix_unresolved",
                    },
                    _ => "type_prefix_unresolved",
                },
            )
        } else {
            (
                resolve_inherent_member_target(
                    headers,
                    &receiver["target_identity"],
                    syntax,
                    endpoints,
                )?,
                "member_lookup_unproven",
            )
        }
    };
    // REQ-RUST-053.0 N3: a member reached through a builtin type that no stored impl and no
    // trait in read source declares is named as the call writes it; its source was not read.
    let builtin_member = (stopped == "member_lookup_unproven"
        && resolution == unresolved_call_target_value(HldUnresolvedReasonKind::MissingTarget))
    .then(|| find_builtin_call_member(&evidence, syntax, context, members))
    .transpose()?
    .flatten();
    let builtin_member = if receiver_evidence["receiver_shape_kind"] == "own_self_value"
        && matches!(resolution, HldEdgeResolutionValue::Unresolved { .. })
    {
        find_self_receiver_member(owner, receiver_evidence, members)
    } else {
        builtin_member
    };
    let (resolution, stopped) = match &builtin_member {
        Some((Ok(named), _)) => (
            HldEdgeResolutionValue::Extracted {
                target: named.external_target_identity_value.clone(),
            },
            "",
        ),
        Some((Err(withheld), _)) => (resolution, *withheld),
        None => (resolution, stopped),
    };
    let target = match &resolution {
        HldEdgeResolutionValue::Extracted { target } => Some(target.clone()),
        _ => None,
    };
    let bound_evidence = bound
        .map(|(_, evidence)| evidence)
        .filter(|evidence| !evidence.is_null());
    // `dispatch_kind` says how the target was chosen; a row without one chose nothing.
    let mut dispatch = json!({
        "dispatch_kind":match (&target, &bound_evidence) {
            (None, _) => "undetermined",
            (Some(_), _) if callee.callee_path_row_decision.is_some() => {
                callee.callee_path_dispatch_kind
            }
            (Some(_), _) if cast.is_some() => "cast_receiver_impl_method",
            (Some(_), Some(_)) => "generic_bound_trait_method",
            (Some(_), _) if matches!(builtin_member, Some((Ok(_), _))) => "external_builtin_member",
            (Some(_), None) => "type_path_associated_function",
        },
        "call_shape_kind":if !receiver_evidence.is_null() {
            "method_call"
        } else if !syntax["type_source_evidence"].is_null() {
            "qualified_path_call"
        } else if !syntax["member_declared_name_value"].is_null() {
            "bare_name_call"
        } else {
            "other_callee_expression"
        },
        "call_source_text":text,
        "callee_source_text":syntax["callee_source_text"],
        "call_syntax_evidence":syntax
    });
    if target.is_none() {
        dispatch["unresolved_detail_kind"] = json!(stopped);
    }
    if let Some(evidence) = bound_evidence {
        dispatch["bound_dispatch_evidence"] = evidence;
    }
    if let Some(cast) = cast {
        dispatch["cast_receiver_lookup_evidence"] = cast.cast_receiver_lookup_evidence;
    }
    if let Some(evidence) = callee.callee_path_lookup_evidence {
        dispatch["callee_path_lookup_evidence"] = evidence;
    }
    if let Some(proof) = callee.external_target_proof_record {
        dispatch["external_target_proof_record"] = proof;
    }
    if let Some((named, lookup)) = builtin_member {
        dispatch["type_relative_member_evidence"] = lookup;
        if let Ok(named) = named {
            dispatch["external_target_proof_record"] = named.external_proof_metadata_value;
        }
    }
    if let Some(evidence) = callee.trait_method_lookup_evidence {
        dispatch["trait_method_lookup_evidence"] = evidence;
    }
    let origin = origins.build_site_origin_value(&evidence, context)?;
    if !origin.is_null() {
        dispatch["expansion_source_origin"] = origin;
    }
    Ok(Some(HldEdgeInputRecord {
        run_generation_digest_value: generation,
        edge_source_reference: source.clone(),
        edge_relation_kind_value: HldRelationKindValue::Calls,
        edge_resolution_value: resolution,
        edge_evidence_row_key: HldEvidenceRowKey {
            evidence_site_occurrence_key: occurrence,
            evidence_source_occurrence_key: source,
            evidence_relation_kind_value: HldRelationKindValue::Calls,
            evidence_target_reference_value: target,
        },
        edge_dispatch_metadata_text: Some(dispatch.to_string()),
    }))
}

/// REQ-RUST-053.0 N3: the member lookup of a call `T::name(..)` whose prefix is a stored
/// type, and the external name it gives when `T` is a builtin type. `None` when the lookup
/// gives no name and no reason to withhold one.
fn find_builtin_call_member(
    span: &Value,
    syntax: &Value,
    context: &ImplHeaderSourceContext<'_>,
    members: &ImplMemberLookupIndex,
) -> Result<Option<(BuiltinMemberNameOutcome, Value)>, OriginalCallResolutionError> {
    let receiver = context.resolve_type(span, &syntax["type_source_evidence"], false)?;
    let (Some(prefix), Some(name)) = (
        syntax["type_source_evidence"]["type_source_text"].as_str(),
        syntax["member_declared_name_value"].as_str(),
    ) else {
        return Ok(None);
    };
    Ok(members
        .find_type_member_target(prefix, &receiver, name, "FN")
        .and_then(|found| {
            members
                .find_builtin_member_name(&found.member_lookup_evidence, name)
                .map(|named| (named, found.member_lookup_evidence))
        }))
}

/// REQ-RUST-053.0 N6: the external name of `self.method(..)` when `self` is taken by
/// value in a member of an impl whose stored self type is a builtin: the receiver's type
/// is that builtin. `None` when the impl is not stored, is inside a function body, or
/// its self type gives no name and no reason to withhold one.
fn find_self_receiver_member(
    owner: &str,
    receiver: &Value,
    members: &ImplMemberLookupIndex,
) -> Option<(BuiltinMemberNameOutcome, Value)> {
    let method = receiver["method_declared_name_value"].as_str()?;
    let header = members.impl_header_value_rows.iter().find(|header| {
        header["impl_member_occurrence_rows"]
            .as_array()
            .is_some_and(|rows| {
                rows.iter()
                    .any(|member| member["member_source_occurrence_key"] == owner)
            })
    })?;
    if header["impl_function_body_ancestry_flag"] != false {
        return None;
    }
    let self_type = &header["impl_self_type_resolution"];
    let text = self_type["type_source_evidence"]["type_source_text"].as_str()?;
    let found = members.find_type_member_target(text, self_type, method, "FN")?;
    let named = members.find_builtin_member_name(&found.member_lookup_evidence, method)?;
    let mut evidence = found.member_lookup_evidence;
    evidence["receiver_type_proof_kind"] = json!("own_self_value_of_impl");
    evidence["enclosing_impl_header_occurrence"] = header["impl_semantic_occurrence_key"].clone();
    // The name holds when the method is found at the receiver's own type. A method found
    // only after an automatic reference would be declared for `&T`; `core` was not read.
    evidence["method_probe_step_assumed"] = json!("receiver_type_itself");
    evidence["method_probe_unread_source"] = json!(["core"]);
    Some((named, evidence))
}

/// What the stored rows say about `(x as T).method(..)`: the row's resolution, what
/// stopped the lookup (empty with a target), and the record of what was looked at.
#[derive(Debug)]
struct CastReceiverLookupValue {
    cast_receiver_row_decision: (HldEdgeResolutionValue, &'static str),
    cast_receiver_lookup_evidence: Value,
}

/// REQ-RUST-052.0 G13: the target of a method call whose receiver is a cast to a
/// stored type. The receiver's type is the cast's type, so no inference is needed.
fn resolve_cast_receiver_target(
    site: &Value,
    span: &Value,
    receiver: &Value,
    context: &ImplHeaderSourceContext<'_>,
    members: &ImplMemberLookupIndex,
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
) -> Result<CastReceiverLookupValue, OriginalCallResolutionError> {
    let cast_type = context.resolve_type(span, &receiver["receiver_cast_type_evidence"], false)?;
    let (member, stopped, evidence) =
        find_cast_receiver_member(site, receiver, &cast_type, members);
    let stop = |reason, stopped, evidence| CastReceiverLookupValue {
        cast_receiver_row_decision: (unresolved_call_target_value(reason), stopped),
        cast_receiver_lookup_evidence: evidence,
    };
    let Some(member) = member else {
        let reason = match cast_type["unresolved_reason_kind"].as_str() {
            Some("cfg_inactive") => HldUnresolvedReasonKind::CfgInactive,
            Some("external_source_missing") => HldUnresolvedReasonKind::ExternalSourceMissing,
            _ => HldUnresolvedReasonKind::TraitSolverNeeded,
        };
        return Ok(stop(reason, stopped, evidence));
    };
    let target = HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(
        decode_occurrence_identity_value(&member["member_semantic_occurrence_key"])?,
    ));
    Ok(if endpoints.contains(&target) {
        CastReceiverLookupValue {
            cast_receiver_row_decision: (HldEdgeResolutionValue::Extracted { target }, ""),
            cast_receiver_lookup_evidence: evidence,
        }
    } else {
        stop(
            HldUnresolvedReasonKind::MissingTarget,
            "cast_member_not_node",
            evidence,
        )
    })
}

/// The impl member `(x as T).method(..)` runs, when the stored impl headers prove
/// one; otherwise what stopped the lookup. The record of what was looked at is
/// returned either way. `cast_type` is the stored resolution of `T`.
///
/// The member is the one stored impl member with that name among the impls of the
/// type (and the impls whose type is not stored), when it takes `self` or `&self`
/// in a trait impl, and that trait is the trait of the impl the call is written in.
/// The last condition proves the trait is in scope: a trait is in scope inside its
/// own impls. The call must be written directly in the member's body.
fn find_cast_receiver_member<'a>(
    site: &Value,
    receiver: &Value,
    cast_type: &Value,
    members: &'a ImplMemberLookupIndex,
) -> (Option<&'a Value>, &'static str, Value) {
    let source = &receiver["receiver_cast_type_evidence"];
    let lookup = source["type_source_text"]
        .as_str()
        .zip(receiver["method_declared_name_value"].as_str())
        .and_then(|(text, method)| members.find_type_member_target(text, cast_type, method, "FN"));
    let Some(lookup) = lookup else {
        return (
            None,
            "cast_type_not_stored",
            json!({"self_type_source_text":source["type_source_text"],
                "member_lookup_outcome_kind":"cast_type_not_stored",
                "cast_type_unresolved_reason":cast_type["unresolved_reason_kind"]}),
        );
    };
    let headers = members.impl_header_value_rows.as_slice();
    let holds = |header: &'a Value, field: &str, key: &Value| {
        header["impl_member_occurrence_rows"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|member| member[field] == *key)
    };
    let trait_of = |header: &'a Value| {
        let stored = &header["impl_trait_type_resolution"];
        (stored["resolution_state_kind"] == "resolved").then_some(&stored["target_identity"])
    };
    let enclosing = headers.iter().find(|header| {
        holds(
            header,
            "member_source_occurrence_key",
            &site["site_owner_occurrence_key"],
        )
        .is_some()
    });
    let chosen = lookup.member_target_occurrence.and_then(|key| {
        headers.iter().find_map(|header| {
            holds(header, "member_semantic_occurrence_key", key).map(|member| (header, member))
        })
    });
    let mut evidence = lookup.member_lookup_evidence;
    evidence["enclosing_impl_header_occurrence"] = enclosing
        .map(|header| header["impl_semantic_occurrence_key"].clone())
        .unwrap_or(Value::Null);
    // Inline, so a reader need not open the impl header table to judge the row.
    for (field, source) in [
        (
            "enclosing_impl_trait_identity",
            "impl_trait_type_resolution",
        ),
        (
            "enclosing_impl_self_type_identity",
            "impl_self_type_resolution",
        ),
    ] {
        evidence[field] = enclosing
            .map(|header| header[source]["target_identity"].clone())
            .unwrap_or(Value::Null);
    }
    for (field, source) in [
        (
            "enclosing_impl_trait_source_text",
            "impl_trait_type_resolution",
        ),
        (
            "enclosing_impl_self_type_source_text",
            "impl_self_type_resolution",
        ),
    ] {
        evidence[field] = enclosing
            .map(|header| header[source]["type_source_evidence"]["type_source_text"].clone())
            .unwrap_or(Value::Null);
    }
    evidence["call_member_body_flag"] = receiver["receiver_member_body_flag"].clone();
    evidence["trait_scope_proof_kind"] = Value::Null;
    let Some((header, member)) = chosen else {
        return (None, "cast_member_unproven", evidence);
    };
    evidence["selected_member_occurrence_key"] = member["member_semantic_occurrence_key"].clone();
    evidence["member_self_parameter_kind"] = member["member_self_parameter_kind"].clone();
    if !matches!(
        member["member_self_parameter_kind"].as_str(),
        Some("value" | "reference")
    ) {
        return (None, "cast_member_not_method", evidence);
    }
    let in_scope = receiver["receiver_member_body_flag"] == true
        && enclosing.is_some_and(|enclosing| {
            enclosing["impl_function_body_ancestry_flag"] == false
                && trait_of(enclosing).is_some_and(|named| Some(named) == trait_of(header))
        });
    if !in_scope {
        return (None, "cast_trait_scope_unproven", evidence);
    }
    evidence["trait_scope_proof_kind"] = json!("enclosing_impl_trait");
    (Some(member), "", evidence)
}

/// What the stored name lookup of a call's whole callee path says.
#[derive(Debug, Default)]
struct CalleePathLookupValue {
    /// The row's resolution and what stopped the lookup (empty with a target),
    /// when the callee path decides the row. `None` leaves the row to the
    /// type-prefix lookup.
    callee_path_row_decision: Option<(HldEdgeResolutionValue, &'static str)>,
    /// What was stored for the callee path; kept on the row either way.
    callee_path_lookup_evidence: Option<Value>,
    /// How a target was chosen, when the callee path gave one.
    callee_path_dispatch_kind: &'static str,
    /// For a trait's method named through the trait: the stored impl members
    /// that could run.
    trait_method_lookup_evidence: Option<Value>,
    /// REQ-RUST-053.0 N4: how the site reached the external name the row targets.
    external_target_proof_record: Option<Value>,
}

/// The stored impls of `trait_identity` that declare a function `name`: what a
/// call of the trait's method through the trait can run (REQ-RUST-052.0 G10).
fn list_trait_method_candidates(headers: &[Value], trait_identity: &Value, name: &Value) -> Value {
    let rows = headers
        .iter()
        .filter(|header| {
            header["impl_trait_type_resolution"]["resolution_state_kind"] == "resolved"
                && header["impl_trait_type_resolution"]["target_identity"] == *trait_identity
        })
        .flat_map(|header| {
            header["impl_member_occurrence_rows"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|member| {
                    member["member_declared_name_value"] == *name
                        && member["member_syntax_kind_value"] == "FN"
                })
                .map(move |member| {
                    let self_type = &header["impl_self_type_resolution"];
                    json!({
                        "impl_header_occurrence":header["impl_semantic_occurrence_key"],
                        "impl_self_type_source_text":self_type["type_source_evidence"]["type_source_text"],
                        "impl_self_type_stored_flag":self_type["resolution_state_kind"] == "resolved",
                        "impl_generic_parameters_flag":header["impl_generic_parameters_flag"],
                        "member_semantic_occurrence_key":member["member_semantic_occurrence_key"]
                    })
                })
        })
        .collect::<Vec<_>>();
    json!({
        "trait_member_outcome_kind":"trait_method_declaration",
        "trait_member_candidate_rows":rows,
        "receiver_type_stored_flag":false,
        "sources_not_read":["impls of the trait in crates whose source was not read"]
    })
}

/// What `Trait::name(self, ..)` runs when `self` is the calling method's own
/// by-value `self` (REQ-RUST-052.0 G12): the member `name` of the one stored impl
/// of `trait_identity` for the type whose impl holds the calling method
/// `site_owner_occurrence_key`, with that impl's header.
///
/// The calling method's impl must be for a stored type written as a plain path,
/// without generic parameters, not negative and not inside a function body. The
/// candidates are the functions `name` in every stored trait impl that is not
/// shown to be of another trait or for another type (an impl whose trait or self
/// type has no stored target may be one of this trait for this type). The member
/// is chosen only when there is exactly one candidate, its impl is as plain as
/// the calling method's and of the stored trait, and both take `self` by value,
/// so the argument's type is the impl's `Self`.
fn find_self_argument_member<'a>(
    headers: &'a [Value],
    trait_identity: &Value,
    site: &Value,
) -> Option<(&'a Value, &'a Value)> {
    let syntax = &site["call_syntax_evidence"];
    let owner_key = &site["site_owner_occurrence_key"];
    if syntax["call_first_argument_self_flag"] != true || !owner_key.is_string() {
        return None;
    }
    let plain = |header: &Value| {
        let stored = &header["impl_self_type_resolution"];
        header["impl_generic_parameters_flag"] == false
            && header["impl_negative_implementation_flag"] == false
            && header["impl_function_body_ancestry_flag"] == false
            && stored["resolution_state_kind"] == "resolved"
            && stored["type_source_evidence"]["type_simple_path_flag"] == true
    };
    let members = |header: &'a Value| {
        header["impl_member_occurrence_rows"]
            .as_array()
            .into_iter()
            .flatten()
    };
    let by_value = |member: &Value| {
        member["member_syntax_kind_value"] == "FN"
            && member["member_self_parameter_kind"] == "value"
    };
    let owner = headers
        .iter()
        .filter(|header| plain(header))
        .find(|header| {
            members(header).any(|member| {
                member["member_source_occurrence_key"] == *owner_key && by_value(member)
            })
        })?;
    let self_type = &owner["impl_self_type_resolution"]["target_identity"];
    let stored = |resolution: &Value| resolution["resolution_state_kind"] == "resolved";
    let of_trait = |header: &Value| {
        let named = &header["impl_trait_type_resolution"];
        stored(named) && named["target_identity"] == *trait_identity
    };
    let mut found = headers
        .iter()
        .filter(|header| {
            let named = &header["impl_trait_type_resolution"];
            let target = &header["impl_self_type_resolution"];
            !named.is_null()
                && (!stored(named) || of_trait(header))
                && (!stored(target) || target["target_identity"] == *self_type)
        })
        .flat_map(|header| members(header).map(move |member| (header, member)))
        .filter(|(_, member)| {
            member["member_syntax_kind_value"] == "FN"
                && member["member_declared_name_value"]
                    .as_str()
                    .map(|name| name.trim_start_matches("r#"))
                    == syntax["member_declared_name_value"].as_str()
        });
    match (found.next(), found.next()) {
        (Some((header, member)), None) => {
            (plain(header) && of_trait(header) && by_value(member)).then_some((member, owner))
        }
        _ => None,
    }
}

/// Reads the stored name lookup of a call's whole callee path. The call gets a
/// target only when the path names one function that is a graph node.
///
/// A qualified callee is decided here only when its qualifier is stored as a
/// module or a trait. A trait's function named through the trait has the
/// trait's declaration as target, with the stored impl members that could run
/// as evidence (G10). A qualifier that is a type names a member, whose target
/// depends on more than the path; such a row keeps the record and goes to the
/// type-prefix lookup. A struct found in the Value namespace is a tuple
/// struct's constructor and is the target of the call (G11).
/// When the first argument of a trait's function is the calling method's own
/// `self`, the member of the impl for that method's type is the target (G12).
/// One exception: a qualifier stored as a builtin type whose
/// whole path has a stored reason. `str::from_utf8_unchecked` under
/// `use core::str` is in a module that was not read, whatever the Type lookup
/// of `str` says.
fn resolve_callee_path_target(
    site: &Value,
    syntax: &Value,
    context: &ImplHeaderSourceContext<'_>,
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
    members: &ImplMemberLookupIndex,
) -> Result<CalleePathLookupValue, OriginalCallResolutionError> {
    let headers = members.impl_header_value_rows.as_slice();
    let source = &syntax["callee_path_evidence"];
    if source.is_null() || syntax["call_lexical_shadowing_flag"] == true {
        return Ok(CalleePathLookupValue::default());
    }
    let [path] = context.read_type_path_records(site, source)? else {
        return Ok(CalleePathLookupValue::default());
    };
    let qualified = !syntax["type_source_evidence"].is_null();
    let targets = path["path_target_symbol_rows"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let kinds = targets
        .iter()
        .map(|target| target["target_symbol_kind_value"].clone())
        .collect::<Vec<_>>();
    let prefix_symbol = if qualified {
        match context.read_type_path_records(site, &syntax["type_source_evidence"])? {
            [prefix] => match prefix["path_target_symbol_rows"]
                .as_array()
                .map(Vec::as_slice)
            {
                Some([target]) => Some(target),
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    };
    let prefix_kind = prefix_symbol
        .map(|symbol| symbol["target_symbol_kind_value"].clone())
        .unwrap_or(Value::Null);
    let finish = |decision: Option<(HldEdgeResolutionValue, &'static str)>, outcome: &str| {
        CalleePathLookupValue {
            callee_path_row_decision: decision,
            callee_path_dispatch_kind: "",
            trait_method_lookup_evidence: None,
            external_target_proof_record: None,
            callee_path_lookup_evidence: Some(json!({
                "callee_path_state_kind":path["path_resolution_state_kind"],
                "callee_path_unresolved_reason":path["path_unresolved_reason_kind"],
                "callee_target_symbol_kinds":kinds,
                "callee_qualifier_symbol_kind":prefix_kind,
                "callee_path_outcome_kind":outcome
            })),
        }
    };
    let stop = |reason, stopped: &'static str, decides: bool| {
        finish(
            decides.then(|| (unresolved_call_target_value(reason), stopped)),
            stopped,
        )
    };
    let symbol = match (path["path_resolution_state_kind"].as_str(), targets) {
        (Some("resolved"), [symbol]) => symbol,
        (Some("ambiguous"), _) | (_, [_, _, ..]) => {
            return Ok(stop(
                HldUnresolvedReasonKind::AmbiguousTarget,
                "callee_path_ambiguous",
                !qualified,
            ));
        }
        _ => {
            let stored = &path["path_unresolved_reason_kind"];
            // REQ-RUST-053.0 N1, N5: the callee's source was not read; the path the call
            // writes is proved. A qualified path whose last name a local trait declares may
            // name a local impl's member, so it gets no name.
            let named = (stored == "external_source_missing")
                .then(|| decode_external_name_record(&path["path_external_name_record"]))
                .transpose()
                .map_err(|error| OriginalCallResolutionError::InvalidEvidence { field: error.0 })?
                .flatten();
            if let Some(named) = named {
                let local_trait_member = qualified
                    && syntax["member_declared_name_value"]
                        .as_str()
                        .is_none_or(|name| {
                            members
                                .trait_declared_member_names
                                .contains(name.trim_start_matches("r#"))
                        });
                if local_trait_member {
                    return Ok(stop(
                        HldUnresolvedReasonKind::ExternalSourceMissing,
                        "external_member_name_in_local_trait",
                        true,
                    ));
                }
                return Ok(CalleePathLookupValue {
                    callee_path_dispatch_kind: "external_written_path",
                    external_target_proof_record: Some(named.external_proof_metadata_value),
                    ..finish(
                        Some((
                            HldEdgeResolutionValue::Extracted {
                                target: named.external_target_identity_value,
                            },
                            "",
                        )),
                        "external_name_proved",
                    )
                });
            }
            return Ok(stop(
                read_stored_reason_kind(stored),
                "callee_path_unresolved",
                !qualified || (!stored.is_null() && prefix_kind == "Builtin"),
            ));
        }
    };
    let kind = symbol["target_symbol_kind_value"].as_str();
    let through_trait = qualified && prefix_kind == "Trait";
    // (dispatch kind, outcome with a target): how this path names what it calls.
    let (dispatch, outcome) = match kind {
        Some("Function") if through_trait => ("trait_path_method", "single_trait_method"),
        Some("Function") if !qualified || prefix_kind == "Module" => {
            ("callee_path_function", "single_function_node")
        }
        Some("Struct") if !qualified || prefix_kind == "Module" => {
            ("tuple_struct_constructor", "single_struct_node")
        }
        Some("Function" | "Struct") => return Ok(finish(None, "callee_qualifier_not_module")),
        _ => {
            return Ok(stop(
                HldUnresolvedReasonKind::TraitSolverNeeded,
                "callee_is_not_function",
                !qualified,
            ));
        }
    };
    let declared = context.read_single_target_identity(symbol)?;
    let target = declared
        .as_ref()
        .map(decode_symbol_target_value)
        .transpose()?
        .map(HldTargetIdentityValue::from)
        .filter(|target| endpoints.contains(target));
    let Some(target) = target else {
        return Ok(stop(
            HldUnresolvedReasonKind::MissingTarget,
            "callee_target_not_node",
            true,
        ));
    };
    let name = &syntax["member_declared_name_value"];
    let trait_identity = match prefix_symbol.filter(|_| through_trait) {
        Some(prefix) => context.read_single_target_identity(prefix)?,
        None => None,
    };
    // G12: the first argument is the calling method's own `self`.
    let by_self = trait_identity
        .as_ref()
        .and_then(|identity| find_self_argument_member(headers, identity, site))
        .map(|(member, owner)| {
            decode_occurrence_identity_value(&member["member_semantic_occurrence_key"]).map(|key| {
                (
                    HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(key)),
                    (member, owner),
                )
            })
        })
        .transpose()?
        .filter(|(member, _)| endpoints.contains(member));
    let evidence = trait_identity.as_ref().map(|identity| {
        let listed = list_trait_method_candidates(headers, identity, name);
        match &by_self {
            Some((_, (member, owner))) => json!({
                "trait_member_outcome_kind":"single_impl_member_by_self",
                "trait_member_candidate_rows":listed["trait_member_candidate_rows"],
                "selected_member_occurrence_key":member["member_semantic_occurrence_key"],
                "receiver_type_stored_flag":true,
                "receiver_self_type_identity":owner["impl_self_type_resolution"]["target_identity"],
                "receiver_self_type_source_text":owner["impl_self_type_resolution"]["type_source_evidence"]["type_source_text"],
                "receiver_impl_header_occurrence":owner["impl_semantic_occurrence_key"],
                "trait_method_declaration_identity":declared,
                "sources_not_read":listed["sources_not_read"]
            }),
            None => listed,
        }
    });
    let (target, dispatch) = match by_self {
        Some((member, _)) => (member, "trait_path_impl_method"),
        None => (target, dispatch),
    };
    Ok(CalleePathLookupValue {
        callee_path_dispatch_kind: dispatch,
        trait_method_lookup_evidence: evidence,
        external_target_proof_record: None,
        ..finish(
            Some((HldEdgeResolutionValue::Extracted { target }, "")),
            outcome,
        )
    })
}

/// Bound rows by owner item, traits that declare supertraits, methods with a
/// `self` parameter, and the graph's containment pairs: all a bound walk needs.
#[derive(Default)]
pub(super) struct BoundMethodLookupIndex {
    owner_bound_rows: BTreeMap<String, Vec<Value>>,
    trait_owner_keys: BTreeMap<HldTargetIdentityValue, String>,
    self_method_targets: Vec<(String, HldTargetIdentityValue)>,
    pub(super) contained_item_pairs: BTreeSet<(HldTargetIdentityValue, HldTargetIdentityValue)>,
}

/// The traits a type parameter is bound by, with their supertraits.
/// `bound_chain_texts` starts with the parameter; `missing_source_traits`
/// names each bound whose trait has no source in the run.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct BoundTraitWalkValue {
    pub(super) bound_chain_texts: Vec<String>,
    pub(super) missing_source_traits: Vec<String>,
    pub(super) bound_trait_targets: Vec<HldTargetIdentityValue>,
}

impl BoundTraitWalkValue {
    /// The record an edge carries to say which bounds chose its target.
    pub(super) fn build_walk_evidence_value(&self) -> Value {
        json!({"bound_chain":self.bound_chain_texts,"missing_source_traits":self.missing_source_traits})
    }
}

pub(super) fn build_bound_method_index(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    context: &ImplHeaderSourceContext<'_>,
    input: &HldMaterializationInputRecord,
) -> Result<BoundMethodLookupIndex, OriginalCallResolutionError> {
    // Rows whose owner is not a graph-identifiable span cannot take part in a walk.
    let identify = |key: &str| -> Option<HldTargetIdentityValue> {
        let occurrence = context.occurrence(context.span(key).ok()?).ok()?;
        Some(
            NameSymbolTargetKey::Definition(decode_occurrence_identity_value(&occurrence).ok()?)
                .into(),
        )
    };
    let mut index = BoundMethodLookupIndex::default();
    for row in read_impl_stage_values(store, scope, "field-signature-detail-table")? {
        let Some(owner) = row["site_owner_occurrence_key"].as_str().map(str::to_owned) else {
            continue;
        };
        match row["site_syntax_kind_value"].as_str() {
            Some("TYPE_BOUND") => {
                if row["bound_syntax_evidence"]["bound_subject_kind"] == "supertrait"
                    && let Some(identity) = identify(&owner)
                {
                    index.trait_owner_keys.insert(identity, owner.clone());
                }
                index.owner_bound_rows.entry(owner).or_default().push(row);
            }
            Some("SELF_PARAM") => {
                let Ok(span) = context.span(&owner) else {
                    continue;
                };
                if span["span_syntax_kind_value"] == "FN"
                    && let Some(name) = span["span_declared_name_value"].as_str()
                    && let Some(identity) = identify(&owner)
                {
                    index
                        .self_method_targets
                        .push((name.trim_start_matches("r#").to_owned(), identity));
                }
            }
            _ => {}
        }
    }
    index.contained_item_pairs = input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Contains)
        .filter_map(|edge| match &edge.edge_resolution_value {
            HldEdgeResolutionValue::Extracted { target } => {
                Some((edge.edge_source_reference.clone(), target.clone()))
            }
            _ => None,
        })
        .collect();
    Ok(index)
}

/// Walk the bounds that `owner_key` declares on its type parameter
/// `parameter`, then the supertraits of each trait found. `None` when a bound
/// has a form the walk cannot prove. A name that is not a bounded type
/// parameter of the owner gives a walk with no trait and no missing source.
pub(super) fn walk_parameter_bound_traits(
    owner_key: &str,
    parameter: &str,
    context: &ImplHeaderSourceContext<'_>,
    index: &BoundMethodLookupIndex,
) -> Result<Option<BoundTraitWalkValue>, OriginalCallResolutionError> {
    let mut walk = BoundTraitWalkValue {
        bound_chain_texts: vec![parameter.to_owned()],
        ..BoundTraitWalkValue::default()
    };
    let mut visited = BTreeSet::new();
    let mut pending = vec![(owner_key.to_owned(), false)];
    while let Some((key, supertraits)) = pending.pop() {
        let span = context.span(&key)?;
        for row in index.owner_bound_rows.get(&key).into_iter().flatten() {
            let bound = &row["bound_syntax_evidence"];
            let applies = if supertraits {
                bound["bound_subject_kind"] == "supertrait"
            } else {
                matches!(
                    bound["bound_subject_kind"].as_str(),
                    Some("type_parameter" | "where_predicate")
                ) && bound["bound_subject_name_value"] == parameter
            };
            if !applies || bound["bound_form_kind"] == "lifetime" {
                continue;
            }
            let text = row["site_syntax_source_text"].as_str().unwrap_or_default();
            if bound["bound_form_kind"] != "trait" {
                // `?Sized` relaxes a default and adds no member; other forms are unproven.
                if text.starts_with('?') {
                    continue;
                }
                return Ok(None);
            }
            let resolved = context.resolve_type(span, &bound["trait_type_source"], true)?;
            if resolved["resolution_state_kind"] != "resolved" {
                if resolved["unresolved_reason_kind"] == "external_source_missing" {
                    walk.missing_source_traits.push(text.to_owned());
                    continue;
                }
                return Ok(None);
            }
            let found: HldTargetIdentityValue =
                decode_symbol_target_value(&resolved["target_identity"])?.into();
            if !visited.insert(found.clone()) {
                continue;
            }
            walk.bound_chain_texts.push(text.to_owned());
            if let Some(next) = index.trait_owner_keys.get(&found) {
                pending.push((next.clone(), true));
            }
            walk.bound_trait_targets.push(found);
        }
    }
    Ok(Some(walk))
}

/// `parameter.method(..)` where the parameter's declared type is a type
/// parameter of the enclosing function: the candidates are the methods of the
/// parameter's bound traits and their supertraits. Exactly one gives an edge to
/// that trait method; a trait whose source is absent is recorded, not guessed.
fn resolve_bound_method_target(
    owner_key: &str,
    owner_span: &Value,
    receiver: &Value,
    context: &ImplHeaderSourceContext<'_>,
    index: &BoundMethodLookupIndex,
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
) -> Result<(HldEdgeResolutionValue, Value), OriginalCallResolutionError> {
    let unproven = (
        unresolved_call_target_value(HldUnresolvedReasonKind::TraitSolverNeeded),
        Value::Null,
    );
    let (Some(parameter), Some(method)) = (
        receiver["receiver_parameter_type_text"].as_str(),
        receiver["method_declared_name_value"].as_str(),
    ) else {
        return Ok(unproven);
    };
    if receiver["receiver_shape_kind"] != "parameter_identifier"
        || receiver["receiver_generic_owner_kind"] != "function"
        || receiver["receiver_rebinding_flag"] != false
        || owner_span["span_syntax_kind_value"] != "FN"
    {
        return Ok(unproven);
    }
    let Some(walk) = walk_parameter_bound_traits(owner_key, parameter, context, index)? else {
        return Ok(unproven);
    };
    let candidates = walk
        .bound_trait_targets
        .iter()
        .flat_map(|found| {
            index
                .self_method_targets
                .iter()
                .filter(move |(name, target)| {
                    name == method
                        && index
                            .contained_item_pairs
                            .contains(&(found.clone(), target.clone()))
                })
                .map(|(_, target)| target.clone())
        })
        .collect::<BTreeSet<_>>();
    let missing = &walk.missing_source_traits;
    let evidence = walk.build_walk_evidence_value();
    let mut found = candidates.into_iter();
    let resolution = match (found.next(), found.next()) {
        (Some(target), None) if endpoints.contains(&target) => {
            HldEdgeResolutionValue::Extracted { target }
        }
        (Some(_), Some(_)) => {
            unresolved_call_target_value(HldUnresolvedReasonKind::AmbiguousTarget)
        }
        (None, _) if !missing.is_empty() => {
            unresolved_call_target_value(HldUnresolvedReasonKind::ExternalSourceMissing)
        }
        _ => unresolved_call_target_value(HldUnresolvedReasonKind::MissingTarget),
    };
    Ok((resolution, evidence))
}

fn unresolved_call_target_value(reason: HldUnresolvedReasonKind) -> HldEdgeResolutionValue {
    HldEdgeResolutionValue::Unresolved { reason }
}

fn resolve_inherent_member_target(
    headers: &[Value],
    receiver: &Value,
    syntax: &Value,
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
) -> Result<HldEdgeResolutionValue, OriginalCallResolutionError> {
    // Validate the canonical receiver; matching is identity equality, never a name.
    let _ = decode_symbol_target_value(receiver)?;
    let name = syntax["member_declared_name_value"].as_str().ok_or(
        OriginalCallResolutionError::InvalidEvidence {
            field: "member_declared_name_value",
        },
    )?;
    let mut candidates = BTreeSet::new();
    let mut unsupported_inherent = false;
    let mut unsupported_trait = false;
    for header in headers {
        if header["impl_self_type_resolution"]["target_identity"] != *receiver {
            continue;
        }
        for member in header["impl_member_occurrence_rows"].as_array().ok_or(
            OriginalCallResolutionError::InvalidEvidence {
                field: "impl_member_occurrence_rows",
            },
        )? {
            if member["member_declared_name_value"]
                .as_str()
                .map(|name| name.trim_start_matches("r#"))
                != Some(name)
                || member["member_syntax_kind_value"] != "FN"
            {
                continue;
            }
            if !header["impl_trait_type_resolution"].is_null() {
                unsupported_trait = true;
                continue;
            }
            if header["impl_generic_parameters_flag"] != false
                || header["impl_negative_implementation_flag"] != false
                || member["member_generic_parameters_flag"] != false
                || member["member_self_parameter_flag"] != false
            {
                unsupported_inherent = true;
                continue;
            }
            let key = decode_occurrence_identity_value(&member["member_semantic_occurrence_key"])?;
            candidates.insert(HldTargetIdentityValue::from(
                NameSymbolTargetKey::Definition(key),
            ));
        }
    }
    Ok(if candidates.len() > 1 {
        unresolved_call_target_value(HldUnresolvedReasonKind::AmbiguousTarget)
    } else if unsupported_inherent {
        unresolved_call_target_value(HldUnresolvedReasonKind::TraitSolverNeeded)
    } else if let Some(target) = candidates.into_iter().next() {
        if endpoints.contains(&target) {
            HldEdgeResolutionValue::Extracted { target }
        } else {
            unresolved_call_target_value(HldUnresolvedReasonKind::MissingTarget)
        }
    } else if unsupported_trait {
        unresolved_call_target_value(HldUnresolvedReasonKind::TraitSolverNeeded)
    } else {
        unresolved_call_target_value(HldUnresolvedReasonKind::MissingTarget)
    })
}

/// The type a method call's receiver is cast to (`(x as T).m()`, through any
/// parentheses), when the receiver is a cast.
fn read_cast_receiver_type(call: &ast::MethodCallExpr) -> Option<ast::Type> {
    let mut receiver = call.receiver()?;
    while let ast::Expr::ParenExpr(inner) = &receiver {
        receiver = inner.expr()?;
    }
    match receiver {
        ast::Expr::CastExpr(cast) => cast.ty(),
        _ => None,
    }
}

/// The path of a type that is one path without generic arguments or a type anchor.
fn read_simple_type_path(ty: &ast::Type) -> Option<ast::Path> {
    ast::PathType::cast(ty.syntax().clone())
        .and_then(|ty| ty.path())
        .filter(|path| {
            !path.syntax().descendants().any(|child| {
                matches!(
                    child.kind(),
                    SyntaxKind::GENERIC_ARG_LIST | SyntaxKind::TYPE_ANCHOR
                )
            })
        })
}

/// The whole path of the type a method call's receiver is cast to
/// (REQ-RUST-052.0 G13) is submitted to the namespace engine as a Type request.
pub(super) fn check_cast_receiver_type(path: &ast::Path) -> bool {
    path.syntax()
        .ancestors()
        .find_map(ast::MethodCallExpr::cast)
        .and_then(|call| read_cast_receiver_type(&call))
        .and_then(|ty| read_simple_type_path(&ty))
        .is_some_and(|whole| whole.syntax() == path.syntax())
}

/// The immediate qualifier of a path-expression callee is submitted to the
/// existing namespace engine as a Type request, in the actual lexical scope.
pub(super) fn check_body_call_prefix(path: &ast::Path) -> bool {
    path.syntax()
        .parent()
        .and_then(ast::Path::cast)
        .is_some_and(|outer| {
            outer.qualifier().as_ref() == Some(path)
                && outer
                    .syntax()
                    .parent()
                    .and_then(ast::PathExpr::cast)
                    .is_some_and(|expression| {
                        expression
                            .syntax()
                            .parent()
                            .and_then(ast::CallExpr::cast)
                            .is_some_and(|call| {
                                call.expr()
                                    .is_some_and(|callee| callee.syntax() == expression.syntax())
                            })
                    })
        })
}

/// Whether `path` is the whole callee of a call expression and may name a
/// function: it has no `<T as Trait>` anchor and its single name is not bound
/// by a parameter or pattern in scope. It is submitted to the namespace engine
/// as a Value request, in the actual lexical scope.
pub(super) fn check_body_call_callee(path: &ast::Path) -> bool {
    path.syntax()
        .parent()
        .and_then(ast::PathExpr::cast)
        .is_some_and(|expression| {
            expression
                .syntax()
                .parent()
                .and_then(ast::CallExpr::cast)
                .and_then(|call| call.expr())
                .is_some_and(|callee| callee.syntax() == expression.syntax())
        })
        && !path
            .syntax()
            .descendants()
            .any(|child| child.kind() == SyntaxKind::TYPE_ANCHOR)
        && !super::body_value_read_records::check_local_value_binding(path)
}

/// The argument is the path `self`, and the function it is written in declares
/// `self` by value with no type and is not inside a block: the argument's type
/// is the `Self` of the impl that holds that function, which owns the call site.
fn check_own_self_argument(argument: &ast::Expr) -> bool {
    ast::PathExpr::cast(argument.syntax().clone())
        .and_then(|expr| expr.path())
        .and_then(|path| path.segment())
        .is_some_and(|segment| segment.self_token().is_some())
        && argument
            .syntax()
            .ancestors()
            .find_map(ast::Fn::cast)
            .filter(|function| {
                !function
                    .syntax()
                    .ancestors()
                    .skip(1)
                    .any(|node| node.kind() == SyntaxKind::BLOCK_EXPR)
            })
            .and_then(|function| function.param_list())
            .and_then(|parameters| parameters.self_param())
            .is_some_and(|parameter| parameter.amp_token().is_none() && parameter.ty().is_none())
}

pub(super) fn extract_original_call_syntax(
    input: &SyntaxFileInputData,
    node: &SyntaxNode,
    removed: &[usize],
) -> Value {
    let path = ast::CallExpr::cast(node.clone())
        .and_then(|call| call.expr())
        .and_then(|expr| ast::PathExpr::cast(expr.syntax().clone()))
        .and_then(|expr| expr.path());
    let prefix = path.as_ref().and_then(ast::Path::qualifier);
    let supported = path.as_ref().is_some_and(|path| {
        !path.syntax().descendants().any(|child| {
            matches!(
                child.kind(),
                SyntaxKind::GENERIC_ARG_LIST | SyntaxKind::TYPE_ANCHOR
            )
        })
    }) && prefix.is_some();
    let type_source = prefix.as_ref().map(|prefix| {
        let range=prefix.syntax().text_range();
        let start=restore_source_byte_offset(range.start(),removed);
        let end=restore_source_byte_offset(range.end(),removed);
        json!({"span_byte_start_offset":start,"span_byte_end_offset":end,"type_source_text":std::str::from_utf8(&input.file_source_content_bytes[start..end]).ok(),"type_simple_path_flag":supported})
    });
    let member = path
        .as_ref()
        .and_then(ast::Path::segment)
        .and_then(|segment| segment.name_ref())
        .map(|name| name.text().trim_start_matches("r#").to_owned());
    let shadowed = prefix
        .as_ref()
        .is_some_and(|prefix| check_call_lexical_shadowing(prefix, node));
    let receiver = ast::MethodCallExpr::cast(node.clone())
        .map(|call| extract_method_receiver_syntax(&call, input, removed));
    let own_self = ast::CallExpr::cast(node.clone())
        .and_then(|call| ast::HasArgList::arg_list(&call))
        .and_then(|list| list.args().next())
        .is_some_and(|argument| check_own_self_argument(&argument));
    // What is called, as written: the callee expression, or the method's name.
    let callee = ast::CallExpr::cast(node.clone())
        .and_then(|call| call.expr())
        .map(|expr| expr.syntax().text_range())
        .or_else(|| {
            ast::MethodCallExpr::cast(node.clone())
                .and_then(|call| call.name_ref())
                .map(|name| name.syntax().text_range())
        })
        .and_then(|range| {
            let start = restore_source_byte_offset(range.start(), removed);
            let end = restore_source_byte_offset(range.end(), removed);
            std::str::from_utf8(input.file_source_content_bytes.get(start..end)?).ok()
        });
    let callee_path = path
        .as_ref()
        .filter(|path| check_body_call_callee(path))
        .and_then(|path| {
            let range = path.syntax().text_range();
            let start = restore_source_byte_offset(range.start(), removed);
            let end = restore_source_byte_offset(range.end(), removed);
            let text = std::str::from_utf8(input.file_source_content_bytes.get(start..end)?).ok()?;
            Some(json!({"span_byte_start_offset":start,"span_byte_end_offset":end,"type_source_text":text}))
        });
    json!({"callee_source_text":callee,"callee_path_evidence":callee_path,"call_supported_path_flag":supported,"member_declared_name_value":member,"type_source_evidence":type_source,"call_lexical_shadowing_flag":shadowed,"call_first_argument_self_flag":own_self,"method_receiver_evidence":receiver})
}

/// Syntax-only evidence for `receiver.method(..)`. Nothing is resolved here:
/// the record says whether the receiver is a plain parameter name, what type
/// that parameter declares, and whether that type is a type parameter.
fn extract_method_receiver_syntax(
    call: &ast::MethodCallExpr,
    input: &SyntaxFileInputData,
    removed: &[usize],
) -> Value {
    let cast_type = read_cast_receiver_type(call).map(|ty| {
        let range = ty.syntax().text_range();
        let start = restore_source_byte_offset(range.start(), removed);
        let end = restore_source_byte_offset(range.end(), removed);
        json!({"span_byte_start_offset":start, "span_byte_end_offset":end,
            "type_source_text":input.file_source_content_bytes.get(start..end)
                .and_then(|bytes| std::str::from_utf8(bytes).ok()),
            "type_simple_path_flag":read_simple_type_path(&ty).is_some()})
    });
    // The call is written directly in the body of an impl member or a free function:
    // its nearest item is a function that is not inside any block.
    let member_body = call
        .syntax()
        .ancestors()
        .find_map(ast::Item::cast)
        .is_some_and(|item| {
            matches!(item, ast::Item::Fn(_))
                && !item
                    .syntax()
                    .ancestors()
                    .skip(1)
                    .any(|node| node.kind() == SyntaxKind::BLOCK_EXPR)
        });
    let function = call.syntax().ancestors().find_map(ast::Fn::cast);
    let receiver = call
        .receiver()
        .and_then(|expr| read_plain_identifier_text(&expr));
    let parameter = receiver
        .as_deref()
        .zip(function.as_ref())
        .and_then(|(name, function)| find_named_function_parameter(function, name));
    let declared = parameter.as_ref().and_then(read_simple_type_name);
    let owner = declared
        .as_deref()
        .zip(function.as_ref())
        .and_then(|(name, function)| find_type_parameter_owner(function, name));
    let rebound = receiver
        .as_deref()
        .zip(function.as_ref())
        .is_some_and(|(name, function)| check_receiver_name_rebound(function, call, name));
    json!({
        "receiver_shape_kind":if parameter.is_some() {
            "parameter_identifier"
        } else if call.receiver().is_some_and(|expr| check_own_self_argument(&expr)) {
            "own_self_value"
        } else if cast_type.is_some() {
            "cast_expression"
        } else {
            "other"
        },
        "receiver_cast_type_evidence":cast_type,
        "receiver_member_body_flag":member_body,
        "receiver_declared_name_value":receiver,
        "method_declared_name_value":call.name_ref().map(|name| name.text().trim_start_matches("r#").to_owned()),
        "method_generic_arguments_flag":call.generic_arg_list().is_some(),
        "receiver_parameter_type_text":declared,
        "receiver_generic_owner_kind":owner,
        "receiver_rebinding_flag":rebound,
    })
}

/// Syntax-only evidence for one bound: what it bounds (a type parameter, the
/// trait itself as a supertrait, or a `where` subject) and the trait path.
/// Lifetime bounds and modified bounds (`?Sized`, `for<'a> ..`) carry no path.
pub(super) fn extract_type_bound_syntax(
    input: &SyntaxFileInputData,
    bound: &ast::TypeBound,
    removed: &[usize],
) -> Value {
    let bounded = bound
        .syntax()
        .parent()
        .filter(|list| list.kind() == SyntaxKind::TYPE_BOUND_LIST)
        .and_then(|list| list.parent());
    let (subject, name) = match bounded {
        Some(owner) if owner.kind() == SyntaxKind::TYPE_PARAM => (
            "type_parameter",
            owner
                .children()
                .find_map(ast::Name::cast)
                .map(|name| name.text().trim_start_matches("r#").to_owned()),
        ),
        Some(owner) if owner.kind() == SyntaxKind::TRAIT => ("supertrait", None),
        Some(owner) if owner.kind() == SyntaxKind::WHERE_PRED => (
            "where_predicate",
            match ast::WherePred::cast(owner).and_then(|predicate| predicate.ty()) {
                Some(ast::Type::PathType(subject)) => subject
                    .path()
                    .and_then(|path| read_single_segment_name(&path)),
                _ => None,
            },
        ),
        _ => ("other", None),
    };
    let path = match bound.ty() {
        Some(ast::Type::PathType(found)) => found.path(),
        _ => None,
    }
    .filter(|path| path.syntax().text_range() == bound.syntax().text_range());
    let form = if bound.lifetime().is_some() {
        "lifetime"
    } else if path.is_some() {
        "trait"
    } else {
        "other"
    };
    let trait_source = path.map(|path| {
        let range = path.syntax().text_range();
        let start = restore_source_byte_offset(range.start(), removed);
        let end = restore_source_byte_offset(range.end(), removed);
        let simple = !path.syntax().descendants().any(|child| {
            matches!(
                child.kind(),
                SyntaxKind::GENERIC_ARG_LIST
                    | SyntaxKind::TYPE_ANCHOR
                    | SyntaxKind::PARENTHESIZED_ARG_LIST
            )
        });
        json!({"span_byte_start_offset":start,"span_byte_end_offset":end,"type_source_text":std::str::from_utf8(&input.file_source_content_bytes[start..end]).ok(),"type_simple_path_flag":simple})
    });
    json!({"bound_subject_kind":subject,"bound_subject_name_value":name,"bound_form_kind":form,"trait_type_source":trait_source})
}

fn read_plain_identifier_text(expr: &ast::Expr) -> Option<String> {
    let ast::Expr::PathExpr(expr) = expr else {
        return None;
    };
    read_single_segment_name(&expr.path()?)
}

/// One unqualified segment without generic arguments; never `self` or `Self`.
fn read_single_segment_name(path: &ast::Path) -> Option<String> {
    let segment = path.segment()?;
    if path.qualifier().is_some()
        || segment.generic_arg_list().is_some()
        || segment.syntax().children().count() != 1
    {
        return None;
    }
    let name = segment.name_ref()?;
    name.syntax()
        .first_token()
        .is_some_and(|token| token.kind() == SyntaxKind::IDENT)
        .then(|| name.text().trim_start_matches("r#").to_owned())
}

fn find_named_function_parameter(function: &ast::Fn, name: &str) -> Option<ast::Param> {
    function.param_list()?.params().find(|parameter| {
        matches!(parameter.pat(), Some(ast::Pat::IdentPat(pattern))
            if pattern.pat().is_none()
                && pattern.name().is_some_and(|found| found.text().trim_start_matches("r#") == name))
    })
}

fn read_simple_type_name(parameter: &ast::Param) -> Option<String> {
    let ast::Type::PathType(declared) = parameter.ty()? else {
        return None;
    };
    read_single_segment_name(&declared.path()?)
}

/// A type parameter of the function itself, or of the impl or trait the
/// function is a direct member of. A nested function sees neither.
fn find_type_parameter_owner(function: &ast::Fn, name: &str) -> Option<&'static str> {
    let declares = |list: Option<ast::GenericParamList>| {
        list.is_some_and(|list| {
            list.type_or_const_params().any(|parameter| {
                matches!(parameter, ast::TypeOrConstParam::Type(found)
                    if found.name().is_some_and(|found| found.text().trim_start_matches("r#") == name))
            })
        })
    };
    if declares(function.generic_param_list()) {
        return Some("function");
    }
    let container = function
        .syntax()
        .parent()
        .filter(|list| list.kind() == SyntaxKind::ASSOC_ITEM_LIST)?
        .parent()?;
    let outer = ast::Impl::cast(container.clone())
        .map(|item| item.generic_param_list())
        .or_else(|| ast::Trait::cast(container).map(|item| item.generic_param_list()))?;
    declares(outer).then_some("enclosing_item")
}

/// Conservative: any binding or local item of that name anywhere in the body,
/// or an earlier macro call that is handed the name, may rebind the receiver.
fn check_receiver_name_rebound(function: &ast::Fn, call: &ast::MethodCallExpr, name: &str) -> bool {
    let Some(body) = function.body() else {
        return false;
    };
    let call_start = call.syntax().text_range().start();
    body.syntax().descendants().any(|node| {
        if node.kind() == SyntaxKind::MACRO_CALL {
            return node.text_range().start() < call_start
                && node
                    .descendants_with_tokens()
                    .filter_map(|element| element.into_token())
                    .any(|token| token.kind() == SyntaxKind::IDENT && token.text() == name);
        }
        (node.kind() == SyntaxKind::IDENT_PAT || crate::check_syntax_item_node(&node))
            && node
                .children()
                .filter_map(ast::Name::cast)
                .any(|found| found.text().trim_start_matches("r#") == name)
    })
}

fn check_call_lexical_shadowing(prefix: &ast::Path, call: &SyntaxNode) -> bool {
    let Some(head) = prefix
        .syntax()
        .descendants()
        .find_map(ast::PathSegment::cast)
        .and_then(|segment| segment.name_ref())
        .map(|name| name.text().trim_start_matches("r#").to_owned())
    else {
        return false;
    };
    // Explicit crate/self/super/absolute paths cannot be shadowed by local names.
    let source = prefix.syntax().text().to_string();
    if source.starts_with("::")
        || matches!(source.split("::").next(), Some("crate" | "self" | "super"))
    {
        return false;
    }
    if call
        .ancestors()
        .filter(|node| ast::Fn::can_cast(node.kind()) || ast::ClosureExpr::can_cast(node.kind()))
        .flat_map(|node| {
            node.children()
                .filter(|child| child.kind() == SyntaxKind::PARAM_LIST)
                .collect::<Vec<_>>()
        })
        .flat_map(|parameters| {
            parameters
                .descendants()
                .filter(|child| child.kind() == SyntaxKind::IDENT_PAT)
                .collect::<Vec<_>>()
        })
        .any(|pattern| {
            pattern
                .children()
                .filter_map(ast::Name::cast)
                .any(|name| name.text().trim_start_matches("r#") == head)
        })
    {
        return true;
    }
    call.ancestors()
        .filter(|node| ast::BlockExpr::can_cast(node.kind()))
        .any(|block| {
            block
                .descendants()
                .filter(|node| {
                    node.ancestors()
                        .skip(1)
                        .find(|ancestor| ast::BlockExpr::can_cast(ancestor.kind()))
                        .as_ref()
                        == Some(&block)
                })
                .any(|node| {
                    // Local items/imports are hoisted; value patterns are conservatively
                    // retained even when their precise activation point is unsupported.
                    (crate::check_syntax_item_node(&node) || node.kind() == SyntaxKind::IDENT_PAT)
                        && node
                            .children()
                            .filter_map(ast::Name::cast)
                            .any(|name| name.text().trim_start_matches("r#") == head)
                        || ast::Use::can_cast(node.kind())
                })
        })
}
