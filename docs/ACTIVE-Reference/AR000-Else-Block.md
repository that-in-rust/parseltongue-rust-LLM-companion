
---

## GOVERNING THOUGHT

V200 architecture is ~70% decided at the crate and protocol level.
Implementation is blocked on 37 open questions that fall into 3 tiers.
**Tier 1 must resolve before any crate code is written.**
Tier 2 must resolve before TDD stubs are written for affected crates.
Tier 3 can resolve during implementation without blocking it.

```text
                    V200 DECISION STATE
                    ───────────────────
	                    ┌─────────────────┐
	                    │  GOVERNING IDEA │
	                    │  ~70% decided,  │
	                    │  37 OQs remain  │
	                    └────────┬────────┘
	             ┌───────────────┼──────────────────┐
	             ▼               ▼                  ▼
	      ┌─────────────┐ ┌─────────────┐  ┌─────────────┐
	      │  DECIDED    │ │  OPEN       │  │  OPEN       │
	      │  4 clusters │ │  TIER 1     │  │  TIER 2+3   │
	      │  binding    │ │  8 blockers │  │  29 OQs     │
	      │  now        │ │  arch-level │  │  impl-level │
	      └─────────────┘ └─────────────┘  └─────────────┘
```

---

# PART A — DECIDED (Binding, Not Revisitable Without a New Entry)

---

## D1 — Crate Scope and Architecture

**Decision**: V200 active scope is exactly 8 crates. Clean room — zero reuse from v1.x pt* crates.

```text
rust-llm-core-foundation        EntityKey, Entity, Edge, Error types
rust-llm-store-runtime          Single read/write contract, no direct queries in handlers
rust-llm-tree-extractor         File walk + tree-sitter per file, all languages
rust-llm-rust-semantics         rust-analyzer enrichment, Rust files only
rust-llm-cross-boundaries       HTTP/FFI/WASM detection, confidence scoring, all langs
rust-llm-graph-reasoning        Datalog/Ascent rules: blast-radius, SCC, cycles
rust-llm-interface-gateway      Entry point: MCP / HTTP / Tauri / CLI
rust-llm-test-harness           Internal only

DEFERRED TO V210:
rust-llm-context-packer         Token-minimization. Deferred — prove graph fidelity first.
```

**Principle**: Prove graph fidelity and read-path determinism first. Add context packing only after measured insufficiency.

**Rationale for deferral**: Context-packing can mask weak retrieval/fidelity in core graph contracts. Deferring reduces coupling and pass count so high-risk foundations converge faster.

---

## D2 — Interface Protocol

**Decision**: JSON-RPC 2.0 over stdio for MCP. This is a protocol compliance requirement — Anthropic's MCP spec mandates it.

**Why not the alternatives**:

- REST/HTTP — stateless by design, cannot correlate async tool responses, session state (loaded workspace) must be re-sent every call
- gRPC — requires .proto schema + code generation, binary encoding, not human-readable, not in MCP spec
- GraphQL — designed for open-ended schema queries, wrong abstraction for a fixed 16-tool surface
- Raw WebSocket — would reinvent JSON-RPC id correlation and notification semantics from scratch

**Why JSON-RPC 2.0 wins beyond compliance**: transport-agnostic (stdio / HTTP+SSE / WebSocket same message format), request/response correlation via id field, human-readable (inspectable with jq), notification support (id=null), no schema compilation step.

**Crate impact**: rust-llm-interface-gateway implements framing and tool dispatch. No other crate is aware of the wire protocol.

---

## D3 — Language Priority Tiers

**Decision**: All 12 languages supported at tree-sitter layer. Investment in semantic depth is tiered.

```text
TIER 1 — Deep extraction, highest test coverage:
  Rust   (.rs)              — tree-sitter + rust-analyzer semantic enrichment
  C      (.c .h)            — tree-sitter + clangd/libclang (header-first)
  C++    (.cpp .hpp)        — tree-sitter + clangd + IWYU
  TypeScript (.ts .tsx)     — tree-sitter + TypeScript compiler API (ts-morph)
  JavaScript (.js .jsx)     — tree-sitter + OXC (native Rust, no subprocess)
  Ruby   (.rb)              — tree-sitter + Prism + rails routes (Rails-aware)

TIER 2 — Structural extraction, lower test priority:
  Python (.py)              — tree-sitter; deeper only if PyO3 boundary present
  Go     (.go)              — tree-sitter; service-boundary peer, rarely FFI-coupled

TIER 3 — Tree-sitter coverage present, not Rust-adjacent:
  Java / C# / Swift / PHP   — supported, not prioritized for semantic depth
```

**Principle**: Support all 12 languages uniformly at tree-sitter layer. Invest in depth proportional to Rust ecosystem coupling patterns.

---

## D4 — Lifecycle and Gateway Requirements Bundle

**Decision**: The following requirements are promoted to explicit V200 scope (from PRD_v173 backlog):

```text
#7   Route prefix nesting              → rust-llm-interface-gateway
#8   Auto port + port file lifecycle   → rust-llm-interface-gateway
#10  Shutdown CLI command              → rust-llm-interface-gateway
#25  XML-tagged response categories    → rust-llm-interface-gateway
#27  Project slug in URL path          → rust-llm-interface-gateway
#28  Slug-aware port file naming       → rust-llm-interface-gateway
#29  Token count at ingest             → rust-llm-store-runtime
#35  Data-flow tree-sitter queries     → rust-llm-tree-extractor  [SEE OQ-X03]
```

**Note**: Item #35 is flagged in OQ-X03 as a contradiction — the FUJ contains no data flow edges. Must be resolved before rust-llm-tree-extractor implementation begins.

**Tauri app stance**: Companion consumer of stable gateway contracts. Not a core crate. Scope = instance manager only (manage workspace processes — HTTP server start/stop, MCP config write, CLI command display). No graph exploration UI.

---

# PART B — OPEN QUESTIONS

Organized by what they block, not by when they were raised.
Every OQ is "not decided" unless explicitly marked otherwise.

---

## TIER 1 — Architecture Blockers

**Must resolve before any crate code is written. These affect crate interfaces, store schema, and the overall system boundary.**

---

### OQ-X01 ★ Is HTTP API in v2.0 scope? [RESOLVED 2026-02-25]

**Decision**: HTTP API is IN v2.0 scope as a first-class transport alongside MCP.

**Binding implementation rule**:
- `rust-llm-interface-gateway` ships one shared handler core and two transport adapters:
  - MCP: JSON-RPC 2.0 over stdio
  - HTTP: request/response over TCP
- Transport layers SHALL only frame/unframe requests and responses.
- No transport-specific business logic is allowed in handler implementations.

**Rationale**:
- Tauri instance-manager workflow depends on port-addressable services.
- Curl/custom tooling and non-MCP consumers require HTTP interoperability.
- Shared contracts avoid MCP/HTTP behavior drift and duplicate logic.

**Impact**:
- Unblocks Tauri process-management OQs (OQ-T01..T10) from a scope perspective.
- Locks architecture to “one binary, shared core, multi-transport adapters”.

---

### OQ-X03 ★ Data flow edges (assign/param/return) — in v2.0 or not?

Decision log D4 item #35 promotes data-flow tree-sitter queries to V200 scope. The FUJ never mentions a dataflow edge anywhere — not in examples, not in contract blocks, not in the Edge Type Reference.

Either the FUJ is incomplete (data flow edges must be added) or item #35 is stale and should be removed. These are opposite conclusions with different implementation costs.

**Working analysis (2026-02-25)**:
- The effort is not binary; it is tiered by language capability.
- Tier 1 (Rust primary): full interprocedural dataflow is feasible in v2.0 using rust-analyzer-enriched symbol resolution plus tree-sitter extraction for assign/param/return edges. Highest precision, highest implementation effort.
- Tier 2 (selected secondary languages: TypeScript/JavaScript, C/C++): shallow-to-mid depth dataflow is feasible in v2.0 using language-server/compiler APIs where available; quality is medium and requires confidence scoring on unresolved symbols/aliases.
- Tier 3 (all remaining languages): syntax-only dataflow extraction (assign/param/return) is feasible, but precision is low without robust semantic/type engines; should be marked heuristic with explicit uncertainty.

**Proposed scope cut (recommended)**:
- v2.0: ship `dataflow::assign|param|return` for Rust at high confidence, and for secondary languages at medium confidence where symbol resolution succeeds.
- v2.0: include capability markers in responses (`flow_capability = full | partial | heuristic`) so downstream MCP/HTTP consumers can reason about trust level.
- v2.1+: deepen interprocedural and alias-aware flow for non-Rust languages, and add security taint propagation on top of stabilized dataflow edges.

**Compiler-level feasibility beyond tree-sitter (2026-02-25)**:
- TypeScript: strong compiler-grade option (`tsc` Program + TypeChecker, `tsserver`/LSP). Accurate flow is feasible with CFG + symbol resolution + module path mapping.
- JavaScript: partial compiler-grade path (`tsc --allowJs` with JSDoc inference). Accuracy is materially lower on dynamic patterns (`eval`, prototype mutation, dynamic import construction).
- C++: strong compiler-grade option (`clangd`/libclang/libTooling + `compile_commands.json`). Hard parts are alias/points-to, templates, macros, and virtual dispatch precision.
- Ruby: no universally reliable compiler-grade static engine for untyped code. Sorbet/Steep can provide type-level semantics where projects adopt annotations/RBIs.
- Rails: framework-heavy dynamic metaprogramming requires runtime boot/reflection (`rails routes`, ActiveRecord association introspection) for credible flow mapping.

**External analyzer federation option (2026-02-25)**:
- We can ingest outputs from mature analyzers (e.g., CodeQL/Semgrep/Brakeman/clang analyzers) as enrichment inputs into the Parseltongue graph.
- Recommended contract: external results are stored as `evidence` edges/findings with provenance (`tool`, `version`, `rule_id`, `timestamp`, `confidence`) rather than silently promoted to canonical core edges.
- Promotion rule for “accurate-only mode”: an external flow may be promoted to first-class graph edge only if entity-key mapping is exact and replay tests pass on a pinned corpus.
- This gives fast non-Rust coverage without pretending unsupported precision; trust is preserved by explicit provenance and capability markers.

**Local evidence check (competitor_research) — mapping viability (2026-02-25)**:
- Verified local repos: `competitor_research/mcp-grep-servers/semgrep-mcp` and `.../mcp-server-semgrep`.
- Observed semgrep finding tuple in local wrappers:
  - `check_id` (rule identity)
  - `path`
  - `start.line`, `start.col`
  - `end.line`, `end.col`
  - `extra.severity`, `extra.message`
- The wrappers already use this tuple for filtering/diffing and SARIF export, which confirms stable location-addressable findings in practice.
- Mapping implication for Parseltongue:
  - `path + start.line + start.col + end.line + end.col` is sufficient to map to file/span and then resolve to an internal `EntityKey` via AST range lookup.
  - Rule/finding identity (`check_id`) should remain an attached annotation key, not part of canonical entity identity.
- Gating rule: if a finding span overlaps multiple entities or zero entities, do not force-map; retain as file-level evidence with `mapping_status=ambiguous|unmapped`.

Blocks: rust-llm-tree-extractor TDD stub design, Edge Type Reference completeness.

---

### OQ-X04 ★ V2.0 CLI binary name [RESOLVED 2026-02-25]

**Decision**: Canonical user-facing binary name remains `parseltongue`.

**Naming rule**:
- Official docs, FUJ flows, Tauri UI command display, and onboarding prompts MUST use `parseltongue`.
- Optional shorthand alias `pt` may exist as a convenience, but is non-canonical.
- Crate/internal executable naming (`rust-llm-interface-gateway`) is implementation detail, not the user contract.

**Rationale**:
- Preserves continuity with v1.x scripts and user muscle memory.
- Avoids launch-time command churn across docs, prompts, and automation.

**Impact**:
- Unblocks Tauri UI labels, installation docs, and command-copy UX.

---

### OQ-X06 ★ Full-file ingest observability contract (coverage truth)

Current docs define parsing behavior and endpoint outputs, but not a hard accountability rule for "what exactly was seen vs skipped vs parsed."
Without this, we cannot prove ingestion fidelity, and FUJ metrics can look healthy while blind spots exist.

Decision needed:
- Discovery source of truth SHALL be git-aware file inventory:
  - `git ls-files -co --exclude-standard`
- Every discovered file SHALL end a run in exactly one top-level bucket:
  - `docs`
  - `non_eligible_text` (unsupported language/extension or non-code text)
  - `identifiable_tests`
  - `code_graph`
- Files in `code_graph` SHALL have sub-status:
  - `parsed`
  - `parse_failed(reason_code)`
- A persistent ledger SHALL track at least:
  - `path`, `bucket`, `sub_status`, `reason_code`, `parser`, `checksum`, `last_seen_run_id`
- Quality gate: `unexplained_files = 0` (no silently dropped files).

Recommended direction:
- Treat this as a Tier-1 architectural contract and make FUJ-v2 ingest flow explicitly dependent on it.
- All higher-level analysis claims (blast radius, SCC, taint/dataflow enrichment) are valid only after this gate passes.

Blocks: FUJ-v2 ingest contract, store schema for ingestion observability, success metrics credibility.

---

### OQ-X07 ★ External index sidecar adoption (codemogger) for Big-Rock-01

Local research clone: `CR09/codemogger` (git: `glommer/codemogger`).

What this gives us immediately (proven in code):
- Incremental per-file ingestion with checksum tracking (`indexed_files.file_hash`, stale-file removal, changed-only reprocessing).
- Multi-language AST chunking via tree-sitter WASM (Rust/TS/JS/C/C++/Go/Python/Ruby/etc.).
- Fast local retrieval path (FTS + vector + hybrid RRF) with MCP tools (`index`, `search`, `reindex`).
- Zero-server local runtime model (single SQLite file per project).

Where it does **not** satisfy Big-Rock-01 as-is:
- No hard terminal-class ledger for every discovered file (`docs | non_eligible_text | identifiable_tests | code_graph`).
- Simplified ignore semantics and hidden-file skipping can create silent blind spots.
- Identity key is line-range based (`file:start:end`) and is unstable under edits; not compatible with canonical EntityKey rules.
- No graph edge model (calls/imports/boundaries), no truth-tiering (`verified/heuristic/rejected`), no conflict quarantine.

Decision options:
- (a) **Reference-only**: borrow patterns (hash ledger, stale removal, MCP ergonomics), reimplement inside V200 crates.
- (b) **Sidecar federation**: ingest codemogger chunks as `evidence` records (search acceleration), never as canonical graph facts.
- (c) **Replace ingestion core with codemogger**: fastest initial velocity but violates current canonical key + truth-contract requirements.

Recommended direction:
- Choose **(b)** now. Use codemogger as a non-canonical sidecar index for discovery UX and fallback retrieval.
- Keep Parseltongue ingestion ledger + canonical graph as source of truth.
- Add a mapping gate: sidecar records are only promoted if path/span maps unambiguously to canonical EntityKey and pass truth checks.

Good candidate feature for AR000/FUJ-v2:
- `Ingestion Explorer + Evidence Search`:
  - Explorer view/API: complete per-run file accounting with bucket + reason.
  - Evidence search tool: semantic lookup across sidecar chunks, returned with provenance tag `source=external_sidecar`.
  - Promotion pipeline: explicit user/agent action to convert evidence into canonical graph facts after validation.

Blocks: Big-Rock-01 contract closure, ingestion schema boundaries, FUJ-v2 ingest/query separation.

---

### OQ-I01 ★ shared_context and public_module_context double-storage

Every public entity pair in a file receives both a shared_context edge AND a public_module_context edge. This doubles storage and traversal cost for public entities. It also makes query results ambiguous — a caller filtering by edge kind gets public_module_context; one filtering by shared_context also gets the same pair.

Options:
- (a) emit shared_context only for pairs where at least one entity is private; emit public_module_context only for public-to-public pairs (no overlap, halves edge count)
- (b) accept double storage; filter by edge kind at query time

Option (a) is cleaner and is the recommended resolution. Requires a single conditional in emit_shared_context_pair.

Blocks: rust-llm-store-runtime schema design, emit_shared_context_pair contract.

---

### OQ-P01 ★ shared_context edge cap per file

emit_shared_context_pair emits for ALL entity pairs in the same file — O(n²). A 500-entity file (not unusual in generated code) produces 124,750 shared_context edges. No cap is specified. An engineer implements exactly the spec and ships an OOM crash on day 1.

Decision needed: maximum entity count per file before shared_context emission is skipped or sampled. Proposed: if entity count > 150, emit shared_context only for public entities. If public entity count also > 150, skip shared_context for that file entirely and emit a diagnostic.

Blocks: rust-llm-tree-extractor implementation, store capacity planning.

---

### OQ-P02 ★ public_module_context edge cap per file

Same O(n²) problem as OQ-P01, applied to public entity pairs only. A file with 50 public entities produces 1,225 public_module_context edges. Cap needed by the same mechanism.

Blocks: same as OQ-P01.

---

### OQ-T01 ★ Tauri: what is the unit of management?

If a user starts an HTTP server from terminal AND clicks Start in Tauri for the same workspace, Tauri cannot detect the externally-spawned process by workspace path alone. Options:
- Tauri owns only processes it spawned; ignores external ones (can cause port conflict)
- Tauri reads the port file written by the CLI (item #8) to detect any running instance before starting
- Tauri polls the HTTP health endpoint before spawning

Port file approach aligns with D4 item #8 and is cleanest. Tauri reads the port file to detect a running instance; if found, shows "running (external)" status and disables the Start button.

Blocks: all other OQ-T items, Tauri architecture.

---

### OQ-T02 ★ Tauri: process lifecycle on app close

If Tauri owns the HTTP server child processes (OQ-T01 resolution), what happens when the Tauri window is closed?
- Option A: child processes die (Tauri default). User loses running servers.
- Option B: processes are spawned detached (daemonized). Tauri re-attaches via PID file on next launch.
- Option C: prompt on close — "3 servers are running. Keep alive?" → detach if yes, kill if no.

Option C is the correct product behavior. Requires PID file management per workspace alongside the port file.

Blocks: OQ-T09 (system tray), Tauri process management implementation.

---

## TIER 2 — Implementation Contract Blockers

**Must resolve before TDD stubs are written for affected crates. These define exact types, behaviors, and error codes that contracts depend on.**

---

### OQ-E01 ★ MCP error code table

Every MCP tool in the FUJ only shows success responses. No error code table exists. Engineers will invent error behavior without a spec. JSON-RPC application errors live in the -32000 to -32099 range by convention.

Minimum set needed:

```text
-32001  EntityNotFound        key does not exist in store
-32002  StoreNotReady         no workspace loaded / store not open
-32003  IngestInProgress      concurrent ingest guard (see OQ-E04)
-32004  IngestFailed          ingest aborted, partial state rolled back
-32005  FileMoved             entity key valid but file deleted/moved
-32006  PermissionDenied      cannot read source file (Gate G3)
-32007  WorkspaceNotFound     --workspace path does not exist
-32008  HopsOutOfRange        max_hops < 1 or > 10
-32009  EmptyQuery            fuzzy search query is empty string
-32010  IngestRequired        store exists but schema version mismatch
```

Blocks: all MCP tool handler implementations, client error handling.

---

### OQ-E04 Concurrent ingest guard

Two simultaneous ingest triggers on the same workspace (MCP tools/call + Tauri button) produce undefined behavior. The single-write-path contract prevents corrupt writes but does not serialize callers.

Recommended resolution: return -32003 IngestInProgress immediately to the second caller. Document this as specified behavior. The first ingest completes normally.

Blocks: rust-llm-interface-gateway ingest handler.

---

### OQ-E03 rust-analyzer enrichment timeout

No timeout specified for rust-analyzer semantic enrichment. A hanging enrichment blocks the entire ingest. Recommended: 5-second timeout per file. On timeout, store the entity with tree-sitter-only data and emit a stderr warning. Enrichment failure must never block storage of already-extracted entities.

Blocks: rust-llm-rust-semantics implementation.

---

### OQ-E05 Gate G3 + stale line numbers after file change

Gate G3 mandates live disk reads for entity source. But start_line/end_line are stored at ingest time. If the file changes between ingest and query, stored line numbers may point to wrong lines. The file watcher detects changes but re-ingest is async.

Recommended resolution: when the file watcher marks a file as changed, invalidate start_line/end_line for all entities in that file. get_entity_detail_live() for an invalidated entity returns the full file content with a warning: "Line range may be stale — file changed since last ingest."

Blocks: rust-llm-store-runtime file watcher integration contract.

---

### OQ-E02 uncertain=true edges — what does Claude do with them?

The FUJ specifies uncertain: true for cross-language edges with confidence 0.60–0.79. But the MCP tool description (what Claude reads) never instructs Claude how to handle this flag. Claude may silently ignore it or hallucinate behavior.

Decision needed: (a) tool description must explicitly say "edges with uncertain=true are low-confidence; surface this caveat to the user," and (b) default behavior of list_cross_language_all should filter to confidence >= 0.80 unless the caller sets min_confidence lower.

Blocks: MCP tool description authoring, Claude system prompt.

---

### OQ-E06 Zero-entity ingest response

If parseltongue ingests a folder with no parseable files, the current FUJ response format implies a success with 0 counts. This will confuse users who selected the wrong folder.

Recommended: return a structured warning in the MCP response listing: files scanned, file types found, languages detected, and a suggestion to verify the workspace path.

Blocks: rust-llm-interface-gateway ingest response formatting.

---

### OQ-I02 TypeScript EntityKey scope field construction rule

FUJ shows scope = `frontend.api` for `frontend/src/api/auth.ts`. No rule is stated for how scope is derived from a TypeScript file path. Is it the directory path with `/` replaced by `.`? The module path? What about files in the root?

Recommended rule: scope = relative directory path from project root with `/` replaced by `.` and `src/` stripped if present. Root-level files get scope = project folder name.

Blocks: build_entity_key_canonical() implementation for TypeScript.

---

### OQ-I03 Anonymous function EntityKey

Arrow functions assigned to variables (`const handler = async (req, res) => {}`) have no function name. The EntityKey name segment is undefined.

Recommended: use the variable name the arrow function is assigned to, as detected by tree-sitter. If not assigned to a named variable (e.g., inline callback), use `anonymous_N` where N is 0-indexed per file. Document that anonymous entities have reduced blast-radius utility.

Blocks: rust-llm-tree-extractor TypeScript extraction.

---

### OQ-I04 CommonJS vs ESM visibility detection

detect_entity_visibility_flag() handles `export` keyword for ESM. CommonJS uses `module.exports` or `exports.foo`. Many real Node.js files use CommonJS.

Recommended: treat any assignment to `module.exports` or `exports.*` as is_public=true. Default all CommonJS module-level function and class declarations to is_public=true (CommonJS has no private concept at module level). Document this approximation.

Blocks: detect_entity_visibility_flag() implementation for JavaScript.

---

### OQ-I05 Rust use imports and "import feeds fn" semantics

The public_module_context spec marks Rust `use` imports with note="import feeds co-located fns." In TypeScript, `import` directly feeds specific functions in the same file. In Rust, `use` feeds the entire module — mapping to specific functions is an approximation.

Recommended: drop the "import feeds fn" note for Rust `use` statements. Apply it only to TypeScript/JavaScript `import` statements where the feeding relationship is file-scoped and accurate.

Blocks: emit_public_module_context() implementation.

---

### OQ-I06 rust-analyzer workspace root discovery

rust-llm-rust-semantics needs a Cargo workspace root. In a polyglot repo, the Rust root is a subdirectory (e.g., `backend/`), not the project root. No discovery algorithm is specified.

Recommended: during file walk in rust-llm-tree-extractor, collect all Cargo.toml paths. Pass the top-level Cargo.toml (workspace root) to rust-llm-rust-semantics. If multiple non-member Cargo.toml files exist (separate Rust crates), run a rust-analyzer instance per root.

Blocks: rust-llm-rust-semantics initialization, rust-llm-tree-extractor → rust-llm-rust-semantics interface.

---

### OQ-I08 Language enum Unknown variant and unrecognized files

If a file extension is not recognized, what Language variant is assigned? This affects store schema (Language is a crate boundary type) and coverage diagnostics.

Recommended: skip unrecognized files entirely at extraction time (do not emit entities). Count them in the coverage diagnostics as "skipped: unrecognized extension." Language::Unknown is valid only as a Datalog placeholder, never for new entity creation.

Blocks: rust-llm-core-foundation Language enum definition.

---

### OQ-I10 MCP server restart behavior

If Claude Desktop restarts parseltongue-mcp, does it: (a) open the existing store and resume query-ready immediately, or (b) require a fresh ingest call?

Option (a) is strongly preferred. Requires: store is always in a consistent state on disk (no partial writes), and a schema version check on startup. If schema version mismatches, return -32010 IngestRequired on any query tool call.

Blocks: rust-llm-interface-gateway startup sequence, rust-llm-store-runtime schema versioning.

---

### OQ-I11 ★ Pre-computed caller/callee names vs boolean tags in entity store

The Supermodel competitive analysis (THESIS-supermodel-competitive-analysis.md) surfaces a concrete design question: Supermodel bakes boolean auto-tags (`High-Dependency`, `Many-Imports`, `Complex`, `Isolated`) onto every entity node at build time to serve faceted search UI filtering. Should v200 do the same?

**Verdict: No. Wrong abstraction for our user.**

Supermodel's user is a developer browsing a static website. Faceted tag filtering (`"show me Complex entities"`) is appropriate for a human search UI. Parseltongue's user is an AI agent. An AI agent does not filter on boolean labels. An AI agent asks: *"who calls this function?"* and needs names to reason about blast radius — not a `High-Dependency` flag.

**The correct principle (steal) vs the wrong artifact (skip):**
- STEAL: pre-compute relationship summaries at ingest time so no follow-up graph traversal is needed at query time
- SKIP: boolean threshold tags (`caller_count >= 5 → "High-Dependency"`) — these discard information; the scalar is already better

**Decision needed**: adopt the following schema additions to the entity record in rust-llm-store-runtime:

```
top_callers:   [EntityKey; 0..10]   -- entities that call this one, sorted by call frequency desc
top_callees:   [EntityKey; 0..10]   -- entities this one calls, sorted by call frequency desc
caller_count:  u32                  -- total callers (not capped)
callee_count:  u32                  -- total callees (not capped)
is_isolated:   bool                 -- caller_count + callee_count == 0
```

`fn_count` already exists. `is_isolated` is the only boolean because "zero edges" is categorically different behavior, not a threshold approximation.

**MCP tool response contract change**: every tool that returns entity data must include `top_callers` and `top_callees` inline. The AI gets caller/callee names in the first call — no follow-up `reverse_callers_query_graph` needed for the common case.

**New endpoint (Tier 3, not blocking)**:
```
GET /entities-by-coupling-rank?min_callers=5&top=20
```
Returns entities sorted by `caller_count DESC` where `caller_count >= min_callers`. Replaces any need for tag-based filtering with a real query. Pure CozoDB; no new schema needed beyond the scalar `caller_count`.

Options:
- (a) adopt top_callers/top_callees arrays + scalars as above (recommended)
- (b) also add boolean tags as a UI convenience layer on top — not recommended (adds redundant data, maintenance burden, and no query value for AI agents)
- (c) skip pre-computation entirely; always traverse graph at query time — not recommended (defeats the purpose of the store)

Blocks: rust-llm-store-runtime entity schema, rust-llm-tree-extractor entity emission contract, MCP tool response shapes in rust-llm-interface-gateway.

---

### OQ-P03 Cross-language boundary detection pre-filter

detect_http_boundary_edge() runs over all (RouteSignal, FetchSignal) pairs. At 500 Rust route handlers × 2,000 TypeScript fetch call sites = 1,000,000 pair comparisons per ingest. No pre-filter is specified.

Recommended: pre-filter by URL path segment overlap before scoring. Only pass pairs where route_path and url_pattern share at least one non-trivial path segment. Reduces candidate set by ~95%.

Blocks: rust-llm-cross-boundaries implementation, ingest performance.

---

### OQ-P05 Ascent Datalog max_hops — runtime value in compile-time program

The FUJ blast radius rule references max_hops as a runtime variable, but Ascent compiles rules at build time. How does max_hops get into the compiled program?

Recommended: compile with a fixed ceiling (e.g., max_hops=10). Pass the user's requested max as a Datalog fact: `relation max_hops(u32)`. The rule reads it: `(n < max_hops)` where max_hops is drawn from the relation. Post-filter results to the user's requested N.

Blocks: rust-llm-graph-reasoning Ascent program design.

---

### OQ-P06 Ascent base relations — eager load or lazy?

On startup, does rust-llm-graph-reasoning load ALL entity() and edge() facts into memory (eager), or only at query time (lazy)?

Recommended: lazy per-query load for the MCP server (always-on process — large graphs would consume too much idle memory). Cache the last-loaded fact set with an invalidation flag set by the file watcher. Reload only when the flag is set.

Blocks: rust-llm-graph-reasoning startup and query architecture.

---

### OQ-T03 Tauri: port assignment strategy

Who assigns the HTTP server port per workspace?

Recommended: the CLI binary assigns the port (auto-incrementing from 7777 per workspace, checking availability) and writes it to the port file (D4 item #8). Tauri reads the port file after spawn — it never assigns ports itself. This makes the CLI the single source of truth for port state.

Blocks: OQ-T08 (workspace persistence), Tauri spawn logic.

---

### OQ-T05 Tauri: ingest progress protocol

When Tauri spawns the ingest command as a child process, how does it show per-language progress? If the CLI only emits human-readable logs to stderr, Tauri can only show a raw log tail — no progress bars.

Recommended: the CLI binary emits structured JSON lines to stdout during ingest:
`{"type":"progress","lang":"rust","files_done":234,"files_total":312,"entities":1089}`
Tauri parses these and renders per-language progress bars. This is a new contract between CLI and Tauri that must be specced before either is implemented.

Blocks: rust-llm-interface-gateway ingest stdout protocol, Tauri progress UI.

---

## TIER 3 — Launch and Messaging (Resolve During Implementation)

**These do not block implementation but will cause launch problems if unresolved.**

---

### OQ-X02 Token savings headline figure — 94.6% or 99%?

FUJ ingest response shows 94.6% savings. V1.x README claims 99%. These measure different scenarios (entity-only tokens vs smart-context tokens). At launch, one number goes on the landing page. The wrong one will be used without an explicit decision.

Decision needed: define the measurement methodology and the canonical headline number. Suggested: "up to 99% reduction when using smart-context endpoint; ~94% reduction for full entity list."

---

### OQ-X05 Ruby has Tier 1 priority but no FUJ presence

Language tier decision places Ruby as Tier 1. Every FUJ example uses Rust + TypeScript. No Ruby EntityKey, no Rails route example, no Ruby tree-sitter query pattern appears in the 2,400-line document.

Decision needed: add at least one Ruby/Rails example to the FUJ (e.g., a Rails controller action as a third entity in the cross-language graph), or formally document that FUJ uses Rust+TS as the canonical example and Ruby coverage is tested separately.

---

### OQ-E02-UX uncertain=true user-facing copy

Beyond the Claude instruction (covered in Tier 2), the actual copy Claude shows the user for uncertain edges needs to be defined. E.g., "I found a likely HTTP connection between these two functions (medium confidence — the URL patterns match but the HTTP method couldn't be confirmed)."

---

### OQ-I07 Project slug uniqueness guarantee

Two users each with a folder named `myapp` at different paths will produce the same slug prefix. Timestamp suffix makes them unique per ingest session but confusing for humans.

Recommended slug format: `{folder_name}-{YYYYMMDD}-{HHMMSS}` where folder_name is the last path component, lowercased, non-alphanumeric replaced by hyphens. Document that uniqueness is timestamp-based only and the folder_name component may collide.

---

### OQ-I09 Four-word naming final arbitration pass

All function and MCP tool names must be exactly 4 underscore-delimited words (Design101). A final pass over all 30 names (14 contract functions + 16 MCP tools) is needed before implementation. Known ambiguities: "detail_view" (compound noun, counts as 1 or 2?), "cross_language" (compound).

Rule: each underscore-delimited segment = exactly 1 word regardless of compound nouns. Run the pass; fix any violations before TDD stubs are written.

---

### OQ-P04 Blast radius latency spec baseline

Success metric "blast radius 3 hops < 200ms" is not tied to a graph size. Needs a baseline: "< 200ms on a graph with up to 10,000 entities and 50,000 edges on Apple M-class hardware." Graphs above this threshold may return paginated results or a timeout warning.

---

### OQ-T06 Delta ingest vs full re-ingest button distinction

If the file watcher handles live changes automatically, the [Re-ingest] button only recovers from corrupted state or large offline batch changes. Consider two separate actions: [Re-ingest All] (full, slow, safe) and [Resume Watching] (re-attach watcher after the process was stopped). Naming must be clear enough that users never click the wrong one.

---

### OQ-T07 Tauri HTTP API query boundary rule

Tauri workspace card may call health and stats endpoints (/server-health-check-status, /codebase-statistics-overview-summary, /file-watcher-status-check). Explicit rule needed to prevent scope creep:

Tauri MAY call: any endpoint returning aggregate status (counts, timestamps, health booleans).
Tauri MAY NOT call: any endpoint returning entity-level data (search results, entity detail, graph traversal).

---

### OQ-T08 Workspace persistence store location

The registered workspace list (paths, port assignments, PID state) must persist across Tauri restarts. Recommended: read the port files written by the CLI binary (D4 item #8) as the primary source of truth. Tauri maintains only a lightweight registry of user-registered workspace paths in tauri-plugin-store. On launch, cross-reference registry paths against existing port files to reconstruct running state.

---

### OQ-T09 System tray requirement

If Tauri owns child processes (OQ-T02 resolution: prompt on close, detach if user says keep-alive), a system tray icon is needed so the user can reopen the management window without relaunching. Requires tauri-plugin-system-tray. Not a heavy lift but must be in the Tauri feature spec before implementation.

---

### OQ-T10 macOS sandbox and code signing for child process spawning

Tauri spawning an external binary (the parseltongue CLI) on macOS requires explicit shell scope in tauri.conf.json. If the binary is not bundled as a Tauri sidecar, the user may see a Gatekeeper prompt on first run. Allowlist must be explicit (binary path pattern, argument patterns). Decide: bundle as sidecar (simpler security model) or require separate install (more flexible).

---

## APPENDIX — OQ Resolution Priority Order

```text
ALREADY RESOLVED (2026-02-25):
  OQ-X01  HTTP is in v2.0 scope (shared MCP/HTTP handler core)
  OQ-X04  Canonical binary name = parseltongue (pt optional alias)

MUST RESOLVE FIRST (blocks everything):
  OQ-X03  Data flow edges in or out?
  OQ-X06  Full-file ingest observability contract
  OQ-X07  External sidecar adoption boundary (codemogger)
  OQ-I01  Double-storage of shared_context + pub_mod_ctx
  OQ-T01  Tauri: unit of management (process vs workspace)

RESOLVE BEFORE CRATE STUBS:
  OQ-P01  shared_context edge cap
  OQ-P02  public_module_context edge cap
  OQ-E01  MCP error code table
  OQ-I08  Language enum Unknown variant
  OQ-T02  Tauri process lifecycle on close

RESOLVE BEFORE AFFECTED CRATE IMPLEMENTATION:
  OQ-E04  Concurrent ingest guard         → interface-gateway
  OQ-E03  rust-analyzer timeout           → rust-semantics
  OQ-E05  Gate G3 + stale line numbers    → store-runtime
  OQ-I02  TypeScript scope rule           → tree-extractor
  OQ-I03  Anonymous function EntityKey    → tree-extractor
  OQ-I04  CommonJS visibility detection   → tree-extractor
  OQ-I05  Rust use import semantics       → tree-extractor
  OQ-I06  rust-analyzer workspace root    → rust-semantics
  OQ-I10  MCP server restart behavior     → interface-gateway
  OQ-I11  top_callers/callees vs tags     → store-runtime + tree-extractor + gateway
  OQ-P03  Boundary detection pre-filter   → cross-boundaries
  OQ-P05  Ascent max_hops mechanism       → graph-reasoning
  OQ-P06  Ascent eager vs lazy load       → graph-reasoning
  OQ-T03  Tauri port assignment           → Tauri app
  OQ-T05  Tauri ingest progress protocol  → interface-gateway + Tauri app

RESOLVE BEFORE LAUNCH (not blocking implementation):
  OQ-X02  Token savings headline
  OQ-X05  Ruby FUJ example
  OQ-E02  uncertain=true user copy
  OQ-E06  Zero-entity response copy
  OQ-I07  Slug uniqueness documentation
  OQ-I09  Four-word naming final pass
  OQ-P04  Blast radius latency baseline
  OQ-T06  Re-ingest button naming
  OQ-T07  Tauri HTTP query boundary rule
  OQ-T08  Workspace persistence store
  OQ-T09  System tray decision
  OQ-T10  macOS sidecar vs external binary
```
