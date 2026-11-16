# LOG — progress on the itoa full item graph

**Where we are** (below) is rewritten in place whenever it changes. **Entries** are added at
the bottom, never edited. The full log up to step 7.8 (2,788 lines) is in git:
`git show cb40f3d2f:docs/phase-20261113/LOG-itoa-full-item-graph-progress.md`.

## Where we are (2026-11-16, production code of commit `37ea1755d`)

**Two states. Accepted: at commit `822da80cf` the mapper stored a graph of pinned itoa
1.0.18 equal to the frozen expected graph (151 nodes, 682 edge rows), and the acceptance run
passed SOP steps S1 to S4 on one frozen executable. That graph is marked `incomplete`: 133
method calls into `core` have no target, and the expected graph was written and checked by
one author. Not accepted: the branch head, where part 8 has changed production to name those
calls. No SOP step has been run on it and one real-source test fails.**

### The branch head (part 8, scope `declarations`; no SOP run)

```text
                              accepted run   part 8 pushed   head (WIP, run once,
                              822da80cf      7ed552478       targets not checked)
nodes                         151            153             not counted
edge rows                     682            682             682
calls with a target            64 of 197      73 of 197       81 of 197
edge rows with a target       549 of 682     558 of 682      566 of 682
method calls into core named    0 of 133       9 of 133       17 of 133
```

- **Failing at the head:** `test_itoa_core_member_targets`. The run names 17 calls, the test
  lists 9, and the 8 new targets are not checked against the source.
- **Default scope at `7ed552478`:** 151 nodes and 682 edge rows, equal to the expected graph;
  60 of 60 real-source tests passed. Not run again since.
- **Step open:** 8.3b-2b, GREEN not closed (last two entries below).

Everything from here to "What is left" describes the accepted run at `822da80cf`.

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

### What the whole-graph test compares (`test_itoa_whole_graph_equality`)

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

### The code, from the codebase-memory index of `822da80cf`

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

- **133 method calls have no target** (`buf[i].write(..)`, `x.try_into()`, `MAX.ilog10()`,
  `string.len()`). The receiver's type is declared in `core`, which the accepted run does
  not read. Part 8 reads it with scope `declarations` (table above).
- **Nodes store no visibility, reachability or signature.** The PRD allows "unknown".
- **No edge from a module to the contents of its file** (OD-7). `itoa` and `u128_ext` do
  not lead to their items by an edge.
- **Struct fields and enum variants are not nodes** (open in the PRD).
- **Reading the whole sysroot** did not finish in 2,400 s on 2026-11-13, and stops with a
  store copy mismatch since; accepted itoa runs exclude it.
- **The `core` files part 8 reads are not frozen.** The run id hashes the compiler version,
  not those files, and SOP S1 does not list them.
- **Rows depend on the source directory.** Two copies of the pinned source in different
  directories store different run ids and macro-expansion ids. Cause: the store's file
  identity is the absolute path, and the run id hashes those rows. Not fixed.
- **Only itoa has been run** in this phase. Other repositories and higher-level tables are
  out of scope.

### What is left

The SOP run at `822da80cf` is done. Part 8 is under way:

0. First, by the decision of 2026-11-16 20:48: write the expected target of each of the 133
   calls from source and re-freeze, before more lookup code (spec in part 8, section 9).
1. Close step 8.3b-2b: find why 37 of the 46 calls typed `&mut Self::Buffer` do not reach
   the new rule, check the 8 new targets against the source, then GREEN.
2. Steps 8.3b-3, 8.4 and 8.5 (not started).
3. Step 8.6: change the SOP for the `core` read, re-freeze the expected graph, repeat the
   acceptance run.

Open, each needing a decision from the user:

1. Why rows depend on the source directory.
2. `test_lang_items_in_sysroot`.
3. The PRD's open points: struct fields and enum variants as nodes; the edge from a module
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


## 2026-11-16 21:50 IST Part 8 step 9.2: the expected target of the 133 method calls, drafted

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

