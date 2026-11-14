use crate::impl_header_source_records::{
    ImplHeaderSourceContext, ImplHeaderSourceError, read_impl_source_context,
    read_impl_stage_values,
};
use crate::syntax_namespace_record_codec::{
    decode_occurrence_identity_value, decode_symbol_target_value,
};
use crate::{SyntaxFileInputData, restore_source_byte_offset};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use ra_ap_syntax::{
    AstNode, SyntaxKind, SyntaxNode,
    ast::{self, HasGenericArgs, HasGenericParams, HasName},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

#[cfg(test)]
#[path = "original-call-resolution-tests.rs"]
mod original_call_resolution_tests;

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
    let headers = read_impl_stage_values(store, source_scope, "impl-header-index-table")?;
    let endpoints = input
        .input_node_records
        .iter()
        .filter(|node| {
            !node.node_function_body_ancestry_flag
                && node.node_input_category == HldNodeInputCategory::LocalItem
        })
        .map(|node| &node.node_target_reference)
        .collect::<BTreeSet<_>>();
    let mut edges = Vec::new();
    for site in read_impl_stage_values(store, source_scope, "body-site-reference-table")? {
        if cancel.check_stage_work_cancelled() {
            return Err(OriginalCallResolutionError::Cancelled);
        }
        if let Some(edge) = build_original_call_edge(
            &site,
            &context,
            &headers,
            &endpoints,
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
    headers: &[Value],
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
    generation: RunGenerationDigestValue,
) -> Result<Option<HldEdgeInputRecord>, OriginalCallResolutionError> {
    if !matches!(
        site["site_syntax_kind_value"].as_str(),
        Some("CALL_EXPR" | "METHOD_CALL_EXPR")
    ) || !site["span_expansion_provenance_chain"]
        .as_array()
        .is_some_and(Vec::is_empty)
    {
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
    let syntax = &site["call_syntax_evidence"];
    let resolution = if syntax["call_supported_path_flag"] != true
        || syntax["call_lexical_shadowing_flag"] != false
    {
        unresolved_call_target_value(HldUnresolvedReasonKind::TraitSolverNeeded)
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
            unresolved_call_target_value(HldUnresolvedReasonKind::AmbiguousTarget)
        } else if receiver["resolution_state_kind"] != "resolved" {
            unresolved_call_target_value(match receiver["unresolved_reason_kind"].as_str() {
                Some("cfg_inactive") => HldUnresolvedReasonKind::CfgInactive,
                Some("external_source_missing") => HldUnresolvedReasonKind::ExternalSourceMissing,
                Some("trait_solver_needed") => HldUnresolvedReasonKind::TraitSolverNeeded,
                _ => HldUnresolvedReasonKind::MissingTarget,
            })
        } else {
            resolve_inherent_member_target(
                headers,
                &receiver["target_identity"],
                syntax,
                endpoints,
            )?
        }
    };
    let target = match &resolution {
        HldEdgeResolutionValue::Extracted { target } => Some(target.clone()),
        _ => None,
    };
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
        edge_dispatch_metadata_text: Some(
            json!({
                "dispatch_kind":"original_associated_function",
                "call_syntax_evidence":syntax
            })
            .to_string(),
        ),
    }))
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
    let receiver =
        ast::MethodCallExpr::cast(node.clone()).map(|call| extract_method_receiver_syntax(&call));
    json!({"call_supported_path_flag":supported,"member_declared_name_value":member,"type_source_evidence":type_source,"call_lexical_shadowing_flag":shadowed,"method_receiver_evidence":receiver})
}

/// Syntax-only evidence for `receiver.method(..)`. Nothing is resolved here:
/// the record says whether the receiver is a plain parameter name, what type
/// that parameter declares, and whether that type is a type parameter.
fn extract_method_receiver_syntax(call: &ast::MethodCallExpr) -> Value {
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
        "receiver_shape_kind":if parameter.is_some() {"parameter_identifier"} else {"other"},
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
