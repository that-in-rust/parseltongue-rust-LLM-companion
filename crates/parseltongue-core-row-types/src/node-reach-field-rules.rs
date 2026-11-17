//! REQ-RUST-069.0 V2-V7, R1-R5: the effective visibility of a node and whether a public
//! path from the crate root can name it. Pure: no store, no source text.

use crate::HldEffectiveVisibilityKind;
use std::collections::BTreeMap;

#[cfg(test)]
#[path = "node-reach-field-tests.rs"]
mod node_reach_field_tests;

/// What a node's reachability depends on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NodeReachParentValue<K> {
    /// The crate root.
    CrateRoot,
    /// A name outside the read source or a builtin type: it counts as reachable (R3).
    Outside,
    /// Not resolved to a node (R4).
    Unresolved,
    Node(K),
}

/// How a node is owned, which decides the parents of R2 and the visibility of V2 to V5.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NodeReachOwnerValue<K> {
    /// A file area, with the module that contains it. It has no fields of its own; an item
    /// in it is reachable when that module is.
    FileArea {
        module: NodeReachParentValue<K>,
    },
    MacroCall,
    /// An item in a module: the parent is its file area or its inline module.
    Container(NodeReachParentValue<K>),
    TraitMember(NodeReachParentValue<K>),
    /// REQ-RUST-071.0 F1, F2: an enum variant, or a field of one. It takes the visibility
    /// and the reachability of its enum or variant.
    VariantMember(NodeReachParentValue<K>),
    InherentMember {
        self_type: NodeReachParentValue<K>,
    },
    TraitImplMember {
        trait_type: NodeReachParentValue<K>,
        self_type: NodeReachParentValue<K>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeReachInputRecord<K> {
    /// V1, V7: what the item's own visibility text says.
    pub node_declared_visibility_kind: HldEffectiveVisibilityKind,
    pub node_reach_owner_value: NodeReachOwnerValue<K>,
    /// R5: the node's unit has a `use` that is not private.
    pub node_reexport_possible_flag: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeReachFieldRecord {
    pub node_effective_visibility_kind: HldEffectiveVisibilityKind,
    pub node_reachable_outside_crate_flag: Option<bool>,
}

/// The two fields of every node of `nodes` but its file areas, which store neither (V6).
pub fn derive_node_reach_fields<K: Ord + Clone>(
    nodes: &BTreeMap<K, NodeReachInputRecord<K>>,
) -> BTreeMap<K, NodeReachFieldRecord> {
    nodes
        .iter()
        .filter(|(_, record)| {
            !matches!(
                record.node_reach_owner_value,
                NodeReachOwnerValue::FileArea { .. }
            )
        })
        .map(|(key, _)| {
            let fields = NodeReachFieldRecord {
                node_effective_visibility_kind: derive_effective_visibility_kind(nodes, key, &[]),
                node_reachable_outside_crate_flag: derive_node_reach_flag(nodes, key, &[]),
            };
            (key.clone(), fields)
        })
        .collect()
}

/// V2 to V7. `path` holds the nodes already being decided, so a parent cycle ends.
fn derive_effective_visibility_kind<K: Ord>(
    nodes: &BTreeMap<K, NodeReachInputRecord<K>>,
    key: &K,
    path: &[&K],
) -> HldEffectiveVisibilityKind {
    use HldEffectiveVisibilityKind::{NotApplicable, Public, Unknown};
    let Some(record) = nodes.get(key).filter(|_| !path.contains(&key)) else {
        return Unknown;
    };
    match &record.node_reach_owner_value {
        // V6: a file area declares nothing.
        NodeReachOwnerValue::FileArea { .. } => Unknown,
        NodeReachOwnerValue::MacroCall => NotApplicable,
        NodeReachOwnerValue::TraitMember(trait_type)
        | NodeReachOwnerValue::VariantMember(trait_type)
        | NodeReachOwnerValue::TraitImplMember { trait_type, .. } => match trait_type {
            NodeReachParentValue::Node(trait_key) => {
                derive_effective_visibility_kind(nodes, trait_key, &[path, &[key]].concat())
            }
            NodeReachParentValue::Outside | NodeReachParentValue::CrateRoot => Public,
            NodeReachParentValue::Unresolved => Unknown,
        },
        NodeReachOwnerValue::Container(_) | NodeReachOwnerValue::InherentMember { .. } => {
            record.node_declared_visibility_kind
        }
    }
}

/// R1, R2, R5: false unless public; then what the parents allow.
fn derive_node_reach_flag<K: Ord>(
    nodes: &BTreeMap<K, NodeReachInputRecord<K>>,
    key: &K,
    path: &[&K],
) -> Option<bool> {
    let record = nodes.get(key).filter(|_| !path.contains(&key))?;
    let parents = match &record.node_reach_owner_value {
        NodeReachOwnerValue::FileArea { .. } => return None,
        NodeReachOwnerValue::MacroCall => return Some(false),
        NodeReachOwnerValue::Container(parent)
        | NodeReachOwnerValue::TraitMember(parent)
        | NodeReachOwnerValue::VariantMember(parent) => vec![parent],
        NodeReachOwnerValue::InherentMember { self_type } => vec![self_type],
        NodeReachOwnerValue::TraitImplMember {
            trait_type,
            self_type,
        } => vec![trait_type, self_type],
    };
    match derive_effective_visibility_kind(nodes, key, path) {
        HldEffectiveVisibilityKind::Unknown => None,
        HldEffectiveVisibilityKind::Public => {
            let inner = [path, &[key]].concat();
            let flags = parents
                .into_iter()
                .map(|parent| derive_parent_reach_flag(nodes, parent, &inner))
                .collect::<Vec<_>>();
            if flags.contains(&Some(false)) {
                (!record.node_reexport_possible_flag).then_some(false)
            } else {
                flags.iter().all(Option::is_some).then_some(true)
            }
        }
        _ => Some(false),
    }
}

/// R3, R4: whether a path through `parent` is public. A file area stands for its module.
fn derive_parent_reach_flag<K: Ord>(
    nodes: &BTreeMap<K, NodeReachInputRecord<K>>,
    parent: &NodeReachParentValue<K>,
    path: &[&K],
) -> Option<bool> {
    match parent {
        NodeReachParentValue::CrateRoot | NodeReachParentValue::Outside => Some(true),
        NodeReachParentValue::Unresolved => None,
        NodeReachParentValue::Node(key) => match nodes.get(key)?.node_reach_owner_value {
            NodeReachOwnerValue::FileArea { ref module } if !path.contains(&key) => {
                derive_parent_reach_flag(nodes, module, &[path, &[key]].concat())
            }
            NodeReachOwnerValue::FileArea { .. } => None,
            _ => derive_node_reach_flag(nodes, key, path),
        },
    }
}
