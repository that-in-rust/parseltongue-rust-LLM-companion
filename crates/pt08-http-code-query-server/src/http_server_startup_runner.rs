//! HTTP server startup and state management
//!
//! # 4-Word Naming: http_server_startup_runner

use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::command_line_argument_parser::{HttpServerStartupConfig, find_available_port_number};
use crate::route_definition_builder_module::build_complete_router_instance;
use parseltongue_core::storage::CozoDbStorage;

/// Shared application state container
///
/// # 4-Word Name: SharedApplicationStateContainer
#[derive(Clone)]
pub struct SharedApplicationStateContainer {
    /// Database storage connection (optional CozoDbStorage)
    pub database_storage_connection_arc: Arc<RwLock<Option<Arc<CozoDbStorage>>>>,

    /// Server start timestamp
    pub server_start_timestamp_utc: DateTime<Utc>,

    /// Last request timestamp for idle timeout
    pub last_request_timestamp_arc: Arc<RwLock<DateTime<Utc>>>,

    /// Codebase statistics metadata
    pub codebase_statistics_metadata_arc: Arc<RwLock<CodebaseStatisticsMetadata>>,
}

/// Codebase statistics metadata
///
/// # 4-Word Name: CodebaseStatisticsMetadata
#[derive(Debug, Clone, Default)]
pub struct CodebaseStatisticsMetadata {
    /// Total CODE entities count
    pub total_code_entities_count: usize,

    /// Total TEST entities count
    pub total_test_entities_count: usize,

    /// Total dependency edges count
    pub total_dependency_edges_count: usize,

    /// Languages detected in codebase
    pub languages_detected_list_vec: Vec<String>,

    /// Database file path
    pub database_file_path_string: String,

    /// Ingestion timestamp
    pub ingestion_timestamp_utc_option: Option<DateTime<Utc>>,
}

impl SharedApplicationStateContainer {
    /// Create new application state
    ///
    /// # 4-Word Name: create_new_application_state
    pub fn create_new_application_state() -> Self {
        let now = Utc::now();
        Self {
            database_storage_connection_arc: Arc::new(RwLock::new(None)),
            server_start_timestamp_utc: now,
            last_request_timestamp_arc: Arc::new(RwLock::new(now)),
            codebase_statistics_metadata_arc: Arc::new(RwLock::new(CodebaseStatisticsMetadata::default())),
        }
    }

    /// Create state with database storage
    ///
    /// # 4-Word Name: create_with_database_storage
    pub fn create_with_database_storage(storage: CozoDbStorage) -> Self {
        let now = Utc::now();
        Self {
            database_storage_connection_arc: Arc::new(RwLock::new(Some(Arc::new(storage)))),
            server_start_timestamp_utc: now,
            last_request_timestamp_arc: Arc::new(RwLock::new(now)),
            codebase_statistics_metadata_arc: Arc::new(RwLock::new(CodebaseStatisticsMetadata::default())),
        }
    }

    /// Update last request timestamp
    ///
    /// # 4-Word Name: update_last_request_timestamp
    pub async fn update_last_request_timestamp(&self) {
        let mut timestamp = self.last_request_timestamp_arc.write().await;
        *timestamp = Utc::now();
    }

    /// Query entity counts from database
    ///
    /// # 4-Word Name: query_entity_counts_from_database
    pub async fn query_entity_counts_from_database(&self) -> (usize, usize, usize) {
        let db_guard = self.database_storage_connection_arc.read().await;
        if let Some(storage) = db_guard.as_ref() {
            // Query CODE entities count
            let code_count = storage.raw_query(
                "?[count(ISGL1_key)] := *CodeGraph{ISGL1_key, entity_class}, entity_class == 'CODE'"
            ).await.ok()
            .and_then(|r| r.rows.first().cloned())
            .and_then(|row| row.first().cloned())
            .and_then(|v| match v {
                cozo::DataValue::Num(n) => match n {
                    cozo::Num::Int(i) => Some(i as usize),
                    cozo::Num::Float(f) => Some(f as usize),
                },
                _ => None,
            })
            .unwrap_or(0);

            // Query TEST entities count
            let test_count = storage.raw_query(
                "?[count(ISGL1_key)] := *CodeGraph{ISGL1_key, entity_class}, entity_class == 'TEST'"
            ).await.ok()
            .and_then(|r| r.rows.first().cloned())
            .and_then(|row| row.first().cloned())
            .and_then(|v| match v {
                cozo::DataValue::Num(n) => match n {
                    cozo::Num::Int(i) => Some(i as usize),
                    cozo::Num::Float(f) => Some(f as usize),
                },
                _ => None,
            })
            .unwrap_or(0);

            // Query edges count
            let edges_count = storage.raw_query(
                "?[count(from_key)] := *DependencyEdges{from_key}"
            ).await.ok()
            .and_then(|r| r.rows.first().cloned())
            .and_then(|row| row.first().cloned())
            .and_then(|v| match v {
                cozo::DataValue::Num(n) => match n {
                    cozo::Num::Int(i) => Some(i as usize),
                    cozo::Num::Float(f) => Some(f as usize),
                },
                _ => None,
            })
            .unwrap_or(0);

            (code_count, test_count, edges_count)
        } else {
            // No database connected, return metadata values
            let stats = self.codebase_statistics_metadata_arc.read().await;
            (stats.total_code_entities_count, stats.total_test_entities_count, stats.total_dependency_edges_count)
        }
    }
}

/// Start the HTTP server in blocking loop
///
/// # 4-Word Name: start_http_server_blocking_loop
pub async fn start_http_server_blocking_loop(config: HttpServerStartupConfig) -> Result<()> {
    // Determine port
    let port = config.http_port_override_option
        .unwrap_or_else(|| find_available_port_number(3333).unwrap_or(3333));

    // Connect to database if path provided
    let db_path = &config.database_connection_string_value;
    let state = if !db_path.is_empty() && db_path != "mem" {
        println!("Connecting to database: {}", db_path);
        match CozoDbStorage::new(db_path).await {
            Ok(storage) => {
                println!("✓ Database connected successfully");
                SharedApplicationStateContainer::create_with_database_storage(storage)
            }
            Err(e) => {
                println!("⚠ Warning: Could not connect to database: {}", e);
                println!("  Starting with empty state");
                SharedApplicationStateContainer::create_new_application_state()
            }
        }
    } else {
        SharedApplicationStateContainer::create_new_application_state()
    };

    // Update database path in stats
    {
        let mut stats = state.codebase_statistics_metadata_arc.write().await;
        stats.database_file_path_string = config.database_connection_string_value.clone();
    }

    // Build router
    let router = build_complete_router_instance(state);

    // Print startup message
    println!("Parseltongue HTTP Server");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("HTTP Server running at: http://localhost:{}", port);
    println!();
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│  Add to your LLM agent: PARSELTONGUE_URL=http://localhost:{}  │", port);
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();
    println!("Quick test:");
    println!("  curl http://localhost:{}/server-health-check-status", port);
    println!();

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_new_application_state() {
        let state = SharedApplicationStateContainer::create_new_application_state();
        assert!(state.database_storage_connection_arc.read().await.is_none());
    }

    #[tokio::test]
    async fn test_update_last_request_timestamp() {
        let state = SharedApplicationStateContainer::create_new_application_state();
        let before = *state.last_request_timestamp_arc.read().await;

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        state.update_last_request_timestamp().await;

        let after = *state.last_request_timestamp_arc.read().await;
        assert!(after > before);
    }
}
