# Part 4c — `reads` edges from value paths in bodies

Written 2026-11-15 with `rust-executable-specs-01`, before any code.

## The answer

**State: BUILT (all 7 steps, 2026-11-15). Pinned itoa stores 21 `reads` rows: 15 with a
target and 6 unresolved. 3 of the expected graph's 62 grouped `reads` rows are matched;
56 of the rest wait on part 5. One `reads` row per value path in an original function body
that names a `static`, a `const` or an associated const which is a graph node. The first
target is `enc_16lsd --reads--> DECIMAL_PAIRS`.**

- **What is missing is resolution, not syntax.** The body table already stores every path
  expression as a site (175 in original bodies), but no path-resolution request is made
  for a value path. Requests exist only for signature, import and call-prefix paths.
- **Measured on pinned itoa** (probe dump of the stored tables, sysroot excluded):

```text
original value-path sites   what they name                             4c row
14  DECIMAL_PAIRS           static in itoa: fmt for u128 6, enc_16lsd 8  target
1   I::MAX_STR_LEN          trait const through a bound (Buffer::format) target, gap on Copy
6   i128/u128::MAX_STR_LEN  const of a macro-generated impl              unresolved, part 5
1   i128::MAX_STR_LEN       same const, in a struct field type          none (Q2)
7   EXP SIG D M_HIGH SH_POST consts declared inside a function body      none (not nodes)
8   OFFSET                  const generic parameter                    none
114 buf, offset, n, ...     locals and parameters                      none
24  callee position         already a call site                        none
```

- **Expected graph:** 62 `reads` rows, 56 from macro-generated bodies (part 5). Of the 6
  from original source, 3 have an original target: the two `DECIMAL_PAIRS` rows and
  `Buffer::format -> Integer::MAX_STR_LEN`. The other 3 target macro-generated consts.
- **Row grain:** one row per site, as for `calls` and `uses_type`. The expected graph
  groups sites per pair; part 7 reconciles the two.

## 1 — Work mode

Spec mode and delivery mode. Component: L2 syntax extractor and resolver, L3 the CLI facade.
The main risk is a local name being stored as an unresolved read, which would flood the
graph with false rows. The second is a read whose target is wrong because a local item
shadows a module item.

## 2 — Executable requirements

**REQ-RUST-048.0 — `reads` edges from value paths in original bodies.**

- **R1.** WHEN a path expression in an original function body is not in callee position
  and resolves to a `static`, `const` or associated const that is a graph node, THEN one
  `reads` row SHALL be stored from the body's owning item to that node, `Extracted`.
- **R2.** WHEN the path's first segment is bound by an enclosing parameter, `let`, pattern,
  closure parameter, generic parameter or item declared in an enclosing block, THEN no row
  SHALL be stored.
- **R3.** WHEN the path resolves to a function, type, module or other non-value item, THEN no
  `reads` row SHALL be stored.
- **R4.** WHEN a type-relative path `T::NAME` has a type parameter `T` of the same function,
  THEN the target SHALL be found by the part-3 bound and supertrait lookup, with the same
  "proven, gap" record for traits whose source is missing.
- **R5.** WHEN the path is not bound locally and does not resolve to one value node, THEN
  one unresolved row SHALL be stored with one reason, by the part-4a reason rule.
- **R6.** The evidence site SHALL be the path expression's own byte range.
- **R7.** Paths in macro-generated bodies are out of scope; their owners are not nodes until
  part 5. The stage line SHALL count them as `reads_in_expanded_bodies`.
- **R8.** Two reads of the same store SHALL return identical rows in identical order.

## 3 — Design by layer

- **L2 syntax** (`syntax-namespace-input-extraction.rs`): a path-resolution request for each
  path expression in a function body that is not a call's callee, with context
  `body_value_path` and namespace `Value`. Its metadata carries a local-binding flag
  computed by generalising `check_call_lexical_shadowing` to `let`, patterns and block
  items.
- **L2 resolver:** new module `body-value-read-records.rs` with
  `resolve_body_read_edges(store, scope, input, cancel) -> Result<BodyReadEdgeSet, BodyReadEdgeError>`.
  It reuses `read_stored_reason_kind` and the part-3 bound index. `BodyReadEdgeError` is a
  `thiserror` enum with no free-text variant. Cancellation is checked per row.
- **L3 facade:** adds the rows to the full graph and prints `reads=N
  reads_in_expanded_bodies=N` on the stage line.
- **L1:** no change expected. The validator already accepts `reads` between item nodes.

## 4 — Verification matrix

```text
req    test                                   type          assertion
R1 R6  test_decimal_pairs_static_read         real source   enc_16lsd --reads--> DECIMAL_PAIRS, 8 rows, ranges
R1     test_static_read_row_edge              fixture       fn body reads a static: one row
R2     test_local_binding_no_read             fixture       param, let, pattern, block const: zero rows
R3     test_function_value_no_read            fixture       path to a fn or unit struct: zero rows
R4     test_generic_const_bound_read          real source   Buffer::format --reads--> Integer::MAX_STR_LEN, gap Copy
R5     test_unresolved_read_keeps_reason      fixture       missing and ambiguous names keep a reason
R7     test_itoa_reads_row_counts             real source   14 + 1 extracted, 6 unresolved, 0 from locals
R8     test_reads_rows_read_twice             real source   two reads equal, no duplicate
```

Metric for every row: exact equality of source, target or reason, relation and site range.

## 5 — Implementation plan

Each step is one commit of about 200 changed lines, test first.

1. RED: `test_decimal_pairs_static_read` on pinned itoa (0 `reads` rows today).
2. L2 syntax: value-path requests; syntax fixture tests. DONE (`9b88de807`, `35d163f8c`).
   Built differently from section 3: a name bound by a parameter or pattern in scope gets no
   request, and there is no flag. Items declared in a block and generic parameters are
   requested and left to name resolution, which resolves them in the block scope.
3. L2 resolver: single-segment and module paths to value nodes (R1, R2, R3, R6).
   DONE (`674173fd6`, `fcc930c47`). It joins stored rows only: a body site, its stored
   value-path resolution, and the node set. A target that is not a node gives no row.
4. L3 facade: rows in the full graph; step 1 goes GREEN. DONE (`de4c64b03`).
5. Generic `T::NAME` (R4). The target is already found: name resolution treats a type
   parameter's bounds as glob imports. What was left is the "proven, gap" record.
   DONE (`c06d960b2`, `00cda7db9`). The part-3 walk is now one function,
   `walk_parameter_bound_traits`, used by calls and reads. The target is kept only if it
   is a member of a trait the walk reached; otherwise the row is unresolved.
6. Unresolved rows and the counts (R5, R7, R8); `test_itoa_reads_row_counts`.
   DONE (`443123077`, `083b6c63a`, `fc0ebe4e0`).
7. Refactor pass, workspace gates, one run's counts and cost. DONE at `fc0ebe4e0`.

## 6 — Quality gate results

Step 2, measured 2026-11-15 on pinned itoa (sysroot excluded): 70 `body_value_path`
requests, 36 in original bodies and 34 in macro-generated bodies.

```text
original, 36                        state       names
14  DECIMAL_PAIRS                   resolved    Static
1   I::MAX_STR_LEN                  resolved    Const
7   D EXP M_HIGH SH_POST SIG        resolved    Const declared in the function body
8   OFFSET                          resolved    ConstParam
6   i128:: 1, u128::MAX_STR_LEN 5   unresolved  no reason stored
```

Syntax crate 79 + 17 tests pass; the 22 real-source itoa tests pass with
`test_decimal_pairs_static_read` still ignored; format and lint clean on the two touched
crates. Workspace gates run at step 7.

Steps 3 and 4, measured 2026-11-15 on pinned itoa (debug build, sysroot excluded): 58
nodes, 185 edges, `reads=15 reads_in_expanded_bodies=34`.

```text
rows  source           target                 expected graph (grouped rows)
8     enc_16lsd        DECIMAL_PAIRS          matched: source, target, 8 site starts
6     fmt for u128     DECIMAL_PAIRS          matched: source, target, 6 site starts
1     Buffer::format   Integer::MAX_STR_LEN   matched: source, target, exact range
```

3 of the expected graph's 62 grouped `reads` rows are matched by source and target. Syntax
crate 84 + 17 tests pass; the 23 real-source itoa tests pass, none ignored.

Steps 5 and 6, measured 2026-11-15 on pinned itoa by `test_itoa_reads_row_counts` (debug
build, sysroot excluded): 21 `reads` rows.

```text
rows  site text            state        reason or target
14    DECIMAL_PAIRS        extracted    the static
1     I::MAX_STR_LEN       extracted    Integer::MAX_STR_LEN; bound_chain I, Integer,
                                        private::Sealed; missing source: Copy
5     u128::MAX_STR_LEN    unresolved   TraitSolverNeeded
1     i128::MAX_STR_LEN    unresolved   TraitSolverNeeded
```

Still 3 of 62 expected grouped rows matched: an unresolved row has no target to match.
Syntax crate 85 + 17 tests pass; the 26 real-source itoa tests pass, none ignored (296 s).

Step 7, measured 2026-11-15 at `fc0ebe4e0` (release build, sysroot excluded, 2 workers):

- Workspace gates: format, lint and release build exit 0; 486 tests passed, 0 failed, 4
  ignored, `test_lang_items_in_sysroot` skipped (so not a full PASS).
- One run, 3 times: 13.0 to 14.4 s; 358 to 368 MB peak over the process tree (sampled about
  every 17 ms, not the SOP's 10 ms); 11.4 MB store.
- Stage lines: 58 nodes, 191 edges, `reads=21`, `completeness=incomplete`.
- Refactor: the bound walk was split out in step 5; nothing further. Q6 is left open.

## 7 — Open questions

- **Q8. The reason on the 6 unresolved rows says too little.** Name resolution stores no
  reason for `u128::MAX_STR_LEN`, so the part-4a rule gives `TraitSolverNeeded`. The real
  cause is that the const belongs to a macro-generated impl, which is not a node until
  part 5. Part 5 should turn these into rows with a target; if any remain, name resolution
  should store why.

- **Q7. Site of a read through a field.** The expected graph anchors the 14
  `DECIMAL_PAIRS` sites on `DECIMAL_PAIRS.0`; R6 and production use the path
  `DECIMAL_PAIRS`. Starts agree, ends differ by 2 bytes. Part 7 rules on it.

- **Q1. Concrete type-relative consts.** `i128::MAX_STR_LEN` targets the const of the
  macro-generated `impl Integer for i128`. Until part 5 makes it a node, 4c stores the 6
  sites unresolved. Pointing them at the trait's `Integer::MAX_STR_LEN` would be a wrong
  target, so it is not done.
- **Q2. Non-function sources (OD-8).** `Buffer`'s field type `[_; i128::MAX_STR_LEN]` reads a
  const from a struct. 4c covers function bodies only; OD-8 decides the rest.
- **Q3. Consts inside function bodies** (`EXP`, `D`, ...) are not nodes, so reads of them
  have no row. If the graph later gains body-local items, they get rows then.
- **Q5. `I::MAX_STR_LEN` already resolves.** Checked in step 4: it names the trait const
  `Integer::MAX_STR_LEN` (bytes 3680 to 3834), found because name resolution imports a type
  parameter's bounds by glob. Since step 5 the row says that `Copy` was not read (R4).
- **Q6. Two older scope checks.** `check_call_lexical_shadowing` and
  `check_receiver_name_rebound` are separate from the new pattern-scope check. The first
  treats any `use` in a block as shadowing every name. Merging them is step 7 work.
- **Q4. Field access.** `DECIMAL_PAIRS.0` reads the static; the field `.0` is not a node
  (OD-4), so the row targets the static.
