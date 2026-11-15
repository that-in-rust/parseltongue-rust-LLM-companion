use crate::impl_header_source_records::{
    ImplHeaderSourceContext, ImplHeaderSourceError, read_impl_source_context,
    read_impl_stage_values,
};
use crate::syntax_namespace_record_codec::{
    decode_occurrence_identity_value, decode_symbol_target_value,
};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[cfg(test)]
#[path = "use-import-edge-tests.rs"]
mod use_import_edge_tests;

#[derive(Debug, thiserror::Error)]
pub enum UseImportEdgeError {
    #[error("use import edge resolution cancelled")]
    Cancelled,
    #[error("use import edge store: {0}")]
    Store(#[from] StoreBackendFailureError),
    #[error("use import edge identity: {0}")]
    Identity(#[from] ImplHeaderSourceError),
    #[error("use import edge evidence field is invalid: {field}")]
    InvalidEvidence { field: &'static str },
    #[error("use import edge input scope does not match its graph")]
    InvalidScope,
}

/// The `imports` rows of one graph, and how many import sites gave no row:
/// a glob, a site in macro-expanded text, a site inside no `use` node.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UseImportEdgeSet {
    pub import_edge_input_records: Vec<HldEdgeInputRecord>,
    pub imports_glob_skipped: usize,
    pub imports_in_expanded_items: usize,
    pub imports_without_source: usize,
}

/// One `imports` edge per path of an original `use` item, from the `use`
/// node to what the path names. It joins the import requests the syntax stage
/// stored with the outcomes name resolution stored and never looks a name up
/// itself. The caller supplies the frozen source scope of `input`.
pub fn resolve_use_import_edges(
    store: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    input: &HldMaterializationInputRecord,
    cancel: &RunCancelFlagValue,
) -> Result<UseImportEdgeSet, UseImportEdgeError> {
    if cancel.check_stage_work_cancelled() {
        return Err(UseImportEdgeError::Cancelled);
    }
    if source_scope.run_generation_digest_value
        != blake3::Hash::from(input.run_generation_digest_value.0)
            .to_hex()
            .as_str()
        || input.source_coverage_kind != HldSourceCoverageKind::FullSemantic
    {
        return Err(UseImportEdgeError::InvalidScope);
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
    let use_nodes = collect_use_node_keys(input);
    let outcomes = read_impl_stage_values(store, source_scope, "import-resolution-site-records")?
        .into_iter()
        .map(|outcome| {
            decode_occurrence_identity_value(&outcome["import_site_occurrence_key"])
                .map(|site| (site, outcome))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut set = UseImportEdgeSet::default();
    for request in read_impl_stage_values(store, source_scope, "syntax-namespace-input-imports")? {
        if cancel.check_stage_work_cancelled() {
            return Err(UseImportEdgeError::Cancelled);
        }
        if request["import_resolution_source_kind"] != "Import" {
            continue;
        }
        let site = decode_occurrence_identity_value(&request["import_site_occurrence_key"])?;
        if !site
            .read_semantic_occurrence_parts()
            .span_expansion_provenance_chain
            .is_empty()
        {
            set.imports_in_expanded_items += 1;
            continue;
        }
        let Some(source) = find_enclosing_use_node(&use_nodes, &site) else {
            set.imports_without_source += 1;
            continue;
        };
        if request["import_glob_expansion_flag"] == true {
            set.imports_glob_skipped += 1;
            continue;
        }
        let outcome = outcomes
            .get(&site)
            .ok_or(UseImportEdgeError::InvalidEvidence {
                field: "import_site_occurrence_key",
            })?;
        let metadata = json!({
            "import_kind":"use_path",
            "import_path_text":read_import_path_text(&request)?,
            "import_alias_text":request["import_binding_alias_text"]
        })
        .to_string();
        set.import_edge_input_records.extend(
            resolve_import_outcome_targets(outcome, &context, &endpoints)?
                .into_iter()
                .map(|resolution| {
                    build_import_edge_record(input, source, &site, resolution, &metadata)
                }),
        );
    }
    set.import_edge_input_records.sort();
    set.import_edge_input_records.dedup();
    Ok(set)
}

/// One `imports` row: from the `use` node `source`, sited at the import
/// request's own range, to `resolution`.
fn build_import_edge_record(
    input: &HldMaterializationInputRecord,
    source: &HldTargetIdentityValue,
    site: &SemanticOccurrenceKeyValue,
    resolution: HldEdgeResolutionValue,
    metadata: &str,
) -> HldEdgeInputRecord {
    let target = match &resolution {
        HldEdgeResolutionValue::Extracted { target } => Some(target.clone()),
        _ => None,
    };
    HldEdgeInputRecord {
        run_generation_digest_value: input.run_generation_digest_value,
        edge_source_reference: source.clone(),
        edge_relation_kind_value: HldRelationKindValue::Imports,
        edge_resolution_value: resolution,
        edge_evidence_row_key: HldEvidenceRowKey {
            evidence_site_occurrence_key: site.clone(),
            evidence_source_occurrence_key: source.clone(),
            evidence_relation_kind_value: HldRelationKindValue::Imports,
            evidence_target_reference_value: target,
        },
        edge_dispatch_metadata_text: Some(metadata.to_owned()),
    }
}

/// The `use` nodes of a graph that sit in original text outside any function
/// body, each with the occurrence that identifies it.
fn collect_use_node_keys(
    input: &HldMaterializationInputRecord,
) -> Vec<(&SemanticOccurrenceKeyValue, &HldTargetIdentityValue)> {
    input
        .input_node_records
        .iter()
        .filter(|node| {
            node.node_item_kind_value == Some(HldItemKindValue::UseStatement)
                && node.node_input_category == HldNodeInputCategory::LocalItem
                && !node.node_function_body_ancestry_flag
        })
        .filter_map(|node| match &node.node_target_reference {
            HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key))
                if key
                    .read_semantic_occurrence_parts()
                    .span_expansion_provenance_chain
                    .is_empty() =>
            {
                Some((key, &node.node_target_reference))
            }
            _ => None,
        })
        .collect()
}

/// The one `use` node of the same unit and file whose byte range holds an
/// import site. `None` when no node or more than one node holds it.
fn find_enclosing_use_node<'a>(
    use_nodes: &[(&SemanticOccurrenceKeyValue, &'a HldTargetIdentityValue)],
    site: &SemanticOccurrenceKeyValue,
) -> Option<&'a HldTargetIdentityValue> {
    let site = site.read_semantic_occurrence_parts();
    let (start, end) = site.span_byte_range_value.read_source_range_pair();
    let mut holders = use_nodes.iter().filter(|(key, _)| {
        let node = key.read_semantic_occurrence_parts();
        let (node_start, node_end) = node.span_byte_range_value.read_source_range_pair();
        node.unit_compilation_identity_key == site.unit_compilation_identity_key
            && node.span_source_file_path == site.span_source_file_path
            && node_start <= start
            && end <= node_end
    });
    match (holders.next(), holders.next()) {
        (Some((_, node)), None) => Some(node),
        _ => None,
    }
}

/// The path an import request names, segments joined by `::`. A trailing
/// `self` is dropped: `core::mem::{self}` imports `core::mem`.
fn read_import_path_text(request: &Value) -> Result<String, UseImportEdgeError> {
    let invalid = UseImportEdgeError::InvalidEvidence {
        field: "import_path_segment_rows",
    };
    let Some(segments) = request["import_path_segment_rows"].as_array() else {
        return Err(invalid);
    };
    let mut names = segments
        .iter()
        .map(|segment| match segment["kind"].as_str() {
            Some("name") => segment["name"].as_str(),
            Some("crate") => Some("crate"),
            Some("self") => Some("self"),
            Some("super") => Some("super"),
            Some("global") => Some(""),
            Some("defining_crate") => Some("$crate"),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(invalid)?;
    if names.len() > 1 && names.last() == Some(&"self") {
        names.pop();
    }
    Ok(names.join("::"))
}

/// What one stored import outcome gives: one extracted resolution per
/// distinct target that is a graph node, or one unresolved resolution with
/// one reason.
fn resolve_import_outcome_targets(
    outcome: &Value,
    context: &ImplHeaderSourceContext<'_>,
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
) -> Result<Vec<HldEdgeResolutionValue>, UseImportEdgeError> {
    let unresolved = |reason| Ok(vec![HldEdgeResolutionValue::Unresolved { reason }]);
    match outcome["import_resolution_state_kind"].as_str() {
        Some("resolved") => {}
        Some("ambiguous") => return unresolved(HldUnresolvedReasonKind::AmbiguousTarget),
        Some("unresolved") => {
            return unresolved(match outcome["import_unresolved_reason_kind"].as_str() {
                None => HldUnresolvedReasonKind::MissingTarget,
                Some("external_source_missing") => HldUnresolvedReasonKind::ExternalSourceMissing,
                Some("cfg_inactive") => HldUnresolvedReasonKind::CfgInactive,
                Some("trait_solver_needed") => HldUnresolvedReasonKind::TraitSolverNeeded,
                Some(_) => HldUnresolvedReasonKind::Other,
            });
        }
        _ => {
            return Err(UseImportEdgeError::InvalidEvidence {
                field: "import_resolution_state_kind",
            });
        }
    }
    let mut targets = BTreeSet::new();
    for symbol in outcome["import_target_symbol_rows"].as_array().ok_or(
        UseImportEdgeError::InvalidEvidence {
            field: "import_target_symbol_rows",
        },
    )? {
        if let Some(identity) = context.read_single_target_identity(symbol)? {
            let target = HldTargetIdentityValue::from(decode_symbol_target_value(&identity)?);
            if endpoints.contains(&target) {
                targets.insert(target);
            }
        }
    }
    if targets.is_empty() {
        return unresolved(HldUnresolvedReasonKind::MissingTarget);
    }
    Ok(targets
        .into_iter()
        .map(|target| HldEdgeResolutionValue::Extracted { target })
        .collect())
}
