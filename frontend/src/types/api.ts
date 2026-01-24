/**
 * TypeScript types matching Rust backend API responses.
 * These types are derived from parseltongue-core and websocket_streaming_module.
 */

// =============================================================================
// Workspace Types (match parseltongue-core/src/workspace/types.rs)
// =============================================================================

/**
 * Metadata for a single workspace.
 */
export interface WorkspaceMetadata {
  workspace_identifier_value: string;
  workspace_display_name: string;
  source_directory_path_value: string;
  base_database_path_value: string;
  live_database_path_value: string;
  watch_enabled_flag_status: boolean;
  created_timestamp_utc_value: string;
  last_indexed_timestamp_option: string | null;
}

/**
 * Request payload for creating a new workspace.
 */
export interface WorkspaceCreateRequest {
  source_path_directory_value: string;
  workspace_display_name_option?: string;
}

/**
 * Request payload for toggling workspace watch state.
 */
export interface WorkspaceWatchToggleRequest {
  workspace_identifier_target_value: string;
  watch_enabled_desired_state: boolean;
}

/**
 * Response from GET /workspace-list-all endpoint.
 */
export interface WorkspaceListResponse {
  success: boolean;
  endpoint: string;
  workspaces: WorkspaceMetadata[];
  total_workspace_count_value: number;
  token_estimate: number;
}

/**
 * Response from workspace operations (create, toggle watch).
 */
export interface WorkspaceOperationResponse {
  success: boolean;
  endpoint: string;
  workspace: WorkspaceMetadata;
  token_estimate: number;
}

/**
 * Error response from workspace operations.
 */
export interface WorkspaceErrorResponse {
  error: string;
  code: 'PATH_NOT_FOUND' | 'PATH_NOT_DIRECTORY' | 'WORKSPACE_ALREADY_EXISTS' | string;
  existing_workspace_id?: string;
}

// =============================================================================
// WebSocket Types (match websocket_streaming_module/message_types.rs)
// =============================================================================

/**
 * Line range data for entity locations.
 */
export interface LineRangeData {
  start: number;
  end: number;
}

/**
 * Summary of diff changes.
 */
export interface DiffSummaryData {
  total_before_count: number;
  total_after_count: number;
  added_entity_count: number;
  removed_entity_count: number;
  modified_entity_count: number;
  unchanged_entity_count: number;
  relocated_entity_count: number;
}

/**
 * Messages sent from client to server.
 */
export type WebSocketClientMessage =
  | { action: 'subscribe'; workspace_id: string }
  | { action: 'unsubscribe' }
  | { action: 'ping' };

/**
 * Events sent from server to client.
 */
export type WebSocketServerEvent =
  | { event: 'subscribed'; workspace_id: string; workspace_name: string; timestamp: string }
  | { event: 'unsubscribed'; timestamp: string }
  | { event: 'pong'; timestamp: string }
  | { event: 'diff_started'; workspace_id: string; files_changed: number; triggered_by: string; timestamp: string }
  | { event: 'entity_added'; workspace_id: string; entity_key: string; entity_type: string; file_path: string; line_range: LineRangeData | null; timestamp: string }
  | { event: 'entity_removed'; workspace_id: string; entity_key: string; entity_type: string; file_path: string; timestamp: string }
  | { event: 'entity_modified'; workspace_id: string; entity_key: string; entity_type: string; file_path: string; before_line_range: LineRangeData | null; after_line_range: LineRangeData | null; timestamp: string }
  | { event: 'edge_added'; workspace_id: string; from_entity_key: string; to_entity_key: string; edge_type: string; timestamp: string }
  | { event: 'edge_removed'; workspace_id: string; from_entity_key: string; to_entity_key: string; edge_type: string; timestamp: string }
  | { event: 'diff_completed'; workspace_id: string; summary: DiffSummaryData; blast_radius_count: number; duration_ms: number; timestamp: string }
  | { event: 'error'; code: string; message: string; timestamp: string };

// =============================================================================
// Visualization Types (for react-force-graph-3d)
// =============================================================================

/**
 * Change type for entities in the diff visualization.
 */
export type ChangeType = 'added' | 'removed' | 'modified' | 'affected' | null;

/**
 * Node in the force graph.
 * x, y, z are added by ForceGraph3D at runtime.
 */
export interface GraphNode {
  id: string;
  name: string;
  nodeType: string;
  changeType: ChangeType;
  filePath?: string;
  lineStart?: number;
  lineEnd?: number;
  // Runtime properties added by ForceGraph3D
  x?: number;
  y?: number;
  z?: number;
}

/**
 * Link/edge in the force graph.
 */
export interface GraphLink {
  source: string;
  target: string;
  edgeType: string;
}

/**
 * Data structure for react-force-graph-3d.
 */
export interface ForceGraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

/**
 * Non-null change types for use as Record keys.
 */
export type ChangeTypeKey = 'added' | 'removed' | 'modified' | 'affected' | 'unchanged';

/**
 * Color mapping for change types.
 */
export const CHANGE_TYPE_COLORS: Record<ChangeTypeKey, string> = {
  added: '#22c55e',     // green-500
  removed: '#ef4444',   // red-500
  modified: '#f59e0b',  // amber-500
  affected: '#3b82f6',  // blue-500
  unchanged: '#6b7280', // gray-500
};

/**
 * Get color for a change type, handling null.
 */
export function getChangeTypeColor(changeType: ChangeType): string {
  if (changeType === null) return CHANGE_TYPE_COLORS.unchanged;
  return CHANGE_TYPE_COLORS[changeType] ?? CHANGE_TYPE_COLORS.unchanged;
}

// =============================================================================
// API Response Types for Diff Visualization
// =============================================================================

/**
 * API response format for diff visualization data.
 */
export interface ApiDiffVisualization {
  nodes: Array<{
    id: string;
    label: string;
    node_type: string;
    change_type: 'added' | 'removed' | 'modified' | 'affected' | null;
    file_path?: string;
    line_start?: number;
    line_end?: number;
  }>;
  edges: Array<{
    source: string;
    target: string;
    edge_type: string;
  }>;
}

// =============================================================================
// Connection Status Types
// =============================================================================

/**
 * WebSocket connection status.
 */
export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error';
