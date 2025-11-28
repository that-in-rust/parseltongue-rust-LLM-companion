//! Server health check endpoint handler
//!
//! # 4-Word Naming: server_health_check_handler
//!
//! Endpoint: GET /server-health-check-status

use axum::{
    extract::State,
    Json,
};
use serde::Serialize;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Health check response
///
/// # 4-Word Name: HealthCheckResponsePayload
#[derive(Debug, Serialize)]
pub struct HealthCheckResponsePayload {
    pub success: bool,
    pub status: String,
    pub server_uptime_seconds_count: i64,
    pub endpoint: String,
}

/// Handle server health check status request
///
/// # 4-Word Name: handle_server_health_check_status
///
/// # Contract
/// - Precondition: Server is running
/// - Postcondition: Returns 200 with status "ok"
/// - Performance: <10ms
pub async fn handle_server_health_check_status(
    State(state): State<SharedApplicationStateContainer>,
) -> Json<HealthCheckResponsePayload> {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Calculate uptime
    let uptime = chrono::Utc::now()
        .signed_duration_since(state.server_start_timestamp_utc)
        .num_seconds();

    Json(HealthCheckResponsePayload {
        success: true,
        status: "ok".to_string(),
        server_uptime_seconds_count: uptime,
        endpoint: "/server-health-check-status".to_string(),
    })
}
