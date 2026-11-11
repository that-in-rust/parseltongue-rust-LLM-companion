//! Query helpers for agent JSON graph traversal
//!
//! # Purpose
//! Enable LLM agents to query Parseltongue JSON exports ergonomically.
//!
//! # Performance (S06 Principle #5)
//! All queries: < 100ms for 1,500 entities (validated by tests)

use crate::query_json_graph_errors::JsonGraphQueryError;
use serde_json::Value;

/// Find entities that depend on target (reverse dependencies)
///
/// # 4-Word Name: find + reverse_dependencies + by + key
pub fn find_reverse_dependencies_by_key(
    json: &Value,
    target_key: &str,
) -> Result<Vec<String>, JsonGraphQueryError> {
    json["entities"]
        .as_array()
        .ok_or_else(|| JsonGraphQueryError::MalformedJson("entities not array".into()))?
        .iter()
        .find(|e| e["isgl1_key"].as_str() == Some(target_key))
        .ok_or_else(|| JsonGraphQueryError::EntityNotFound(target_key.into()))?
        ["reverse_deps"]
        .as_array()
        .ok_or_else(|| JsonGraphQueryError::MalformedJson("reverse_deps not array".into()))
        .map(|deps| deps.iter().filter_map(|v| v.as_str()).map(String::from).collect())
}

/// Build execution call chain from root function
///
/// # 4-Word Name: build + call_chain + from + root
pub fn build_call_chain_from_root(
    json: &Value,
    root_key: &str,
) -> Result<Vec<String>, JsonGraphQueryError> {
    let edges = json["edges"]
        .as_array()
        .ok_or_else(|| JsonGraphQueryError::MalformedJson("edges not array".into()))?;

    let mut chain = vec![root_key.to_string()];
    let mut current = root_key;

    while let Some(next_edge) = edges.iter().find(|e| {
        e["from_key"].as_str() == Some(current) && e["edge_type"].as_str() == Some("Calls")
    }) {
        let next = next_edge["to_key"]
            .as_str()
            .ok_or_else(|| JsonGraphQueryError::MalformedJson("to_key not string".into()))?;
        chain.push(next.to_string());
        current = next;
    }

    Ok(chain)
}

/// Filter edges by type only (Calls, Uses, Implements)
///
/// # 4-Word Name: filter + edges + by_type + only
pub fn filter_edges_by_type_only(
    json: &Value,
    edge_type: &str,
) -> Result<Vec<Value>, JsonGraphQueryError> {
    match edge_type {
        "Calls" | "Uses" | "Implements" => {},
        _ => return Err(JsonGraphQueryError::InvalidEdgeType(edge_type.into())),
    }

    json["edges"]
        .as_array()
        .ok_or_else(|| JsonGraphQueryError::MalformedJson("edges not array".into()))
        .map(|edges| {
            edges
                .iter()
                .filter(|e| e["edge_type"].as_str() == Some(edge_type))
                .cloned()
                .collect()
        })
}

/// Collect all entities in file path (substring match)
///
/// # 4-Word Name: collect + entities + in_file + path
pub fn collect_entities_in_file_path(
    json: &Value,
    file_path_pattern: &str,
) -> Result<Vec<Value>, JsonGraphQueryError> {
    json["entities"]
        .as_array()
        .ok_or_else(|| JsonGraphQueryError::MalformedJson("entities not array".into()))
        .map(|entities| {
            entities
                .iter()
                .filter(|e| {
                    e["file_path"]
                        .as_str()
                        .map(|p| p.contains(file_path_pattern))
                        .unwrap_or(false)
                })
                .cloned()
                .collect()
        })
}
