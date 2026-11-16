//! REQ-RUST-053.0: named external targets, shared by every edge producer.
use parseltongue_core_row_types::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The external name one stored lookup proved: the node identity it gives, and what the
/// edge row records about how its site reached the name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExternalNameProofRecord {
    pub external_target_identity_value: HldTargetIdentityValue,
    pub external_proof_metadata_value: Value,
}

/// The field of a stored external name record that could not be read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExternalNameFieldError(pub &'static str);

/// The ways a site reaches an external name through a stored lookup.
const STORED_SITE_PROOF_KINDS: [&str; 3] = [
    "path_root_names_crate",
    "use_import_names_path",
    "implicit_prelude_name",
];

/// REQ-RUST-053.0 N4: what an edge row to a named external target records: the kind of
/// name, how this site reached it, the `use` item it went through, that the item's source
/// was not read, and that the path is the written one, not the item's defining path.
pub(crate) fn build_external_proof_value(
    name_kind: &str,
    site_proof_kind: &str,
    import_site_key: Value,
) -> Value {
    json!({
        "external_name_kind": name_kind,
        "external_site_proof_kind": site_proof_kind,
        "external_import_site_key": import_site_key,
        "external_source_read_flag": false,
        "external_definition_path_claimed_flag": false,
    })
}

/// REQ-RUST-053.0 N3, N4: reads the `*_external_name_record` field of a stored lookup
/// outcome. `None` when the lookup proved no external name. A builtin member is never
/// stored by name lookup, so that kind is refused here.
pub(crate) fn decode_external_name_record(
    record: &Value,
) -> Result<Option<ExternalNameProofRecord>, ExternalNameFieldError> {
    if record.is_null() {
        return Ok(None);
    }
    let name_kind = match record["external_name_kind"].as_str() {
        Some("crate_rooted_path") => HldExternalNameKind::CrateRootedPath,
        Some("prelude_name") => HldExternalNameKind::PreludeName,
        _ => return Err(ExternalNameFieldError("external_name_kind")),
    };
    let crate_name = match &record["external_crate_name"] {
        Value::Null => None,
        Value::String(name) => Some(name.clone()),
        _ => return Err(ExternalNameFieldError("external_crate_name")),
    };
    let path = record["external_item_path_text"]
        .as_str()
        .ok_or(ExternalNameFieldError("external_item_path_text"))?;
    let site_proof_kind = record["external_site_proof_kind"]
        .as_str()
        .filter(|kind| STORED_SITE_PROOF_KINDS.contains(kind))
        .ok_or(ExternalNameFieldError("external_site_proof_kind"))?;
    Ok(Some(ExternalNameProofRecord {
        external_target_identity_value: HldTargetIdentityValue::ExternalNamed(
            HldExternalNamedKey {
                external_name_kind_value: name_kind,
                external_crate_name_value: crate_name,
                external_item_path_text: path.to_owned(),
            },
        ),
        external_proof_metadata_value: build_external_proof_value(
            record["external_name_kind"].as_str().unwrap_or_default(),
            site_proof_kind,
            record["external_import_site_key"].clone(),
        ),
    }))
}

/// REQ-RUST-053.0 N1: one external node per named external target the edges end at, in
/// identity order. The node claims no file, kind, owner, visibility or signature.
pub fn collect_external_endpoint_nodes(edges: &[HldEdgeInputRecord]) -> Vec<HldNodeInputRecord> {
    edges
        .iter()
        .filter_map(|edge| match &edge.edge_resolution_value {
            HldEdgeResolutionValue::Extracted {
                target: target @ HldTargetIdentityValue::ExternalNamed(key),
            } => Some((target, (edge.run_generation_digest_value, key))),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(target, (generation, key))| HldNodeInputRecord {
            run_generation_digest_value: generation,
            node_target_reference: target.clone(),
            node_input_category: HldNodeInputCategory::ExternalDefinition,
            node_item_kind_value: None,
            node_owner_kind_value: None,
            node_function_body_ancestry_flag: false,
            node_effective_visibility_kind: None,
            node_reachable_outside_crate_flag: None,
            node_source_relative_path: None,
            node_qualified_path_text: key.external_item_path_text.clone(),
            node_signature_text: None,
            node_generating_call_reference: None,
        })
        .collect()
}
