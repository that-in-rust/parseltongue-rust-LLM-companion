//! Ingestion coverage folder report endpoint handler
//!
//! # 4-Word Naming: ingestion_coverage_folder_handler
//!
//! Endpoint: GET /ingestion-coverage-folder-report
//!
//! Walks the current directory to find eligible files (by extension), compares
//! against parsed files in the database, and generates a folder-by-folder
//! coverage report. Also writes ingestion errors to a log file.

use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use chrono::Utc;
use walkdir::WalkDir;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Supported file extensions for parsing
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "ts", "go", "java", "c", "h", "cpp", "hpp", "rb", "php", "cs", "swift"
];

/// Directories to exclude from walking
const EXCLUDED_DIRECTORIES: &[&str] = &[
    ".git", "target", "node_modules", "__pycache__", ".build", "vendor", ".idea", ".vscode"
];

/// Query parameters for coverage report
///
/// # 4-Word Name: IngestionCoverageFolderQueryParams
#[derive(Debug, Deserialize)]
pub struct IngestionCoverageFolderQueryParams {
    /// Folder depth for grouping (default: 2)
    #[serde(default = "default_depth_value")]
    pub depth: usize,
}

fn default_depth_value() -> usize {
    2
}

/// Per-folder coverage metrics data
///
/// # 4-Word Name: FolderCoverageMetricsData
#[derive(Debug, Serialize, Clone)]
pub struct FolderCoverageMetricsData {
    pub folder_path: String,
    pub depth: usize,
    pub total_files: usize,
    pub eligible_files: usize,
    pub parsed_files: usize,
    pub coverage_pct: f64,
}

/// Overall coverage report summary
///
/// # 4-Word Name: CoverageReportSummaryData
#[derive(Debug, Serialize)]
pub struct CoverageReportSummaryData {
    pub root_directory: String,
    pub total_files: usize,
    pub eligible_files: usize,
    pub parsed_files: usize,
    pub coverage_pct: f64,
    pub entity_count: usize,
    pub edge_count: usize,
    pub errors_file: String,
    pub error_count: usize,
    pub unparsed_files: Vec<String>,
}

/// Full coverage report data payload
///
/// # 4-Word Name: CoverageReportDataPayload
#[derive(Debug, Serialize)]
pub struct CoverageReportDataPayload {
    pub summary: CoverageReportSummaryData,
    pub folders: Vec<FolderCoverageMetricsData>,
}

/// Coverage report response payload
///
/// # 4-Word Name: CoverageReportResponsePayload
#[derive(Debug, Serialize)]
pub struct CoverageReportResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: CoverageReportDataPayload,
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

/// Handle ingestion coverage folder report request
///
/// # 4-Word Name: handle_ingestion_coverage_folder_report
///
/// # Contract
/// - Precondition: Database connected with parsed files
/// - Postcondition: Returns folder-by-folder coverage report + writes error log
/// - Performance: O(N) where N is number of files
/// - Error Handling: Returns error JSON if database unavailable
pub async fn handle_ingestion_coverage_folder_report(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<IngestionCoverageFolderQueryParams>,
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
                        endpoint: "/ingestion-coverage-folder-report".to_string(),
                        error: "Database not connected".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    }; // Lock released here

    // Get database file path for workspace directory derivation
    let database_file_path_string = {
        let stats = state.codebase_statistics_metadata_arc.read().await;
        stats.database_file_path_string.clone()
    };

    // Derive workspace directory from database path
    let workspace_dir = derive_workspace_directory_from_database(&database_file_path_string);

    // Walk directory and collect file paths
    let (all_files, eligible_files, walk_errors) = walk_directory_collect_files_and_errors(".");

    // Query parsed files from database
    let parsed_files = match query_parsed_file_paths_from_database(&storage).await {
        Ok(files) => files,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponsePayloadStructure {
                    success: false,
                    endpoint: "/ingestion-coverage-folder-report".to_string(),
                    error: format!("Failed to query parsed files: {}", e),
                }),
            )
                .into_response()
        }
    };

    // Query entity and edge counts
    let entity_count = match query_entity_count_from_database(&storage).await {
        Ok(count) => count,
        Err(_) => 0,
    };

    let edge_count = match query_edge_count_from_database(&storage).await {
        Ok(count) => count,
        Err(_) => 0,
    };

    // Compute unparsed files (eligible minus parsed)
    let unparsed_files = compute_unparsed_files_list(&eligible_files, &parsed_files);

    // Group files by folder at requested depth
    let folder_metrics = group_files_by_folder_depth(&eligible_files, &parsed_files, params.depth);

    // Compute overall coverage
    let total_eligible = eligible_files.len();
    let total_parsed = parsed_files.len();
    let overall_coverage_pct = if total_eligible > 0 {
        (total_parsed as f64 / total_eligible as f64) * 100.0
    } else {
        0.0
    };

    // Write error log file
    let error_file_path = write_ingestion_errors_log_file(
        &workspace_dir,
        &database_file_path_string,
        &unparsed_files,
        &walk_errors,
    );

    let error_count = unparsed_files.len() + walk_errors.len();

    // Cap unparsed files list at 100 for response
    let unparsed_files_capped: Vec<String> = unparsed_files
        .iter()
        .take(100)
        .cloned()
        .collect();

    // Compute token estimate before moving values
    let tokens = 100 + (folder_metrics.len() * 40) + (unparsed_files.len() * 10);

    // Build response
    let summary = CoverageReportSummaryData {
        root_directory: ".".to_string(),
        total_files: all_files.len(),
        eligible_files: total_eligible,
        parsed_files: total_parsed,
        coverage_pct: overall_coverage_pct,
        entity_count,
        edge_count,
        errors_file: error_file_path,
        error_count,
        unparsed_files: unparsed_files_capped,
    };

    let data = CoverageReportDataPayload {
        summary,
        folders: folder_metrics,
    };

    (
        StatusCode::OK,
        Json(CoverageReportResponsePayload {
            success: true,
            endpoint: "/ingestion-coverage-folder-report".to_string(),
            data,
            tokens,
        }),
    )
        .into_response()
}

/// Derive workspace directory from database path
///
/// # 4-Word Name: derive_workspace_directory_from_database
fn derive_workspace_directory_from_database(db_path: &str) -> PathBuf {
    // Strip engine prefix (rocksdb: or sqlite:) if present
    let path_str = db_path.strip_prefix("rocksdb:")
        .or_else(|| db_path.strip_prefix("sqlite:"))
        .unwrap_or(db_path);

    // Parse as path and get parent directory of analysis.db
    let path = Path::new(path_str);

    // If path ends with "analysis.db", get parent; otherwise use as-is
    if path.file_name().and_then(|n| n.to_str()) == Some("analysis.db") {
        path.parent().unwrap_or(Path::new(".")).to_path_buf()
    } else {
        path.parent().unwrap_or(Path::new(".")).to_path_buf()
    }
}

/// Walk directory and collect files and errors
///
/// # 4-Word Name: walk_directory_collect_files_and_errors
///
/// Returns: (all_files, eligible_files, walk_errors)
fn walk_directory_collect_files_and_errors(root: &str) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<String>) {
    let mut all_files = Vec::new();
    let mut eligible_files = Vec::new();
    let mut walk_errors = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_str().unwrap_or("");
                !EXCLUDED_DIRECTORIES.contains(&name)
            } else {
                true
            }
        })
    {
        match entry {
            Ok(e) => {
                if e.file_type().is_file() {
                    let path = e.path().to_path_buf();
                    all_files.push(path.clone());

                    // Check if eligible by extension
                    if is_file_eligible_for_parsing(&path) {
                        eligible_files.push(path);
                    }
                }
            }
            Err(e) => {
                walk_errors.push(format!("{}", e));
            }
        }
    }

    (all_files, eligible_files, walk_errors)
}

/// Check if file is eligible for parsing by extension
///
/// # 4-Word Name: is_file_eligible_for_parsing
fn is_file_eligible_for_parsing(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            return SUPPORTED_EXTENSIONS.contains(&ext_str);
        }
    }
    false
}

/// Query parsed file paths from database
///
/// # 4-Word Name: query_parsed_file_paths_from_database
async fn query_parsed_file_paths_from_database(
    storage: &Arc<parseltongue_core::storage::CozoDbStorage>,
) -> Result<HashSet<PathBuf>, String> {
    let query = "?[file_path] := *CodeGraph{file_path}";
    let result = storage
        .raw_query(query)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

    let mut parsed_files = HashSet::new();
    for row in &result.rows {
        if !row.is_empty() {
            let file_path_str = extract_string_from_datavalue(&row[0]);

            // Filter out unresolved/external paths
            if !file_path_str.starts_with("unresolved") && !file_path_str.starts_with("external") {
                parsed_files.insert(PathBuf::from(file_path_str));
            }
        }
    }

    Ok(parsed_files)
}

/// Query entity count from database
///
/// # 4-Word Name: query_entity_count_from_database
async fn query_entity_count_from_database(
    storage: &Arc<parseltongue_core::storage::CozoDbStorage>,
) -> Result<usize, String> {
    let query = "?[count(ISGL1_key)] := *CodeGraph{ISGL1_key}";
    let result = storage
        .raw_query(query)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

    if let Some(row) = result.rows.first() {
        if !row.is_empty() {
            return Ok(extract_usize_from_datavalue(&row[0]));
        }
    }

    Ok(0)
}

/// Query edge count from database
///
/// # 4-Word Name: query_edge_count_from_database
async fn query_edge_count_from_database(
    storage: &Arc<parseltongue_core::storage::CozoDbStorage>,
) -> Result<usize, String> {
    let query = "?[count(from_key)] := *DependencyEdges{from_key}";
    let result = storage
        .raw_query(query)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

    if let Some(row) = result.rows.first() {
        if !row.is_empty() {
            return Ok(extract_usize_from_datavalue(&row[0]));
        }
    }

    Ok(0)
}

/// Compute list of unparsed files
///
/// # 4-Word Name: compute_unparsed_files_list
fn compute_unparsed_files_list(
    eligible_files: &[PathBuf],
    parsed_files: &HashSet<PathBuf>,
) -> Vec<String> {
    eligible_files
        .iter()
        .filter(|f| !parsed_files.contains(*f))
        .map(|f| f.display().to_string())
        .collect()
}

/// Group files by folder depth and compute coverage
///
/// # 4-Word Name: group_files_by_folder_depth
fn group_files_by_folder_depth(
    eligible_files: &[PathBuf],
    parsed_files: &HashSet<PathBuf>,
    depth: usize,
) -> Vec<FolderCoverageMetricsData> {
    // Map folder_path -> (eligible_count, parsed_count)
    let mut folder_map: HashMap<String, (usize, usize)> = HashMap::new();

    for file in eligible_files {
        let folder = extract_folder_at_depth(file, depth);
        let entry = folder_map.entry(folder).or_insert((0, 0));
        entry.0 += 1; // eligible

        if parsed_files.contains(file) {
            entry.1 += 1; // parsed
        }
    }

    // Convert to metrics data
    let mut metrics: Vec<FolderCoverageMetricsData> = folder_map
        .into_iter()
        .map(|(folder_path, (eligible, parsed))| {
            let coverage_pct = if eligible > 0 {
                (parsed as f64 / eligible as f64) * 100.0
            } else {
                0.0
            };

            FolderCoverageMetricsData {
                folder_path,
                depth,
                total_files: eligible,
                eligible_files: eligible,
                parsed_files: parsed,
                coverage_pct,
            }
        })
        .collect();

    // Sort by coverage_pct ascending (lowest coverage first - most interesting)
    metrics.sort_by(|a, b| {
        a.coverage_pct
            .partial_cmp(&b.coverage_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    metrics
}

/// Extract folder at given depth from file path
///
/// # 4-Word Name: extract_folder_at_depth
fn extract_folder_at_depth(file: &Path, depth: usize) -> String {
    if depth == 0 {
        return ".".to_string();
    }

    let components: Vec<_> = file
        .parent()
        .unwrap_or(Path::new("."))
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .take(depth)
        .collect();

    if components.is_empty() {
        ".".to_string()
    } else {
        components.join("/") + "/"
    }
}

/// Write ingestion errors log file
///
/// # 4-Word Name: write_ingestion_errors_log_file
fn write_ingestion_errors_log_file(
    workspace_dir: &Path,
    db_path: &str,
    unparsed_files: &[String],
    walk_errors: &[String],
) -> String {
    let error_file_path = workspace_dir.join("ingestion-errors.txt");

    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S");
    let total_errors = unparsed_files.len() + walk_errors.len();

    let mut content = String::new();
    content.push_str("# Parseltongue Ingestion Errors\n");
    content.push_str(&format!("# Generated: {}\n", timestamp));
    content.push_str(&format!("# Database: {}\n", db_path));
    content.push_str(&format!("# Total errors: {}\n\n", total_errors));

    for file in unparsed_files {
        content.push_str(&format!("[UNPARSED] {}\n", file));
    }

    for err in walk_errors {
        content.push_str(&format!("[WALK_ERROR] {}\n", err));
    }

    // Write to file (ignore errors silently)
    let _ = std::fs::write(&error_file_path, content);

    error_file_path.display().to_string()
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

/// Extract usize from CozoDB DataValue
///
/// # 4-Word Name: extract_usize_from_datavalue
fn extract_usize_from_datavalue(value: &cozo::DataValue) -> usize {
    match value {
        cozo::DataValue::Num(n) => match n {
            cozo::Num::Int(i) => *i as usize,
            cozo::Num::Float(f) => *f as usize,
        },
        _ => 0,
    }
}
