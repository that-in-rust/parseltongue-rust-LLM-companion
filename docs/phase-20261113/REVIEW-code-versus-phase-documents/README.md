# Review: the code as built versus the phase-20261113 PRD

Measured on branch `research/tiny-rust-code-mapper` at commit `37ea1755d` (2026-11-16).
Read: `AGENTS.md`, PRD, ARCH, SOP, MAP, BACKLOG, LOG, SPEC README and part 8.
Tools: source reads, `grep`, Cargo manifests and lock file, `codebase-memory-mcp` 0.12.1
(26,719 nodes, 62,875 edges; some SQL files only partly parsed, so SQL claims come from source reads).

**Not done:** nothing was built or run. This VM has no `~/repos/rust-tools` (the rust-analyzer
checkout and pinned itoa), so `cargo` cannot load the workspace. Test results below are quoted
from the LOG, not re-measured.

## The answer

**The code follows the PRD's architecture: Rust resolves, DuckDB projects, Turso stores and
reads back, and one CLI stage builds the graph. Three things differ. HEAD is 16 commits past
the accepted commit and has a failing test. Part 8 reads `core` source, which the PRD, SOP and
ARCH still say stays out of the run. The documents disagree with each other about what is built.**

This is a dated review, not a document in force. It changes no rule.

Diagrams (C4-PlantUML sources beside them):

- [parseltongue-crate-architecture.png](parseltongue-crate-architecture.png): crates and stores.
- [full-graph-run-pipeline.png](full-graph-run-pipeline.png): the run, in call order.

## 1 — Where the code matches the PRD

- **Ownership.** Parsing, macro expansion and all five edge resolvers are Rust
  (`syntax-span-extractor`). Name-resolution rounds and node projection are DuckDB. Durable
  rows, snapshots, write and readback are Turso. No third store.
- **Dependency direction.** `core-row-types` depends on nothing. Adapters depend on the ports
  crate, not on each other. (MAP says the DuckDB adapter depends on Turso; today that is a
  dev-dependency only.)
- **Process boundary.** One `Command::new` in production code, `crate-graph-loader/src/lib.rs:48`.
- **Nothing keyed on itoa.** No `itoa`, `MAX_STR_LEN` or `impl_Integer` in non-test code.
- **Edge kinds.** The relation enum has 10 kinds; 8 have a producer (`calls`, `uses_type`,
  `reads`, `implements`, `imports`, `contains`, `generates`, `expands`). `reexports` and
  `derives` have none. This is what the PRD says itoa needs.
- **Readback.** `publish_full_graph_checkpoint` writes, reads back and fails the run if the
  two differ. `completeness=incomplete` is a constant in the code, as the MAP says.

## 2 — Where the code differs from the PRD

### 2.1 HEAD is not the accepted state

- The PRD headline (151 nodes, 682 edge rows, S1 to S4 PASS) is for commit `822da80cf`.
- Since then: 16 commits, 2,173 lines added in 13 production files (part 8).
- The LOG says `test_itoa_core_member_targets` fails at HEAD (17 targets found, 9 expected).
- So no SOP status applies to HEAD until S2 to S4 are run again.

### 2.2 Part 8 reads `core` source; four documents say it does not

- PRD (decision of 2026-11-15), SOP line 186, ARCH C3 and `AGENTS.md` line 80: the sysroot
  source stays out of the run; runs pass `--sysroot-source-scope-kind excluded`.
- Code: a third value `declarations`, a new stage `sysroot-member-declaration-stage`, a new
  table `sysroot-member-declaration-records`, and `persist_sysroot_member_rows`, which reads
  `<sysroot>/lib/rustlib/src/rust/library`.
- Only SPEC part 8 and the LOG describe this. `AGENTS.md` line 203 requires ARCH, SOP and
  `AGENTS.md` to change in the same commit as an architecture change.
- Open question I could not answer from the code read so far: whether those `core` files are
  covered by the generation digest and by the inputs frozen in SOP S1.

### 2.3 Node fields the ARCH requires are not built

- ARCH G4, G5, E9: each node stores effective visibility, reachable-outside-crate, and
  signature text.
- Code: all three are empty on every node. "Unknown" is also written two ways: `None` for
  original items (`original-item-graph-projection.rs:80`), `Some(Unknown)` for
  macro-generated items (`generated-item-graph-projection.rs:123`).
- The PRD stage table does say "visibility stored unknown". The whole-graph test does not
  compare these fields.

### 2.4 Stages 6 and 7 are one function, not two stages

- PRD stage table: stage 6 writes lowered signatures; stage 7 writes edge evidence.
- Code: `lowered-signature-type-table` is registered but nothing writes it. Signature types go
  straight to `uses_type` edges. All resolvers run inside one CLI facade and the node set
  passes from DuckDB to Rust in memory, then one call writes both
  `item-effective-visibility-records` and `edge-evidence-record-table`.
- PRD: "No memory crosses a stage boundary." This holds only if the checkpoint counts as one
  stage. Worth stating in the PRD either way.

### 2.5 Smaller differences

- **DuckDB pin.** PRD: "no extensions, no external access". Code opens with external access
  on, runs `LOAD json`, then turns external access off
  (`hld-original-source-projection.rs:85-103`).
- **Errors.** `AGENTS.md`: "Errors are typed enums. No free-text-only variant." Six error
  types are a struct with one text field; `StoreBackendFailureError` is built 45 times in
  non-test code.
- **Network-capable code in the lock file.** `libduckdb-sys` pulls in `ureq` and `rustls`.
  I did not check whether any of it is reachable at run time.
- **Largest function.** `build_original_call_edge` is 336 lines
  (`original-call-resolution-records.rs:135-470`), the highest complexity in the workspace
  per codebase-memory. Part 8 is adding to this function.

## 3 — In the tree but out of scope

The PRD rules out oracle comparison, HLD tables and rollups. They are still compiled in:

- CLI verb `oracle-edge-differ-run` and the HLD and oracle facades.
- About 4,700 lines of the DuckDB adapter (HLD builder, oracle comparison).
- 6 of the 36 stage-owned tables: five `hld-*` and `oracle-edge-agreement-table`.

The MAP already says this. It is not a violation; it is a third of the largest crate.

## 4 — Where the documents disagree with each other

```text
document     stale statement                                            what is true at HEAD
PRD s.2      stage 6 "impl headers only", stage 7 "contains and         8 edge kinds produced (PRD headline)
             calls only"
PRD s.3      build order: "next Buffer::format -> Sealed::write"        built; part 7 passed
ARCH top     "Not built: uses_type, reads, implements, imports"         all four built
ARCH s.4     "What remains" lists the same targets as not built         built
MAP          103 files, 50,210 lines; "only two edge kinds have a       138 files, 70,497 lines; 8 kinds
             producer"; flag values "included | excluded"               a third value, declarations
SPEC README  "Updated 2026-11-14"; "fourteen requirements in seven      8 parts; 2,173 production lines of
             parts"; part 8 "nothing built"; "11 of 11" tests           part 8; 60 tests in the itoa test file
BACKLOG      "No comparator for full-graph rows. S3 has nothing         test_itoa_whole_graph_equality exists;
             to run yet"; List A still lists core as undecided          core decided further down the same file
AGENTS.md    "Where we are (2026-11-14): a bounded checkpoint"          S1 to S4 passed on 2026-11-16 at 822da80cf
```

The LOG's "Where we are" page and the PRD headline are the two places that are current.

## 5 — What codebase-memory got wrong

Its package-boundary report claims 331 calls from the DuckDB adapter into the macro expander.
There is no such Cargo dependency; the calls are name matches on `map` and `iter`. Every claim
above was checked against source lines.
