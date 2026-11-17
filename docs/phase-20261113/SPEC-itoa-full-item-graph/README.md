# SPEC — requirements for the itoa full item graph

Updated 2026-11-17. This folder states only what is in force. Scope: pinned itoa 1.0.18, the
full item dependency graph only.

## The answer

**Parts in build order; 1 to 9 are closed, 10 to 15 are the open points of the PRD the user picked on 2026-11-17 (10 is specified; 11 to 15 are not written yet). Parts 1 to 7 were closed by the acceptance run at commit
`822da80cf`: the whole stored graph equalled the frozen expected graph. Part 8 (a target for the
133 method calls into `core`) was closed by the acceptance run at commit `753665eb9`, with
the scope value `declarations`: 132 of the 133 calls are named and equal their frozen
expected rows, and `cast` is listed without a target. Part 9 (the exported graph as a
DuckDB file, in a separate crate) is built: one full run of pinned itoa on 2026-11-17 gave
a DuckDB file equal to the exported rows, 170 node rows and 682 edge rows.**

Counts and the current step are on the [log](../LOG-itoa-full-item-graph-progress.md)'s
"Where we are" page.

```text
part  file                                       requirements                   state
1     part-1-nodes-and-containment.md            033.0 035.0 038.0 038.1        MET (bounded)
                                                 043.0 045.0
2     part-2-first-call-default-new.md           034.0 036.0                    PARTLY
3     part-3-generic-call-format-write.md        042.0                          BUILT (one shape)
4     part-4-signatures-other-edge-kinds.md      018.2 032.0                    4a BUILT
4b    part-4b-implements-from-impl-headers.md    047.0                          BUILT
4c    part-4c-reads-of-consts-statics.md         048.0                          BUILT (original bodies)
4d    part-4d-imports-from-use-items.md          049.0                          BUILT
5     part-5-macro-generated-items.md            046.0 050.0 051.0 052.0        5a-5c BUILT; 5d steps 1-9, 10a, 10b, 10c, 11 BUILT
6     part-6-targets-in-core.md                  053.0                          6.1-6.5 BUILT (145 nodes; workspace gate not a full PASS: one skipped test)
7     part-7-whole-graph-acceptance.md           043.1, 044.0 to 044.3          acceptance run passed S1 to S4 on 2026-11-16 (log)
8     part-8-method-calls-into-core.md           054.0 to 066.0                 132 of 133 calls named; a `declarations` run equals its frozen expected graph in a test; 1 listed without a target (cast); S1 to S4 passed at `753665eb9`
9     part-9-graph-export-into-duckdb.md         067.0                          built in a separate crate; the full run of 2026-11-17 gave a DuckDB file equal to the export row for row (log); the mapper is not changed
10    part-10-json-extension-built-in.md         068.0                          BUILT 2026-11-17: the mapper runs with an empty HOME and opens no `.duckdb/` path; 62 real-source tests pass
12    part-12-module-contains-file-area.md       070.0                          BUILT 2026-11-17: a module contains the file area of its file; a run equals both re-frozen graphs (170 nodes, 684 edge rows); 63 real-source tests pass
```

MET and BUILT mean the part's real-source tests pass on pinned itoa. The last full run of
the 60 tests was at commit `eb031e680` (60 pass, log). Neither word means the SOP passed:
only an acceptance run decides that.

Out of scope: every other repository, any rust-analyzer export or oracle comparison, and
every higher-abstraction table (interface level, HLD, file and folder roll-ups).

## How a part is written

1. **One part per target, written before its code.** When a target is picked, write its part
   with the skill `rust-executable-specs-01`, in that skill's seven
   sections: work mode, executable requirements, design by layer, verification matrix,
   implementation plan, quality gate results, open questions.
2. **Until a part is picked it is a stub**: the requirement in a few lines and what blocks it.
   Do not design ahead.
3. **Built parts are records.** Parts 1 and 2 state the requirement, the tests that exist and
   what is still missing. They are not rewritten into seven sections after the fact.
4. **This index changes with every part**: update the state column and the line under "The
   answer" in the same commit as the part.

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

## Reference

### Where each requirement is checked

```text
PRD acceptance                        requirement                  SOP step
A1 production binary, real source     033.0                        S2
A2 every expected node and edge       032.0 034.0 036.0 042.0      S3
                                      018.2 047.0 046.0 035.0 043.0
A3 no extra, missing or wrong row     044.0                        S3
A4 unresolved and inactive sites      034.0 038.1 044.0            S3
A5 digests, time, memory, disk        044.0                        S2, S4
A6 frozen readback, failure safety,   045.0 044.0                  S2, S4
   same output at 1, 4, 16 workers
```

### Tests that exist

```bash
cargo test --release -p parseltongue-cli-dispatcher-binary \
  --test itoa-source-acceptance-tests -- --test-threads=1          # 60 real-source tests
```

Occurrences of each requirement id in the Rust files under `crates/`, counted 2026-11-17
at commit `eb031e680` (one occurrence can cover several tests):

```text
018.2  15    032.0   7    033.0   2    034.0  11    035.0   4    036.0   8
038.0   4    038.1   1    042.0  17    043.0  12    043.1   4    044.0   5
044.1   3    044.2   1    044.3   2    045.0  38    046.0  21    047.0  13
048.0  16    049.0  11    050.0   8    051.0   9    052.0  72    053.0  60
054.0  29    055.0  11    056.0  14    057.0   7    058.0   5    059.0  15
060.0   9    061.0   6    062.0  27
```

### Open points across parts

- **Two reason lists.** `SiteUnresolvedReasonKind` (sites) and `HldUnresolvedReasonKind` (graph
  edges) both still exist in the code.

### Requirement ids not listed here

Tests of earlier-work code (name resolution, storage, HLD builder, oracle comparison) cite
ids defined in [the wider-scope SPEC](../../phase-20261111/SPEC-wider-scope-requirements-not-current.md),
which is history. This phase adds no test against those ids.

Five of the 60 real-source tests are tagged REQ-RUST-040.0 (top-level containment through
the `original-top-level-containment` stage). They guard source ingest, module ownership and
visibility scope on pinned itoa, so they stay in the suite, but that stage's output is not
this phase's deliverable.
