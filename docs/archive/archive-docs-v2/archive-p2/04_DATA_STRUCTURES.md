# Data Structures: Single Source of Truth

> Canonical TypeScript interfaces and Rust structs for diff visualization system
> All other documents reference this file for data shapes.

---

## 1. API Response Envelope (All Endpoints)

Every Parseltongue API response uses this envelope:

### TypeScript
```typescript
interface ApiResponse<T> {
  success: boolean;
  endpoint: string;        // Echo of called endpoint, e.g., "/code-entities-list-all"
  data?: T;                // Present when success=true
  error?: string;          // Present when success=false
  tokens: number;          // Token estimate for LLM context budgeting
}
```

### Rust
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub tokens: u32,
}
```

---

## 2. Entity (from /code-entities-list-all)

### TypeScript
```typescript
interface Entity {
  key: string;              // "rust:fn:main:__crates_src_main_rs:10-50"
  file_path: string;        // "./crates/src/main.rs"
  entity_type: string;      // "fn", "struct", "enum", "trait", "impl", "mod", "file"
  entity_class: string;     // "CODE" | "TEST"
  language: string;         // "rust", "python", "typescript", etc.
}

// Response wrapper
interface EntitiesListResponse {
  entities: Entity[];
  total_count: number;
}
```

### Rust
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Entity {
    pub key: String,
    pub file_path: String,
    pub entity_type: String,
    pub entity_class: EntityClass,
    pub language: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum EntityClass {
    Code,
    Test,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitiesListResponse {
    pub entities: Vec<Entity>,
    pub total_count: usize,
}
```

### Key Format Specification
```
{language}:{entity_type}:{name}:{path_hash}:{start_line}-{end_line}

Examples:
  rust:fn:handle_auth:__crates_pt08_src_handler_rs:10-50
  rust:struct:AppState:__crates_core_src_lib_rs:5-20
  rust:fn:map:unknown:0-0   // External reference (stdlib/crate)

Path hash rules:
  - Prefix: __
  - Directory separator: _ (replaces /)
  - Hyphen: preserved
  - Example: ./crates/pt08-http/src/main.rs -> __crates_pt08-http_src_main_rs
```

---

## 3. Edge (from /dependency-edges-list-all)

### TypeScript
```typescript
interface Edge {
  from_key: string;         // Source entity key
  to_key: string;           // Target entity key
  edge_type: EdgeType;      // "Uses" | "Calls" | "Implements" | "Contains"
  source_location: string;  // "./crates/path/file.rs:123"
}

type EdgeType = 'Uses' | 'Calls' | 'Implements' | 'Contains';

// Response wrapper (supports pagination)
interface EdgesListResponse {
  edges: Edge[];
  total_count: number;
  returned_count: number;
  limit: number;
  offset: number;
}
```

### Rust
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from_key: String,
    pub to_key: String,
    pub edge_type: EdgeType,
    pub source_location: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeType {
    Uses,
    Calls,
    Implements,
    Contains,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgesListResponse {
    pub edges: Vec<Edge>,
    pub total_count: usize,
    pub returned_count: usize,
    pub limit: usize,
    pub offset: usize,
}
```

---

## 4. Cluster (from /semantic-cluster-grouping-list)

### TypeScript
```typescript
interface Cluster {
  cluster_id: number;       // Integer ID, NOT a string name
  entity_count: number;     // Number of entities in cluster
  entities: string[];       // Array of entity keys
  internal_edges: number;   // Edges within cluster
  external_edges: number;   // Edges to/from other clusters
}

// Response wrapper
interface ClustersListResponse {
  clusters: Cluster[];
  total_clusters: number;
}
```

### Rust
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub cluster_id: u32,
    pub entity_count: usize,
    pub entities: Vec<String>,
    pub internal_edges: usize,
    pub external_edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClustersListResponse {
    pub clusters: Vec<Cluster>,
    pub total_clusters: usize,
}
```

---

## 5. Statistics (from /codebase-statistics-overview-summary)

### TypeScript
```typescript
interface CodebaseStatistics {
  code_entities_total_count: number;
  test_entities_total_count: number;
  dependency_edges_total_count: number;
  languages_detected_list: string[];
  database_file_path: string;
}
```

### Rust
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseStatistics {
    pub code_entities_total_count: usize,
    pub test_entities_total_count: usize,
    pub dependency_edges_total_count: usize,
    pub languages_detected_list: Vec<String>,
    pub database_file_path: String,
}
```

---

## 6. Diff Result (NEW - for pt09/pt10)

### TypeScript
```typescript
// Change types for entities
type EntityChangeType = 'added' | 'removed' | 'modified' | 'moved' | 'relocated';

interface EntityChange {
  change_type: EntityChangeType;
  entity: Entity;
  base_entity?: Entity;     // Present for modified/moved/relocated
  stable_id: string;        // Key without line numbers
}

// Change types for edges
type EdgeChangeType = 'added' | 'removed';

interface EdgeChange {
  change_type: EdgeChangeType;
  edge: Edge;
}

// Complete diff result
interface DiffResult {
  summary: DiffSummary;
  entities: {
    added: Entity[];
    removed: Entity[];
    modified: ModifiedEntity[];
    moved: MovedEntity[];
  };
  edges: {
    added: Edge[];
    removed: Edge[];
  };
  affected_neighbors: string[];  // Keys of 1-hop neighbors
}

interface DiffSummary {
  entities_added: number;
  entities_removed: number;
  entities_modified: number;
  entities_moved: number;
  edges_added: number;
  edges_removed: number;
  blast_radius: number;       // Total affected entities
}

interface ModifiedEntity {
  stable_id: string;
  base: Entity;
  live: Entity;
  change_description: string; // "body changed", "signature changed", etc.
}

interface MovedEntity {
  stable_id: string;
  base: Entity;
  live: Entity;
  lines_shifted: number;      // Positive = moved down, negative = moved up
}
```

### Rust
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityChangeType {
    Added,
    Removed,
    Modified,
    Moved,
    Relocated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeChangeType {
    Added,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub summary: DiffSummary,
    pub entities: EntityDiff,
    pub edges: EdgeDiff,
    pub affected_neighbors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiffSummary {
    pub entities_added: usize,
    pub entities_removed: usize,
    pub entities_modified: usize,
    pub entities_moved: usize,
    pub edges_added: usize,
    pub edges_removed: usize,
    pub blast_radius: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityDiff {
    pub added: Vec<Entity>,
    pub removed: Vec<Entity>,
    pub modified: Vec<ModifiedEntity>,
    pub moved: Vec<MovedEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EdgeDiff {
    pub added: Vec<Edge>,
    pub removed: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedEntity {
    pub stable_id: String,
    pub base: Entity,
    pub live: Entity,
    pub change_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovedEntity {
    pub stable_id: String,
    pub base: Entity,
    pub live: Entity,
    pub lines_shifted: i32,
}
```

---

## 7. Workspace (NEW - for pt09)

### TypeScript
```typescript
interface Workspace {
  id: string;                 // SHA256 hash of source_path
  name: string;               // Human-readable name (from folder)
  source_path: string;        // Absolute path to source
  base_commit: string;        // Git commit SHA of base snapshot
  base_branch: string;        // Branch name at base time
  created_at: string;         // ISO 8601 timestamp
  last_accessed: string;      // ISO 8601 timestamp
  watching: boolean;          // Currently watching for changes
}

interface WorkspacesListResponse {
  workspaces: Workspace[];
}

interface CreateWorkspaceRequest {
  source_path: string;        // Path to git repository
}

interface CreateWorkspaceResponse {
  workspace: Workspace;
  initial_stats: CodebaseStatistics;
}
```

### Rust
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub source_path: PathBuf,
    pub base_commit: String,
    pub base_branch: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub watching: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacesListResponse {
    pub workspaces: Vec<Workspace>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub source_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateWorkspaceResponse {
    pub workspace: Workspace,
    pub initial_stats: CodebaseStatistics,
}
```

---

## 8. Git Status (NEW - for pt09)

### TypeScript
```typescript
type GitState =
  | 'clean'           // Working directory matches HEAD
  | 'modified'        // Files changed, not staged
  | 'staged'          // Files staged, not committed
  | 'committed'       // Commits ahead of base
  | 'branch_changed'; // On different branch than base

interface GitStatus {
  state: GitState;
  current_branch: string;
  base_branch: string;
  base_commit: string;
  head_commit: string;
  commits_ahead: number;      // Commits since base
  modified_files: string[];   // Unstaged changes
  staged_files: string[];     // Staged changes
}
```

### Rust
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GitState {
    Clean,
    Modified,
    Staged,
    Committed,
    BranchChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub state: GitState,
    pub current_branch: String,
    pub base_branch: String,
    pub base_commit: String,
    pub head_commit: String,
    pub commits_ahead: u32,
    pub modified_files: Vec<String>,
    pub staged_files: Vec<String>,
}
```

---

## 9. WebSocket Messages (NEW - for pt09)

### TypeScript
```typescript
// Server -> Client messages
type ServerMessage =
  | { type: 'connected'; workspace_id: string; git_status: GitStatus }
  | { type: 'file_changed'; files: string[]; git_status: GitStatus }
  | { type: 'diff_updated'; diff: DiffResult }
  | { type: 'git_status'; status: GitStatus }
  | { type: 'base_updated'; new_commit: string }
  | { type: 'error'; message: string }
  | { type: 'indexing_started'; files: string[] }
  | { type: 'indexing_complete'; duration_ms: number };

// Client -> Server messages
type ClientMessage =
  | { type: 'request_full_diff' }
  | { type: 'pause_watch' }
  | { type: 'resume_watch' }
  | { type: 'update_base' };
```

### Rust
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Connected {
        workspace_id: String,
        git_status: GitStatus,
    },
    FileChanged {
        files: Vec<String>,
        git_status: GitStatus,
    },
    DiffUpdated {
        diff: DiffResult,
    },
    GitStatus {
        status: GitStatus,
    },
    BaseUpdated {
        new_commit: String,
    },
    Error {
        message: String,
    },
    IndexingStarted {
        files: Vec<String>,
    },
    IndexingComplete {
        duration_ms: u64,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    RequestFullDiff,
    PauseWatch,
    ResumeWatch,
    UpdateBase,
}
```

---

## 10. Visualization Node (Frontend Only)

### TypeScript
```typescript
type NodeStatus = 'added' | 'removed' | 'modified' | 'moved' | 'neighbor' | 'ambient';

interface VisualizationNode {
  key: string;
  stable_id: string;
  status: NodeStatus;
  position: { x: number; y: number; z: number };
  cluster_id: number;
  entity_type: string;
  is_external: boolean;       // true if key contains "unknown:0-0"

  // Visual properties (computed)
  color: string;
  size: number;
  opacity: number;
  glow: number;
  pulse: boolean;
  label_visible: boolean;
}

interface VisualizationEdge {
  from_key: string;
  to_key: string;
  edge_type: string;
  status: 'added' | 'removed' | 'unchanged';
  color: string;
  opacity: number;
}

// Complete visualization state
interface VisualizationState {
  nodes: VisualizationNode[];
  edges: VisualizationEdge[];
  clusters: ClusterRegion[];
  camera: CameraState;
}

interface ClusterRegion {
  cluster_id: number;
  display_name: string;       // Derived from entity paths
  center: { x: number; y: number; z: number };
  radius: number;
  entity_count: number;
}

interface CameraState {
  position: { x: number; y: number; z: number };
  target: { x: number; y: number; z: number };
  zoom: number;
}
```

---

## 11. Constants and Enums

### TypeScript
```typescript
// Visual constants
const STATUS_COLORS: Record<NodeStatus, string> = {
  added:    '#00ff88',
  removed:  '#ff4444',
  modified: '#ffcc00',
  moved:    '#4a9eff',
  neighbor: '#ffa94d',
  ambient:  '#888888',
};

const ENTITY_TYPE_COLORS: Record<string, string> = {
  fn:     '#4a9eff',
  struct: '#ff6b6b',
  enum:   '#ffa94d',
  impl:   '#69db7c',
  method: '#748ffc',
  mod:    '#f783ac',
  file:   '#868e96',
  trait:  '#da77f2',
  type:   '#20c997',
};

const EXTERNAL_ENTITY_COLOR = '#4a4a4a';

// Animation timings (ms)
const ANIMATION_TIMINGS = {
  fileChangeDebounce: 500,
  nodeScaleIn: 200,
  glowFadeIn: 300,
  neighborFadeIn: 200,
  removedfadeOut: 500,
};

// Size multipliers
const SIZE_MULTIPLIERS: Record<NodeStatus, number> = {
  added:    1.5,
  removed:  1.0,  // Shrinks during fade
  modified: 1.5,
  moved:    1.2,
  neighbor: 1.0,
  ambient:  0.5,
};

// Opacity values
const OPACITY_VALUES: Record<NodeStatus, number> = {
  added:    1.0,
  removed:  0.5,  // Fading
  modified: 1.0,
  moved:    1.0,
  neighbor: 0.7,
  ambient:  0.15,
};
```

---

## 12. Helper Functions

### TypeScript
```typescript
// Extract stable identity from full key
function extractStableId(key: string): string {
  // Key format: lang:type:name:path_hash:start-end
  const lastColon = key.lastIndexOf(':');
  if (lastColon === -1) return key;

  const beforeLines = key.substring(0, lastColon);
  const secondLastColon = beforeLines.lastIndexOf(':');
  if (secondLastColon === -1) return key;

  // Verify suffix looks like line numbers
  const suffix = key.substring(secondLastColon + 1);
  if (/^\d+-\d+$/.test(suffix.split(':').pop() || '')) {
    return beforeLines;
  }

  return key;
}

// Check if entity is external
function isExternalEntity(key: string): boolean {
  return key.includes('unknown:0-0') || key.endsWith(':0-0');
}

// Parse line numbers from key
function parseLineNumbers(key: string): { start: number; end: number } | null {
  const match = key.match(/:(\d+)-(\d+)$/);
  if (!match) return null;
  return { start: parseInt(match[1], 10), end: parseInt(match[2], 10) };
}

// Derive cluster display name from entities
function deriveClusterName(cluster: Cluster): string {
  if (cluster.entities.length === 0) {
    return `Cluster ${cluster.cluster_id}`;
  }

  // Find common path prefix
  const paths = cluster.entities
    .map(key => {
      const match = key.match(/:(__[^:]+):/);
      return match ? match[1] : null;
    })
    .filter((p): p is string => p !== null);

  if (paths.length === 0) {
    return `Cluster ${cluster.cluster_id}`;
  }

  // Extract meaningful name from path hash
  const commonPath = paths[0];
  const segments = commonPath.replace(/__/g, '').split('_');

  // Look for crate name (starts with pt or is a known pattern)
  const crateName = segments.find(s => s.startsWith('pt') || s === 'parseltongue');
  const moduleName = segments[segments.length - 1];

  if (crateName) {
    return `${crateName}/${moduleName}`;
  }

  return moduleName || `Cluster ${cluster.cluster_id}`;
}
```

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-22 | Initial creation from gap analysis |

---

*This is the single source of truth for data structures.*
*All other documents should reference this file.*
