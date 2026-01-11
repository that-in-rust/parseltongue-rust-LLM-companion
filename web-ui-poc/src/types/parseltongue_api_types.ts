/**
 * API Types for Parseltongue HTTP Server
 *
 * These types match the ACTUAL response format from pt08-http-code-query-server
 * Based on analysis of the real Rust structs
 */

/**
 * Single entity in the list response
 * Matches EntitySummaryListItem from code_entities_list_all_handler.rs
 */
export interface EntitySummaryListItem {
  key: string;                    // ISGL1 entity key (e.g., "rust:fn:main:src_main_rs:1-50")
  file_path: string;              // File path where entity is defined
  entity_type: string;            // Type (Function, Struct, Method, etc.)
  entity_class: string;           // "CODE" or "TEST"
  language: string;               // Language (rust, javascript, etc.)
  lines_of_code: number | null;   // Lines of code (null if not available)
}

/**
 * Data payload for entities list
 * Matches EntitiesListDataPayload from code_entities_list_all_handler.rs
 */
export interface EntitiesListDataPayload {
  total_count: number;
  entities: EntitySummaryListItem[];
}

/**
 * Response wrapper for entities list
 * Matches EntitiesListResponsePayload from code_entities_list_all_handler.rs
 */
export interface EntitiesListResponsePayload {
  success: boolean;
  endpoint: string;
  data: EntitiesListDataPayload;
  tokens: number;
}

/**
 * Data payload for statistics
 * Matches StatisticsOverviewDataPayload from codebase_statistics_handler.rs
 */
export interface StatisticsOverviewDataPayload {
  code_entities_total_count: number;
  test_entities_total_count: number;
  dependency_edges_total_count: number;
  languages_detected_list: string[];
  database_file_path: string;
}

/**
 * Response wrapper for statistics
 */
export interface StatisticsOverviewResponsePayload {
  success: boolean;
  endpoint: string;
  data: StatisticsOverviewDataPayload;
  tokens: number;
}

/**
 * Health check response
 */
export interface HealthCheckResponsePayload {
  success: boolean;
  endpoint: string;
  status: string;
  tokens?: number;
}

/**
 * Query parameters for entities list endpoint
 */
export interface EntitiesListQueryParams {
  entity_type?: string;
}

/**
 * API error response (inferred pattern)
 */
export interface ApiErrorResponse {
  success: false;
  endpoint: string;
  error: string;
  tokens?: number;
}
