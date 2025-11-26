# Session-Based UX Design - Parseltongue v1.1.0

**Date**: 2025-11-25
**Status**: 🎨 DESIGN PHASE
**Philosophy**: Session isolation + 4-word naming + Progressive disclosure

---

## 🎯 **VISION**

Every parseltongue invocation creates a **self-contained session folder** with:
- ✅ Timestamped isolation (multiple runs don't conflict)
- ✅ Everything in one place (DB + exports + visualizations)
- ✅ LLM-friendly (agents can analyze one session folder)
- ✅ 4-word naming everywhere (commands, crates, folders)

---

## ❌ **CURRENT VIOLATIONS: 4-Word Naming Convention**

### **Crate Names (5 crates)**

| Current Name | Words | Status | Proposed Fix |
|-------------|-------|--------|--------------|
| `parseltongue` | 1 | ❌ | `parseltongue-main-cli-binary` |
| `parseltongue-core` | 2 | ❌ | `parseltongue-core-entities-library` |
| `pt01-folder-to-cozodb-streamer` | 5 | ❌ | `parseltongue-ingest-folder-streamer` |
| `pt02-llm-cozodb-to-context-writer` | 6 | ❌ | `parseltongue-export-context-writer` |
| `pt07-visual-analytics-terminal` | 4 | ✅ | Keep (already compliant) |

### **CLI Commands (6 commands)**

| Current Command | Words | Status | Proposed Fix |
|----------------|-------|--------|--------------|
| `pt01-folder-to-cozodb-streamer` | 5 | ❌ | `ingest-folder-create-session` |
| `pt02-level00` | 2 | ❌ | `export-level-zero-edges` |
| `pt02-level01` | 2 | ❌ | `export-level-one-entities` |
| `pt02-level02` | 2 | ❌ | `export-level-two-types` |
| `pt07` | 1 | ❌ | `visualize-graph-terminal-output` |
| `help` | 1 | ❌ | Keep (standard convention) |

### **Subcommands (pt07)**

| Current | Words | Status | Proposed Fix |
|---------|-------|--------|--------------|
| `entity-count` | 2 | ❌ | `show-entity-count-chart` |
| `cycles` | 1 | ❌ | `detect-circular-dependency-cycles` |

---

## 📂 **NEW SESSION-BASED ARCHITECTURE**

### **Folder Structure**

```
<target-directory>/
└── .parseltongue/
    ├── session_20251125_083045/          # First run
    │   ├── database/
    │   │   └── codebase.db/              # RocksDB
    │   ├── exports/
    │   │   ├── level_zero_edges.json
    │   │   ├── level_one_entities.json
    │   │   └── level_two_types.json
    │   ├── visualizations/
    │   │   ├── entity_count_chart.txt
    │   │   └── circular_dependencies.txt
    │   └── session_metadata.json
    │
    ├── session_20251125_090312/          # Second run (re-run)
    │   ├── database/
    │   ├── exports/
    │   └── ...
    │
    ├── latest -> session_20251125_090312  # Symlink to latest
    └── .gitignore
```

### **Session Metadata (`session_metadata.json`)**

```json
{
  "session_id": "session_20251125_083045",
  "created_at": "2025-11-25T08:30:45Z",
  "parseltongue_version": "1.1.0",
  "target_directory": "/Users/foo/my_project",
  "scan_summary": {
    "files_found": 212,
    "files_processed": 78,
    "code_entities": 107,
    "test_entities": 932,
    "errors": 134
  },
  "exports": [
    {
      "level": 0,
      "file": "exports/level_zero_edges.json",
      "edges": 3393,
      "tokens": 5000
    },
    {
      "level": 1,
      "file": "exports/level_one_entities.json",
      "entities": 107,
      "tokens": 600000
    }
  ],
  "visualizations": [
    {
      "type": "entity-count",
      "file": "visualizations/entity_count_chart.txt"
    },
    {
      "type": "cycles",
      "file": "visualizations/circular_dependencies.txt",
      "cycles_found": 0
    }
  ]
}
```

---

## 🚀 **USER JOURNEY: Session-Based Workflow**

### **Journey 1: First-Time User (Simple)**

**Goal**: Analyze a codebase and get a dependency graph

```bash
# Step 1: Navigate to target codebase
cd ~/projects/my_rust_app

# Step 2: Run parseltongue (creates session automatically)
parseltongue ingest-folder-create-session .

# Output:
# ✓ Session created: .parseltongue/session_20251125_083045/
# ✓ Database: session_20251125_083045/database/codebase.db
# ✓ Scanned 78 files, created 107 entities
# ✓ Use 'parseltongue export-level-zero-edges' to export

# Step 3: Export dependency graph
parseltongue export-level-zero-edges

# Output:
# ✓ Exported to: .parseltongue/session_20251125_083045/exports/level_zero_edges.json
# ✓ 3,393 edges, ~5K tokens

# Step 4: Visualize
parseltongue show-entity-count-chart

# Output:
# [Shows bar chart in terminal]
# ✓ Saved to: .parseltongue/session_20251125_083045/visualizations/entity_count_chart.txt
```

**Result**: All artifacts in `.parseltongue/session_20251125_083045/`

---

### **Journey 2: Power User (Re-analyzing Codebase)**

**Goal**: Track changes over time by creating multiple sessions

```bash
# Initial analysis
cd ~/projects/my_rust_app
parseltongue ingest-folder-create-session .
# → Creates: .parseltongue/session_20251125_083045/

# ... Make code changes ...

# Re-analyze (creates NEW session)
parseltongue ingest-folder-create-session .
# → Creates: .parseltongue/session_20251125_090312/
# → Updates: .parseltongue/latest -> session_20251125_090312

# Compare sessions
parseltongue compare-session-entity-count \
  session_20251125_083045 \
  session_20251125_090312

# Output:
# Session 1: 107 entities
# Session 2: 112 entities (+5)
#
# New entities:
# - rust:fn:validate_input:src_validators_rs:45-67
# - rust:struct:Config:src_config_rs:10-30
# ...
```

---

### **Journey 3: LLM Agent Integration**

**Goal**: Agent analyzes codebase using parseltongue exports

```bash
# Agent workflow:
# 1. Ingest codebase
parseltongue ingest-folder-create-session ~/target_repo

# 2. Export for agent consumption
parseltongue export-level-one-entities

# 3. Agent reads JSON
SESSION_DIR=".parseltongue/latest"
ENTITIES="$SESSION_DIR/exports/level_one_entities.json"

# 4. Agent queries: "What functions call validate_payment?"
cat $ENTITIES | jq '.entities[] | select(.entity_name == "validate_payment") | .reverse_deps'

# Output:
# [
#   "rust:fn:process_payment:src_payment_rs:145-167",
#   "rust:fn:handle_checkout:src_checkout_rs:200-245"
# ]
```

**Benefit**: Agent only needs to know session folder structure

---

### **Journey 4: Team Collaboration (Shared Sessions)**

**Goal**: Share analysis results with team via git

```bash
# Developer 1: Analyze and commit session
cd ~/team_project
parseltongue ingest-folder-create-session .
git add .parseltongue/session_20251125_083045/
git commit -m "parseltongue: Add dependency analysis session"
git push

# Developer 2: Use existing session
cd ~/team_project
git pull
ls .parseltongue/session_20251125_083045/exports/
# → level_zero_edges.json
# → level_one_entities.json

# Visualize without re-ingesting
parseltongue show-entity-count-chart \
  --session session_20251125_083045
```

---

## 🎨 **PROPOSED CLI DESIGN**

### **New 4-Word Command Structure**

```bash
parseltongue <verb>-<constraint>-<target>-<qualifier> [OPTIONS]

Examples:
- ingest-folder-create-session
- export-level-zero-edges
- export-level-one-entities
- export-level-two-types
- show-entity-count-chart
- detect-circular-dependency-cycles
- compare-session-entity-count
- list-all-available-sessions
```

### **Command Reference**

#### **1. Ingestion Commands**

```bash
# Ingest codebase and create session
parseltongue ingest-folder-create-session <DIRECTORY>

Options:
  --session-name <NAME>      Custom session name (default: session_YYYYMMDD_HHMMSS)
  --exclude <PATTERNS>       Exclusion patterns (e.g., "target/,*.tmp")

Example:
  parseltongue ingest-folder-create-session ~/my_project
  parseltongue ingest-folder-create-session . --session-name "release_v1.0"
```

#### **2. Export Commands**

```bash
# Export Level 0: Pure edge list (~5K tokens)
parseltongue export-level-zero-edges [--session <NAME>]

# Export Level 1: Entities with dependencies (~30K tokens)
parseltongue export-level-one-entities [--session <NAME>]

# Export Level 2: Type system details (~60K tokens)
parseltongue export-level-two-types [--session <NAME>]

Options:
  --session <NAME>           Target session (default: latest)
  --where-clause <CLAUSE>    Filter query (default: ALL)
  --include-code <0|1>       Include code snippets (Level 1+ only)

Example:
  parseltongue export-level-one-entities
  parseltongue export-level-zero-edges --session session_20251125_083045
  parseltongue export-level-one-entities --where-clause "file_path ~ 'src/payment'"
```

#### **3. Visualization Commands**

```bash
# Show entity count bar chart
parseltongue show-entity-count-chart [--session <NAME>]

# Detect circular dependencies
parseltongue detect-circular-dependency-cycles [--session <NAME>]

Options:
  --session <NAME>           Target session (default: latest)
  --save                     Save to file (default: true)

Example:
  parseltongue show-entity-count-chart
  parseltongue detect-circular-dependency-cycles --session session_20251125_083045
```

#### **4. Session Management**

```bash
# List all sessions
parseltongue list-all-available-sessions

# Compare two sessions
parseltongue compare-session-entity-count <SESSION1> <SESSION2>

# Delete old sessions
parseltongue delete-old-session-artifacts <SESSION_NAME>

Example:
  parseltongue list-all-available-sessions
  parseltongue compare-session-entity-count session_20251125_083045 latest
  parseltongue delete-old-session-artifacts session_20251124_120000
```

---

## 🔧 **IMPLEMENTATION PLAN**

### **Phase 1: Session Infrastructure (v1.1.0-alpha)**

**Effort**: 4-6 hours
**Priority**: 🔴 CRITICAL

#### **Tasks:**

1. **Create session manager module**
   - File: `crates/parseltongue-core/src/session_manager.rs`
   - Functions:
     - `create_new_session(target_dir, session_name?) -> Session`
     - `get_latest_session(target_dir) -> Option<Session>`
     - `list_all_sessions(target_dir) -> Vec<Session>`
     - `get_session_metadata(session) -> Metadata`

2. **Update pt01 to use sessions**
   - File: `crates/parseltongue-ingest-folder-streamer/src/cli.rs`
   - Default DB path: `.parseltongue/{session}/database/codebase.db`
   - Create session metadata after ingestion

3. **Update pt02 to use sessions**
   - File: `crates/parseltongue-export-context-writer/src/cli.rs`
   - Default output: `.parseltongue/{session}/exports/level_{n}_*.json`
   - Auto-detect latest session if none specified

4. **Update pt07 to use sessions**
   - File: `crates/pt07-visual-analytics-terminal/src/cli.rs`
   - Default output: `.parseltongue/{session}/visualizations/`
   - Save visualizations to files

#### **Acceptance Criteria:**
- ✅ Running pt01 creates `.parseltongue/session_TIMESTAMP/`
- ✅ All artifacts go to session folder
- ✅ `latest` symlink points to newest session
- ✅ Session metadata JSON created

---

### **Phase 2: Rename Commands (v1.1.0-beta)**

**Effort**: 3-4 hours
**Priority**: 🟡 HIGH

#### **Tasks:**

1. **Rename CLI commands**
   - File: `crates/parseltongue/src/main.rs`
   - Old: `pt01-folder-to-cozodb-streamer` → New: `ingest-folder-create-session`
   - Old: `pt02-level00` → New: `export-level-zero-edges`
   - Old: `pt02-level01` → New: `export-level-one-entities`
   - Old: `pt02-level02` → New: `export-level-two-types`
   - Old: `pt07 entity-count` → New: `show-entity-count-chart`
   - Old: `pt07 cycles` → New: `detect-circular-dependency-cycles`

2. **Add command aliases (backwards compatibility)**
   ```rust
   #[command(alias = "pt01-folder-to-cozodb-streamer")]
   IngestFolderCreateSession { ... }
   ```

3. **Update documentation**
   - README.md
   - COMMANDS.md
   - .claude/agents/*.md

#### **Acceptance Criteria:**
- ✅ All commands follow 4-word convention
- ✅ Old commands still work (aliases)
- ✅ Documentation updated

---

### **Phase 3: Session Management Commands (v1.1.0)**

**Effort**: 2-3 hours
**Priority**: 🟢 MEDIUM

#### **Tasks:**

1. **Add session listing**
   ```bash
   parseltongue list-all-available-sessions
   ```

2. **Add session comparison**
   ```bash
   parseltongue compare-session-entity-count session1 session2
   ```

3. **Add session cleanup**
   ```bash
   parseltongue delete-old-session-artifacts session_name
   ```

---

### **Phase 4: Rename Crates (v1.2.0)**

**Effort**: 2 hours
**Priority**: 🟢 LOW (not breaking for users)

#### **Tasks:**

1. **Rename crate directories**
   - `parseltongue` → `parseltongue-main-cli-binary`
   - `parseltongue-core` → `parseltongue-core-entities-library`
   - `pt01-folder-to-cozodb-streamer` → `parseltongue-ingest-folder-streamer`
   - `pt02-llm-cozodb-to-context-writer` → `parseltongue-export-context-writer`

2. **Update Cargo.toml references**
3. **Update imports across codebase**

**Note**: This is internal refactoring, doesn't affect users

---

## 📊 **MIGRATION GUIDE (v1.0 → v1.1)**

### **Old Workflow (v1.0.0)**

```bash
# Scattered files, manual paths
parseltongue pt01-folder-to-cozodb-streamer . \
  --db "rocksdb:my_db.db"

parseltongue pt02-level01 \
  --include-code 1 \
  --where-clause "ALL" \
  --output results.json \
  --db "rocksdb:my_db.db"
```

### **New Workflow (v1.1.0)**

```bash
# Session-based, auto-organized
parseltongue ingest-folder-create-session .
# → Creates: .parseltongue/session_20251125_083045/

parseltongue export-level-one-entities
# → Exports to: .parseltongue/latest/exports/level_one_entities.json
```

### **Backwards Compatibility**

```bash
# Old commands still work via aliases
parseltongue pt01-folder-to-cozodb-streamer . \
  --db "rocksdb:custom.db"  # Still works!

# But new default is session-based
parseltongue ingest-folder-create-session .
# → Auto-creates session
```

---

## ✅ **SUCCESS METRICS**

### **UX Improvements**
- ✅ 0 manual path specifications needed (default session)
- ✅ 100% isolation (each run = new session)
- ✅ 1 folder to share with team (session folder)
- ✅ 100% 4-word naming compliance

### **Developer Experience**
- ✅ "It just works" - no configuration needed
- ✅ Clear folder structure (no confusion)
- ✅ Git-friendly (`.parseltongue/` in .gitignore)
- ✅ LLM-friendly (structured JSON + metadata)

---

## 🎯 **NEXT STEPS**

1. **Review this design** - Confirm approach
2. **Phase 1 implementation** - Session infrastructure
3. **Phase 2 implementation** - Rename commands
4. **Testing** - Verify all journeys work
5. **Documentation** - Update guides
6. **Release v1.1.0-alpha** - Beta testing

---

**Questions for Decision:**

1. ✅ Approve session-based architecture?
2. ✅ Approve 4-word naming for all commands?
3. ✅ Ship in v1.1.0 or v1.0.1?
4. ✅ Keep old command aliases for compatibility?

---

**Status**: 🎨 AWAITING APPROVAL
