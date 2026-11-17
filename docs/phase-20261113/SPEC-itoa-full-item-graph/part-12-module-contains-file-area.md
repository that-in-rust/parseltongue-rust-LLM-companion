# Part 12 — a module contains the file area of its file

Back to the [index](README.md).

Status on 2026-11-17: **specified, not built.** Picked by the user on 2026-11-17 ("a walk
from the crate root finds nothing"). The form is decided by Rule 6: an edge, because it
tells a reader more than no edge. This closes OD-7 of the backlog.

**The defect.** The crate root `itoa` and the module `u128_ext` have no outgoing edge. The
items of `src/lib.rs` hang under `area:src/lib.rs` and the item of `src/u128_ext.rs` under
`area:src/u128_ext.rs`, and nothing joins a module to the area of the file that holds its
items. A walk that starts at the crate root stops at once.

**The change.** Two new `contains` rows on itoa, one per file:

```text
itoa      --contains-->  area:src/lib.rs        site: the whole file src/lib.rs
u128_ext  --contains-->  area:src/u128_ext.rs   site: `mod u128_ext;`, src/lib.rs line 54
```

Built before part 11: whether an item of `src/u128_ext.rs` can be reached from outside
the crate depends on the module that declares that file, which is this edge.

## 1. Rust work mode

- **Component:** the node and containment projection of the DuckDB adapter crate.
- **Modes:** Spec and Delivery. The graph gains rows, so the expected graph is re-frozen.
- **Measured before writing** (2026-11-17, run of the executable `0285ac70…` on pinned
  itoa, scope `declarations`):
  - 88 `contains` rows; none has a file area as its target; none has the crate root or
    `u128_ext` as its source.
  - The stage table `module-source-closure-requests` has one row per `mod name;`
    declaration. Its key is the declaration's occurrence key and its payload lists the
    candidate file paths (`syntax-span-extractor/src/lib.rs`, lines 542 to 593 and 818 to
    830). The graph projection does not read this table today.
  - The crate-root file is known to the projection (`file_is_crate_root` in
    `original-item-graph-projection.rs`).
- **codebase-memory:** `trace_path derive_original_graph_input` (both directions, depth
  2): called by `publish_full_graph_checkpoint`; 11 tests in
  `original-item-projection-tests.rs` reach it through `derive_fixture_graph_input`.

## 2. Executable requirements: REQ-RUST-070.0

```text
A1  WHEN a source file is the root file of a mapped target
    THEN the graph SHALL store one `contains` row from the crate-root node to the file-area
    node of that file, whose site is the whole file (byte 0 to its length)
A2  WHEN an active `mod name;` declaration outside a function body names a file the run read
    THEN the graph SHALL store one `contains` row from that module's node to the file-area
    node of that file, whose site is the declaration
A3  WHEN a module is written inline (`mod name { ... }`)
    THEN no such row SHALL be stored for it; its items keep the module as their container
A4  WHEN the run has ended THEN every file-area node SHALL be the target of exactly one
    `contains` row
A5  WHEN a run of pinned itoa with scope `declarations` is compared with the re-frozen
    expected graph THEN it SHALL be equal: 170 nodes, 684 edge rows, 90 of them `contains`
A6  WHEN the same run uses scope `excluded` THEN it SHALL equal its re-frozen expected
    graph, which gains the same two rows
```

Out of scope: a `mod name;` whose file is absent (the run's behaviour is unchanged), and
files read through `include!`.

## 3. Design (L1 / L2 / L3)

- **L1 (row types):** no new type. The relation is `Contains`; the rows are ordinary
  `HldEdgeInputRecord` values with an `Extracted` target.
- **L3 (DuckDB adapter):**
  - `derive_original_graph_input` copies one more table,
    `module-source-closure-requests`.
  - A new view `original-module-file-links` joins each request of kind
    `module_declaration` to its declaration span (by occurrence key) and to the file-area
    candidate whose path is one of the request's candidates.
  - A new file `module-file-area-projection.rs` turns each link into one edge row
    (`build_module_area_records`), and builds the crate-root row where the crate-root node
    is built today.
- **Nothing is guessed.** The link is the path the extractor computed from the declaration
  (including `#[path]`) and the file the run read. A declaration with no read file gives no
  row.

## 4. Verification matrix

```text
req      test                                   type          assertion
A1 A2    test_module_file_area_rows             real source   the two rows above, with file,
                                                              byte range and digest of each site
A3 A4    test_module_file_area_rows             real source   `private` is no source of such a
                                                              row; each area has one incoming row
A2 A3    test_module_declaration_area_edge      fixture       a two-file unit and an inline module
A5       test_itoa_declarations_graph_equality  real source   equal to the re-frozen graph
A6       test_itoa_whole_graph_equality         real source   equal to the re-frozen graph
```

## 5. Implementation plan

```text
12.1  the two rows added to the expected graph builders from source; both graphs re-frozen
12.2  test_module_file_area_rows, seen failing because the rows are absent
12.3  the view, the projection file and the crate-root row; the tests pass
12.4  gates, log entry
```

## 6. Quality gate results

Not run: nothing is built.

## 7. Open questions

None.
