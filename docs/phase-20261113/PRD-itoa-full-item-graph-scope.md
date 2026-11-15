# PRD — full item dependency graph of itoa

## The answer

**This phase builds one thing: the full item dependency graph of pinned itoa 1.0.18, produced
deterministically by Rust code from the real source, with no LLM in production. Every item
outside a function body is a node, private ones included, and every dependency proven from
source is an edge with evidence. The phase is done when that graph equals the frozen expected
itoa graph.**

Out of scope for this phase: every other repository, any rust-analyzer export or oracle
comparison, and every higher-abstraction table (interface level, HLD, file and folder
roll-ups). Those are later queries over this graph, not part of it.

Status on 2026-11-14: **in progress, not passed.**

```text
itoa run today, sysroot excluded, marked incomplete by the code
nodes   45    43 items + 2 file areas                       draft expected: 73
edges  104    43 contains + 61 calls                        draft expected: 368
              2 calls resolved, 59 stored unresolved with a reason
kinds    2 of the 6 that itoa needs: contains, calls
proven   <Buffer as Default>::default --calls--> Buffer::new, from source through frozen readback
next     Buffer::format --calls--> private::Sealed::write
```

Proof procedure: [SOP](SOP-itoa-full-item-graph-acceptance.md). Requirements:
[SPEC](SPEC-itoa-full-item-graph/README.md). Design:
[ARCH](ARCH-itoa-full-item-graph-pipeline.md). Expected graph:
[itoa-target-dataset](itoa-target-dataset/). History: [log](LOG-itoa-full-item-graph-progress.md).

## 1 — What is built

One graph in two tables: `item-effective-visibility-records` (one row per node) and
`edge-evidence-record-table` (one row per edge fact). Each row links back to the source range
that proves it.

Graph rules:

1. Nodes: every module, function, struct, enum, trait, type alias, const, static and associated
   item, private items included. Nothing inside a function body. Impl blocks are not nodes.
2. One file-area node per source file, separate from its module node. It contains the file's
   top-level nodes.
3. Each `use` outside a function body is a node, with one `imports` edge per resolved binding.
4. An inline module is an ordinary module node with its own scope.
5. Every extracted edge is stored uncollapsed, private to private included. The edge type has
   eight kinds: `calls`, `uses_type`, `implements`, `reexports`, `contains`, `derives`,
   `imports`, `reads`. itoa needs six of them; it has no `derives` or `reexports`.
6. A call, type use or read targets the real item. The `use` line is evidence, never the target.
7. The graph is the two tables above. No table is added.

What is never guessed:

- Unknown visibility and unknown reachability are stored as unknown.
- A site that cannot be resolved is stored unresolved with one typed reason.
- A generic or dynamic call targets the trait method, not a guessed implementation.

## 2 — How it is built

Ownership:

- **Rust** parses, expands macros, lowers signatures, infers within bodies, resolves calls and
  creates canonical identities.
- **DuckDB** does relational work on a copy of one published generation: name-resolution
  rounds, item and container projection, joins and deduplication.
- **Turso** holds durable rows, generations, frozen snapshots and atomic publication.
- No third database. No memory crosses a stage boundary; every stage reads and writes declared
  tables.

Stages:

```text
stage                                writes                                    state today
0   crate graph                      crate-dependency-graph-table              built
1   source ingest                    file-source-text-table                    built
2/3 syntax walker                    span, signature-detail, body-site rows    built
4+5 name resolution + item macros    bindings, path-resolution sites,          built
                                     macro-expanded items
6   signature-type-lowering-stage    lowered signatures, impl-header-index     impl headers only
7   semantic-edge-evidence-stage     edge-evidence-record-table                contains and calls only
-   item-effective-visibility-stage  item-effective-visibility-records         visibility stored unknown
```

The run for this phase is
`parseltongue map-workspace-full-run --mapping-final-stage-kind full-item-dependency-graph`.

Process boundary: production may start only `cargo metadata --offline`, `rustc --print cfg`
and `rustc --print sysroot`. It never compiles, never runs build scripts or proc macros, never
reads `target/`, and never uses the network.

## 3 — How it is proven

Only the SOP decides PASS. Unit tests are progress.

```text
repository  dtolnay/itoa 1.0.18, commit af77385d0daf4d0e949e81f2588be2e44f69f086
target      lib, edition 2021, x86_64-unknown-linux-gnu, no features, no build script
size        src/lib.rs 466 lines, src/u128_ext.rs 22 lines
```

Acceptance:

- **A1** The production binary runs on the pinned source with no pre-resolved input.
- **A2** Every expected node and edge is present with full identity and evidence range.
- **A3** No unexplained extra, missing or wrong node, edge or evidence row.
- **A4** Unresolved and inactive sites match the reviewed list. A reason does not excuse a
  resolvable missing edge.
- **A5** Source, configuration and executable digests, time, peak memory and disk are recorded.
- **A6** Frozen readback preserves evidence and generation identity; a cancelled or failed run
  leaves the previous output serving; output is identical at 1, 4 and 16 workers.

The expected graph is written from the pinned source, reviewed row by row and frozen before
comparison. Today's dataset is an unreviewed draft. It is comparison data only, never
production input.

Build order, one real-source test per relationship:

```text
done  <Buffer as Default>::default --calls--> Buffer::new
next  Buffer::format --calls--> private::Sealed::write          one generic edge, not twelve
      <Buffer as Clone>::clone --calls--> Buffer::new
      Integer --uses_type--> private::Sealed
      <u128 as Sealed>::write --> <u128 as Unsigned>::fmt --> div_rem_1e16 --> u128_ext::mulhi
      Buffer::new --reads--> <i128 as Integer>::MAX_STR_LEN
      Buffer::format --calls--> core::hint::unreachable_unchecked   the use node is evidence
      use nodes of lib.rs --imports--> core::hint, core::mem, core::mem::MaybeUninit, core::str
      file-area nodes --contains--> their top-level nodes
last  whole library equal to the frozen expected graph (A1–A6)
```

Method: expected rows live only in the test; fix the earliest missing producer; add only
general behaviour, nothing keyed on itoa names.

The next target must prove `i: I -> Integer -> private::Sealed -> write` and emit exactly one
generic target, with distractor, shadowing, ambiguity and rename checks.

## Reference

### Pins

- itoa: [phase-one-benchmark-pin.json](../phase-20261111/phase-one-benchmark-pin.json).
- Turso `=0.9.0-pre.5`, concurrent writes. A busy publication is tried up to 6 times, waiting
  10, 20, 40, 80, then 160 ms.
- DuckDB `=1.20000.1`, bundled, no extensions, no external access, run-local temp directory.

### Decisions in force

- **Calls** are resolved in Rust, reading persisted namespace and impl rows.
- **Nodes** are projected in DuckDB and written under `item-effective-visibility-stage`.
- **Inference:** build only what the current target needs; store the rest unresolved.
- **Expected graph:** written from the pinned source and reviewed. No oracle.
- **Containment:** each item has one lexical source container. File-level items and impl
  members belong to the file area; nested items belong to their inline module or trait. Impl
  identity, self type and trait are metadata, never a second parent.
- **Missing source (2026-11-14).** Codebases are expected to be partial or broken, so an edge
  is one of three things: *proven* (every candidate was visible in source); *proven, gap* (one
  target found, and the edge names the source that was missing, so it is correct if the code
  compiles); *unresolved* (zero or several targets, with one reason). A PASS run needs zero
  gap edges, or an explicit accepted list.
- **Memory:** measured and reported on every run; no cap in this phase.

- **Targets in `core` (2026-11-15).** Stored with a named external target: the name as far as
  itoa's source proves it, the crate, and a flag that the target's source was not read. The
  sysroot source stays out of the run. Chosen because it is the most explicit form for an
  agent reading the graph.
- **Macro definitions and calls (2026-11-15).** Both are nodes, for the same reason.
- **Design rule (user, 2026-11-15).** "Our intention should be to provide maximum
  information which helps the coding agent code better." When a graph design choice is
  open, the form that tells a coding agent the most is chosen and recorded here as decided.
- **One edge row per source site (2026-11-15).** Repeated sites with the same source,
  relation and target stay separate rows, each with its own range. Rows are grouped only
  when they are compared to the expected graph.
- **Macro-generated items (2026-11-15).**
  - Identity: the macro call plus the item's place in that call's expanded text. Two calls
    of one macro give two nodes.
  - Container: the file area or module where the call is written (one `contains` row), as
    for original impl members.
  - Generating call: named twice, as a field on the node and as a `generates` edge from
    the call node to the item (itoa: 45 rows). `generates` is a new relation kind.
  - A macro call points to the definition it expands by an `expands` edge (itoa: 15 rows).
  - None of this is built yet; it is part 5.

### Open

- **Targets in `core`, not built.** 119 of the 368 draft edges point into `core`. Resolving them needs
  the sysroot source in the run; measured on 2026-11-13, that run did not finish name
  resolution in 2,400 s (the run report is deleted). Runs today use
  `--sysroot-source-scope-kind excluded` and store those edges unresolved; the named external
  target is not built yet.
- **Macro-generated items, not built.** 45 of the 73 draft nodes come from `impl_Integer`,
  `impl_Integer_size` and `impl_Unsigned` calls. The rules are decided (above); the store
  already holds the 45 items as spans (measured 2026-11-15).
- **Enum variants and struct fields:** nodes, or evidence only.
- **File module and file area:** how a file's module node relates to its file-area node.
- **Non-function sources:** may `reads` and `uses_type` start at a non-function item? itoa
  needs it for `const MAX_STR_LEN` and the array length in `Buffer`. Lean: yes.
- **Edge reasons.** The code has two unresolved-reason lists: `SiteUnresolvedReasonKind` for
  sites and `HldUnresolvedReasonKind` for graph edges. One must win before unresolved edges
  are compared (A4).

The enum-variant, file-module and non-function-source points block the affected expected
rows from being frozen.

### Out of scope

- Any repository other than itoa.
- Any rust-analyzer export, oracle harness or oracle comparison.
- Interface level, HLD tables, file and folder roll-ups, coverage banner.
- Recovering code that does not compile; any LLM or training.
- Running proc macros or build scripts, or reading their outputs.
- Test, bench and example targets; feature exploration.
- Memory or speed optimisation.

Code for some of these exists in the repository from earlier work (the HLD builder, the
`original-top-level-containment` stage, the oracle comparison command). This phase does not
use, extend or test against it.
