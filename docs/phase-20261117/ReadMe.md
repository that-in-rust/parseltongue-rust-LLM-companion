# Backend release phase — Linux and macOS

## The answer

**This phase proposes how to package and release our Rust CLI with a pinned
`codebase-memory-mcp` executable for non-Rust indexing and navigation on Linux and
macOS. It does not replace the Rust pipeline.**

**State: proposal only.** These documents implement no backend integration, dependency
lock, installer, release CI or acceptance procedure. No platform or API/schema combination
has been qualified, no combined package has been tested, and no release has been published
by this phase. A source-declared upstream asset is not proof that our combined product works.

## Scope and boundary

- Plan target-to-asset selection, source/version/checksum and license pins, native Rust
  builds, backend download verification, combined packaging, qualification and rollback.
- Keep the executables independently versioned. No binary or index database goes into Git.
- Do not change the [phase-20261113 producer or frozen dataset](../phase-20261113/PRD-itoa-full-item-graph-scope.md),
  its evidence rules, graph tables, expected rows or acceptance state.
- Do not change the [phase-20261116 consumer CLI](../phase-20261116/ReadMe.md). Its
  read-only contract over existing published rows does not gain an extraction pipeline,
  MCP process adapter or hidden enrichment through this packaging proposal.
- Any future process adapter, imported overlay or cross-language relationship needs its own
  authorized scope, source-backed contract and tests. Packaging a backend does not prove a
  relationship or permit writing external facts into the frozen Rust tables.

## Replicate phase-20261113's working style

Start with [AGENTS.md](../../AGENTS.md), then read the previous phase for method, not to
copy its implementation scope:

1. [PRD](../phase-20261113/PRD-itoa-full-item-graph-scope.md): fix scope and exclusions first.
2. [SPEC index](../phase-20261113/SPEC-itoa-full-item-graph/README.md): one selected target
   per executable part; stable requirement ids and expected results before code.
3. [Architecture](../phase-20261113/ARCH-itoa-full-item-graph-pipeline.md): preserve the
   existing ownership and publication boundaries.
4. [Acceptance SOP](../phase-20261113/SOP-itoa-full-item-graph-acceptance.md): freeze inputs
   and executable bytes; compare production results exactly and measure the run.
5. [Progress log](../phase-20261113/LOG-itoa-full-item-graph-progress.md): one current-state
   summary followed by append-only measured entries; built, tested and accepted differ.

The producer's SOP accepts the producer only. Its results and S1–S4 labels do not qualify
a backend package. Follow the [consumer phase's method](../phase-20261116/ReadMe.md) of
giving a new surface its own acceptance contract while leaving prior phases unchanged.

## Next steps, not delivered artifacts

Before implementation, create the phase's PRD, ARCH, SPEC index and selected part, frozen
qualification expectations, SOP, LOG and BACKLOG as the work is picked. They do not exist
merely because this entry document describes their responsibilities.

For each selected target, the proposed loop is:

1. Verify the exact upstream release source, asset identity, signed checksum policy,
   licenses and required API/schema behavior. Freeze source-backed expectations independently
   of the implementation: correct package contents and target mapping, supported behavior,
   empty results, unresolved cases and operational errors.
2. Write one executable SPEC part before code, using `rust-executable-specs-01` and its
   seven sections. Define the package/adapter boundary, deterministic outputs, resource
   expectations and unsupported states. Keep unselected parts as stubs.
3. Write a failing real test using the production CLI, the exact backend bytes and a
   pinned real non-Rust source repository. Add integrity/error cases such as a bad digest,
   wrong architecture, missing backend and incompatible schema; do not feed expected
   results into production.
4. Make the smallest general change, refactor, then test the actual packaged bytes on each
   claimed OS baseline and architecture. Verify storage, process lifecycle, source evidence,
   read-only Rust graph boundaries, deterministic results and upgrade/rollback behavior.
5. Record requirement/test ids, exact commands, nonzero selected-test counts, results,
   executable/source digests, target/OS, elapsed time, peak process-tree memory and commit
   in the phase LOG. Use PASS / FAIL / BLOCKED / NOT STARTED with measured evidence; report
   unrun gates and unknowns explicitly, never a bare completion percentage.

The immediate next target is release-specific API/schema qualification and frozen packaging
expectations, not production code or a claim of platform support.
