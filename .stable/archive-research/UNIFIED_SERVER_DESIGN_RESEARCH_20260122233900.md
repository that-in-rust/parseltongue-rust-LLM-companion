# Parseltongue Unified Server: Design Research Document

> Single binary, live diff watching, git state awareness

---

## Reduced Scope: LIVE WATCH ONLY

**What we're building**: A dashboard to watch dependency graph changes in real-time as files are edited (primarily by AI agents).

**What we're NOT building** (for now):
- Historical commit-to-commit comparison
- GitHub URL support
- Multiple comparison modes

---

## Core Problem: Database Design for Diff

### The Challenge

Entity keys can change between base and current state:
- Function renamed: `rust:fn:old_name` → `rust:fn:new_name`
- Function moved: `rust:mod:a::fn:foo` → `rust:mod:b::fn:foo`
- Function split: `rust:fn:big` → `rust:fn:part1` + `rust:fn:part2`

**We cannot simply diff by key.**

### Solution: Content-Based Matching + Heuristics

```
┌─────────────────────────────────────────────────────────────────────┐
│                     DIFF MATCHING STRATEGY                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  1. EXACT KEY MATCH (easy case)                                     │
│     Base: rust:fn:handle_auth    Current: rust:fn:handle_auth       │
│     → Same key exists in both → compare signatures/body             │
│                                                                      │
│  2. KEY MISSING (added or removed)                                  │
│     Base: rust:fn:old_handler    Current: (not found)               │
│     → Mark as REMOVED                                               │
│     Current: rust:fn:new_helper  Base: (not found)                  │
│     → Mark as ADDED                                                 │
│                                                                      │
│  3. RENAME DETECTION (heuristic)                                    │
│     Base: rust:fn:process_data   Current: rust:fn:handle_data       │
│     → Same signature, same body hash → likely RENAMED               │
│     → Show as: process_data → handle_data (RENAMED)                 │
│                                                                      │
│  For MVP: Skip rename detection, just show added/removed            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Database Schema (Single Table Per Workspace)

**Note**: This is the conceptual schema. The actual Parseltongue API returns slightly different field names (see ARCHITECTURE_API_GROUNDED doc for exact structures).

```sql
-- entities table (same schema for base.db and live.db)
CREATE TABLE entities (
  key TEXT PRIMARY KEY,        -- rust:fn:handle_auth:__path_hash:10-50
  entity_type TEXT,            -- fn, struct, trait, enum (NOT "kind")
  entity_class TEXT,           -- CODE or TEST
  language TEXT,               -- rust, python, etc.
  file_path TEXT,              -- ./crates/path/to/file.rs
  -- Note: line numbers are encoded in the key, not separate fields
  signature_hash TEXT,         -- SHA256 of function signature
  body_hash TEXT               -- SHA256 of function body
);

-- edges table
CREATE TABLE edges (
  from_key TEXT,
  to_key TEXT,
  edge_type TEXT,              -- Uses, Calls, Implements, Contains
  source_location TEXT,        -- ./path/file.rs:123
  PRIMARY KEY (from_key, to_key, edge_type)
);
```

**API Response Envelope**: All Parseltongue API endpoints return:
```json
{
  "success": true,
  "endpoint": "/code-entities-list-all",
  "data": { ... },
  "tokens": 50
}
```

### Diff Algorithm

```
Given base.db and live.db:

1. Load all entity keys from both
2. For each key in base but not in live → REMOVED
3. For each key in live but not in base → ADDED
4. For each key in both:
   - Compare signature_hash → SIGNATURE_CHANGED
   - Compare body_hash → BODY_CHANGED
   - Compare fan_in/fan_out → COUPLING_CHANGED
5. Same for edges

Result: List of changes with blast radius calculation
```

---

## Git State Awareness

The dashboard shows the **actual git state** of the working directory:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         GIT STATE DISPLAY                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  STATE 1: Clean                                                      │
│  ─────────────────                                                   │
│  "Working directory clean - no changes since base commit"           │
│  Base: abc123 (main)  │  Status: ✓ CLEAN                           │
│                                                                      │
│  STATE 2: Uncommitted Changes                                        │
│  ────────────────────────────                                        │
│  "3 files modified, not staged"                                     │
│  Base: abc123 (main)  │  Status: MODIFIED (unstaged)               │
│  Files: src/auth.rs, src/main.rs, tests/auth_test.rs               │
│                                                                      │
│  STATE 3: Staged Changes                                             │
│  ───────────────────────                                             │
│  "3 files staged, ready to commit"                                  │
│  Base: abc123 (main)  │  Status: STAGED                            │
│                                                                      │
│  STATE 4: New Commit Made                                            │
│  ────────────────────────                                            │
│  "1 commit ahead of base"                                           │
│  Base: abc123 (main)  │  HEAD: def456 │  Status: COMMITTED         │
│  Option: [Update Base to HEAD]                                      │
│                                                                      │
│  STATE 5: Branch Changed                                             │
│  ────────────────────────                                            │
│  "On different branch than base"                                    │
│  Base: abc123 (main)  │  Current: feature/auth │  Status: BRANCH   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Centralized Storage Architecture

All data lives in `~/.parseltongue/`:

```
~/.parseltongue/
├── config.json                      # Global config, port, theme
├── workspaces/
│   ├── {workspace-id}/              # SHA256 hash of absolute path
│   │   ├── meta.json
│   │   │   {
│   │   │     "name": "my-project",
│   │   │     "source_path": "/Users/dev/my-project",
│   │   │     "base_commit": "abc123",
│   │   │     "base_branch": "main",
│   │   │     "created_at": "2026-01-22T...",
│   │   │     "last_accessed": "2026-01-22T..."
│   │   │   }
│   │   ├── base.db                  # CozoDB snapshot at base commit
│   │   └── live.db                  # CozoDB of current working state
│   └── ...
└── server.pid                       # PID file when server is running
```

**Workspace Operations**:
- **Add**: User provides folder path → index current HEAD as base → create workspace
- **Watch**: Monitor file changes → re-index live.db → push diff via WebSocket
- **Delete**: User clicks delete → removes entire workspace folder
- **Update Base**: User commits → optionally update base to new HEAD

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         parseltongue serve                               │
│                              (single binary)                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────┐   ┌──────────────────┐   ┌──────────────────┐    │
│  │   Diff-First UI  │   │  Workspace Mgr   │   │   Watch Engine   │    │
│  │   (Three.js)     │   │  (~/.parseltongue)│   │   (File Events)  │    │
│  └────────┬─────────┘   └────────┬─────────┘   └────────┬─────────┘    │
│           │                      │                      │               │
│           ▼                      ▼                      ▼               │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     HTTP + WebSocket Router                      │   │
│  │  GET  /                           → Diff Dashboard               │   │
│  │  GET  /api/workspaces             → List all workspaces          │   │
│  │  POST /api/workspaces             → Add folder (set base=HEAD)   │   │
│  │  DELETE /api/workspaces/{id}      → Remove workspace             │   │
│  │  GET  /api/workspaces/{id}/diff   → Get current diff from base   │   │
│  │  GET  /api/workspaces/{id}/git    → Get git status               │   │
│  │  POST /api/workspaces/{id}/watch  → Start/stop live watch        │   │
│  │  POST /api/workspaces/{id}/base   → Update base to current HEAD  │   │
│  │  WS   /api/workspaces/{id}/live   → WebSocket for live updates   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## User Journeys (Live Watch Only)

### Journey 1: Agent Making Changes (Primary Use Case)

**Persona**: Developer using an AI agent to modify code
**Goal**: Watch what the agent is changing in real-time, decide whether to commit

```
1. Developer has Claude Code or another agent editing their project

2. Open Parseltongue: parseltongue serve
   → Browser opens: http://localhost:7777

3. Click "Add Workspace" → Enter: /Users/dev/my-project
   → Detects git repo
   → Indexes current HEAD as base → stores in ~/.parseltongue/workspaces/{hash}/base.db
   → Automatically starts watching

4. LIVE DIFF DASHBOARD:
   ┌─────────────────────────────────────────────────────────────┐
   │  my-project                    Base: abc123 ↔ WATCHING     │
   │  ─────────────────────────────────────────────────────────  │
   │                                                             │
   │  GIT STATUS: MODIFIED (3 files unstaged)                   │
   │                                                             │
   │  DIFF SUMMARY                                               │
   │  +3 entities added  │  -1 removed  │  ~2 modified          │
   │  +5 new edges       │  -2 removed edges                    │
   │  Blast radius: 7 entities affected                         │
   │  Risk: LOW ✓                                               │
   │                                                             │
   │  [Pause Watch]  [Update Base]  [Delete Workspace]          │
   │                                                             │
   │  ┌───────────────────────────────────────────────────────┐ │
   │  │                3D DIFF VISUALIZATION                  │ │
   │  │     (green = added, red = removed, yellow = modified) │ │
   │  └───────────────────────────────────────────────────────┘ │
   └─────────────────────────────────────────────────────────────┘

5. Agent saves a file → Dashboard UPDATES LIVE:
   → Toast: "File changed: src/auth.rs"
   → Git status updates: "MODIFIED (4 files unstaged)"
   → Diff recalculates
   → Visualization animates new node appearing

6. Agent commits → Git status updates:
   → "COMMITTED (1 ahead of base)"
   → Option appears: [Update Base to HEAD]

7. Developer clicks "Update Base" → base.db re-indexed to new HEAD
```

### Journey 2: Reopen After Days

**Persona**: Developer returning to a project after time away
**Goal**: Quickly see what's changed since they last worked

```
1. Open laptop after a week
2. Run: parseltongue serve

3. Dashboard shows:
   WORKSPACES
   ├── my-project (last: 7 days ago)
   │   Base: abc123 (main)  │  Status: 3 commits ahead
   │   [Watch] [Delete]
   └── [Add Workspace]

4. Click "Watch" on my-project
   → Loads base.db from ~/.parseltongue/workspaces/{hash}/
   → Indexes current working directory → live.db
   → Shows diff: "+15 entities, -3 removed"

5. Developer sees: "Oh, I committed 3 times since base, should update"
   → Clicks [Update Base to HEAD]
```

### Journey 3: Delete Workspace

**Persona**: Developer done with a project or cleaning up
**Goal**: Remove workspace completely

```
1. Open dashboard
2. Click [Delete] on a workspace
3. Confirm: "Delete workspace 'my-project'? This removes all indexed data."
4. Workspace folder removed from ~/.parseltongue/workspaces/{hash}/
5. Workspace disappears from sidebar
```

---

## API Design (Live Watch Only)

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Web UI (diff dashboard) |
| `/api/workspaces` | GET | List all workspaces |
| `/api/workspaces` | POST | Add new workspace (local path only) |
| `/api/workspaces/{id}` | DELETE | Remove workspace and all data |
| `/api/workspaces/{id}/diff` | GET | Get diff from base to current state |
| `/api/workspaces/{id}/git` | GET | Get git status (clean/modified/staged/committed/branch) |
| `/api/workspaces/{id}/watch` | POST | Start/stop watch mode |
| `/api/workspaces/{id}/base` | POST | Update base to current HEAD |
| `/api/workspaces/{id}/live` | WS | WebSocket for live diff updates |

### WebSocket Protocol (`/api/workspaces/{id}/live`)

```json
// Client connects
{"type": "connected", "base_commit": "abc123", "git_status": "clean"}

// File changes detected (debounced 500ms)
{"type": "file_changed", "files": ["src/auth.rs"], "git_status": "modified"}

// Diff recalculated
{"type": "diff_updated", "diff": {
  "entities": {"added": 3, "removed": 1, "modified": 2},
  "edges": {"added": 5, "removed": 2},
  "blast_radius": 7
}}

// Git status changed (commit made, branch changed, etc.)
{"type": "git_status", "status": "committed", "commits_ahead": 1}
```

---

## Visualization Data Format

The diff engine produces data for Three.js visualization:

```json
{
  "summary": {
    "entities_added": 3,
    "entities_removed": 1,
    "entities_modified": 2,
    "edges_added": 5,
    "edges_removed": 2,
    "blast_radius": 7
  },
  "nodes": [
    {
      "key": "rust:fn:new_auth:__crates_path_src_auth_rs:10-50",
      "status": "added",
      "file_path": "./crates/path/src/auth.rs",
      "entity_type": "fn",
      "entity_class": "CODE",
      "is_external": false
    },
    {
      "key": "rust:fn:old_handler:__crates_path_src_lib_rs:20-30",
      "status": "removed"
    },
    {
      "key": "rust:fn:handler:__crates_path_src_lib_rs:42-60",
      "status": "modified",
      "change": "body"
    },
    {
      "key": "rust:fn:map:unknown:0-0",
      "status": "ambient",
      "is_external": true
    }
  ],
  "edges": [
    {
      "from_key": "rust:fn:handler:__path:42-60",
      "to_key": "rust:fn:new_auth:__path:10-50",
      "edge_type": "Calls",
      "status": "added",
      "source_location": "./crates/path/src/lib.rs:45"
    },
    {
      "from_key": "rust:fn:handler:__path:42-60",
      "to_key": "rust:fn:old_handler:__path:20-30",
      "edge_type": "Calls",
      "status": "removed"
    }
  ]
}
```

**Note on External Entities**: Entities with `unknown:0-0` suffix are external references (stdlib, external crates). They should be rendered differently (e.g., smaller, grayed out, no source link).

---

## Technical Implementation Notes

### New Crate Structure
```
crates/
├── parseltongue/                    # CLI binary (update)
├── parseltongue-core/               # Existing core
├── pt01-folder-to-cozodb-streamer/  # Existing indexer (reuse)
├── pt08-http-code-query-server/     # Existing API (refactor for reuse)
├── pt09-unified-web-server/         # NEW: Unified server
│   ├── src/
│   │   ├── lib.rs
│   │   ├── central_storage.rs       # ~/.parseltongue/ management
│   │   ├── workspace_manager.rs     # Workspace CRUD
│   │   ├── watch_engine.rs          # File system watcher
│   │   ├── git_status.rs            # Git state detection
│   │   ├── websocket.rs             # Live updates
│   │   └── routes.rs                # HTTP endpoints
│   └── static/
│       ├── index.html
│       ├── app.js                   # Three.js visualization
│       └── styles.css
└── pt10-diff-graph-calculator/      # NEW: Diff logic
    └── src/
        ├── lib.rs
        ├── entity_diff.rs
        ├── edge_diff.rs
        └── blast_radius.rs
```

### Key Dependencies

```toml
# pt09-unified-web-server
[dependencies]
axum = "0.7"                    # HTTP + WebSocket
tokio = { version = "1", features = ["full"] }
notify = "6.0"                  # File system watcher
git2 = "0.18"                   # Git operations
rust-embed = "8.0"              # Embed static files in binary
tower-http = { version = "0.5", features = ["cors"] }

# pt10-diff-graph-calculator
[dependencies]
cozo = "0.7"                    # Database access
```

---

## Design Decisions (Live Watch Only)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Scope** | Live watch only | MVP focused, no historical comparison complexity |
| **Primary Use Case** | Watch agent edits | Developer sees changes as AI agent works |
| **Storage Location** | ~/.parseltongue/ | Centralized, doesn't pollute repos |
| **Watch Mode** | `notify` crate | File system events, debounced 500ms |
| **Live Updates** | WebSocket | Push diff changes to browser in real-time |
| **Git State** | Show actual status | Clean/modified/staged/committed/branch |
| **Visualization** | Three.js/3D | Diff nodes colored by status |
| **Delete** | Full workspace removal | User can clean up unwanted workspaces |

---

## Implementation Phases

### Phase 1: Foundation + Watch Mode
**Goal**: MVP for watching file changes

- [ ] Create pt09-unified-web-server crate
- [ ] Central storage manager (~/.parseltongue/)
- [ ] Workspace CRUD (add, list, delete)
- [ ] File watcher using `notify` crate
- [ ] WebSocket server for live updates
- [ ] Git status detection (`git2` crate)
- [ ] Basic HTML dashboard

**Exit Criteria**: Add workspace, watch files, see git status updates live

### Phase 2: Diff Engine
**Goal**: Compute meaningful dependency graph diffs

- [ ] Create pt10-diff-graph-calculator crate
- [ ] Entity diff (added/removed/modified by key)
- [ ] Edge diff (new/removed dependencies)
- [ ] Blast radius calculation
- [ ] Integrate with watch mode

**Exit Criteria**: WebSocket pushes entity/edge diff, not just file counts

### Phase 3: 3D Diff Visualization
**Goal**: Visual representation of changes

- [ ] Three.js scene (embedded in binary)
- [ ] Node rendering (green=added, red=removed, yellow=modified)
- [ ] Edge rendering (same color scheme)
- [ ] Animated transitions on diff update
- [ ] Click-to-inspect entity details

**Exit Criteria**: Can visually see what changed in 3D

### Phase 4: Polish
**Goal**: Production-ready

- [ ] Update base to HEAD action
- [ ] Loading states and error handling
- [ ] Export diff as JSON
- [ ] Documentation

---

## Critical Files to Modify/Create

| File | Action | Purpose |
|------|--------|---------|
| `crates/parseltongue/src/main.rs` | Modify | Add `serve` subcommand |
| `crates/pt09-unified-web-server/` | Create | New crate |
| `crates/pt09-unified-web-server/src/central_storage.rs` | Create | ~/.parseltongue/ management |
| `crates/pt09-unified-web-server/src/workspace_manager.rs` | Create | Workspace CRUD |
| `crates/pt09-unified-web-server/src/watch_engine.rs` | Create | File system watcher |
| `crates/pt09-unified-web-server/src/git_status.rs` | Create | Git state detection |
| `crates/pt09-unified-web-server/src/websocket.rs` | Create | Live updates |
| `crates/pt09-unified-web-server/static/` | Create | Embedded HTML/JS (Three.js) |
| `crates/pt10-diff-graph-calculator/` | Create | New crate for diff logic |

---

## Summary

### Scope: LIVE WATCH ONLY

This is an MVP focused on the primary use case: **watching AI agents edit code**.

**NOT included** (future work):
- Historical commit-to-commit comparison
- GitHub URL support
- Multiple comparison modes

### What Users Get

1. **Add Workspace**: Point to local git repo → indexes HEAD as base
2. **Watch**: See file changes live, git status updates
3. **Diff**: See entities added/removed/modified, edges changed, blast radius
4. **Delete**: Remove workspace when done

### Dashboard

```
┌─────────────────────────────────────────────────────────────────────┐
│  my-project                         Base: abc123 ↔ WATCHING        │
│  ─────────────────────────────────────────────────────────────────  │
│                                                                      │
│  GIT: MODIFIED (3 files unstaged)                                   │
│                                                                      │
│  +3 entities  │  -1 removed  │  ~2 modified  │  Blast radius: 7    │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │              3D DIFF (Three.js)                                │ │
│  │   ● green = added   ● red = removed   ● yellow = modified     │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  [Pause Watch]  [Update Base]  [Delete Workspace]                   │
└─────────────────────────────────────────────────────────────────────┘
```

### New Crates

- `pt09-unified-web-server`: HTTP + WebSocket server, workspace management
- `pt10-diff-graph-calculator`: Entity diff, edge diff, blast radius

---

*MVP: Watch AI agents edit code, see dependency graph diff in real-time, understand blast radius before committing.*
