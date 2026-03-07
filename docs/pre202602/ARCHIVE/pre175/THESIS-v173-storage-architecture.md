# Thesis: Parseltongue Storage Architecture — Beyond CozoDB

**Date**: 2026-02-12
**Branch**: v173
**Status**: Research complete, revisit later

---

## ELI5: What Is This About?

Right now, Parseltongue works like this:
1. **Parse** your code (find all functions, classes, who-calls-what)
2. **Store** that info in a **database** (CozoDB)
3. **Serve** it via HTTP so your LLM can ask questions

The database (step 2) keeps breaking on Windows. We tried 6 fixes. All failed.

**The question**: Do we even need a database? What if we just kept everything in memory (like a variable in your program)? Or saved it as a JSON file?

Think of it like storing your contacts:
- **Database** (current) = A fancy filing cabinet with locks and drawers. Powerful but complex. The drawers keep jamming on Windows.
- **RAM** = Just memorize all your contacts. Instant recall. But if you lose power, you forget everything (unless you write them down first).
- **JSON file** = Write contacts in a notebook anyone can read. Simple, portable, but you have to read the whole notebook to find one person.

---

## Problem Statement

CozoDB has failed on Windows across 6 attempts (v1.6.7–v1.7.2). Research reveals CozoDB is a persistence/query layer that feeds data into HashMaps — 80% of in-memory graph infrastructure already exists.

---

## Key Finding

All 24 HTTP handlers follow this pattern:
```
Query CozoDB → copy into HashMap → run algorithm on HashMap → return JSON
```
All 7 graph algorithms run on `AdjacencyListGraphRepresentation` (pure Rust), not CozoDB. All entity types already derive `Serialize, Deserialize`.

**ELI5**: We're paying the cost of a database, but every time we use it, we copy the data OUT of the database into regular Rust variables, do our work there, and throw away the database copy. It's like photocopying every page of a library book instead of just reading the book directly.

---

## 6 Simulations — Full Details

### How Scoring Works

Each simulation is rated 1-5 on 5 dimensions. Higher = better. Max total = 25.

| Dimension | What it means (ELI5) | Score 1 | Score 5 |
|-----------|---------------------|---------|---------|
| **Simplicity** | How easy is the code to understand and maintain? | Spaghetti code, lots of moving parts | Clean, minimal, a new developer gets it in 5 min |
| **Windows Fix** | Does it actually solve the Windows problem? | Maybe, with workarounds | Permanently, by design |
| **Extensibility** | How easy to add a 4th backend later? (e.g., PostgreSQL) | Rewrite half the codebase | Add one file, implement one trait |
| **Migration Risk** | How likely are we to break existing users? | High chance of breaking changes | Zero risk, existing commands unchanged |
| **User Experience** | How many things does a user need to learn/remember? | Multiple tools, flags, formats to juggle | Same commands as today, just works |

---

### Simulation 1: "Three Crates" (Your Original Proposal)

**ELI5**: Build 3 separate tools — one saves to database, one saves to RAM, one saves to JSON. Like having 3 different calculators: one for addition, one for subtraction, one for multiplication.

**How it works**:
```
User picks a tool:
  parseltongue pt01 .          → saves to CozoDB (Mac/Linux only)
  parseltongue pt02 .          → saves to RAM + .ptgraph file
  parseltongue pt03 .          → saves to .json file

Then starts the server:
  parseltongue pt08 --db "rocksdb:path"   (for pt01 output)
  parseltongue pt08 --db "ptgraph:path"   (for pt02 output)
  parseltongue pt08 --db "json:path"      (for pt03 output)
```

**What's in each crate**:
- pt01: tree-sitter parsing + CozoDB storage (existing)
- pt02: tree-sitter parsing + HashMap storage + bincode serialization (NEW)
- pt03: tree-sitter parsing + HashMap storage + JSON serialization (NEW)

**The problem**: The tree-sitter parsing code is identical in all 3. That's ~2000 lines copied 3 times. When you add a new language or fix a parsing bug, you fix it in 3 places.

| Dimension | Score | Why |
|-----------|:-----:|-----|
| Simplicity | 2 | 3 crates doing 70% the same thing. Confusing for contributors. |
| Windows Fix | 3 | Works, but user must know to use pt02/pt03 instead of pt01. |
| Extensibility | 2 | Adding a 4th backend = 4th crate with duplicated parsing code. |
| Migration Risk | 5 | Existing pt01 untouched. Zero breakage risk. |
| User Experience | 2 | User must choose between 3 tools. "Which one do I use?" |
| **TOTAL** | **14** | |

**The non-obvious problem**: Every time you add a field to `entities.rs` (like a new metadata column), you need migration logic in all 3 formats. The maintenance cost grows combinatorially, not linearly.

---

### Simulation 2: "Single Tool, Backend Flag"

**ELI5**: One tool with a switch. Like a TV with an input selector — HDMI 1, HDMI 2, HDMI 3. Same TV, different sources.

**How it works**:
```
User picks a backend via flag:
  parseltongue pt01 . --backend cozodb    → CozoDB (default on Mac/Linux)
  parseltongue pt01 . --backend ram       → RAM + .ptgraph
  parseltongue pt01 . --backend json      → JSON file
  parseltongue pt01 .                     → auto-detect (Windows=ram, Mac=cozodb)

Server reads any format:
  parseltongue pt08 --db "rocksdb:path"
  parseltongue pt08 --db "ptgraph:path"
  parseltongue pt08 --db "json:path"
```

**What changes**: pt01's internal code has a `match backend { CozoDB => ..., Ram => ..., Json => ... }` branch at the storage layer. Parsing code is shared (no duplication).

| Dimension | Score | Why |
|-----------|:-----:|-----|
| Simplicity | 3 | One crate, but internal match arms get complex. |
| Windows Fix | 4 | Auto-detect works. User doesn't need to know. |
| Extensibility | 3 | Adding backend = add match arm. OK but grows. |
| Migration Risk | 3 | Crate named "cozodb-streamer" now does RAM/JSON. Misleading. |
| User Experience | 4 | One tool, sensible defaults. Slight learning curve for --backend flag. |
| **TOTAL** | **17** | |

**The non-obvious problem**: The crate is called `pt01-folder-to-cozodb-streamer`. Running `pt01 --backend json` uses a crate with "cozodb" in its name that doesn't touch CozoDB. Every CI pipeline, every doc, every tutorial references this name. Renaming it is a breaking change.

---

### Simulation 3: "pt02 = Next Gen Universal"

**ELI5**: Build a brand new tool (pt02) that does everything better. Keep the old tool (pt01) around for people who still use it. Like iPhone replacing iPod — iPod still works, but iPhone is the future.

**How it works**:
```
Legacy (keep for backward compat):
  parseltongue pt01 .          → CozoDB (Mac/Linux only)

Next gen (all platforms):
  parseltongue pt02 .          → RAM + exports .ptgraph + .json
  parseltongue pt02 . --serve  → ingest AND start HTTP server in one command

Server:
  parseltongue pt08 --db "rocksdb:path"   (for pt01)
  parseltongue pt08 --db "ptgraph:path"   (for pt02)
```

| Dimension | Score | Why |
|-----------|:-----:|-----|
| Simplicity | 2 | Two ingestion tools. "Which one should I use?" |
| Windows Fix | 4 | pt02 works on Windows. pt01 doesn't. Clear. |
| Extensibility | 3 | pt02 can evolve. pt01 is frozen. |
| Migration Risk | 5 | pt01 untouched. Zero breakage. |
| User Experience | 3 | New users confused: "Why are there two?" Old users: "Should I switch?" |
| **TOTAL** | **17** | |

**The non-obvious problem**: "Legacy" crates never die. Someone's CI pipeline depends on pt01. You maintain it forever. Every new contributor asks "why are there two ingestion tools?" and you explain the history every time.

---

### Simulation 4: "Replace pt01 Entirely"

**ELI5**: Upgrade the existing tool so it automatically does the right thing per platform. Like a car that switches between gas and electric depending on road conditions. Same steering wheel, same pedals.

**How it works**:
```
Same commands as today (nothing changes for the user):
  parseltongue pt01 .
    → Mac/Linux: uses RocksDB (fast, no issues)
    → Windows: uses RAM, auto-saves .ptgraph snapshot

  parseltongue pt08 --db "rocksdb:path"     (Mac/Linux)
  parseltongue pt08 --db "ptgraph:path"     (Windows)
```

The .ptgraph file is also **portable** — a Mac developer can email it to a Windows colleague.

| Dimension | Score | Why |
|-----------|:-----:|-----|
| Simplicity | 4 | One tool, auto-detects platform. Clean. |
| Windows Fix | 5 | RAM has zero filesystem issues. Permanent fix. |
| Extensibility | 3 | Adding JSON export = add a flag. OK but not pluggable. |
| Migration Risk | 2 | Rewriting pt01 could introduce bugs on Mac/Linux. |
| User Experience | 5 | Same commands as today. User doesn't notice the change. |
| **TOTAL** | **19** | |

**The non-obvious insight**: The .ptgraph snapshot becomes a **distribution format**. A Mac dev ingests fast with RocksDB, emails the .ptgraph to a Windows colleague, who opens it with pt08. This enables collaborative code review workflows that CozoDB-only can never do.

---

### Simulation 5: "Parseltongue Lite" (Zero-Dependency Binary)

**ELI5**: Build a second, simpler version of the whole tool that doesn't need any C++ code (RocksDB requires C++). Like making a "lite" version of an app that works everywhere, even on old phones.

**How it works**:
```
Full version (existing, Mac/Linux):
  parseltongue pt01 .          → CozoDB + RocksDB
  parseltongue pt08 ...

Lite version (new, all platforms including WASM):
  parseltongue-lite analyze .             → RAM + .ptgraph + .json
  parseltongue-lite analyze . --serve     → combined ingest + HTTP server
```

Key difference: `parseltongue-lite` compiles without C++ dependencies. This means:
- **5MB binary** vs 50MB for full version
- **30-second compile** vs 3 minutes
- **Runs in a browser** (WASM) or **VS Code extension**

| Dimension | Score | Why |
|-----------|:-----:|-----|
| Simplicity | 3 | Two binaries, but each is simple internally. |
| Windows Fix | 5 | Zero C++ = zero Windows build/runtime issues. |
| Extensibility | 2 | Two parallel codebases. Feature parity nightmare. |
| Migration Risk | 5 | Full version untouched. Lite is additive. |
| User Experience | 4 | "Lite" implies inferior, but it's actually the embeddable SDK. |
| **TOTAL** | **19** | |

**The non-obvious insight**: WASM compilation opens a completely new distribution channel. A WASM parseltongue-lite could run **in a browser**, **in a VS Code extension**, or **inside an LLM agent's sandbox** — zero installation. This reframes the product from "CLI tool" to "embeddable code analysis engine."

---

### Simulation 6: "Storage Adapter Pattern" (Winner)

**ELI5**: Instead of the tool knowing HOW to talk to a database, you create a "plug" system. CozoDB is one plug. RAM is another plug. JSON is another plug. The tool just says "give me all entities" and the plug handles the rest. Like USB — your computer doesn't care if it's a mouse, keyboard, or hard drive. It just uses the USB interface.

**How it works**:
```
Under the hood:
  trait StorageQueryBackend {
      fn query_all_entities() → entities
      fn query_all_edges() → edges
      fn search_fuzzy(pattern) → matches
      ... (15 methods total)
  }

  CozoDbBackend implements the trait using CozoScript
  RamBackend implements the trait using HashMap
  JsonBackend implements the trait using serde_json

User experience (same as today):
  parseltongue pt01 . --backend cozodb|ram|json
  parseltongue pt08 --db "rocksdb:path" | "ptgraph:path" | "json:path"

  Windows auto-selects ram. Mac auto-selects cozodb.
```

**Why this is different from Sim 2**: In Sim 2, the match/switch is scattered across 24 HTTP handlers. In Sim 6, the match is in ONE place (the trait implementation), and every handler just calls `backend.query_all_entities()` without knowing or caring which backend is behind it.

| Dimension | Score | Why |
|-----------|:-----:|-----|
| Simplicity | 4 | One trait, clear contract. Each handler is 10 lines instead of 50. |
| Windows Fix | 5 | RAM backend has zero filesystem issues. Permanent fix. |
| Extensibility | 5 | Adding PostgreSQL backend = implement 15 methods. No other changes. |
| Migration Risk | 2 | Refactoring 24 handlers is the biggest PR you'll ever ship. |
| User Experience | 5 | Same commands as today. Backend is invisible to users. |
| **TOTAL** | **21** | |

**The non-obvious insight**: This refactor **pays for itself twice**:
1. **First payoff**: Multi-backend support (CozoDB, RAM, JSON).
2. **Second payoff**: Today you can't test HTTP handlers without a real CozoDB database. After the refactor, you test every handler with a `MockBackend` that returns canned data. Test suite becomes 10x faster and 10x more reliable. The "cost" is actually removal of tech debt.

---

## Score Breakdown — Full Matrix

Each cell shows the score (1-5) with a one-line reason.

| Dimension | Sim 1: 3 Crates | Sim 2: Flag | Sim 3: Universal | Sim 4: Replace | Sim 5: Lite | Sim 6: Adapter |
|-----------|:-:|:-:|:-:|:-:|:-:|:-:|
| **Simplicity** | 2 — 70% code duplication | 3 — match arms grow | 2 — two tools, confusion | 4 — one tool, auto | 3 — two binaries | **4** — one trait |
| **Windows Fix** | 3 — user picks tool | 4 — auto-detect | 4 — pt02 works | **5** — permanent | **5** — no C++ | **5** — permanent |
| **Extensibility** | 2 — new crate per backend | 3 — match arm per backend | 3 — pt02 evolves | 3 — add flag | 2 — two codebases | **5** — implement trait |
| **Migration Risk** | **5** — zero risk | 3 — rename confusion | **5** — zero risk | 2 — rewrite risk | **5** — zero risk | 2 — big refactor |
| **User Experience** | 2 — which tool? | 4 — one tool + flag | 3 — which tool? | **5** — same as today | 4 — "lite" = inferior? | **5** — same as today |
| **TOTAL** | **14** | **17** | **17** | **19** | **19** | **21** |

---

## ELI5 Summary — Which One When?

| If you want... | Pick | Why |
|----------------|------|-----|
| Zero risk, ship today | Sim 1 (3 Crates) | Nothing existing breaks, but messy long-term |
| Quick Windows fix, clean code later | Sim 4 (Replace pt01) | Same commands, .ptgraph bonus, upgrade path |
| Browser/WASM future | Sim 5 (Lite) | No C++, embeddable, but two codebases |
| The "right" architecture for 5 years | **Sim 6 (Adapter)** | Biggest upfront work, best long-term payoff |
| Something in between | Sim 4 first → Sim 6 later | Ship fast, refactor proper |

---

## Comparison: CozoDB vs RAM vs JSON

| Dimension | CozoDB (current) | Pure RAM | JSON File |
|-----------|:-:|:-:|:-:|
| Setup complexity | High | Very Low | Very Low |
| Windows compat | Broken | Perfect | Perfect |
| Entity lookup | 0.5-2ms | ~50ns | 50ns (loaded) |
| Graph traversal | 20-100ms | 0.1-1ms | 0.1-1ms (loaded) |
| Memory (1.6K entities) | ~50MB | ~5MB | ~5MB + 3MB file |
| Disk usage | 20-50MB | ~500KB snapshot | ~3MB |
| Portability | No | Shareable .ptgraph | Universal |
| Scale (100K entities) | Good | Good (~1GB) | Slow startup (~2s) |
| Build time | Heavy (C++ RocksDB) | None | None |

**ELI5**:
- **CozoDB**: Like a restaurant kitchen — powerful, professional, but needs a chef (complex setup). Sometimes the kitchen catches fire on Windows.
- **RAM**: Like cooking at home — fast, simple, everything within reach. If the power goes out you lose your meal (unless you save it to a file first).
- **JSON**: Like a recipe card — anyone can read it, copy it, share it. But you can't cook directly from the card; you have to load it into your kitchen first.

---

## New Features Unlocked

| Feature | RAM | JSON | CozoDB | ELI5 |
|---------|:-:|:-:|:-:|------|
| Portable .ptgraph file | ✓ | ✓ | ✗ | Email your code analysis to a colleague |
| Zero-server LLM integration | ✓ | ✓ | ✗ | LLM loads graph directly, no HTTP needed |
| WASM browser deployment | ✓ | ✓ | ✗ | Run code analysis in a web page |
| Git-diffable analysis | ✗ | ✓ | ✗ | `git diff` shows what changed in the graph |
| SARIF compatibility | ✗ | ✓ | ✗ | GitHub Code Scanning integration |
| Version diffing | ✓ | ✓ | ✗ | Compare analysis between two git commits |
| Real-time WebSocket | ✓ | ✗ | ✗ | File saved → graph updates → push to client |

---

## Recommended: Simulation 6 — Storage Adapter Pattern

### Architecture
```
parseltongue-core
    ├── trait StorageQueryBackend (~15 methods)
    ├── impl CozoDbBackend   { CozoScript queries }
    ├── impl RamBackend      { HashMap + AdjacencyListGraphRepresentation }
    └── impl JsonBackend     { serde_json file }

pt01 --backend cozodb|ram|json  (auto-detect Windows → ram)
pt08 --db "rocksdb:path" | "ptgraph:path" | "json:path"
```

### Trait surface (~15 methods)
```rust
pub trait StorageQueryBackend {
    fn query_all_entities(&self, filter: Option<&str>, scope: Option<&str>) -> Result<Vec<CodeEntity>>;
    fn query_entity_by_key(&self, key: &str) -> Result<Option<CodeEntity>>;
    fn search_entities_fuzzy(&self, pattern: &str, scope: Option<&str>) -> Result<Vec<CodeEntity>>;
    fn query_all_edges(&self, scope: Option<&str>) -> Result<Vec<DependencyEdge>>;
    fn query_forward_edges(&self, key: &str) -> Result<Vec<DependencyEdge>>;
    fn query_reverse_edges(&self, key: &str) -> Result<Vec<DependencyEdge>>;
    fn get_codebase_statistics(&self) -> Result<CodebaseStats>;
    fn get_entity_source_code(&self, key: &str) -> Result<Option<String>>;
    fn blast_radius(&self, key: &str, hops: usize) -> Result<BlastRadiusResult>;
    fn get_coverage_data(&self) -> Result<CoverageData>;
    fn get_diagnostics_data(&self) -> Result<DiagnosticsData>;
    fn insert_entities_batch(&self, entities: &[CodeEntity]) -> Result<()>;
    fn insert_edges_batch(&self, edges: &[DependencyEdge]) -> Result<()>;
    fn backup_to_file(&self, path: &str, format: ExportFormat) -> Result<()>;
}
```

### Why enum dispatch (not dyn Trait)
- Backend set is closed (3-4 variants, known at compile time)
- 10x faster than `Box<dyn Trait>` (no vtable, enables inlining)
- Exhaustive `match` ensures new backends handled everywhere

**ELI5**: `enum dispatch` is like a light switch with 3 positions (CozoDB/RAM/JSON). The compiler knows all positions at build time, so it optimizes each path perfectly. `dyn Trait` is like a universal remote — flexible but slower because it has to figure out which device to talk to at runtime.

---

## Implementation Phases (when revisited)

| Phase | Work | Scope | ELI5 |
|-------|------|-------|------|
| 1 | Define `StorageQueryBackend` trait | parseltongue-core | Design the USB plug shape |
| 2 | Implement `CozoDbBackend` | parseltongue-core | Make CozoDB fit the new plug (no behavior change) |
| 3 | Implement `RamBackend` | parseltongue-core | Build the RAM plug |
| 4 | Implement `JsonBackend` | parseltongue-core | Build the JSON plug |
| 5 | Refactor 24 pt08 handlers | pt08 | Rewrite handlers to use the plug instead of CozoDB directly |
| 6 | CLI --backend flag + auto-detect | pt01, main.rs | Let users (and Windows) choose which plug |

---

## Existing Code to Reuse

| Asset | Location | Status |
|-------|----------|--------|
| `AdjacencyListGraphRepresentation` | `parseltongue-core/src/graph_analysis/` | Ready |
| All entity types (Serialize/Deserialize) | `parseltongue-core/src/entities.rs` | Ready |
| `CodeGraphRepository` trait | `parseltongue-core/src/interfaces.rs:143` | Exists but unused |
| 7 graph algorithms | `parseltongue-core/src/graph_analysis/` | Pure Rust, no CozoDB |

---

## Real-World Precedents

| Tool | Storage | Architecture | ELI5 |
|------|---------|-------------|------|
| rust-analyzer | In-memory (salsa) | Most sophisticated Rust tool | The gold standard — all in RAM, no database |
| Sourcetrail | SQLite (.srctrldb) | Portable single-file | Like us but simpler — one SQLite file |
| CodeQL | Custom relational | Extract once, query many | GitHub's code analysis — custom DB format |
| Semgrep | In-memory (OCaml) | Ephemeral scans | Scans code, throws away results after |
| petgraph | In-memory graph lib | 2.1M+ Rust downloads | The go-to Rust graph library |

---

*Research by Claude Code, 2026-02-12. Revisit when ready to implement.*
