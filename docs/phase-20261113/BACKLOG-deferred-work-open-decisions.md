# BACKLOG — deferred work and open decisions

Updated 2026-11-17.

## The answer

**This file holds two lists. List A is the decisions of this phase: three are open and the
rest are made. The expected itoa graph was frozen on 2026-11-16 with the open ones left as
they are, so deciding one needs a new freeze. List B is work that waits until the itoa full
item graph passes. Nothing in list B is built, extended or tested in this phase.**

- Deferred means not decided, not built and not passed.
- A correctness blocker for the itoa full graph never goes into list B because it is hard.
- Anything higher than the full graph is a later query over the stored graph. It needs no
  second source-analysis pipeline, and it cannot recover a fact the graph is missing.

## A — Decisions of this phase

Open:

```text
id      decision                                                    state in the frozen graph
OD-4    Enum variants and struct fields: nodes, or evidence only.   not nodes; itoa has private
                                                                    fields only
fields  Visibility, reachability and signature text on nodes.       not filled, not compared
```

Decided by the user (2026-11-15), on the rule "choose what is most explicit for a coding agent
that reads the graph":

- **core:** an edge into `core` is stored with a named external target: the name as far as
  itoa's source proves it, the crate, and a flag that the target's source was not read. A site
  whose target name cannot be proven without `core` (a method on a value whose type comes from
  `core`) stays unresolved, with the method name and a reason. No sysroot run in this phase.
- **Macro nodes:** each `macro_rules!` definition and each macro call at item position is a
  node (itoa: 3 and 15). A macro-generated item records the call that generated it. A call
  points to the definition it expands by an `expands` row (decided and built 2026-11-16,
  part 5c; itoa: 15 rows).

- **Module and file area (OD-7, decided 2026-11-17 by Rule 6):** a module `contains` the
  file area of its file. Built in SPEC part 12 (REQ-RUST-070.0); itoa gains 2 rows.

- **Macro-generated items (OD-2, decided 2026-11-15):** identity is the macro call plus the
  item's place in its expansion. The file area or module where the call is written contains
  the item. The generating call is named twice: a field on the node, and a `generates` edge
  from the call to the item (itoa: 45 rows). Part 5a, REQ-RUST-046.0 R2, R3, R5.

- **Method calls into `core` (decided 2026-11-16):** the run reads the declarations of
  `core` as a frozen input; the label is the type as `core` writes it (`[T]::len`); a call
  a prelude trait could also answer is named with that assumption stored on the row. A
  trait method is named by its trait (`TryInto::try_into`). A receiver that is a method
  call takes the return type `core` declares, and a type name in it is read only when the
  file's scope proves which declaration it is (2026-11-17). The frozen graph still has
  those 133 calls without a target; their expected targets are frozen in a separate file,
  and the re-freeze of the graph is part 8.
- **OD-8 (decided 2026-11-16, built in part 5d steps 10a and 10c):** `reads` and
  `uses_type` start at any item a path is written in, not only a function.
- **Disagreement with the expected graph:** show the source lines, the user rules, and the
  wrong side is fixed.

Also open, in code rather than in the graph rules:

- **Two unresolved-reason lists.** `SiteUnresolvedReasonKind` for sites and
  `HldUnresolvedReasonKind` for graph edges both still exist.
- **Rows depend on the source directory** and **`test_lang_items_in_sysroot` fails**: both
  are described in the PRD ("Open") and the SOP.

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
BL-09   A node field naming the impl header of an impl     today only the label names it
        member (`node_impl_header_reference`)               (part 5, section 8, Q8)
BL-10   The proven receiver type on a method call row       `string.len()` at lib.rs 109: source
        without a target (a local bound once to a call      proves `&str`, the row does not say
        whose stored target declares its return type)       so (part 7, Q4)
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
