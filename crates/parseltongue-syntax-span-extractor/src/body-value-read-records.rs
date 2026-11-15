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
        if let Some(edge) = build_body_read_edge(
            &site,
            request,
            &context,
            &lookup,
            &endpoints,
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
    let (prefix, member) = text.split_once("::")?;
    let prefix = prefix.trim();
    (!prefix.is_empty()
        && !member.contains("::")
        && prefix
            .chars()
            .all(|letter| letter.is_alphanumeric() || letter == '_'))
    .then_some(prefix)
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
            return unresolved(read_stored_reason_kind(
                &request["path_unresolved_reason_kind"],
            ));
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

/// The `reads` edge of one original path expression whose owner is a graph
/// node, with or without a target.
fn build_body_read_edge(
    site: &Value,
    request: &Value,
    context: &ImplHeaderSourceContext<'_>,
    lookup: &BoundMethodLookupIndex,
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
    generation: RunGenerationDigestValue,
) -> Result<Option<HldEdgeInputRecord>, BodyReadEdgeError> {
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
    let Some((resolution, metadata)) =
        resolve_read_path_target(site, request, context, lookup, endpoints)?
    else {
        return Ok(None);
    };
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

/// Whether `path` is a whole path expression in a function body that may name
/// a `static` or `const`: it is not a call's callee and its single name is not
/// bound by a parameter or pattern in scope. Items and generic parameters are
/// left to name resolution, which knows the block scopes.
pub(super) fn check_body_value_path(path: &ast::Path) -> bool {
    let Some(expression) = path.syntax().parent().and_then(ast::PathExpr::cast) else {
        return false;
    };
    let site = expression.syntax();
    let in_body = site
        .ancestors()
        .find_map(ast::Fn::cast)
        .and_then(|function| function.body())
        .is_some_and(|body| body.syntax().text_range().contains_range(site.text_range()));
    let callee = site
        .parent()
        .and_then(ast::CallExpr::cast)
        .and_then(|call| call.expr())
        .is_some_and(|callee| callee.syntax() == site);
    in_body && !callee && !check_local_value_binding(path)
}

/// Whether the single name of `path` is `self` or is bound by a pattern whose
/// scope covers the path. A pattern of another function never covers it.
fn check_local_value_binding(path: &ast::Path) -> bool {
    let name = match (path.qualifier(), path.segment()) {
        (None, Some(segment)) => segment.syntax().text().to_string(),
        _ => return false,
    };
    let name = name.trim_start_matches("r#");
    let site = path.syntax();
    let function = site.ancestors().find(|node| node.kind() == SyntaxKind::FN);
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
