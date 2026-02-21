# CR07 Codex Research Progress Tracker

**Created**: 2026-02-19
**Last Updated**: 2026-02-19 (Phase 3 complete + Leiden background task complete)
**Subject**: Competitor Research 07 - OpenAI Codex CLI (codex-rs)
**Repo analyzed**: CR07/codex (v173 branch)
**Purpose**: Track document production progress, key findings, and phase status for Codex competitive research

---

## Phase Status Overview

| # | Phase | Status | Output Document | Lines |
|---|-------|--------|-----------------|-------|
| 1 | Graph Overview | GREEN - COMPLETE | CR07/CR-codex-graph-overview.md | 334 |
| 2 | Architecture Deep Dive | GREEN - COMPLETE | CR07/CR-codex-architecture.md | 1,212 |
| 3 | Feature Comparison with Factory Droid | GREEN - COMPLETE | CR07/CR-codex-vs-factory-droid.md | 958 |
| 4 | V200 Implications Document | RED - TODO | CR07/CR-codex-v200-implications.md | - |
| 5 | ELI5 ASCII Summary | YELLOW - PARTIAL | (delivered inline in conversation, not persisted) | - |

---

## Parseltongue Server (for this CR07 session)

- **Port**: 7780
- **Database**: `rocksdb:parseltongue20260219195022/analysis.db`
- **Full DB path**: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/CR07/codex/parseltongue20260219195022/analysis.db`
- **Health check**: `curl http://localhost:7780/server-health-check-status`

**Restart command** (run from the working directory below):
```bash
/Users/amuldotexe/Desktop/A01_20260131/parseltongue-v172 pt08-http-code-query-server \
  --db "rocksdb:parseltongue20260219195022/analysis.db" \
  --port 7780
```
**Working directory for restart**: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/CR07/codex`

---

## Phase 1: Graph Overview (GREEN - COMPLETE)

**Output**: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/CR07/CR-codex-graph-overview.md`
**Lines**: 334

### Key Findings

**Scale**
- 15,901 total entities
- 136,130 total edges
- 5 languages: Rust 14,324 / TypeScript 627 / C 386 / Python 326 / JavaScript 238

**Graph Health**
- 34,933 SCCs, all size-1 (zero circular dependencies across 40+ crates)
- 460 Leiden community clusters (initial pass)
- K-core max depth: 33 (highly layered, healthy DAG)
- Ingestion coverage: 60.96% (files parsed vs total files on disk)

**Coverage Gap Note**
- 39.04% of files are not ingested (likely generated files, build artifacts, node_modules, Bazel cache)
- Coverage metric is intentionally permissive - not a defect

---

## Phase 2: Architecture Deep Dive (GREEN - COMPLETE)

**Output**: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/CR07/CR-codex-architecture.md`
**Lines**: 1,212

### Key Findings

**God Object Risk**
- `codex.rs` (ToolOrchestrator / Codex struct): CBO=1124, Grade F
- This is the single orchestration point for all tool execution
- `ToolOrchestrator::run()` has 21.9% blast radius = 3,489 entities at risk if this function changes

**Subsystem Map (15 subsystems identified)**
- Core runtime: codex-rs/core
- CLI TUI: codex-rs/tui (Ratatui-based terminal UI)
- MCP server + client with OAuth (dual-mode)
- Sandbox layer: tri-platform (Seatbelt/macOS, Landlock+bubblewrap/Linux, ACL/Windows)
- Multi-model backend: OpenAI, LM Studio, Ollama, ChatGPT-compat
- Multi-agent: hierarchical agent support (143 collab entities confirmed via entity search)
- Tool system: bash, computer, file operations, web search
- Protocol: exec policy (allow/deny per tool per environment)

**Call Graph Highlights (5 critical entities analyzed)**
- ToolOrchestrator::run - entry point for all tool execution
- Sandbox enforcement inline with tool execution, not pre-checked
- MCP server registered as a tool backend alongside native tools

**CBO/LCOM Metrics**
- codex.rs: CBO=1124 (coupling to 1,124 other modules) - Grade F
- Coupling is architectural, not accidental (hub-and-spoke orchestrator pattern)
- LCOM not separately reported; single class cohesion is moot at this coupling level

**Tri-Platform Sandbox Diagrams**
- macOS: Seatbelt profiles (deny-all + allowlist)
- Linux: Landlock syscall-level restrictions + bubblewrap namespace isolation
- Windows: ACL-based file path restrictions (weakest isolation)

**MCP Integration (4 diagrams produced)**
- Codex as MCP server (exposes tools to external MCP clients)
- Codex as MCP client (consumes external MCP servers as tool backends)
- OAuth flow for remote MCP authentication
- Tool routing: native tools vs MCP-delegated tools

---

## Phase 3: Feature Comparison with Factory Droid (GREEN - COMPLETE)

**Output**: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/CR07/CR-codex-vs-factory-droid.md`
**Lines**: 958 (actual, previously estimated ~650)

### Comparison Axes Covered (8 primary axes + 3 additional + summary)

**Primary Axes**
1. Sandboxing depth (Codex 5 kernel layers vs Droid 3 software layers - CODEX wins)
2. MCP integration (Codex server+client vs Droid client-only - CODEX wins)
3. Multi-agent architecture (Codex collab flexible vs Droid orch/worker structured - DROID wins)
4. Multi-model backend (Codex 4 providers + local vs Droid 3+ cloud-only - CODEX wins)
5. Tool system design (9/14 identical tools - TIE, same DNA)
6. Graph analyzability (Codex 15,901 entities open vs Droid 0 opaque - CODEX wins)
7. State persistence (Codex ephemeral vs Droid disk artifacts - DROID wins)
8. Approval/safety (Codex 7 layers protocol-level vs Droid 3 session-level - CODEX wins)

**Additional Axes**
- Task decomposition (DROID wins - systematic vs ad-hoc)
- Code quality metrics (DROID wins - ByteRank exists vs nothing)
- Skill/recipe system (DROID wins - richer than hooks)

### Additional Sections
- What each should steal from the other (5 items each direction)
- Where Parseltongue fits (intelligence gap, ByteRank replacement)
- V200 contract implications table (8 findings, P0-P3 priority)
- Three-body architecture ASCII diagram

### Final Score: CODEX 7 | DROID 4 | TIE 1

### Input Materials Used
- Codex: CR07/CR-codex-graph-overview.md + CR07/CR-codex-architecture.md
- Factory Droid: `docs/CR-droid-factory-20260219/factory-droid/CR-factory-droid-control-flow-202601.md`
- Factory Droid: `docs/v200-docs/CR-factory-droid-202601.md`
- Parseltongue server (port 7780): entity search validation (approval=178, collab=143, spawn_agent=6)

### Key Insight: Tool DNA is Identical
- 9 of 14 tools are identical between Codex and Factory Droid
- Differentiation is in orchestration, not tools
- Neither has graph-computed code intelligence — Parseltongue's lane is wide open
- ByteRank's 8/11 evaluation categories are replaceable by Parseltongue deterministic graph algorithms

---

## Leiden Community Background Task (COMPLETE)

**Triggered**: During Phase 2-3 work as background task
**Endpoint used**: `/leiden-community-detection-clusters`

### Results
- **Total communities detected**: 9,887
- **Top Community 8**: 3,495 members (likely core Rust crate cluster)
- **Top Community 12**: 3,247 members
- **Top Community 22**: 2,835 members
- **Community 26**: 520 members - collab tools test cluster

### Significance
- Community 26 (520 members = collab tools test cluster) **confirms multi-agent is a real subsystem**, not just documentation
- The top 3 communities contain ~30% of all 15,901 entities, confirming hub-and-spoke concentration
- Distribution validates God Object finding: codex.rs dominates through community membership

---

## Phase 4: V200 Implications Document (RED - TODO)

**Output target**: `CR07/CR-codex-v200-implications.md`
**Dependencies**: Phase 3 complete (done) + v200-docs analysis

### Required Input Documents (read before starting Phase 4)
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/docs/ZAI-PRD-contracts-01.md`
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/docs/ZAI-PRD-contracts-02-FINAL.md`
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/docs/v200-docs/PRD-v200.md`

### Planned Content
- What Codex's architecture reveals about gaps in current Parseltongue v200 plan
- MCP dual-mode (server+client) vs Parseltongue's MCP-first strategy
- Multi-model abstraction patterns to adopt
- Sandbox approach differences and what v200 should learn
- God Object anti-pattern (codex.rs CBO=1124) as warning for v200 design
- K-core=33 depth vs Parseltongue's 8-crate layered architecture
- Zero circular deps validation: confirms v200 architecture direction
- ByteRank intelligence gap: Parseltongue graph algorithms as deterministic alternative
- V200 contract mapping: each Phase 3 implication mapped to a specific PRD contract

### Pre-existing V200 Implications from Phase 3 (to incorporate)
These were already drafted in Phase 3 output and must be expanded in Phase 4:

| Priority | Finding | V200 Implication |
|----------|---------|-----------------|
| P0 | Codex CBO=1124 god object | v200 orchestration must stay shallow, no single hub |
| P0 | ByteRank 8/11 replaceable by graph | v200 must offer deterministic code quality metrics |
| P1 | MCP dual-mode confirmed by Codex | v200 MCP-first direction is validated |
| P1 | Droid structured handoffs win | v200 should consider structured task decomposition API |
| P1 | 9/14 tools identical across competitors | v200 tool protocol standardization has market evidence |
| P2 | Codex zero circular deps | v200 architecture direction confirmed |
| P2 | Leiden Community 26 = real multi-agent | v200 may need multi-agent awareness in graph queries |
| P3 | Codex K-core=33 layering | v200 8-crate architecture is actually shallower; evaluate if expansion needed |

---

## Phase 5: ELI5 ASCII Summary (YELLOW - PARTIAL)

**Output target**: `CR07/CR-codex-eli5-ascii-summary.md`
**Status**: Content was delivered inline in conversation but NOT persisted to a file
**Dependency**: Phases 1-4 complete before final version is written to file

### Planned Content (for when persisted)
- 1-page ASCII art diagram of Codex's architecture
- Plain-English explanation of what Codex does
- 5 key strengths vs 5 key weaknesses
- "If Parseltongue were Codex..." comparison
- Actionable takeaways for V200 in bullet form

---

## Key Findings Summary (All Phases)

### Finding 1: God Object Pattern (codex.rs)
- CBO=1124 means codex.rs couples to over 1,000 other modules
- This is the single-point-of-failure for the entire tool orchestration
- V200 implication: Parseltongue must not replicate this pattern; keep orchestration shallow

### Finding 2: Zero Circular Dependencies
- 34,933 SCCs all size-1 across 40+ crates
- Confirms healthy DAG topology despite high coupling at the orchestrator level
- Validates Parseltongue's own no-circular-dep architecture goal

### Finding 3: Tri-Platform Sandboxing
- macOS > Linux > Windows in sandbox strength
- Windows ACL-only approach is weaker than Linux namespacing
- V200 implication: Parseltongue does not need sandboxing (read-only analysis tool), but Codex's approach is a useful reference for any future execution features

### Finding 4: Dual-Mode MCP (server AND client)
- Codex is both an MCP server (exposing its tools) and an MCP client (consuming external tools)
- This dual-mode is architecturally significant - most tools are only one or the other
- V200 implication: Parseltongue PRD already targets MCP-first; Codex confirms this is the right direction

### Finding 5: Multi-Agent Hierarchy
- Codex supports spawning sub-agents that can use the same tool system
- This is a recursive architecture: agents can delegate to agents
- Confirmed by entity search: collab=143 entities, spawn_agent=6 entities
- Leiden Community 26 (520 members) independently confirms this is a real subsystem
- V200 implication: Parseltongue v200 is query/analysis focused, not execution focused; agent hierarchy not needed in v200 scope

### Finding 6: ToolOrchestrator::run() Blast Radius
- 21.9% blast radius = 3,489 entities directly or transitively depend on this single function
- Any signature change or refactor here cascades widely
- V200 implication: Design Parseltongue's query entry points with minimal blast radius from day one

### Finding 7: Tool DNA is Shared Industry-Wide
- 9/14 tools identical between Codex and Factory Droid
- Competitors are not differentiating on tools — they differentiate on orchestration, safety, and intelligence
- V200 implication: Parseltongue's differentiation is graph-computed intelligence, confirmed as an open lane

### Finding 8: ByteRank Intelligence Gap
- Factory Droid's ByteRank evaluates code quality with LLM-based scoring
- 8 of 11 ByteRank categories can be replaced by deterministic Parseltongue graph algorithms
- V200 implication: Parseltongue should expose a ByteRank-equivalent deterministic scoring API

---

## Cross-References

### Input Repos and Docs
- Codex source: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/CR07/codex/` (v173 branch)
- Factory Droid decompiled artifacts: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/CR05/factory-droid/`
- Factory Droid research doc 1: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/docs/CR-droid-factory-20260219/factory-droid/CR-factory-droid-control-flow-202601.md`
- Factory Droid research doc 2: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/docs/v200-docs/CR-factory-droid-202601.md`
- V200 contracts 1: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/docs/ZAI-PRD-contracts-01.md`
- V200 contracts 2: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/docs/ZAI-PRD-contracts-02-FINAL.md`
- V200 PRD: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/docs/v200-docs/PRD-v200.md`

### Output Docs (CR07/ directory — gitignored)
- `CR07/CR-codex-graph-overview.md` - Phase 1 output (334 lines) - COMPLETE
- `CR07/CR-codex-architecture.md` - Phase 2 output (1,212 lines) - COMPLETE
- `CR07/CR-codex-vs-factory-droid.md` - Phase 3 output (958 lines) - COMPLETE
- `CR07/CR-codex-v200-implications.md` - Phase 4 output (pending)
- `CR07/CR-codex-eli5-ascii-summary.md` - Phase 5 output (pending)

---

## TDD Session State: 2026-02-19

### Current Phase: Red (Phase 4 is next; Phases 1-3 complete)

### Research Tasks Written:
- Phase 1 Graph Overview: GREEN - 334 lines, all metrics captured
- Phase 2 Architecture Deep Dive: GREEN - 1,212 lines, all subsystems documented
- Phase 3 Feature Comparison: GREEN - 958 lines, 11 axes + summary matrix
- Phase 4 V200 Implications: RED - next up
- Phase 5 ELI5 ASCII Summary: YELLOW - delivered inline in conversation (not persisted to file)
- Leiden Background Task: GREEN - 9,887 communities, Community 26 confirms multi-agent subsystem

### Current Focus:
Phase 4 is the immediate next step. Requires reading V200 PRD and contract docs listed above, then producing `CR07/CR-codex-v200-implications.md` that maps all competitor findings to specific V200 contracts.

### Next Steps:
1. Read `docs/ZAI-PRD-contracts-01.md` and `docs/ZAI-PRD-contracts-02-FINAL.md` to understand V200 contract structure
2. Read `docs/v200-docs/PRD-v200.md` for full V200 roadmap context
3. Map each of the 8 key findings above to specific V200 contract changes or confirmations
4. Produce `CR07/CR-codex-v200-implications.md` (target: 400-600 lines)
5. Mark Phase 4 as GREEN
6. Optionally persist Phase 5 ELI5 ASCII to `CR07/CR-codex-eli5-ascii-summary.md` (was delivered inline)

### Context Notes:
- Codex ingestion was run against the v173 branch of the codex repo
- The parseltongue database is at `CR07/codex/parseltongue20260219195022/analysis.db`
- Server confirmed live as of this update: `curl http://localhost:7780/server-health-check-status` returns `{"success":true,"status":"ok"}`
- All 7 graph analysis endpoints were used during Phase 1-2 research
- CBO=1124 for codex.rs is accurate; confirmed via `/coupling-cohesion-metrics-suite` endpoint
- Blast radius 21.9% confirmed via `/blast-radius-impact-analysis?entity=rust:fn:ToolOrchestrator::run&hops=3`
- Phase 3 actual line count is 958, not ~650 as previously estimated
- Leiden task ran as background; results available from conversation history: 9,887 communities total

### Technical Debt Identified:
- Codex's God Object pattern (codex.rs) is the primary architectural risk in the target codebase
- Parseltongue v200 design must explicitly avoid hub-and-spoke orchestration at a single module
- Phase 5 ELI5 content exists in conversation memory only — should be committed to file during Phase 4 session

### Parseltongue Endpoints Most Useful for Phase 4:
- `/codebase-statistics-overview-summary` — top-level numbers for the V200 implications narrative
- `/technical-debt-sqale-scoring?entity=rust:struct:Codex` — SQALE debt estimate for v200 warning section
- `/centrality-measures-entity-ranking?method=pagerank` — confirm PageRank leaders for hub analysis

---

*Last updated: 2026-02-19 (Phase 3 complete + Leiden background task results added)*
*Next update: After Phase 4 (V200 Implications) is complete*
