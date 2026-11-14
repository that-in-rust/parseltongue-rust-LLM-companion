# SPEC — requirements for the itoa full item graph

Updated 2026-11-14. States only what is in force. Scope: pinned itoa 1.0.18, the full item
dependency graph only.

## The answer

**Thirteen requirements define the itoa full item graph. Six are met from real source for the
bounded graph, three are partly met, four are not started. The next one to build is
REQ-RUST-042.0, the generic call `Buffer::format --calls--> private::Sealed::write`.**

```text
id      requirement                         state        real-source test on pinned itoa
033.0   real-source acceptance harness      MET          test_itoa_pipeline_reads_published
038.0   sysroot scope switch                MET          test_sysroot_scope_excluded_units
038.1   names that reach an absent sysroot  MET          test_sysroot_reached_names_reasoned
035.0   impl header rows                    MET          test_impl_header_row_contents
043.0   one source container per item       MET          test_original_member_graph_roundtrip
045.0   incomplete graph checkpoint         MET          test_original_member_graph_roundtrip
034.0   paths inside bodies                 PARTLY       test_default_new_call_roundtrip
036.0   Type::member lookup                 PARTLY       test_default_new_call_roundtrip
032.0   the full item graph                 PARTLY       2 of 6 edge kinds; original items only
042.0   generic call through a supertrait   NOT STARTED  none
018.2   signatures and bounds               NOT STARTED  none; impl headers only
046.0   macro-generated items as nodes      NOT STARTED  none
044.0   whole graph equals the expected     NOT STARTED  none
```

MET means the named test passes on pinned source (11 of 11 at commit `15643ec4f`). It does not
mean the SOP passed: only REQ-RUST-044.0 decides that.

Out of scope: every other repository, any rust-analyzer export or oracle comparison, and
every higher-abstraction table (interface level, HLD, file and folder roll-ups).

## Rules for every requirement

1. **One real-source test per requirement.** It runs the production CLI on pinned itoa, reads
   only published rows, and keeps expected rows in its own assertions. Component tests on
   fixtures support it; they never replace it.
2. **Resolve or give a reason.** A site becomes one edge to a proven target, or one unresolved
   row with one typed reason. Nothing is guessed, and a reason never excuses an edge that
   source can prove.
3. **General behaviour only.** No code is keyed on an itoa name, and no expected row enters
   production.
4. **Ids are stable.** Every test names its requirement in a doc comment
   (`/// REQ-RUST-NNN.R`). A changed requirement gets a new revision number.

## Requirements

### Met on pinned source

- **REQ-RUST-033.0 — harness.** A real-source test feeds the pinned tree through the production
  entry point and reads only the published generation. It fails, not passes empty, when a
  required stage did not run.
- **REQ-RUST-038.0 — sysroot switch.** With `--sysroot-source-scope-kind excluded`, no sysroot
  unit is loaded, the scope is recorded in the generation, and no sysroot identity is
  invented. The default is `included`. An excluded run cannot be an SOP PASS.
- **REQ-RUST-038.1 — absent sysroot names.** In an excluded run, a path that reaches `core`
  through an import, or a prelude name such as `Default`, is stored unresolved with
  `external_source_missing`. A resolved or ambiguous path is never changed by this rule.
- **REQ-RUST-035.0 — impl headers.** Every active impl block, hand-written or macro-generated,
  has one header row: its occurrence, self type, trait when present, and members. An
  unresolved self type or trait carries one reason. itoa has 33.
- **REQ-RUST-043.0 — source container.** Every item has exactly one lexical container: its
  file area, or the enclosing inline module or trait. An impl member uses the container of its
  impl. Self type and trait are metadata, never a second parent. Items in a function body are
  evidence, not nodes.
- **REQ-RUST-045.0 — incomplete checkpoint.** The `full-item-dependency-graph` stage persists
  nodes and edges under one frozen generation, reads them back, and fails if they differ. It
  stays marked `incomplete`, stores unknown visibility as unknown, rejects a stale or
  cancelled input, and leaves the previous output serving on failure.

### Partly met

- **REQ-RUST-034.0 — paths inside bodies.** A path in a function body, const or static
  initializer resolves to an item or is stored unresolved with one reason. A name shadowed by
  a local binding never resolves to an item.
  *Today:* 2 of 61 call sites resolve. Type uses and reads in bodies are not produced.
- **REQ-RUST-036.0 — `Type::member` lookup.** Resolve `Type`, search its inherent impls, then
  impls of traits in scope, and emit a target only when exactly one member matches. Zero or
  several matches are unresolved.
  *Today:* non-generic inherent associated functions only. Trait impls and generic heads are
  stored as `TraitSolverNeeded`.
- **REQ-RUST-032.0 — the full item graph.** Two tables, no others:
  `item-effective-visibility-records` for nodes and `edge-evidence-record-table` for edges.
  Every item outside a function body is a node, private ones included; each source file has
  one file-area node; each active `use` is a node. Every proven dependency is one uncollapsed
  edge with the source range of its site, targeting the real item and never the `use` line.
  *Today:* `contains` and `calls` only; `uses_type`, `reads`, `implements` and `imports` have
  no producer. Nodes cover original items, not macro-generated ones.

### Not started

- **REQ-RUST-042.0 — generic call through a supertrait.** For a method call whose receiver is
  a generic parameter, lookup uses the receiver's type, its bounds and supertrait members. One
  proven declaration gives one `calls` edge to the trait method, never to an implementation.
  Name equality alone selects nothing. Cyclic bounds terminate.
  *itoa:* `i.write(...)` in `Buffer::format`, with `i: I`, `I: Integer`,
  `Integer: private::Sealed`. Exactly one edge, not one per integer type.
  *Test to write:* `test_format_sealed_write_call`, with distractor, shadowing, ambiguity and
  rename checks.
- **REQ-RUST-018.2 — signatures and bounds.** Parameter and return types, `Self`, generic
  parameters, bounds and supertraits are stored as resolved records. They feed REQ-RUST-042.0
  and the `uses_type` and `implements` edges. Every type path resolves or carries one reason.
  Revision 018.1 measured this against an oracle; 018.2 measures it against the expected graph.
- **REQ-RUST-046.0 — macro-generated items as nodes.** Items produced by `macro_rules`
  expansion are nodes with their own identity and the invocation they came from; their
  signatures and bodies are analysed like original ones. New id: the expansion itself is
  REQ-RUST-006.2 in the wider-scope SPEC and has 10 passing tests, but no generated item
  reaches the graph.
  *Blocked on:* the identity and container rule for generated items (OD-2 in the
  [backlog](BACKLOG-deferred-work-open-decisions.md)).
- **REQ-RUST-044.0 — whole-graph acceptance.** The exported nodes, edges, evidence ranges and
  unresolved sites equal the frozen expected graph, with no extra, missing or wrong row; the
  output is identical at 1, 4 and 16 workers; source, configuration and executable digests,
  time, peak memory and disk are recorded.
  *Test to write:* `test_itoa_whole_graph_equality`.
  *Blocked on:* the expected graph is an unreviewed draft ([SOP](SOP-itoa-full-item-graph-acceptance.md) S1).

## Reference

### Where each requirement is checked

```text
PRD acceptance                        requirement                  SOP step
A1 production binary, real source     033.0                        S2
A2 every expected node and edge       032.0 034.0 036.0 042.0      S3
                                      018.2 046.0 035.0 043.0
A3 no extra, missing or wrong row     044.0                        S3
A4 unresolved and inactive sites      034.0 038.1 044.0            S3
A5 digests, time, memory, disk        044.0                        S2, S4
A6 frozen readback, failure safety,   045.0 044.0                  S2, S4
   same output at 1, 4, 16 workers
```

### Tests that exist

```bash
cargo test --release -p parseltongue-cli-dispatcher-binary \
  --test itoa-source-acceptance-tests -- --test-threads=1          # 11 real-source tests
```

Component tests by requirement, counted from doc comments in `crates/`:

```text
045.0  38    043.0  12    034.0  10    036.0  8    032.0  7
035.0   4    033.0   2    038.0   2    038.1  1    044.0  2 (validator and round-trip only)
042.0   0    018.2   0    046.0  0
```

### Open points

- **Two reason lists.** `SiteUnresolvedReasonKind` (sites) and `HldUnresolvedReasonKind` (graph
  edges) differ. One must win before REQ-RUST-044.0 compares unresolved rows.
- **Labels and identities.** The expected graph names items by symbolic label; production uses
  canonical identities. REQ-RUST-044.0 needs the mapping defined.
- **Targets in `core`.** REQ-RUST-038.0 keeps an excluded run from passing. Whether to run
  with the sysroot or accept those edges as unresolved is undecided
  ([PRD](PRD-itoa-full-item-graph-scope.md), Open).

### Requirement ids not listed here

Tests of earlier-work code (name resolution, storage, HLD builder, oracle comparison) cite
ids defined in [the wider-scope SPEC](../phase-20261111/SPEC-wider-scope-requirements-not-current.md),
which is history. This phase adds no test against those ids.
