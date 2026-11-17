//! REQ-RUST-069.0 V2-V7, R1-R5: the effective visibility of every node and whether it is
//! reachable from outside the crate. Reads what the impl header stage stored and the
//! `contains` rows of the graph; the rules themselves are `derive_node_reach_fields`.

use crate::external_named_target_records::decode_external_name_record;
use crate::impl_header_implements_records::{ImplHeaderEdgeError, read_header_node_target};
use crate::impl_header_source_records::{ImplHeaderSourceError, read_impl_stage_values};
use crate::signature_type_edge_records::read_stored_reason_kind;
use crate::syntax_namespace_record_codec::decode_occurrence_identity_value;
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, thiserror::Error)]
pub enum NodeVisibilityFieldError {
    #[error("node visibility field derivation cancelled")]
    Cancelled,
    #[error("node visibility field input scope does not match its graph")]
    InvalidScope,
    #[error("node visibility field impl header: {0}")]
    Header(#[from] ImplHeaderEdgeError),
    #[error("node visibility field impl header source: {0}")]
    Source(#[from] ImplHeaderSourceError),
    #[error("node visibility field store: {0}")]
    Store(#[from] StoreBackendFailureError),
    #[error("node visibility field impl header field is invalid: {field}")]
    InvalidHeader { field: &'static str },
}

type NodeReachOwner = NodeReachOwnerValue<HldTargetIdentityValue>;
type NodeReachParent = NodeReachParentValue<HldTargetIdentityValue>;

/// The nodes of `input` with their effective visibility and reachability set. A node
/// arrives with its declared visibility (V1); the crate root, names outside the read
/// source and builtin types keep no value (V6). The caller supplies the frozen source
/// scope of `input`.
pub fn resolve_node_visibility_fields(
    store: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    input: &HldMaterializationInputRecord,
    cancel: &RunCancelFlagValue,
) -> Result<Vec<HldNodeInputRecord>, NodeVisibilityFieldError> {
    if cancel.check_stage_work_cancelled() {
        return Err(NodeVisibilityFieldError::Cancelled);
    }
    if source_scope.run_generation_digest_value
        != blake3::Hash::from(input.run_generation_digest_value.0)
            .to_hex()
            .as_str()
        || input.source_coverage_kind != HldSourceCoverageKind::FullSemantic
    {
        return Err(NodeVisibilityFieldError::InvalidScope);
    }
    let local_items = input
        .input_node_records
        .iter()
        .filter(|node| {
            !node.node_function_body_ancestry_flag
                && node.node_input_category == HldNodeInputCategory::LocalItem
        })
        .collect::<Vec<_>>();
    let endpoints = local_items
        .iter()
        .map(|node| &node.node_target_reference)
        .collect::<BTreeSet<_>>();
    let member_owners = read_impl_member_owners(store, source_scope, &endpoints, cancel)?;
    let containers = input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Contains)
        .filter_map(|edge| match &edge.edge_resolution_value {
            HldEdgeResolutionValue::Extracted { target } => {
                Some((target, &edge.edge_source_reference))
            }
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    // R5: taken over the whole graph, which is never less careful than per unit.
    let reexport = local_items.iter().any(|node| {
        node.node_item_kind_value == Some(HldItemKindValue::UseStatement)
            && node.node_effective_visibility_kind
                != Some(HldEffectiveVisibilityKind::PrivateEquivalent)
    });
    let records = input
        .input_node_records
        .iter()
        .filter(|node| !node.node_function_body_ancestry_flag)
        .filter_map(|node| {
            let key = &node.node_target_reference;
            let container = || match containers.get(key) {
                Some(HldTargetIdentityValue::Symbol(NameSymbolTargetKey::CrateRoot(_))) => {
                    NodeReachParent::CrateRoot
                }
                Some(holder) => NodeReachParent::Node((*holder).clone()),
                None => NodeReachParent::Unresolved,
            };
            let owner = match (
                node.node_input_category,
                node.node_item_kind_value,
                node.node_owner_kind_value,
            ) {
                (HldNodeInputCategory::FilePublicArea, ..) => NodeReachOwner::FileArea {
                    module: container(),
                },
                (HldNodeInputCategory::LocalItem, Some(HldItemKindValue::MacroInvocation), _) => {
                    NodeReachOwner::MacroCall
                }
                // A member no stored header lists keeps unknown fields.
                (HldNodeInputCategory::LocalItem, _, Some(HldOwnerKindValue::Impl)) => {
                    member_owners
                        .get(key)
                        .cloned()
                        .unwrap_or(NodeReachOwner::TraitImplMember {
                            trait_type: NodeReachParent::Unresolved,
                            self_type: NodeReachParent::Unresolved,
                        })
                }
                (HldNodeInputCategory::LocalItem, _, Some(HldOwnerKindValue::Trait)) => {
                    NodeReachOwner::TraitMember(container())
                }
                (HldNodeInputCategory::LocalItem, ..) => NodeReachOwner::Container(container()),
                _ => return None,
            };
            let record = NodeReachInputRecord {
                node_declared_visibility_kind: node
                    .node_effective_visibility_kind
                    .unwrap_or(HldEffectiveVisibilityKind::Unknown),
                node_reach_owner_value: owner,
                node_reexport_possible_flag: reexport,
            };
            Some((key.clone(), record))
        })
        .collect::<BTreeMap<_, _>>();
    let fields = derive_node_reach_fields(&records);
    Ok(input
        .input_node_records
        .iter()
        .cloned()
        .map(|node| match fields.get(&node.node_target_reference) {
            Some(found) => HldNodeInputRecord {
                node_effective_visibility_kind: Some(found.node_effective_visibility_kind),
                node_reachable_outside_crate_flag: found.node_reachable_outside_crate_flag,
                ..node
            },
            None => node,
        })
        .collect())
}

/// Each member of a stored impl header with what its header implements and for which type.
fn read_impl_member_owners(
    store: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
    cancel: &RunCancelFlagValue,
) -> Result<BTreeMap<HldTargetIdentityValue, NodeReachOwner>, NodeVisibilityFieldError> {
    read_impl_stage_values(store, source_scope, "impl-header-index-table")?
        .iter()
        .map(|header| {
            if cancel.check_stage_work_cancelled() {
                return Err(NodeVisibilityFieldError::Cancelled);
            }
            let self_type =
                read_header_reach_parent(&header["impl_self_type_resolution"], endpoints)?;
            let trait_type = &header["impl_trait_type_resolution"];
            let owner = if trait_type.is_null() {
                NodeReachOwner::InherentMember { self_type }
            } else {
                NodeReachOwner::TraitImplMember {
                    trait_type: read_header_reach_parent(trait_type, endpoints)?,
                    self_type,
                }
            };
            header["impl_member_occurrence_rows"]
                .as_array()
                .ok_or(NodeVisibilityFieldError::InvalidHeader {
                    field: "impl_member_occurrence_rows",
                })?
                .iter()
                .map(|member| {
                    let occurrence = decode_occurrence_identity_value(
                        &member["member_semantic_occurrence_key"],
                    )?;
                    Ok((
                        HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(occurrence)),
                        owner.clone(),
                    ))
                })
                .collect::<Result<Vec<_>, NodeVisibilityFieldError>>()
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|members| members.into_iter().flatten().collect())
}

/// R3, R4: what the self type or trait of a header is for reachability: a node of the
/// graph, a builtin type or a proven name outside the read source, or not resolved.
fn read_header_reach_parent(
    resolution: &Value,
    endpoints: &BTreeSet<&HldTargetIdentityValue>,
) -> Result<NodeReachParent, NodeVisibilityFieldError> {
    if let Some(target) = read_header_node_target(resolution, endpoints)? {
        return Ok(match target {
            HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(_)) => {
                NodeReachParent::Outside
            }
            node => NodeReachParent::Node(node),
        });
    }
    let named = read_stored_reason_kind(&resolution["unresolved_reason_kind"])
        == HldUnresolvedReasonKind::ExternalSourceMissing
        && decode_external_name_record(&resolution["external_name_record"])
            .map_err(|error| ImplHeaderEdgeError::InvalidEvidence { field: error.0 })?
            .is_some();
    Ok(if named {
        NodeReachParent::Outside
    } else {
        NodeReachParent::Unresolved
    })
}
