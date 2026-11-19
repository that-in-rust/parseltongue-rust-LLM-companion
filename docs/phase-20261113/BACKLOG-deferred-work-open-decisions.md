# BACKLOG — deferred work and open decisions

Updated 2026-11-19.

## The answer

**The itoa full item graph passed its acceptance run on 2026-11-18, and nothing is in
progress. This file holds three lists. List A is the decisions of the phase, all made.
List B is work that was deferred until the graph passed. List C is what the accepted run
left open: 12 items, moved here from the PRD on 2026-11-19 (user decision: "whatever
needs to be added, put it in the backlog"). Nothing in B or C is built, and none of it is
picked.**

- Suggested order for an MVP, a judgement and not a measurement: BF-01 (a second, mid-size
  crate) first, then BC-01, BC-11 and BC-06. The rest waits until a crate needs it.

- Deferred means not decided, not built and not passed.
- A correctness blocker for the itoa full graph never goes into list B because it is hard.
- Anything higher than the full graph is a later query over the stored graph. It needs no
  second source-analysis pipeline, and it cannot recover a fact the graph is missing.

## A — Decisions of this phase

None is open. The last two made:

```text
id      decision                                                    state in the frozen graph
OD-4    Enum variants and struct fields: nodes, or evidence only.   decided 2026-11-18 by Rule 6: nodes
                                                                    (SPEC part 13, built). Uses of
                                                                    a field as edges: BC-06
fields  Visibility, reachability and signature text on nodes.       built 2026-11-17 (SPEC part 11), compared
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
  file's scope proves which declaration it is (2026-11-17). All 133 calls have a target
  in the graph frozen on 2026-11-18 (SPEC parts 8 and 15).
- **OD-8 (decided 2026-11-16, built in part 5d steps 10a and 10c):** `reads` and
  `uses_type` start at any item a path is written in, not only a function.
- **Disagreement with the expected graph:** show the source lines, the user rules, and the
  wrong side is fixed.

Already decided, so not open: one lexical source container per item; impl blocks are records;
items inside function bodies are evidence; calls are resolved in Rust; nodes are projected in
DuckDB; the expected graph is written from source with no oracle
([PRD](PRD-itoa-full-item-graph-scope.md)).

## B — Deferred until the itoa full graph passed (it has; none is picked)

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

## C — Left open by the accepted run of 2026-11-18

Each row is a fact about the code or the proof at commit `9b31b6b30`. None blocked the
pass. "Back when" is the condition that brings the item back.

```text
id      what is open                                         back when
BC-01   The completeness mark. The run writes `incomplete`   before anyone reads the mark; small.
        as a constant, although every edge row of the        Needs a rule for "complete" (no row
        accepted run has a target. It is not computed.       without a target? no gap edge?)
BC-02   The run id hashes the toolchain path, which is       two machines must share or compare
        under the home directory. Node and edge rows hold    a stored run
        no such path (SPEC part 14).
BC-03   Reachability through a private supertrait. The       an agent asks "what can outside code
        methods of `private::Sealed` are stored as not       call"; needs a rule for bounds
        reachable; outside code calls them through
        `I: Integer` (SPEC part 11).
BC-04   Two unresolved-reason lists,                         a crate has unresolved edges to
        `SiteUnresolvedReasonKind` and                       compare (itoa has none now)
        `HldUnresolvedReasonKind`. One has to win (A4).
BC-05   Reading the whole sysroot (scope `included`) did     not planned: scope `declarations`
        not finish name resolution in 2,400 s on             replaced it. Close unless a crate
        2026-11-13. No itoa run uses it.                     needs items of `core` as nodes
BC-06   Uses of a field as edges (`self.bytes`,              "who uses this field" on a real
        `DECIMAL_PAIRS.0`). Fields are nodes; the run        crate. Needs a site row for a field
        stores no site for a plain field access.             access, a spec and a new freeze
BC-07   A field or variant written by a macro expansion      BF-01: the first crate that has one
        has no real-source test. The code path has the
        change; itoa has no such field (SPEC part 13).
BC-08   `test_lang_items_in_sysroot` fails (native sysroot   before the workspace gate can be
        timeout) and is skipped, so the workspace test       called PASS
        gate is not a full PASS (AGENTS.md, Rule 3).
BC-09   The expected graph was written and checked by one    before the graph is used as proof to
        author. Rule 5 (second review) was retired on        someone outside; a second reader, or
        2026-11-16.                                          a compiler-based cross-check
BC-10   A6, failure safety: "a failed or cancelled run       before the store serves readers
        keeps the previous output" is tested in the Turso    during a run
        adapter on fixtures, not by cancelling a real
        itoa run. The SOP has no such step.
BC-11   One command from source to the DuckDB file. Today    the first user who is not us
        it is three: the run, two exports, the loader
        (AGENTS.md, Commands).
BC-12   `lowered-signature-type-table` is registered under   with BC-03 or any work on lowered
        stage 6 and nothing writes it (PRD, section 2).      signatures; else delete the table
```

- BC-01 to BC-05 were the PRD's "Open" list. BC-06 and BC-07 were limits stated in SPEC
  part 13. BC-08 to BC-10 were limits stated in the run report. BC-11 and BC-12 are new.
- The largest gap for an MVP is BF-01 in list B: only itoa, 488 lines, has been run.

## Code from earlier work that this phase does not use

It stays in the repository, untouched. Deleting it is a separate decision.

- The HLD builder and its DuckDB and Turso stores.
- The `original-top-level-containment` final stage, which publishes HLD tables.
- The `oracle-edge-differ-run` verb and its comparison code in the DuckDB adapter.
- The `status-coverage-banner-show` verb.

## How to use this file

- New deferred work gets a stable id, a reason and the condition that brings it back.
- Progress goes in the [log](LOG-itoa-full-item-graph-progress.md), not here.
