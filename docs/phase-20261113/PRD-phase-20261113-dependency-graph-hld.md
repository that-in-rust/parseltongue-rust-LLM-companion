# PRD — full dependency graph of Rust code that compiles

## The answer

**Build one product: the `full-item-dependency-graph` of a compiling Rust workspace, produced
deterministically by Rust code from real source, with no LLM in production. Every item outside
a function body is a node, private ones included, and every dependency proven from source is an
edge with evidence. Every higher-level view (interface level, HLD, roll-ups) is a query over
that graph.**

An external coding agent reads the graph and its evidence to reason about control flow and data
flow before editing.

Status on 2026-11-14: **Phase 1 is in progress and not passed; Phase 2 is not started.**

```text
bounded itoa run, sysroot excluded, marked incomplete by the code
nodes   45    43 items + 2 file areas
edges  104    43 contains + 61 calls; 2 calls resolved, 59 stored unresolved with a reason
kinds    2 of 8 produced: contains, calls
proven   <Buffer as Default>::default --calls--> Buffer::new, from source through frozen readback
next     Buffer::format --calls--> private::Sealed::write
```

This document claims no complete graph, no whole-itoa agreement and no budget met.

Contracts: [SPEC](SPEC-phase-20261113-executable-requirements.md). Design:
[ARCH](ARCH-phase-20261113-pipeline-graph-tables.md). Proof procedure:
[SOP](SOP-phase-20261113-full-item-dependency-graph.md). History: [log](phase1-log-p1.md).
Deferred work: [backlog](phase1-abstractions-decision-backlog.md).

## 1 — What the product is

One logical graph stored in two tables: `item-effective-visibility-records` (one row per node)
and `edge-evidence-record-table` (one row per edge fact). Rows link to retained source, span,
signature, body and resolution evidence.

Graph rules (D6):

1. The full graph is built first. A level adds no fact; it only filters, collapses and counts.
2. Nodes: every module, function, struct, enum, trait, type alias, const, static and associated
   item, private items included. Nothing inside a function body. Impl blocks are not nodes.
3. One file-area node per source file, separate from its module node. It contains the file's
   top-level nodes.
4. Each `use` outside a function body is a node, with one `imports` edge per resolved binding.
5. An inline module is an ordinary module node with its own scope.
6. Every extracted edge is stored, private to private included. Eight kinds: `calls`,
   `uses_type`, `implements`, `reexports`, `contains`, `derives`, `imports`, `reads`.
7. A call, type use or read targets the real item. The `use` line is evidence, never the target.
8. The full graph is the two tables above. No table is added.
9. The interface level keeps the crate root, non-private items, file-area nodes and `use` nodes.
   Private items remain as steps in evidence paths.
10. Collapse walks only `calls`, `uses_type`, `implements`, `reexports`, `derives`, `reads`. A
    `contains` or `imports` edge is never a step of a collapsed path.

What is never guessed:

- Unknown visibility and unknown reachability are stored as unknown.
- A site that cannot be resolved is stored unresolved with one typed reason.
- A generic or dynamic call targets the trait method, not a guessed implementation.

Rules 9 and 10 and the HLD tables are deferred queries; see the backlog. They are not Phase 1
exit requirements.

## 2 — How it is built

Ownership:

- **Rust** parses, expands macros, lowers signatures, infers within bodies, resolves calls and
  creates canonical identities.
- **DuckDB** does relational work on a copy of one published generation: name-resolution
  rounds, item and container projection, joins, deduplication, and later the derived levels.
- **Turso** holds durable rows, generations, frozen snapshots and atomic publication.
- No third database. No memory crosses a stage boundary; every stage reads and writes declared
  tables.

Stages:

```text
stage                                writes                                    state today
0   crate graph                      crate-dependency-graph-table              built
1   source ingest                    file-source-text-table                    built
2/3 syntax walker                    span, signature-detail, body-site rows    built
4+5 name resolution + item macros    bindings, path-resolution sites,          built, not qualified
                                     macro-expanded items
6   signature-type-lowering-stage    lowered signatures, impl-header-index     impl headers only
7   semantic-edge-evidence-stage     edge-evidence-record-table                bounded: contains, calls
-   item-effective-visibility-stage  item-effective-visibility-records         bounded: visibility unknown
9   hld-current-materialization      five HLD tables                           fixtures only; deferred
10  coverage banner                  per-level coverage                        partial
11  oracle edge differ (offline)     oracle-edge-agreement-table               path sites only
```

The CLI final stage `full-item-dependency-graph` runs stages 0–5, then writes the bounded
stage 6, 7 and visibility rows and freezes them.

Process boundary (D1, D4, D5): production may start only `cargo metadata --offline`,
`rustc --print cfg` and `rustc --print sysroot`. It never compiles, never runs build scripts or
proc macros, never reads `target/`, and never uses the network.

Source of algorithms: rust-analyzer logic is ported over persisted rows. Its analysis database
(`hir`, `ide`, `salsa`) is never linked into production; only the offline oracle harness uses it.

## 3 — How it is proven

Only the SOP decides PASS. Unit tests are progress.

### Phase 1 — pinned itoa, exact comparison

```text
repository  dtolnay/itoa 1.0.18, commit af77385d0daf4d0e949e81f2588be2e44f69f086
target      lib, edition 2021, x86_64-unknown-linux-gnu, no features, no build script
size        src/lib.rs 466 lines, src/u128_ext.rs 22 lines
```

Acceptance:

- **A1** The production binary runs on the pinned source with no pre-resolved input.
- **A2** Every expected node and edge is present with full identity and evidence range.
- **A3** No unexplained extra, missing or wrong node, edge or evidence row.
- **A4** Unresolved and inactive sites match the reviewed inventory. A reason does not excuse a
  resolvable missing edge, and a sysroot-excluded run is not complete coverage.
- **A5** Source, configuration and executable digests, time, peak memory and disk are recorded.
- **A6** Frozen readback preserves evidence and generation identity; a cancelled or failed run
  leaves the previous output serving; output is identical at 1, 4 and 16 workers.

The expected graph must be reviewed and frozen before comparison. Today's
[itoa-target-dataset](itoa-target-dataset/) is an unreviewed LLM-parsed draft and is never
production input.

Relationships the reference must contain:

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
last  whole library equal to the frozen reference (A1–A6)
```

Method: one real-source test per relationship; expected rows live only in the test; fix the
earliest missing producer; add only general behaviour, nothing keyed on itoa names.

The next target (REQ-RUST-042.0) must prove `i: I -> Integer -> private::Sealed -> write` and
emit exactly one generic target, with distractor, shadowing, ambiguity and rename checks.

### Phase 2 — five repositories

Order: serde, rust-analyzer, ripgrep, tokio, cargo-modules. All five qualify equally (D2).

```text
M0  skeleton, core ids and errors, oracle harness build
M1  crate graph, ingest, parser, Turso runtime, generations
M2  name resolution + macros, built-ins, lang items
M3  proc-macro shapes checked against real oracle expansions (D3)
M4  signature lowering, impl index
M5  body inference: locals, inherent methods, autoderef
M6  trait methods in scope; ≥ 95% coverage on all five allows skipping M7
M7  optional trait-solver adapter
M8  derived levels, roll-ups, banner; full S1–S10 on all five with one executable
```

Per-repository gates, scored against the PRIMARY oracle export:

```text
path resolution                        ≥ 99.5%
macro expansion items and body sites   ≥ 99%
proc-macro shape expansions            = 100%
impl relations                         ≥ 99.5%
method targets, precision              ≥ 99%
method targets, coverage               ≥ 95%   mandatory at M8
selection, visibility, reachability    = 100%
```

Budgets:

- Memory: measured on every run with no interim cap. After all five repositories have working
  end-to-end results, whole-process-tree peak ≤ 500,000,000 bytes.
- Time target: body inference under 30 minutes on 4 cores for rust-analyzer. Not measured yet.
- Disk and per-stage time are reported, with no threshold.

## Reference

### Scope and context

- Map lib and bin targets of the workspace and their normal dependency closure.
- Exclude tests, benches, examples, dev-dependencies and build-script targets. `cfg(test)` is off.
- The semantic source set is the files reachable from mapped module trees under active `cfg`,
  plus includes and macro expansions. Other `.rs` files are stored byte-exact only.
- Items under inactive `cfg` are counted as unresolved `cfg_inactive`, never dropped.
- A compilation unit is Cargo package id + target name and kind + host/target context. Features,
  `cfg` and edition are per unit. Equal names or bytes never merge two units.
- Unresolved reasons, one per site, in this order: `cfg_inactive`, `build_script`, `proc_macro`,
  `external_source_missing`, `unknown_receiver`, `trait_solver_needed`
  (`SiteUnresolvedReasonKind`).
- KL1: sites that depend on build-script output are a known limitation, identified only by
  paired oracle evidence. It never excuses an oracle-proven error.

### Pins

- Turso `=0.9.0-pre.5`, concurrent writes. A busy publication is tried up to 6 times, waiting
  10, 20, 40, 80, then 160 ms.
- DuckDB `=1.20000.1`, bundled, no extensions, no external access, run-local temp directory.
- Oracle: rust-analyzer `2a362a41a8eb6e090f1a12d9b712df85af6f0c33` plus the local parser patch.
- Repositories, commits, reuse mode and licenses: SPEC section 10. Copied code keeps its
  headers. cargo-modules (MPL-2.0) is idea-only.

### Decisions in force

- **D1–D6, KL1:** as stated above.
- **AD-1, sysroot:** interim option B. Measured on 2026-11-13: itoa with the sysroot (619 units
  parsed) did not finish name resolution within 2,400 s (exit 124, nothing published). That run
  report has since been deleted; the figures are from the PRD and journal of that day.
  Development runs therefore use `--sysroot-source-scope-kind excluded` (REQ-RUST-038.0), and
  edges into `core` are stored unresolved. Phase 1 cannot pass this way; the sysroot path must
  be made to work.
- **AD-2, body paths:** calls are resolved in Rust, reading persisted namespace and impl rows.
- **AD-3, nodes:** projected in DuckDB and written under `item-effective-visibility-stage`.
- **AD-4, inference:** build only what the current target needs; store the rest unresolved.
- **AD-5, reference:** an expected graph written from the pinned source and reviewed, with the
  oracle as a cross-check. It is comparison data only.
- **OD-7, containment:** each item has one lexical source container. File-level items and impl
  members belong to the file area; nested items belong to their inline module or trait. Impl
  identity, self type and trait are metadata, never a second parent.

### Open

- **OD-2** Identity and container of macro-generated items.
- **OD-4** Enum variants and struct fields: nodes, or evidence only.
- **OD-7** (rest) How a file's module node relates to its file-area node.
- **OD-8** May `reads` and `uses_type` start at a non-function item? itoa needs it for
  `const MAX_STR_LEN` and the array length in `Buffer`. Lean: yes.

- **Edge reasons** Graph edge rows use a second reason type in code, `HldUnresolvedReasonKind`:
  missing target, ambiguous target, cfg inactive, external source missing, trait solver needed,
  other. It has no `build_script`, `proc_macro` or `unknown_receiver`, and adds three values
  the list above lacks. One list must win before unresolved edges are compared in Phase 1 (A4).

OD-4, OD-7 and OD-8 block the affected reference rows from being frozen.

### Out of scope

- Recovering code that does not compile; any LLM or training.
- Running proc macros or build scripts, or reading their outputs.
- Test, bench and example targets; all-feature exploration.
- Intent tables, incremental remap, CLI query verbs.
- In Phase 1: the five repositories, the derived levels, and memory or speed optimisation.
