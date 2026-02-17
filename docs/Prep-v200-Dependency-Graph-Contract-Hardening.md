# Prep-v200-Dependency-Graph-Contract-Hardening
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

## Pass order policy
Default order for V200:
1. `rust-llm-core-foundation`
2. `rust-llm-store-runtime`
3. `rust-llm-rust-semantics`
4. `rust-llm-tree-extractor`
5. `rust-llm-cross-boundaries`
6. `rust-llm-graph-reasoning`
7. `rust-llm-context-packer`
8. `rust-llm-interface-gateway`
9. `rust-llm-test-harness`

## Evidence rule
- Never reduce Risk/Unclear from narrative confidence.
- Reduce only when probe artifacts exist.
- If new coupling is discovered, update graph first, then scores.

## Per-pass output contract
Every pass must produce:
- Contract snapshot for one crate
- Failure mode table
- Probe definitions
- Probe outcomes
- Updated Risk/Unclear deltas
- Graph/interface deltas (if any)

## Canonical location for pass execution
All pass-by-pass execution artifacts are tracked in:
- `docs/Prep-v200-Hashing-Risks-v01.md`

This method doc is stable and reusable. The hashing-risks doc is the evolving ledger.

## Enrichment protocol
As we move forward, we enrich this method doc only when process changes (not every pass):
- add better scoring heuristics
- add better probe taxonomy
- add better dependency-walk rules
- add exit criteria refinements

Pass-level findings remain in the evolving crate ledger doc.
