# LOG — progress on the itoa full item graph

**Where we are** (below) is rewritten in place whenever it changes. **Entries** are added at
the bottom, never edited. The full log up to step 7.8 (2,788 lines) is in git:
`git show cb40f3d2f:docs/phase-20261113/LOG-itoa-full-item-graph-progress.md`.

## Where we are (2026-11-17, acceptance run of commit `753665eb9`, scope `declarations`)

**Accepted: the mapper stores a graph of pinned itoa 1.0.18 equal to the frozen expected
graph of a `declarations` run, row for row (170 nodes, 682 edge rows), and the acceptance
run passed SOP steps S1 to S4 on one frozen executable. 132 of the 133 method calls into
`core` are named. The graph is still marked `incomplete`: one call, `cast`, has no target,
and it is an expected row. The expected graph was written and checked by one author.**

**Since that run the code has changed (SPEC parts 10 to 14, 2026-11-17 and 18): the
`json` extension is inside the executable; each module contains the file area of its
file; every item node stores its visibility, whether it is reachable from outside the
crate, and its signature; each struct field is a node; and stored rows no longer depend
on the source directory. The run now stores 172 nodes and 686 edge rows. Both expected
graphs are re-frozen with those rows and the two equality tests pass at `a57f7e7b7`, but
S1 to S4 are NOT STARTED for this head. They run once, after part 15 (`cast`).**

**A separate crate copies the exported graph into one DuckDB file, and the full run from
pinned itoa to that file passed at `c21eacd2b`: the file's rows equal the exported rows,
170 of 170 and 682 of 682 (under "What is left").**

### The two accepted runs

```text
                              822da80cf      753665eb9
scope                         excluded       declarations
nodes                         151            170
edge rows                     682            682
calls with a target            64 of 197     196 of 197
edge rows with a target       549 of 682     681 of 682
method calls into core named    0 of 133     132 of 133   equal to the frozen rows: 132, wrong: 0
real-source tests             58 pass        61 pass
```

- **Named:** 49 `MaybeUninit<T>::write`, 39 `[T]::get_unchecked`, 13 `TryInto::try_into`,
  8 `Result<T, E>::expect`, 5 `Result<T, E>::unwrap`, 5 `[T]::len`, 1 `Iterator::rev`,
  1 `[T]::as_mut_ptr`, 1 `str::len`, and `ilog10` once on each of `u8`, `u16`, `u32`,
  `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`.
- **Without a target (1):** `cast` in `Buffer::format`. Its impl has the bound
  `T: PointeeSized`, which the run cannot prove harmless (immediate backlog, entry 7).
- **The `core` files are frozen and in the run id:** 288 `.rs` files, 6,419,461 bytes,
  listing sha256 `39a4d534…ea40780` (REQ-RUST-065.0).
- **Both expected graphs stay frozen.** `test_itoa_whole_graph_equality` compares a run
  with `excluded`, `test_itoa_declarations_graph_equality` a run with `declarations`.

### What the pinned itoa run stores (measured, scope `declarations`)

```text
nodes  172   88 items of itoa (45 written by macro expansions) and 2 fields, 66 names
             outside itoa (19 of them methods declared in core), 13 builtin types,
             2 file areas, 1 crate root
edges  686   relation    rows  with a target
             calls        197   196   (cast has none)
             uses_type    203   203
             reads         98    98
             contains      92    92   (88 to items, 2 from a module to its file area,
                                       2 from a struct to its field)
             generates     45    45
             expands       15    15
             implements    32    32
             imports        4     4
run          22.5 to 26.6 s wall, 363 to 404 MB peak (whole process tree), 36.8 MB store
workers      1, 4 and 16, twice each, store byte-identical node and edge rows
```

### What the two whole-graph tests compare

- **Nodes, all of them:** label, category, item kind, owner kind, file, generating macro
  call, declared in a function body or not, byte range in the file (the 43 nodes written
  directly in source), and the kind of a name outside itoa.
- **Sites of calls, type uses, reads, imports and macro rows, all 562:** source, relation,
  target, exact byte range, and the record (how a call's target was chosen; signature or
  expression; body or macro argument).
- **`contains` and `implements`, 120 rows:** source, relation and target only.
- **The 133 calls into `core`** (`test_itoa_core_member_targets`): also the crate, the
  owner kind, the declaration's file and line, and the assumptions the expected row asks
  for.
- **Not compared:** the rest of the lookup evidence kept on a row, and the range of a node
  written by a macro expansion.

### Checks at the last full run

```text
real-source itoa tests   61 pass, 0 fail (966 s, one thread)                   at c21eacd2b
workspace tests          25 test binaries: 657 pass, 0 fail, 4 ignored, exit 0     at c21eacd2b
                         (1,131 s, max RSS 674 MB). The loader crate adds 14 tests.
                         test_lang_items_in_sysroot skipped; it fails when run
format, lint, release    pass (workspace clippy, -D warnings, and build)        at c21eacd2b
expected-data tests      25 pass                                                at 753665eb9
```

The workspace gate is not a full PASS while that test is skipped. It reads the whole
native sysroot, which no itoa run does.

### SOP status

```text
S1  freeze inputs and expected graph   PASS   source, compiler and core digests equal the freeze record
S2  gates and production run           PASS   audit clean; one workspace test skipped (accepted by the user)
S3  exact comparison                   PASS   61 tests on the frozen executable
S4  determinism, time, memory          PASS   six runs at 1, 4 and 16 workers store identical rows
```

Run report: the last entry below. Executable sha256
`af84bb460f6d73cf0fb465117cfad671f94fab339497c72a0efc6872771cf354`.

### The code, from the codebase-memory index of `822da80cf` (not re-measured here)

Counts at the head are in the [MAP](MAP-parseltongue-crates-as-measured.md).

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

- **One method call has no target:** `cast` (above).
- **The accepted run is not of the current executable.** Part 10 changed production code
  (the `json` extension is now inside the executable), and parts 11 to 15 will change the
  graph. S1 to S4 are NOT STARTED for the new head; they run once, after part 15.
- **Nodes store no visibility, reachability or signature.** The PRD allows "unknown".
- **No edge from a module to the contents of its file** (OD-7). `itoa` and `u128_ext` do
  not lead to their items by an edge.
- **No edge from a trait member to the impl members that implement it**, and none from a
  type to its methods (immediate backlog, entries 1 to 3).
- **Struct fields and enum variants are not nodes** (open in the PRD).
- **Reading the whole sysroot** did not finish in 2,400 s on 2026-11-13, and stops with a
  store copy mismatch since. No itoa run does it.
- **Rows depend on the source directory.** Two copies of the pinned source in different
  directories store different run ids and macro-expansion ids. Cause: the store's file
  identity is the absolute path, and the run id hashes those rows. Not fixed.
- **Only itoa has been run** in this phase. Other repositories and higher-level tables are
  out of scope.

### What is left

Part 8 is done as far as it was decided: the acceptance run of `753665eb9` passed.

**Part 9, the exported graph as a DuckDB file, is built and its full run passed**
(2026-11-17, commit `c21eacd2b`; user decision of the same day). The user's bar: one full
run in which pinned itoa goes through `parseltongue` and the DuckDB file equals the
exported rows.

```text
1 mapper   pinned itoa, scope declarations, frozen executable   22.0 s, 354.8 MB   170 nodes, 682 edge rows
2 export   the node and edge tables as JSON lines; rows equal the S2 and S4 exports
3 loader   parseltongue-graph-dataset-loader                      0.87 s, 87.9 MB    one DuckDB file, 7,876,608 bytes
4 check    a Python script outside the repository reads the file  170 of 170 node rows and 682 of 682 edge rows
                                                                  equal the JSON in all 11 columns
```

- 681 edge rows join to a target node; 1 (`cast`) has no target and the reason
  `TraitSolverNeeded`.
- The two export files keep their digests; the loader opens no file under `~/.duckdb`,
  makes no network call and starts no process.
- In the repository the same is tested by `test_itoa_export_dataset_roundtrip`.
- The DuckDB file is a copy for SQL. It is not a store of the run and no acceptance step
  reads it.

Open, each needing a decision from the user:

1. `cast`: which source facts prove that every type meets `PointeeSized`.
2. The DuckDB `json` extension: closed by part 10 (built into the executable).
3. Why rows depend on the source directory.
4. `test_lang_items_in_sysroot`.
5. The PRD's open points: struct fields and enum variants as nodes; the edge from a module
   to its file's contents; visibility, reachability and signature on nodes.
6. The navigation links of the immediate backlog (entries 1 to 3).

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

### Part 8 step 8.1b — the scan as a stage of the run (2026-11-16)

- **REQ:** REQ-RUST-054.0 D6 to D9 (part 8, section 6).
- **STUB:** `collect_sysroot_member_rows` and `persist_sysroot_member_rows` with `todo!`, in
  `crates/parseltongue-cli-dispatcher-binary/src/sysroot-member-declaration-facade.rs`.
- **RED:**

```text
cargo test --release -p parseltongue-cli-dispatcher-binary --bin parseltongue test_core_directory_member_rows
  not yet implemented: REQ-RUST-054.0 D6
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out
cargo test --release -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests test_core_member_rows_published
  Error: "unknown sysroot source scope declarations"
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 58 filtered out
```

- **GREEN:** same two commands

```text
test_core_directory_member_rows ... ok     1 passed; 0 failed
test_core_member_rows_published ... ok     1 passed; 0 failed; finished in 31.22s
```

- **What changed:** a third value `declarations` for `--sysroot-source-scope-kind`; the
  stage table `sysroot-member-declaration-records`, registered in the Turso adapter; the
  stage runs after the impl header rows.
- **Measured on pinned itoa, 4 workers, `declarations`:**

```text
rows written        5,336 in 0.500 s
whole run           15.63 s
main process peak   363.7 MB   (one process, not the whole tree)
store               28,377,088 bytes   (21,046,310 with excluded)
nodes, edges        151, 682, the same labels as an excluded run
```

- **Deliberate breaks, each fails a test:** `declarations` treated as `included` (the run
  fails after 223 s with the store copy mismatch); the table written for `excluded`; the
  key separator; non-`.rs` files read.
- **REFACTOR:** none needed.
- **Checked:** format on the workspace; lint on the two touched crates.
- **Not run:** the other 58 real-source tests, the workspace tests, the process audit.
- **Not built:** nothing reads the table yet. The acceptance runs still use `excluded`.
- **Open:** the run id does not hash the `core` source. It hashes the compiler version
  text and the scope value.
- **Acceptance:** the run of 2026-11-16 is no longer valid for this commit; production
  changed. A new one follows step 8.6.

## TDD entry — step 8.2a, a parameter's written type (REQ-RUST-055.0 R1, R3, R4)

```text
STUB   read_parameter_written_type(parameter, input, removed) -> Option<Value>, body todo!();
       new key receiver_written_type_evidence on the method receiver record; cargo check passes
RED    cargo test --release -p parseltongue-syntax-span-extractor test_parameter_written_type_rows
       1 selected: FAILED, "not yet implemented: REQ-RUST-055.0 R1"
       with the body returning None: FAILED at the first assertion,
       left Some("&[MaybeUninit<u8>]"), right None
GREEN  the type node's range mapped back to the original source, and its text
       test_parameter_written_type_rows ok; crate 142 passed, 0 failed (lib), 17 passed (other target)
       test_capture_generic_receiver_evidence compares the whole record, so it gained the new key
BREAKS rebinding ignored      -> FAILED (rb.len(): left None, right Some("&[u8]"))
       range starts one late  -> FAILED (left Some("&[MaybeUninit<u8>]"), right Some("[MaybeUninit<u8>]"))
GATES  cargo fmt --check and clippy --all-targets -D warnings pass on the crate
REFACTOR none needed: one function of five lines
NOT RUN the real-source itoa tests and the workspace tests
NEXT   8.2b: fields, statics and index expressions (R2)
```

## TDD entry — step 8.2b, the receiver as a place (REQ-RUST-055.0 R2, R3, R4)

```text
STUB   read_receiver_place_steps(receiver, call, input, removed) -> Option<Vec<Value>>, body todo!();
       new key receiver_place_step_rows on the method receiver record
RED    cargo test --release -p parseltongue-syntax-span-extractor test_field_and_static_receivers
       1 selected: FAILED, "not yet implemented: REQ-RUST-055.0 R2"
GREEN  steps, base first: parameter_identifier (with its written type), self_value, value_path
       (text and range), then field_access (name) and index_access (is the index a range).
       Any other expression, a local, a closure parameter, a rebound name: no rows.
       test_field_and_static_receivers ok; crate 143 passed, 0 failed
BREAKS rebinding ignored            -> FAILED (left None, right Some([parameter.., index_access:false]))
       range index not flagged      -> FAILED (index_access:true expected, false stored)
       any expression is a base     -> FAILED (make().f.len(): left None, right Some([self_value, field_access:f]))
GATES  cargo fmt --check and clippy --all-targets -D warnings pass on the crate
REFACTOR none
WEAK   the RED run failed on the stub only; no run with an empty result was made
NOT RUN the real-source itoa tests and the workspace tests
NEXT   8.3a: the probe over the declaration table
```

## TDD entry — step 8.3a, the lookup rule over the core table (REQ-RUST-056.0 P1 to P5; REQ-RUST-054.0 D6 to D9)

```text
STUB   receiver-type-probe-records.rs: WrittenTypeShapeValue, ScopeTraitMethodValue, CoreMemberProbeOutcome,
       parse_written_type_shape and probe_core_member_declaration with todo!() bodies
RED    cargo test --release -p parseltongue-syntax-span-extractor receiver_type_probe_tests
       5 selected, 0 passed, 5 failed: "not yet implemented: REQ-RUST-056.0 P1"
       cargo test ... sysroot_member_declaration_tests: 4 new tests failed on the missing keys
       (left (false, ["T"]), right (Null, Null); left ["struct S", ..], right [])
GREEN  the table gained: impl_generic_parameter_names, impl_bound_restriction_flag (D6);
       macro_self_parameter_flag, macro_owner_shape_kind (D7); rows for fn written in macro call
       arguments (D8); rows for type names: struct, enum, union, alias, use rename (D9)
       the probe: receiver, each dereference, array to slice; per step by value, &, &mut;
       inherent members first, then traits in scope; one fit names, anything undecided withholds
       receiver_type_probe_tests 5 passed; sysroot_member_declaration_tests 8 passed; crate 152 passed
BREAKS array not unsized to a slice -> FAILED test_array_unsize_probe_order
       scope traits ignored         -> FAILED test_scope_trait_blocks_target
       mutability ignored           -> FAILED three tests
       type name not checked        -> FAILED test_unproven_member_stays_withheld (Twice<T>::get named)
       impl bound ignored           -> FAILED test_unproven_member_stays_withheld (Cell<T>::get named)
       macro member ignored         -> FAILED test_unproven_member_stays_withheld
GATES  cargo fmt --check and clippy --all-targets -D warnings pass on the crate
REFACTOR none
WEAK   the probe tests were seen failing on the stub only
       a type or an inherent method that core declares through a macro with a metavariable
       name, beyond what the rows show, is an assumption, not a proof
       rustc uses impl-level bounds, not method-level where clauses, to select a candidate:
       stated from knowledge of rustc, not measured here
NOT RUN the real-source itoa tests (the declaration row count changes) and the workspace tests
NEXT   8.3b: connect the probe to the resolver; real-source test on pinned itoa
```

## TDD entry — step 8.3b-1 (i), type path ranges on a parameter's written type (REQ-RUST-055.0 R5)

```text
WHY    a stored path resolution is keyed by the path's own byte range; the lookup needs that key
       to show that itoa's MaybeUninit is core's
RED    cargo test --release -p parseltongue-syntax-span-extractor test_written_type_path_ranges
       0 passed, 1 failed: left Some(["m::Slot<u8>", "u8"]), right None
GREEN  read_parameter_written_type adds type_path_range_rows; crate 153 passed
BREAK  a path under an expression counted -> first not caught (the fixture's array length held no
       type path); fixture changed to size_of::<Inner>() and the break then fails the test
GATES  cargo fmt --check and clippy --all-targets -D warnings pass on the crate
REFACTOR none
NEXT   8.3b-1 (ii): the L2 lookup for parameter receivers with a plain written type (9 itoa sites)
```

## TDD entry — step 8.3b-1 (ii-a), prelude re-export rows in the core declaration table (REQ-RUST-054.0 D10)

```text
WHY    the lookup must know which core traits are in scope without an import; that is the set
       the prelude files re-export
RED    cargo test --release -p parseltongue-syntax-span-extractor test_prelude_use_leaf_rows
       0 passed, 1 failed: left [ExactSizeIterator, Iterator, TryInto, "* of super::v1", Future], right []
GREEN  build_prelude_leaf_row: one "prelude_use_leaf" row per use leaf in core/src/prelude/ files;
       a glob is a row named * that keeps its path; a use in a private inline module gives no row
       crate 154 passed
BREAK  private-module leaves counted -> test fails (right gains "* of crate", "Secret")
GATES  cargo fmt --check and clippy --all-targets -D warnings pass on the crate
REFACTOR none
NOT RUN the real-source tests (the row count of the table on pinned itoa is not re-measured)
NEXT   8.3b-1 (ii-b): core-member-target-records.rs, the lookup for a parameter receiver
```


## 2026-11-16 Part 8 step 8.3b-1(ii-b): first targets for method calls into core (GREEN)

```text
 phase      STUB -> RED -> GREEN, REQ-RUST-054.0 D11 D12, REQ-RUST-055.0, REQ-RUST-056.0
 result     pinned itoa, scope "declarations": 9 of the 133 calls have a target
              8  MaybeUninit<T>::write   receiver MaybeUninit<u8>, step 0 by mutable reference
              1  [T]::get_unchecked      receiver &[MaybeUninit<u8>], step 0 by value
            nodes 153 (151 + the two members), edge rows 682, calls without a target 124
            default scope "excluded": unchanged, 151 nodes, 682 rows, frozen graph still equal
 withheld   78 receiver_type_not_determined (field, static, call result, local receivers)
            46 receiver_type_name_unproven  (parameter typed "&mut Self::Buffer")
 new rows   use_rename keeps rename_source_name_value; extern_crate_rename is its own kind;
            macro_call_name_tokens: per file and macro name, the names written in its calls
            core declaration rows 6,528 (5,336 before)
 new kind   external name kind core_declared_member, dispatch kind core_inherent_member
 tests      test_parameter_receiver_core_member, test_rename_source_name_rows,
            test_macro_call_token_rows, test_macro_name_parameter_calls (each seen failing first),
            test_itoa_core_member_targets (real source), test_core_member_rows_published (amended)
 gates      extractor crate 158 + 17 pass; cargo fmt --check pass; workspace clippy -D warnings pass
            60 real-source itoa tests: 59 passed before the amendment of
            test_core_member_rows_published, which then passed alone; full rerun not repeated
 found      1 benches/bench.rs is stored but belongs to no mapped unit: only files with a
              stored span are read for names; an "extern crate" in another file withholds
            2 "extern crate self as core" was read as a rename of any trait
            3 macro impl_fold_via_try_fold takes its member's name from its caller; proven
              from every call of it in core that no call writes the probed name
 not done   deliberate breaks; workspace tests; release build; re-freeze; acceptance run
 next       receivers typed "&mut Self::Buffer" (46), then fields and statics (78 group)
```


## 2026-11-16 Part 8 step 8.3b-2a: the written type of an impl's associated type (GREEN)

```text
 phase      RED -> GREEN, REQ-RUST-055.0 R1, REQ-RUST-056.0 P1 (no STUB: one new field)
 change     an impl member row of a "type NAME = ..;" carries member_alias_type_evidence:
            the type's text, byte range and the range of each type path in it
            (read_written_type_evidence, the reader a parameter's written type already used)
 test       test_alias_member_type_evidence: first run failed in its own setup (wrong row key),
            not counted; second run failed on the missing field ("Buffer -"); passes after
 gates      extractor crate 159 + 17 pass; cargo fmt --check pass; workspace clippy pass;
            60 of 60 real-source itoa tests pass (also 60 of 60 on the commit before, 07b95dfbc)
 effect     no call gains a target yet; nothing reads the field
 next       8.3b-2b: a parameter typed "&mut Self::Buffer" takes its type from the one stored
            impl of the enclosing impl's self type that writes "type Buffer", when that impl's
            trait is the enclosing impl's trait or one of its supertraits
```


## 2026-11-16 Part 8 step 8.3b-2b: spec packet (no code)

```text
 phase      SPEC, REQ-RUST-057.0 A1 to A10 (new), part-8 file section 8
 why        the rule "follow Self::NAME only through the impl's trait and its supertraits"
            was not written down; step 8.3b-2a also had no STUB phase (one stored field)
 decisions  A5 supertrait reach; A6 same self-type target and text; A7 alias paths proven as a
            parameter's; reasons associated_type_impl_unproven, associated_type_trait_unproven
 rejected   reading "type Buffer" from any impl of the self type without A5 (an unread
            supertrait could declare the name); naming buf.len() (P5 stands)
 next       STUB: find_alias_receiver_type, check_trait_supertrait_reach
```


## 2026-11-16 Part 8 step 8.3b-2b: STUB

```text
 phase      STUB, REQ-RUST-057.0 A2 to A8
 added      core-member-target-records.rs      AliasReceiverTypeValue, find_alias_receiver_type  (todo!)
            original-call-resolution-records.rs BoundMethodLookupIndex::check_trait_supertrait_reach (todo!)
 not wired  neither is called yet, so production behaviour is unchanged
 command    cargo check --release -p parseltongue-syntax-span-extractor
 output        --> crates/parseltongue-syntax-span-extractor/src/core-member-target-records.rs:423:19
            warning: function `find_alias_receiver_type` is never used
               --> crates/parseltongue-syntax-span-extractor/src/core-member-target-records.rs:430:15
            warning: method `check_trait_supertrait_reach` is never used
                --> crates/parseltongue-syntax-span-extractor/src/original-call-resolution-records.rs:1079:19
            EXIT=0
 next       RED: test_alias_parameter_core_member (fixture), test_itoa_core_member_targets (real source)
```


## 2026-11-16 Part 8 step 8.3b-2b: RED

```text
 phase      RED, REQ-RUST-057.0 A2 to A9
 tests      test_alias_parameter_core_member (fixture, new)          FAILED
              command  cargo test --release -p parseltongue-syntax-span-extractor test_alias_parameter_core_member
              output   left: Extracted { .. "MaybeUninit<T>::write" }  right: Unresolved { reason: TraitSolverNeeded }
                       stored reason on the row: receiver_type_name_unproven
                       test result: FAILED. 0 passed; 1 failed; 159 filtered out
            test_itoa_core_member_targets (real source, one assertion added: a call is named
            through an associated type)                               FAILED
              command  cargo test --release -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests test_itoa_core_member_targets
              output   thread 'test_itoa_core_member_targets' (1168062) panicked at crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs:5955:5:
                       no call is named through an associated type
                       test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 59 filtered out; finished in 19.88s
                       error: test failed, to rerun pass `-p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`
                       EXIT=101
 fixture    read_core_fixture_rows(.., impls = true): stores a path lookup for each trait name,
            each u8 and each Buffer<..>, then stores the impl headers again over them
 cases      2 named (own impl; impl of a supertrait two bounds away), 4 withheld (A4, A5, A6, A7)
 next       GREEN: find_alias_receiver_type, check_trait_supertrait_reach, call from find_core_member_target
```


## 2026-11-16 19:53 IST STOPPED by user instruction: state of part 8 (method calls into core)

Work stopped in the middle of GREEN of step 8.3b-2b. Nothing after this entry was run.

```text
 WHAT IS PUSHED (branch research/tiny-rust-code-mapper, head 221b64ade)
   bbbe0d2f6  acceptance report: all four itoa SOP steps PASS on one frozen binary
              (151 nodes, 682 edge rows, 133 calls without a target; graph marked incomplete)
   91acad84c  part 8 design: syntax-only read of core (288 files, 0.27 s, 22 MB)
   3f2f20840  core declaration extractor
   bd35ace75  8.1   production stage, scope value "declarations"
   2e900024e  spec  REQ-RUST-055.0 (receiver type), REQ-RUST-056.0 (target)
   81e57e87b  8.2a  parameter receiver carries its written type
   06105de4d  8.2b  receiver recorded as a place (parameter, self, path, field, index)
   c701bb2e2  8.3a  lookup rule over the core table
   39d0ed696  8.3b-1(i)    type path ranges on a written type
   ffcdc6061  8.3b-1(ii-a) prelude re-export rows
   07b95dfbc  8.3b-1(ii-b) first targets: 9 calls named
   7ed552478  8.3b-2a      impl "type NAME = T" stores its written type
   221b64ade  spec  REQ-RUST-057.0 (the type behind Self::NAME)

 MEASURED ON THE PUSHED HEAD, pinned itoa, scope "declarations"
   nodes 153, edge rows 682
   calls with a target      73 of 197   (64 before part 8)
   calls without a target  124
   edge rows with a target 558 of 682   (549 before part 8)
   default scope: 151 nodes, 682 edge rows, equal to the frozen expected graph
   checks run on 7ed552478: 60 of 60 real-source itoa tests, 176 extractor tests, format,
   workspace lint

 NOT PUSHED: working tree, step 8.3b-2b (parameters typed "&mut Self::Buffer")
   phase reached   STUB done, RED done (both entries above), GREEN code written, GREEN not closed
   files           core-member-target-records.rs       find_alias_receiver_type, read_self_alias_parts,
                                                       find_core_member_target takes the bound index
                   original-call-resolution-records.rs BoundMethodLookupIndex::check_trait_supertrait_reach
                   core-member-target-tests.rs         read_core_fixture_rows, test_alias_parameter_core_member
                   itoa-source-acceptance-tests.rs     one added assertion in test_itoa_core_member_targets
   saved as        ~/phase7-work/83b2-b-stopped-working-tree.patch (git diff of crates/, not in the repository)

 RESULTS OF THE UNPUSHED CODE (each run once)
   cargo test --release -p parseltongue-syntax-span-extractor
       160 passed, 17 passed, 0 failed; test_alias_parameter_core_member passes
       (2 cases named, 4 cases withheld: clauses A4, A5, A6, A7)
   cargo test --release ... test_itoa_core_member_targets
       FAILED, as expected at this point: the test still lists the 9 old targets.
       the run names 17 calls: 16 MaybeUninit<T>::write and 1 [T]::get_unchecked,
       that is 8 more than the pushed head (81 of 197 calls, 566 of 682 edge rows)
   a temporary grouping of the calls that reached the new rule (removed from the test file again)
       8  write  named, alias text "[MaybeUninit <u8 >; Self ::MAX_STR_LEN ]"
       1  len    withheld, scope_trait_may_fit (a prelude trait also declares len; clause P5)

 NOT KNOWN
   - which 8 source sites the new targets are; none is checked against the source yet,
     so the count 17 is not fixed in the test
   - why only 9 calls reached the new rule when 46 calls were counted as typed
     "&mut Self::Buffer". Not investigated. One guess, unverified: text from a macro
     expansion is stored with spaces ("Self ::Buffer"), and the new rule matches "Self::" only.

 NOT DONE FOR 8.3b-2b
   - GREEN journal entry and commit
   - three deliberate breaks (drop the supertrait check, the self-type check, the path proof)
   - REFACTOR, lint, the 60 real-source tests, workspace tests, release build
   - SPEC section 8 matrix has no row for clause A6; the fixture test has one
   - codebase-memory index is from 10:34 UTC and does not hold the unpushed code

 NOT STARTED
   8.3b-3  receivers that are a field or a static (part of the 78 calls with no receiver type)
   8.4     try_into, expect, unwrap (26 calls; need return types from core)
   8.5     ilog10 (10 calls; written by a macro in core)
   8.6     re-freeze the expected graph and repeat the acceptance run
   The acceptance run of bbbe0d2f6 covers no commit after it: production changed.

 OPEN DEFECTS, unchanged
   - rows depend on the absolute source directory (cause found, not fixed)
   - a run with the whole sysroot read stops with a store copy mismatch (test_lang_items_in_sysroot)
   - the run id does not hash the core source, only the compiler version

 TO RESUME
   git apply ~/phase7-work/83b2-b-stopped-working-tree.patch if the working tree is lost;
   then find why 37 of the 46 calls do not reach find_alias_receiver_type, before fixing any count.
```


## 2026-11-16 19:56 IST Part 8 step 8.3b-2b: work in progress committed by user instruction

```text
 phase      GREEN not closed; the working tree of the stop entry above is committed as it was
 contents   find_alias_receiver_type, read_self_alias_parts, check_trait_supertrait_reach,
            read_core_fixture_rows, test_alias_parameter_core_member, one added assertion in
            test_itoa_core_member_targets
 no run     nothing was built or run after the stop entry; results are the ones recorded there
 FAILING    test_itoa_core_member_targets (real source): the run names 17 calls, the test lists 9.
            The 8 new targets are not checked against the source, so the list is not updated.
 passing    extractor crate, one run before the stop: 160 and 17 passed, 0 failed
 not run    format check, lint, the other 59 real-source tests, workspace tests, release build,
            deliberate breaks
 next       find why 37 of the 46 "&mut Self::Buffer" calls do not reach the new rule; check the
            8 new targets against the source; then close GREEN
```


## 2026-11-16 20:40 IST Documents brought in line with the code (documents only)

```text
 why        a review of the code against these documents (REVIEW-code-versus-phase-documents/)
            found six of them describing the state of 2026-11-14
 changed    AGENTS.md  0ac91465c   status only in this log; core targets decided; scope
                                   "declarations" named as not accepted; macro edge kinds
            PRD        640977f93   stage table, build order, DuckDB wording, part 8 and node
                                   fields under "Open"
            ARCH       4774ef207   eight edge kinds, five resolvers, step 6a, section 4
            SOP        a9dd33379   the PASS covers 822da80cf only; scope "declarations" is
                                   outside the SOP
            MAP        c4e701d96   re-measured at 37ea1755d
            SPEC index f584373fd   part 8 state, 60 tests, requirement counts
            BACKLOG    196d5d9b5   list A: four open decisions, the rest made
            LOG        this commit "Where we are" shows the accepted run and the head apart
 unchanged  all code, all tests, the frozen expected graph, every SOP rule and step
 not run    nothing was built or run: this machine has no ~/repos/rust-tools. Counts of
            files, lines, tests and requirement ids were taken with git and grep; run
            results are quoted from the entries above
 not fixed  part 8 changed the pipeline without the ARCH, SOP and AGENTS.md change that
            AGENTS.md asks for in the same commit. The documents now say what the code
            does and that no run with it is accepted; SOP Rule 2 and S1 are as they were
```


## 2026-11-16 20:48 IST Decided by the user: method calls into core (documents only)

```text
 rule used   "choose what is most explicit for a coding agent that reads the graph" (2026-11-15)
 decided  1  the run reads the declarations of core; they are an input like the pinned
             source: frozen by digest in SOP S1, covered by the run id
          2  Q1: the label is the impl's self type as core writes it ([T]::len, *mut T::cast)
          3  Q2: a call a prelude trait could also answer (5 buf.len() calls) is named
             [T]::len with that assumption stored on the row; never named without it
          4  order: expected targets of the 133 calls from source, freeze, then code
 written in  PRD (decisions in force), SOP (Rule 2, S1, S2, last section), ARCH (step 6a),
             AGENTS.md (Rule 2), SPEC part 8 section 5, BACKLOG list A: one commit
 not built   the S1 record of the compiler version and core digest; the run id over the
             core files; the expected targets; the assumption field for decision 3
 also        rust-agent-mega-01 installed under .agents/skills (bd9e43350); AGENTS.md asks
             for functional Rust wherever possible
 not run     nothing was built or run
```


## 2026-11-16 20:59 IST Part 8 step 9.2: the expected target of the 133 method calls, drafted

```text
 what        itoa-expected-core-member-targets.json, written by a new builder from the pinned
             source and the core source; status DRAFT_NOT_FROZEN; not merged into the graph
 inputs      itoa: the vendored copy in the test tree, sha256 equal to the pin
             core: rustc 1.99.1 (58cf8f9c3 2026-10-13), 288 files, 6,419,461 bytes,
                   listing sha256 39a4d53484c4df57cff47403663a190d610682430406f82458395e1b1ea40780
 rows        133 of 133 sites have a target label; 0 rows say no_target
   49  MaybeUninit<T>::write
   39  [T]::get_unchecked
   13  TryInto::try_into                 trait label; the blanket impl is cited
    8  Result<T, E>::expect              receiver type from the return type of try_into
    5  Result<T, E>::unwrap
    5  [T]::len                          assumption prelude_trait_impl_unrefuted
   10  u8::ilog10 .. i128::ilog10        assumption core_macro_body_declaration
    1  str::len    1  [T]::as_mut_ptr    1  *mut T::cast    1  Iterator::rev
 checked     every cited line of itoa and core holds the cited text; each declaration's owner
             is the nearest enclosing impl or trait; the core digest equals the shell command
 tests       test_itoa_expected_core_targets: 9 of 9 pass
             test_itoa_expected_graph_rules: 6 pass, 3 FAIL here: they need
             /home/ubuntu/repos/rust-tools/itoa, which this VM does not have
 not done    user review of the rows (9.3); freeze (9.4); the Rust test that compares a run
             with these rows (9.5); the run id over the core files; any production change
 not known   no compiler or second reader checked a label; the author of the rows is the
             author of the mapper
```


## 2026-11-16 21:05 IST Part 8 steps 9.3 and 9.4: self-review of the 133 rows, then frozen

```text
 asked       the user: review the rows as a coding agent would read them, then take the call
 found       1  the label did not say which crate; an agent sees `core::mem::MaybeUninit::uninit`
                and `MaybeUninit<T>::write` and cannot tell they are one crate
             2  a trait label (14 rows) did not say that the impl that runs is not named
             3  type fields held prose ("an integer literal (its type is inferred)")
             4  one type text was invented by me: `Result<T, <T as TryFrom<_>>::Error>`
             5  three claims were prose only: "core declares no write for a slice",
                "no len / get_unchecked / as_mut_ptr for an array", "the index is not a range"
             6  `core_macro_body_declaration` was listed as an assumption; it is a proven fact,
                so an agent would have read doubt into 10 rows that have none
 changed     1  target_crate_name on every row
             2  target_owner_kind = trait_declaration_impl_not_named
             3  type absent + receiver_type_source_kind = inferred_not_written (14 rows; a test
                allows it only for try_into and rev, where the name alone decides the target)
             4  the text core writes: `Result<T, Self::Error>`
             5  the file lists every `fn NAME` of core with its enclosing header; tests check
                the three claims against that list and against the receiver text
             6  declaration_form_kind = core_macro_body; assumptions now hold only the unproven:
                5 rows, all `buf.len()`
             also: compared_fields names what a stored row must equal (T8)
 call        frozen. A later change needs a source citation and a new freeze.
 result      133 rows, 133 with a target; itoa-expected-core-target-freeze.json written
             test_itoa_expected_core_targets: 11 of 11 pass
 not known   whether any label is wrong in a way the same author repeats in the mapper: no
             compiler and no second reader checked one. A compile check of the 11 label
             groups is possible on this VM; the PRD says "No oracle", so it was not run.
 next        9.5: the Rust test that compares a `declarations` run with these rows
```


## 2026-11-16 21:08 IST Compile check of the 133 expected core targets (one-off, user approved)

```text
 why         the rows and the mapper have one author; a misreading of Rust's lookup rules would
             repeat in both. The user allowed one check against the compiler ("ok go ahead").
 how         the pinned src/lib.rs, unchanged (sha256 e962338e...), compiled outside the repo:
             RUSTC_BOOTSTRAP=1 rustc --edition 2021 --crate-type lib --crate-name itoa \
               -Zmir-opt-level=0 --emit=mir src/lib.rs
             then, for each of the 11 method names, the callee each MIR call names was counted.
             rustc 1.99.1, target x86_64-unknown-linux-gnu, no features.
 result      133 of 133 agree with the frozen labels. No row changed.
   rows  label                    callee in the MIR
     49  MaybeUninit<T>::write    MaybeUninit::<u8>::write
     39  [T]::get_unchecked       slice::<impl [u8]>::get_unchecked 38, <impl [MaybeUninit<u8>]> 1
     13  TryInto::try_into        <i32 as TryInto<uN>>::try_into 8,
                                  <&mut [MaybeUninit<u8>] as TryInto<&mut [MaybeUninit<u8>; N]>> 5
      8  Result<T, E>::expect     Result::<uN, TryFromIntError>::expect
      5  Result<T, E>::unwrap     Result::<&mut [MaybeUninit<u8>; N], TryFromSliceError>::unwrap
      5  [T]::len                 slice::<impl [MaybeUninit<u8>]>::len
     10  u8::ilog10 .. i128       core::num::<impl u8>::ilog10 .. <impl i128>, one each
      1  str::len                 str::<impl str>::len
      1  [T]::as_mut_ptr          slice::<impl [MaybeUninit<u8>]>::as_mut_ptr
      1  *mut T::cast             core::ptr::mut_ptr::<impl *mut MaybeUninit<u8>>::cast
      1  Iterator::rev            <core::ops::Range<usize> as Iterator>::rev
 learned     the literal in `999.try_into()` is i32, as assumed.
             The 5 `buf.len()` calls are `[T]::len`. The assumption stays on those rows: it says
             what the mapper did not prove, not what is true.
             With default MIR optimisation those 5 calls do not appear (the compiler lowers a
             slice `len` call to a pointer-metadata read), hence -Zmir-opt-level=0.
 limits      matched by label and count, not site by site. One compiler, one target.
             RUSTC_BOOTSTRAP=1 turns on an unstable flag on the stable compiler; used for this
             check only, never by production or a gate.
 recorded    provenance of itoa-expected-core-member-targets.json; PRD "No oracle" names the
             exception; re-frozen (rows unchanged), 11 of 11 data-set tests pass
```


## 2026-11-16 21:55 IST Part 8 step 9.5: the real-source test compares a run with the frozen rows

```text
 what        test_itoa_core_member_targets no longer holds a list of its own. It runs the
             mapper with `declarations` on pinned itoa and compares every call it names in
             core with the frozen row of that site: label, crate, owner kind, declaration
             file and line, and the assumptions the row asks for (REQ-RUST-058.0 T8).
             It also fails when the core source of the compiler in use is not the one the
             rows were written from (T6), or the file differs from its freeze record (T7).
 result      stored 17, equal 17, wrong 0, without a target 116 (of 133)
               16  MaybeUninit<T>::write      1  [T]::get_unchecked
             nodes 153, edge rows 682, calls 197
 RED shown   with one expected line number raised by one, the test reported
             "17 of 17 stored core targets differ"; the change was reverted
 also        the expected rows gained site_span_key (copied from the frozen graph, the key
             the test joins on); re-frozen, rows otherwise unchanged; 11 data-set tests pass
             format_stored_site_span is now shared with test_itoa_whole_graph_equality
 gates       cargo fmt --check                                  ok
             clippy -D warnings, cli crate, all targets         ok   359 s
             real-source tests, frozen executable               60 passed, 0 failed, 1,177 s
             executable sha256 216d454cd70eb807..., same before and after the run
             not run: the workspace clippy, test and build gates
 this VM     ~/repos/rust-tools restored: rust-analyzer 2a362a4 plus the bundle in tools/
             (bf6b209), itoa af77385 (digests equal the pin). Release build 7 min 56 s.
             `cargo fetch --locked` was needed once so that `cargo metadata --offline` finds
             no-panic. The DuckDB json extension had to be downloaded (immediate backlog 6).
             One run with `declarations`: 16.5 s, 376 MB resident, one process, untraced.
 not done    no SOP step was run for this head. S1 to S4: NOT STARTED.
```


## 2026-11-16 22:28 IST Part 8 step 8.3b-2b closed: a type written by a macro expansion

```text
 cause       a macro expansion writes the parameter type as `&mut Self ::Buffer`, with a
             space. read_self_alias_parts matched the text `Self::` and missed it. Seen in
             the stored rows of a run: 37 calls carried receiver_written_type_text
             "&mut Self ::Buffer" and the reason receiver_type_name_unproven.
 RED         fixture in test_alias_parameter_core_member: the same impl with the type
             written `& mut Self :: Store` stayed without a target
 GREEN       read_self_alias_parts reads `Self`, `::` and the name with any white space
             between them; strip_reference_layer_prefix takes one `&` or `&mut` off
 real source test_itoa_core_member_targets, against the frozen expected rows
               before   stored 17   equal 17   wrong 0   without a target 116
               after    stored 50   equal 50   wrong 0   without a target  83
               49 MaybeUninit<T>::write (all of them), 1 [T]::get_unchecked
             the 4 buf.len() calls of impl_Unsigned moved to the reason scope_trait_may_fit,
             as the fifth one already was
             nodes 153, edge rows 682, calls 197; edge rows with a target 599 of 682
 gates       cargo fmt --check                                        ok
             clippy -D warnings, extractor and cli crates             ok
             extractor tests                                          177 passed, 0 failed
             real-source tests, frozen executable                     60 passed, 0 failed, 906 s
             executable sha256 2004bd6262fe63b1..., same before and after the run
             not run: the workspace clippy, test and build gates
 not done    S1 to S4 for this head: NOT STARTED. 83 calls into core have no target.
             The "Where we are" page above still shows 81 of 197; it is updated with the
             next step.
```


## 2026-11-16 22:56 IST Part 8: a receiver that is a `static`, then fields (REQ-RUST-059.0)

```text
 spec        SPEC part 8 section 10, written before the code: S1 to S6
 rule        a value path that the stored lookup resolved to one static of the workspace has
             the type the static declares; each field step takes the stored field of the one
             struct that type names (a tuple field by position, a record field by name)
 RED         test_static_field_receiver_member failed without the production change
 GREEN       7 fixture cases: tuple field, record field behind `&`, no field step, field then
             index, a field removed by cfg (not stored, takes no position), a stored field
             with an attribute (withheld), a tuple type (withheld), an unproven name (withheld)
 real source test_itoa_core_member_targets, against the frozen expected rows
               before   stored 50   equal 50   wrong 0   without a target 83
               after    stored 88   equal 88   wrong 0   without a target 45
               49 MaybeUninit<T>::write, 39 [T]::get_unchecked (all of both)
             nodes 153, edge rows 682, calls 197; edge rows with a target 637 of 682
 gates       cargo fmt --check                                        ok
             clippy -D warnings, extractor and cli crates             ok
             extractor tests                                          178 passed, 0 failed
             real-source tests, frozen executable                     60 passed, 0 failed, 956 s
             executable sha256 f7fc48388bdf2c49..., same before and after the run
             not run: the workspace clippy, test and build gates
 not built   a receiver rooted at `self` or at a parameter followed by a field
 not done    S1 to S4 for this head: NOT STARTED. 45 calls into core have no target.
```


## 2026-11-16 23:19 IST Part 8: a prelude trait, a sole trait method name, and `self` then a field

```text
 specs       SPEC part 8 sections 11 and 12 and S7 of section 10, each written before its code
 REQ-060.0   Q2 as decided: when no inherent member fits at a receiver form and only traits
             of core that are in scope through the prelude alone may fit, the probe goes on.
             A name found later carries the assumption prelude_trait_impl_unrefuted and the
             trait names. Nothing found later: scope_trait_may_fit, as before.
 REQ-061.0   a method name that exactly one trait of core declares, and nothing else in
             core or the workspace, names that trait's declaration whatever the receiver is.
             dispatch_kind core_trait_declared_member; assumption
             source_compiles_method_resolves.
 REQ-059.0   S7: `self` then a field starts from the self type of the stored impl.
 fixtures    test_scope_trait_blocks_target (4 new cases), test_parameter_receiver_core_member
             (2 new), test_sole_trait_method_name (6 cases), test_self_field_receiver_member
             (5 cases; failed without the production change)
 real source test_itoa_core_member_targets, against the frozen expected rows
               before   stored  88   equal  88   wrong 0   without a target 45
               after    stored 108   equal 108   wrong 0   without a target 25
               5 [T]::len (REQ-060.0), each with the assumption its expected row asks for;
               no other row carries that assumption
               13 TryInto::try_into, 1 Iterator::rev (REQ-061.0)
               1 [T]::as_mut_ptr (S7)
             nodes 157 (6 names of core members), edge rows 682, calls 197;
             edge rows with a target 657 of 682
 a gate run  failed first and was stopped: test_core_member_rows_published pinned the two
 that failed names a `declarations` run adds and 153 nodes. The run now adds six. The pins of
             both tests that use `declarations` were changed to the measured values, both
             tests were run alone and passed, then the gates below were run.
 gates       cargo fmt --check                                        ok
             clippy -D warnings, extractor and cli crates             ok
             extractor tests                                          180 passed, 0 failed
             real-source tests, frozen executable                     60 passed, 0 failed, 910 s
             executable sha256 7e742a5376defd69..., same before and after the run
             not run: the workspace clippy, test and build gates
 corrected   the times in the headings of the last three entries were wrong by up to an
             hour; they now show the commit times (21:55, 22:28, 22:56)
 not built   a receiver that is itself a method call (8 expect, 5 unwrap, 1 cast: needs the
             return type core declares), a local bound to a call's result (str::len), and
             members core writes inside a macro (10 ilog10)
 not done    S1 to S4 for this head: NOT STARTED. 25 calls into core have no target.
```

## 2026-11-17 08:27 IST Part 8: the inputs of REQ-RUST-062.0 (C1, C3, C4, C5); no new call named

```text
 spec        SPEC part 8 section 13 (REQ-RUST-062.0), pushed before this code (a7f952424)
 built       C3  a row of a core declaration now stores trait_generic_parameter_names,
                 trait_associated_type_names, owner_unsized_relaxed_names and
                 member_generic_parameter_names
             C4  parse_return_type_shape reads a return type such as Result<T, Self::Error>;
             C5  a generic parameter or associated type without ?Sized, standing as a
                 generic argument, is "a type not named, proven Sized" (SizedUnknown), which
                 fits a generic parameter of an impl and nothing else
             C1  the stored evidence of a method call whose receiver is a method call holds
                 receiver_inner_call_evidence (the inner call's name and receiver evidence)
             refactor: the probe tail of find_core_member_target is name_probed_core_member
 not built   C2 and C6: nothing reads the inner call's return type yet, and no fixture of a
             chained call exists. parse_return_type_shape is called by its unit test only.
 fixtures    test_return_type_sized_arguments (new), test_trait_member_row_fields and
             test_inherent_member_row_fields (extended)
 real source test_itoa_core_member_targets, against the frozen expected rows: unchanged
               stored 108   equal 108   wrong 0   without a target 25
             nodes 157, edge rows 682, calls 197; edge rows with a target 657 of 682
 gates       cargo fmt --check                                        ok
             clippy -D warnings, extractor and cli crates             ok
             extractor tests                                          181 passed, 0 failed
             real-source tests                                        60 passed, 0 failed, 910 s
             (whole test process: 15 m 22 s wall, 673 MB peak resident set of the largest
             process, not the whole tree)
             executable sha256 14c359199898271f..., read after the run
             not run: the workspace clippy, test and build gates
 queries     codebase-memory trace_path inbound, depth 1: build_owned_function_row (1 caller,
             list_owned_function_rows), extract_method_receiver_syntax (1,
             extract_original_call_syntax), find_core_member_target (1,
             build_original_call_edge). The index is older than these functions:
             parse_return_type_shape was not found. Re-indexed after the commit.
 next        C2 and C6 with a fixture of `999.try_into().expect("x")`, RED then GREEN, then
             the real-source comparison (predicted, not measured: 121 named, 12 without)
 not done    S1 to S4 for this head: NOT STARTED. 25 calls into core have no target.
```

## 2026-11-17 09:15 IST Part 8: REQ-RUST-062.0 C8 to C13, the 13 `expect` and `unwrap` calls are named; documents brought to this state

```text
 spec        SPEC part 8 section 13, C8 to C13 (565c57fb5), pushed before this code
 why         C2 and C6 as first written named none of the 13 calls on real source: try_into
             returns Result<T, Self::Error>, and core declares the name Result three times
             (the enum in result.rs, type aliases in fmt/mod.rs and io/error.rs). The fixture
             had one Result. The lookup refuses a type name declared more than once.
 built       C9   extract_member_declaration_rows stores one file_scope_name_binding row per
                  direct child of a core file that may bind a type name (item, module,
                  extern_crate, use_leaf, use_glob, unread_form), with its attribute texts
             C10  find_file_scope_type: the name is an item of the file that writes it, or
             C11  it comes from the prelude import of core/src/lib.rs, followed to the one
                  declaration; an alias, a glob, a macro call, a second binding, a cfg or a
                  #[path] leaves the name unproven
             C8   find_call_result_member reads a duplicated type name only with that proof
             C12  and only when every impl with the probed method name writes the same
                  declaration; otherwise receiver_type_name_unproven
             C13  the proof (files and offsets of each step) is stored on the edge row in
                  receiver_type_name_proofs
 defect      the first real-source run failed: UNIQUE constraint on the declaration rows.
             A file scope row and the declaration row of the same name had one stored key
             (file and name offset). build_declaration_row_key adds the kind to the key of a
             file scope row; a glob row sits at its `*`. Not caught by a fixture: the key is
             built in the cli crate, and only the real core source has such a pair.
 fixtures    test_file_scope_binding_rows, test_file_scope_type_proven,
             test_file_scope_type_unproven (new); the chained-call fixture now has three
             Result declarations and a call through the alias impl that stays unproven
 real source command: parseltongue map-workspace-full-run --workspace-root-path-value
             ~/phase8-work/run1/itoa --workspace-store-file-path <store.db>
             --worker-thread-count-value 2 --mapping-final-stage-kind
             full-item-dependency-graph --sysroot-source-scope-kind declarations
               run 1   exit 0   22.30 s wall   385,768 kB peak resident set
               run 2   exit 0   23.25 s wall   380,704 kB peak resident set
             (peak of the largest process, not the whole tree). The 682 exported edge rows
             of the two runs are equal after sorting. Executable sha256 058ee3ebe9801ab2...,
             read before and after run 2.
             against the frozen expected rows (test_itoa_core_member_targets):
               seen 133   named and equal 121   wrong 0   without a target 12
               new: 8 Result<T, E>::expect, 5 Result<T, E>::unwrap, proof kind prelude_import
               open: 10 ilog10 (receiver_type_not_determined), 1 len on a local &str
               (receiver_type_not_determined), 1 cast (inherent_member_unproven)
             nodes 159, edge rows 682, calls with a target 185 of 197, edge rows with a
             target 670 of 682
 gates       cargo fmt --check                                        ok
             clippy -D warnings, extractor and cli crates             ok
             extractor tests                                          168 and 17 passed, 0 failed
             real-source tests                                        60 passed, 0 failed, 923 s
             cli crate, all targets: 1 failed, test_lang_items_in_sysroot (the known open
             defect); every other test passed
             not run: the workspace clippy, test and build gates
 commits     e367248ff, 740db3a26, 8144218d0, 86350f3a5, eb031e680 (code and tests, pushed).
             8144218d0 is 309 lines (a new module and its test in one commit) and
             eb031e680 is 195; the others are under 200.
 documents   every file of docs/phase-20261113 was read against the code and this run.
             Changed: this log's "Where we are" page, SPEC README and part 8, PRD (two new
             decisions, the DuckDB json line, the open list), ARCH (step 6a row kinds, C2,
             the not-built table), SOP (the table of what the next run needs), BACKLOG, MAP
             (re-measured at eb031e680), immediate backlog entry 6. AGENTS.md: no change
             needed; it holds no numbers.
 queries     codebase-memory index_repository twice (27,262 nodes, 64,103 edges; last at
             03:40Z, after the code commits). trace_path inbound, depth 1:
             probe_core_member_declaration (4 callers), name_probed_core_member (it
             lists 1, find_core_member_target; the source has a second call, in
             find_call_result_member), extract_member_declaration_rows (the sysroot facade
             and test helpers), find_file_scope_type (4: two tests, check_impl_name_differs,
             prove_return_type_names), prove_return_type_names (1, find_call_result_member),
             list_file_scope_rows (1, extract_member_declaration_rows),
             build_declaration_row_key (1, collect_sysroot_member_rows). trace_path
             outbound: extract_member_declaration_rows, find_core_member_target. The
             callers were checked against the source with grep.
 next        the 10 ilog10 calls (spec first: a method written by a macro in core), then
             len on a local &str. cast stays without a target.
 not done    S1 to S4 for this head: NOT STARTED. The core files are not in the run id. The
             targets are not merged into the expected graph. 12 calls into core have no
             target.
```

### 2026-11-17 — part 8, REQ-RUST-063.0 built: `ilog10` on the ten integer types (131 of 133)

```text
 what        A method called on a const of a builtin type, both written into the type's impl
             by a macro of core: u8::MAX.ilog10() and the nine others.
             M1 to M3  new stored rows of the core read: impl_macro_call, macro_definition
                       (with arm count), const beside fn in a macro body, each body row
                       with arm, nesting depth, repetition, attributes, boundary flag and
                       the offset of its macro's name
             M4, M5    core-macro-impl-member-records.rs: find_macro_scope_definition,
                       find_macro_impl_member (pure, over the stored rows)
             M6, M7    find_builtin_constant_member, called from find_core_member_target
 found       three facts of the real core source the spec's first draft missed, each now
             in part 8 section 14: a second macro named uint_impl (num/imp/int_bits.rs);
             a doc comment is a token, not trivia; an impl_macro_call row can share file
             and offset with a member row (stored key now ends in the kind).
 tests       unit (extractor): test_macro_impl_scope_rows, test_macro_impl_member_proven,
             test_macro_impl_member_unproven, test_builtin_constant_receiver_member.
             The fixture test's first failure was a missing struct in the fixture, not the
             missing behaviour: RED for the intended reason was not observed for M6.
             real source: test_itoa_core_member_targets (pins 131, ten ilog10 labels,
             169 nodes, 682 edge rows, 197 calls, 2 without a target),
             test_core_member_rows_published (169 nodes, the ten labels added).
 run         parseltongue map-workspace-full-run --workspace-root-path-value
             ~/phase8-work/run1/itoa --workspace-store-file-path stepL.db
             --worker-thread-count-value 2 --mapping-final-stage-kind
             full-item-dependency-graph --sysroot-source-scope-kind declarations
               exit 0   22.83 s wall   385,340 kB peak resident set (largest process)
             One run; it was not repeated for equality.
             against the frozen expected rows (test_itoa_core_member_targets, and the
             scratch compare_run.py on the export):
               seen 133   named and equal 131   wrong 0   without a target 2
               new: ilog10 once on u8 u16 u32 u64 u128 i8 i16 i32 i64 i128
               open: 1 len on a local &str (receiver_type_not_determined), 1 cast
               (inherent_member_unproven)
             nodes 169, edge rows 682, calls with a target 195 of 197, edge rows with a
             target 680 of 682 (counted from the export)
 gates       cargo fmt --check                                        ok
             clippy -D warnings, extractor and cli crates             ok
             extractor tests                                          172 and 17 passed, 0 failed
             real-source tests                                        60 passed, 0 failed, 961 s
             Executable sha256 17fd1bbaf1faae37..., read after the tests.
             Run on the final tree only. The commits b602790f2 and 09be852fa compile but
             add functions that the next commit first calls, so clippy -D warnings would
             not pass at those two.
             not run: the workspace clippy, test and build gates; the cli crate's other
             test targets
 commits     b602790f2, b97b67156, 960fc5577, 09be852fa, 61c05b95c, 8b14e481c, 79326d155
             (code and tests). 09be852fa is 218 changed lines; the others are under 200.
 documents   part 8 (status, M6 wording, verification matrix, measured section), SPEC
             README, PRD open list, ARCH (seven row kinds, the new lookup, the not-built
             table), SOP table, MAP (re-measured at 79326d155), this log's "Where we are".
 queries     codebase-memory index_repository on the working tree, then trace_path
             inbound, depth 1: find_core_member_target (2: build_original_call_edge,
             find_call_result_member), find_builtin_constant_member (1),
             find_macro_impl_member (3: find_builtin_constant_member and two tests),
             find_macro_scope_definition (1), list_impl_macro_rows (1),
             read_item_attribute_texts (1), read_macro_body_position (1),
             extract_member_declaration_rows (18), build_declaration_row_key (1),
             count_macro_rule_arms (it lists 0; the source has one call, in
             sysroot-member-declaration-records.rs). Callers checked with grep.
 next        len on a local &str. cast stays without a target. Then the core files in the
             run id, the merge of the targets into the expected graph, its new freeze,
             and S1 to S4.
 not done    S1 to S4 for this head: NOT STARTED. The core files are not in the run id. The
             targets are not merged into the expected graph. 2 calls into core have no
             target.
```

### 2026-11-17 — part 8, REQ-RUST-064.0 built: `len` on a local `&str` (132 of 133)

```text
 what        A method called on a local variable that one plain `let` binds to a method
             call: let string = i.write(..); string.len().
             L1        local-binding-receiver-records.rs: find_local_binding_statement,
                       stored as receiver_local_binding_evidence (commit fb94f7ff9)
             L3        list_bound_method_candidates, taken out of
                       resolve_bound_method_target
             L2 to L7  find_local_binding_type, called from find_core_member_target; the
                       index keeps the RET_TYPE rows and a count of each identifier of
                       the unit's source files
 found       two facts the spec's first draft missed, both now in part 8 section 15:
             L2 counted names in all stored text and must count the source files of the
             unit only; private::Sealed has the supertrait Copy, whose source the run does
             not read, so L3 accepts the walk only for a method that takes self by value
             and stores the assumption source_compiles_method_resolves.
 tests       unit (extractor): test_local_binding_receiver_evidence (L1),
             test_local_binding_receiver_member (L2 to L7, nine cases). Its first failure
             was a fixture without trait-contains-method edges, not the missing behaviour:
             RED for the intended reason was not observed.
             real source: test_itoa_core_member_targets (pins 132, str::len label, 170
             nodes, 682 edge rows, 197 calls, 1 without a target),
             test_core_member_rows_published (170 nodes, str::len added).
 run         parseltongue map-workspace-full-run --workspace-root-path-value
             ~/phase8-work/run1/itoa --workspace-store-file-path stepM.db
             --worker-thread-count-value 2 --mapping-final-stage-kind
             full-item-dependency-graph --sysroot-source-scope-kind declarations
               exit 0   22.81 s wall   377,608 kB peak resident set (largest process)
             One run; it was not repeated for equality.
             against the frozen expected rows (test_itoa_core_member_targets, and the
             scratch compare_run.py on the export):
               seen 133   named and equal 132   wrong 0   without a target 1
               new: str::len (core/src/str/mod.rs), proof kind local_binding_call_result
               open: 1 cast (inherent_member_unproven)
             nodes 170, edge rows 682, calls 197, calls without a target 1
 gates       cargo fmt --check                                        ok
             clippy -D warnings, extractor and cli crates             ok
             extractor tests                                          174 and 17 passed, 0 failed
             real-source tests                                        59 passed, 1 failed, 893 s
               failed: test_core_member_rows_published, its list of added labels lacked
               str::len. The list was corrected (test only) and the test passed alone
               (38 s). The 60 were not run again together.
             not run: the workspace clippy, test and build gates; the cli crate's other
             test targets
 commits     fb94f7ff9, 903738163, 317942e17, baee92849, 3f6ebad08 (code and tests).
             baee92849 is 203 changed lines; the others are under 200. 903738163 and
             317942e17 were not gated alone.
 documents   part 8 (status, section 15 L2 and L3, measured section; section 16 is the
             spec of the next step), SPEC README, part 7 Q4 note, PRD open list, ARCH
             not-built table, SOP table, this log's "Where we are". MAP not re-measured.
 queries     codebase-memory trace_path inbound, depth 1, before the change:
             scan_source_scope_tokens (read_core_member_index), read_core_member_index
             (resolve_original_call_edges), resolve_bound_method_target
             (build_original_call_edge), find_core_member_target (build_original_call_edge,
             find_call_result_member), list_core_fixture_edges (read_core_fixture_rows,
             test_call_result_receiver_member). index_repository after the commits, then
             find_local_binding_type (find_core_member_target),
             list_bound_method_candidates (resolve_bound_method_target,
             find_local_binding_type), find_local_binding_statement
             (extract_method_receiver_syntax), check_plain_function_text (it lists 0; the
             source has one call, in find_local_binding_type). For the next step:
             collect_sysroot_member_rows (persist_sysroot_member_rows and one test),
             persist_sysroot_member_rows and compute_workspace_generation_digest
             (run_workspace_mapping_pipeline each).
 next        REQ-RUST-065.0, the core files in the run id. Then the merge of the targets
             into the expected graph, its new freeze, and S1 to S4.
 not done    S1 to S4 for this head: NOT STARTED. The core files are not in the run id. The
             targets are not merged into the expected graph. 1 call into core has no
             target (cast).
```

### 2026-11-17 — part 8, REQ-RUST-065.0 built: the `core` files are in the run id

```text
 what        A run with scope declarations stores which core files it read, and that
             record is part of the run id.
             D1, D4, D5  sysroot-member-declaration-facade.rs: read_sysroot_source_files
                         reads every .rs file under core/src once, in byte order of the
                         relative path; build_source_listing_record hashes those bytes
                         (the S1 listing); collect_sysroot_member_rows parses the same
                         bytes and returns the record with the rows;
                         persist_sysroot_member_rows fails before any row is written when
                         the record differs from the one in the workspace row
                         (check_sysroot_source_record). Commit 6755ab0a9.
             D1 to D3    main.rs: with scope declarations the workspace row gets
                         workspace_sysroot_source_record before the run id is computed.
                         A run with excluded has no such field. Commit 8787f0223.
 tests       unit (cli): test_core_source_listing_record. RED: the stub record was Null
             and the check returned Ok. GREEN: recursion, .rs only, order, byte count, the
             digest computed by hand, changed bytes, a record that differs, a missing
             directory.
             real source: test_core_member_rows_published also asserts that the stored
             record equals core_source of the frozen targets file, that a run with
             excluded stores none, and that the two run ids differ.
 measured    stored record of a declarations run on pinned itoa:
               core/src   288 files   6,419,461 bytes
               listing sha256 39a4d53484c4df57cff47403663a190d610682430406f82458395e1b1ea40780
             equal to the S1 command's digest and to the frozen targets file.
 gates       cargo fmt --check                                        ok
             clippy -D warnings, cli crate                            ok
             cli tests (bins and three contract test targets)         ok
             real-source tests   60 passed, 0 failed, 893.82 s; wall 14:54.01,
                                 390,288 kB largest process
             not run: the workspace gates.
 commits     6755ab0a9 (152 added, 24 removed), 8787f0223 (45 added). Documents e3a0101b1.
 queries     codebase-memory trace_path inbound, depth 1: collect_sysroot_member_rows
             (test_core_directory_member_rows, persist_sysroot_member_rows),
             persist_sysroot_member_rows (run_workspace_mapping_pipeline),
             compute_workspace_generation_digest (run_workspace_mapping_pipeline).
             index_repository after the commits: 27,629 nodes, 64,952 edges.
 next        REQ-RUST-066.0: merge the frozen targets into an expected graph of a
             declarations run.
```

### 2026-11-17 — part 8, REQ-RUST-066.0: the expected graph of a `declarations` run, frozen

```text
 what        A third builder reads the frozen expected graph and the frozen targets file
             and writes the expected graph of a declarations run. The two inputs are not
             changed, so the expected graph of the accepted run (scope excluded) stays as
             frozen on 2026-11-16.
             M1 to M7  itoa-target-dataset/itoa-expected-declarations-graph-builder.py and
                       itoa-expected-declarations-graph-{nodes,edges,summary,freeze}.json.
                       Commit 5f43747e4 (36,527 added lines, nearly all generated JSON;
                       over the 200-line rule for that reason).
             M8        test_itoa_declarations_graph_equality; the comparison of
                       test_itoa_whole_graph_equality moved into
                       verify_stored_graph_equality, which takes the names of the expected
                       files. Test file only. Commit 753665eb9.
 decided     cast stays without a target in this graph (REQ-RUST-062.0 C7). The row keeps
             the record undetermined/inherent_member_unproven and names the known label
             *mut T::cast beside it. It is the one accepted row without a target
             (immediate backlog, entry 7). Mine, not asked of the user: it is what the
             user was told on 2026-11-17 ("cast stays without a target by decision").
 measured    builder output:
               nodes 170 (151 + 19 names declared in core)   grouped edge rows 473
               edge sites 682   merged from the targets file 132   without a target 1
               records: 118 core_inherent_member, 14 core_trait_declared_member
             digests: nodes f597b26379ba575119c885134ffe32fe50d770c33317be10dbaa856ab616b5df
                      edges c40f8b0c581a9f84a13d1d883ae37b9ed196921e70a3687e17c853d26e001bb3
 tests       dataset: python3 -m unittest discover -s docs/phase-20261113/itoa-target-dataset
             -p 'test_*.py': Ran 25 tests, OK (5 new: the rebuild is byte-equal; input
             digests; a missing, an unused and a doubled targets row each fail; record
             counts; added nodes).
             real source: cargo test --release -p parseltongue-cli-dispatcher-binary --test
             itoa-source-acceptance-tests -- --test-threads=1 graph_equality:
               test_itoa_declarations_graph_equality ... ok
               test_itoa_whole_graph_equality ... ok
               2 passed; 0 failed; 59 filtered out; 39.88 s
             No RED run for M8: the expected files and the production code both existed
             before the test. In its place, one deliberate break: the new test pointed at
             the excluded-scope files fails with "nodes expected 151 stored 170", 19
             differing nodes, 266 differing edge rows. Reverted.
 gates       cargo fmt --check                                        ok
             clippy -D warnings, cli crate                            ok
             cli tests (bins and three contract test targets)         ok
               test_lang_items_in_sysroot skipped: in the first try it ran for more than
               60 s and I stopped it (the known open defect)
             real-source tests   61 passed, 0 failed, 930.40 s; wall 15:38.45,
                                 456,048 kB largest process
 commits     e3a0101b1 (spec), 5f43747e4 (builder, files, dataset tests), 753665eb9
             (Rust test), efb5bbfb0 (PRD, SOP, immediate backlog).
 queries     codebase-memory trace_path inbound on verify_frozen_expected_files and
             collect_expected_graph_rows listed 0 callers; the source has them
             (test_itoa_whole_graph_equality, verify_stored_graph_equality), found with
             grep. index_repository after 753665eb9, then search_graph for
             run_scoped_graph_fixture, verify_stored_graph_equality and
             test_itoa_declarations_graph_equality: all three found.
 limit       The expected graph of a declarations run has one author, like the two files
             it is built from. The targets file was checked once against compiler output
             by label and count, not site by site.
 next        The workspace gates, then S1 to S4 on one frozen executable.
```

### Acceptance run report — pinned itoa, scope `declarations`, frozen executable (2026-11-17)

```text
source      itoa af77385d0daf4d0e949e81f2588be2e44f69f086, clean
            src/lib.rs       e962338e1886873aa7c3399ec89aa5378ccbfed712b9690f043628079390a34f
            src/u128_ext.rs  498cc94f13805e551248fd82f47cc318f51d953a5002a9f8698858cd12fbae3c
            Cargo.toml 44ca1ae5…dec0483, Cargo.lock 965fa758…a8d019d, benches/bench.rs 29a0bdb9…498b154
compiler    rustc 1.99.1 (58cf8f9c3 2026-10-13), host x86_64-unknown-linux-gnu
core        core/src: 288 .rs files, 6,419,461 bytes
            listing sha256 39a4d53484c4df57cff47403663a190d610682430406f82458395e1b1ea40780
executable  af84bb460f6d73cf0fb465117cfad671f94fab339497c72a0efc6872771cf354 (104,047,000 bytes),
            built by cargo build --release --workspace --all-targets --all-features;
            same digest after S3
expected    declarations graph
              nodes f597b26379ba575119c885134ffe32fe50d770c33317be10dbaa856ab616b5df
              edges c40f8b0c581a9f84a13d1d883ae37b9ed196921e70a3687e17c853d26e001bb3
            built from  graph nodes 29aed6ef…f7954e7, graph edges a51f7d6e…d06b156,
                        core targets acf9a04a47c60a0f0dcfdd9e22e7ae9bd7999819dd5d8ed5466ae3e1dbda7b9c
commits     production code of 8787f0223, the last commit that changes it. 753665eb9
            changes the test file only; efb5bbfb0 changes three documents only.
            Format and lint ran with 753665eb9 checked out. efb5bbfb0 was committed one
            second after lint ended, so the build and the four steps ran with it checked
            out. The crates are equal in both.
```

```text
S1  PASS  source commit and digests equal the pin; rustc -vV and the core digest equal the
          frozen targets file; the three builders reproduce the frozen files byte for byte
          (0 of 12 JSON files differ); 25 expected-data tests pass
S2  PASS  run publishes; completeness = incomplete (recorded, Rule 3); the stored
          workspace_sysroot_source_record equals the S1 core record
          untraced, 4 workers: 23.93 s, 398.1 MB peak, 2,054 samples
          strace audit: auditor pass, no violation rows; 0 connect calls; the only file
          opened under a target/ directory is .rustc_info.json; of the sysroot source,
          the 288 .rs files under core/src and its 55 directories are opened, nothing else
S3  PASS  61 passed, 0 failed, one thread, 928.22 s (wall 15:28.42), on the frozen
          executable. 170 nodes and 682 edge rows equal the frozen expected graph of a
          declarations run (test_itoa_declarations_graph_equality); 132 of 133 calls into
          core named and equal (test_itoa_core_member_targets); 1 call without a target
          (cast), an expected row. The excluded-scope graph still equals its own frozen
          graph (test_itoa_whole_graph_equality).
S4  PASS  six runs, node and edge rows byte-identical after the header line, and
          identical to the S2 store
          nodes body aadbd84f2d2a79a0e36dc576be4146e943e730976f55204c4c7e015055307c08
          edges body 3e637bdd01a9d635207c475f04a6a78d881f3ee9017a4a959add379d7c27c1af
```

```text
workers  wall s  peak MB  store bytes  nodes  edges
1        22.66   389.0    36,831,232   170    682
1 again  23.35   363.1    36,831,232   170    682
4        22.47   391.9    36,831,232   170    682
4 again  23.75   387.0    36,831,232   170    682
16       26.45   403.6    36,831,232   170    682
16 again 26.62   397.6    36,831,232   170    682
```

- **Workspace gates at `753665eb9`:** format, lint (all targets, all features) and release
  build pass. Tests, with `test_lang_items_in_sysroot` skipped (the known open defect):
  the first run ended with exit 101 after 20 test binaries, 639 passed, 2 failed,
  4 ignored. The two are `test_no_forbidden_semantic_dependencies` and
  `test_parser_source_pin_aligned` (`workspace-dependency-build-gates.rs`). Both run
  `cargo metadata --offline --locked`, which failed with "failed to download
  `android_system_properties v0.1.6`": the crate cache of this machine held only the
  crates of this platform. After `cargo fetch --locked` the same command exits 0. No file
  of the repository changed. The rerun of all 21 test binaries (`--no-fail-fast`,
  `~/phase8-work/accept/ws-test2.log`) ended with exit 0: 643 passed, 0 failed, 4 ignored,
  wall 17:04, maximum resident set 388,832 kB. It ran at `efb5bbfb0`, a documents-only
  commit after `753665eb9`, with no change under `crates/`. After it,
  `target/release/parseltongue` still has the digest of the frozen executable.
- **Open, found by the audit:** the run opens
  `~/.duckdb/extensions/v2.0.0/linux_amd64/json.duckdb_extension` (sha256
  `dd65944f1b176ec293fc212eb108218947712df14edd0a436dacb2f9c6bf57d1`). It is not one of
  the frozen inputs of SOP Rule 2 and S1 does not record it. The auditor does not check
  it and passes. I downloaded this file on this machine on 2026-11-16; without it the run
  fails.
  Immediate backlog, entry 6.
- **Deviations from the SOP text:**
  - The run maps a copy of the pinned source (`~/phase8-work/run1/itoa`, byte-equal to the
    pinned checkout for the four source files) with the pinned `Cargo.lock` added, as in
    the run of 2026-11-16.
  - S3's tests map their own copy of the pinned source in a temporary directory, not the
    S2 store. Rows depend on the source directory, so the S2 export is not byte-compared
    with the test's rows; both are compared by label.
  - The SOP says the freeze record of the expected graph names the compiler version and
    the `core` digest. The freeze record names the targets file by digest, and that file
    holds both values.
  - Order: S1, S2, S4, then S3, then the workspace tests.
- **Limits:** the graph is marked incomplete (1 call, `cast`); the two expected graphs and
  the targets file have one author; the targets were checked once against compiler output
  by label and count, not site by site; only itoa was run. The lookup evidence kept on a
  row is compared only as far as the assumptions the expected row asks for.
- **Commands:** `~/phase8-work/accept/run-all.sh`, which calls `run-s1.sh`, `run-s2.sh`
  and `run-s4.sh` (not in the repository). They run the SOP commands with
  `--sysroot-source-scope-kind declarations`.
- **codebase-memory:** index_repository after `753665eb9`; no query was needed for the run.

## 2026-11-17 13:25 IST Part 9: the exported graph as a DuckDB file, built, and the full run

**The full run the user set as the bar for this part passed. Pinned itoa 1.0.18 went
through the frozen `parseltongue` executable, the node and edge tables were exported, and
`parseltongue-graph-dataset-loader` wrote a DuckDB file whose rows equal the exported rows:
170 of 170 node rows and 682 of 682 edge rows, every column, 0 differences.**

- **Commits:** `0aa9b2de8` (crate, error enum), `1bd385e0a` (parser), `317f9bb82`
  (checks), `df44a54b7` (DuckDB write and read-back), `4125982e9` (unit tests),
  `c21eacd2b` (binary, real-source test, two binary tests). No file of another crate
  changed; `Cargo.toml` gained one member and `Cargo.lock` one package.
- **The full run** (`~/phase8-work/e2e/run-e2e.sh`, output `e2e.txt`, at `c21eacd2b`; no
  file under `crates/` differs from the commit). It ran twice. The numbers below are the
  second run, made with the two executables the workspace build left. The first run used
  a loader built with `-p` alone (sha256 `62f96d7f…`: without the `unbounded_depth` feature
  of `serde_json` that another crate of the workspace turns on) and gave the same result:
  mapper 22.88 s and 353.3 MB, loader 1.00 s, 0 failed checks.

```text
inputs    itoa 1.0.18, a copy of the four pinned source files (byte-equal to commit af77385d0)
          with the pinned Cargo.lock; rustc 1.99.1 (58cf8f9c3 2026-10-13)
          mapper sha256 af84bb460f6d73cf0fb465117cfad671f94fab339497c72a0efc6872771cf354 (the frozen executable)
          loader sha256 67743f0c79752b517ec25d0dc300c2979fd980f43f254b316f34f536e2e758fc
1 mapper  map-workspace-full-run, 4 workers, full-item-dependency-graph, scope declarations
          exit 0, 22.00 s, peak 354.8 MB (process tree)
2 export  export-published-stage-rows for both tables, exit 0: 170 node rows, 682 edge rows
          nodes sha256 f3c56289bb8f6b20a26fde9a99ffdfd5e110bc4cc0409843add9a52a4cbbd863
          edges sha256 409820741e7e58fe629a117d504647770e6f71a5a57651a957dd589662e0d41d
          row lines equal those of the accepted run (S2 and S4): node rows aadbd84f…, edge
          rows 3e637bdd…; only the header's stage attempt value differs
3 loader  exit 0, 0.87 s, max RSS 87,852 kB; full-graph-dataset.duckdb, 7,876,608 bytes
4 again   the same output path: exit 1 "the output path exists", file digest unchanged
5 traced  a load to a second path under strace: exit 0, 0 connect calls, 1 execve (itself),
          0 child processes, 0 opens with `.duckdb/` in the path, 0 opens under the home
          directory outside the run directory, the two inputs never opened for writing
6 inputs  both export files have the same sha256 after the three loads
7 check   check-dataset-rows.py (Python duckdb 2.0.0, not the loader's code), both files:
          three tables; info row = generation, 170, 682, the two input digests;
          170 node rows and 682 edge rows equal the JSON in all 11 columns;
          each payload text parses back to the exported payload;
          682 edges join to a source node, 681 to a target node,
          1 without a target (Unresolved, TraitSolverNeeded);
          the two loads store equal rows; failed checks 0
          rows by relation: Calls 197, UsesType 203, Reads 98, Contains 88, Generates 45,
          Implements 32, Expands 15, Imports 4
```

- **Tests of the crate** (`cargo test --release -p parseltongue-graph-dataset-loader
  --all-targets --all-features`): 14 pass, 0 fail. Unit: `test_canonical_json_text_sorted`,
  `test_export_header_line_required`, `test_run_generation_rows_agree`,
  `test_node_identity_text_checked`, `test_duplicate_row_identity_rejected`,
  `test_edge_source_node_required`, `test_edge_target_node_required`,
  `test_export_file_rows_required`, `test_small_export_dataset_roundtrip`,
  `test_failed_check_writes_nothing`, `test_existing_output_path_kept`. Binary:
  `test_wrong_flag_usage_exit`, `test_missing_export_file_exit`. Real source:
  `test_itoa_export_dataset_roundtrip` (23.8 s; it runs the mapper, both exports and the
  loader, and compares every stored row with rows it parses from the JSON itself).
- **RED, then GREEN:** with `parse_graph_export_text` and `write_graph_dataset_file`
  stubbed, the 11 unit tests gave 1 pass, 10 fail, each on the stub's error; with the
  bodies in, 11 pass. **Not test first:** the real-source test and the two binary tests
  were written after the code and passed on their first run, so they were never seen to
  fail.
- **Gates:**
  - Crate gates at `c21eacd2b`: `cargo fmt --all -- --check` exit 0; `cargo clippy
    --release -p parseltongue-graph-dataset-loader --all-targets --all-features -- -D
    warnings` exit 0; the crate's tests as above.
  - Workspace gates at `c21eacd2b` (`~/phase8-work/e2e/gates.sh`): format exit 0; clippy
    `--workspace --all-targets --all-features -- -D warnings` exit 0; tests exit 0, 25
    test binaries, 657 pass, 0 fail, 4 ignored, 1,131 s, max RSS 674,252 kB; build exit 0.
    The 61 real-source itoa tests are among them: 61 pass (966 s).
  - `test_lang_items_in_sysroot` is skipped, so the workspace gate is not a full PASS.
  - The mapper built by the workspace build has the sha256 of the frozen executable, so
    the acceptance run of `753665eb9` stands.
- **Deviations:**
  - Four of the six commits are over the 200-line rule (310, 287, 509 and 415 lines); each
    is one file or one test file with its wiring.
  - The row check of step 7 uses the PyPI package `duckdb` 2.0.0 in a throwaway virtual
    environment (`~/phase8-work/e2e-venv`). It is not a repository tool and not a gate.
  - The mapper run of step 1 still loads DuckDB's `json` extension from
    `~/.duckdb/extensions` (immediate backlog, entry 6). The loader does not.
- **Limits:** only itoa, only scope `declarations`. The DuckDB file is a copy of what the
  run exported: it adds no fact, and it inherits `incomplete` and the one row without a
  target. No acceptance step of the SOP reads it.
- **codebase-memory** (run after `c21eacd2b`, not before each commit as Rule 4 asks):
  `index_repository`; `trace_path` inbound, depth 1, on `load_graph_export_files`,
  `check_graph_export_rows`, `write_graph_dataset_file`, `read_graph_dataset_file` and
  `parse_graph_export_text`. Every caller it names is in the new crate, and no function of
  another crate was changed. Checked against source: it misses two callers, the test
  `test_failed_check_writes_nothing` of `load_graph_export_files` and the real-source test
  of `read_graph_dataset_file`.
- **Next:** nothing is in progress. The open decisions are listed under "What is left".

## 2026-11-17 16:15 IST Rule change: a commit may be up to about 1,000 changed lines

**The limit on one commit is raised from about 200 to about 1,000 changed lines (user
decision, 2026-11-17).** `AGENTS.md` Rule 3 and its rule for document commits now say so.

- **Why:** a new file with its tests rarely fits in 200 lines. Four of the six loader
  commits were over it (287 to 509 lines), and so were commits of part 8.
- **Unchanged:** one focused change per commit, pushed after each; a delete-only commit
  may be larger.
- **Not edited:** the plans of parts 3, 4, 4b, 4c and 4d name the old limit. They record
  how those closed parts were built.
- No code changed, so no gate was run and no codebase-memory query was needed.

## 2026-11-17 Part 10 built: the mapper needs no DuckDB extension file

**A run of pinned itoa with an empty `HOME` publishes the graph, and a traced run opens no
path with `.duckdb/`.** REQ-RUST-068.0 A1 to A4 pass. S1 to S4: NOT STARTED for this head.

- **Change:** the adapter's `duckdb` dependency gains the feature `json`; `LOAD json` is
  removed from `open_source_projection_store` (commit `dd213b8f9`). Test: commit
  `6cf6ea851`.
- **RED:** `test_empty_home_graph_published`, 0 passed, 1 failed, with DuckDB's
  "Extension .../empty-home/.duckdb/extensions/v2.0.0/linux_amd64/json.duckdb_extension
  not found".
- **GREEN:** the same test, 1 passed, 22.0 s.
- **A3:** `strace -f -e trace=openat,open,connect,execve` of
  `parseltongue map-workspace-full-run ... --worker-thread-count-value 4
  --sysroot-source-scope-kind declarations` with `~/.duckdb` moved away: exit 0, 0 opens
  with `.duckdb/`, 0 network connects, children `cargo` and `rustc` only, 31.4 s under
  strace, 381 MB. Exported rows (170 nodes, 682 edge rows) equal the accepted run's export.
- **Gates:** formatting exit 0; clippy on the two crates exit 0; adapter tests 187 passed;
  real-source tests 62 passed, 0 failed, 931 s. Workspace gates not run (after part 15).
- **Cost:** DuckDB is compiled once more, 18 min 30 s at two jobs.
- **Order of the open points changed:** part 12 (module to file area) is built before
  part 11 (node fields), because reachability of an item in `src/u128_ext.rs` needs the
  link from the module to its file. Part 12 is specified (commit `a170cb130`).
- **codebase-memory:** `index_repository`; `trace_path open_source_projection_store`
  (inbound: `publish_full_graph_checkpoint`, `publish_original_source_hld`, projection
  tests); `trace_path run_itoa_mapping_command`; `search_graph` for the test helpers;
  `trace_path derive_original_graph_input` (both, depth 2) for part 12.
- **Next:** part 12: the two rows added to the expected graph, then a failing real-source
  test, then the projection.

## 2026-11-17 Part 12 built: a module contains the file area of its file

**The crate root and `u128_ext` each have a `contains` row to the file area of their file,
and a run equals both re-frozen expected graphs: 170 nodes, 684 edge rows (scope
`declarations`).** REQ-RUST-070.0 A1 to A6 pass. S1 to S4: NOT STARTED for this head.

- **Rows:** `itoa` to `area:src/lib.rs`, sited at the whole file; `u128_ext` to
  `area:src/u128_ext.rs`, sited at `mod u128_ext;` (`src/lib.rs`, bytes 1980 to 1993).
  The inline module `private` has no such row.
- **Order:** expected rows and re-freeze `d52a528a2`; failing test `aafaeb186`; code
  `b32b47200`.
- **RED:** `test_module_file_area_rows`, 0 passed, 1 failed (expected two rows, stored
  none). **GREEN:** 1 passed, 16.5 s.
- **How:** the stored `module-source-closure-requests` row of a `mod name;` declaration
  is joined in DuckDB to the declaration's span and to the file the run read
  (`module-file-area-projection.sql`, `build_module_area_records`). The crate-root row is
  built from the unit's root path (`build_root_area_edge`). A declaration naming two read
  files is an error, not a choice.
- **Gates:** formatting exit 0; clippy on the two crates exit 0; adapter tests 188
  passed; real-source tests 63 passed, 0 failed, 933 s (command in `AGENTS.md`).
- **Run:** `parseltongue map-workspace-full-run ... --worker-thread-count-value 4
  --sysroot-source-scope-kind declarations`: exit 0, 22.4 s, 377 MB maximum resident set,
  170 nodes, 684 edge rows.
- **Tests changed:** seven real-source tests and two fixture tests stated 682 rows, 88
  `contains`, or "no edge names the crate root".
- **Deviation:** the fixture test `test_module_declaration_area_edge` was written after
  the code. Checked afterwards by turning the new call off: it fails with 1 row, not 2.
- **Decision recorded:** OD-7 closed in the PRD and the backlog (Rule 6; 2 rows).
- **codebase-memory:** `trace_path derive_original_graph_input` inbound
  (`publish_full_graph_checkpoint`, `derive_fixture_graph_input`) and outbound;
  `trace_path publish_full_graph_checkpoint` outbound (17 callees);
  `index_repository` after `b32b47200`.
- **Next:** part 11, node visibility, reachability and signature: spec first.


## 2026-11-18 Part 11 built: visibility, reachability and signature on every item node

- **Result:** a run of pinned itoa equals the re-frozen expected graph with three more
  fields compared on every node, for scope `declarations` (170 nodes) and `excluded`
  (151). `test_itoa_declarations_graph_equality` and `test_itoa_whole_graph_equality`
  pass. The "Where we are" page above still describes the accepted run of `753665eb9`;
  S1 to S4 are NOT STARTED for this head and run once, after the last open point.
- **Stored (exported node rows of the run below):** 46 Public, 26 PrivateEquivalent,
  1 RestrictedNonPrivate, 15 NotApplicable, 82 without a value; reachable 19 true,
  69 false, 82 without; 88 signatures.
- **Commits:** spec `054a8fdba`; expected signatures `c94154305` (11.1); failing test
  `e51f014ab` (11.2); signature printer `2edeb931c` (11.3); rules and `NotApplicable`
  `ca12fad6b` (11.4); expected-graph correction `860c62c45`; projection, pass and
  checkpoint call `28f1db595` (11.5); documents `1c140290b`, `ce870fb9c`, `9a1f6d26f`.
- **Gates at `28f1db595`:** `cargo fmt --all -- --check` exit 0; workspace clippy exit 0;
  core-row-types 26, extractor 177 + 17, DuckDB adapter 188, Turso adapter 46 + 22 + 4
  tests pass; real-source suite 63 passed, 0 failed, 1007.70 s; 25 expected-data tests.
- **Run:** `parseltongue map-workspace-full-run ... --worker-thread-count-value 4
  --sysroot-source-scope-kind declarations`: exit 0, 22.46 s, 368,704 kB peak, 170 nodes,
  90 `contains` rows.
- **Test names:** `test_item_signature_text_printed`, `test_expanded_signature_text_equal`,
  `test_item_visibility_text_read`, `test_node_reach_field_rules`,
  `test_reexport_reach_flag_unknown`, `test_parent_cycle_reach_ends`, and the two equality
  tests.
- **Expected graph corrected:** the 2 file areas were listed as public and not reachable.
  No source text declares that and the node rules forbid it; they now have no value.
- **A miss from part 10, found here:** `test_no_forbidden_semantic_dependencies` still
  expected DuckDB features `[bundled]` and had failed since part 10, because the
  workspace tests were not run after it. Fixed in `f43382e34`.
- **Deviations:** listed in section 6 of the part file (real-source suite not run at the
  11.3 and 11.4 commits; 11.5 pushed while the suite ran, at the user's request; no fixture
  test for the pass itself; one stale adapter fixture assertion). The workspace test gate
  was not run.
- **Open question left:** whether a member of a private supertrait of a reachable trait
  counts as reachable (PRD, "Open").
- **codebase-memory queries:** `trace_path extract_file_syntax_rows` inbound;
  `trace_path resolve_node_visibility_fields` both; `trace_path
  publish_full_graph_checkpoint` outbound; `index_repository` after each code commit.
- **Next:** part 13, enum variants and struct fields as nodes: spec, expected rows, new
  freeze, then code.

## 2026-11-18 Part 13 built: struct fields and enum variants are nodes

- **Result:** a run of pinned itoa stores `Buffer::bytes` and `DecimalPairs::0` as nodes,
  each contained by its struct, and equals the re-frozen expected graph for scope
  `declarations` (172 nodes, 686 edge rows) and `excluded` (153 nodes, 686 edge rows).
  `test_itoa_declarations_graph_equality` and `test_itoa_whole_graph_equality` pass. S1
  to S4 are NOT STARTED for this head.
- **Stored (exported rows of the run below):** both fields have kind `Field`, owner kind
  `Struct`, `PrivateEquivalent`, not reachable, and their signature. Four rows start at
  a field: `Buffer::bytes` uses `MaybeUninit` and `u8` and reads
  `<i128 as Integer>::MAX_STR_LEN`; `DecimalPairs::0` uses `u8`. 92 `contains` rows.
- **Commits:** expected rows `d18aa3fc6` (13.1); spec `ac799a1bd`; owner kinds, the
  owner rule and `VariantMember` `9240249ab` (13.3); field span rows, site owner and
  variant signature `902408ccd` (13.4); projection and visibility pass `ffb60e7c0`
  (13.5); body-local field fix `40cf94ccf`.
- **RED (13.2):** the two equality tests, 0 passed and 2 failed, on "nodes expected 172
  stored 170; edge rows expected 686 stored 684" and "nodes expected 153 stored 151".
- **Gates at `40cf94ccf`:** `cargo fmt --all -- --check` exit 0; workspace clippy exit 0;
  core-row-types 28, extractor 181 + 17, DuckDB adapter 189, Turso adapter 46 + 22 + 4
  tests pass; real-source suite 63 passed, 0 failed, 1012.26 s; 25 expected-data tests.
- **Run:** `parseltongue map-workspace-full-run ... --worker-thread-count-value 4
  --sysroot-source-scope-kind declarations`: exit 0, 21.24 s, 346,900 kB peak, 172 node
  rows, 686 edge rows.
- **Test names:** `test_member_owner_kind_pairs`, `test_variant_member_reach_rules`,
  `test_field_span_rows_written`, `test_field_site_owner_rows`,
  `test_field_signature_text_printed`, `test_body_field_binds_nothing`,
  `test_member_nodes_are_projected`, and the two equality tests.
- **First suite run failed 11 of 63:** 13 node, row and label literals that the two new
  nodes moved (151 to 153 nodes, 90 to 92 `contains` rows, 43 to 45 source-written
  items, and the source of the `i128::MAX_STR_LEN` read). Each was read and changed;
  the second full run passed.
- **A defect of 13.4, fixed before a run showed it:** a field in a function body counted
  as a name binding in `check_receiver_name_rebound` and `check_call_lexical_shadowing`.
- **Deviations:** `test_member_nodes_are_projected` was written after the SQL change;
  no test covers a field written by a macro expansion (G1); the workspace test gate was
  not run. Listed in section 6 of the part file.
- **Not built:** an edge for each use of a field (`self.bytes`, `DECIMAL_PAIRS.0`).
  Backlog, OD-4.
- **codebase-memory queries:** `trace_path` inbound, depth 1, on
  `check_syntax_item_node` (6 callers; two of them needed the fix above),
  `decode_original_item_kinds`, `validate_graph_input_records`,
  `derive_node_reach_fields`, `find_struct_field_type` and `print_item_signature_text`;
  `index_repository` after each code commit.
- **Next:** part 14, stored rows that do not depend on the source directory.

## 2026-11-18 Part 14 built: stored rows do not depend on the source directory

- **Result:** two copies of pinned itoa in two directories, the second mapped through a
  symbolic link, export equal rows: 172 of 172 node rows, 686 of 686 edge rows, and one
  run id. Measured before the change: 0 of 172 and 0 of 686, and two run ids. No row
  holds a real directory. `test_itoa_rows_directory_independent` passes. S1 to S4 are
  NOT STARTED for this head.
- **How:** the loader stores every path under the workspace root, and the package id,
  with the fixed root `/parseltongue-workspace-root`. The real root is used only to open
  files.
- **Commits:** spec `4f438c599`; failing test `2c4cb9aa0` (14.1); the path rule
  `2b45fe36f` (14.2); loader, CLI and syntax port `a57f7e7b7` (14.3).
- **RED (14.1):** `test_itoa_rows_directory_independent`, 1 selected, 0 passed, 1 failed,
  on `((0, 173, 173), (0, 687, 687))` equal lines of each export.
- **Gates at `a57f7e7b7`:** `cargo fmt --all -- --check` exit 0; workspace clippy exit 0;
  core-row-types 29, loader 10 + 3, extractor 181 + 17, DuckDB adapter 189, Turso
  adapter 46 + 22 + 4 tests pass; real-source suite 62 passed, 2 failed, 1017.85 s; the
  2 failed tests pass after their path assertions were changed (rerun by name, 2 passed).
- **Run:** `parseltongue map-workspace-full-run ... --worker-thread-count-value 4
  --sysroot-source-scope-kind declarations`: exit 0, 26.33 s, 352,876 kB peak, 172 node
  rows, 686 edge rows, run id `fdbe5e29…600eda`, equal to the run id of the test's runs
  under `/tmp`.
- **Test names:** `test_workspace_root_path_relocation`,
  `test_loaded_rows_hold_fixed_root`, `test_itoa_rows_directory_independent`, and the
  two equality tests.
- **A second place needed the real root:** the first run after the change stopped with
  "module u128_ext" missing. The macro expansion port builds its own projection and
  asked the disk for the fixed path. It now gets the same value.
- **Spec corrected while building:** D1 said the export header would be equal. Its
  attempt value is a nonce of the run and differs between any two runs; D1 now says so.
- **Deviations:** two tests written with or after their code; the full suite was not
  run again after the two test-only changes; the workspace test gate was not run.
  Listed in section 6 of the part file.
- **Left open:** the run id still hashes the toolchain directory, which is under the
  home directory (PRD, "Open").
- **codebase-memory queries:** `trace_path` inbound, depth 1, on
  `read_safe_source_bytes`, `ingest_requested_source_file`,
  `read_canonical_source_path` and `load_workspace_crate_graph`; `index_repository`
  after each code commit.
- **Next:** part 15, `cast`. Its spec (`7b6c5e5aa`) finds that no trait solver is
  needed.
