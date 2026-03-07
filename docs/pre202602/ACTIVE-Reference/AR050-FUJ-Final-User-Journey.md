# v200 Final User Journey (FUJ)

**Status**: Canonical Draft v01
**Date**: 2026-02-21
**Philosophy**: Shreyas Doshi — every moment matters. Every loading state, every error, every micro-interaction is a product decision.
**Example Codebase Throughout**: Rust (Axum) backend + React (TypeScript) frontend — the archetypal polyglot app.
**Diagram Format**: Minto Pyramid — L1 (aggregate) → L2 (detailed) → L3 (implementation). All ASCII in ` ```text ``` ` blocks.

---

## Crate Architecture: Control Flow Spine

Every step in both journeys maps to this diagram. Read this first — it is the skeleton.

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  8 CRATES — CALL DIRECTION DURING INGEST                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  rust-llm-interface-gateway   (entry point: MCP / HTTP / Tauri / CLI)       │
│            │                                                                │
│            ├──────────────────────────────────────────────────────────┐    │
│            ▼                                                          ▼    │
│  rust-llm-tree-extractor             rust-llm-store-runtime               │
│  (file walk, language detect,        (single write path,                   │
│   tree-sitter parse, entity+edge     single getter contract,               │
│   extraction per file)               no direct queries in handlers)        │
│            │                                  ▲                            │
│            ├──────────────────┐               │ writes all                 │
│            ▼                  ▼               │ entities+edges             │
│  rust-llm-rust-semantics  rust-llm-cross-boundaries                        │
│  (rust-analyzer enrich,   (HTTP/FFI/WASM/queue                            │
│   typed call edges,        detection, confidence                           │
│   Rust files only)         scoring, all languages)                         │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  8 CRATES — CALL DIRECTION DURING QUERY                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  rust-llm-interface-gateway   (receives MCP tools/call or HTTP GET)         │
│            │                                                                │
│            ├──────────────────────────────┐                                │
│            ▼                              ▼                                │
│  rust-llm-store-runtime      rust-llm-graph-reasoning                      │
│  (single getter — all        (Datalog/Ascent rules:                        │
│   read paths go here,         blast-radius, SCC, dead code,               │
│   no exceptions)              layer violations, cycle detect)              │
│                                           │                                │
│                                           ▼                                │
│                               rust-llm-store-runtime                       │
│                               (reads base relations:                        │
│                                entity(), edge() facts                       │
│                                for Datalog evaluation)                      │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  DEFERRED TO V210 (not in scope):                                           │
│  rust-llm-context-packer   rust-llm-test-harness (internal only)           │
└─────────────────────────────────────────────────────────────────────────────┘
```

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  DEPENDENCY BUILD ORDER (compile-time)                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  rust-llm-core-foundation                                                   │
│       ↓                                                                     │
│  rust-llm-store-runtime                                                     │
│       ↓                                                                     │
│  rust-llm-rust-semantics   rust-llm-tree-extractor                          │
│                      ↘   ↙                                                 │
│               rust-llm-cross-boundaries                                     │
│                           ↓                                                 │
│               rust-llm-graph-reasoning                                      │
│                           ↓                                                 │
│               rust-llm-interface-gateway                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Section 1: MCP Client Journey

The user is inside Claude Desktop. They want their LLM to understand their codebase. Not grep. Not file dumps. A live, queryable graph.

---

### L1 — Three-Phase Overview

```text
┌─────────────────────────────────────────────────────────────────────┐
│            MCP CLIENT JOURNEY — LEVEL 1 OVERVIEW                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│   ┌─────────────┐      ┌─────────────┐      ┌─────────────┐        │
│   │  PHASE 1    │      │  PHASE 2    │      │  PHASE 3    │        │
│   │             │      │             │      │             │        │
│   │    SETUP    │─────▶│   INGEST    │─────▶│    QUERY    │        │
│   │             │      │             │      │             │        │
│   │ Config MCP  │      │ Parse code  │      │ Ask Claude  │        │
│   │ server once │      │ build graph │      │ get answers │        │
│   └─────────────┘      └─────────────┘      └─────────────┘        │
│        5 min                2-30 sec            Ongoing             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

### L2 — Phase 1: Setup (Detailed)

```text
┌─────────────────────────────────────────────────────────────────────┐
│                  PHASE 1: SETUP — LEVEL 2                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1.1 INSTALL          1.2 CONFIGURE         1.3 RESTART             │
│  ┌───────────┐        ┌───────────┐         ┌───────────┐           │
│  │ cargo     │        │ Edit      │         │ Claude    │           │
│  │ install   │───────▶│ claude_   │────────▶│ Desktop   │           │
│  │ rust-llm- │        │ desktop_  │         │ restarts  │           │
│  │ interface │        │ config    │         │           │           │
│  │ -gateway  │        │ .json     │         └─────┬─────┘           │
│  └───────────┘        └───────────┘               │                 │
│                                                   ▼                 │
│                                         1.4 HANDSHAKE               │
│                                         ┌───────────┐               │
│                                         │ JSON-RPC  │               │
│                                         │ initialize│               │
│                                         │ → tools/  │               │
│                                         │   list    │               │
│                                         └───────────┘               │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

### L3 — Phase 1: Setup (Implementation Detail)

#### 1.1 Install

User runs in terminal:
```bash
cargo install rust-llm-interface-gateway
```

Binary lands at `~/.cargo/bin/rust-llm-interface-gateway`. This binary is the single entry point — it dispatches to MCP mode, HTTP mode, or CLI mode via subcommand.

#### 1.2 Configure

**User Action**: Open `~/Library/Application Support/Claude/claude_desktop_config.json`

**Exact JSON they write**:
```json
{
  "mcpServers": {
    "parseltongue": {
      "command": "rust-llm-interface-gateway",
      "args": ["mcp-stdio-server-bridge"]
    }
  }
}
```

**What this means**: No `--db` flag yet — the server starts without a database. It will respond to `ingest_codebase` tool calls to create one. This is the zero-friction entry point.

**Error state**: If binary not found in PATH, Claude Desktop shows "Failed to start MCP server: parseltongue". User must check `$PATH` includes `~/.cargo/bin`.

#### 1.3 Restart

User quits Claude Desktop (Cmd+Q) and reopens.

**System actions on restart**:
1. Claude Desktop reads config JSON
2. Spawns subprocess: `rust-llm-interface-gateway mcp-stdio-server-bridge`
3. Opens two stdio pipes:
   - `stdin` pipe → sends JSON-RPC requests TO parseltongue
   - `stdout` pipe → reads JSON-RPC responses FROM parseltongue
4. All `stderr` from parseltongue → Claude Desktop's internal log (never shown to user)
5. **Critical constraint**: parseltongue MUST NEVER write anything to stdout except valid JSON-RPC frames. Any debug print to stdout corrupts the protocol.

#### 1.4 Handshake (Automatic — user sees nothing)

**Step 1 — Claude sends initialize**:
```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": { "roots": { "listChanged": true } },
    "clientInfo": { "name": "claude-desktop", "version": "0.7.0" }
  }
}
```

**Step 2 — Parseltongue responds with capabilities**:
```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": { "listChanged": false },
      "resources": { "subscribe": false, "listChanged": false }
    },
    "serverInfo": {
      "name": "parseltongue-rust-llm",
      "version": "2.0.0"
    }
  }
}
```

**Step 3 — Claude sends initialized notification** (no response expected):
```json
{
  "jsonrpc": "2.0",
  "method": "initialized"
}
```

**Step 4 — Claude auto-calls tools/list**:
```json
{ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }
```

**Parseltongue responds with all 16 tools**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "name": "ingest_codebase_from_path",
        "description": "Parse a local folder into the graph database. Returns workspace ID.",
        "inputSchema": {
          "type": "object",
          "properties": {
            "path": { "type": "string", "description": "Absolute path to project root" }
          },
          "required": ["path"]
        }
      },
      {
        "name": "search_entities_fuzzy_query",
        "description": "Fuzzy search entities by name across all languages. Returns matching EntityKeys with metadata.",
        "inputSchema": {
          "type": "object",
          "properties": {
            "query": { "type": "string" },
            "language": { "type": "string", "description": "Optional: rust | ts | js | go | py" }
          },
          "required": ["query"]
        }
      },
      {
        "name": "get_entity_detail_view",
        "description": "Full detail for one entity: source lines (live from disk), callers, callees, edges, token count.",
        "inputSchema": {
          "type": "object",
          "properties": {
            "key": { "type": "string", "description": "EntityKey in language|||kind|||scope|||name|||file_path||| format" }
          },
          "required": ["key"]
        }
      },
      {
        "name": "get_callers_reverse_graph",
        "description": "Who calls this entity? Returns direct callers with edge types.",
        "inputSchema": { "type": "object", "properties": { "key": { "type": "string" } }, "required": ["key"] }
      },
      {
        "name": "get_callees_forward_graph",
        "description": "What does this entity call? Returns direct callees with edge types.",
        "inputSchema": { "type": "object", "properties": { "key": { "type": "string" } }, "required": ["key"] }
      },
      {
        "name": "analyze_blast_radius_impact",
        "description": "N-hop transitive impact. Returns all entities affected if this one changes.",
        "inputSchema": {
          "type": "object",
          "properties": {
            "key": { "type": "string" },
            "hops": { "type": "integer", "default": 2, "minimum": 1, "maximum": 6 }
          },
          "required": ["key"]
        }
      },
      {
        "name": "list_cross_language_boundaries",
        "description": "All detected cross-language edges (HTTP, FFI, WASM, message queue) with confidence scores.",
        "inputSchema": {
          "type": "object",
          "properties": {
            "min_confidence": { "type": "number", "default": 0.60 }
          }
        }
      },
      {
        "name": "detect_circular_dependency_scan",
        "description": "Find all cycles in the dependency graph using Tarjan SCC.",
        "inputSchema": { "type": "object", "properties": {} }
      },
      {
        "name": "rank_complexity_hotspots_view",
        "description": "Top N entities by coupling (CBO) and cohesion (LCOM) metrics.",
        "inputSchema": {
          "type": "object",
          "properties": { "top": { "type": "integer", "default": 10 } }
        }
      },
      {
        "name": "get_folder_structure_tree",
        "description": "L1/L2 folder tree with entity counts per folder per language.",
        "inputSchema": { "type": "object", "properties": { "depth": { "type": "integer", "default": 2 } } }
      },
      {
        "name": "get_codebase_statistics_summary",
        "description": "Entity counts, edge counts, language breakdown, token budget estimates.",
        "inputSchema": { "type": "object", "properties": {} }
      },
      {
        "name": "get_rust_typed_call_edges",
        "description": "Rust-only. rust-analyzer enriched call edges: Direct | TraitMethod | DynDispatch | ClosureInvoke.",
        "inputSchema": { "type": "object", "properties": { "key": { "type": "string" } }, "required": ["key"] }
      },
      {
        "name": "get_rust_dataflow_analysis",
        "description": "Rust-only. Assign/param/return flow edges for a function.",
        "inputSchema": { "type": "object", "properties": { "key": { "type": "string" } }, "required": ["key"] }
      },
      {
        "name": "get_strongly_connected_components",
        "description": "Tarjan SCC membership. Which cycle does this entity belong to?",
        "inputSchema": { "type": "object", "properties": { "key": { "type": "string" } } }
      },
      {
        "name": "get_llm_context_ranked_budget",
        "description": "LLM-optimized context ranked by blast-radius, SCC, PageRank, entropy. Capped at token budget.",
        "inputSchema": {
          "type": "object",
          "properties": {
            "focus": { "type": "string", "description": "EntityKey to center context around" },
            "budget_tokens": { "type": "integer", "default": 8000 }
          },
          "required": ["focus"]
        }
      },
      {
        "name": "get_codebase_ingestion_status",
        "description": "Is a database loaded? What workspace? When was last ingest?",
        "inputSchema": { "type": "object", "properties": {} }
      }
    ]
  }
}
```

**UI state after handshake**: Claude Desktop shows a small tool icon in the sidebar. Parseltongue is ready. No database loaded yet — the user must trigger ingestion.

---

### L2 — Phase 2: Ingest (Detailed)

```text
┌─────────────────────────────────────────────────────────────────────┐
│                  PHASE 2: INGEST — LEVEL 2                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  2.1 USER TRIGGERS    2.2 FILE WALK        2.3 TREE-SITTER          │
│  ┌───────────────┐    ┌───────────────┐    ┌───────────────┐        │
│  │ User tells    │    │ Walk all files │    │ Parse each    │        │
│  │ Claude: ingest│───▶│ detect lang   │───▶│ file per lang │        │
│  │ /path/to/repo │    │ by extension  │    │ extract nodes │        │
│  └───────────────┘    └───────────────┘    └──────┬────────┘        │
│                                                   │                 │
│                    ┌──────────────────────────────┘                 │
│                    ▼                                                 │
│  2.4 RUST-ANALYZER  2.5 CROSS-LANG         2.6 EDGES + STORE        │
│  ┌───────────────┐  ┌───────────────┐    ┌───────────────┐          │
│  │ Semantic enr. │  │ HTTP/FFI/WASM │    │ Emit edges    │          │
│  │ for .rs files │─▶│ boundary scan │───▶│ assign keys   │          │
│  │ typed edges   │  │ confidence    │    │ write storage │          │
│  └───────────────┘  └───────────────┘    └───────────────┘          │
│   (Rust-only)        (all languages)                                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

### L3 — Phase 2: Ingest (Implementation Detail)

#### 2.1 User Triggers Ingest

**User types in Claude chat**:
> "Ingest my project at /Users/dev/myapp"

**Claude calls MCP tool**:
```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "tools/call",
  "params": {
    "name": "ingest_codebase_from_path",
    "arguments": { "path": "/Users/dev/myapp" }
  }
}
```

**Parseltongue begins — user has to wait**. A good MCP server streams progress via `notifications/progress` but V200 starts with a blocking response. The response comes when ingest is complete.

#### 2.2 File Walk

```text
┌─────────────────────────────────────────────────────────────┐
│  FILE WALK — What gets scanned                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  /Users/dev/myapp/                                          │
│  ├── backend/                                               │
│  │   ├── src/                                               │
│  │   │   ├── main.rs          → rust                        │
│  │   │   ├── api/auth.rs      → rust                        │
│  │   │   ├── models/user.rs   → rust                        │
│  │   │   └── db/queries.rs    → rust                        │
│  │   └── Cargo.toml           → project manifest (Rust)     │
│  ├── frontend/                                              │
│  │   ├── src/                                               │
│  │   │   ├── App.tsx          → typescript                  │
│  │   │   ├── api/auth.ts      → typescript                  │
│  │   │   ├── components/      → typescript                  │
│  │   │   │   ├── LoginForm.tsx                              │
│  │   │   │   └── Navbar.tsx                                 │
│  │   │   └── utils/shared.js  → javascript                  │
│  │   └── package.json         → project manifest (JS)       │
│  └── .parseltongue-ignore     → (optional skip rules)       │
│                                                             │
│  SKIP: node_modules/, target/, .git/, *.lock               │
│                                                             │
│  RESULT: 47 source files across 3 languages                 │
└─────────────────────────────────────────────────────────────┘
```

**Extension → Language mapping** (rust-llm-tree-extractor):
```
.rs           → rust
.ts           → typescript
.tsx          → typescript (tsx variant)
.js           → javascript
.jsx          → javascript (jsx variant)
.py           → python
.go           → go
.java         → java
.c / .h       → c
.cpp / .hpp   → cpp
.rb           → ruby
.cs           → csharp
```

#### 2.3 Tree-sitter Extraction (Per File)

For **every file**, rust-llm-tree-extractor does:

**Step 1 — Create parser for language**:
```rust
let mut parser = tree_sitter::Parser::new();
parser.set_language(language_for_extension(&ext))?;
```

**Step 2 — Parse source bytes**:
```rust
let source_bytes = fs::read(&file_path)?;
let tree = parser.parse(&source_bytes, None)?;
// tree is a CST (Concrete Syntax Tree)
```

**Step 3 — Run extraction queries**

For `.rs` files — **function extraction**:
```scheme
(function_item
  name: (identifier) @fn.name
  parameters: (parameters
    (parameter
      pattern: (identifier) @fn.param.name
      type: (_) @fn.param.type)*) @fn.params
  return_type: (_)? @fn.return_type
  body: (block) @fn.body) @fn.def
```

For `.rs` files — **struct extraction**:
```scheme
(struct_item
  name: (type_identifier) @struct.name
  body: (field_declaration_list
    (field_declaration
      name: (field_identifier) @field.name
      type: (_) @field.type)*)) @struct.def
```

For `.rs` files — **impl block extraction**:
```scheme
(impl_item
  trait: (type_identifier)? @impl.trait
  type: (type_identifier) @impl.type
  body: (declaration_list
    (function_item name: (identifier) @method.name) @method.def)*) @impl.def
```

For `.rs` files — **use/import extraction**:
```scheme
(use_declaration
  argument: (_) @use.path) @use.stmt
```

For `.tsx` files — **function component extraction**:
```scheme
(function_declaration
  name: (identifier) @fn.name
  parameters: (formal_parameters) @fn.params
  return_type: (type_annotation)? @fn.return_type) @fn.def

(variable_statement
  (variable_declaration_list
    (variable_declarator
      name: (identifier) @fn.name
      type: (type_annotation)? @fn.type
      value: [(arrow_function) (function)] @fn.def)))
```

For `.tsx` files — **import extraction**:
```scheme
(import_statement
  (import_clause
    (named_imports
      (import_specifier name: (identifier) @import.name)*))
  source: (string) @import.source) @import.stmt
```

For `.tsx` files — **interface/type extraction**:
```scheme
(interface_declaration
  name: (type_identifier) @iface.name
  body: (object_type
    (property_signature
      name: (property_identifier) @prop.name
      type: (type_annotation) @prop.type)*)) @iface.def

(type_alias_declaration
  name: (type_identifier) @type.name
  value: (_) @type.value) @type.def
```

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: extract_entities_from_file()                                     │
│  CRATE: rust-llm-tree-extractor                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  FileBytes { path: PathBuf, content: Vec<u8>, language: Language }│
│  DATA OUT: ExtractionResult { entities: Vec<Entity>, raw_calls: Vec<RawCall>}│
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - content is valid UTF-8 (or lossy-converted)                              │
│  - language is a supported Language enum variant                            │
│  - tree-sitter grammar is loaded for language                               │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN file has N function nodes THEN entities contains N Entity records   │
│  - WHEN entity is declared with pub/export THEN entity.is_public = true     │
│  - WHEN entity is private/unexported THEN entity.is_public = false          │
│  - WHEN file has 0 parseable nodes THEN returns empty Vec (not error)       │
│  - SHALL set token_count = char_count / 4 for each entity                  │
│  - SHALL NOT set start_line == end_line for multi-line entities             │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - ExtractionError::ParseFailed if tree-sitter returns error node at root  │
│  - ExtractionError::UnsupportedLanguage if no grammar registered           │
│  - ExtractionError::EmptyFile returns Ok(empty) — not an error             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Step 4 — Build EntityKey for each extracted node**

Format: `language|||kind|||scope|||name|||file_path|||discriminator`

```rust
fn build_entity_key(
    language: &str,
    kind: &str,
    scope: &str,          // module path: "backend::api::auth"
    name: &str,
    file_path: &str,      // normalized: "backend/src/api/auth.rs"
    discriminator: &str,  // "" for unique, "u64,String" for overloads
) -> EntityKey {
    EntityKey(format!(
        "{}|||{}|||{}|||{}|||{}|||{}",
        language, kind, scope, name, file_path, discriminator
    ))
}

// Examples from our Rust+React codebase:
// rust|||fn|||backend::api::auth|||handle_auth_request|||backend/src/api/auth.rs|||
// ts|||fn|||frontend.api|||fetchAuthData|||frontend/src/api/auth.ts|||
// ts|||interface|||frontend.types|||User|||frontend/src/types.ts|||
// rust|||struct|||backend::models|||User|||backend/src/models/user.rs|||
// ts|||component|||frontend.components|||LoginForm|||frontend/src/components/LoginForm.tsx|||
```

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: build_entity_key_canonical()                                     │
│  CRATE: rust-llm-tree-extractor                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  KeyParts { language, kind, scope, name, file_path, discriminator }│
│  DATA OUT: EntityKey(String)                                                │
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - file_path is a valid UTF-8 path string                                   │
│  - name is non-empty                                                        │
│  - language is a known Language variant                                     │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN path is ./foo/bar.rs THEN key contains foo/bar.rs (no ./ prefix)   │
│  - WHEN path is /abs/root/foo/bar.rs THEN key contains foo/bar.rs (relative)│
│  - WHEN path is foo/bar.rs THEN key contains foo/bar.rs (unchanged)        │
│  - SHALL NOT include line numbers in key (Gate G1)                         │
│  - WHEN two overloaded fns exist THEN discriminators differ                 │
│  - WHEN scope extraction fails THEN scope = "" with capability_marker set  │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - KeyError::EmptyName if name is empty string                             │
│  - KeyError::InvalidLanguage if language string unrecognized               │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Path normalization rule** (Gate G4): `./backend/src/api.rs`, `backend/src/api.rs`, and `/Users/dev/myapp/backend/src/api.rs` all canonicalize to `backend/src/api.rs` (relative to project root). Same entity regardless of how path was expressed.

**Step 5 — Emit shared-context edges (visibility-aware)**

Every pair of entities in the same file gets a `shared_context` edge. But not all shared-context relationships are equal. The key insight:

> **If two entities are in the same file AND both are public (exported), they share module-level semantic context.** The file is a module boundary. Public entities collectively define what that module does. A public import statement and a public function in the same file are almost certainly semantically related — the import exists *because* the function needs it.

```text
┌────────────────────────────────────────────────────────────────────┐
│  SHARED CONTEXT — TWO TIERS                                        │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Tier 1: shared_context (any two entities, same file)              │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  backend/src/api/auth.rs                                     │  │
│  │  ┌──────────────────┐     shared_context     ┌───────────┐   │  │
│  │  │ pub fn           │ ─────────────────────▶ │ pub fn    │   │  │
│  │  │ handle_auth_req  │                         │ health_   │   │  │
│  │  └──────────────────┘                         │ check     │   │  │
│  │                           confidence: 1.0     └───────────┘   │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                    │
│  Tier 2: public_module_context (BOTH entities are public)          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  frontend/src/api/auth.ts                                    │  │
│  │                                                              │  │
│  │  import { User } from '../types'     ← public import        │  │
│  │  export interface AuthResponse { ... } ← public interface   │  │
│  │  export function fetchAuthData() { ... } ← public fn        │  │
│  │                                                              │  │
│  │  ┌─────────────────┐  public_module_context  ┌───────────┐  │  │
│  │  │ import User     │ ──────────────────────▶  │ export fn │  │  │
│  │  └─────────────────┘                          │ fetchAuth │  │  │
│  │                                               └───────────┘  │  │
│  │  ┌──────────────────┐  public_module_context  ┌───────────┐  │  │
│  │  │ export interface │ ──────────────────────▶  │ export fn │  │  │
│  │  │ AuthResponse     │  (interface defines the  │ fetchAuth │  │  │
│  │  └──────────────────┘   return type of fn)     └───────────┘  │  │
│  │                           confidence: 1.0, public: true       │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                    │
│  WHY THIS MATTERS:                                                 │
│  • public_module_context surfaces the module's API surface         │
│  • LLM context: "give me all public entities co-located with X"   │
│    = give me the module X lives in, instantly                      │
│  • Blast radius through public_module_context = module-level impact│
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

**Visibility detection via tree-sitter** (runs during entity extraction):

For `.rs` files — detect `pub` visibility:
```scheme
; Rust: public function
(function_item
  (visibility_modifier) @vis (#eq? @vis "pub")
  name: (identifier) @fn.name) @public.fn

; Rust: public struct
(struct_item
  (visibility_modifier) @vis (#eq? @vis "pub")
  name: (type_identifier) @struct.name) @public.struct

; Rust: public use (re-export)
(use_declaration
  (visibility_modifier) @vis (#eq? @vis "pub")
  argument: (_) @use.path) @public.use

; Rust: pub(crate) — still module-public, mark as crate_public
(function_item
  (visibility_modifier "pub" "(" "crate" ")") @vis
  name: (identifier) @fn.name) @crate.fn
```

For `.ts` / `.tsx` files — detect `export` visibility:
```scheme
; TypeScript: export function
(export_statement
  declaration: (function_declaration
    name: (identifier) @fn.name)) @public.fn

; TypeScript: export interface
(export_statement
  declaration: (interface_declaration
    name: (type_identifier) @iface.name)) @public.interface

; TypeScript: export const arrow
(export_statement
  declaration: (lexical_declaration
    (variable_declarator
      name: (identifier) @fn.name
      value: (arrow_function)))) @public.fn

; TypeScript: import statement (all imports are "public to this file")
(import_statement
  source: (string) @import.source) @file.import
```

**Edge emission logic** (two tiers):

```rust
for file_entities in entities_by_file.values() {
    let file_path = &file_entities[0].file;

    for i in 0..file_entities.len() {
        for j in (i+1)..file_entities.len() {
            let a = &file_entities[i];
            let b = &file_entities[j];

            let both_public = a.is_public && b.is_public;

            // Tier 1: always emit shared_context
            emit_edge(Edge {
                from: a.key.clone(),
                to:   b.key.clone(),
                kind: EdgeKind::SharedContext,
                confidence: 1.0,
                metadata: json!({
                    "file": file_path,
                    "public": both_public,
                }),
            });

            // Tier 2: additionally emit public_module_context if both public
            // This is a SEPARATE edge — queryable independently
            if both_public {
                emit_edge(Edge {
                    from: a.key.clone(),
                    to:   b.key.clone(),
                    kind: EdgeKind::PublicModuleContext,
                    confidence: 1.0,
                    metadata: json!({
                        "file": file_path,
                        "a_kind": a.kind,
                        "b_kind": b.kind,
                        // Special case: import + fn → import is a dependency of fn
                        "import_feeds_fn": a.kind == "import" || b.kind == "import",
                    }),
                });
            }
        }
    }
}
```

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: detect_entity_visibility_flag()                                  │
│  CRATE: rust-llm-tree-extractor                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  VisibilityNode { node: tree_sitter::Node, language: Language }   │
│  DATA OUT: bool  (true = public/exported)                                   │
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - node is a valid tree-sitter node from a parsed CST                       │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN Rust node has (visibility_modifier) child = "pub" THEN true         │
│  - WHEN Rust node has "pub(crate)" THEN true (crate-public, still public)   │
│  - WHEN Rust node has no visibility_modifier THEN false                     │
│  - WHEN TS/JS node is wrapped in export_statement THEN true                 │
│  - WHEN TS/JS node is import_statement THEN true (public to file)          │
│  - WHEN TS/JS node has no export wrapper THEN false                         │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - Returns false (not error) for unknown node types — safe default          │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: emit_shared_context_pair()                                       │
│  CRATE: rust-llm-tree-extractor                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  (Entity, Entity)  — any two entities from same file              │
│  DATA OUT: Edge { kind: EdgeKind::SharedContext, confidence: 1.0 }          │
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - Both entities have same file_path in their EntityKey                     │
│  - Both entities are already assigned valid EntityKeys                      │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN file has N entities THEN SHALL emit N*(N-1)/2 shared_context edges  │
│  - SHALL always set confidence = 1.0 (no uncertainty — same file is fact)  │
│  - SHALL set metadata.file = normalized file_path                           │
│  - SHALL set metadata.public = (a.is_public && b.is_public)                 │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - EdgeError::DifferentFiles if entities have different file_paths          │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: emit_public_module_context()                                     │
│  CRATE: rust-llm-tree-extractor                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  (Entity, Entity)  — both must have is_public = true              │
│  DATA OUT: Edge { kind: EdgeKind::PublicModuleContext, confidence: 1.0 }    │
│            OR None if either entity is private                              │
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - Both entities have same file_path                                        │
│  - detect_entity_visibility_flag() has run on both (is_public is set)       │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN both.is_public = true THEN SHALL emit PublicModuleContext edge      │
│  - WHEN either.is_public = false THEN SHALL NOT emit (returns None)         │
│  - WHEN a.kind = "import" OR b.kind = "import" THEN                         │
│      metadata.import_feeds_fn = true                                        │
│  - WHEN a.kind = "interface" AND b.kind = "fn" THEN                         │
│      metadata.interface_contract = true                                     │
│  - SHALL be emitted IN ADDITION TO shared_context (not instead of)          │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - Returns None if either entity.is_public is absent/unset (safe default)  │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Why `import + public fn` is a special case**: An import statement at the top of `auth.ts` is not just "near" `fetchAuthData` — it *enables* it. The import `{ User }` from `'../types'` exists because `fetchAuthData` returns or accepts a `User`. This makes `public_module_context` edges between imports and co-located public functions particularly high-signal for blast radius and LLM context selection.

**Step 6 — Emit call edges (within-file basic)**

```scheme
; Rust call detection
(call_expression
  function: [(identifier) @callee
             (field_expression field: (field_identifier) @callee)
             (scoped_identifier name: (identifier) @callee)] @call)

; TypeScript/JS call detection
(call_expression
  function: [(identifier) @callee
             (member_expression property: (property_identifier) @callee)] @call)
```

When a call target name matches an entity name in the same file → `calls` edge with confidence 0.95.
When a call target name matches an entity in a different file (via import graph) → `calls` edge with confidence 0.75 (lower because import resolution is heuristic without type system).

**Step 7 — Emit import edges**

```rust
// "import { fetchAuthData } from '../api/auth'"
// → ts|||fn|||frontend.api|||fetchAuthData|||frontend/src/api/auth.ts|||
// gets an `imports` edge from the importing file's module entity
```

**Step 8 — Compute token_count per entity (Gate R7)**

```rust
fn compute_token_count(source_lines: &[&str]) -> u32 {
    // Approximate: 1 token ≈ 4 chars for code
    let char_count: usize = source_lines.iter().map(|l| l.len()).sum();
    (char_count / 4) as u32
}
// Stored on entity. Used for LLM budget allocation in get_llm_context_ranked_budget.
```

#### 2.4 Rust-Analyzer Semantic Enrichment (Rust files only)

After tree-sitter completes, rust-llm-rust-semantics runs rust-analyzer on the Rust workspace.

**Step 1 — Load Cargo workspace**:
```rust
let workspace = ProjectWorkspace::load(
    ProjectManifest::from_manifest_file(cargo_toml_path)?,
    &CargoConfig::default(),
    &|progress| eprintln!("ra loading: {progress}"), // stderr only!
)?;
```

**Step 2 — Create analysis host + apply change**:
```rust
let (host, vfs, _proc_macro_srv) = load_workspace_at(
    &cargo_toml_path,
    &CargoConfig::default(),
    &AnalysisHostConfig::default(),
    &|msg| eprintln!("{msg}"),
)?;
let db = host.raw_database();
```

**Step 3 — For each Rust function, get typed call edges**:
```rust
for fn_def in all_rust_functions {
    // Get the hir::Function
    let fn_id = find_fn_in_hir(db, &fn_def.key)?;

    // Get all expressions in function body
    let body = db.body(fn_id.into());
    let body_source = db.body_with_source_map(fn_id.into());

    for (expr_id, expr) in body.exprs.iter() {
        if let Expr::Call { callee, args } = expr {
            // Resolve callee type
            let callee_ty = db.infer(fn_id.into())[*callee].clone();

            let edge_kind = match callee_ty.kind(Interner) {
                TyKind::FnDef(..) => TypedCallKind::Direct,
                TyKind::Dyn(..)   => TypedCallKind::DynDispatch,
                TyKind::Closure(..,..) => TypedCallKind::ClosureInvoke,
                _ => TypedCallKind::Direct,
            };

            // Check if it's a trait method
            let edge_kind = if is_trait_method_call(db, &callee_ty) {
                TypedCallKind::TraitMethod
            } else {
                edge_kind
            };

            emit_typed_edge(fn_def.key.clone(), resolved_callee_key, edge_kind);
        }
    }
}
```

**Step 4 — Trait implementation resolution**:
```rust
// For each trait impl block found by tree-sitter:
// rust|||impl|||backend::models|||User:Display|||backend/src/models/user.rs|||
// emit an `implements` edge:
// rust|||struct|||backend::models|||User → rust|||trait|||std::fmt|||Display
```

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: enrich_rust_entity_typed()                                       │
│  CRATE: rust-llm-rust-semantics                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  RustEnrichInput { key: EntityKey, fn_id: hir::FunctionId,        │
│                              db: &dyn HirDatabase }                         │
│  DATA OUT: Vec<TypedCallEdge>                                               │
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - entity is rust kind = "fn" (only valid input)                            │
│  - rust-analyzer workspace loaded without fatal errors                      │
│  - fn_id resolves in the HIR database                                       │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN callee resolves to FnDef THEN edge.kind = TypedCallKind::Direct     │
│  - WHEN callee resolves through dyn Trait THEN edge.kind = DynDispatch      │
│  - WHEN callee is a Closure invocation THEN edge.kind = ClosureInvoke       │
│  - WHEN callee is trait method on concrete type THEN edge.kind = TraitMethod│
│  - WHEN function body has 0 calls THEN returns empty Vec (not error)        │
│  - SHALL emit all edges for entire body, not just first call                │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - EnrichError::NotARustFn if entity.language != Rust                       │
│  - EnrichError::HirResolutionFailed emits degrade annotation, returns []   │
│    (never propagates as hard error — partial enrichment is acceptable)      │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: resolve_trait_impl_target()                                      │
│  CRATE: rust-llm-rust-semantics                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  ImplBlock { impl_key: EntityKey, db: &dyn HirDatabase }          │
│  DATA OUT: Option<ImplEdge { struct_key: EntityKey, trait_key: EntityKey }> │
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - impl_key is an impl entity (kind = "impl")                               │
│  - impl block has a trait: field (impl Trait for Type, not bare impl)       │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN impl Foo for Bar THEN returns Some(struct=Bar, trait=Foo)           │
│  - WHEN impl Bar (no trait) THEN returns None (not an error)                │
│  - SHALL emit edge kind = EdgeKind::Implements                              │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - Returns None if trait cannot be resolved (safe default, no panic)        │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Step 5 — Proc-macro and build-script handling**:

If a crate uses proc-macros and rust-analyzer cannot expand them, emit an explicit degrade annotation:
```json
{
  "entity": "rust|||macro|||backend|||my_derive|||backend/src/lib.rs|||",
  "degrade_reason": "proc_macro_expansion_failed",
  "capability_marker": "ra_enrichment=partial"
}
```

#### 2.5 Cross-Language Boundary Detection

rust-llm-cross-boundaries scans for edges that cross language boundaries.

**HTTP boundary detection** (Rust Axum server → React fetch):

**Rust side — Axum route signals**:
```scheme
; tree-sitter pattern for Axum route registration
(call_expression
  function: (field_expression
    field: (field_identifier) @method
    (#match? @method "^(get|post|put|delete|patch|route)$"))
  arguments: (arguments
    (string_literal) @route.path
    (_) @handler.fn)) @route.registration
```
Signal: `POST /api/auth` bound to `handle_auth_request`

**TypeScript side — fetch signals**:
```scheme
; tree-sitter pattern for fetch/axios calls
(call_expression
  function: (identifier) @fn (#eq? @fn "fetch")
  arguments: (arguments
    (string_literal) @url)) @fetch.call

(call_expression
  function: (member_expression
    object: (identifier) @obj (#match? @obj "^(axios|http|api)$")
    property: (property_identifier) @method)) @axios.call
```
Signal: `fetch('/api/auth', { method: 'POST' })` in `frontend/src/api/auth.ts`

**Confidence scoring**:
```text
Route path match (exact):           +0.40
HTTP method match:                  +0.20
Handler name similarity:            +0.15
Framework detection bonus:          +0.10
  (Axum detected + fetch/axios)
Same repo bonus:                    +0.05
                                   ──────
Total confidence:                    0.90  → HIGH (≥0.80, include)

Edge emitted:
  from: rust|||fn|||backend::api::auth|||handle_auth_request|||backend/src/api/auth.rs|||
  to:   ts|||fn|||frontend.api|||fetchAuthData|||frontend/src/api/auth.ts|||
  kind: http_boundary
  confidence: 0.90
  metadata: { "route": "POST /api/auth", "framework": "axum+fetch" }
```

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: detect_http_boundary_edge()                                      │
│  CRATE: rust-llm-cross-boundaries                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  BoundaryCandidate { server_signal: RouteSignal,                  │
│                                client_signal: FetchSignal }                 │
│  DATA OUT: Option<Edge<HttpBoundary>>                                       │
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - RouteSignal has: route_path, http_method, handler_entity_key             │
│  - FetchSignal has: url_pattern, http_method, caller_entity_key             │
│  - Both signals extracted from tree-sitter parse of respective files        │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN score_boundary_confidence_signal() >= 0.80 THEN                     │
│      returns Some(Edge { confidence, uncertain: false })                    │
│  - WHEN score >= 0.60 AND < 0.80 THEN                                       │
│      returns Some(Edge { confidence, uncertain: true })                     │
│  - WHEN score >= 0.40 AND < 0.60 THEN returns None (opt-in threshold)      │
│  - WHEN score < 0.40 THEN returns None (rejected)                           │
│  - SHALL set edge.kind = EdgeKind::HttpBoundary                             │
│  - SHALL set metadata.route = route_path, metadata.method = http_method    │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - Returns None on any signal extraction failure (never hard errors)        │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: score_boundary_confidence_signal()                               │
│  CRATE: rust-llm-cross-boundaries                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  Vec<ConfidenceSignal>  (each signal has type + score delta)      │
│  DATA OUT: f32  (range 0.0–1.0, clamped)                                   │
│                                                                             │
│  SIGNAL TABLE:                                                              │
│  - RoutePathMatch::Exact       → +0.40                                      │
│  - HttpMethodMatch             → +0.20                                      │
│  - HandlerNameSimilarity       → +0.15                                      │
│  - FrameworkDetectionBonus     → +0.10  (Axum+fetch, Express+axios, etc.)  │
│  - SameRepoBonus               → +0.05                                      │
│  - RoutePathMatch::Fuzzy       → +0.20  (partial path overlap)             │
│  - MismatchedMethod            → -0.10                                      │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN no signals provided THEN returns 0.0                                │
│  - SHALL clamp output to [0.0, 1.0]                                         │
│  - SHALL sum all provided signal deltas before clamping                     │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - No errors — pure function, always returns f32                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Confidence thresholds**:
```text
≥ 0.80 → HIGH confidence  → include by default
0.60–0.79 → MEDIUM       → include with uncertain: true marker
0.40–0.59 → LOW          → opt-in only (user must request)
< 0.40    → REJECT       → not stored
```

#### 2.6 Write to Storage

All entities and edges write to rust-llm-store-runtime.

**Single getter contract (Gate G2)**: Every read path — MCP tool, HTTP endpoint, CLI — goes through one canonical read function. No direct storage queries in handler code.

**Ingest complete response**:
```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "result": {
    "content": [{
      "type": "text",
      "text": "Ingest complete.\n\nWorkspace: myapp-20260221-143022\n\nEntities:\n  rust: 312 entities\n  typescript: 445 entities\n  javascript: 89 entities\n  Total: 846 entities\n\nEdges:\n  calls: 1,203\n  imports: 234\n  shared_context: 3,891\n  implements: 67\n  http_boundary: 23 (high confidence)\n  http_boundary: 8 (medium confidence, uncertain=true)\n\nRust semantic enrichment:\n  typed_calls::Direct: 456\n  typed_calls::TraitMethod: 89\n  typed_calls::DynDispatch: 12\n  typed_calls::ClosureInvoke: 34\n\nToken budget estimate:\n  Total entity tokens: ~48,000\n  vs raw file dump: ~890,000 tokens\n  Savings: 94.6%"
    }]
  }
}
```

**Claude shows this to user in chat.** User sees the stats. They now know the graph is ready.

---

### L2 — Phase 3: Query (Detailed)

```text
┌─────────────────────────────────────────────────────────────────────┐
│                  PHASE 3: QUERY — LEVEL 2                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  3.1 USER ASKS        3.2 CLAUDE PICKS      3.3 MCP CALL           │
│  ┌───────────────┐    ┌───────────────┐    ┌───────────────┐        │
│  │ Natural lang  │    │ Claude selects│    │ JSON-RPC      │        │
│  │ question in   │───▶│ which tool(s) │───▶│ tools/call    │        │
│  │ Claude chat   │    │ to call       │    │ to parseltongue│        │
│  └───────────────┘    └───────────────┘    └──────┬────────┘        │
│                                                   │                 │
│                    ┌──────────────────────────────┘                 │
│                    ▼                                                 │
│  3.4 STORAGE QUERY  3.5 RESULTS BACK       3.6 CLAUDE FORMATS       │
│  ┌───────────────┐  ┌───────────────┐    ┌───────────────┐          │
│  │ rust-llm-     │  │ JSON through  │    │ Claude answers│          │
│  │ store-runtime │─▶│ stdio back to │───▶│ in natural    │          │
│  │ single getter │  │ Claude        │    │ language      │          │
│  └───────────────┘  └───────────────┘    └───────────────┘          │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

### L3 — Phase 3: Query (Implementation Detail)

#### Query 1: "Search for functions with 'auth' in the name"

**User types**: "Find all auth-related functions"

**Claude calls**:
```json
{
  "jsonrpc": "2.0", "id": 20, "method": "tools/call",
  "params": {
    "name": "search_entities_fuzzy_query",
    "arguments": { "query": "auth" }
  }
}
```

**Parseltongue storage query** (rust-llm-store-runtime single getter):
```rust
fn get_entities_fuzzy(query: &str) -> Vec<EntitySummary> {
    // Trigram index scan on entity name field
    // Returns entities where name contains query as substring (case-insensitive)
    // Ranked by: exact match > prefix match > contains match
}
```

**Response**:
```json
{
  "jsonrpc": "2.0", "id": 20,
  "result": {
    "content": [{
      "type": "text",
      "text": "<entities>\n  <entity key=\"rust|||fn|||backend::api::auth|||handle_auth_request|||backend/src/api/auth.rs|||\" lang=\"rust\" kind=\"fn\" token_count=\"89\"/>\n  <entity key=\"rust|||fn|||backend::middleware|||auth_middleware|||backend/src/middleware.rs|||\" lang=\"rust\" kind=\"fn\" token_count=\"45\"/>\n  <entity key=\"ts|||fn|||frontend.api|||fetchAuthData|||frontend/src/api/auth.ts|||\" lang=\"ts\" kind=\"fn\" token_count=\"34\"/>\n  <entity key=\"ts|||component|||frontend.components|||AuthGuard|||frontend/src/components/AuthGuard.tsx|||\" lang=\"ts\" kind=\"component\" token_count=\"67\"/>\n  <entity key=\"ts|||fn|||frontend.hooks|||useAuth|||frontend/src/hooks/useAuth.ts|||\" lang=\"ts\" kind=\"fn\" token_count=\"112\"/>\n</entities>"
    }]
  }
}
```

**Note**: Responses use XML-tagged grouping (Gate R4) — deterministic semantic structure for LLM parsing. Claude can parse `<entities>` reliably.

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: get_entities_fuzzy_query()                                       │
│  CRATE: rust-llm-store-runtime                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  FuzzyQueryInput { query: String, max_results: Option<u32> }      │
│  DATA OUT: Vec<EntitySummary>                                               │
│            EntitySummary { key: EntityKey, kind: EntityKind,                │
│                            lang: Language, token_count: u32 }               │
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - query.len() >= 1 (empty string rejected)                                 │
│  - Store must be open (IngestionStatus::Ready)                              │
│  - Trigram index must exist for entity name field                           │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN query matches any entity name substring THEN returns Vec (non-empty)│
│  - WHEN no matches THEN returns Vec::new() (never an error)                 │
│  - SHALL rank: exact match > prefix match > contains match                  │
│  - SHALL be case-insensitive                                                │
│  - SHALL NOT return more than max_results (default: 200)                    │
│  - SHALL cross all languages in a single pass                               │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - StoreNotReady → returns Err(QueryError::StoreNotReady)                   │
│  - QueryTooShort (len 0) → returns Err(QueryError::EmptyQuery)              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

#### Query 2: "What React components call the Rust auth endpoint?"

**Claude calls** (two tool calls, potentially in sequence):

**First**: `list_cross_language_boundaries` to find HTTP boundary edges
```json
{ "name": "list_cross_language_boundaries", "arguments": { "min_confidence": 0.60 } }
```

**Response**: Returns the `http_boundary` edge:
```json
{
  "content": [{
    "type": "text",
    "text": "<boundaries>\n  <edge kind=\"http_boundary\" confidence=\"0.90\"\n    from=\"rust|||fn|||backend::api::auth|||handle_auth_request|||backend/src/api/auth.rs|||\"\n    to=\"ts|||fn|||frontend.api|||fetchAuthData|||frontend/src/api/auth.ts|||\"\n    route=\"POST /api/auth\" framework=\"axum+fetch\"/>\n  ...\n</boundaries>"
  }]
}
```

**Second**: `get_callers_reverse_graph` on `fetchAuthData` to see what React calls it
```json
{
  "name": "get_callers_reverse_graph",
  "arguments": { "key": "ts|||fn|||frontend.api|||fetchAuthData|||frontend/src/api/auth.ts|||" }
}
```

**Response**:
```json
{
  "content": [{
    "type": "text",
    "text": "<callers>\n  <caller key=\"ts|||component|||frontend.components|||LoginForm|||frontend/src/components/LoginForm.tsx|||\" edge_kind=\"calls\" confidence=\"0.95\"/>\n  <caller key=\"ts|||fn|||frontend.hooks|||useAuth|||frontend/src/hooks/useAuth.ts|||\" edge_kind=\"calls\" confidence=\"0.95\"/>\n</callers>"
  }]
}
```

Claude synthesizes both responses and tells the user: "Two React components call the Rust auth endpoint: `LoginForm` directly, and `useAuth` hook which is used by multiple components."

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: list_cross_language_all()                                        │
│  CRATE: rust-llm-store-runtime                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  CrossLangQuery { min_confidence: f32 }                           │
│  DATA OUT: Vec<CrossLangEdge>                                               │
│            CrossLangEdge { from: EntityKey, to: EntityKey,                  │
│                            kind: EdgeKind, confidence: f32,                 │
│                            uncertain: bool,                                 │
│                            metadata: serde_json::Value }                    │
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - min_confidence in range [0.0, 1.0]                                       │
│  - Store must be open and ingest complete                                   │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN min_confidence = 0.60 THEN returns high + medium edges              │
│  - WHEN min_confidence = 0.80 THEN returns high edges only                  │
│  - SHALL set uncertain=true on all edges with confidence in [0.60, 0.80)   │
│  - SHALL include edge kinds: http_boundary, ffi_boundary,                   │
│      wasm_boundary, pyo3_boundary, queue_boundary                           │
│  - SHALL NOT include same-language edges                                    │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - InvalidConfidence (< 0.0 or > 1.0) → Err(QueryError::InvalidParam)      │
│  - StoreNotReady → Err(QueryError::StoreNotReady)                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

#### Query 3: "If I change handle_auth_request, what breaks?"

**Claude calls**: `analyze_blast_radius_impact` with hops=3
```json
{
  "name": "analyze_blast_radius_impact",
  "arguments": {
    "key": "rust|||fn|||backend::api::auth|||handle_auth_request|||backend/src/api/auth.rs|||",
    "hops": 3
  }
}
```

**Parseltongue runs Datalog transitive reachability rule**:
```rust
// Ascent Datalog rule (rust-llm-graph-reasoning):
ascent! {
    relation reachable(EntityKey, EntityKey, u32); // (from, to, hop_count)

    reachable(a, b, 1) <-- edge(a, b, _kind, _conf);
    reachable(a, c, n+1) <-- reachable(a, b, n), edge(b, c, _kind, _conf), (n < max_hops);
}
```

**Response** (cross-language blast radius):
```json
{
  "content": [{
    "type": "text",
    "text": "<blast_radius target=\"rust|||fn|||backend::api::auth|||handle_auth_request|||...\" hops=\"3\">\n  <hop n=\"0\" lang=\"rust\">\n    <entity key=\"...handle_auth_request...\" kind=\"fn\"/>\n  </hop>\n  <hop n=\"1\" lang=\"rust\">\n    <entity key=\"...auth_middleware...\" kind=\"fn\" edge=\"calls\"/>\n    <entity key=\"...validate_user...\" kind=\"fn\" edge=\"calls\"/>\n  </hop>\n  <hop n=\"1\" lang=\"ts\" crossing=\"http_boundary\">\n    <entity key=\"...fetchAuthData...\" kind=\"fn\" edge=\"http_boundary\" confidence=\"0.90\"/>\n  </hop>\n  <hop n=\"2\" lang=\"ts\">\n    <entity key=\"...LoginForm...\" kind=\"component\" edge=\"calls\"/>\n    <entity key=\"...useAuth...\" kind=\"fn\" edge=\"calls\"/>\n  </hop>\n  <hop n=\"3\" lang=\"ts\">\n    <entity key=\"...Navbar...\" kind=\"component\" edge=\"calls\"/>\n    <entity key=\"...App...\" kind=\"component\" edge=\"calls\"/>\n  </hop>\n  <summary rust_entities=\"3\" ts_entities=\"5\" total=\"8\" language_crossings=\"1\"/>\n</blast_radius>"
  }]
}
```

Claude tells the user: "Changing `handle_auth_request` ripples through 8 entities: 3 in Rust and 5 in TypeScript across 3 hops. The boundary crossing is via `POST /api/auth` — if you change the response shape, `fetchAuthData`, `LoginForm`, `useAuth`, `Navbar`, and `App` will all need updates."

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: analyze_blast_radius_hops()                                      │
│  CRATE: rust-llm-graph-reasoning  (calls rust-llm-store-runtime for facts) │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  BlastRadiusInput { root: EntityKey, max_hops: u32 }              │
│  DATA OUT: BlastRadiusResult                                                │
│            BlastRadiusResult { hops: Vec<HopGroup>,                         │
│                                summary: ImpactSummary }                    │
│            HopGroup { n: u32, entities: Vec<EntitySummary>,                 │
│                       crossing: Option<CrossingKind> }                      │
│            ImpactSummary { by_language: HashMap<Language, u32>,             │
│                            language_crossings: u32, total: u32 }           │
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - root must exist in store (key is valid)                                  │
│  - max_hops in range [1, 10]                                                │
│  - Ascent Datalog program compiled at crate build time                      │
│  - Base relations entity() and edge() loaded from store                     │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN root has no outgoing edges THEN hops=[HopGroup{n:0}], total=1      │
│  - SHALL run Ascent `reachable` rule transitive closure                     │
│  - SHALL group results by hop distance                                      │
│  - SHALL annotate hops that cross language boundaries with CrossingKind     │
│  - SHALL respect max_hops cutoff (no entity beyond N hops is included)      │
│  - SHALL NOT mutate store (read-only Datalog evaluation)                    │
│                                                                             │
│  ASCENT RULE (illustrative):                                                │
│  reachable(a, b, 1) <-- edge(a, b, _kind, _conf);                          │
│  reachable(a, c, n+1) <-- reachable(a, b, n), edge(b, c, _, _), (n < max);│
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - EntityNotFound → Err(ReasoningError::UnknownEntity(root))                │
│  - HopsOutOfRange → Err(ReasoningError::InvalidHops(max_hops))              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

#### Query 4: "Show me the Rust type info for handle_auth_request's parameter"

**Claude calls**: `get_entity_detail_view`
```json
{
  "name": "get_entity_detail_view",
  "arguments": {
    "key": "rust|||fn|||backend::api::auth|||handle_auth_request|||backend/src/api/auth.rs|||"
  }
}
```

**Parseltongue reads live from disk (Gate G3)**:
```rust
fn get_entity_detail(key: &EntityKey) -> Result<EntityDetail, DetailError> {
    let entity = store.get_entity(key)?; // single getter (Gate G2)

    // Read current source lines from disk — NOT from stored snapshot
    let file_path = key.file_path(); // "backend/src/api/auth.rs"
    let source = fs::read_to_string(&project_root.join(&file_path))
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound    => DetailError::FileMoved(file_path.clone()),
            ErrorKind::PermissionDenied => DetailError::PermissionDenied(file_path.clone()),
            _ => DetailError::IoError(e),
        })?;

    let lines: Vec<&str> = source.lines().collect();
    let start = entity.start_line.saturating_sub(1);
    let end = (entity.end_line).min(lines.len());
    let source_snippet = lines[start..end].join("\n");

    // Include rust-analyzer typed edges if available
    let typed_edges = store.get_typed_edges(key)?;

    EntityDetail { entity, source_snippet, typed_edges }
}
```

**Response**:
```json
{
  "content": [{
    "type": "text",
    "text": "<entity_detail key=\"rust|||fn|||backend::api::auth|||handle_auth_request|||backend/src/api/auth.rs|||\">\n  <source file=\"backend/src/api/auth.rs\" start_line=\"45\" end_line=\"67\">\npub async fn handle_auth_request(\n    State(db): State&lt;DbPool&gt;,\n    Json(credentials): Json&lt;LoginCredentials&gt;,\n) -&gt; Result&lt;Json&lt;AuthToken&gt;, AppError&gt; {\n    let user = validate_user(&db, &credentials).await?;\n    let token = generate_token(&user)?;\n    Ok(Json(token))\n}\n  </source>\n  <typed_edges>\n    <edge kind=\"Direct\" to=\"...validate_user...\" confidence=\"1.0\"/>\n    <edge kind=\"Direct\" to=\"...generate_token...\" confidence=\"1.0\"/>\n  </typed_edges>\n  <token_count>89</token_count>\n</entity_detail>"
  }]
}
```

If the file has moved or been deleted since ingest, the response includes an explicit error — it never returns stale cached source (Gate G3).

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: get_entity_detail_live()                                         │
│  CRATE: rust-llm-store-runtime  (store lookup) +                            │
│         rust-llm-interface-gateway  (disk read, Gate G3)                   │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  EntityKey  (language|||kind|||scope|||name|||file|||disc)         │
│  DATA OUT: EntityDetail                                                     │
│            EntityDetail { entity: Entity, source_snippet: String,           │
│                           typed_edges: Vec<TypedCallEdge>,                  │
│                           public_module_context: Vec<EntitySummary> }       │
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - EntityKey must match stored record (exact string match)                  │
│  - File at entity.file_path must be readable (Gate G3)                      │
│  - entity.start_line and entity.end_line must be set                        │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN file exists at disk THEN source_snippet = live lines[start..end]   │
│  - SHALL include all TypedCallEdge records from rust-analyzer (Rust only)   │
│  - SHALL include all public_module_context siblings from store              │
│  - SHALL NOT return cached/stored source (always reads disk — Gate G3)      │
│  - SHALL NOT include private siblings in public_module_context list         │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - EntityNotFound → Err(DetailError::EntityNotFound(key))                   │
│  - FileMoved (file gone from disk) → Err(DetailError::FileMoved(path))      │
│  - PermissionDenied → Err(DetailError::PermissionDenied(path))              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

#### Query 5: "What is the full public API surface of the file handle_auth_request lives in?"

This is the `public_module_context` query in action. The user wants to understand the **module**, not just the function.

**User types**: "What else is publicly exported from the same file as handle_auth_request?"

**Claude calls**: `get_entity_detail_view` then filters `public_module_context` edges
```json
{
  "name": "get_entity_detail_view",
  "arguments": {
    "key": "rust|||fn|||backend::api::auth|||handle_auth_request|||backend/src/api/auth.rs|||"
  }
}
```

**Response includes public_module_context edges**:
```json
{
  "content": [{
    "type": "text",
    "text": "<entity_detail ...>\n  ...\n  <public_module_context file=\"backend/src/api/auth.rs\">\n    <sibling key=\"rust|||fn|||backend::api::auth|||login_handler|||backend/src/api/auth.rs|||\" kind=\"fn\" public=\"true\"/>\n    <sibling key=\"rust|||use|||backend::api::auth|||axum::Json|||backend/src/api/auth.rs|||\" kind=\"import\" public=\"true\" note=\"import feeds co-located public fns\"/>\n    <sibling key=\"rust|||use|||backend::api::auth|||AppError|||backend/src/api/auth.rs|||\" kind=\"import\" public=\"true\"/>\n    <sibling key=\"rust|||struct|||backend::api::auth|||AuthRouter|||backend/src/api/auth.rs|||\" kind=\"struct\" public=\"true\"/>\n  </public_module_context>\n  <private_siblings count=\"3\" note=\"3 private helpers in same file, not in public surface\"/>\n</entity_detail>"
  }]
}
```

**Claude tells the user**:

> "The file `backend/src/api/auth.rs` has 4 public entities besides `handle_auth_request`:
> - `login_handler` (fn) — another public route handler
> - `AuthRouter` (struct) — likely the Axum router struct that wires these handlers
> - `use axum::Json` and `use AppError` — imports shared by all public fns in this file
>
> The 3 private helpers are there too but not exported — they're implementation details.
> Changing the return type of `handle_auth_request` likely ripples to `AuthRouter` too."

**Why this query is impossible without a graph**: Grep for `pub fn` in `auth.rs` gets you the functions. But it doesn't tell you that `use axum::Json` is a *dependency shared by all public functions* in that file, or that `AuthRouter` is the *container* that depends on all of them. The `public_module_context` edge captures this invisible semantic relationship.

```text
┌────────────────────────────────────────────────────────────────────┐
│  PUBLIC MODULE CONTEXT GRAPH — backend/src/api/auth.rs             │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │                  backend/src/api/auth.rs                    │  │
│  │                                                             │  │
│  │   ┌─────────────┐   pub_mod_ctx   ┌──────────────────┐     │  │
│  │   │ use Json    │ ──────────────▶ │ handle_auth_req  │     │  │
│  │   │ (import)    │ ──────────────▶ │ (pub fn)         │     │  │
│  │   └─────────────┘                 └──────────────────┘     │  │
│  │          │                                  │               │  │
│  │          │ pub_mod_ctx                      │ pub_mod_ctx   │  │
│  │          ▼                                  ▼               │  │
│  │   ┌─────────────┐   pub_mod_ctx   ┌──────────────────┐     │  │
│  │   │ login_      │ ──────────────▶ │ AuthRouter       │     │  │
│  │   │ handler     │                 │ (pub struct)     │     │  │
│  │   │ (pub fn)    │                 └──────────────────┘     │  │
│  │   └─────────────┘                                          │  │
│  │                                                             │  │
│  │   ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─┐    │  │
│  │     helper_parse_token  (private, shared_context only)      │  │
│  │   └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─┘    │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                    │
│  Solid lines = public_module_context (both public)                 │
│  Dashed box  = shared_context only (private entity)               │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CONTRACT: get_public_module_siblings()                                     │
│  CRATE: rust-llm-store-runtime                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│  DATA IN:  EntityKey  (the anchor entity whose file we want to inspect)     │
│  DATA OUT: Vec<EntitySummary>  (public siblings in the same file)           │
│            EntitySummary { key: EntityKey, kind: EntityKind,                │
│                            lang: Language, token_count: u32,                │
│                            is_public: bool, note: Option<String> }          │
│                                                                             │
│  PRECONDITIONS:                                                             │
│  - EntityKey must exist in store                                            │
│  - public_module_context edges must have been emitted during ingest         │
│    (i.e., emit_public_module_context() ran for this file)                  │
│                                                                             │
│  POSTCONDITIONS (success):                                                  │
│  - WHEN anchor entity is public THEN returns all other public entities      │
│      in the same file (via public_module_context edge traversal)            │
│  - WHEN anchor entity is private THEN returns Vec::new()                    │
│      (private entities are not connected by public_module_context)          │
│  - SHALL include import entities with note="import feeds co-located fns"   │
│      when import.is_public && co-located with at least one public fn        │
│  - SHALL NOT include the anchor entity itself in the returned list          │
│  - SHALL NOT include shared_context-only (private) siblings                 │
│                                                                             │
│  ERROR CONDITIONS:                                                          │
│  - EntityNotFound → Err(QueryError::EntityNotFound(key))                    │
│  - StoreNotReady → Err(QueryError::StoreNotReady)                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Section 2: Tauri App Journey

The user wants to manage Parseltongue instances without memorizing commands. Tauri is a **process / instance manager** — it launches, monitors, and stops parseltongue processes. It does not explore graphs.

**Scope boundary** (DECIDED — see ES-V200-Decision-log-01.md D4):

- **IN scope**: start/stop HTTP server process, write MCP config, display CLI command, log tail, workspace list
- **OUT of scope**: graph search, entity detail panels, blast-radius visualisation, type-alignment, export

There are three access modes for each workspace, all accessible from one window:

| Mode | Button label | What the button does |
|------|-------------|----------------------|
| HTTP | `[Start HTTP Server]` | Spawns `parseltongue pt08-http-code-query-server --db <path>` as a child process |
| MCP  | `[Write MCP Config]`  | Writes `~/.config/claude/claude_desktop_config.json` entry for this workspace |
| CLI  | `[Show CLI Command]`  | Displays the ingest command so the user can run it themselves |

> **OQ-T01** — Should the ingest step also be triggerable from Tauri, or is CLI-only for ingest acceptable?
> **OQ-T02** — Does the MCP config write need a restart prompt for Claude Desktop?

---

### L1 — Instance Manager Overview

```text
┌─────────────────────────────────────────────────────────────────────┐
│            TAURI APP JOURNEY — LEVEL 1 OVERVIEW                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│   ┌─────────────┐      ┌─────────────────────────────────────┐     │
│   │  PHASE 1    │      │           PHASE 2                   │     │
│   │             │      │                                     │     │
│   │    OPEN     │─────▶│   MANAGE WORKSPACE INSTANCES        │     │
│   │             │      │                                     │     │
│   │ Double-click│      │  [Start HTTP]  [Write MCP]          │     │
│   │ app, pick   │      │  [Show CLI]    Log tail below       │     │
│   │ workspace   │      │                                     │     │
│   └─────────────┘      └─────────────────────────────────────┘     │
│       <1 sec                        Ongoing                        │
│                                                                     │
│   No EXPLORE phase. Queries go through HTTP API / MCP / terminal.  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

### L2 — Phase 1: Open (Detailed)

```text
┌─────────────────────────────────────────────────────────────────────┐
│                  PHASE 1: OPEN — LEVEL 2                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1.1 LAUNCH              1.2 WORKSPACE LIST     1.3 SELECT          │
│  ┌────────────────┐      ┌────────────────┐    ┌────────────────┐   │
│  │ Parseltongue   │      │ Scans ~/  for  │    │ User clicks    │   │
│  │ .app launches  │─────▶│ parseltongue*/ │───▶│ workspace row  │   │
│  │ WebView opens  │      │ folders        │    │ → Phase 2      │   │
│  └────────────────┘      └────────────────┘    └────────────────┘   │
│                                                                     │
│  1.4 ADD NEW                                                        │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ [+ Add Workspace]                                            │   │
│  │ → native folder picker → user selects folder with analysis.db│   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

### L3 — Phase 1: Open (Implementation Detail)

#### 1.1 App Launch

User double-clicks `Parseltongue.app` (macOS) or `Parseltongue.exe` (Windows).

**What happens in sequence**:
1. Tauri runtime loads the native shell
2. WebView (WKWebView on macOS, WebView2 on Windows) initialises
3. Rust backend scans `~/parseltongue*/` for `analysis.db` files
4. Frontend receives workspace list via Tauri invoke `list_known_workspaces`
5. If no workspaces found → empty state with `[+ Add Workspace]`

**Tauri command**:
```rust
#[tauri::command]
async fn list_known_workspaces(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<WorkspaceEntry>, String> {
    // Scans home dir for parseltongue*/ folders containing analysis.db
    state.workspace_registry.lock().await.scan()
        .map_err(|e| e.to_string())
}
```

**Empty state UI**:
```text
┌────────────────────────────────────────────────────────────┐
│  Parseltongue v2.0                                         │
│                                                            │
│         No workspaces found.                               │
│                                                            │
│         [+ Add Workspace]                                  │
│                                                            │
│         Tip: ingest a folder first with:                   │
│         $ parseltongue pt01-folder-to-cozodb-streamer .    │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

> **OQ-T03** — Scan range: just `~/`? Or let user configure a scan root? (OQ-T03)

---

### L2 — Phase 2: Manage (Detailed)

```text
┌─────────────────────────────────────────────────────────────────────┐
│                  PHASE 2: MANAGE — LEVEL 2                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  WORKSPACE LIST                INSTANCE CONTROLS                   │
│  ┌────────────────────────┐    ┌───────────────────────────────┐   │
│  │ myapp-20260221  ●      │───▶│ [Start HTTP Server]           │   │
│  │ myapp-20260115  ○      │    │ [Stop HTTP Server]  (if live) │   │
│  │ other-20260110  ○      │    │ [Write MCP Config]            │   │
│  │                        │    │ [Show CLI Command]            │   │
│  │ [+ Add Workspace]      │    └───────────────────────────────┘   │
│  └────────────────────────┘                                        │
│                                                                     │
│  ● = HTTP server running on :7777                                   │
│  ○ = idle                                                           │
│                                                                     │
│  LOG TAIL (bottom pane, auto-scroll)                               │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ [14:30:22] HTTP server started on :7777                      │   │
│  │ [14:30:22] DB: rocksdb:parseltongue20260221/analysis.db      │   │
│  │ [14:30:25] GET /code-entities-list-all → 200 (43ms)         │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

### L3 — Phase 2: Manage (Implementation Detail)

#### 2.1 Workspace List

User sees a list of known workspaces. Each row shows:
- Workspace name (folder name)
- Status indicator: running (●) / idle (○)
- Port number if HTTP is active

Clicking a row selects it and shows the three mode buttons on the right.

#### 2.2 Mode A — Start HTTP Server

User clicks `[Start HTTP Server]`.

```rust
#[tauri::command]
async fn start_http_server_process(
    workspace_path: String,
    port: u16,
    state: tauri::State<'_, AppState>,
    window: tauri::Window,
) -> Result<(), String> {
    let db_path = format!("rocksdb:{}/analysis.db", workspace_path);
    let child = Command::new("parseltongue")
        .args(["pt08-http-code-query-server", "--db", &db_path,
               "--port", &port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    // Stream stdout/stderr to frontend as "log_line" events
    state.process_registry.lock().await
        .register(workspace_path, child, window);
    Ok(())
}
```

**What the user sees while HTTP is running**:

```text
┌────────────────────────────────────────────────────────────┐
│  myapp-20260221                              ● :7777        │
│                                                            │
│  [Stop HTTP Server]   [Write MCP Config]   [Show CLI]      │
│                                                            │
│  ────────────────────────────────────────── Log ─────────  │
│  [14:30:22] HTTP server started on :7777                   │
│  [14:30:22] DB: rocksdb:parseltongue20260221/analysis.db   │
│  [14:30:25] GET /code-entities-list-all → 200 (43ms)      │
│  [14:31:01] GET /blast-radius-impact-analysis → 200 (12ms)│
│                                                            │
│  Open in browser: http://localhost:7777                    │
└────────────────────────────────────────────────────────────┘
```

> **OQ-T04** — What port conflict policy? Auto-increment? Error dialog?
> **OQ-T05** — Should Tauri detect if the HTTP process dies and update the status indicator?

#### 2.3 Mode B — Write MCP Config

User clicks `[Write MCP Config]`.

The button writes an entry to `~/.config/claude/claude_desktop_config.json` (macOS path; Windows TBD).

```rust
#[tauri::command]
async fn write_mcp_config_entry(
    workspace_path: String,
    port: u16,
) -> Result<String, String> {
    let config_path = dirs::home_dir()
        .ok_or("No home dir")?
        .join(".config/claude/claude_desktop_config.json");

    let entry = serde_json::json!({
        "mcpServers": {
            "parseltongue": {
                "url": format!("http://localhost:{}", port)
            }
        }
    });
    // Merge into existing file or create new
    write_or_merge_json_config(&config_path, entry)
        .map_err(|e| e.to_string())?;

    Ok(config_path.to_string_lossy().to_string())
}
```

**Confirmation shown**:
```text
┌────────────────────────────────────────────────────────────┐
│  MCP config written.                                       │
│                                                            │
│  File: ~/.config/claude/claude_desktop_config.json         │
│                                                            │
│  Restart Claude Desktop to pick up the new server.        │
│                                                            │
│  [OK]                                                      │
└────────────────────────────────────────────────────────────┘
```

> **OQ-T06** — MCP config path varies by OS and Claude Desktop version. Needs a lookup table or user-configurable override.
> **OQ-T07** — Should we require HTTP server to be running before writing MCP config, or allow pre-configuration?

#### 2.4 Mode C — Show CLI Command

User clicks `[Show CLI Command]`.

A modal appears with the full ingest + serve command pair, ready to copy:

```text
┌────────────────────────────────────────────────────────────┐
│  CLI Commands for myapp-20260221                           │
│                                                            │
│  Ingest (run once or on change):                           │
│  $ parseltongue pt01-folder-to-cozodb-streamer \           │
│      /Users/dev/myapp                                      │
│                                                            │
│  Serve (start HTTP API):                                   │
│  $ parseltongue pt08-http-code-query-server \              │
│      --db "rocksdb:parseltongue20260221/analysis.db"       │
│      --port 7777                                           │
│                                                            │
│  [Copy All]                               [Close]          │
└────────────────────────────────────────────────────────────┘
```

No process is spawned. This is display-only. The user runs it themselves.

> **OQ-T08** — Should the CLI modal also show the MCP server startup command?
> **OQ-T09** — Should "Copy All" copy both commands as a shell script?
> **OQ-T10** — Should the app persist per-workspace port preferences?

---

## Data Type Boundary Reference

All structs that cross crate boundaries are listed here. These are the **contract types** — the exact shapes that each crate must agree on. A TDD stub on day 1 starts with these types.

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CRATE BOUNDARY: rust-llm-tree-extractor → rust-llm-store-runtime           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  struct Entity {                                                            │
│      key:         EntityKey,      // language|||kind|||scope|||name|||      │
│                                   //   file_path|||discriminator            │
│      kind:        EntityKind,     // fn | struct | impl | use |             │
│                                   //   component | interface | class | ...  │
│      language:    Language,       // Rust | TypeScript | JavaScript | ...   │
│      file:        String,         // normalized relative path               │
│      start_line:  u32,            // 1-indexed                              │
│      end_line:    u32,            // 1-indexed, inclusive                   │
│      is_public:   bool,           // pub / export keyword detected          │
│      token_count: u32,            // rough LLM token estimate               │
│  }                                                                          │
│                                                                             │
│  struct Edge {                                                              │
│      from:       EntityKey,                                                 │
│      to:         EntityKey,                                                 │
│      kind:       EdgeKind,        // calls | imports | shared_context |     │
│                                   //   public_module_context | implements | │
│                                   //   http_boundary | ...                  │
│      confidence: f32,             // 1.0 for structural; scored for cross-  │
│                                   //   language boundaries                  │
│      uncertain:  bool,            // true if confidence in [0.60, 0.80)     │
│      metadata:   serde_json::Value, // edge-kind-specific JSON blob         │
│  }                                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CRATE BOUNDARY: rust-llm-rust-semantics → rust-llm-store-runtime           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  struct TypedCallEdge {                                                     │
│      from:       EntityKey,       // caller                                 │
│      to:         EntityKey,       // callee                                 │
│      kind:       TypedCallKind,   // Direct | TraitMethod |                 │
│                                   //   DynDispatch | ClosureInvoke          │
│      confidence: f32,             // always 1.0 (rust-analyzer resolved)    │
│  }                                                                          │
│                                                                             │
│  struct ImplEdge {                                                          │
│      impl_key:   EntityKey,       // impl block entity                      │
│      struct_key: EntityKey,       // concrete type being implemented        │
│      trait_key:  EntityKey,       // trait being implemented                │
│      kind:       EdgeKind,        // EdgeKind::Implements                   │
│  }                                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CRATE BOUNDARY: rust-llm-cross-boundaries → rust-llm-store-runtime         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  // Already covered by Edge above — cross-boundary crate emits Edge structs │
│  // with kind = EdgeKind::HttpBoundary (or Ffi/Wasm/PyO3/Queue)            │
│                                                                             │
│  struct CrossLangEdge {                                                     │
│      from:       EntityKey,       // server-side entity (Rust/Go/etc.)      │
│      to:         EntityKey,       // client-side entity (TS/JS/Python/etc.) │
│      kind:       EdgeKind,        // HttpBoundary | FfiBoundary | ...       │
│      confidence: f32,             // scored 0.0–1.0                         │
│      uncertain:  bool,            // confidence in [0.60, 0.80)             │
│      metadata:   serde_json::Value,                                         │
│                                   // { route, method, framework, signals }  │
│  }                                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CRATE BOUNDARY: rust-llm-store-runtime → rust-llm-graph-reasoning          │
│  (Datalog base relations loaded at query time)                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  // Ascent Datalog relations (type-safe at compile time via ascent! macro)  │
│                                                                             │
│  relation entity(EntityKey, EntityKind, Language);                          │
│  relation edge(EntityKey, EntityKey, EdgeKind, f32);  // f32 = confidence   │
│                                                                             │
│  // These are the ONLY facts that rust-llm-graph-reasoning reads.           │
│  // All Datalog rules operate on these two base relations.                  │
│  // No direct storage queries inside the reasoning crate.                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  CRATE BOUNDARY: rust-llm-graph-reasoning → rust-llm-interface-gateway      │
│  (Query results returned to handler layer)                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  struct BlastRadiusResult {                                                 │
│      hops:    Vec<HopGroup>,                                                │
│      summary: ImpactSummary,                                                │
│  }                                                                          │
│                                                                             │
│  struct HopGroup {                                                          │
│      n:        u32,              // hop distance from root                  │
│      entities: Vec<EntitySummary>,                                          │
│      crossing: Option<CrossingKind>,  // Some(HttpBoundary) etc. if         │
│                                        //   this hop crosses a lang boundary│
│  }                                                                          │
│                                                                             │
│  struct ImpactSummary {                                                     │
│      by_language:       HashMap<Language, u32>,  // count per language      │
│      language_crossings: u32,                    // # of boundary crossings │
│      total:              u32,                    // total impacted entities  │
│  }                                                                          │
│                                                                             │
│  struct EntitySummary {                                                     │
│      key:         EntityKey,                                                │
│      kind:        EntityKind,                                               │
│      lang:        Language,                                                 │
│      token_count: u32,                                                      │
│      is_public:   bool,                                                     │
│      note:        Option<String>,  // e.g. "import feeds co-located fns"   │
│  }                                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  KEY TYPE: EntityKey  (crosses ALL crate boundaries as the graph node ID)   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  // Newtype wrapper — never a raw String in function signatures             │
│  struct EntityKey(String);                                                  │
│                                                                             │
│  // Format: language|||kind|||scope|||name|||file_path|||discriminator       │
│  // Built by: build_entity_key_canonical()                                  │
│  // Validation: EntityKey::parse(s: &str) -> Result<EntityKey, KeyError>    │
│                                                                             │
│  // Gate G1: line numbers NEVER embedded in the key                         │
│  // Gate G4: file_path always normalized to project-relative form           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Edge Type Reference

```text
┌─────────────────────────────────────────────────────────────────────┐
│  ALL EDGE TYPES — v2.0                                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  UNIVERSAL (all languages, tree-sitter):                           │
│  ├── shared_context         Same file. Always 1.0. Any two          │
│  │                          entities that live together.            │
│  ├── public_module_context  Same file + BOTH public/exported.       │
│  │                          Defines the module's API surface.       │
│  │                          Import + public fn = import feeds fn.   │
│  │                          Interface + public fn = contract bond.  │
│  ├── calls             Function invocation (resolved by name).      │
│  ├── imports           Module-level import/use/require.             │
│  ├── contains          Module/class contains function.              │
│  └── implements        Class/struct implements interface/trait.     │
│                                                                     │
│  RUST-ONLY (rust-analyzer enriched):                               │
│  ├── typed::Direct         Direct fn call (fully resolved).         │
│  ├── typed::TraitMethod    Call through trait object.               │
│  ├── typed::DynDispatch    dyn Trait dispatch.                      │
│  ├── typed::ClosureInvoke  Closure call.                            │
│  └── dataflow::assign      Variable assignment flows.               │
│                                                                     │
│  CROSS-LANGUAGE (confidence-scored):                               │
│  ├── http_boundary    Rust route ↔ JS/TS fetch (confidence based)  │
│  ├── ffi_boundary     Rust extern "C" ↔ C function                 │
│  ├── wasm_boundary    Rust wasm_bindgen ↔ JS import                │
│  ├── pyo3_boundary    Rust #[pyfunction] ↔ Python import           │
│  └── queue_boundary   Producer ↔ consumer via message queue        │
│                                                                     │
│  GRAPH-DERIVED (Datalog/Ascent):                                   │
│  ├── reachable         Transitive reachability (N hops)            │
│  ├── scc_member        Belongs to same SCC (cycle)                 │
│  └── layer_violation   Crosses architectural layer rule            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## EntityKey Format Reference

```text
┌─────────────────────────────────────────────────────────────────────┐
│  ENTITY KEY FORMAT                                                  │
│  language|||kind|||scope|||name|||file_path|||discriminator          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Rust function:                                                     │
│  rust|||fn|||backend::api::auth|||handle_auth_request|||            │
│  backend/src/api/auth.rs|||                                         │
│                                                                     │
│  Rust struct:                                                       │
│  rust|||struct|||backend::models|||User|||                          │
│  backend/src/models/user.rs|||                                      │
│                                                                     │
│  TypeScript function:                                               │
│  ts|||fn|||frontend.api|||fetchAuthData|||                          │
│  frontend/src/api/auth.ts|||                                        │
│                                                                     │
│  TypeScript React component:                                        │
│  ts|||component|||frontend.components|||LoginForm|||                │
│  frontend/src/components/LoginForm.tsx|||                           │
│                                                                     │
│  TypeScript interface:                                              │
│  ts|||interface|||frontend.types|||LoginCredentials|||              │
│  frontend/src/types.ts|||                                           │
│                                                                     │
│  Overloaded Rust function (discriminator = param types):            │
│  rust|||fn|||backend::db|||query|||backend/src/db.rs|||String,u32   │
│  rust|||fn|||backend::db|||query|||backend/src/db.rs|||String       │
│                                                                     │
│  Rules:                                                             │
│  • Line numbers NEVER in key (Gate G1)                             │
│  • Path normalized to project root (Gate G4)                       │
│  • ./path == path == /abs/path (all same key)                      │
│  • Discriminator: ParamTypes → Index → ContentHash fallback         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Success Metrics for Both Journeys

```text
┌─────────────────────────────────────────────────────────────────────┐
│  SUCCESS METRICS                                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  MCP Journey:                                                       │
│  ✓ User configures in < 5 min                                      │
│  ✓ Ingest of 1000-file repo completes in < 30 sec                  │
│  ✓ Tool response latency < 500ms for any query                     │
│  ✓ Cross-language boundary detected with ≥ 0.80 confidence          │
│  ✓ Blast radius 3 hops returns in < 200ms                          │
│  ✓ Token budget: 94%+ reduction vs raw file dumps                  │
│  ✓ Zero stdout contamination (logs only to stderr)                 │
│                                                                     │
│  Tauri Journey:                                                     │
│  ✓ App opens in < 1 sec (cold start)                               │
│  ✓ Dashboard appears in < 200ms after ingest                       │
│  ✓ Search results appear within 150ms of keystroke                 │
│  ✓ Blast radius graph renders in < 300ms                           │
│  ✓ Source code panel always shows live disk state (Gate G3)        │
│  ✓ Every UI action shows its CLI equivalent                        │
│  ✓ Power users switch to CLI within 7 days of first use            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```
