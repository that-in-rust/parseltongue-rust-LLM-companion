# LOG — progress on the itoa full item graph

Two parts. **Current state** is rewritten in place whenever it changes. **Entries** is
newest first: add each new checkpoint at the top of that section and never edit older ones.

## Current state (2026-11-16)

**The pipeline runs end to end on pinned itoa 1.0.18 and stores a bounded graph: 103 nodes
(45 written by macro expansions) and 502 edges (88 `contains`, 45 `generates`, 15 `expands`), with 3 of 197 calls resolved (136 of the 197 start in macro-generated function bodies; the 194 without a target: 178 `TraitSolverNeeded`, 10 `MissingTarget`, 6 `ExternalSourceMissing`), 34 `uses_type` rows (29 with a target, 5 unresolved),
32 `implements` rows (29 with a target, 3 unresolved), 55 `reads` rows (all 55 with a
target; 34 of the 55 start in macro-generated function bodies) and 4 `imports` rows (all unresolved: their targets are in `core`).
Five dependencies
are proven from source by name: `<Buffer as Default>::default --calls--> Buffer::new`,
`Buffer::format --calls--> private::Sealed::write`, `Buffer::new --uses_type--> Buffer`,
`u128 --implements--> Unsigned` and `enc_16lsd --reads--> DECIMAL_PAIRS`. No SOP step has passed and the graph is marked
`incomplete`.**

- **Next target:** `calls` to this crate's own functions: the callee path of a call is
  never sent to name lookup (26 bare-name calls, 1 `u128_ext::mulhi`), and the 10 generated
  `Unsigned ::fmt(..)` calls need the member lookup of step 5 with a trait as prefix.
  Part 5d steps 1 to 5 are built: `reads` and `calls` from generated function bodies,
  `uses_type` from generated function signatures, each row naming where its first and
  last token were copied from in the source file, and a member reached through a type
  (`Self::NAME`, `uN::NAME`) has the member of the one stored trait impl as target
  (16 `reads` and 5 `uses_type` rows; `uses_type` now has 62 of 66 with a target).
  Every `calls` row carries the call's text, the callee's text, its shape, and what stopped
  the lookup when it has no target.
  Parts 5a (generated items as nodes, contained, each naming its macro call as a field and
  by a `generates` row), 5b (readable labels: all 103 distinct, each impl member names
  its impl header) and 5c (`expands`: each of the 15 macro calls points to its definition,
  with the matched arm index) are built. Parts 4a to 4d are built.
- **Not built in 5a:** a generated item whose macro call is itself written by a macro
  (nested expansion). The run stops with a typed error; itoa has none.
- **Decided 2026-11-15:** targets in `core` are named external targets (sysroot stays out);
  macro definitions and macro calls at item position are nodes. Neither is built.
- **Produced today:** `contains`, `calls`, `uses_type` from signatures, `implements`
  from impl headers, original and macro-generated, `reads` from original function
  bodies, and `imports` from original `use` items. Every edge kind the expected graph
  uses now has a producer.
- **Expected graph:** an unreviewed draft of 73 nodes and 368 edges
  ([itoa-target-dataset](itoa-target-dataset/)). Nothing can be accepted against it yet.

## Where the work stands

```text
step                                   state         measured or missing
1  item nodes and containment          BOUNDED       43 original items + 2 file areas, 43 contains
2  first call from source              DONE          default -> Buffer::new, read back frozen
3  generic call                        DONE (1 shape) format -> Sealed::write; gap on Copy recorded
4  remaining edge kinds                BUILT (original text)  uses_type 34, implements 32, reads 21 (+34 in part 5d), imports 4
5  macro-generated items and bodies    5a-5c BUILT, 5d 4 of 6  45 generated nodes, 45 generates, 15 expands; generated fn bodies give 34 reads, 136 calls
6  targets in core                     BLOCKED       119 expected edges; sysroot run did not finish
7  expected graph reviewed and frozen  NOT STARTED   339 of 368 expected edges are blocked
8  exact comparison (SOP S3)           NOT STARTED   no comparator for full-graph rows
9  determinism, time, memory (SOP S4)  NOT STARTED
```

## What is proven

- **Run.** `parseltongue map-workspace-full-run --mapping-final-stage-kind
  full-item-dependency-graph` on the pinned source writes and reads back a frozen graph.
- **Nodes.** 103: 43 original items outside function bodies, 45 items written by macro
  expansions, one file area per source file (2), and 13 builtin types that an edge starts
  or ends at. Each generated node names its macro call.
  Visibility and reachability are stored as unknown, not guessed.
- **Edges.** 300: 88 `contains`, 45 `generates`, 15 `expands`, 61 `calls`, 34 `uses_type`, 32 `implements`, 21 `reads` and 4 `imports`. 3 calls are resolved; 58 are stored unresolved
  with a typed reason (`HldUnresolvedReasonKind`). Trait dispatch and generic or ambiguous
  inherent members are stored as `TraitSolverNeeded`. The 58 by stored reason: 1 `ExternalSourceMissing`, 10 `MissingTarget`, 47 `TraitSolverNeeded`.
- **Impl headers.** All 33 active impl blocks have header rows. They are records, not nodes.
- **Tests.** 36 real-source itoa tests pass at commit `b0733521f` (80 s, release build,
  default threads).
- **Workspace gates** after part 5a: format, lint and release build pass; 514 tests pass,
  0 fail, 4 ignored. `test_lang_items_in_sysroot` is skipped, so the gate is not a full PASS.
- **Cost of one run** (2026-11-16, after part 5a, sysroot excluded, 2 workers): 13.7 to
  15.9 s, 359 to 378 MB peak over the whole process tree (sampled about every 18 ms, not the
  SOP's 10 ms), 12.9 MB store, no memory cap.

All counts above were read at commit `bbdb98524`.

## What is missing, in build order

1. **Generic calls, the rest.** Built: a function parameter whose type is that function's own
   type parameter, with bound and supertrait lookup. Not built: type parameters of an
   enclosing impl or trait, `where Self` bounds, bounds with generic arguments.
2. **Body typing.** Types of parameters, `self`, locals and intermediate expressions, as far as
   itoa's calls need them. Everything else stays unresolved with a reason.
3. **Signature lowering.** Parameter and return types, `Self`, generics and bounds as semantic
   records. Only impl headers exist.
4. **Other edge kinds.** Built for original source text. Not built: `uses_type` from
   bodies, and any edge that starts in a macro-generated signature or body.
5. **Macro-generated items.** Nodes are built (5a). Not built: labels, `expands`,
   signatures and bodies as edge sources, for items generated by
   `impl_Integer`, `impl_Integer_size` and `impl_Unsigned`, with expansion identity. Needed for
   `<u128 as Sealed>::write -> <u128 as Unsigned>::fmt -> div_rem_1e16 -> u128_ext::mulhi`.
6. **Targets in `core`.** Real identities for `MaybeUninit`, `Default`, `Clone`, `hint`,
   `str`. Runs today exclude the sysroot and store these unresolved.
7. **Proof.** Freeze the expected graph, compare every row, then check determinism and cost
   ([SOP](SOP-itoa-full-item-graph-acceptance.md) S1 to S4).

## Decisions that block freezing the expected graph

```text
OD-2  DECIDED 2026-11-15: call + place in expansion;       45 generates rows to add
      lexical container; generates edge and node field
OD-8  may reads and uses_type start at a non-function item  working decision: yes
OD-7  how a file's module node relates to its file area     blocks 2 edges
OD-4  enum variants and struct fields: nodes or evidence
core  DECIDED 2026-11-15: named external target             119 expected rows to re-cut
```

Detail and deferred work: [backlog](BACKLOG-deferred-work-open-decisions.md).

## Entries — newest first

### 2026-11-16 02:30 IST — Part 5d step 5: a member reached through a type (REQ-RUST-052.0 G5)

- **Built.** `Self::NAME` and `uN::NAME` in function bodies (`reads`) and `Self::NAME` in
  member signatures (`uses_type`) that name lookup left without a target are looked up in
  the stored impl headers. The target is the impl member, when exactly one stored impl of
  the type (or of a type with no stored target) declares the name and it is a trait impl
  with a stored trait, no generic parameter and no negation. New file
  `type-relative-member-records.rs`; one new Type request per qualifier of a two-segment
  body value path (context `body_value_type_prefix`).
- **RED** (real source, before the code): `test_type_relative_member_targets` failed with
  `u128::MAX_STR_LEN has no target`. Command:
  `cargo test --release -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests test_type_relative_member_targets`.
- **GREEN**, same command: `1 passed; 0 failed; 40 filtered out`. Whole file:
  `test result: ok. 41 passed; 0 failed` (97.6 s). Syntax crate: `102 passed` and `17 passed`.
- **Measured on pinned itoa at `705efeae5`** (release, 3 runs): 15.7 to 20.4 s, 344 to
  360 MB peak, 16 MB store. 103 nodes, 502 edges, both unchanged. `reads` 55 of 55 with a
  target (was 39). `uses_type` 62 of 66 (was 57). `calls` 3 of 197, unchanged. 21 rows
  carry `type_relative_member_evidence`, all with outcome `single_trait_impl_member`.
  Path rows: 12 new prefix requests (u128 6; i128, u8, u16, u32, u64, `I` 1 each).
- **Prediction off by one:** 4 `uses_type` rows predicted, 5 measured (the source-written
  `fmt` for u128 also has `Self::Buffer`).
- **Four older assertions changed** to the new measured values: generated reads without a
  target 10 to 0; the 6 source-written `u128::`/`i128::MAX_STR_LEN` reads now have targets;
  generated `fmt -> Self::Buffer` has a target; path totals count the prefix requests apart.
- **Second opinion** (GLM-5.3, pi, max reasoning, read-only), recorded in
  `docs/devin-research/1794776377-second-opinion-glm-5-3-max-type-relative-member-step-five.md`.
  Adopted: unread trait impls for builtins in the record; candidates kept on rows without a
  target; rule G5 reworded to the code. Disagreed: it says a type of this crate has nothing
  unread; a blanket impl in an unread crate applies to it, and the row says so.
- **Refactor:** the lookup returns one value (target or not, plus the record) so both
  producers keep the record on a row without a target; `ReadTargetLookupScope` groups four
  borrowed indexes (clippy `too_many_arguments`).
- **Weak evidence:** the 9 fixture tests passed on first run. Four mutations of the code
  each made at least one fail (unknown self type dropped from the candidates, member kind
  not checked, inherent impl accepted, prefix requested for any qualified path).
- **Not built:** pipeline-level fixtures for an impl inside a function body, `Self` in a
  closure against a nested function, and a `use` alias as prefix (Q23). No itoa row has an
  outcome other than `single_trait_impl_member`, so the three other outcomes are shown only
  by fixtures.
- **Rule 4:** `detect_changes` run before the commit. Workspace gates not run; they run at
  the end of 5d.
- **Next:** send the callee path of a call to name lookup (26 calls to own free functions,
  1 `u128_ext::mulhi`), then `Unsigned ::fmt(..)`.

### 2026-11-16 04:10 IST — Part 5d step 4b: correction to step 4, and the stored reason on 5 calls (REQ-RUST-052.0 G6)

- **Correction to the entry below.** It says the 10 generated `fmt` calls are `Self::fmt` /
  `uN::fmt` with a prefix that has no stored target, and counts 27 calls to this crate's
  own free functions. Both were written without reading the rows. Read from the stored
  path table of the step-4 run: the callee is `Unsigned ::fmt`; its prefix is stored as
  resolved, to the trait `Unsigned`. The bare names are 26 calls to own free functions
  (18 generated, 8 original) and 1 `DecimalPairs(..)` constructor.
- **Stored prefix rows of call sites** (table `path-resolution-site-records`, context
  `body_call_type_prefix`): 10 `Unsigned` resolved Trait; 1 `u128_ext` resolved Module;
  9 `u128`, 1 `str` Builtin; 2 `Buffer` Struct; 4 `mem`, 1 `hint`, 1 `MaybeUninit::<u8>`
  unresolved `external_source_missing`.
- **RED.** `test_generated_body_call_rows` with the corrected expectation: 0 passed, 1
  failed, `left: {.. "qualified_path_call fmt type_prefix_unresolved": 10,
  "qualified_path_call size_of callee_path_not_requested": 4} right: {.. "qualified_path_call
  fmt type_prefix_is_trait": 10, "qualified_path_call size_of type_prefix_unresolved": 4}`.
- **GREEN.** A prefix stored as a trait or a module gives `type_prefix_is_trait` /
  `type_prefix_is_module`. A call shape the lookup does not handle, whose prefix is stored
  unresolved with `cfg_inactive` or `external_source_missing`, takes that stored reason
  instead of `TraitSolverNeeded` (5 rows on itoa: 4 `mem ::size_of ::<Self >`, 1
  `MaybeUninit::<u8>::uninit`).
- **Measured after.** 197 calls, 502 edges, 3 resolved. By typed reason: 178
  `TraitSolverNeeded`, 10 `MissingTarget`, 6 `ExternalSourceMissing`.
- **Gates.** `cargo fmt --all -- --check`: pass. Lint on the syntax crate: pass. Syntax
  crate: 93 + 17 passed. Real-source itoa tests: 40 passed, 0 failed, 88.42 s.
- **Weak evidence.** No fixture test pins the two new detail values or the stored-reason
  branch; only the itoa test does.
- **Next.** Step 5 (G5), unchanged.

### 2026-11-16 03:40 IST — Part 5d step 4 built: `calls` from generated bodies (REQ-RUST-052.0 G1-G3, G6)

- **Result.** Pinned itoa stores 197 `calls` rows (was 61) and 502 edges (was 366), read
  back from the stored table. Predicted 197 and 502; measured 197 and 502. Resolved calls
  stay at 3: all 136 new rows are without a target.
- **What each `calls` row now carries** (original and generated): `call_source_text`,
  `callee_source_text`, `call_shape_kind`, `dispatch_kind`, and, without a target,
  `unresolved_detail_kind`. Generated rows add `expansion_source_origin`.
- **The 136 generated rows:** 104 method calls (`receiver_type_not_determined`), 18 bare
  names `divmod100` / `slice_buffer_to_str` (`callee_path_not_requested`), 10 `fmt`
  through `Self` or `uN` (`type_prefix_unresolved`), 4 `size_of`
  (`callee_path_not_requested`).
- **STUB.** None: no new signature; one parameter added to `build_original_call_edge`.
- **RED.** `cargo test --release -p parseltongue-cli-dispatcher-binary --test
  itoa-source-acceptance-tests test_generated_body_call_rows`: 0 passed, 1 failed, 39
  filtered out, `called Option::unwrap() on a None value` at the read of
  `call_source_text`.
- **GREEN.** `build_original_call_edge` no longer drops sites in expanded text, rejects a
  site whose chain differs from its owner's, and writes the new metadata;
  `extract_original_call_syntax` stores the callee's text.
- **My prediction was wrong once.** I asserted `member_lookup_unproven` for the 10
  generated `fmt` calls. Measured `type_prefix_unresolved`: the prefix `Self` / `uN` has
  no stored target, so the lookup stops before members are read. The test pins the measured
  value.
- **Tests.** New real-source `test_generated_body_call_rows`. New fixture
  `test_call_metadata_site_mismatch` (weak: written after the code, passed on first run).
  `test_generates_edge_row_counts` 366 -> 502 (it failed on `left: 502 right: 366` first).
- **Decisions (Rule 6, in the PRD).** `dispatch_kind` says how the target was chosen, so a
  row without a target is `undetermined`; `original_associated_function` is renamed
  `type_path_associated_function`. Rejected: a separate value for generated calls (where
  the call is written is already in the site and the origin key). Rejected for now: new
  typed reason kinds in the core row types; the typed reason is unchanged and the detail
  key carries the cause (part file Q22).
- **Not done.** The callee path of a call is never requested from name resolution, so the
  27 calls to this crate's own free functions (18 generated, 9 original) cannot get a
  target yet (part file Q21).
- **REFACTOR.** None. `build_original_call_edge` is long; splitting it is left for the step
  that adds callee-path targets.
- **Gates.** `cargo fmt --all -- --check`: pass. `cargo clippy --release -p
  parseltongue-syntax-span-extractor -p parseltongue-cli-dispatcher-binary --all-targets
  --all-features -- -D warnings`: pass. Syntax crate: 93 + 17 passed. `cargo test --release
  -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`: 40 passed,
  0 failed, 81.93 s. Workspace gates not run; they run at the end of 5d.
- **codebase-memory.** `trace_path` inbound on `build_original_call_edge` before the change
  (one caller, `resolve_original_call_edges`; fixtures reach it through that).
- **Next.** Step 5: trait members through a type (G5), which gives 16 `reads` and 4
  `uses_type` rows a target and unblocks the 10 `fmt` calls.

### 2026-11-16 03:00 IST — Part 5d step 3 built: `uses_type` from generated function signatures (REQ-RUST-052.0 G1-G4)

- **Result.** Pinned itoa stores 66 `uses_type` rows (was 34) and 366 edges (was 334), read
  back from the stored table. Predicted 66 and 366; measured 66 and 366. The 32 new rows:
  12 `write -> Buffer` (the type alias of the same impl), 12 `write -> str`, 4 `fmt -> usize`,
  4 `fmt -> Self::Buffer` without a target (`TraitSolverNeeded`, step 5). All 32 carry
  `expansion_source_origin`, both tokens from a macro definition.
- **STUB.** None: no new signature; the change lifts a filter and reuses step 2's helper.
- **RED.** `cargo test --release -p parseltongue-cli-dispatcher-binary --test
  itoa-source-acceptance-tests test_generated_signature_type_uses`: 0 passed, 1 failed,
  `left: {} right: {("fmt", "Self::Buffer", "TraitSolverNeeded"): 4, ("fmt", "usize",
  "builtin usize"): 4, ("write", "Self::Buffer", "own Buffer"): 12, ("write", "str",
  "builtin str"): 12}`.
- **GREEN.** `build_signature_row_edges` no longer drops rows in expanded text and rejects a
  row whose file or chain differs from its owner's; `build_type_path_edge` adds the origin
  key. Test passes.
- **Tests changed.** `test_body_and_macro_rows_skipped` is now `test_body_owner_rows_skipped`
  and expects a typed error for an expanded row under an original owner (weak: not run
  against the old code). `test_generates_edge_row_counts` 334 -> 366 (it failed on
  `left: 366 right: 334` first). `read_edge_site_range` in the real-source tests returns
  nothing for a site in expanded text (closes Q19).
- **REFACTOR.** None needed.
- **Gates.** `cargo fmt --all -- --check`: pass. `cargo clippy --release -p
  parseltongue-syntax-span-extractor -p parseltongue-cli-dispatcher-binary --all-targets
  --all-features -- -D warnings`: pass. Syntax crate: 92 + 17 passed. `cargo test --release
  -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`: 39 passed,
  0 failed, 80.91 s.
- **codebase-memory.** `trace_path` both directions on `resolve_signature_type_edges` (3
  callers: the full-graph publisher and two fixtures) before the change.
- **Next.** Step 4: `calls` from generated bodies (predicted 197 calls, 502 edges), with the
  callee's source text, a dispatch kind that does not say "original", the stored reason, and
  the origin key.

### 2026-11-16 02:20 IST — Part 5d step 2 built: where a generated site was copied from (REQ-RUST-052.0 G3)

- **Result.** Each of the 34 `reads` rows sited in expanded text now carries
  `expansion_source_origin` in its metadata: the source file, byte range and kind
  (definition or call) of its first token and of its last token, stored apart. Read back
  from the stored edge table (measured): 24 `DECIMAL_PAIRS` rows, both tokens from the
  `impl_Unsigned` definition; 5 `Self ::MAX_STR_LEN`, both from the `impl_Integer`
  definition; 5 `uN ::MAX_STR_LEN`, first token from the call (`u8` .. `u128`), last from the
  definition. The 21 rows sited in original text carry no such key. Graph size unchanged:
  103 nodes, 334 edges.
- **STUB.** `expanded-site-origin-records.rs`: `read_expanded_token_origins`,
  `locate_site_token_pair`, `build_site_origin_value`, all `todo!()`.
- **RED.** `cargo test --release -p parseltongue-syntax-span-extractor expanded_site_origin`:
  0 passed, 2 failed, `not yet implemented: REQ-RUST-052.0 G3`
  (`test_site_token_origin_pair`, `test_site_origin_missing_rejected`).
  `cargo test --release -p parseltongue-cli-dispatcher-binary --test
  itoa-source-acceptance-tests test_generated_read_source_origins`: 0 passed, 1 failed,
  `left: Null right: "src/lib.rs"`.
- **GREEN.** The three functions read `macro-expanded-item-table` and pick the stored tokens
  inside a site; `build_body_read_edge` adds the key for a site in expanded text. Unit: 2
  passed. Real source: 1 passed, 15.03 s.
- **REFACTOR.** `ImplHeaderSourceContext::relate_unit_source_path` taken out of
  `occurrence`, so the origin path and the occurrence path are made relative by one rule.
- **Gates.** `cargo fmt --all -- --check`: pass. `cargo clippy --release -p
  parseltongue-syntax-span-extractor -p parseltongue-cli-dispatcher-binary --all-targets
  --all-features -- -D warnings`: pass (first run failed on `unwrap` in the new test file;
  it now carries the same allow line as the other test files). Syntax crate: 92 + 17 passed.
  `cargo test --release -p parseltongue-cli-dispatcher-binary --test
  itoa-source-acceptance-tests`: 38 passed, 0 failed, 90.95 s.
- **Decisions.** A site whose edge cuts a stored token is a typed error, not a nearest-token
  guess. Rejected: storing the origin of every token of the site (a call site can hold a
  whole argument list; first and last bound it and stay small).
- **Not built.** The key on `uses_type` and `calls` rows (steps 3 and 4).
- **codebase-memory.** `search_graph --name_pattern '.*origin.*' --label Function` (no
  existing reader of token origins outside the adapter); `trace_path` inbound on
  `build_body_read_edge` (1 caller) and `occurrence` (8 callers, all still served by the
  unchanged signature).
- **Next.** Step 3: `uses_type` from generated function signatures (predicted 66 rows, 366
  edges), with the same origin key.

### 2026-11-16 01:35 IST — Part 5d specified; step 1 built: `reads` from generated function bodies (REQ-RUST-052.0)

- **Result.** A function written by a macro expansion is now a source of `reads` rows.
  Pinned itoa stores 55 `reads` rows (was 21) and 334 edges (was 300), read back from the
  stored table. Predicted 55 and 334 from a hand-lifted filter; measured 55 and 334.
  - 24 new rows with a target: `<uN as Unsigned>::fmt --reads--> DECIMAL_PAIRS`, 6 in each of
    the 4 generated `fmt`.
  - 10 new rows without a target (`TraitSolverNeeded`): `Self::MAX_STR_LEN` and
    `uN::MAX_STR_LEN` in generated `write` bodies. Step 5 (rule G5) gives them targets.
- **Against the expected graph:** 4 more grouped `reads` rows now match by source and target
  (the four `fmt -> DECIMAL_PAIRS`); 7 of 62 in all. Sites differ by design: the stored site
  is in expanded text, the expected one in `src/lib.rs`; step 2 adds the source origin.
- **Measured before the spec.** Lifting the expansion filter in the three producers by hand
  (then restoring): reads +34, uses_type +32, calls +136 (all unresolved), 502 edges.
- **STUB.** None needed: no new signature. The change removes a filter.
- **RED.** `cargo test --release -p parseltongue-cli-dispatcher-binary --test
  itoa-source-acceptance-tests test_generated_fmt_static_reads`: 0 passed, 1 failed,
  `itoa::src/lib.rs::<u8 as Unsigned>::fmt left: 0 right: 6`. (First run failed on my own
  test: 5 nodes end in `as Unsigned>::fmt`; the one for `u128` is source-written.)
  `cargo test --release -p parseltongue-syntax-span-extractor
  test_expanded_site_owner_mismatch`: 0 passed, 1 failed (no typed error returned).
- **GREEN.** `resolve_body_read_edges` builds a row for every site; `build_body_read_edge`
  rejects a site whose file or chain differs from its owner's. Both tests pass.
- **Tests changed.** `test_expanded_body_read_counted` removed (REQ-RUST-048.0 R7 is
  superseded by G1). `test_itoa_reads_row_counts` counts rows sited in original text (21).
  `test_reads_rows_read_twice` 21 -> 55. Edge total 300 -> 334.
- **REFACTOR.** None; the change is 20 lines.
- **Gates.** `cargo fmt --all -- --check`: pass. `cargo clippy --release -p
  parseltongue-syntax-span-extractor -p parseltongue-cli-dispatcher-binary --all-targets
  --all-features -- -D warnings`: pass. Syntax crate: 90 + 17 passed. `cargo test --release
  -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`: 37 passed,
  0 failed, 82.61 s. Workspace gates and run cost not measured for this step.
- **Decisions.** Site stays in expanded text; the source origin goes in metadata, first and
  last token apart (G2, G3). Rejected: siting the row in `src/lib.rs` (no range there holds
  the text of `u8::MAX_STR_LEN`), and one merged origin range.
- **Second opinion.** GLM-5.3, pi, max reasoning:
  `docs/devin-research/1794772360-second-opinion-glm-5-3-max-part-5d-generated-edge-sources.md`.
  It moved the origin step ahead of `uses_type` and `calls`.
- **codebase-memory.** `index_status`; `trace_path` inbound and outbound on
  `resolve_body_read_edges`, `resolve_signature_type_edges`, `resolve_original_call_edges`
  before the experiment; `trace_path` inbound on `resolve_body_read_edges` and
  `build_body_read_edge` before the commit (3 callers each, one of them the removed test).
- **Next.** Step 2: the origin of a site's first and last token, from
  `macro-expanded-item-table`, on the 34 generated `reads` rows.

### 2026-11-16 01:10 IST — Part 5c: `expands`, macro call to macro definition (REQ-RUST-051.0)

- **Result.** Pinned itoa stores 15 `expands` rows, one per macro call written in a source
  file, each to the `macro_rules!` definition it expands: 5 to `impl_Integer`, 6 to
  `impl_Integer_size`, 4 to `impl_Unsigned`. Each row is sited at the whole call and carries
  `{"macro_matched_arm_index":0,"macro_name_text":".."}`. 103 nodes, 300 edges (was 285),
  read back from the stored edge table. Predicted 15 and 300; measured 15 and 300.
- **Against the expected graph:** no row gained or lost. It has no macro nodes; part 7
  repairs it.
- **STUB.** `check_expands_edge_record` with `todo!("REQ-RUST-051.0 E9")`; the new relation
  kind in the three store codecs; `expands=N` on the stage line.
- **RED.** `cargo test --release -p parseltongue-cli-dispatcher-binary --test
  itoa-source-acceptance-tests expands_edge_row`: 0 passed, 1 failed on
  `not yet implemented: REQ-RUST-051.0`. Validator fixture `test_expands_row_endpoint_kinds`
  failed on the same stub.
- **GREEN.** Producer `build_macro_expands_records` (SQL plus one Rust file), called after
  the generated items. Same command: 1 passed, 18.97 s.
- **Defect on the way.** First GREEN attempt stopped with "expanded macro call has no
  definition node in textual scope". Cause: the SQL read the compilation unit from
  `legacy_source_unit_key`, which is empty in the stored rows; it now reads it from
  `legacy_source_site_key.unit_compilation_identity_key`.
- **Spec revised when built (E4).** The first wording required every stored lookup of a
  call to name the same definition. The stored table has one lookup per scope revision, and
  the existing name resolver uses the highest revision; the producer now does the same.
- **Fixture tests (adapter).** 5 new, all passed on first run, so I disabled the revision
  filter and the token check and re-ran: `test_expands_latest_scope_revision` and
  `test_expands_token_origin_mismatch` failed, 3 passed. The other three were never seen
  failing. Code restored before the gates.
- **REFACTOR.** None. Producer is 156 lines, SQL 75, tests 201.
- **Gates.** `cargo fmt --all -- --check`: pass. `cargo clippy --release` on the four touched
  crates, `--all-targets -D warnings`: pass. Core 22 passed; adapter 184 passed; Turso
  adapter 71 passed, 3 ignored. `cargo test --release -p parseltongue-cli-dispatcher-binary
  --test itoa-source-acceptance-tests`: 36 passed, 0 failed, 79.92 s (default threads).
  Workspace gates not run; they run at the end of part 5.
- **Cost, 3 runs (binary `117b3ba5…ee1f`):** 14.91, 15.14, 15.75 s; 360, 343, 361 MB peak;
  13.1 MB store.
- **Not built.** A macro called by a path (`crate::m!`): typed error, itoa has none. The
  validator checks one row at a time; "every macro call has exactly one row" is pinned by
  the real-source test only (part file Q15, Q16).
- **Second opinion.** GLM-5.3, pi, max reasoning, before the spec:
  `docs/devin-research/1794770900-second-opinion-glm-5-3-max-part-5c-expands-edge.md`.
- **codebase-memory.** `search_graph` for `build_generated_item_records` and
  `derive_original_graph_input` (the call site of the new producer); `detect_changes`
  before the commit. Not run before each test, which Rule 4 asks for.
- **Commits.** `3232b76cb` (spec), `e9c5bfd59` (kind, validator, codecs), `b0733521f`
  (producer and tests).
- **Next.** Part 5d, generated bodies and signatures as edge sources.

### 2026-11-16 00:40 IST — Part 5b: readable node labels (REQ-RUST-050.0)

- **Result.** All 103 node labels on pinned itoa are distinct (was 45), read back from the
  stored table. Labels start `itoa::src/lib.rs::`; none carries the checkout path.
  An impl member names its impl header (`<u8 as private::Sealed>::write`, `Buffer::new`);
  a macro call and a use item read as their source text; a generated `const _` names its
  call (`const _ #impl_Integer!(i8, u8)`). Nodes and edges unchanged: 103 and 285.
- **Against the expected graph:** no row gained or lost. Its labels have no crate and file
  prefix and write `const _ #impl_Integer(i8,u8)`; part 7 repairs it to the stored form.
- **STUB.** `item-node-label-text.rs` with four `todo!()` functions.
- **RED (fixture).** `cargo test --release -p parseltongue-duckdb-analysis-adapter
  item_node_label`: 0 passed, 4 failed, each on `not yet implemented: REQ-RUST-050.0`.
- **RED (real source).** `test_node_labels_all_distinct`: `left: 45 right: 103`.
  `test_impl_member_label_segment`: `itoa::src/lib.rs::<file-area>`, `left: 0 right: 1`.
- **GREEN.** One helper, called by both projections. Adapter: 179 passed. Real source:
  `cargo test --release -p parseltongue-cli-dispatcher-binary --test
  itoa-source-acceptance-tests -- --test-threads=1`: 35 passed, 0 failed, 448.34 s.
- **Two failures on the way, both tests that named a node by its old label:**
  `test_generated_item_node_counts` (`[0, 12, 12, 12, 4]` for `[5, 12, 12, 12, 4]`) and the
  fixture `test_preserve_known_syntax_kinds`, whose macro call carried an invented name.
  Both now use the new form. `test_generated_node_names_call` finds `const _` by kind.
- **REFACTOR.** None needed: the two copies of the label code became one function.
- **Gates.** Format and lint pass on the adapter and CLI crates. Workspace gates not run;
  they run at the end of part 5.
- **Cost, 3 runs (binary `bcc71b7f…d8a1`):** 13.89, 14.36, 18.78 s; 340, 336, 353 MB peak;
  12.89 MB store.
- **Decisions.** One label string, no typed impl-header field (backlog BL-09). Call text only
  on a generated item with no name. Crate target name, not package id. Cut at 160
  characters, not at the first newline as the review proposed.
- **Second opinion.** GLM-5.3, pi, max reasoning, before the spec:
  `docs/devin-research/1794768300-second-opinion-glm-5-3-max-part-5b-node-labels.md`.
- **codebase-memory.** `index_status`; `search_graph` for the four projection functions;
  `trace_path build_generated_item_pair` both ways, depth 2; `detect_changes` before the
  commit. Not run before each test, which Rule 4 asks for.
- **Commits.** `f8a95c80b` (spec), `47d4d05e0` (code and tests).
- **Next.** Part 5c, `expands`.

Add new entries directly below this line. One entry per checkpoint: time, what changed, the
test names and exact command, pass and fail counts, measured numbers, commit, next target.

### 2026-11-16 00:05 IST — Part 5a closed: each generated node names its macro call, twice

**Pinned itoa now stores 285 edges (was 240): 45 `generates` rows, one from a macro call
to each active item it wrote. Each of the 45 generated nodes also carries that call as a
typed field. Nodes are unchanged at 103. The graph is still marked incomplete.**

- **Node field.** `node_generating_call_reference` on `HldNodeInputRecord`: the call for a
  generated node, nothing for a source-written node. RED on real source:
  `test_generated_node_names_call`, "no call named: ...::src/lib.rs::_". GREEN after the
  candidate view also returned the call's own span.
- **Relation.** `HldRelationKindValue::Generates`, written and read through DuckDB text,
  the Turso graph input and publication. RED: `test_generates_edge_row_counts`, 0 rows.
- **Validator.** A `generates` row must start at the call its target names and be sited
  at that call. RED: `test_generates_edge_site_checked` and
  `test_generates_edge_source_checked` returned `Ok(())`; GREEN with one check in the
  edge loop (`InvalidGeneratingCall`). Core validator tests: 21 passed.
- **My test was wrong once.** I asserted that all 15 expanded macro calls write an item.
  Measured: 11 do. Five `impl_Integer!` calls write 7 items each, the two
  `impl_Integer_size!` calls for 64-bit pointers write 3 each, four `impl_Unsigned!` calls
  write 1 each (35 + 6 + 4 = 45). The other four `impl_Integer_size!` calls write only
  items under a `#[cfg]` that is off. The assertion now pins
  `[1, 1, 1, 1, 3, 3, 7, 7, 7, 7, 7]`.
- **Repeat.** `test_generated_nodes_read_twice`: a second read of the store gives the same
  node list and the same edge list, in order. It passed on first run, so it records
  behaviour and was not seen failing.
- **Fixture tests, 5, all passed on first run** (so they record behaviour):
  `test_expanded_item_projected_node`, `test_expanded_text_digest_mismatch`,
  `test_inactive_expanded_item_skipped`, `test_expanded_body_item_evidence`,
  `test_nested_expansion_call_rejected`.
- **Not built: nested expansion (R4).** The projection finds the generating call only
  when it is written in a source file. A call written by another macro stops the run with
  `InvalidSourceRow` ("generating macro call is not a source-written macro call span").
  itoa has 15 expanded calls, all one level deep, so nothing in itoa reaches it. The
  fixture test pins the rejection, not the rule in R4.
- **Stage line corrected.** `containment=` printed the count of all structural rows (133
  after this step). It now prints `containment=88 generates=45`. No test reads that line;
  checked on one run.
- **Workspace gates at `bbdb98524`:** format, lint, release build pass; 514 tests passed,
  0 failed, 4 ignored (`test_lang_items_in_sysroot` skipped, so not a full PASS). The 33
  real-source itoa tests took 461 s.
- **Cost, 3 runs** (binary sha256 `6981f029…9107250`): 13.7, 14.6, 15.9 s; 359, 371,
  378 MB peak over the process tree; 12.88 MB store; 103 nodes, 285 edges each run.
- **Against the expected graph:** it has no `generates` rows, no macro call nodes and no
  macro definition nodes yet, so nothing here can be matched until it is repaired (part 7).
  45 of its 45 generated nodes have a stored node of the same kind and name count
  (5 `const _`, 12 `MAX_STR_LEN`, 12 `Buffer`, 12 `write`, 4 `fmt`).
- **Codebase-memory:** `trace_path publish_full_graph_checkpoint --direction inbound`
  (2 callers, both in `main.rs`) before the stage-line change; re-index after the commit.
  The queries before the earlier step-2 edits were not written down.
- **Commits:** `a59f8c4a9` (field), `21af05d9d` (real-source test), `a5d23c030`
  (`generates` rows), `decaf5962`, `2a6879741` (real-source tests), `bbdb98524` (fixture
  tests), and this commit (stage line, docs).
- **Next:** part 5b, labels. Today all 15 macro calls read `itoa::src/lib.rs::MACRO_CALL`
  and the 12 generated `write` functions share one label.

### 2026-11-15 23:25 IST — Part 5a step 1: macro-generated items are nodes with one container

**Pinned itoa now stores 103 nodes (was 58): the 45 active items written by macro expansions
are nodes, each held by the `src/lib.rs` file area. `contains` rows are 88 (was 43). Both
match the prediction. The generating call is not yet on the node or an edge (step 2).**

- **RED:** `test_generated_const_node_contained` failed on behaviour: "the generated const _
  is one node", left 0, right 1 (the pipeline ran; the node was absent).
- **GREEN, three runs:** first failed in the node validator ("node category does not match
  its typed identity"): it required a local node's file to equal the file in its identity.
  Fixed in the validator for nodes whose expansion chain is not empty, with three validator
  tests (one failed first: `test_generated_node_call_file`, `InvalidNodeCategory`). Second
  run passed.
- **Measured** by `test_generated_item_node_counts` (passed on first run, so it records
  behaviour and was not seen failing): 5 `const _`, 12 `MAX_STR_LEN`, 12 `Buffer`,
  12 `write`, 4 `fmt`; 45 generated nodes; 103 nodes; 88 `contains`; every generated node
  has one container, the `src/lib.rs` file area. Inventory of expanded spans: 45 included,
  28 impl blocks, 20 inactive.
- **Edge total:** 240, computed (195 + 45), not read from a run report.
- **Regression found by the adapter tests, not by the itoa tests:** the original-item fixture
  carried a span with an expansion chain and no macro call, to show it was left out. The new
  projection rejects it, so 5 fixture tests failed and every `test_reject_*` in that file
  would have passed for the wrong reason. The span is out of the base fixture and has its
  own test, `test_reject_orphan_generated_span`.
- **Changed existing test:** `test_original_member_graph_roundtrip` now reads original
  nodes only (43).
- **Commands:**
  - `cargo test --release -p parseltongue-cli-dispatcher-binary --test
    itoa-source-acceptance-tests -- --test-threads=1`: 30 passed, 0 failed, 385 s.
  - `cargo test --release -p parseltongue-duckdb-analysis-adapter --lib --
    --test-threads=1`: 170 passed, 0 failed.
  - `cargo test --release -p parseltongue-core-row-types`: 15 passed, 0 failed.
  - `cargo fmt --all -- --check` and `cargo clippy --release -p
    parseltongue-core-row-types -p parseltongue-duckdb-analysis-adapter -p
    parseltongue-cli-dispatcher-binary --all-targets --all-features -- -D warnings`: pass.
  - Workspace gates not run; they run at the end of 5a.
- **Refactor:** none yet. `build_generated_item_pair` is 95 lines and reuses the original
  item helpers through an `OriginalGraphSourceContext` that now carries the chain; it is
  split when step 2 adds the `generates` edge.
- **Not checked:** a nested expansion (a macro call inside expanded text) and a macro call
  inside an inline module. itoa has neither at item position.
- **Codebase-memory queries:** `search_code` for the validator message (found
  `validate_hld_node_input`); `detect_changes`; `trace_path validate_hld_node_input`
  inbound, depth 2 (3 callers); `search_code node_signature_text` (27 places build or read
  a node record, the cost of step 2's field).
- **Commits:** `74829b098` (validator), `64521d5ae` (projection), `fae98c496` (itoa tests).
- **Next:** step 2, `node_generating_call_reference` on the node and the `generates`
  relation, test first on the same `const _`.

### 2026-11-15 17:40 IST — Part 5a specified: macro-generated items as nodes

**Part 5 is cut into five sub-parts and 5a is specified, no code. The store already holds
all 45 generated items the expected graph has; one SQL filter keeps them out of the graph.**

- **Measured** from the exported store of one release run at `499e04559`:
  - `span-reference-text-table`: 148 rows, 93 from expanded text.
  - Of the 93: 45 active non-impl items (17 `CONST`, 16 `FN`, 12 `TYPE_ALIAS`), 28 active
    impl blocks, 20 spans under `#[cfg]` that is off.
  - `macro-expanded-item-table`: 15 item-position calls expanded, 16 calls of `core`
    macros unresolved (`external_source_missing`).
  - Sites already stored in expanded text: 497 of 765 body sites, 108 of 168 signature
    rows, 28 of 33 impl headers.
- **Predicted, not measured:** 103 nodes and 88 `contains` rows after 5a.
- **Sub-parts:** 5a nodes and containment; 5b labels; 5c `expands`; 5d generated bodies as
  edge sources (reads, uses_type, calls); 5e the chain `write -> fmt -> div_rem_1e16 ->
  mulhi`.
- **Found while measuring, not fixed:** node path text does not tell nodes apart. All 15
  macro calls read `itoa::src/lib.rs::MACRO_CALL`, and `Buffer::new` reads
  `itoa::src/lib.rs::new`. Ten generated `write` functions would share one text. This is 5b.
- **Second opinion (Rule 5):** GLM-5.3 through pi, max reasoning, read-only. Used: container
  from the call site, smallest first slice, the cost of a `generates` edge against the
  expected graph, the need for the definition range in part 7. Three of its claims were
  wrong when checked (the `const _` wrapper, path-free unit key, relative stored path).
  Record: `docs/devin-research/1794745800-second-opinion-glm-5-3-max-part-5-macro-items-design.md`.
- **Codebase-memory queries:** `index_repository` (26,105 nodes); `search_graph` for
  `.*(expanded_item|macro_item|generated_item|expansion_item|macro_expan).*` (13 functions,
  none projects a generated item into the graph); `search_graph` for
  `.*(original_item|item_node|node_input|inventory).*` (found
  `build_original_item_identity`, `validate_original_graph_inventory`); `search_graph` for
  `HldNodeInputRecord|build_original_item_identity|read_semantic_occurrence_parts`
  (`HldNodeInputRecord` has 26 inbound references, which is the cost of adding a field).
- **Decided by the user the same day:** R2, R3, and for R5 both a `generates` edge and a
  node field, under the rule "provide maximum information which helps the coding agent code
  better". That rule is now AGENTS.md Rule 6 and is in the PRD.

### 2026-11-15 16:50 IST — Part 4d built: `imports` from `use` items

**Part 4d is built. One run on pinned itoa stores 58 nodes and 195 edges, 4 of them
`imports`, all unresolved with reason `ExternalSourceMissing`. 0 of the 4 expected rows
match by target until part 6; 4 of 4 match by source node, 2 of 4 by exact site range.**

- **Predicted and measured:** 4 rows and 195 edges predicted, 4 and 195 measured.
- **What was built.**
  - Name resolution writes `import-resolution-site-records`: one row per `use` import with
    its state, targets and unresolved reason. "External source missing" comes from the
    SQL rule path requests already use.
  - `resolve_use_import_edges` joins those rows to the import requests and the `use`
    nodes. It looks no name up.
  - The full graph adds the rows; the stage line prints `imports=4` and three skip counts
    (all 0 on itoa).
- **Defect 1, found by the real-source tests.** The first SQL moved the 6 import rows out
  of `path-resolution-site-records` (301 rows became 295), and `i.write(..)` in
  `Buffer::format` lost its target (3 resolved calls became 2). Fixed: the path table is
  unchanged (301 rows, no row differs from `fc0ebe4e0`) and the import row is extra.
- **Defect 2, found by the second review (Rule 5, GLM-5.3 through pi, on the built code).**
  The table held 8 rows on itoa, 4 of them for trait-bound requests of kind `Signature`.
  Fixed: `namespace-import-request-records` stores `request_kind`, and only kind `import`
  gets a row. Measured after the fix: 4 rows.
- **Also from that review:** a stored `trait_solver_needed` reason is kept; the skip test
  uses two globs so swapped counters fail; an absent outcome row is a tested typed error.
  Not done, recorded as Q5 to Q8 in the part file: a site with two targets, path text for
  `::a` and `$crate::a`, a counted skip for an unmapped unit, `ambiguous` at the SQL step.
  Record: `docs/devin-research/1794740417-second-opinion-glm-5-3-max-part-4d-diff-review.md`.
- **Test first.**
  - Step 2b: `test_import_outcome_rows_written` failed with an extra row for the bound
    request (`bf4be6dd4`), then passed (`fc6019e03`).
  - Step 3: 4 of 5 resolver tests failed on the stub (`3420dc73c`), then passed (`06aca66b6`).
  - Step 3b: `test_unresolved_import_keeps_reason` failed with `Other` where
    `TraitSolverNeeded` is expected (`0990f9a68`), then passed (`4f42d9c87`).
  - Passed on first run, so they describe existing behaviour: the two-glob count and the
    absent-outcome error.
- **Refactor:** `build_import_edge_record` taken out of `resolve_use_import_edges`
  (`5dabc3279`). Two files stay over the 200-line guide: the resolver (289) and its test
  file (420).
- **Gates at `5dabc3279`**, release, executable sha256 `22fd0e31...dd88a84`:
  - `cargo fmt --all -- --check`: pass.
  - `cargo clippy --release --workspace --all-targets --all-features -- -D warnings`: pass.
  - `cargo test --release --workspace --all-targets --all-features -- --skip
    test_lang_items_in_sysroot --test-threads=1`: 494 passed, 0 failed, 4 ignored.
    `test_lang_items_in_sysroot` is skipped, so this is not a full PASS.
  - Real-source itoa tests inside that run: 28 passed, 0 failed, 0 ignored, 331 s.
  - `cargo build --release --workspace --all-targets --all-features`: pass.
- **Cost of one run, 3 times:** 13.5 to 14.1 s, 334 to 345 MB peak for the whole process
  tree, 11.5 MB store.
- **Codebase-memory queries:** `detect_changes` (before the last test commit; its base is
  `origin/main`, so it lists 744 files and is of little use here); `search_graph` for
  `resolve_use_import_edges|resolve_import_outcome_targets|read_import_path_text`, which
  showed `resolve_use_import_edges` at 104 lines and led to the refactor; `index_repository`
  after the last commit.
- **Slips against the rules:** steps 4 and 5 are one commit; the resolver commit is over
  200 lines; no codebase-memory trace was run before the step 2b and 3b edits.
- **Next:** part 5, macro-generated items as nodes. Its part file comes first, with the
  identity and container rule put to the user before code depends on it.

### 2026-11-15 14:05 IST — Part 4d specified: `imports` from `use` items

**Part 4d is written, no code: REQ-RUST-049.0, 4 `imports` rows on pinned itoa, all into
`core` and so stored unresolved until part 6.**

- **Part file:** `SPEC-itoa-full-item-graph/part-4d-imports-from-use-items.md`.
- **Measured** from the exported store of one run (`export-published-stage-rows`, no
  temporary test): 3 `UseStatement` nodes; 8 import request rows, 4 of kind `Import`; no
  stored row says what an import resolved to.
- **Second opinion (Rule 5):** GLM-5.3 through pi, about 5 minutes. It changed one choice:
  my draft re-derived "external source missing" in Rust; the analysis stage already
  computes it in SQL, so the stage will write one outcome row per import and the resolver
  will read it. Record:
  `docs/devin-research/1794730674-second-opinion-glm-5-3-max-part-4d-imports-design.md`.
  One claim of the review was wrong (it counted `resolve_use_import_edges` as three words).
- **Codebase-memory queries:** `index_status`; `search_graph Function
  '.*(import|use_tree|use_item|reexport).*'`; `trace_path resolve_import_endpoint_rounds
  both 1`; `trace_path resolve_impl_header_edges both 1`; `trace_path
  read_header_fixture_edges both 1`; `search_graph Function
  '.*(final_output|namespace_output|binding_output|scope_binding).*'`.
- **Next:** step 1, the failing real-source test `test_itoa_imports_row_counts`.

### 2026-11-15 13:40 IST — Part 4c step 7: gates and one run's cost; 4c is built

**Part 4c is built. One run on pinned itoa stores 58 nodes and 191 edges, 21 of them
`reads`, in 13.0 to 14.4 s and 358 to 368 MB.**

- **Code under test:** `fc0ebe4e0` (the later commit `d2a6197dd` changes documents only).
- **Workspace gates,** run from `/home/ubuntu/zcode-runs/4c-step7-gates.sh`:
  - `cargo fmt --all -- --check`: exit 0.
  - `cargo clippy --release --workspace --all-targets --all-features -- -D warnings`: exit 0.
  - `cargo test --release --workspace --all-targets --all-features -- --skip
    test_lang_items_in_sysroot --test-threads=1`: 486 passed, 0 failed, 4 ignored, 1
    skipped by the filter. The 26 real-source itoa tests took 312 s.
  - `cargo build --release --workspace --all-targets --all-features`: exit 0.
  - The skipped test is the open sysroot defect, so the workspace gate is not a full PASS.
- **Cost of one run** (3 runs, binary sha256 `72163607…dd6f2e89`, sysroot excluded, 2
  workers, no memory cap): 13.03 to 14.41 s; 357.8 to 367.5 MB peak, resident memory
  summed over the process tree (at most 2 processes); 11.4 MB store. Sampled about every
  17 ms (757 to 803 samples), not the SOP's 10 ms.
- **Counts from the stage lines,** identical in the 3 runs: `nodes=58 containment=43
  calls=61 resolved_calls=3 type_uses=34 implements=32 reads=21`, read back frozen as
  `nodes=58 edges=191 completeness=incomplete`. The 191 written in the last entry by
  addition is now measured.
- **Store grew** from 9.5 MB after part 4b to 11.4 MB. Not broken down; the new rows are
  the 70 value-path requests and their resolutions, and 21 edges.
- **Refactor:** done in step 5, none added here. Functions in the two files touched, by
  complexity (codebase-memory `query_graph`):
  - `build_original_call_edge`, 143 lines, 18: not split. It is call code this part did
    not change; it should be split when the four unhandled call shapes are built.
  - `walk_parameter_bound_traits`, 57 lines, 10: new in step 5, split out of
    `resolve_bound_method_target`, which went from 100 lines to 61.
  - `resolve_read_path_target`, 84 lines, 10: new in step 6, one match per rule; not split.
  - `build_bound_method_index`, 57 lines, 10: unchanged.
- **Not done, on purpose:** merging `check_call_lexical_shadowing` and
  `check_receiver_name_rebound` with the pattern-scope check (part 4c, Q6). It would change
  which calls are resolved, so it belongs with the call work, test first.
- **Codebase-memory queries:** `query_graph` `MATCH (f:Function) WHERE f.file_path ENDS
  WITH 'body-value-read-records.rs' OR f.file_path ENDS WITH
  'original-call-resolution-records.rs' RETURN f.name, f.start_line, f.end_line,
  f.complexity ORDER BY f.complexity DESC LIMIT 12`.
- **Next:** part 4d, `imports` from `use` items (4 expected rows). Its part file is a
  stub and is written first.

### 2026-11-15 12:40 IST — Part 4c steps 5 and 6: the bound record, and unresolved reads

**Every value path in an original itoa function body that could be a read now has one
`reads` row: 15 with a target and 6 unresolved with a reason. The row for
`I::MAX_STR_LEN` names the bounds walked and that `Copy` was not read.**

- **Commits:** step 5 `c06d960b2` (shared bound walk), `00cda7db9` (real-source test);
  step 6 `443123077` (unresolved rows), `083b6c63a` (fixture test), `fc0ebe4e0`
  (real-source count and read-twice tests).
- **Step 5, what changed:** the part-3 bound and supertrait walk was inside
  `resolve_bound_method_target`. It is now `walk_parameter_bound_traits`, called by the
  method-call resolver and by the reads resolver, so the two cannot drift. A reads row
  whose path starts at a bounded type parameter of its own function carries
  `bound_read_evidence`: `{"bound_chain":["I","Integer","private::Sealed"],
  "missing_source_traits":["Copy"]}`.
- **Step 6, what changed:** a value path stored unresolved or ambiguous gives one row
  without a target (`resolve_read_path_target`). The reason follows the part-4a rule: a
  stored `external_source_missing` or `cfg_inactive` is kept, several targets give
  `AmbiguousTarget`, no stored reason gives `TraitSolverNeeded`. A path to a function, a
  const generic parameter or a const declared in a body still gives no row.
- **Test first:**
  - `test_generic_const_bound_read` failed on `left: Null` where the bound record was
    expected (89 s), then passed.
  - `test_unresolved_read_keeps_reason` failed with only the `TABLE` row where 4 rows
    were expected, then passed.
  - `test_itoa_reads_row_counts` and `test_reads_rows_read_twice` both failed on
    `left: 15, right: 21` (86 s), then passed.
- **Tests:** `cargo test -p parseltongue-syntax-span-extractor`: 85 + 17 passed, 0 failed.
  `cargo test -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`:
  26 passed, 0 failed, 0 ignored (296 s, debug build). Format and lint clean on the two
  touched crates. Workspace gates run at step 7.
- **Measured** by `test_itoa_reads_row_counts` on pinned itoa: 21 `reads` rows.

```text
rows  site text            state        reason or target
14    DECIMAL_PAIRS        extracted    the static
1     I::MAX_STR_LEN       extracted    Integer::MAX_STR_LEN, gap: Copy not read
5     u128::MAX_STR_LEN    unresolved   TraitSolverNeeded
1     i128::MAX_STR_LEN    unresolved   TraitSolverNeeded (source: Buffer::new)
```

- **Edge total:** 191 by addition (170 before part 4c, plus 21). Not yet read from a run's
  stage line; step 7 measures it.
- **Against the expected graph:** still 3 of 62 grouped `reads` rows matched by source and
  target. The 6 unresolved rows have the expected source and no target, so they do not
  count as matched until part 5 makes the macro-generated consts nodes.
- **Weak point, recorded as Q8 in part 4c:** the reason `TraitSolverNeeded` on the 6 rows
  is what the rule gives when name resolution stored no reason. The real cause is that the
  target const is inside a macro-generated impl. A reader of the row cannot tell that today.
- **Rule 4 slip:** before the step-6 change to `build_body_read_edge` I did not run
  `trace_path` on it in both directions; I ran the inbound traces after the commit. Each
  changed function has one caller, and `walk_parameter_bound_traits` has the two intended.
- **Codebase-memory queries:**
  - `trace_path --function-name build_bound_method_index --direction both --depth 1` and
    `trace_path --function-name resolve_body_read_edges --direction both --depth 1`
    (before step 5).
  - `search_graph --label Function --name-pattern
    '.*(type_param|generic_bound|bound_member|assoc|type_relative).*'` (before adding the
    walk: found `resolve_bound_method_target`, which was split instead of copied).
  - `search_graph --label Function --name-pattern
    '.*(unresolved_reason|reason_kind|read_unresolved|map_.*reason).*'` (before step 6:
    found `read_stored_reason_kind`, reused).
  - `trace_path --function-name run_original_graph_fixture --direction inbound --depth 1`
    (returned no test callers; the index does not hold integration-test calls to it, so
    the helpers were found by reading the test file).
  - `detect_changes` before the step-5 commit (useless here: it diffs against
    `origin/main` and listed 739 files).
  - `trace_path --direction inbound --depth 1` on `resolve_read_path_target`,
    `build_body_read_edge`, `walk_parameter_bound_traits`, `resolve_bound_method_target`
    (after the step-6 commits).
  - `index_repository` after each of the two code pushes.
- **Next:** step 7: refactor note, workspace gates, one run's counts and cost.

### 2026-11-15 — part 4c steps 3 and 4: `reads` rows stored from original bodies

**`enc_16lsd --reads--> DECIMAL_PAIRS` is stored from pinned itoa, 8 rows, one per site.
One run now stores 15 `reads` rows and counts 34 value paths in macro-generated bodies.**

- **Commits:** `674173fd6` (resolver `resolve_body_read_edges`), `fcc930c47` (fixture
  tests), `de4c64b03` (facade, `test_decimal_pairs_static_read` no longer ignored).
- **Test first:** the five fixture tests were written against a stub that returned no rows.
  4 failed on behaviour (`[]` where rows were expected, count 0 where 1, `Ok` where a
  typed error); `test_local_binding_no_read` passed on the stub and proves nothing alone.
- **Tests:** `cargo test -p parseltongue-syntax-span-extractor`: 84 + 17 passed, 0 failed.
  `cargo test -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`:
  23 passed, 0 failed, 0 ignored (304 s, debug build). Format and lint clean on the two
  touched crates. Workspace gates run at step 7.
- **Measured** (debug build, sysroot excluded, 2 workers; stage line and exported rows):
  58 nodes, 185 edges, `reads=15 reads_in_expanded_bodies=34`.

```text
rows  source           target                 expected graph (grouped rows)
8     enc_16lsd        DECIMAL_PAIRS          matched: source, target, 8 site starts
6     fmt for u128     DECIMAL_PAIRS          matched: source, target, 6 site starts
1     Buffer::format   Integer::MAX_STR_LEN   matched: source, target, exact range
```

- **Against the expected graph:** 3 of its 62 grouped `reads` rows are matched by source
  and target. The other 59: 56 start in macro-generated bodies (part 5), 2 target the
  macro-generated `<i128 as Integer>::MAX_STR_LEN` or `<u128 ...>` const from an original
  function (stored as requests, unresolved, no row yet: step 6), 1 starts at the struct
  `Buffer` (OD-8).
- **Disagreement found, not settled:** for the 14 `DECIMAL_PAIRS` sites the expected graph
  anchors the site on `DECIMAL_PAIRS.0` (15 bytes); production sites the path
  `DECIMAL_PAIRS` (13 bytes), as R6 says. Starts agree, ends differ by 2. Part 7 rules on it.
- **Finding for step 5:** `I::MAX_STR_LEN` needed no new lookup. Name resolution already
  treats a type parameter's bounds as glob imports into the parameter's scope
  (`syntax-namespace-input-extraction.rs`, the `TYPE_PARAM | TRAIT` bound-list branch), so
  the stored row resolves to the trait const at bytes 3680 to 3834. What the row lacks is
  the part-3 record that `Copy` (supertrait of `Sealed`) was not read.
- **Not done:** unresolved rows (R5), the read-twice real-source test (R8; the fixture
  helper does compare two reads), SOP-grade cost of one run.
- **Codebase-memory queries:**
  `search_graph --label Function --name-pattern '.*(read_edge|body_read|value_read|body_site_row|read_body).*'`
  (no existing resolver for reads; 2 test helpers);
  `trace_path --function-name make_call_fixture_occurrence --direction inbound` (4 callers, all in the call tests);
  `trace_path --function-name publish_full_graph_checkpoint --direction both`;
  `trace_path --function-name read_type_path_records --direction inbound` (call and signature resolvers);
  `trace_path --function-name read_single_target_identity --direction inbound` (signature resolver);
  `search_graph --label Function --name-pattern '.*(type_param|generic_bound|bound_member|assoc|type_relative).*'`;
  `index_repository` after the three commits.

### 2026-11-15 11:55 IST — Part 4c step 2: value paths in bodies are requested

- REQ-RUST-048.0. A path expression in a function body that is not a callee and is not
  bound by a parameter or pattern in scope is now a name-resolution request in the value
  namespace, context `body_value_path`. No `reads` row is stored yet; the graph is
  unchanged at 58 nodes and 170 edges.
- Measured on pinned itoa (release build, sysroot excluded, 2 workers, 14.0 s, read with
  `export-published-stage-rows --stage-owned-table-name path-resolution-site-records`):
  301 path rows, 70 of them `body_value_path`.

```text
original bodies, 36                       state       names
14  DECIMAL_PAIRS                         resolved    Static
1   I::MAX_STR_LEN                        resolved    Const
7   D EXP M_HIGH SH_POST SIG              resolved    Const declared in the function body
8   OFFSET                                resolved    ConstParam
6   i128::MAX_STR_LEN 1, u128:: 5         unresolved  no reason stored
macro-generated bodies, 34
24  DECIMAL_PAIRS                         resolved    Static
10  Self:: 5, u8 u16 u32 u64 u128:: 1 each  unresolved  no reason stored
```

- Differs from the part as written: the part planned a local-binding flag on every path. Built
  instead: a locally bound name gets no request at all, and items declared in a block and
  generic parameters are left to name resolution, which already has block scopes
  (`test_value_path_block_scope`: for `static S; fn f() { const S: u8 = 2; S }` the request
  is made in the block scope, where only the block const is bound). This test passed on
  first run, so it describes existing behaviour and was not a RED.
- Correction: the 2026-11-15 second-opinion review said this check "can produce a wrong
  edge" for a block-local const. The request-level test above does not support that. The
  end-to-end proof (no `reads` row for such a path) belongs to step 3.
- `test_sysroot_reached_names_reasoned` pinned total path rows; it now counts
  `body_value_path` rows separately and asserts 38 resolved `DECIMAL_PAIRS` requests.
- Tests: `cargo test -p parseltongue-syntax-span-extractor` 79 + 17 passed, 0 failed.
  `cargo test -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`
  22 passed, 0 failed, 1 ignored (`test_decimal_pairs_static_read`, RED until step 4),
  279.0 s. `cargo fmt --all -- --check` and `cargo clippy -p
  parseltongue-syntax-span-extractor -p parseltongue-cli-dispatcher-binary --all-targets --
  -D warnings` clean. Workspace gates not run; they run at step 7.
- codebase-memory queries: `index_status`; `trace_path check_call_lexical_shadowing both 1`;
  `trace_path check_receiver_name_rebound both 1`; `search_graph Function
  '.*(shadow|binding|scope_covers).*'`; `trace_path check_body_call_prefix inbound 1`
  (one caller, the extraction file); `index_repository` after the commits.
- Not done: the two older call-site checks (`check_call_lexical_shadowing`,
  `check_receiver_name_rebound`) are not merged with the new one. The older one still
  treats any `use` in a block as shadowing every name.
- Commits: `9b88de807` (201 lines), `35d163f8c`.
- Next: step 3, `resolve_body_read_edges` on fixtures.

### 2026-11-15 11:00 IST — Process rules and two graph decisions

- `AGENTS.md` Rule 4 (codebase-memory at every step, queries listed in each entry) and
  Rule 5 (GLM-5.3 second opinion at max reasoning): `953b16a3f`, `20fdc9afe`,
  `09e19e8e6`, `eacbfe0f2`. The first review is recorded verbatim in
  `docs/devin-research/1794719045-second-opinion-glm-5-3-max-two-harness-review-of-itoa-open-questions.md`.
- Decided by the user's rule "the more explicit thing for a future coding agent"
  (`c22a3fe32`): a target in `core` is a named external target, as far as itoa's source
  proves the name, with the crate and a source-not-read flag; macro definitions and macro
  calls at item position are nodes. A relation `expands` (call to definition) is proposed,
  not decided. Nothing is built.
- Found by the review and confirmed in source, not yet repaired: the expected graph lists 4
  of 9 `u128::from` sites in `mulhi`; all 70 expected `contains` rows have an empty site;
  9 unresolved calls in original bodies target itoa's own functions and fail on callee
  shape (bare name, module prefix, generic arguments), not on type inference.

### 2026-11-15 09:40 IST — Part 4b built: 32 `implements` rows on pinned itoa

- All 7 steps of part 4b are done (REQ-RUST-047.0). One run stores 58 nodes and 170 edges:
  43 `contains`, 61 `calls` (3 resolved), 34 `uses_type`, 32 `implements`.
- The 32 rows: 12 to `Integer`, 12 to `private::Sealed`, 5 to `Unsigned`, 3 from `Buffer`
  unresolved with `ExternalSourceMissing`. `implements_without_source` is 0.
- 28 rows come from macro-generated headers. Their site is the trait path in expanded text
  with the expansion chain; the frozen store accepts it, so they did not move to part 5.
- The validator accepts a builtin as an edge source for `implements` only.
- Gates at `7e2a2ef87`: format, lint and release build pass; 470 tests pass, 0 fail,
  4 ignored; `test_lang_items_in_sysroot` skipped. The 22 real-source itoa tests pass
  (`cargo test --release -p parseltongue-cli-dispatcher-binary --test
  itoa-source-acceptance-tests -- --test-threads=1`, 232.7 s).
- Cost of one run (3 runs, sysroot excluded, 2 workers, executable sha256
  `5be64c7b…41d825f3`): 13.0 to 13.7 s, 369 to 371 MB peak over the whole process tree
  (sampled about every 18 ms, not the SOP's 10 ms), 9.5 MB store.
- Found: a row's identity includes the workspace's absolute path, so runs in two
  directories do not give equal rows (Q6 of the part).
- Commits: `7c15ddbec`, `e1c144ce8`, `6025dc9a5`, `9b1d33825`, `60d1cc40f`, `802ca8a72`,
  `7e2a2ef87`.
- Next: write part 4c (`reads`) with the executable-spec skill.

### 2026-11-15 06:40 IST — Part 4b specified: `implements` from impl headers

- Part 4b is written before any code: REQ-RUST-047.0, one `implements` row per trait impl
  header, from the self type to the trait.
- Measured on pinned itoa: 33 impl headers. 32 name a trait; 29 resolve both ends and 3
  (`Buffer` to `Default`, `Clone`, `Copy`) have a trait in `core`, so they are unresolved.
- That matches the expected graph's 32 `implements` rows by count.
- One contract change is needed: the validator must accept a builtin source for
  `implements` only (`impl Unsigned for u128`).
- 28 of the 32 headers are macro-generated; their site is in expanded text. Whether the
  frozen store accepts that is open (Q1 of the part).
- Next: step 1, the failing real-source test `test_unsigned_impl_builtin_source`.

### 2026-11-15 06:20 IST — Part 4a step 7: refactor and one run's cost; 4a is built

- Part 4a is built for original signatures: 34 `uses_type` rows on pinned itoa.
- One run costs 14.1 s and 360 MB peak over the whole process tree, with a 9.3 MB store
  (sysroot excluded, 2 workers, executable sha256 `fc23c1c4…35d10932`).
- Memory is resident memory summed over the process tree, sampled about every 17 ms.
- Refactor: a builtin's identity is built directly; a builtin row without a name is a
  typed error.
- Workspace gates on the refactored code: format, lint and release build pass; 457 tests
  passed, 0 failed, 4 ignored. The 17 real-source tests pass (174.0 s).
- My first cost run hung: the measuring script let the stderr pipe fill. I killed it,
  wrote stderr to a file and re-ran. The product was not at fault.
- Next: part 4b, `implements`. The part file is written first.

### 2026-11-15 05:50 IST — Part 4a step 6: unresolved type paths are stored with a reason

- Every type path in an original itoa signature now has one `uses_type` row: 34 rows,
  29 with a target and 5 unresolved. One run stores 51 nodes and 138 edges.
- The 5 unresolved: 4 `ExternalSourceMissing` (`MaybeUninit<u8>` three times, `Copy`
  once) and 1 `TraitSolverNeeded` (`Self::Buffer` in a parameter).
- `test_signature_external_type_unresolved` passes on pinned itoa.
- RED first: three fixture tests and the real-source test failed on behaviour (0 rows
  where 1 or more are expected) before the code changed.
- Workspace gates: format, lint and release build pass; 457 tests passed, 0 failed,
  4 ignored. The 17 real-source tests pass (165.9 s).
- Corrections: 5 unresolved sites, not the 6 predicted. The stage log line now prints
  `nodes=51`.
- R5 in part 4 now names two more reasons the code uses: `CfgInactive` and `MissingTarget`.
- Next: step 7, the refactor pass and one run's cost over the whole process tree.

### 2026-11-15 05:15 IST — Part 4a step 5: builtin types are targets, with one node each

- `private::Sealed::write --uses_type--> str` is stored from pinned itoa, site on `str`;
  `test_write_builtin_type_edge` passes.
- One run now stores 51 nodes and 133 edges: 43 `contains`, 61 `calls`, 29 `uses_type`
  (7 to items, 1 `Self`, 21 to builtins). Six `Builtin` nodes were added, one per name.
- This also confirms step 4's expected count: 8 `uses_type` before builtins.
- RED first: the fixture test found 0 edges where 4 are expected; the real-source test
  found 0 edges to `str` where 1 is expected.
- One failed attempt: the frozen graph did not match its input, because the builtin nodes
  were appended after the node list was sorted. The facade now sorts after adding them.
- Workspace gates: format, lint and release build pass; 454 tests passed, 0 failed,
  4 ignored. The 16 real-source tests pass (151.3 s).
- Known defect: the stage log line prints `nodes=45`; the readback line prints 51.
- No refactor made. Next: step 6, unresolved rows with a reason.

### 2026-11-15 04:45 IST — Part 4a step 4: `Self` in a signature is the impl's self type

- `<Buffer as Clone>::clone --uses_type--> Buffer` is stored from pinned itoa, site on
  `Self`; `test_clone_self_type_edge` passes.
- The resolver reads the self type from the impl header row of the member. `Self` in a trait
  declaration has no header, so it stores nothing.
- RED: the fixture test failed first (0 edges, 3 expected). The real-source test did not
  compile on its first run, so I checked it by disabling the new branch: it failed with
  0 edges where 1 is expected.
- Workspace gates: format, lint and release build pass; 452 tests passed, 0 failed,
  4 ignored. The 15 real-source tests pass (138.3 s).
- Not re-read: the edge counts. 112 edges and 8 `uses_type` are expected.
- Refactor: the resolver's shared inputs moved into one struct, `SignatureEdgeGraphScope`.
- Next: step 5, builtin targets and their nodes.

### 2026-11-15 04:15 IST — Part 4a step 3: `uses_type` edges to items are stored

- `Buffer::new --uses_type--> Buffer` is produced from pinned itoa and read back frozen;
  `test_new_return_type_edge` passes with its `#[ignore]` removed.
- One run now stores 45 nodes and 111 edges (was 104): 7 `uses_type` edges. Calls are
  unchanged at 61 sites, 3 resolved. All 7 are rows of the expected graph (listed in part 4).
- `resolve_signature_type_edges` joins signature rows to the path rows name resolution
  already stored. It looks no name up. A bare `Self`, a builtin and an unresolved path store
  nothing yet (steps 4 to 6).
- RED first: 4 fixture tests failed against a stub that returned no edges; 7 pass now.
- Workspace gates: format, lint and release build pass; 450 tests passed, 0 failed,
  4 ignored. The 14 real-source tests pass (140.9 s).
- Found: `static` and `const` declared types have no signature row, so
  `DECIMAL_PAIRS --uses_type--> DecimalPairs` cannot be produced yet (part 4, Q3).
- No refactor made. Next: step 4, `Self` through the impl header.

### 2026-11-15 03:30 IST — Part 4a step 2: signature rows list their type paths

- Each parameter, return type, field, generic parameter, where-predicate and bound row in
  `field-signature-detail-table` now carries `type_path_source_rows`: range, text and a
  bare-`Self` flag for every type path it holds.
- A path is listed once, on the nearest row that holds it (a bound's path is on the bound
  row, not also on its type parameter). Paths under an expression are not listed. Rows under
  an expression of their item (closure parameters) carry no list.
- RED first: 3 fixture tests selected, 3 failed because no row had a list. GREEN: the
  extractor crate's 58 unit tests and 17 contract tests pass.
- Known limit: `Foo<LEN>` with a const `LEN` parses as a type argument, so it is listed as
  a type path. Step 6 will store it unresolved, not as an edge.
- Workspace gates: format, lint and release build pass; 445 tests passed, 0 failed,
  5 ignored; `test_lang_items_in_sysroot` skipped as before.
- No refactor needed: one new function of about 30 lines.
- Next: step 3, the resolver for item targets and the facade call.

### 2026-11-15 03:00 IST — Part 4a step 1: RED on the first `uses_type` edge

- `test_new_return_type_edge` (REQ-RUST-018.2) runs the production CLI on pinned itoa and
  asks for one `uses_type` edge from `Buffer::new` to the struct `Buffer`, with the site on
  the `Buffer` of `-> Buffer`.
- RED, 1 test selected, 12.9 s: it reached the assertion and found 0 `uses_type` edges from
  `Buffer::new` (expected 1). Nothing produces that edge kind.
- The test is committed with `#[ignore]` so the workspace test gate stays green; step 3
  removes the attribute. It was run without the attribute for the RED result above.
- Workspace gates with the test ignored: format, lint and release build pass; 442 tests
  passed, 0 failed, 5 ignored (the four from before plus this one);
  `test_lang_items_in_sysroot` skipped as before.
- Next: step 2, type-path ranges on signature rows.

### 2026-11-15 02:40 IST — Part 4a specified: `uses_type` from signatures

- Part 4 is split into 4a (`uses_type` from signatures, REQ-RUST-018.2), 4b `implements`,
  4c `reads`, 4d `imports`. Only 4a is written out; the rest stay stubs.
- Read from a run on pinned itoa before writing: name resolution already stores a resolved
  row for each type path in a signature (`Buffer`, `Self`, `u8`, `Self::Buffer`), and
  signature rows exist per parameter, return type, field and bound. No producer joins them.
- 35 type-path sites in original signatures: 9 name items, 21 builtins, 5 are unresolved.
- `<Buffer as Clone>::clone --calls--> Buffer::new` was already stored (one of the first two
  resolved calls), so the first target is `Buffer::new --uses_type--> Buffer` instead.
- One rule found in code: an edge's target must be a node, so builtin targets need
  `Builtin` nodes (part 4, Q1).
- Next: step 1, the failing real-source test `test_new_return_type_edge`.

### 2026-11-15 02:00 IST — Part 3 closed for one receiver shape; workspace gates run

- Gates at `b1115a5f4`: format and lint pass; 442 tests pass, 0 fail, 4 ignored, in 3 min 39 s
  with `test_lang_items_in_sysroot` skipped as before; release build passes.
- Steps 2 to 5 had been pushed after gates on the two touched crates only.
- Part 3 lists what it leaves open: type parameters of an enclosing impl or trait,
  `where Self` bounds, bounds with generic arguments, a rename test on real source.
- Next: write part 4 with the executable-spec skill, then its failing real-source test.

### 2026-11-15 01:30 IST — Part 3 step 5: resolver fixture tests

- Ten fixture tests on a small bound-method graph (`make_bound_fixture_graph`): supertrait
  method through inline and `where` bounds, two candidates, same-name distractors, rebound
  receiver, other receiver shapes, cyclic supertraits, missing source with and without a
  found target, no candidate, renamed traits and method. Extractor crate: 55 + 17 pass.
- They were written after the code, so none has a failing run. As a substitute, four
  one-line breakages of the resolver were tried and each made exactly one test fail:
  rebinding check removed, type-parameter-owner check removed, method name ignored, visited
  set ignored. A fifth (missing source blocks the edge) did not compile, so it shows nothing.
- No direct breakage was tried for the two-candidate or distractor tests.
- Next: refactor pass, workspace gates, then close part 3.

### 2026-11-15 01:00 IST — Part 3 step 4: GREEN, `Buffer::format` calls `private::Sealed::write`

- `test_format_sealed_write_call` passes on pinned itoa. The stored edge targets the trait
  method, with no fan-out to implementations.
- Its dispatch record: kind `generic_bound_trait_method`, bound chain
  `I -> Integer -> private::Sealed`, and `missing_source_traits: ["Copy"]`, because this run
  excludes the sysroot source. This is the "proven, gap" state decided for Q1.
- Measured in that run: 45 nodes, 104 edges, 61 call sites, 3 resolved (was 2), 58 unresolved.
  17.3 s wall; largest single process 355 MB peak (not a process-tree figure). These were
  taken just before the call site in `build_original_call_edge` was reshaped to shrink the
  diff; the counts were not re-read after it.
- On the committed code: all 13 real-source itoa tests pass (113.8 s), extractor crate
  45 + 17 pass, format and lint clean for the two crates touched.
- Supported: a receiver that is a plain function parameter whose declared type is a type
  parameter of that same function. A type parameter of the enclosing impl or trait, and
  every other receiver shape, stays unresolved with `TraitSolverNeeded`.
- Not done yet for part 3: the resolver-level fixture tests (ambiguous, distractor, rebinding,
  cycle, missing source with no candidate, rename), the refactor pass, and the workspace
  gates (full test run, release build).

### 2026-11-15 00:30 IST — Part 3 step 3: bound evidence, without a new table

- Every `TYPE_BOUND` row in `field-signature-detail-table` now carries
  `bound_syntax_evidence`: what is bounded (a type parameter by name, the trait itself as a
  supertrait, or a `where` subject) and the trait path with its source range.
- Plan change: part 3 first called for a new table. The rows already existed, one per bound,
  owned by their function or trait, so the evidence went onto them. Part 3 is updated.
- Real-source test `test_generic_bound_rows_written` passes on pinned itoa: one row each for
  `I: Integer`, `Integer: private::Sealed`, `Sealed: Copy`. One fixture test passes.
- Not covered: `where Self: Trait` (no subject name is recorded), bounds on associated types.
- `test_format_sealed_write_call` is still RED: the resolver does not read any of this yet.
- Next: step 4, the resolver branch.

### 2026-11-15 00:05 IST — Part 3 step 2: method-call syntax evidence

- Every `METHOD_CALL_EXPR` site row now carries `method_receiver_evidence`: whether the
  receiver is a plain parameter name, the method name, the parameter's declared type when it
  is one plain name, whether that name is a type parameter of the function or of its impl or
  trait, and whether the receiver name may be rebound.
- Rebinding is conservative: any binding or local item of that name anywhere in the body, or
  an earlier macro call that is handed the name, sets the flag.
- Syntax only. The resolver does not read the new record yet, so no edge changes and the
  real-source test is still RED.
- 4 new fixture tests pass (crate: 44 + 17 pass; format and lint clean for the crate). They
  were written together with the code, so no separate failing run is recorded for them.
- Next: step 3, bound and supertrait rows.

### 2026-11-14 23:40 IST — Part 3 step 1: RED on the generic call

- Test `test_format_sealed_write_call` (REQ-RUST-042.0) added. It runs the production CLI on
  pinned itoa and fails at the intended assertion: 1 test selected, 0 passed, 1 failed, 13 s.
- What it shows: the `i.write(unsafe { &mut *buf_ptr })` site is stored once, as a `calls`
  edge from `Buffer::format`, unresolved with `TraitSolverNeeded`. The trait method
  `private::Sealed::write` is already a graph node.
- Counts re-measured on this run, unchanged after the macro commits: 45 nodes, 104 edges,
  61 calls, 2 resolved. Peak memory of the test process tree's largest process: 362 MB.
- Decision recorded the same day: an edge whose chain has a trait without source is stored
  and names the gap (PRD, Decisions in force; part 3, R9).
- Next: step 2, method-call syntax evidence in `extract_original_call_syntax`.

### 2026-11-14 23:10 IST — SPEC is a folder; part 3 specified, not built

- The SPEC is now `SPEC-itoa-full-item-graph/`: an index plus one part per target in build
  order. Each part is written with the rust-executable-specs skill when it is picked.
- Part 3 (`Buffer::format --calls--> private::Sealed::write`, REQ-RUST-042.0) is written:
  eight rules, design, ten tests named, six build steps. No code yet.
- Read in the code: the `i.write` site is already a `METHOD_CALL_EXPR` row; the resolver only
  handles path calls. Missing: method-call syntax evidence, bound rows, one resolver branch.
- Checked with rustc 1.99.1: a type parameter's bound methods win over other traits in scope.
- Waiting on one decision: `Sealed: Copy` reaches `core`, so uniqueness of `write` is not
  fully provable without the sysroot (part 3, Q1).
- Next: step 1 of part 3, the failing real-source test `test_format_sealed_write_call`.

### 2026-11-14 22:45 IST — Documents cut to the itoa full item graph

- Scope fixed: itoa only, full item graph only. Other repositories, rust-analyzer export,
  oracle comparison and higher-abstraction tables are out of this phase.
- Oracle harness deleted. Target dataset cut to the full graph and renamed; its 16 tests pass.
- Three macro commits from 2026-11-13 merged onto this branch.
- Re-run at `15643ec4f`: `cargo test --release -p parseltongue-cli-dispatcher-binary --test
  itoa-source-acceptance-tests -- --test-threads=1`, 11 passed, 0 failed, 104 s.
- No production code changed. Next target unchanged: `Buffer::format -> private::Sealed::write`.

### 2026-11-14 20:24 IST — Baseline: everything before this log was compressed

- First call proven from source: `<Buffer as Default>::default --calls--> Buffer::new`.
- Graph stored and read back frozen: 45 nodes, 43 `contains`, 61 `calls` (2 resolved, 59
  unresolved with a reason). All 33 impl headers produced.
- Bounded workspace tests: 415 passed, 4 ignored, 1 excluded (`test_lang_items_in_sysroot`).
- One run: 14.8 s, 411 MB peak for the whole process tree.
- Decisions taken on the way: the full graph is built first and anything higher is a later
  query (2026-11-13); one VM only, worker sessions archived (2026-11-14).
- The run reports behind these numbers were removed from the tree; the last commit that has
  them is `a2e13a3bd`. Earlier entry-by-entry detail is in this file's git history.
