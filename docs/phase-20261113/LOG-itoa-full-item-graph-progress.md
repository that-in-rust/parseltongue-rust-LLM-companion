# LOG — progress on the itoa full item graph

**Where we are** (below) is rewritten in place whenever it changes. **Entries** are added at
the bottom, never edited. The full log up to step 7.8 (2,788 lines) is in git:
`git show cb40f3d2f:docs/phase-20261113/LOG-itoa-full-item-graph-progress.md`.

## Where we are (2026-11-16, commit `822da80cf`)

**The mapper stores a graph of pinned itoa 1.0.18 that equals the frozen expected graph:
151 nodes and 682 edge rows. The acceptance run passed SOP steps S1 to S4 on one frozen
executable. The graph is marked `incomplete` because 133 method calls into `core` have no
target, and the expected graph was written and checked by one author.**

### What the pinned itoa run stores (measured)

```text
nodes  151   88 items of itoa (45 written by macro expansions), 47 names outside itoa,
             13 builtin types, 2 file areas, 1 crate root
edges  682   relation    rows  with a target
             calls        197    64   (133 method calls into core have none)
             uses_type    203   203
             reads         98    98
             contains      88    88
             generates     45    45
             expands       15    15
             implements    32    32
             imports        4     4
run          15.3 to 19.3 s wall, 370 to 414 MB peak (whole process tree), 21.0 MB store
workers      1, 4 and 16, twice each, store byte-identical node and edge rows
```

### What the whole-graph test compares (`test_itoa_whole_graph_equality`, passes)

- **Nodes, all 151:** label, category, item kind, owner kind, file, generating macro call,
  declared in a function body or not, byte range in the file (the 43 nodes written directly
  in source), and the kind of a name outside itoa.
- **Sites of calls, type uses, reads, imports and macro rows, all 562:** source, relation,
  target, exact byte range, and the record (how a call's target was chosen; signature or
  expression; body or macro argument).
- **`contains` and `implements`, 120 rows:** source, relation and target only.
- **Not compared:** the lookup evidence kept on a row, and the range of a node written by
  a macro expansion.

### Checks at the last full run

```text
real-source itoa tests   58 pass, 0 fail (872 s, one thread, frozen executable)  at 822da80cf
workspace tests          602 pass, 0 fail, 4 ignored                   at 822da80cf
                         test_lang_items_in_sysroot skipped; it fails when run
format, lint, release    pass                                          at 822da80cf
expected-data tests      9 pass (7 tests of the removed "blocked rows" layer deleted)
```

The workspace gate is not a full PASS while that test is skipped. It reads the native
sysroot, which itoa runs exclude.

### SOP status

```text
S1  freeze inputs and expected graph   PASS   digests equal the pin and the freeze record
S2  gates and production run           PASS   audit clean; one workspace test skipped (accepted by the user)
S3  exact comparison                   PASS   58 tests on the frozen executable
S4  determinism, time, memory          PASS   six runs identical; 11 ms samples
```

Run report: the last entry below. Executable sha256
`43d999e173241c4b9a8011aade1871916a5c92cc9c86157dd8e92f6d1e0d1b88`.

### The code, from the codebase-memory index of this commit

Index: 26,725 nodes, 62,052 edges; 1,628 functions, 619 methods, 7,186 call edges.

```text
crate                                   src lines  test fns  what it does
parseltongue-syntax-span-extractor         16,113       154  parses files; items, body sites, call/type/read rows
parseltongue-duckdb-analysis-adapter       19,471       187  name resolution in SQL (949 lines), node rows
parseltongue-turso-store-adapter            6,488        75  durable store, generations, publication
parseltongue-port-macro-expander            4,441        22  expands macro_rules! calls
parseltongue-core-row-types                 2,843        23  row and identity types
parseltongue-cli-dispatcher-binary          2,143       126  the parseltongue binary; itoa tests (11,382 lines)
parseltongue-crate-graph-loader               644        12  cargo metadata to crate graph
parseltongue-table-store-traits               494         1  store traits
parseltongue-macro-span-records               236         2  macro span rows
parseltongue-stage-worker-scheduler            67         4  stage scheduling
```

"what it does" is my reading of each crate, not a graph query result.

### What does not work or is not built

- **133 method calls have no target** (`buf[i].write(..)`, `x.try_into()`, `MAX.ilog10()`,
  `string.len()`). The receiver's type is declared in `core`, which the run does not read.
- **Nodes store no visibility, reachability or signature.** The PRD allows "unknown".
- **No edge from a module to the contents of its file** (OD-7). `itoa` and `u128_ext` do
  not lead to their items by an edge.
- **Struct fields and enum variants are not nodes** (open in the PRD).
- **Reading the sysroot** did not finish in 2,400 s on 2026-11-13; itoa runs exclude it.
- **Rows depend on the source directory.** Two copies of the pinned source in different
  directories store different run ids and macro-expansion ids. Cause: the store's file
  identity is the absolute path, and the run id hashes those rows. Not fixed.
- **Only itoa has been run** in this phase. Other repositories and higher-level tables are
  out of scope.

### What is left

The SOP run is done. Open, each needing a decision from the user before any work:

1. The 133 method calls without a target (needs `core` read, or a trait solver).
2. Why rows depend on the source directory.
3. `test_lang_items_in_sysroot`.
4. The PRD's open points: struct fields and enum variants as nodes; the edge from a module
   to its file's contents; visibility, reachability and signature on nodes.

## Entries — add new ones at the bottom

### Part 7 step 7.9 — node position and external-name kind compared (2026-11-16)

- **REQ:** REQ-RUST-044.1 N4 to N6 (part 7 file, section 7).
- **STUB / RED:** `derive_stored_node_position` and `derive_stored_external_kind` as
  `todo!()`. `test_itoa_whole_graph_equality`: `0 passed; 1 failed; 57 filtered out`,
  `not yet implemented: REQ-RUST-044.1 N4: Symbol(Builtin("i128"))`. This shows the test
  reaches the helper, nothing more.
- **GREEN:** `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 57 filtered out`.
  All 151 nodes match, 43 of them with a range.
- **Five changes to the expected files, each fails the test:** a node start +1, a node end
  -1, a range given to a macro-generated node, `builtin_member` changed to
  `prelude_name`, one site range dropped from a calls row.
- **Weak evidence:** the range rule (leading comments and attributes belong to the node) was
  written after I had seen that production's ranges start there.
- **Command:** `cargo test --release -p parseltongue-cli-dispatcher-binary --test
  itoa-source-acceptance-tests test_itoa_whole_graph_equality`. Expected-data tests: 16 pass.
  Format and lint on the touched crate pass. No production code changed. No refactor needed.
- **codebase-memory:** re-indexed before this step (26,725 nodes, 62,052 edges);
  `search_graph` for the full-run entry functions returned no rows, so the files were read
  directly.

### Part 7 step 7.10 — the expected graph is frozen (2026-11-16)

- **REQ:** REQ-RUST-044.3 F1 to F5 (part 7 file, section 9).
- **What changed:** the builder writes `itoa-expected-graph-freeze.json` (sha256 of the five
  expected files, of the builder and rules files, and of the two source files) only with
  `--write-freeze-manifest`. The "blocked rows" layer of 2026-11-14 is removed: its two
  output files, its classifier and its 7 tests. Every reason it listed is decided since.
- **Frozen digests:** nodes `29aed6ef…f7954e7`, edges `a51f7d6e…d06b156`; all seven are
  in the freeze record.
- **STUB / RED:** `verify_frozen_expected_files` as `todo!()`:
  `0 passed; 1 failed; 57 filtered out`, `not yet implemented: REQ-RUST-044.3 F2`. The two
  new Python tests errored on the missing record. Both show reach only.
- **GREEN:** `test_itoa_whole_graph_equality`: `1 passed; 0 failed; 57 filtered out`.
  Expected-data tests: `Ran 9 tests`, `OK`.
- **One change that fails it:** a newline appended to the edges file:
  `itoa-expected-graph-edges.json differs from the frozen expected graph`.
- **SOP changed in place:** Rule 1 (frozen, one author), Rule 3 (an incomplete run passes
  only when every row without a target is expected), S1 and S2 pass lines, S4 (exports
  compared after the header line).
- **Limit:** the review is by the author alone. No production code changed. No refactor.
- **codebase-memory:** no query; only test and dataset files changed.

### Part 7 step 7.11 — the acceptance tests run a named frozen executable (2026-11-16)

- **REQ:** REQ-RUST-044.3 F6.
- **Why:** the first acceptance run copied `target/release/parseltongue` after
  `cargo build --release` (sha256 `43d999e1…`). The 58 tests then ran
  `CARGO_BIN_EXE_parseltongue`, which `cargo test` builds separately (sha256
  `2803c417…`). That comparison did not count.
- **Change:** `read_mapper_executable_path` returns `PARSELTONGUE_FROZEN_EXECUTABLE_PATH`
  when set. Test file only.
- **Evidence it takes effect:** with the variable set to `/bin/false`,
  `test_itoa_whole_graph_equality`: `0 passed; 1 failed; 57 filtered out`. Set to the
  frozen executable: `1 passed; 0 failed`. Unset: `1 passed; 0 failed`.
- **No stub step:** the change is one function in a test file; the `/bin/false` run is the
  failing evidence. Commit `822da80cf`.

### Acceptance run report — pinned itoa, frozen executable (2026-11-16)

```text
source      itoa af77385d0daf4d0e949e81f2588be2e44f69f086, clean
            src/lib.rs       e962338e1886873aa7c3399ec89aa5378ccbfed712b9690f043628079390a34f
            src/u128_ext.rs  498cc94f13805e551248fd82f47cc318f51d953a5002a9f8698858cd12fbae3c
            Cargo.toml 44ca1ae5…dec0483, Cargo.lock 965fa758…a8d019d, benches/bench.rs 29a0bdb9…498b154
executable  43d999e173241c4b9a8011aade1871916a5c92cc9c86157dd8e92f6d1e0d1b88 (103,268,432 bytes),
            built by cargo build --release at 8580ca2bd; same digest after the last step
expected    nodes 29aed6efdaef15ad4564b787ee381d6fec16ac23cdc726bc9460db824f7954e7
            edges a51f7d6e499d8340e1d37b6b809366a307a3c903b14758482e6cba69ad06b156
commits     S1, S2, S4 at 8580ca2bd; S3 and the workspace gates at 822da80cf
            (822da80cf changes one test function and documents; no production code)
```

```text
S1  PASS  source commit and digests equal the pin; the builder reproduces the frozen files;
          9 expected-data tests pass
S2  PASS  run publishes; completeness = incomplete (recorded, Rule 3)
          untraced, 4 workers: 15.14 s, 387.6 MB peak, 1,388 samples
          strace audit: auditor pass, no violation rows; 0 connect calls; direct children are
          cargo metadata --offline, rustc --print cfg, rustc --print sysroot; the only file
          opened under a target/ directory is .rustc_info.json
S3  PASS  58 passed, 0 failed, one thread, 872 s, on the frozen executable
          151 nodes and 682 edge rows equal the frozen expected graph; 133 calls without a
          target, each an expected row
S4  PASS  six runs, node and edge rows byte-identical after the header line
          nodes body cdc95c726e4e8015f899699198973aa75d6f60d9a8f3bb975766328f4d52235d
          edges body 4560b32c54b48dd2c3dc15403c87623801e7b253cce525fc2cdf4e63eeff17f9
```

```text
workers  wall s  peak MB  store bytes  nodes  edges
1        18.46   385.1    21,046,310   151    682
1 again  15.27   370.7    21,046,310   151    682
4        19.29   384.4    21,046,310   151    682
4 again  16.09   369.8    21,046,310   151    682
16       18.89   393.3    21,046,310   151    682
16 again 16.26   413.9    21,046,310   151    682
```

- **Workspace gates at `822da80cf`:** format, lint (all targets, all features) and release
  build pass. Tests: 602 passed, 0 failed, 4 ignored, with `test_lang_items_in_sysroot`
  skipped. That test fails when run; the user accepted it on 2026-11-16 as a known open
  defect outside the itoa run.
- **Deviations from the SOP text:**
  - The run maps a copy of the pinned source (the test tree, byte-equal to the pinned
    checkout for the four source files) with the pinned `Cargo.lock` added. The checkout
    has no `Cargo.lock`.
  - Memory samples are 11 ms apart, not 10: the sampler sleeps 10 ms and reads `/proc`.
  - The first S4 pass (same executable, another directory) used a 19 ms sampler; it is
    replaced by the table above. Its six runs were also identical to each other.
  - The first S3 pass ran the wrong executable (step 7.11) and is replaced.
- **Found by the run:** the six runs of the first S4 pass and the six of the second map the
  same source from two directories. Their rows differ in `run_generation_digest_full` and
  in the ids inside `expansion_provenance_chain`, in every row. Replacing the directory
  name in the text does not make them equal. Cause not found. S4 compares runs of one
  directory, so it passes.
- **Limits:** the graph is marked incomplete (133 calls); the expected graph was written and
  checked by its author alone; only itoa was run.
- **Commands:** `/home/ubuntu/phase7-work/accept/run-acceptance.sh`, `run-s3.sh`,
  `run-s4.sh` (not in the repository); they run the SOP commands with
  `--sysroot-source-scope-kind excluded`.
- **codebase-memory:** re-indexed after this entry; no query was needed for the run.

### Part 8 step 8.0 — measurement for the 133 method calls (2026-11-16)

- **Asked by the user:** resolve the 133 method calls without a target.
- **Measured, no production change:** the table, the failing sysroot run (170.94 s,
  1,555.9 MB, store copy mismatch) and the syntax-only scan of `core` (0.27 s, 22 MB) are in
  [part 8](SPEC-itoa-full-item-graph/part-8-method-calls-into-core.md), sections 1 to 3.
- **Decision:** build a syntax-only list of the methods `core` declares, the receiver's
  type from itoa's source, and Rust's lookup order over both (steps 8.1 to 8.6). Rejected:
  fixing the full sysroot run first; it would still need type inference, which is not
  built.
- **Source-directory dependence:** cause found (absolute path in the file identity); not
  fixed.
- **Acceptance:** the run of 2026-11-16 stays valid until the first production change of
  step 8.1.

### Part 8 step 8.1a — member declaration rows from one file (2026-11-16)

- **REQ:** REQ-RUST-054.0 D1 to D5 (part 8, section 6).
- **STUB:** `extract_member_declaration_rows` with `todo!`, in
  `crates/parseltongue-syntax-span-extractor/src/sysroot-member-declaration-records.rs`.
- **RED:** `cargo test --release -p parseltongue-syntax-span-extractor sysroot_member`

```text
test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 137 filtered out
each: not yet implemented: REQ-RUST-054.0
```

- **GREEN:** same command

```text
test sysroot_member_declaration_tests::test_macro_body_member_rows ... ok
test sysroot_member_declaration_tests::test_member_rows_offset_order ... ok
test sysroot_member_declaration_tests::test_inherent_member_row_fields ... ok
test sysroot_member_declaration_tests::test_trait_member_row_fields ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 137 filtered out
```

- **Whole crate:** 141 + 17 tests pass; format and lint pass on the crate.
- **REFACTOR:** none needed; the function is a pure walk with one row builder per owner
  kind.
- **Weak evidence:** the four tests failed only on `todo!`, so RED shows they ran, not that
  each assertion can fail. No deliberate breaks yet.
- **Not built:** nothing calls the function. No run reads `core`. Step 8.1b adds the stage.
- **Decision:** the D3 text also covers a `macro` definition, not only `macro_rules!`.
