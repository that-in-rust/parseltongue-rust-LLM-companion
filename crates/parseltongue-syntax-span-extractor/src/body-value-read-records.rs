use crate::expanded_site_origin_records::{ExpandedTokenOriginIndex, read_expanded_token_origins};
use crate::impl_header_source_records::{
    ImplHeaderSourceContext, ImplHeaderSourceError, read_impl_source_context,
    read_impl_stage_values,
};
use crate::original_call_resolution_records::{
    BoundMethodLookupIndex, OriginalCallResolutionError, build_bound_method_index,
    walk_parameter_bound_traits,
};
use crate::signature_type_edge_records::read_stored_reason_kind;
use crate::syntax_namespace_record_codec::{
    decode_occurrence_identity_value, decode_symbol_target_value,
};
use crate::type_relative_member_records::{
    ImplMemberLookupIndex, find_qualified_path_member, read_impl_member_index,
    read_type_relative_parts,
};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use ra_ap_syntax::{AstNode, SyntaxKind, SyntaxNode, ast, ast::HasLoopBody};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

#[cfg(test)]
#[path = "body-value-read-tests.rs"]
mod body_value_read_tests;

#[cfg(test)]
#[path = "body-read-edge-tests.rs"]
mod body_read_edge_tests;

#[derive(Debug, thiserror::Error)]
pub enum BodyReadEdgeError {
    #[error("body read edge resolution cancelled")]
    Cancelled,
    #[error("body read edge store: {0}")]
    Store(#[from] StoreBackendFailureError),
    #[error("body read edge identity: {0}")]
    Identity(#[from] ImplHeaderSourceError),
    #[error("body read edge bound lookup: {0}")]
    Bound(#[from] OriginalCallResolutionError),
    #[error("body read edge evidence field is invalid: {field}")]
    InvalidEvidence { field: &'static str },
    #[error("body read edge input scope does not match its graph")]
    InvalidScope,
}

/// The `reads` edges of one graph, and how many of them are sited in the
/// expanded text of a macro-generated body.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BodyReadEdgeSet {
    pub read_edge_input_records: Vec<HldEdgeInputRecord>,
    pub read_expanded_site_count: usize,
}

/// One `reads` edge per value path in an original function body that names a
/// `static` or `const` graph node, or that name resolution left without one
/// target. It joins rows that earlier stages stored
/// and never looks a name up itself. The caller supplies the frozen source
/// scope of `input`.
pub fn resolve_body_read_edges(
    store: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    input: &HldMaterializationInputRecord,
    cancel: &RunCancelFlagValue,
) -> Result<BodyReadEdgeSet, BodyReadEdgeError> {
    if cancel.check_stage_work_cancelled() {
        return Err(BodyReadEdgeError::Cancelled);
    }
    if source_scope.run_generation_digest_value
        != blake3::Hash::from(input.run_generation_digest_value.0)
            .to_hex()
            .as_str()
        || input.source_coverage_kind != HldSourceCoverageKind::FullSemantic
    {
        return Err(BodyReadEdgeError::InvalidScope);
    }
    let workspace = read_impl_stage_values(store, source_scope, "workspace-config-state-records")?;
    let workspace = workspace
        .first()
        .and_then(|row| row["workspace_source_root_path"].as_str())
        .unwrap_or("");
    let context = read_impl_source_context(store, source_scope, Path::new(workspace))?;
    let lookup = build_bound_method_index(store, source_scope, &context, input)?;
    let origins = read_expanded_token_origins(store, source_scope)?;
    let members = read_impl_member_index(store, source_scope, &input.input_node_records)?;
    let macros = read_impl_stage_values(store, source_scope, "legacy-macro-scope-records")?;
    let endpoints = input
        .input_node_records
        .iter()
        .filter(|node| {
            !node.node_function_body_ancestry_flag
                && node.node_input_category == HldNodeInputCategory::LocalItem
        })
        .map(|node| &node.node_target_reference)
        .collect::<BTreeSet<_>>();
    let mut edges = BodyReadEdgeSet::default();
    for site in read_impl_stage_values(store, source_scope, "body-site-reference-table")? {
        if cancel.check_stage_work_cancelled() {
            return Err(BodyReadEdgeError::Cancelled);
        }
        let Some(request) = read_value_path_request(&site, &context)? else {
            continue;
        };
        if !check_macro_argument_site(&site, &macros) {
            continue;
        }
        if let Some(edge) = build_body_read_edge(
            &site,
            request,
            &context,
            &ReadTargetLookupScope {
                lookup: &lookup,
                members: &members,
                origins: &origins,
                endpoints: &endpoints,
            },
            input.run_generation_digest_value,
        )? {
            let expanded = !site["span_expansion_provenance_chain"]
                .as_array()
                .is_some_and(Vec::is_empty);
            edges.read_expanded_site_count += usize::from(expanded);
            edges.read_edge_input_records.push(edge);
        }
    }
    edges.read_edge_input_records.sort();
    edges.read_edge_input_records.dedup();
    Ok(edges)
}

/// Whether `site` may be read as a value path. A path in the arguments of an
/// assertion macro call may only when the stored textual-scope lookups of the
/// macro's name at that call exist and the latest finds no `macro_rules!`
/// definition: the arguments of a macro of this crate are not known to be
/// expressions (REQ-RUST-052.0 G15). Any other site may.
fn check_macro_argument_site(site: &Value, lookups: &[Value]) -> bool {
    let argument = &site["site_macro_argument_record"];
    if argument.is_null() {
        return true;
    }
    let range = json!([
        argument["macro_path_byte_start_offset"],
        argument["macro_path_byte_end_offset"]
    ]);
    let stored = lookups
        .iter()
        .filter(|row| {
            let key = &row["legacy_source_site_key"];
            row["legacy_macro_name_text"] == argument["macro_path_source_text"]
                && key["span_byte_range_value"] == range
                && key["span_expansion_provenance_chain"] == site["span_expansion_provenance_chain"]
                && key["span_source_file_path"].as_str().is_some_and(|path| {
                    site["file_source_absolute_path"]
                        .as_str()
                        .is_some_and(|file| Path::new(file).ends_with(path))
                })
        })
        .collect::<Vec<_>>();
    let latest = stored
        .iter()
        .filter_map(|row| row["legacy_scope_revision_value"].as_u64())
        .max();
    latest.is_some()
        && stored
            .iter()
            .filter(|row| row["legacy_scope_revision_value"].as_u64() == latest)
            .all(|row| row["legacy_target_identity_key"].is_null())
}

/// A stored field of a body site row that must be text.
fn read_site_text_field<'a>(
    site: &'a Value,
    field: &'static str,
) -> Result<&'a str, BodyReadEdgeError> {
    site[field]
        .as_str()
        .ok_or(BodyReadEdgeError::InvalidEvidence { field })
}

/// The source evidence of a body site, in the shape a span row has.
fn build_site_evidence_value(site: &Value) -> Result<Value, BodyReadEdgeError> {
    let text = read_site_text_field(site, "site_syntax_source_text")?;
    Ok(json!({
        "unit_compilation_identity_key":site["unit_compilation_identity_key"],
        "file_source_absolute_path":site["file_source_absolute_path"],
        "span_byte_start_offset":site["span_byte_start_offset"],
        "span_byte_end_offset":site["span_byte_end_offset"],
        "span_source_code_text":text,
        "span_content_digest_full":blake3::hash(text.as_bytes()).to_hex().to_string(),
        "span_expansion_provenance_chain":site["span_expansion_provenance_chain"]
    }))
}

/// The stored value-path resolution of a path expression site. `None` when
/// the site is not a path expression or no value-path request was made for
/// it: the name is bound locally, or the path is a call's callee.
fn read_value_path_request<'a>(
    site: &Value,
    context: &'a ImplHeaderSourceContext<'_>,
) -> Result<Option<&'a Value>, BodyReadEdgeError> {
    if site["site_syntax_kind_value"] != "PATH_EXPR" {
        return Ok(None);
    }
    let evidence = build_site_evidence_value(site)?;
    Ok(context
        .read_type_path_records(&evidence, &evidence)?
        .iter()
        .find(|record| record["site_path_context_kind"] == "body_value_path"))
}

/// The first segment of a two-segment path such as `T::NAME`, when it is one
/// plain identifier.
fn read_type_relative_prefix(text: &str) -> Option<&str> {
    read_type_relative_parts(text).map(|(prefix, _)| prefix)
}

/// `Self::NAME` or `u8::NAME` that name lookup left without a target: the
/// `const` member of the one stored trait impl of that type, with the record
/// of what proved it (REQ-RUST-052.0 G5). `Self` is the stored self type of the
/// owner's impl; any other prefix is read from its stored type lookup.
fn find_type_relative_const(
    site: &Value,
    text: &str,
    context: &ImplHeaderSourceContext<'_>,
    members: &ImplMemberLookupIndex,
) -> Result<Option<(Option<HldTargetIdentityValue>, Value)>, BodyReadEdgeError> {
    let Some((prefix, name)) = read_type_relative_parts(text) else {
        return Ok(None);
    };
    let looked_up;
    let self_type = if prefix == "Self" {
        members.read_owner_self_type(read_site_text_field(site, "site_owner_occurrence_key")?)
    } else {
        let evidence = build_site_evidence_value(site)?;
        let start = evidence["span_byte_start_offset"].as_u64().ok_or(
            BodyReadEdgeError::InvalidEvidence {
                field: "span_byte_start_offset",
            },
        )?;
        let source = json!({
            "span_byte_start_offset":start,
            "span_byte_end_offset":start + prefix.len() as u64,
            "type_simple_path_flag":true
        });
        looked_up = context.resolve_type(&evidence, &source, false)?;
        Some(&looked_up)
    };
    self_type
        .and_then(|self_type| members.find_type_member_target(prefix, self_type, name, "CONST"))
        .map(|found| {
            let target = found
                .member_target_occurrence
                .map(decode_occurrence_identity_value)
                .transpose()?
                .map(|key| NameSymbolTargetKey::Definition(key).into());
            Ok((target, found.member_lookup_evidence))
        })
        .transpose()
}

/// How one stored value-path resolution becomes a `reads` edge: the edge's
/// resolution and its metadata. `None` when the path names something that is
/// not a `static` or `const` graph node, such as a function, a const generic
/// parameter or a const declared in a function body.
///
/// A path left unresolved or ambiguous keeps one reason. A path through a type
/// parameter of the owner has a target only when the target is a member of a
/// trait the parameter is bound by; the metadata then names the bounds walked
/// and each bound trait whose source was not read.
fn resolve_read_path_target(
    site: &Value,
    request: &Value,
    context: &ImplHeaderSourceContext<'_>,
    lookup: &BoundMethodLookupIndex,
    members: &ImplMemberLookupIndex,
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
) -> Result<Option<(HldEdgeResolutionValue, Value)>, BodyReadEdgeError> {
    let text = read_site_text_field(site, "site_syntax_source_text")?;
    let describe = |kind: Option<&str>, bound: Value| {
        json!({
            "read_kind":"body_value_path",
            "read_source_text":text,
            "read_target_symbol_kind":kind,
            "bound_read_evidence":bound
        })
    };
    let unresolved = |reason| {
        Ok(Some((
            HldEdgeResolutionValue::Unresolved { reason },
            describe(None, Value::Null),
        )))
    };
    let targets = request["path_target_symbol_rows"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let symbol = match (request["path_resolution_state_kind"].as_str(), targets) {
        (Some("resolved"), [symbol]) => symbol,
        (Some("ambiguous"), _) | (_, [_, _, ..]) => {
            return unresolved(HldUnresolvedReasonKind::AmbiguousTarget);
        }
        _ => {
            let reason = read_stored_reason_kind(&request["path_unresolved_reason_kind"]);
            if reason != HldUnresolvedReasonKind::TraitSolverNeeded {
                return unresolved(reason);
            }
            // The member lookup's record stays on the row whether or not it proved a target.
            let Some((target, proof)) = find_type_relative_const(site, text, context, members)?
            else {
                return unresolved(reason);
            };
            let target = target.filter(|target| endpoints.contains(target));
            // REQ-RUST-053.0 N3: no stored impl declares the member of a builtin type.
            let named = match &target {
                Some(_) => None,
                None => read_type_relative_parts(text)
                    .and_then(|(_, name)| members.find_builtin_member_name(&proof, name)),
            };
            let mut metadata = describe(target.as_ref().map(|_| "Const"), Value::Null);
            metadata["type_relative_member_evidence"] = proof;
            let target = match named {
                Some(Ok(named)) => {
                    metadata["external_target_proof_record"] = named.external_proof_metadata_value;
                    Some(named.external_target_identity_value)
                }
                Some(Err(withheld)) => {
                    metadata["external_name_withheld_kind"] = json!(withheld);
                    None
                }
                None => target,
            };
            return Ok(Some((
                match target {
                    Some(target) => HldEdgeResolutionValue::Extracted { target },
                    None => HldEdgeResolutionValue::Unresolved { reason },
                },
                metadata,
            )));
        }
    };
    let kind = symbol["target_symbol_kind_value"].as_str();
    if !matches!(kind, Some("Static" | "Const")) {
        return Ok(None);
    }
    let Some(target) = context
        .read_single_target_identity(symbol)?
        .map(|identity| decode_symbol_target_value(&identity))
        .transpose()?
        .map(HldTargetIdentityValue::from)
        .filter(|target| endpoints.contains(target))
    else {
        return Ok(None);
    };
    // `<A as T>::NAME`: name lookup gives the trait's declaration. The member of the one
    // stored impl of `T` for `A` is the more exact target (REQ-RUST-052.0 G14); the row
    // keeps the declaration and the lookup record either way.
    if kind == Some("Const")
        && let Some(found) = find_qualified_path_member(
            &build_site_evidence_value(site)?,
            text,
            context,
            members,
            "CONST",
        )?
    {
        let member = found
            .member_target_occurrence
            .map(decode_occurrence_identity_value)
            .transpose()?
            .map(|key| HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(key)))
            .filter(|member| endpoints.contains(member));
        let mut metadata = describe(kind, Value::Null);
        metadata["type_relative_member_evidence"] = found.member_lookup_evidence;
        metadata["trait_declaration_symbol_row"] = symbol.clone();
        return Ok(Some((
            HldEdgeResolutionValue::Extracted {
                target: member.unwrap_or(target),
            },
            metadata,
        )));
    }
    let walk = read_type_relative_prefix(text)
        .map(|prefix| {
            walk_parameter_bound_traits(
                read_site_text_field(site, "site_owner_occurrence_key")?,
                prefix,
                context,
                lookup,
            )
            .map_err(BodyReadEdgeError::from)
        })
        .transpose()?;
    let bound = match walk {
        // A bound in a form the walk cannot prove.
        Some(None) => return unresolved(HldUnresolvedReasonKind::TraitSolverNeeded),
        Some(Some(walk))
            if !walk.bound_trait_targets.is_empty() || !walk.missing_source_traits.is_empty() =>
        {
            let member = walk.bound_trait_targets.iter().any(|found| {
                lookup
                    .contained_item_pairs
                    .contains(&(found.clone(), target.clone()))
            });
            if !member {
                return unresolved(HldUnresolvedReasonKind::TraitSolverNeeded);
            }
            walk.build_walk_evidence_value()
        }
        _ => Value::Null,
    };
    Ok(Some((
        HldEdgeResolutionValue::Extracted { target },
        describe(kind, bound),
    )))
}

/// The stored rows a read's target and origin are looked up in.
#[derive(Clone, Copy)]
struct ReadTargetLookupScope<'a> {
    lookup: &'a BoundMethodLookupIndex,
    members: &'a ImplMemberLookupIndex,
    origins: &'a ExpandedTokenOriginIndex,
    endpoints: &'a BTreeSet<&'a HldTargetIdentityValue>,
}

/// The `reads` edge of one path expression whose owner is a graph node, with
/// or without a target. A row sited in expanded text also names where its
/// first and its last token were copied from.
fn build_body_read_edge(
    site: &Value,
    request: &Value,
    context: &ImplHeaderSourceContext<'_>,
    scope: &ReadTargetLookupScope<'_>,
    generation: RunGenerationDigestValue,
) -> Result<Option<HldEdgeInputRecord>, BodyReadEdgeError> {
    let ReadTargetLookupScope {
        lookup,
        members,
        origins,
        endpoints,
    } = *scope;
    let owner = context.span(read_site_text_field(site, "site_owner_occurrence_key")?)?;
    // A site and its owner are in the same text: both original, or both in one expansion.
    if [
        "span_expansion_provenance_chain",
        "file_source_absolute_path",
    ]
    .iter()
    .any(|field| site[field] != owner[field])
    {
        return Err(BodyReadEdgeError::InvalidEvidence {
            field: "span_expansion_provenance_chain",
        });
    }
    if owner["span_configuration_active_flag"] != true {
        return Ok(None);
    }
    let source: HldTargetIdentityValue = NameSymbolTargetKey::Definition(
        decode_occurrence_identity_value(&context.occurrence(owner)?)?,
    )
    .into();
    if !endpoints.contains(&source) {
        return Ok(None);
    }
    let Some((resolution, mut metadata)) =
        resolve_read_path_target(site, request, context, lookup, members, endpoints)?
    else {
        return Ok(None);
    };
    let origin = origins.build_site_origin_value(site, context)?;
    if !origin.is_null() {
        metadata["expansion_source_origin"] = origin;
    }
    let argument = &site["site_macro_argument_record"];
    if !argument.is_null() {
        metadata["read_kind"] = json!("macro_argument_value_path");
        metadata["macro_argument_read_evidence"] = json!({
            "macro_path_source_text":argument["macro_path_source_text"],
            "macro_call_occurrence_key":argument["macro_call_occurrence_key"],
            "macro_textual_scope_outcome_kind":"no_local_macro_definition",
            "sources_not_read":[
                "the definition of the macro in the standard library",
                "a macro of this name brought into scope by a use item or by #[macro_use] extern crate"
            ]
        });
    }
    let evidence = build_site_evidence_value(site)?;
    let occurrence = decode_occurrence_identity_value(&context.occurrence(&evidence)?)?;
    let target = match &resolution {
        HldEdgeResolutionValue::Extracted { target } => Some(target.clone()),
        _ => None,
    };
    Ok(Some(HldEdgeInputRecord {
        run_generation_digest_value: generation,
        edge_source_reference: source.clone(),
        edge_relation_kind_value: HldRelationKindValue::Reads,
        edge_resolution_value: resolution,
        edge_evidence_row_key: HldEvidenceRowKey {
            evidence_site_occurrence_key: occurrence,
            evidence_source_occurrence_key: source,
            evidence_relation_kind_value: HldRelationKindValue::Reads,
            evidence_target_reference_value: target,
        },
        edge_dispatch_metadata_text: Some(metadata.to_string()),
    }))
}

/// Whether `path` is a whole path expression that may name a `static` or
/// `const`: it is not a call's callee and its single name is not bound by a
/// parameter or pattern in scope. It is written in a function body, in the
/// initializer of a `const` or `static`, or in an expression of a type, such as
/// an array length (REQ-RUST-052.0 G7). Items and generic parameters are left
/// to name resolution, which knows the block scopes.
pub(super) fn check_body_value_path(path: &ast::Path) -> bool {
    let Some(expression) = path.syntax().parent().and_then(ast::PathExpr::cast) else {
        return false;
    };
    let site = expression.syntax();
    let in_body = site
        .ancestors()
        .skip(1)
        .any(|node| ast::Item::can_cast(node.kind()));
    let callee = site
        .parent()
        .and_then(ast::CallExpr::cast)
        .and_then(|call| call.expr())
        .is_some_and(|callee| callee.syntax() == site);
    in_body && !callee && !check_local_value_binding(path)
}

/// Whether `path` is the one-name qualifier of a two-segment body value path
/// such as `u8::NAME`. It is submitted to the namespace engine as a Type
/// request, in the actual lexical scope. `Self` is not: it is read from the
/// impl header.
pub(super) fn check_body_value_prefix(path: &ast::Path) -> bool {
    path.qualifier().is_none()
        && path.segment().is_some_and(|segment| {
            segment.name_ref().is_some() && segment.self_type_token().is_none()
        })
        && path
            .syntax()
            .parent()
            .and_then(ast::Path::cast)
            .is_some_and(|outer| {
                outer.qualifier().as_ref() == Some(path) && check_body_value_path(&outer)
            })
}

/// Whether the single name of `path` is `self` or is bound by a pattern whose
/// scope covers the path. A pattern outside the nearest item the path is
/// written in, such as one of another function, never covers it.
pub(super) fn check_local_value_binding(path: &ast::Path) -> bool {
    match (path.qualifier(), path.segment()) {
        (None, Some(segment)) => {
            check_value_name_bound(&segment.syntax().text().to_string(), path.syntax())
        }
        _ => false,
    }
}

/// Whether `name` is `self` or is bound by a pattern whose scope covers `site`.
pub(super) fn check_value_name_bound(name: &str, site: &SyntaxNode) -> bool {
    let name = name.trim_start_matches("r#");
    // An item written in a function sees none of the function's locals.
    let function = site
        .ancestors()
        .find(|node| ast::Item::can_cast(node.kind()));
    name == "self"
        || function.iter().any(|function| {
            function
                .descendants()
                .filter(|pattern| pattern.kind() == SyntaxKind::IDENT_PAT)
                .filter(|pattern| {
                    pattern
                        .children()
                        .filter_map(ast::Name::cast)
                        .any(|bound| bound.text().trim_start_matches("r#") == name)
                })
                .any(|pattern| check_pattern_scope_covers(&pattern, site))
        })
}

/// Whether the names `pattern` binds are in scope at `site`: after a `let`
/// to the end of its block, inside a match arm, inside the taken branch of an
/// `if let`, inside a loop body, or anywhere in a function or closure that
/// takes it as a parameter.
fn check_pattern_scope_covers(pattern: &SyntaxNode, site: &SyntaxNode) -> bool {
    let site = site.text_range();
    let within = |scope: Option<SyntaxNode>| {
        scope.is_some_and(|scope| scope.text_range().contains_range(site))
    };
    let Some(binder) = pattern.ancestors().find(|node| {
        matches!(
            node.kind(),
            SyntaxKind::LET_STMT
                | SyntaxKind::LET_EXPR
                | SyntaxKind::MATCH_ARM
                | SyntaxKind::FOR_EXPR
                | SyntaxKind::PARAM
        )
    }) else {
        return false;
    };
    let after = site.start() >= binder.text_range().end();
    match binder.kind() {
        SyntaxKind::LET_STMT => after && within(binder.parent()),
        SyntaxKind::MATCH_ARM => site.start() >= pattern.text_range().end() && within(Some(binder)),
        SyntaxKind::FOR_EXPR => within(
            ast::ForExpr::cast(binder)
                .and_then(|each| each.loop_body())
                .map(|body| body.syntax().clone()),
        ),
        SyntaxKind::PARAM => within(
            binder
                .ancestors()
                .find(|node| matches!(node.kind(), SyntaxKind::FN | SyntaxKind::CLOSURE_EXPR)),
        ),
        _ => {
            let scope = binder.ancestors().find(|node| {
                matches!(
                    node.kind(),
                    SyntaxKind::IF_EXPR | SyntaxKind::WHILE_EXPR | SyntaxKind::MATCH_ARM
                )
            });
            let taken = scope
                .clone()
                .and_then(ast::IfExpr::cast)
                .and_then(|branch| branch.then_branch())
                .is_none_or(|then| site.end() <= then.syntax().text_range().end());
            after && taken && within(scope)
        }
    }
}
