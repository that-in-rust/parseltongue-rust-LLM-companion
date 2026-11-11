//! Ingestion diagnostics coverage report endpoint handler
//!
//! # 4-Word Naming: ingestion_diagnostics_coverage_handler
//!
//! Endpoint: GET /ingestion-diagnostics-coverage-report
//!
//! Returns comprehensive diagnostics about test entity exclusion and word coverage
//! for all files in the codebase.

use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Query parameters for diagnostics report
///
/// # 4-Word Name: DiagnosticsReportQueryParams
#[derive(Debug, Deserialize)]
pub struct DiagnosticsReportQueryParams {
    /// Filter by section: "test_entities", "word_coverage", "ignored_files", "summary"
    /// If not provided, returns all sections
    pub section: Option<String>,
}

/// Test entity exclusion item
///
/// # 4-Word Name: TestEntityExclusionItem
#[derive(Debug, Serialize)]
pub struct TestEntityExclusionItem {
    pub entity_name: String,
    pub folder_path: String,
    pub filename: String,
    pub entity_class: String,
    pub language: String,
    pub line_start: i64,
    pub line_end: i64,
    pub detection_reason: String,
}

/// Test entities excluded section
///
/// # 4-Word Name: TestEntitiesExcludedSection
#[derive(Debug, Serialize)]
pub struct TestEntitiesExcludedSection {
    pub total_count: usize,
    pub entities: Vec<TestEntityExclusionItem>,
}

/// File word coverage item
///
/// # 4-Word Name: FileWordCoverageItem
#[derive(Debug, Serialize)]
pub struct FileWordCoverageItem {
    pub folder_path: String,
    pub filename: String,
    pub language: String,
    pub source_word_count: i64,
    pub entity_word_count: i64,
    pub import_word_count: i64,
    pub comment_word_count: i64,
    pub raw_coverage_pct: f64,
    pub effective_coverage_pct: f64,
    pub entity_count: i64,
}

/// Word count coverage section
///
/// # 4-Word Name: WordCountCoverageSection
#[derive(Debug, Serialize)]
pub struct WordCountCoverageSection {
    pub avg_raw_coverage_pct: f64,
    pub avg_effective_coverage_pct: f64,
    pub total_files: usize,
    pub files: Vec<FileWordCoverageItem>,
}

/// Ignored file item (v1.6.5 Wave 1)
///
/// # 4-Word Name: IgnoredFileItem
#[derive(Debug, Serialize)]
pub struct IgnoredFileItem {
    pub folder_path: String,
    pub filename: String,
    pub extension: String,
    pub reason: String,
}

/// Ignored files section (v1.6.5 Wave 1)
///
/// # 4-Word Name: IgnoredFilesSection
#[derive(Debug, Serialize)]
pub struct IgnoredFilesSection {
    pub total_count: usize,
    pub by_extension: std::collections::HashMap<String, usize>,
    pub files: Vec<IgnoredFileItem>,
}

/// Summary-only section (v1.6.5 Wave 3)
///
/// # 4-Word Name: SummaryOnlySection
#[derive(Debug, Serialize)]
pub struct SummaryOnlySection {
    pub test_entities_count: usize,
    pub total_files_with_coverage: usize,
    pub avg_raw_coverage_pct: f64,
    pub avg_effective_coverage_pct: f64,
    pub ignored_files_count: usize,
}

/// Diagnostics coverage data payload
///
/// # 4-Word Name: DiagnosticsCoverageDataPayload
#[derive(Debug, Serialize)]
pub struct DiagnosticsCoverageDataPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_entities_excluded: Option<TestEntitiesExcludedSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_count_coverage: Option<WordCountCoverageSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_files: Option<IgnoredFilesSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<SummaryOnlySection>,
}

/// Diagnostics coverage response payload
///
/// # 4-Word Name: DiagnosticsCoverageResponsePayload
#[derive(Debug, Serialize)]
pub struct DiagnosticsCoverageResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: DiagnosticsCoverageDataPayload,
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

/// Handle ingestion diagnostics coverage report request
///
/// # 4-Word Name: handle_ingestion_diagnostics_coverage_report
///
/// # Contract
/// - Precondition: Database connected with TestEntitiesExcluded and FileWordCoverage relations
/// - Postcondition: Returns comprehensive diagnostics report
/// - Performance: <500ms for typical codebases
/// - Error Handling: Returns error JSON if database unavailable
pub async fn handle_ingestion_diagnostics_coverage_report(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<DiagnosticsReportQueryParams>,
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
                        endpoint: "/ingestion-diagnostics-coverage-report".to_string(),
                        error: "Database not connected".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    }; // Lock released here

    // Determine which sections to query
    let section_filter = params.section.as_deref().unwrap_or("all");
    let need_test_entities = section_filter == "all" || section_filter == "test_entities" || section_filter == "summary";
    let need_word_coverage = section_filter == "all" || section_filter == "word_coverage" || section_filter == "summary";
    let need_ignored_files = section_filter == "all" || section_filter == "ignored_files" || section_filter == "summary";
    let summary_only = section_filter == "summary";

    // Query test entities excluded (conditionally)
    let test_entities = if need_test_entities {
        match query_test_entities_excluded_from_database(&storage).await {
            Ok(entities) => entities,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponsePayloadStructure {
                        success: false,
                        endpoint: "/ingestion-diagnostics-coverage-report".to_string(),
                        error: format!("Failed to query test entities: {}", e),
                    }),
                )
                    .into_response()
            }
        }
    } else {
        Vec::new()
    };

    // Query word coverage (conditionally)
    let word_coverage = if need_word_coverage {
        match query_word_coverage_from_database(&storage).await {
            Ok(coverage) => coverage,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponsePayloadStructure {
                        success: false,
                        endpoint: "/ingestion-diagnostics-coverage-report".to_string(),
                        error: format!("Failed to query word coverage: {}", e),
                    }),
                )
                    .into_response()
            }
        }
    } else {
        Vec::new()
    };

    // Query ignored files (v1.6.5 Wave 1) (conditionally)
    let ignored_files = if need_ignored_files {
        match query_ignored_files_from_database(&storage).await {
            Ok(files) => files,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponsePayloadStructure {
                        success: false,
                        endpoint: "/ingestion-diagnostics-coverage-report".to_string(),
                        error: format!("Failed to query ignored files: {}", e),
                    }),
                )
                    .into_response()
            }
        }
    } else {
        Vec::new()
    };

    // Calculate counts and averages
    let test_count = test_entities.len();
    let total_files = word_coverage.len();
    let ignored_count = ignored_files.len();

    let (avg_raw, avg_effective) = if total_files > 0 {
        let sum_raw: f64 = word_coverage.iter().map(|f| f.raw_coverage_pct).sum();
        let sum_effective: f64 = word_coverage.iter().map(|f| f.effective_coverage_pct).sum();
        (sum_raw / total_files as f64, sum_effective / total_files as f64)
    } else {
        (0.0, 0.0)
    };

    // Build response based on section filter
    let data = if summary_only {
        // Summary only - no file-level detail
        DiagnosticsCoverageDataPayload {
            test_entities_excluded: None,
            word_count_coverage: None,
            ignored_files: None,
            summary: Some(SummaryOnlySection {
                test_entities_count: test_count,
                total_files_with_coverage: total_files,
                avg_raw_coverage_pct: avg_raw,
                avg_effective_coverage_pct: avg_effective,
                ignored_files_count: ignored_count,
            }),
        }
    } else {
        // Full or filtered sections
        let test_section = if section_filter == "all" || section_filter == "test_entities" {
            Some(TestEntitiesExcludedSection {
                total_count: test_count,
                entities: test_entities,
            })
        } else {
            None
        };

        let word_section = if section_filter == "all" || section_filter == "word_coverage" {
            Some(WordCountCoverageSection {
                avg_raw_coverage_pct: avg_raw,
                avg_effective_coverage_pct: avg_effective,
                total_files,
                files: word_coverage,
            })
        } else {
            None
        };

        let ignored_section = if section_filter == "all" || section_filter == "ignored_files" {
            let mut by_extension = std::collections::HashMap::new();
            for file in &ignored_files {
                *by_extension.entry(file.extension.clone()).or_insert(0) += 1;
            }
            Some(IgnoredFilesSection {
                total_count: ignored_count,
                by_extension,
                files: ignored_files,
            })
        } else {
            None
        };

        DiagnosticsCoverageDataPayload {
            test_entities_excluded: test_section,
            word_count_coverage: word_section,
            ignored_files: ignored_section,
            summary: None,
        }
    };

    // Estimate tokens based on what we're returning
    let tokens = if summary_only {
        100
    } else {
        100 + (test_count * 30) + (total_files * 50) + (ignored_count * 20)
    };

    (
        StatusCode::OK,
        Json(DiagnosticsCoverageResponsePayload {
            success: true,
            endpoint: "/ingestion-diagnostics-coverage-report".to_string(),
            data,
            tokens,
        }),
    )
        .into_response()
}

/// Query test entities excluded from database
///
/// # 4-Word Name: query_test_entities_excluded_from_database
async fn query_test_entities_excluded_from_database(
    storage: &Arc<parseltongue_core::storage::CozoDbStorage>,
) -> Result<Vec<TestEntityExclusionItem>, String> {
    let query = "?[entity_name, folder_path, filename, entity_class, language, line_start, line_end, detection_reason] := *TestEntitiesExcluded{entity_name, folder_path, filename, entity_class, language, line_start, line_end, detection_reason}";

    let result = storage
        .raw_query(query)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

    let mut entities = Vec::new();
    for row in &result.rows {
        if row.len() >= 8 {
            entities.push(TestEntityExclusionItem {
                entity_name: extract_string_from_datavalue(&row[0]),
                folder_path: extract_string_from_datavalue(&row[1]),
                filename: extract_string_from_datavalue(&row[2]),
                entity_class: extract_string_from_datavalue(&row[3]),
                language: extract_string_from_datavalue(&row[4]),
                line_start: extract_i64_from_datavalue(&row[5]),
                line_end: extract_i64_from_datavalue(&row[6]),
                detection_reason: extract_string_from_datavalue(&row[7]),
            });
        }
    }

    Ok(entities)
}

/// Query word coverage from database
///
/// # 4-Word Name: query_word_coverage_from_database
async fn query_word_coverage_from_database(
    storage: &Arc<parseltongue_core::storage::CozoDbStorage>,
) -> Result<Vec<FileWordCoverageItem>, String> {
    let query = "?[folder_path, filename, language, source_word_count, entity_word_count, import_word_count, comment_word_count, raw_coverage_pct, effective_coverage_pct, entity_count] := *FileWordCoverage{folder_path, filename, language, source_word_count, entity_word_count, import_word_count, comment_word_count, raw_coverage_pct, effective_coverage_pct, entity_count}";

    let result = storage
        .raw_query(query)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

    let mut files = Vec::new();
    for row in &result.rows {
        if row.len() >= 10 {
            files.push(FileWordCoverageItem {
                folder_path: extract_string_from_datavalue(&row[0]),
                filename: extract_string_from_datavalue(&row[1]),
                language: extract_string_from_datavalue(&row[2]),
                source_word_count: extract_i64_from_datavalue(&row[3]),
                entity_word_count: extract_i64_from_datavalue(&row[4]),
                import_word_count: extract_i64_from_datavalue(&row[5]),
                comment_word_count: extract_i64_from_datavalue(&row[6]),
                raw_coverage_pct: extract_f64_from_datavalue(&row[7]),
                effective_coverage_pct: extract_f64_from_datavalue(&row[8]),
                entity_count: extract_i64_from_datavalue(&row[9]),
            });
        }
    }

    Ok(files)
}

/// Query ignored files from database (v1.6.5 Wave 1)
///
/// # 4-Word Name: query_ignored_files_from_database
async fn query_ignored_files_from_database(
    storage: &Arc<parseltongue_core::storage::CozoDbStorage>,
) -> Result<Vec<IgnoredFileItem>, String> {
    let query = "?[folder_path, filename, extension, reason] := *IgnoredFiles{folder_path, filename, extension, reason}";

    let result = storage
        .raw_query(query)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

    let mut files = Vec::new();
    for row in &result.rows {
        if row.len() >= 4 {
            files.push(IgnoredFileItem {
                folder_path: extract_string_from_datavalue(&row[0]),
                filename: extract_string_from_datavalue(&row[1]),
                extension: extract_string_from_datavalue(&row[2]),
                reason: extract_string_from_datavalue(&row[3]),
            });
        }
    }

    Ok(files)
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

/// Extract i64 from CozoDB DataValue
///
/// # 4-Word Name: extract_i64_from_datavalue
fn extract_i64_from_datavalue(value: &cozo::DataValue) -> i64 {
    match value {
        cozo::DataValue::Num(n) => match n {
            cozo::Num::Int(i) => *i,
            cozo::Num::Float(f) => *f as i64,
        },
        _ => 0,
    }
}

/// Extract f64 from CozoDB DataValue
///
/// # 4-Word Name: extract_f64_from_datavalue
fn extract_f64_from_datavalue(value: &cozo::DataValue) -> f64 {
    match value {
        cozo::DataValue::Num(n) => match n {
            cozo::Num::Int(i) => *i as f64,
            cozo::Num::Float(f) => *f,
        },
        _ => 0.0,
    }
}
