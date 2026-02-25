# ES-V200-Dependency-Graph-Contract-Hardening
Status: Living method doc
Purpose: Define the reasoning method we will use on every V200 pass to reduce architecture risk through interface-level evidence.

## Why this method exists
We are not just writing architecture notes. We are converting uncertainty into executable confidence before implementation by hardening crate contracts in dependency order.

## Method name
Dependency Graph Contract Hardening

## Core idea
Use the dependency graph to decide pass order, then harden one crate interface at a time with falsifiable probes.

## Pass loop (repeat every pass)
1. Select one crate (highest dependency impact and/or highest Risk/Unclear).
2. Freeze public interface contract for that crate (input, output, task).
3. Rubber-duck dependency walk:
   - Who calls this interface?
   - What assumptions do callers make?
   - What breaks if this contract fails or changes?
4. Enumerate top failure modes (at least 3).
5. Define minimal probes that can falsify assumptions.
6. Record evidence.
7. Update risk scores only if evidence supports change.
8. Update graph/interfaces if coupling changed.
9. Log unresolved questions to the next pass queue.

## Design101 constraints (mandatory every pass)
1. Express every frozen interface as an executable specification:
   - preconditions
   - postconditions
   - error conditions
2. Define probe work in STUB -> RED -> GREEN -> REFACTOR order before changing scores.
3. Keep proposed function/crate/command names in four-word format.
4. Keep all dependency/interface diagrams in Mermaid.
5. Treat performance or concurrency claims as untrusted until test-backed artifacts exist.

## Lifecycle + companion requirement class (pre-PRD promotion rules)
When a pre-PRD idea is promoted to a V200 requirement, encode it as a crate-linked contract with falsifiable probes before score changes.

Required contract classes for the current promotion bundle:
1. Routing/lifecycle contracts (`#7`, `#8`, `#10`, `#27`, `#28`):
   - route namespace correctness
   - auto-port/port-file discovery lifecycle
   - graceful shutdown command and endpoint behavior
   - slug-aware URL and slug-aware discovery file behavior
2. Response-shape contract (`#25`):
   - deterministic semantic grouping in response schemas.
3. Ingest metric contract (`#29`):
   - token count computed and persisted at ingest with deterministic replay.
4. Extraction contract (`#35`):
   - data-flow edges (`assign`, `param`, `return`) extracted with fixture-backed correctness.

Companion app boundary rule:
- Tauri desktop companion is treated as an external consumer of gateway/runtime contracts.
- Do not add a new core crate or change dependency topology unless contract evidence proves the boundary is insufficient.

## Pass order policy
Default order for V200:
1. `rust-llm-core-foundation`
2. `rust-llm-store-runtime`
3. `rust-llm-rust-semantics`
4. `rust-llm-tree-extractor`
5. `rust-llm-cross-boundaries`
6. `rust-llm-graph-reasoning`
7. `rust-llm-interface-gateway`
8. `rust-llm-test-harness`

Deferred:
- `rust-llm-context-packer` moved to `docs/v210-backlog.md` (out of V200 active pass scope).

## Evidence rule
- Never reduce Risk/Unclear from narrative confidence.
- Reduce only when probe artifacts exist.
- If new coupling is discovered, update graph first, then scores.
- Promoted requirements must include source-trace references to existing research docs in pass evidence notes.

## Per-pass output contract
Every pass must produce:
- Contract snapshot for one crate
- Executable contract clauses (preconditions, postconditions, error conditions)
- Failure mode table
- Probe definitions
- STUB -> RED -> GREEN -> REFACTOR execution intent
- Probe outcomes
- Updated Risk/Unclear deltas
- Graph/interface deltas (if any)

## Canonical location for pass execution
All pass-by-pass execution artifacts are tracked in:
- `docs/ES-V200-Hashing-Risks-v01.md`

This method doc is stable and reusable. The hashing-risks doc is the evolving ledger.

## Enrichment protocol
As we move forward, we enrich this method doc only when process changes (not every pass):
- add better scoring heuristics
- add better probe taxonomy
- add better dependency-walk rules
- add exit criteria refinements

Pass-level findings remain in the evolving crate ledger doc.
