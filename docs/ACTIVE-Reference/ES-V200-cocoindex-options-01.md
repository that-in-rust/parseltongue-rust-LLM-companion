# ES-V200 Option Set from Cocoindex Code Study
Date: 2026-02-27
Inputs studied:
- `/Users/amuldotexe/Downloads/cocoindex-io-cocoindex-code-8a5edab282632443.txt`
- `/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/ACTIVE-PRD/ES-V200-Decision-log-01.md`

## Shreyas-style framing (Leverage vs Overhead)

High leverage to copy:
- zero-config root discovery and local state bootstrap (`config.py`)
- explicit freshness control (`refresh_index` default true in `server.py`)
- lock-protected incremental refresh (`_index_lock` in `server.py`)
- very small "overview first, detail on demand" query envelope

Low leverage or dangerous to copy blindly:
- reducing product surface to only semantic chunk search
- letting embedding similarity masquerade as truth
- mixing docs/text chunk retrieval with canonical graph truth without tiering

## Evidence extracted from cocoindex codebase

1. Root discovery order is deterministic (`.cocoindex_code` -> `.git` -> `cwd`) in `src/cocoindex_code/config.py`.
2. One local state folder (`.cocoindex_code`) stores both index DB and runtime state (`config.py`, `shared.py`).
3. Query freshness is explicit (`refresh_index: bool = True`) in `src/cocoindex_code/server.py`.
4. Concurrent index updates are serialized by `asyncio.Lock` in `src/cocoindex_code/server.py`.
5. Startup does not block on full indexing; refresh runs in background in `_async_serve`.
6. E2E tests validate add/modify/delete incremental behavior (`tests/test_e2e.py`).

## Option Cards for ES-V200 Decision Log

### Option 1: Zero-Config Workspace Boot (Recommended)
Decision log targets:
- BR01 (ingestion truth loop)
- BR04 (operator surface)

Additions to pick:
- auto root discovery precedence: `.parseltongue/` marker -> `.git/` -> `cwd`
- auto-create workspace state dir on first run
- explicit "resolved_root_path" in health and coverage payloads

Why this is strong:
- kills first-run friction and wrong-root ambiguity fast
- directly supports BR01 no-silent-drop accountability

Proposed new decisions:
- `BR01-D8`: ingestion run must record resolved workspace root and discovery reason
- `BR04-D6`: Tauri and CLI must show identical resolved root before ingest starts

### Option 2: Freshness Contract + Singleflight Reindex (Recommended)
Decision log targets:
- BR02 (query trust)
- BR07 (performance envelope)

Additions to pick:
- `freshness_mode` per query (`strict`, `best_effort`, `stale_ok`)
- singleflight lock for reindex to prevent overlapping ingest corruption
- query response includes `index_state` (`fresh`, `refreshing`, `stale`)

Why this is strong:
- user gets explicit latency vs freshness tradeoff
- prevents hidden races in always-on watcher mode

Proposed new decisions:
- `BR02-D7`: every read response must carry index freshness metadata
- `BR07-D5`: only one active merge/write lane per workspace snapshot

### Option 3: Two-Layer Response Envelope (Recommended)
Decision log targets:
- BR02 (query contracts)

Additions to pick:
- default "locator" mode: key, path, span, confidence, truth_grade
- optional "expanded" mode for body/context
- endpoint-level token estimate field (`estimated_tokens`)

Why this is strong:
- preserves token budget without losing trust annotations
- gives MCP agents predictable low-cost first hop

Proposed new decisions:
- `BR02-D8`: all endpoints support compact locator envelope by default
- `BR02-D9`: expanded payloads must be opt-in

### Option 4: Evidence Search Sidecar, Not Canonical Truth (Conditional)
Decision log targets:
- BR03 (compiler truth + LLM judgment)
- BR06 (external evidence federation)

Additions to pick:
- optional embedding/chunk retrieval lane for recall and discovery
- all vector hits enter as `evidence` facts unless promoted
- promotion requires canonical entity mapping + conflict checks

Why this is strong:
- gives semantic recall without polluting core graph truth model
- aligns tightly with BR06 promotion gate philosophy

Risk:
- scope creep into "another search engine"

Proposed guardrail:
- do not allow sidecar hits to emit `verified` truth grade directly

### Option 5: Capability Manifest Endpoint (Recommended)
Decision log targets:
- BR03
- BR05

Additions to pick:
- `/capability-profile-runtime` endpoint:
  - language tier coverage
  - toolchain availability (tree-sitter/LSP/evidence lanes)
  - current degradation reasons
  - truth-grade distribution

Why this is strong:
- converts "implicit limitations" into explicit machine-readable contract
- prevents false confidence in cross-language migration paths

Proposed new decisions:
- `BR05-D5`: every cross-language query must cite capability tier coverage used
- `BR03-D6`: when capability is partial, response must include fallback rationale

### Option 6: Setup Command for MCP Client Registration (Recommended)
Decision log targets:
- BR04

Additions to pick:
- `parseltongue setup` command:
  - detect installed MCP clients
  - write deterministic config entry
  - dry-run mode to show diffs before write

Why this is strong:
- turns BR04 operator value into a one-command onboarding win
- reduces manual config drift and support burden

Proposed new decisions:
- `BR04-D7`: MCP registration path is automated with audit log and dry-run

## Portfolio choices (pick one path)

### Path A: Trust and Operator Moat (default)
Pick now:
- Option 1, 2, 3, 6
Defer:
- Option 4, 5

Best when:
- goal is fastest V200 ship with strongest reliability narrative

### Path B: Semantic Discovery Expansion
Pick now:
- Option 1, 2, 4, 5
Defer:
- Option 3, 6

Best when:
- goal is stronger fuzzy/discovery UX for LLM workflows

### Path C: Platform Handshake First
Pick now:
- Option 1, 3, 6
Defer:
- Option 2, 4, 5

Best when:
- goal is adoption and onboarding before deeper query semantics

## What ES-V200 should explicitly avoid copying

1. Avoid "single search tool" compression; Parseltongue's moat is graph-trust + analysis depth.
2. Avoid scoring-only truth; similarity score is ranking signal, not correctness signal.
3. Avoid blending docs/text chunks into canonical edges without BR06 promotion gates.
