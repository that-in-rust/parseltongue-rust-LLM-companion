# AR054: Codemogger Analysis for Parseltongue V200 PRD
Date: 2026-02-27
Status: PRD input (competitor study, code-grounded)
Source studied: `/Users/amuldotexe/Downloads/glommer-codemogger-8a5edab282632443.txt`

## Purpose
Provide a codemogger-specific decision input for `ES-V200-Decision-log-01.md` with:
1. what to adopt directly,
2. what to adapt as optional evidence lane,
3. what to explicitly reject for canonical truth graph architecture.

## Executive Summary
Codemogger is strongest as a **local retrieval engine**:
- AST chunking across many languages,
- incremental hash-based reindex,
- keyword + semantic + hybrid (RRF) search,
- lightweight MCP surface (`index`, `search`, `reindex`).

It is weaker for V200 core goals:
- no canonical entity identity beyond line-span chunk keys,
- no dependency graph truth contracts,
- no typed edge reasoning for blast radius/architecture workflows.

Best V200 strategy:
- **Borrow retrieval ergonomics and throughput patterns**,
- **keep Parseltongue graph contracts as source of truth**.

## What Codemogger Does Well (Evidence from code)

1. **Pipelined indexing phases with stale-embed batching** (`src/index.ts`)
- scan -> hash-check -> chunk-write -> stale-embed -> stale-delete -> FTS rebuild.
- embedding runs in bounded batches (`EMBED_BATCH`) to avoid memory spikes.

2. **Incremental correctness basics** (`src/index.ts`, `src/db/store.ts`)
- unchanged file skip by SHA-256,
- deleted file cleanup (`removeStaleFiles`),
- embedding invalidation on chunk upsert.

3. **Hybrid retrieval with robust fusion** (`src/search/rank.ts`)
- reciprocal rank fusion over keyword (FTS) and vector results.
- avoids fragile score normalization between BM25-like and cosine domains.

4. **Keyword query preprocessing** (`src/search/query.ts`)
- stopword/filler stripping and term cap for FTS compatibility.
- practical for conversational agent prompts.

5. **Operational guardrail: searchable-state verification** (`verifySearchable` in `src/index.ts`)
- catches large DB with no visible chunks (lock/WAL mismatch style failure).

6. **MCP usability pattern** (`src/mcp.ts`)
- minimal tool set with explicit maintenance behavior (`reindex after edits`).
- tool description updated dynamically based on current indexed state.

## Gaps vs V200 Requirements

1. **Identity stability gap**
- chunk identity is `file:start:end`; line shifts can churn identities.
- incompatible with BR01 stable canonical identity objective.

2. **Truth-grade/provenance gap**
- no explicit `verified/heuristic/rejected` fact model.
- retrieval scores are ranking signals, not truth contracts.

3. **Graph reasoning gap**
- lacks first-class dependency/call graph analysis surface.
- cannot directly satisfy BR02/BR03 architecture and impact trust targets.

4. **File accountability gap**
- simplified `.gitignore` handling and hidden-file skipping in walker.
- weaker fit for BR01 "0 silent drops" accountability invariants.

## V200 Option Cards (Codemogger-inspired)

### Option A: Retrieval Sidecar Lane (Recommended)
Targets: BR02, BR03, BR06

Adopt:
- hybrid retrieval (`semantic|keyword|hybrid`) for discovery endpoints only.
- RRF merge for candidate generation and context packing.

Guardrails:
- never emit `verified` from sidecar alone.
- mark sidecar hits as `evidence` until mapped and promoted.

Why:
- improves recall for fuzzy asks without polluting canonical graph truth.

### Option B: Query Preprocessing for FTS Endpoints (Recommended)
Targets: BR02

Adopt:
- keyword extraction from conversational prompts before FTS lookup.
- max-term caps and de-noising for stable latency.

Guardrails:
- keep raw query available for audit/debug.
- return both `raw_query` and `processed_query` in metadata when `debug=true`.

Why:
- improves practical retrieval quality in agent-driven natural language queries.

### Option C: Searchability Self-Check Contract (Recommended)
Targets: BR01, BR07

Adopt:
- startup/runtime guard that detects "index exists but not searchable" states.

Proposed behavior:
- expose health fields:
  - `index_visible_chunks`,
  - `index_file_size_bytes`,
  - `searchable_state=ok|degraded|blocked`,
  - `degrade_reason`.

Why:
- converts silent runtime failure into explicit operational state.

### Option D: Phase-Bounded Embed/Write Pipeline (Recommended)
Targets: BR07, POL-D2

Adopt:
- explicit bounded batch sizes for parse/write/embed phases.
- keep stale embedding queue separate from chunk ingest writes.

Why:
- better memory predictability and smoother latency envelopes at scale.

### Option E: Dynamic MCP Capability Signaling (Conditional)
Targets: BR04

Adopt:
- dynamic MCP tool descriptions/capability hints based on index state.

Guardrails:
- keep business semantics centralized in BR02 contracts, not MCP text.
- treat this as UX layer only.

Why:
- improves operator and agent discoverability with low implementation risk.

### Option F: Per-Codebase Lexical Index Strategy (Conditional)
Targets: BR02, BR07

Adopt:
- scoped lexical indexes for faster keyword queries by workspace/scope.

Guardrails:
- do not replace canonical graph storage with lexical index structures.

Why:
- practical latency win for identifier-driven lookups.

## Explicit Non-Adoption List (for V200 canonical core)

1. Do not adopt line-span-only chunk keys as canonical entity identity.
2. Do not treat vector similarity as dependency truth.
3. Do not let simplified ignore semantics define BR01 file accountability.
4. Do not collapse full graph endpoints into retrieval-only API shape.

## Proposed Decision-Log Additions

1. `BR02-D10` (new): Discovery endpoints may run `hybrid retrieval lane` with explicit `match_source` and `truth_grade`.
2. `BR01-D9` (new): Health endpoints must expose `searchable_state` and degraded reasons (no silent unreadable index states).
3. `BR07-D6` (new): Ingestion/embedding phases run with bounded batch controls and phase telemetry.
4. `BR04-D8` (new): MCP capability hints may be dynamic, but semantics remain contract-driven by shared getter layer.

## Suggested V200 Sequencing

1. Ship now:
- Option A (sidecar lane contract scaffolding),
- Option B (query preprocessing),
- Option C (searchability guard),
- Option D (batch-bound pipeline).

2. Ship after:
- Option E/F once BR01-BR03 trust contracts are closed.

## Practical PRD Message
Codemogger validates that local-first, fast, hybrid retrieval is valuable for agent UX.
Parseltongue V200 should absorb that retrieval strength as an **evidence lane**, while preserving its core differentiator: **truthful dependency graph reasoning with explicit confidence and provenance contracts**.
