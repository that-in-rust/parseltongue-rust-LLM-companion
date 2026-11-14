use crate::impl_header_source_records::{
    ImplHeaderSourceContext, ImplHeaderSourceError, read_impl_source_context,
    read_impl_stage_values,
};
use crate::syntax_namespace_record_codec::{
    decode_occurrence_identity_value, decode_symbol_target_value,
};
use crate::{SyntaxFileInputData, check_syntax_item_node, restore_source_byte_offset};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use ra_ap_syntax::{AstNode, SyntaxKind, SyntaxNode, ast};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[cfg(test)]
#[path = "signature-type-edge-tests.rs"]
mod signature_type_edge_tests;

fn check_signature_row_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::RECORD_FIELD
            | SyntaxKind::TUPLE_FIELD
            | SyntaxKind::PARAM
            | SyntaxKind::SELF_PARAM
            | SyntaxKind::RET_TYPE
            | SyntaxKind::TYPE_PARAM
            | SyntaxKind::CONST_PARAM
            | SyntaxKind::WHERE_PRED
            | SyntaxKind::TYPE_BOUND
    )
}

/// The type paths one signature row names, each with its own source range.
/// A path is listed on the nearest signature row that holds it, so a bound's
/// path is on the bound row and not also on its type parameter. Paths under an
/// expression (an array length, a const argument) are not listed.
/// `Value::Null` when the row is not in signature position: it sits under an
/// expression of its owning item, such as a closure parameter.
pub(super) fn extract_signature_type_paths(
    input: &SyntaxFileInputData,
    node: &SyntaxNode,
    removed: &[usize],
) -> Value {
    if !check_signature_row_kind(node.kind())
        || node
            .ancestors()
            .skip(1)
            .take_while(|owner| !check_syntax_item_node(owner))
            .any(|owner| ast::Expr::can_cast(owner.kind()))
    {
        return Value::Null;
    }
    node.descendants()
        .filter(|path| path.kind() == SyntaxKind::PATH_TYPE)
        .filter(|path| {
            let mut holders = path.ancestors().skip(1);
            let nearest = holders.by_ref().find(|holder| {
                check_signature_row_kind(holder.kind()) || ast::Expr::can_cast(holder.kind())
            });
            nearest.as_ref() == Some(node)
        })
        .map(|path| {
            let range = path.text_range();
            let start = restore_source_byte_offset(range.start(), removed);
            let end = restore_source_byte_offset(range.end(), removed);
            let text = std::str::from_utf8(&input.file_source_content_bytes[start..end]).ok();
            json!({"span_byte_start_offset":start,"span_byte_end_offset":end,
                "type_source_text":text,"type_path_self_flag":text == Some("Self")})
        })
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum SignatureTypeEdgeError {
    #[error("signature type edge resolution cancelled")]
    Cancelled,
    #[error("signature type edge store: {0}")]
    Store(#[from] StoreBackendFailureError),
    #[error("signature type edge identity: {0}")]
    Identity(#[from] ImplHeaderSourceError),
    #[error("signature type edge evidence field is invalid: {field}")]
    InvalidEvidence { field: &'static str },
    #[error("signature type edge input scope does not match its graph")]
    InvalidScope,
}

/// One `uses_type` edge per type path in an original signature row whose owner
/// is a graph node. It joins rows that earlier stages stored and never looks a
/// name up itself. The caller supplies the frozen source scope of `input`.
pub fn resolve_signature_type_edges(
    store: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    input: &HldMaterializationInputRecord,
    cancel: &RunCancelFlagValue,
) -> Result<Vec<HldEdgeInputRecord>, SignatureTypeEdgeError> {
    if cancel.check_stage_work_cancelled() {
        return Err(SignatureTypeEdgeError::Cancelled);
    }
    if source_scope.run_generation_digest_value
        != blake3::Hash::from(input.run_generation_digest_value.0)
            .to_hex()
            .as_str()
        || input.source_coverage_kind != HldSourceCoverageKind::FullSemantic
    {
        return Err(SignatureTypeEdgeError::InvalidScope);
    }
    let workspace = read_impl_stage_values(store, source_scope, "workspace-config-state-records")?;
    let workspace = workspace
        .first()
        .and_then(|row| row["workspace_source_root_path"].as_str())
        .unwrap_or("");
    let context = read_impl_source_context(store, source_scope, Path::new(workspace))?;
    let endpoints = input
        .input_node_records
        .iter()
        .filter(|node| {
            !node.node_function_body_ancestry_flag
                && node.node_input_category == HldNodeInputCategory::LocalItem
        })
        .map(|node| &node.node_target_reference)
        .collect::<BTreeSet<_>>();
    let graph = SignatureEdgeGraphScope {
        context: &context,
        endpoints: &endpoints,
        self_types: &read_member_self_types(store, source_scope)?,
        generation: input.run_generation_digest_value,
    };
    let mut edges = Vec::new();
    for row in read_impl_stage_values(store, source_scope, "field-signature-detail-table")? {
        if cancel.check_stage_work_cancelled() {
            return Err(SignatureTypeEdgeError::Cancelled);
        }
        edges.extend(build_signature_row_edges(&row, &graph)?);
    }
    edges.sort();
    edges.dedup();
    Ok(edges)
}

/// One `Builtin` node per builtin name the edges target, in name order, so
/// that every stored edge ends at a node.
pub fn collect_builtin_target_nodes(edges: &[HldEdgeInputRecord]) -> Vec<HldNodeInputRecord> {
    edges
        .iter()
        .filter_map(|edge| match &edge.edge_resolution_value {
            HldEdgeResolutionValue::Extracted {
                target: target @ HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(name)),
            } => Some((name, (edge.run_generation_digest_value, target))),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(name, (generation, target))| HldNodeInputRecord {
            run_generation_digest_value: generation,
            node_target_reference: target.clone(),
            node_input_category: HldNodeInputCategory::Builtin,
            node_item_kind_value: None,
            node_owner_kind_value: None,
            node_function_body_ancestry_flag: false,
            node_effective_visibility_kind: None,
            node_reachable_outside_crate_flag: None,
            node_source_relative_path: None,
            node_qualified_path_text: name.clone(),
            node_signature_text: None,
        })
        .collect()
}

/// What every signature row of one graph is joined against.
struct SignatureEdgeGraphScope<'a> {
    context: &'a ImplHeaderSourceContext<'a>,
    endpoints: &'a BTreeSet<&'a HldTargetIdentityValue>,
    /// Impl member identity to the stored resolution of its impl's self type.
    self_types: &'a BTreeMap<HldTargetIdentityValue, Value>,
    generation: RunGenerationDigestValue,
}

/// Each impl member mapped to its impl header's self type resolution, so
/// `Self` in a member signature is read from the header and never looked up.
fn read_member_self_types(
    store: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
) -> Result<BTreeMap<HldTargetIdentityValue, Value>, SignatureTypeEdgeError> {
    let mut self_types = BTreeMap::new();
    for header in read_impl_stage_values(store, source_scope, "impl-header-index-table")? {
        let members = header["impl_member_occurrence_rows"].as_array().ok_or(
            SignatureTypeEdgeError::InvalidEvidence {
                field: "impl_member_occurrence_rows",
            },
        )?;
        for member in members {
            let key = decode_occurrence_identity_value(&member["member_semantic_occurrence_key"])?;
            self_types.insert(
                NameSymbolTargetKey::Definition(key).into(),
                header["impl_self_type_resolution"].clone(),
            );
        }
    }
    Ok(self_types)
}

/// The edges of one signature row: none unless the row is original source,
/// lists its type paths, and is owned by an active item that is a graph node.
fn build_signature_row_edges(
    row: &Value,
    graph: &SignatureEdgeGraphScope<'_>,
) -> Result<Vec<HldEdgeInputRecord>, SignatureTypeEdgeError> {
    let context = graph.context;
    let (Some(paths), Some(owner)) = (
        row["type_path_source_rows"].as_array(),
        row["site_owner_occurrence_key"].as_str(),
    ) else {
        return Ok(Vec::new());
    };
    let Some(source_span) = context.span(owner).ok().filter(|span| {
        span["span_configuration_active_flag"] == true
            && row["span_expansion_provenance_chain"]
                .as_array()
                .is_some_and(Vec::is_empty)
    }) else {
        return Ok(Vec::new());
    };
    let source: HldTargetIdentityValue = NameSymbolTargetKey::Definition(
        decode_occurrence_identity_value(&context.occurrence(source_span)?)?,
    )
    .into();
    if !graph.endpoints.contains(&source) {
        return Ok(Vec::new());
    }
    paths
        .iter()
        .map(|path| build_type_path_edge(row, path, graph, &source))
        .filter_map(Result::transpose)
        .collect()
}

fn build_type_path_edge(
    row: &Value,
    path: &Value,
    graph: &SignatureEdgeGraphScope<'_>,
    source: &HldTargetIdentityValue,
) -> Result<Option<HldEdgeInputRecord>, SignatureTypeEdgeError> {
    let context = graph.context;
    let text =
        path["type_source_text"]
            .as_str()
            .ok_or(SignatureTypeEdgeError::InvalidEvidence {
                field: "type_source_text",
            })?;
    let evidence = json!({
        "unit_compilation_identity_key":row["unit_compilation_identity_key"],
        "file_source_absolute_path":row["file_source_absolute_path"],
        "span_byte_start_offset":path["span_byte_start_offset"],
        "span_byte_end_offset":path["span_byte_end_offset"],
        "span_source_code_text":text,
        "span_content_digest_full":blake3::hash(text.as_bytes()).to_hex().to_string(),
        "span_expansion_provenance_chain":row["span_expansion_provenance_chain"]
    });
    let occurrence = decode_occurrence_identity_value(&context.occurrence(&evidence)?)?;
    let Some(resolution) = resolve_type_path_target(path, &evidence, graph, source)? else {
        return Ok(None);
    };
    let target = match &resolution {
        HldEdgeResolutionValue::Extracted { target } => Some(target.clone()),
        _ => None,
    };
    let metadata = json!({
        "type_use_kind":"signature",
        "signature_row_kind":row["site_syntax_kind_value"],
        "type_source_text":text,
        "type_path_self_flag":path["type_path_self_flag"]
    });
    Ok(Some(HldEdgeInputRecord {
        run_generation_digest_value: graph.generation,
        edge_source_reference: source.clone(),
        edge_relation_kind_value: HldRelationKindValue::UsesType,
        edge_resolution_value: resolution,
        edge_evidence_row_key: HldEvidenceRowKey {
            evidence_site_occurrence_key: occurrence,
            evidence_source_occurrence_key: source.clone(),
            evidence_relation_kind_value: HldRelationKindValue::UsesType,
            evidence_target_reference_value: target,
        },
        edge_dispatch_metadata_text: Some(metadata.to_string()),
    }))
}

/// What a type path names: an item node, a builtin, or nothing provable,
/// with one reason. `None` when the path names no item at all (a type
/// parameter, or `Self` outside an impl).
///
/// `Self` in an impl member is read from the impl header's stored self type.
/// Any other path is read from its stored name resolution row.
fn resolve_type_path_target(
    path: &Value,
    evidence: &Value,
    graph: &SignatureEdgeGraphScope<'_>,
    source: &HldTargetIdentityValue,
) -> Result<Option<HldEdgeResolutionValue>, SignatureTypeEdgeError> {
    let unresolved = |reason| Ok(Some(HldEdgeResolutionValue::Unresolved { reason }));
    let target = if path["type_path_self_flag"] == true {
        match graph.self_types.get(source) {
            None => return Ok(None),
            Some(header) if header["resolution_state_kind"] == "resolved" => {
                decode_symbol_target_value(&header["target_identity"])?
            }
            Some(header) => {
                return unresolved(read_stored_reason_kind(&header["unresolved_reason_kind"]));
            }
        }
    } else {
        let record = match graph.context.read_type_path_records(evidence, path)? {
            [] => return unresolved(HldUnresolvedReasonKind::TraitSolverNeeded),
            [record] => record,
            _ => return unresolved(HldUnresolvedReasonKind::AmbiguousTarget),
        };
        let targets = record["path_target_symbol_rows"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let symbol = match (record["path_resolution_state_kind"].as_str(), targets) {
            (Some("resolved"), [symbol]) => symbol,
            (Some("ambiguous"), _) | (_, [_, _, ..]) => {
                return unresolved(HldUnresolvedReasonKind::AmbiguousTarget);
            }
            _ => {
                return unresolved(read_stored_reason_kind(
                    &record["path_unresolved_reason_kind"],
                ));
            }
        };
        match symbol["target_symbol_kind_value"].as_str() {
            Some("TypeParam" | "ConstParam" | "SelfType") => return Ok(None),
            Some("Builtin") => symbol["target_builtin_name_value"]
                .as_str()
                .map(|name| NameSymbolTargetKey::Builtin(name.to_owned()))
                .ok_or(SignatureTypeEdgeError::InvalidEvidence {
                    field: "target_builtin_name_value",
                })?,
            _ => match graph.context.read_single_target_identity(symbol)? {
                Some(identity) => decode_symbol_target_value(&identity)?,
                None => return unresolved(HldUnresolvedReasonKind::MissingTarget),
            },
        }
    };
    let builtin = matches!(target, NameSymbolTargetKey::Builtin(_));
    let target = HldTargetIdentityValue::from(target);
    if builtin || graph.endpoints.contains(&target) {
        Ok(Some(HldEdgeResolutionValue::Extracted { target }))
    } else {
        unresolved(HldUnresolvedReasonKind::MissingTarget)
    }
}

/// The graph reason for a reason name resolution stored. A path stored
/// without a reason needs more than name lookup to resolve.
fn read_stored_reason_kind(reason: &Value) -> HldUnresolvedReasonKind {
    match reason.as_str() {
        Some("external_source_missing") => HldUnresolvedReasonKind::ExternalSourceMissing,
        Some("cfg_inactive") => HldUnresolvedReasonKind::CfgInactive,
        _ => HldUnresolvedReasonKind::TraitSolverNeeded,
    }
}
