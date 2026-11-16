# LOG — progress on the itoa full item graph

**Where we are** (below) is rewritten in place whenever it changes. **Entries** are added at
the bottom, never edited. The full log up to step 7.8 (2,788 lines) is in git:
`git show cb40f3d2f:docs/phase-20261113/LOG-itoa-full-item-graph-progress.md`.

## Where we are (2026-11-16, commit `cb40f3d2f`)

**The mapper stores a graph of pinned itoa 1.0.18 that equals the expected graph on
everything the comparison checks: 151 nodes and 682 edge rows. No SOP step is PASS yet: the
expected graph is not frozen and no acceptance run on a frozen binary has been made. The
graph is marked `incomplete` because 133 method calls into `core` have no target.**

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
run          16 to 19 s wall, 365 to 386 MB peak (whole process tree), 20.9 MB store
workers      1, 4 and 16 store byte-identical node and edge rows
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
real-source itoa tests   58 pass, 0 fail (145 s)                       at 64a5c0bc9
workspace tests          601 pass, 0 fail, 4 ignored                   at b0cc7ac81
                         test_lang_items_in_sysroot skipped; it fails when run
format, lint, release    pass                                          at b0cc7ac81
expected-data tests      16 pass
```

The workspace gate is not a full PASS while that test is skipped. It reads the native
sysroot, which itoa runs exclude.

### SOP status

```text
S1  freeze inputs and expected graph   NOT PASSED   dataset still marked LLM_PARSED_DRAFT_NOT_FROZEN
S2  gates and production run           NOT PASSED   one test skipped; process audit not run; graph marked incomplete
S3  exact comparison                   NOT PASSED   the test passes, but against an unfrozen graph
S4  determinism, time, memory          NOT PASSED   measured once, not on a frozen binary; 19 ms samples, not 10
```

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
- **Only itoa has been run** in this phase. Other repositories and higher-level tables are
  out of scope.

### What is left to finish the PRD

1. Done (step 7.9): node position and external-name kind are compared.
2. Fix two SOP lines that make PASS impossible as written ("incomplete cannot pass";
   `cmp` on exports whose header carries a run id).
3. S1: freeze the expected graph and record its digests.
4. S2 to S4 on one frozen binary, with the process audit and 10 ms memory samples.
5. The run report as an entry below.

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
