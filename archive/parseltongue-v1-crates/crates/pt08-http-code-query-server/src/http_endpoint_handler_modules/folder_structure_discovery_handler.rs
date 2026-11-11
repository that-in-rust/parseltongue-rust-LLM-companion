//! Folder structure discovery endpoint handler
//!
//! # 4-Word Naming: folder_structure_discovery_handler
//!
//! Endpoint: GET /folder-structure-discovery-tree
//!
//! Returns the L1/L2 folder hierarchy with entity counts for scope filtering.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Serialize;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Folder structure item
///
/// # 4-Word Name: FolderStructureHierarchyItem
#[derive(Debug, Serialize, Clone)]
pub struct FolderStructureHierarchyItem {
    pub l1: String,
    pub l2_children: Vec<String>,
    pub entity_count: usize,
}

/// Folder structure data payload
///
/// # 4-Word Name: FolderStructureDataPayload
#[derive(Debug, Serialize)]
pub struct FolderStructureDataPayload {
    pub folders: Vec<FolderStructureHierarchyItem>,
}

/// Folder structure response payload
///
/// # 4-Word Name: FolderStructureResponsePayload
#[derive(Debug, Serialize)]
pub struct FolderStructureResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: FolderStructureDataPayload,
    pub tokens: usize,
}

/// Error response payload structure
///
/// # 4-Word Name: ErrorResponsePayloadStructure
#[derive(Debug, Serialize)]
pub struct ErrorResponsePayloadStructure {
    pub success: bool,
    pub endpoint: String,
    pub error: String,
}

/// Handle folder structure discovery tree request
///
/// # 4-Word Name: handle_folder_structure_discovery_tree
///
/// # Contract
/// - Precondition: Database connected with root_subfolder_L1 and root_subfolder_L2 columns
/// - Postcondition: Returns hierarchical folder structure with entity counts
/// - Performance: <100ms for typical codebases
/// - Error Handling: Returns error JSON if database unavailable
pub async fn handle_folder_structure_discovery_tree(
    State(state): State<SharedApplicationStateContainer>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Clone Arc inside RwLock scope, release lock
    let storage = {
        let db_guard = state.database_storage_connection_arc.read().await;
        match db_guard.as_ref() {
            Some(s) => s.clone(),
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponsePayloadStructure {
                        success: false,
                        endpoint: "/folder-structure-discovery-tree".to_string(),
                        error: "Database not connected".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    }; // Lock released here

    // Query folder structure from database
    let folders = match query_folder_structure_from_database(&storage).await {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponsePayloadStructure {
                    success: false,
                    endpoint: "/folder-structure-discovery-tree".to_string(),
                    error: format!("Failed to query folder structure: {}", e),
                }),
            )
                .into_response()
        }
    };

    // Estimate tokens
    let tokens = 100 + (folders.len() * 30);

    let data = FolderStructureDataPayload { folders };

    (
        StatusCode::OK,
        Json(FolderStructureResponsePayload {
            success: true,
            endpoint: "/folder-structure-discovery-tree".to_string(),
            data,
            tokens,
        }),
    )
        .into_response()
}

/// Query folder structure from database
///
/// # 4-Word Name: query_folder_structure_from_database
///
/// Queries CodeGraph for all L1/L2 combinations and groups them hierarchically.
async fn query_folder_structure_from_database(
    storage: &Arc<parseltongue_core::storage::CozoDbStorage>,
) -> Result<Vec<FolderStructureHierarchyItem>, String> {
    // Query: Get all L1, L2, and entity combinations
    let query = "?[l1, l2, entity] := *CodeGraph{ISGL1_key: entity, root_subfolder_L1: l1, root_subfolder_L2: l2}";

    let result = storage
        .raw_query(query)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

    // Build hierarchical structure
    // Map: L1 -> (Set<L2>, entity_count)
    let mut l1_map: HashMap<String, (HashSet<String>, usize)> = HashMap::new();

    for row in &result.rows {
        if row.len() >= 3 {
            let l1 = extract_string_from_datavalue(&row[0]);
            let l2 = extract_string_from_datavalue(&row[1]);

            let entry = l1_map.entry(l1.clone()).or_insert((HashSet::new(), 0));
            entry.0.insert(l2); // Add L2 to set
            entry.1 += 1; // Increment entity count
        }
    }

    // Convert to sorted list
    let mut folders: Vec<FolderStructureHierarchyItem> = l1_map
        .into_iter()
        .map(|(l1, (l2_set, count))| {
            let mut l2_children: Vec<String> = l2_set.into_iter().collect();
            l2_children.sort(); // Sort L2 children alphabetically

            FolderStructureHierarchyItem {
                l1: l1.clone(),
                l2_children,
                entity_count: count,
            }
        })
        .collect();

    // Sort by L1 alphabetically
    folders.sort_by(|a, b| a.l1.cmp(&b.l1));

    Ok(folders)
}

/// Extract string from CozoDB DataValue
///
/// # 4-Word Name: extract_string_from_datavalue
fn extract_string_from_datavalue(value: &cozo::DataValue) -> String {
    match value {
        cozo::DataValue::Str(s) => s.to_string(),
        _ => format!("{:?}", value),
    }
}
