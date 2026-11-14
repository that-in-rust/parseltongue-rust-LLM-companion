use crate::impl_header_source_records::{ImplHeaderSourceError, read_impl_stage_values};
use crate::syntax_namespace_record_codec::{
    decode_occurrence_identity_value, decode_symbol_target_value,
};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[cfg(test)]
#[path = "impl-header-implements-tests.rs"]
mod impl_header_implements_tests;

#[derive(Debug, thiserror::Error)]
pub enum ImplHeaderEdgeError {
    #[error("impl header edge resolution cancelled")]
    Cancelled,
    #[error("impl header edge store: {0}")]
    Store(#[from] StoreBackendFailureError),
    #[error("impl header edge identity: {0}")]
    Identity(#[from] ImplHeaderSourceError),
    #[error("impl header edge evidence field is invalid: {field}")]
    InvalidEvidence { field: &'static str },
    #[error("impl header edge input scope does not match its graph")]
    InvalidScope,
}

/// The `implements` rows of one graph, and how many trait impl headers have
/// a self type that is not a graph node and so could not source a row.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ImplHeaderEdgeSet {
    pub implements_edge_records: Vec<HldEdgeInputRecord>,
    pub headers_without_source: usize,
}

/// One `implements` edge per trait impl header, from the header's self type
/// to its trait. It joins what the impl header stage stored and never looks a
/// name up itself. The caller supplies the frozen source scope of `input`.
pub fn resolve_impl_header_edges(
    store: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    input: &HldMaterializationInputRecord,
    cancel: &RunCancelFlagValue,
) -> Result<ImplHeaderEdgeSet, ImplHeaderEdgeError> {
    if cancel.check_stage_work_cancelled() {
        return Err(ImplHeaderEdgeError::Cancelled);
    }
    if source_scope.run_generation_digest_value
        != blake3::Hash::from(input.run_generation_digest_value.0)
            .to_hex()
            .as_str()
        || input.source_coverage_kind != HldSourceCoverageKind::FullSemantic
    {
        return Err(ImplHeaderEdgeError::InvalidScope);
    }
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
    for header in read_impl_stage_values(store, source_scope, "impl-header-index-table")? {
        if cancel.check_stage_work_cancelled() {
            return Err(ImplHeaderEdgeError::Cancelled);
        }
        edges.extend(build_impl_header_edge(
            &header,
            &endpoints,
            input.run_generation_digest_value,
        )?);
    }
    edges.sort();
    edges.dedup();
    Ok(ImplHeaderEdgeSet {
        implements_edge_records: edges,
        headers_without_source: 0,
    })
}

/// The row of one header: none unless it is an original, positive trait impl
/// outside any function body whose self type and trait are both graph nodes.
fn build_impl_header_edge(
    header: &Value,
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
    generation: RunGenerationDigestValue,
) -> Result<Option<HldEdgeInputRecord>, ImplHeaderEdgeError> {
    let (self_type, trait_type) = (
        &header["impl_self_type_resolution"],
        &header["impl_trait_type_resolution"],
    );
    let header_key = decode_occurrence_identity_value(&header["impl_semantic_occurrence_key"])?;
    let header_parts = header_key.read_semantic_occurrence_parts();
    if trait_type.is_null()
        || header["impl_negative_implementation_flag"] == true
        || header["impl_function_body_ancestry_flag"] == true
        || !header_parts.span_expansion_provenance_chain.is_empty()
    {
        return Ok(None);
    }
    let (Some(source), Some(target)) = (
        read_header_node_target(self_type, endpoints)?,
        read_header_node_target(trait_type, endpoints)?,
    ) else {
        return Ok(None);
    };
    let evidence = &trait_type["type_source_evidence"];
    let invalid = |field| ImplHeaderEdgeError::InvalidEvidence { field };
    let text = evidence["type_source_text"]
        .as_str()
        .ok_or(invalid("type_source_text"))?;
    let range = evidence["span_byte_start_offset"]
        .as_u64()
        .zip(evidence["span_byte_end_offset"].as_u64())
        .and_then(|range| SpanByteRangeValue::try_from(range).ok())
        .ok_or(invalid("span_byte_start_offset"))?;
    let site = SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        header_parts.unit_compilation_identity_key,
        header_parts.span_source_file_path,
        range,
        SpanDigestIdValue::from(*blake3::hash(text.as_bytes()).as_bytes()),
        header_parts.span_expansion_provenance_chain,
    )
    .map_err(|_| invalid("impl_semantic_occurrence_key"))?;
    let metadata = json!({
        "implements_kind":"impl_header",
        "self_type_source_text":self_type["type_source_evidence"]["type_source_text"],
        "trait_source_text":text
    });
    Ok(Some(HldEdgeInputRecord {
        run_generation_digest_value: generation,
        edge_source_reference: source.clone(),
        edge_relation_kind_value: HldRelationKindValue::Implements,
        edge_resolution_value: HldEdgeResolutionValue::Extracted {
            target: target.clone(),
        },
        edge_evidence_row_key: HldEvidenceRowKey {
            evidence_site_occurrence_key: site,
            evidence_source_occurrence_key: source,
            evidence_relation_kind_value: HldRelationKindValue::Implements,
            evidence_target_reference_value: Some(target),
        },
        edge_dispatch_metadata_text: Some(metadata.to_string()),
    }))
}

/// What a header's self type or trait names when that is a node of the
/// graph: a builtin, or an item among `endpoints`.
fn read_header_node_target(
    resolution: &Value,
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
) -> Result<Option<HldTargetIdentityValue>, ImplHeaderEdgeError> {
    if resolution["resolution_state_kind"] != "resolved" {
        return Ok(None);
    }
    let target = decode_symbol_target_value(&resolution["target_identity"])?;
    let builtin = matches!(target, NameSymbolTargetKey::Builtin(_));
    let target = HldTargetIdentityValue::from(target);
    Ok((builtin || endpoints.contains(&target)).then_some(target))
}
