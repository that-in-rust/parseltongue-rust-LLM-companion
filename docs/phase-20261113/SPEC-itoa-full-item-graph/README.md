# SPEC — requirements for the itoa full item graph

Updated 2026-11-14. This folder states only what is in force. Scope: pinned itoa 1.0.18, the
full item dependency graph only.

## The answer

**Fourteen requirements in seven parts, in build order. Parts 1 and 2 are built for the
bounded graph. Part 3 is built for one receiver shape: the generic call
`Buffer::format --calls--> private::Sealed::write` is stored. Part 4a is built: `uses_type` rows from original signatures. Part 4b is built: 32
`implements` rows from impl headers, including the 28 macro-generated ones. Part 4c
is built: 21 `reads` rows from original function bodies, 15 with a target. Part 4d
is built: 4 `imports` rows from the 3 `use` nodes, all unresolved until part 6 names
`core` targets.**

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
5     part-5-macro-generated-items.md            046.0                          5a-5c BUILT; 5d steps 1-9, 10a, 10b, 10c, 11 BUILT
6     part-6-targets-in-core.md                  (decision first)               DECIDED, not built
7     part-7-whole-graph-acceptance.md           044.0                          NOT STARTED
```

MET means the part's real-source tests pass on pinned itoa (11 of 11 at commit `15643ec4f`).
It does not mean the SOP passed: only part 7 decides that.

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
  --test itoa-source-acceptance-tests -- --test-threads=1          # 22 real-source tests
```

Doc comments naming each requirement in `crates/`, counted 2026-11-15 (one comment can
cover several tests):

```text
042.0  17    045.0  15    018.2  15    034.0  10    032.0  7
043.0   6    036.0   5    038.0   4    033.0   2    035.0  1
038.1   1    044.0   1    046.0   0    047.0  13
```

### Open points across parts

- **Two reason lists.** `SiteUnresolvedReasonKind` (sites) and `HldUnresolvedReasonKind` (graph
  edges) differ. One must win before part 7 compares unresolved rows.
- **Labels and identities.** The expected graph names items by symbolic label; production uses
  canonical identities. Part 7 needs the mapping defined.

### Requirement ids not listed here

Tests of earlier-work code (name resolution, storage, HLD builder, oracle comparison) cite
ids defined in [the wider-scope SPEC](../../phase-20261111/SPEC-wider-scope-requirements-not-current.md),
which is history. This phase adds no test against those ids.

Five of the eleven real-source tests are tagged REQ-RUST-040.0 (top-level containment through
the `original-top-level-containment` stage). They guard source ingest, module ownership and
visibility scope on pinned itoa, so they stay in the suite, but that stage's output is not
this phase's deliverable.
