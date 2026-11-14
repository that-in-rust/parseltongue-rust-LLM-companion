# BACKLOG — deferred work and open decisions

Updated 2026-11-14.

## The answer

**This file holds two lists. List A is five decisions that must be made in this phase, because
the expected itoa graph cannot be frozen without them. List B is work that waits until the
itoa full item graph passes. Nothing in list B is built, extended or tested in this phase.**

- Deferred means not decided, not built and not passed.
- A correctness blocker for the itoa full graph never goes into list B because it is hard.
- Anything higher than the full graph is a later query over the stored graph. It needs no
  second source-analysis pipeline, and it cannot recover a fact the graph is missing.

## A — Decisions that block this phase

Counts are rows of the draft expected graph that stay blocked until the decision is made
([itoa-target-dataset](itoa-target-dataset/)). One row can wait on several decisions.

```text
id     decision                                                   blocks
OD-2   Macro-generated items: their identity, and which file      279 edges, 45 nodes
       and container own them.
core   Targets in core: run with the sysroot source, or accept    119 edges
       these edges as unresolved. The sysroot run did not
       finish name resolution in 2,400 s (measured 2026-11-13).
OD-8   May reads and uses_type start at a non-function item?      56 edges
       itoa needs it for const MAX_STR_LEN and the array length
       in Buffer. Lean: yes.
OD-7   How a file's module node relates to its file-area node.    2 edges
OD-4   Enum variants and struct fields: nodes, or evidence only.  not counted; itoa has
                                                                  private fields only
```

Also open, in code rather than in the graph rules:

- **Two unresolved-reason lists.** `SiteUnresolvedReasonKind` for sites and
  `HldUnresolvedReasonKind` for graph edges. One must win before unresolved edges are compared.
- **No comparator for full-graph rows.** SOP S3 has nothing to run yet.

Already decided, so not open: one lexical source container per item; impl blocks are records;
items inside function bodies are evidence; calls are resolved in Rust; nodes are projected in
DuckDB; the expected graph is written from source with no oracle
([PRD](PRD-itoa-full-item-graph-scope.md)).

## B — Deferred until the itoa full graph passes

```text
id      deferred work                                       open question it carries
BL-01   Interface level: which nodes are selected           OD-6: private items used by
                                                            several files
BL-02   Collapsed edges through private items               OD-9: all six kinds, or calls
                                                            and reads only
BL-03   Multi-hop evidence paths
BL-04   File and folder roll-ups                            which bucket a generated item
                                                            falls in
BL-06   The five HLD outputs: build, publish, compare
BL-07   Convenience queries, visualisation, intent maps     OD-3: raw tables or a display
                                                            copy
BL-08   original-top-level-containment as a product mode
BD-05   pub use: imports and reexports together             OD-5; itoa has no pub use
BF-01   Any repository other than itoa
BF-02   Proc-macro, async and dynamic-dispatch shapes that
        itoa does not contain
BF-03   Performance tuning, incremental remapping, new CLI
        verbs
BF-06   Any rust-analyzer export or oracle comparison
```

Ids are kept from the earlier backlog so old references still resolve. BL-05, BD-01 to BD-04,
BD-06, BF-04 and BF-05 were folded into the rows above or closed by the scope decision.

## Code from earlier work that this phase does not use

It stays in the repository, untouched. Deleting it is a separate decision.

- The HLD builder and its DuckDB and Turso stores.
- The `original-top-level-containment` final stage, which publishes HLD tables.
- The `oracle-edge-differ-run` verb and its comparison code in the DuckDB adapter.
- The `status-coverage-banner-show` verb.

## How to use this file

- New deferred work gets a stable id, a reason and the condition that brings it back.
- Progress goes in the [log](LOG-itoa-full-item-graph-progress.md), not here.
