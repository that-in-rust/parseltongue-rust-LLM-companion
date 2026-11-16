# LOG — progress on the itoa full item graph

Two parts. **Current state** is rewritten in place whenever it changes. **Entries** is
newest first: add each new checkpoint at the top of that section and never edit older ones.

## Current state (2026-11-16)

**The pipeline runs end to end on pinned itoa 1.0.18 and stores a bounded graph: 103 nodes
(45 written by macro expansions) and 585 edges (88 `contains`, 45 `generates`, 15 `expands`, 106 `uses_type`: the declared type of a const or static and the aliased type of a type alias are sites since part 5d step 10c), with 43 of 197 calls resolved (2 `(self as u64/i64).write(buf)` to the impl member for the cast's type, part 5d step 9; 27 by the callee path, part 5d step 6; 5 `Unsigned ::fmt(self, ..)` to the impl member for the calling method's type, step 8; 5 `Unsigned ::fmt(self.unsigned_abs(), ..)` to the trait's declaration and 1 `DecimalPairs(..)` to the struct, step 7; the bracketed breakdown that follows was measured before step 6) (136 of the 197 start in macro-generated function bodies; the 194 without a target: 178 `TraitSolverNeeded`, 10 `MissingTarget`, 6 `ExternalSourceMissing`), 34 `uses_type` rows (29 with a target, 5 unresolved),
32 `implements` rows (29 with a target, 3 unresolved), 98 `reads` rows (68 with a
target; 76 of the 98 are sited in macro-expanded text; 20 are value paths in the arguments of `assert!` since part 5d step 11; consts, statics, type aliases and structs are sources since part 5d step 10a) and 4 `imports` rows (all unresolved: their targets are in `core`).
Five dependencies
are proven from source by name: `<Buffer as Default>::default --calls--> Buffer::new`,
`Buffer::format --calls--> private::Sealed::write`, `Buffer::new --uses_type--> Buffer`,
`u128 --implements--> Unsigned` and `enc_16lsd --reads--> DECIMAL_PAIRS`. No SOP step has passed and the graph is marked
`incomplete`.**

- **Part 6 (named `core` targets), measured 2026-11-16:** steps 6.1 to 6.5 are built.
  The graph is 145 nodes and 585 edges; 42 nodes are written names outside itoa with no file,
  kind or signature (8 paths into `core`, 3 prelude names, 31 members of builtin types). All
  4 `imports` rows, all 32 `implements` rows, all 106 `uses_type` rows, all 98 `reads` rows
  and 59 of 197 calls have a target; 138 rows have none, all of them method calls whose
  receiver type is not proven.
  The review of steps 6.2 to 6.5 is done; running its untested cases found one wrong-name
  case (a default member of a trait declared in a function body), now withheld. No itoa row
  changed.
  Workspace gates at commit `1044be117`: format, lint and release build pass; 591 tests
  pass, 0 fail, 4 ignored, with `test_lang_items_in_sysroot` skipped. That test failed when
  last run (at `f600a7954`), so the gate is not a full PASS.
  Next is part 7 (review and freeze the expected graph). Numbers above
  this line that say 103 nodes, 43 calls, 68 reads with a target or 3 unresolved
  `implements` are from before part 6.
- **Earlier target:** the 5d workspace gates, then part 6 (named `core` targets). Steps
  10b, 10c and 11 are built (40 new `uses_type` rows; 4 qualified-path rows on the impl
  member; 20 `reads` rows from `assert!` arguments).
  Part 5d step 9 is built: the two cast-receiver calls have a target (part 5 Q34 closed).
  Part 5d step 8 is built: the five `Unsigned ::fmt(self, buf)` calls target
  `<uN as Unsigned>::fmt`, the member of the impl for the calling method's type, and keep
  the trait's declaration and the candidate list on the row (part 5 Q27 closed).
  Part 5d step 7 is built: the 10 generated `Unsigned ::fmt(..)` calls target the trait's
  method declaration and list the five impl members that could run; `DecimalPairs(..)`
  targets the struct.
  Part 5d step 6 is built: the whole callee path of a call is sent to name lookup, and the
  27 calls to this crate's own free functions (`divmod100`, `slice_buffer_to_str`,
  `div_rem_1e16`, `enc_16lsd`, `u128_ext::mulhi`) have a target.
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
6  targets in core                     6.1-6.5 BUILT 42 named external nodes; 138 method calls have no target
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

### 2026-11-16 15:10 IST — Part 7 step 7.2b: macro definitions, macro calls, generates and expands in the expected graph

- **Milestone:** itoa full item graph, part 7 (REQ-RUST-044.0). No SOP step has passed; the
  expected graph is still an unreviewed draft.
- **Changed:** the dataset builder, its rules file (two relation kinds added; a macro call
  has no visibility), one unit test, and the files the builder writes. No production code.
- **Added from the pinned source:** 3 macro definition nodes (lib.rs 135-183, 191-209,
  259-327), 15 macro call nodes (lib.rs 185-189, 211-216, 329-332), 15 `expands` rows, 45
  `generates` rows (one per generated item) and 18 `contains` rows. The four unresolved
  sites at lib.rs 211-214 now say the call is active and its generated impls are inactive.
- **Command:** the builder as in the entry below, then the unit tests: `Ran 16 tests`, `OK`.
  Builder output: `candidate_nodes: 91`, `candidate_edges: 451`.
- **Measured against production rows** (throwaway script, one row per distinct source,
  relation and target):

```text
NODES expected 91 production 145 both 90
 expected only: ['itoa']
 production only: 55 Counter({'ExternalDefinition/None': 42, 'Builtin/None': 13})
EDGES expected rows 451 distinct 451 | production rows 585 distinct 413
 calls       expected  103 production   66 both   38 expected-only   65 production-only   28 (production-only unresolved 23)
 contains    expected   88 production   88 both   88 expected-only    0 production-only    0 (production-only unresolved 0)
 expands     expected   15 production   15 both   15 expected-only    0 production-only    0 (production-only unresolved 0)
 generates   expected   45 production   45 both   45 expected-only    0 production-only    0 (production-only unresolved 0)
 implements  expected   32 production   32 both   32 expected-only    0 production-only    0 (production-only unresolved 0)
 imports     expected    4 production    4 both    4 expected-only    0 production-only    0 (production-only unresolved 0)
 reads       expected   62 production   62 both   62 expected-only    0 production-only    0 (production-only unresolved 0)
 uses_type   expected  102 production  101 both  101 expected-only    1 production-only    0 (production-only unresolved 0)
```

- **Open:** `calls` (65 expected rows without a production match: 60 method calls into
  `core` and 5 `Unsigned::fmt(self.unsigned_abs(), ..)`; wait on part 7 Q1); one
  `uses_type` row per body type (2 sites); the crate-root node `itoa` (Q2); the external and
  builtin nodes are not yet rows of the nodes file.
- **Next:** external and builtin node rows, then the comparison test in Rust.

### 2026-11-16 14:40 IST — Part 7 step 7.2a: expected graph re-cut for names, repeated sites and site starts

- **Milestone:** itoa full item graph, part 7 (REQ-RUST-044.0). No SOP step has passed; the
  expected graph is still an unreviewed draft.
- **Changed:** `itoa-expected-graph-builder.py` and the files it writes. No production code.
  Every change is from the pinned source or a recorded decision, not from production output:

```text
change                                                        basis
core::num::<impl T>::MAX/MIN/BITS -> T::MAX/MIN/BITS (30)     PRD, written-name rule (2026-11-15)
core::default::Default, ::Copy, ::Clone -> Default, Copy,     same rule; lib.rs 76, 83, 86, 128
  Clone (4 rows)
<u128 as core::convert::From<u64>>::from -> u128::from;       same rule; u128_ext.rs 14 (2), 15 (2),
  9 sites, was 4 lines                                          19 (3), 21 (2)
repeated same-line sites counted on the pinned line           u128_ext.rs 7 (u128 x3); lib.rs 231
                                                                (u32 x3), 450 (u128 x2)
DECIMAL_PAIRS --calls--> DecimalPairs                         lib.rs 222; PRD, tuple struct constructor
<$primitive as private::Sealed>::Buffer also names the type   lib.rs 200
  $primitive and the trait private::Sealed (4 rows)
each site records where its first token is written: in the    lib.rs 138-140, 144, 160, 169
  macro definition, or in the macro call when the site starts
  with a macro parameter
```

- **Command:** `python3 docs/phase-20261113/itoa-target-dataset/itoa-expected-graph-builder.py --source-root /home/ubuntu/repos/rust-tools/itoa --output-directory docs/phase-20261113/itoa-target-dataset`,
  then `python3 -m unittest docs/phase-20261113/itoa-target-dataset/test_itoa_expected_graph_rules.py`:
  `Ran 16 tests`, `OK`.
- **Measured against production rows** (throwaway script, one row per site, keyed by source,
  relation, target, file, line of the first token, and whether that token is in the macro
  definition or the macro call):

```text
SITES expected 509 production 585 matched 273 | production rows with no source line: {'contains': 45, 'implements': 28}
 calls       expected  197 production  197 matched   54
 contains    expected   70 production   88 matched    7
 expands     expected    0 production   15 matched    0
 generates   expected    0 production   45 matched    0
 implements  expected   32 production   32 matched    4
 imports     expected    4 production    4 matched    4
 reads       expected   98 production   98 matched   98
 uses_type   expected  108 production  106 matched  106
```

- **Reading:** `reads` and `imports` match site by site. `uses_type` differs by the two
  expected rows for a type written in a function body (lib.rs 107 and 252), which production
  does not make. `contains` and `implements` match by source, relation and target (70 of 70
  and 32 of 32); most of their production rows carry no source line, so the per-site count
  above is low for a reason of the script, not of the rows.
- **Source fact found:** all six `impl_Integer_size!` calls (lib.rs 211-216) are active
  calls. The `#[cfg(target_pointer_width = ..)]` is an argument and applies to the generated
  impls. The draft's four "cfg-inactive item macro" unresolved sites are wrong as written.
- **Not done:** macro definition and call nodes, `generates` and `expands` rows, external
  and builtin nodes, method calls (wait on part 7 Q1), the blocker rules, byte-range
  equality (the comparison is by line).
- **Next:** macro nodes and their rows in the builder.

### 2026-11-16 13:20 IST — Part 7 started: first comparison of the draft expected graph with production rows

- **Milestone:** itoa full item graph, part 7 (REQ-RUST-044.0). No SOP step has passed.
  The expected graph is an unreviewed draft; this is a measurement, not a comparison that
  can pass.
- **Command:** `python3 /home/ubuntu/phase7-work/compare.py` (a throwaway script outside the
  repository) on the node and edge exports of the release run `tfix-3` (executable sha256
  `82eab262…`, commit `a8fd18503`). Rows are keyed by source label, relation and target label.
- **Output:**

```text
NODES expected 73 production 145 both 72
 expected only: ['itoa']
 production only: 73 Counter({'ExternalDefinition/None': 42, 'LocalItem/MacroInvocation': 15, 'Builtin/None': 13, 'LocalItem/MacroDefinition': 3})
EDGES expected rows 368 distinct 368 | production rows 585 distinct 413
 calls       expected  102 production   66 both   36 expected-only   66 production-only   30 (production-only unresolved 23)
 contains    expected   70 production   88 both   70 expected-only    0 production-only   18 (production-only unresolved 0)
 expands     expected    0 production   15 both    0 expected-only    0 production-only   15 (production-only unresolved 0)
 generates   expected    0 production   45 both    0 expected-only    0 production-only   45 (production-only unresolved 0)
 implements  expected   32 production   32 both   29 expected-only    3 production-only    3 (production-only unresolved 0)
 imports     expected    4 production    4 both    4 expected-only    0 production-only    0 (production-only unresolved 0)
 reads       expected   62 production   62 both   32 expected-only   30 production-only   30 (production-only unresolved 0)
 uses_type   expected   98 production  101 both   96 expected-only    2 production-only    5 (production-only unresolved 0)
```

- **Reading of the 101 unmatched expected rows** is in the part 7 file, section 1: 35 differ
  in the name of a `core` target, 60 are method calls into `core`, 5 are
  `Unsigned::fmt(self.unsigned_abs(), ..)`, 1 is a production miss (a type written in a
  function body).
- **Decision asked of the user, not answered:** part 7 Q1 (a run marked incomplete cannot
  pass, against expected unresolved rows).
- **Next:** step 7.1, the comparison test.

### 2026-11-16 12:40 IST — Part 6 workspace gates

- **Milestone:** itoa full item graph, part 6. No SOP step has passed. The graph is still
  marked incomplete. This is not a full workspace PASS (one test is skipped and fails when
  run).
- **Code under test:** commit `1044be117` (the review fix). Log: `/home/ubuntu/phase6-work/ws-gates-4.log`.
- **Commands and results** (`CARGO_BUILD_JOBS=2`):
  - `cargo fmt --all -- --check`: exit 0.
  - `cargo clippy --release --workspace --all-targets --all-features -- -D warnings`: exit 0.
  - `cargo test --release --workspace --all-targets --all-features -- --skip test_lang_items_in_sysroot --test-threads=1`:
    exit 0; 21 test binaries, 591 passed, 0 failed, 4 ignored. The itoa real-source
    binary: `test result: ok. 53 passed; 0 failed` in 824.20 s (single-threaded).
  - `cargo build --release --workspace --all-targets --all-features`: exit 0.
- **Skipped test:** `test_lang_items_in_sysroot`. Run without the skip at `f600a7954`
  (before the review fix) it failed with `StoreBackendFailureError`, "Turso/DuckDB
  row-count or content-digest mismatch". Not rerun on this commit; not fixed. It reads the
  native sysroot, which itoa runs exclude.
- **Executable digests:** the all-features build is
  `0addedfc8e454b913ba4170ce853f5eb5601d4b621fe48c6f1d76915e6fa3756`; the default build
  (`cargo build --release -p parseltongue-cli-dispatcher-binary`), used for the timed
  runs, is `82eab2623b928d9a1a5abd673d99c7440752ae3fda091a673d1026663cae7182`.
- **Earlier gate run at `f600a7954`** (`ws-test-2.log`, `ws-gates-3.log`): the same
  commands, 590 passed, 0 failed, 4 ignored; lint and build exit 0.
- **Not done in part 6:** the 138 method calls (N6), and part 6 Q1 to Q10.
- **Next:** part 7, review and freeze the expected graph against the stored rows.

### 2026-11-16 12:05 IST — Part 6 review fix: a body-declared trait's default member no longer gets an external name

- **Milestone:** itoa full item graph, part 6. No SOP step has passed. The graph is still
  marked incomplete.
- **REQ:** REQ-RUST-053.0 N5 (a fifth condition added to the part file before the code).
- **Defect, found by running code after the GLM-5.3 review of steps 6.2 to 6.5:** a trait
  declared in a function body with a default member, plus an impl of it that declares no
  member, gave a builtin-member name to a local member. Three rows on a small crate that
  compiles (`/home/ubuntu/phase6-work/probe-gaps3/p/src/lib.rs`): `u8::FOO2`, `u16::mk`,
  `u64::FOO5` were stored as external names. The review had judged this case covered.
  No itoa row was wrong: itoa has no impl in a function body.
- **Cause:** the impl declares no member of that name, so the lookup says
  `no_stored_impl_member`; a trait in a function body is not a node, so its member names
  are not in the withheld set.
- **Fix:** `ImplMemberLookupIndex::has_unresolved_trait_impl`. `find_builtin_member_name`
  returns `external_member_trait_not_resolved` when a stored trait impl whose self type is
  this builtin or was not resolved is of a trait that was not resolved.
- **Measured on the way:** the stored header of an impl in a function body has neither its
  self type nor its trait resolved (`impl Loc for u8`: self `unknown_receiver`). One such
  impl withholds every builtin-member name in the crate (part 6 Q6).
- **STUB:** none; the change is one condition in an existing function.
- **RED** (tests written, code unchanged):
  `cargo test --release -p parseltongue-syntax-span-extractor --lib test_builtin_member_external_name`:
  `test result: FAILED. 0 passed; 1 failed`, left `Some(Ok(.. "u8::MAX" ..))`, right
  `Some(Err("external_member_trait_not_resolved"))`.
  `cargo test --release -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests test_external_name_shadow_cases`:
  `test result: FAILED. 0 passed; 1 failed`, left `("u16::mk", "external:u16::mk", "")`,
  `("u8::FOO6", "external:u8::FOO6", "")`.
- **GREEN:** `cargo test --release -p parseltongue-syntax-span-extractor`:
  `test result: ok. 134 passed; 0 failed` and `ok. 17 passed`.
  `cargo test --release -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`:
  `test result: ok. 53 passed; 0 failed` (144.57 s). Clippy with `--all-features -D warnings`
  on the two crates and `cargo fmt --all --check`: exit 0.
- **The first GREEN run of the pipeline test failed twice on my expectations, not on the
  code:** an impl in a function body in the first small crate now withholds `u8::MIN` and
  `u8::BITS` there. I moved the body-impl cases to the second crate.
- **REFACTOR:** no refactor needed.
- **New test `test_external_name_shadow_cases`** (three small crates, each checked with
  `cargo check`, run through the whole pipeline; 17 rows asserted). Its first version (10
  rows, before the defect was found) was written after the code and passed on first run.
- **Deliberate breaks:** 7, each failed a test. Removing the local-trait check and making
  the lookup gate always true each fail the pipeline test; five breaks of
  `has_unresolved_trait_impl` (inherent impl counts; any self type; unresolved self not
  counted; same type not counted; resolved trait counts) each fail
  `test_builtin_member_external_name`.
- **Measured on pinned itoa after the fix (release, three runs `tfix-1..3`, executable
  sha256 `82eab2623b928d9a1a5abd673d99c7440752ae3fda091a673d1026663cae7182`):** unchanged,
  145 nodes, 585 edges, 59 of 197 calls with a target; 15.27 to 16.67 s wall, 343 to 360 MB
  peak resident set, 18.8 MB store.
- **Review record:**
  `docs/devin-research/1794804575-second-opinion-glm-5-3-max-part-6-steps-two-to-five.md`.
  Not built: its test gaps 1 and 3 (part 6 Q7, Q8).
- **Next:** workspace gates on this code.

### 2026-11-16 10:18 IST — Part 6 step 5: members of builtin types (`u128::from`, `uN::MAX`, `MIN`, `BITS`) target named external nodes

- **Milestone:** itoa full item graph, part 6 step 6.5. No SOP step has passed. The graph is
  still marked incomplete.
- **REQ:** REQ-RUST-053.0 N3, N4, N5, N6.
- **Measured on pinned itoa (release run, `/home/ubuntu/phase6-work/s65` and three timed runs
  `t65-1..3`, executable sha256
  `44e44693e66978ffbc2c1e14559e199da2c8410514dacc849e7721a4280888a8`):** 145 nodes (was 114),
  585 edges (unchanged), 59 of 197 calls with a target (was 50), 98 of 98 `reads` with a
  target (was 68), 138 rows without a target (was 177), all `calls` rows of shape
  `method_call`. As predicted in the part file. Stage line:
  `stage=full-item-dependency-graph nodes=145 containment=88 generates=45 expands=15 calls=197 resolved_calls=59 type_uses=106 implements=32 implements_without_source=0 reads=98 reads_in_expanded_bodies=76 imports=4 ... completeness=incomplete`.
- **The 39 rows (read back from the stored edge table):** 9 `calls` `u128::from(..)`
  (`dispatch_kind` `external_builtin_member`, all to the one node `u128::from`); 10 `reads`
  of `uN::MAX` / `iN::MAX` (`body_value_path`); 20 `reads` of `MIN` and `BITS`
  (`macro_argument_value_path`). 31 distinct names, one node each.
- **Rule as built:** in the part file, "Step 6.5 as built". The name is produced from the
  member lookup over stored impl headers (`find_builtin_member_name` on
  `ImplMemberLookupIndex`), gated on the 19 builtin names the graph has nodes for
  (`is_hld_builtin_type`, now public in the core crate).
- **Also changed:** `read_impl_member_index` now takes the node rows and fills the names
  local traits declare, so its three callers (reads, calls, signatures) build the index the
  same way. Before, the reads producer had an empty set.
- **STUB:** `find_builtin_member_name` as `todo!("REQ-RUST-053.0 N3")`, wired into both
  producers; `cargo check --release -p parseltongue-syntax-span-extractor --all-targets`
  finished with one unused-import warning.
- **RED:** `cargo test --release -p parseltongue-syntax-span-extractor --lib`:
  `test result: FAILED. 128 passed; 5 failed`, each `not yet implemented: REQ-RUST-053.0 N3`:
  the new `test_builtin_member_external_name` and `test_builtin_member_call_target`, and
  three existing tests that reach the stub (`test_qualified_callee_needs_module`,
  `test_preserve_absent_target_reason`, `test_preserve_missing_member_reason`).
- **GREEN:** same command: `test result: ok. 134 passed; 0 failed` (+17 contract tests).
  `cargo test --release -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`:
  `test result: ok. 52 passed; 0 failed` (136.94 s) on the final code.
- **REFACTOR:** one type alias (`BuiltinMemberNameOutcome`) for a clippy
  `type_complexity` error; nothing else.
- **Existing tests changed (they pinned the old values):** fixture
  `test_qualified_callee_needs_module` (a builtin qualifier with no stored reason now gives
  `external:str::helper`); real source `test_generated_fmt_static_reads`,
  `test_generated_body_call_rows`, `test_item_initializer_read_rows`,
  `test_macro_argument_read_rows`, `test_itoa_external_path_calls`, and the node count (114
  to 145) and open-row count (177 to 138) in six tests.
- **Weak evidence:** `test_itoa_builtin_member_targets` and `test_builtin_member_read_target`
  were written after the code and passed on first run.
- **Deliberate breaks:** 18, log `/home/ubuntu/phase6-work/mut65.log`. 14 failed a fixture
  test at once. Four changed no outcome and no test: the two extra guards on the call
  (`stopped == "member_lookup_unproven"`, reason `MissingTarget`), the lookup kind `FN`
  against `CONST`, and the guard "a stored target wins" on the read. Read from
  `find_admitted_member_target`: the outcome `no_stored_impl_member` means zero candidates,
  and the kind only chooses a target among candidates, so none of the four can change a
  row. The guards stay as written intent.
- **Not covered on real source:** a local trait that declares `MAX`, `MIN`, `BITS` or
  `from`, a stored impl of a builtin that declares one, and `Self::NAME` in an impl for a
  builtin with no stored member. itoa has none; fixtures cover the first two.
- **Known limits, stated on each row:** the name is the written one. `u128::from` does not
  say which `From` impl runs, and `u8::MAX` does not say it is the inherent const in `core`.
- **Gates:** `cargo fmt --all --check`; clippy `-D warnings` on core-row-types,
  syntax-span-extractor and cli-dispatcher-binary (all targets). Three timed runs:
  15.22 / 14.77 / 14.79 s wall, 342 / 339 / 351 MB peak resident set, 18.3 MB store.
  Workspace gates: not run yet.
- **Slip:** the real-source test and the read fixture were written after the code.
- **Next:** part 6 workspace gates (all tests, clippy on the workspace, release build),
  codebase-memory re-index, and the GLM-5.3 review of steps 6.2 to 6.5.

### 2026-11-16 09:43 IST — Part 6 step 4: prelude names `Clone`, `Copy`, `Default` target named external nodes

- **Milestone:** itoa full item graph, part 6 step 6.4. No SOP step has passed. The graph is
  still marked incomplete.
- **REQ:** REQ-RUST-053.0 N1, N3, N4, N5.
- **Measured on pinned itoa (release run, `/home/ubuntu/phase6-work/s64b`, executable sha256
  `394c1659…7c08`):** 114 nodes (was 111), 585 edges (unchanged), 32 of 32 `implements` rows
  with a target (was 29), 106 of 106 `uses_type` rows with a target (was 105), 177 rows
  without a target (was 181: 147 calls, 30 reads). As predicted in the part file.

```text
row         written in                    name      target node
implements  impl Clone for Buffer         Clone     Clone
implements  impl Copy for Buffer          Copy      Copy
implements  impl Default for Buffer       Default   Default
uses_type   pub trait Sealed: Copy        Copy      Copy   (the node of the row above)
```

- **The node is the written name, not `core::clone::Clone`:** nothing read proves which
  item the prelude gives that name. Each row says `implicit_prelude_name`, source not read,
  defining path not claimed.
- **Cause found for the bound `Copy` (measured in the stored lookup tables):** the lookup
  input stores a bound's request for its trait beside the `use` requests, as a glob
  (4 rows on itoa: `Integer: private::Sealed`, `Sealed: Copy`, `I: Integer`,
  `Unsigned: Integer`). The proof counted each as a glob import of its scope, so the trait
  scope of `Sealed` failed "no glob import on the chain" for its own bound. A bound puts no
  name in lexical scope.
- **What changed:**
  - `namespace-final-path-products.sql`: `glob-import-scope-records` and the import half of
    `scope-name-binder-records` read only `request_kind='import'` rows.
  - `impl-header-source-records.rs`: an unresolved header resolution keeps the lookup's
    proved name as `external_name_record`.
  - `impl-header-implements-records.rs`: `read_header_external_name`; a row left unresolved
    with `ExternalSourceMissing` and a proved name is `Extracted` to that name, with
    `external_target_proof_record` on the row and the name as the evidence target.
  - No change for `uses_type`: the step 6.3 producer reads the new proof row.
- **RED (SQL):** `cargo test --release -p parseltongue-duckdb-analysis-adapter --lib test_external_name_proof_rows`:
  `FAILED. 0 passed; 1 failed` (`left: Null`, right the `core::mem` name) on the new bound
  case. **GREEN:** adapter crate `185 passed; 0 failed`.
- **Measured between the two changes (`s64a`):** 112 nodes; 35 stored lookups with a name
  (was 34). The only new one is the bound `Copy`.
- **STUB:** `read_header_external_name` as `todo!()`, wired in; compiled.
- **RED (producer):** `cargo test --release -p parseltongue-syntax-span-extractor --lib`:
  `test_external_trait_names_target ... FAILED` (`not yet implemented: REQ-RUST-053.0 N1`),
  with the other impl-header tests, which reach the stub too.
- **GREEN:** syntax crate `131 passed; 0 failed`;
  `cargo test --release -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`:
  `51 passed; 0 failed` (163.84 s), with the new `test_itoa_prelude_name_targets`.
- **Existing real-source tests that pinned the old values (7):** the first run after the
  code gave `43 passed; 7 failed`. `test_buffer_clone_trait_unresolved` is renamed
  `test_buffer_prelude_traits_named`; `test_itoa_implements_row_counts`,
  `test_itoa_external_type_uses`, `test_itoa_external_path_calls` and three node counts
  (111 to 114) are updated.
- **One wrong expectation of mine:** the new test first expected the source labels `Buffer`
  and `private::Sealed`; the stored labels are `itoa::src/lib.rs::Buffer` and
  `itoa::src/lib.rs::private::Sealed`. The test pins the stored ones.
- **Deliberate breaks (8), each fails a test:** SQL glob filter off; SQL binder filter off;
  any unresolved reason accepted; proof not on the row; evidence target withheld; a
  malformed stored name swallowed; stored state not checked; the header row not carrying
  the name (fails the real-source test only).
- **Weak evidence:** `test_itoa_prelude_name_targets` was written after the code. The
  fixture test of the header-row field is the real-source test; no fixture covers it.
- **REFACTOR:** none needed.
- **Not covered:** N5 for a prelude name on real source (a local item, an import or a glob
  import that could supply `Clone`); the SQL fixture test covers the lookup, itoa has no
  such site.
- **Checked:** format; `cargo clippy --release --all-targets -D warnings` on the syntax,
  adapter and CLI crates: clean.
- **Not done:** three timed release runs and workspace gates (end of part 6); second
  opinion on steps 6.2 to 6.4.
- **Next:** step 6.5, builtin members (9 calls, 30 reads; 31 new nodes predicted, 145 nodes,
  138 rows without a target).

### 2026-11-16 09:19 IST — Part 6 step 3: the 13 `MaybeUninit<u8>` type uses target the named external node

- **Milestone:** itoa full item graph, part 6 step 6.3. No SOP step has passed. The graph is
  still marked incomplete.
- **REQ:** REQ-RUST-053.0 N1, N3, N4, N5.
- **Measured on pinned itoa (release run, `/home/ubuntu/phase6-work/s63`, executable sha256
  `f0de39df…e5e94`):** 111 nodes (unchanged), 585 edges (unchanged), 105 of 106 `uses_type`
  rows with a target (was 92), 181 rows without a target (was 194). As predicted in the part file.

```text
rows  written in                              type as written    target node
10    generated Buffer type aliases           MaybeUninit<u8>    core::mem::MaybeUninit
2     parameters of enc_16lsd, slice_buffer   MaybeUninit<u8>    core::mem::MaybeUninit
1     the field of struct Buffer              MaybeUninit<u8>    core::mem::MaybeUninit
```

- **No node is added:** the 13 rows end at the node the import `use core::mem::MaybeUninit`
  already targets. One `uses_type` row is left without a target: the bound `Copy`, a
  prelude name (step 6.4).
- **What changed:**
  - `signature-type-edge-records.rs`: a type path left unresolved with
    `ExternalSourceMissing` whose one stored lookup row carries a proved name gives an
    `Extracted` row to that name, with `external_target_proof_record` on the row. `Self`
    is excluded: its reason comes from the impl header, not from a lookup at the site.
  - `external-named-target-records.rs`: `check_local_trait_member(text, names)`, the third
    condition of N5 for a written path. It reads the names outside `<..>` and `(..)`,
    stops at `->`, strips `r#`, and counts a leading `<A as T>` as one name.
  - No SQL change.
- **STUB:** `check_local_trait_member` as `todo!()`, producer wired to it; compiled.
- **RED:** `cargo test --release -p parseltongue-syntax-span-extractor --lib`:
  `FAILED. 128 passed; 2 failed` (`not yet implemented: REQ-RUST-053.0 N5`):
  `test_external_type_names_target`, `test_written_path_last_name`.
- **GREEN:** syntax crate `130 passed; 0 failed` and 17 in the second target;
  `cargo test --release -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`:
  `50 passed; 0 failed` (168.25 s), with the new `test_itoa_external_type_uses`.
- **One failed attempt on the way:** the fixture test first expected one edge; the generic
  argument `Gone` is a second row. The test now picks the row by its site range.
- **REFACTOR:** a second check of the stored reason inside the producer was removed: the
  row's resolution is already that reason. The helper was widened after GREEN (parenthesized
  arguments, `->`, the `<A as T>` qualifier), each with a case in `test_written_path_last_name`.
- **Existing real-source tests that pinned the old values (2):**
  `test_signature_external_type_unresolved` is renamed `test_signature_external_type_named`
  and expects the external node; `test_declared_type_use_rows` expects
  `core::mem::MaybeUninit` on the 10 alias rows.
- **Deliberate breaks (14), each fails a test:** N5 gate off; trait names not filled;
  `Self` not excluded; proof not on the row; target withheld; reason not checked; one name
  blocked too; generic arguments read as names; parentheses read as names; `r#` kept; any
  name instead of the last; qualifier not counted; return type read; arrow closes a group.
  "`Self` not excluded" passed at first; `test_self_unresolved_header_reason` gained a
  stored name at the site of `Self` and now fails it.
- **Weak evidence:** `test_itoa_external_type_uses` was written after the code and passed
  on first run.
- **Not covered on real source:** N5 for a type path. itoa has no type path of two names
  into `core`.
- **Checked:** format; `cargo clippy --release --all-targets -D warnings` on the syntax and
  CLI crates: clean (one `redundant_closure` in the new test fixed).
- **Not done:** three timed release runs and workspace gates (end of part 6); second
  opinion on steps 6.2 and 6.3.
- **Next:** step 6.4, prelude names (3 `implements` rows and the bound `Copy`; 3 new nodes predicted, 114).

### 2026-11-16 08:59 IST — Part 6 step 2: calls by a written path target named external nodes

- **Milestone:** itoa full item graph, part 6 step 6.2. No SOP step has passed. The graph is
  still marked incomplete.
- **REQ:** REQ-RUST-053.0 N1, N3, N4, N5, N6.
- **Clock:** the machine reads 08:59 IST now; the entry below is stamped 10:10 in error.
- **Measured on pinned itoa (release run, `/home/ubuntu/phase6-work/s62`):** 111 nodes (was
  107), 50 of 197 calls with a target (was 43), 585 edges (unchanged), 194 rows without a
  target (was 201). All four numbers are the ones predicted in the part file.

```text
rows  callee as written              target node (the written path)
4     mem::size_of::<Self>           core::mem::size_of
1     MaybeUninit::<u8>::uninit      core::mem::MaybeUninit::uninit
1     hint::unreachable_unchecked    core::hint::unreachable_unchecked
1     str::from_utf8_unchecked       core::str::from_utf8_unchecked
```

- **What changed:**
  - `original-call-resolution-records.rs`: a callee path stored unresolved with
    `external_source_missing` and a proved name gives an `Extracted` row to that name.
    The row says `dispatch_kind: external_written_path`, outcome `external_name_proved`,
    and carries `external_target_proof_record` (here: `use_import_names_path` and the
    `use` item the first name went through).
  - N5: a path of two or more segments whose last name a trait in read source declares
    gets no name; the row stays unresolved with `external_member_name_in_local_trait`.
    A one-segment callee is not blocked (a trait's member is never named by one segment).
  - `external-named-target-records.rs`: `collect_trait_member_names`, from the node rows.
  - No SQL change: the name lookup already stored the record on callee-path rows (step 1b).
- **Nothing is claimed about** which item the path ends at. `core::mem::MaybeUninit::uninit`
  is the written path; the row does not say it is an inherent function.
- **STUB:** `collect_trait_member_names` as `todo!()`; compiled.
- **RED:** `cargo test --release -p parseltongue-syntax-span-extractor --lib`:
  `FAILED. 94 passed; 34 failed` (`not yet implemented: REQ-RUST-053.0 N5`); the new
  `test_external_callee_names_target` among them.
- **GREEN:** syntax crate library suite `128 passed; 0 failed`, 17 in the second target;
  `cargo test --release -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`:
  `49 passed; 0 failed` (173.27 s), with the new `test_itoa_external_path_calls`.
- **Existing real-source tests that pinned the old values (5 assertions changed):** the four
  callee outcomes and `size_of` detail; the dispatch kind list; `Buffer::format` has one
  call to a node of itoa (its `size_of`-free count now excludes external targets); the
  import test lists only the nodes its rows target.
- **Deliberate breaks (10), each fails a test:** N5 gate off; a one-segment callee blocked
  too; stored reason not checked; proof record dropped; dispatch kind renamed; outcome
  renamed; proof not written on the row; raw `r#` prefix kept; shadow check off; trait
  member names taken from impl-owned nodes. The last one passed at first: the fixture graph
  marks every node impl-owned. A case with an inherent member of the same name now fails it.
- **Weak evidence:** `test_itoa_external_path_calls` was written after the code and passed
  on first run. The cases "other stored reason", "shadowed qualifier" and "inherent member"
  of the fixture test were added after GREEN.
- **Known defect, unchanged:** on `str::from_utf8_unchecked` the row's
  `callee_qualifier_symbol_kind` still says `Builtin` beside the proved module path
  (found in part 5d step 6). The target is right; the qualifier field is not.
- **Checked:** format; `cargo clippy --release --all-targets -D warnings` on the syntax and CLI crates: clean.
- **Not done:** three timed release runs and workspace gates (end of part 6); second opinion on this step.
- **Next:** step 6.3, `uses_type` of `MaybeUninit<u8>` (13 rows, no new node predicted).

### 2026-11-16 10:10 IST — Part 6 step 1c: the four imports target named external nodes

- **Milestone:** itoa full item graph, part 6 step 6.1. No SOP step has passed. The graph is
  still marked incomplete.
- **REQ:** REQ-RUST-053.0 N1, N2, N3, N4.
- **Measured on pinned itoa (real-source suite):** 107 nodes (was 103, as predicted), 4 of 4
  `imports` rows with a target (was 0). The four new nodes are `core::hint`, `core::mem`,
  `core::mem::MaybeUninit`, `core::str`; each has no file, kind or signature.
- **What changed:**
  - new `external-named-target-records.rs`: reads a stored external name record, builds the
    proof record of an edge row, and gives one external node per target (`collect_external_endpoint_nodes`);
  - `use-import-edge-records.rs`: an outcome left unresolved with `external_source_missing`
    that carries a proved name gives an `Extracted` row to that name; without one it stays unresolved;
  - `full-graph-checkpoint-facade.rs`: external nodes are collected from every relation, calls included.
- **Each row carries** `external_target_proof_record`: kind of name, how the site reached it,
  the `use` item it went through (none here), `external_source_read_flag: false`, and
  `external_definition_path_claimed_flag: false` (the path is the written one; `core::str`
  may be a re-export for all the read source shows).
- **STUB:** both functions `todo!()`; `cargo test` compiled.
- **RED:** `cargo test --release -p parseltongue-syntax-span-extractor --lib use_import`:
  `FAILED. 1 passed; 6 failed` (`not yet implemented: REQ-RUST-053.0 N3` / `N1`).
- **GREEN:** syntax crate library suite `127 passed; 0 failed` (was 125);
  `cargo test --release -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests`:
  `48 passed; 0 failed` (211.67 s).
- **Deliberate breaks, each failed a test at once (6):** any site proof kind accepted;
  `builtin_member` accepted from a stored record; source-read flag true; no external nodes
  collected; proof record dropped; proved name ignored.
- **Weak evidence:** the real-source assertions (`test_itoa_imports_row_counts`, node count
  107) were changed after the code and not seen failing on the old code.
- **Dropped from the fixture test:** whole-graph validation. The import fixture graph fails
  validation on its own (its `use` nodes have no owner kind). Node validity is covered by
  `test_external_node_row_validation` and by the itoa run, which validates on write.
- **Checked:** format; `cargo clippy --release` with `-D warnings` on the syntax and CLI crates: clean.
- **Not done:** three timed release runs (taken at the end of part 6).
- **Next:** step 6.2, calls by a written path (7 rows, 4 new nodes predicted).

### 2026-11-16 09:05 IST — Part 6 step 1b: name lookup stores the external name it proved

- **Milestone:** itoa full item graph, part 6. No SOP step has passed. No itoa number changed;
  no producer reads the new field yet.
- **REQ:** REQ-RUST-053.0 N2, N3, N5.
- **What changed:** `namespace-final-path-products.sql` gives five new columns per path
  request (kind of name, crate, written path, how the site reached it, the `use` item it
  went through). `path-resolution-site-records` and `import-resolution-site-records` carry
  them as `path_external_name_record` / `import_external_name_record`, only on a row left
  unresolved with `external_source_missing`.
- **Three proofs in the SQL:** crate name at the path root; first name bound by exactly one
  `use` item with the first proof; one name that only the prelude can bind. Each is refused
  when a scope on the lexical chain binds the name another way or holds a glob import.
- **Slip:** the SQL was written before the stub and the test.
- **RED:** `cargo test --release -p parseltongue-duckdb-analysis-adapter --lib
  test_external_name_proof_rows`: `FAILED. 0 passed; 1 failed`; the row had no record
  (`left: Null`), so the SQL ran and the reader did not store the result.
- **GREEN:** the adapter library suite: `185 passed; 0 failed` (was 184).
- **Deliberate breaks, each failed the test:** glob gate off, binder gate off, the check that
  the crate has no unit inverted.
- **Tested on fixture rows only, imports only:** the plain path, an alias, a resolved import,
  an unknown crate, a glob in the scope, another import binding `core`.
- **Not tested yet:** the second and third proof (they need non-import sites; steps 6.2 and
  6.4), a leading `::`, a trailing `self`, a nearer scope on the chain.
- **Checked:** format; `cargo clippy --release` with `-D warnings` on the adapter: clean.
- **Next:** the import producer reads the record and returns external node rows.

### 2026-11-16 08:20 IST — Part 6 step 1a: the named external identity (row types and codecs)

- **Milestone:** itoa full item graph, part 6. No SOP step has passed. No itoa number changed.
- **REQ:** REQ-RUST-053.0 N1, N3 (written in `405c51091`, with the GLM-5.3 design review).
- **STUB:** `HldExternalNameKind`, `HldExternalNamedKey`, the variant
  `HldTargetIdentityValue::ExternalNamed`, the error `MalformedExternalIdentity`, and
  `validate_hld_external_key` as `todo!()`.
- **RED:** `cargo test --release -p parseltongue-core-row-types test_external_node_row_validation`
  gave `test result: FAILED. 0 passed; 1 failed` (panic at the first accepted shape).
- **GREEN:** the same crate, all tests: `23 passed; 0 failed`. One test case was wrong, not the
  code: an empty path is refused earlier as a node without a label, so the test now asks only
  for an error there.
- **Codecs (written before their test, a slip):** the Turso encode and decode and the DuckDB
  encode gained the variant. `test_external_target_codec_roundtrip` passed on first run.
  Three deliberate breaks of the decoder (flag not checked, wrong kind, crate dropped) each
  failed it.
- **Checked:** `cargo test --release -p parseltongue-core-row-types -p parseltongue-turso-store-adapter`:
  23, 46 (3 ignored), 22 and 4 passed, 0 failed. `cargo clippy --release` with
  `-D warnings` on the three touched crates: clean. `cargo check --release --workspace
  --all-targets --all-features`: no error.
- **Not checked:** the DuckDB encode of the variant has no test of its own. Nothing produces
  the variant yet, so no real-source test can reach it.
- **Decision:** the variant is on the graph identity, not on `NameSymbolTargetKey` (GLM-5.3
  proposed the latter). Name lookup never produces it, so three files changed instead of every
  match on the lookup key. Recorded in the PRD.
- **Next:** the name lookup stores the path it proved (`namespace-final-path-products.sql`),
  then the import producer; predicted 107 nodes, 4 imports with a target.

### 2026-11-16 07:56 IST — Part 5d workspace gates at `41567d710`

- **Milestone:** itoa full item graph, part 5d. No SOP step has passed. `completeness=incomplete`.
- **Result: not a full PASS.** `test_lang_items_in_sysroot` is skipped, as the standing
  command says; it is an open defect, not a waiver.
- **Commands (the standing four) and results:**

```text
cargo fmt --all -- --check                                                        ok
cargo clippy --release --workspace --all-targets --all-features -- -D warnings    ok
cargo test --release --workspace --all-targets --all-features
      -- --skip test_lang_items_in_sysroot --test-threads=1                       574 passed, 0 failed, 4 ignored
                                                                                  (21 test binaries; itoa 48 of 48, 1040.61 s
                                                                                  on one thread)
cargo build --release --workspace --all-targets --all-features                    ok
```

- **The skipped test, run once without the skip (debug profile):** it failed after
  2067 s at its first assertion, the mapping run's exit status. That run used
  `cargo test` without the standing flags, which was my mistake; it stopped at this
  test binary, so I ran the standing command afterwards.
- **Two executables, measured:** the `--all-features` build is
  `97045d7f74b060da416ae9892c2b618a944c2387d2fab54377c1da980458833d`; the default
  build used for the step 11 measurements is
  `66a74fde41ed76755d2e262530f2e14fe0dbe1d28f13127ce96401fac82cb0c2`, and building it again after the gates gave the same digest.
- **Next:** part 6.

### 2026-11-16 06:20 IST — Part 5d step 11: value paths in the arguments of `assert!` (REQ-RUST-052.0 G15)

- **Milestone:** itoa full item graph, part 5d. No SOP step has passed. `completeness=incomplete`.
- **Built at** `97894638f`. Spec: part 5 section 10.18.
- **Cause, measured first:** a macro call's arguments are a token tree; the paths in
  `assert!(i8::MIN < 0, ..)` had no site row and no name lookup.
- **STUB:** `read_macro_argument_paths` with `todo!()`; `cargo check -p
  parseltongue-syntax-span-extractor` finished with 3 dead-code warnings.
- **RED:** `test_macro_argument_paths_read` panicked at `not yet implemented:
  REQ-RUST-052.0 G15 step 11`; `test_macro_argument_read_rows` got `[]` for 20 rows;
  `test_reads_rows_read_twice` got 78 for 98 (0 passed, 2 failed).
- **GREEN:** 48 of 48 itoa tests (118.93 s), 125 + 17 syntax crate tests, format, lint on
  the two touched crates. Four older itoa expectations changed to the new totals.
- **Second RED/GREEN:** the named format argument (`n = C`) was read as a path; the
  fixture case failed, then passed. Found by the second opinion.
- **Measured (3 release runs, executable `66a74fde41ed76755d2e262530f2e14fe0dbe1d28f13127ce96401fac82cb0c2`):**

```text
nodes=103 containment=88 generates=45 expands=15 calls=197 resolved_calls=43 type_uses=106
implements=32 reads=98 reads_in_expanded_bodies=76 imports=4 edges=585 completeness=incomplete
run1 wall=16.20s maxrss=343184kB
run2 wall=17.29s maxrss=375944kB
run3 wall=18.69s maxrss=380104kB
store_bytes=18608128
```

- **New rows (20), as predicted before the code:** `iN::MIN`, `uN::MIN`, `iN::BITS`,
  `uN::BITS` in each of the 5 generated `const _` blocks. None has a target (`core`).
  Read back from the stored edge table.
- **Decision (PRD):** arguments of the six assertion macros are parsed again as
  expressions; the macro is not expanded. Rejected: reading every macro's arguments
  (their grammar is not known), and refusing all message arguments (positional ones are
  evaluated expressions).
- **Weak evidence:** `test_macro_argument_site_gate` and `test_macro_argument_read_edge`
  were written after the code and passed on first run.
- **Deliberate breaks:** 26. 24 failed a test at once; one was an equivalent break of a
  check written twice (the copy was removed); one needed a new assertion on the site rows.
- **Second opinion on 10a, 10b, 10c:** GLM-5.3 found no wrong target. Its one claimed
  miss (a const block in a function hiding an item read behind an outer `let`) does not
  compile on rustc 1.99.1 (`E0005`: the `let` name is a constant pattern), so it cannot
  occur. Its test gaps are Q54; the alias anchor is Q53.
- **Slips:** no codebase-memory trace before the change.
- **Workspace gates:** not run; they run next, at the end of part 5d.
- **Next:** 5d workspace gates, then part 6.

### 2026-11-16 02:05 IST — Part 5d step 10b: the impl member for `<T as Trait>::NAME` (REQ-RUST-052.0 G14)

- **Milestone:** itoa full item graph, part 5d. No SOP step has passed. `completeness=incomplete`.
- **Built at** `9171dab2e`. Spec: part 5 section 10.17.
- **Cause, measured first:** name lookup of the whole path drops the anchor type and gives
  the trait's declaration. The lookups of the anchor and the trait path are stored for a
  path outside a function body, so no new request was needed (my 10a note said they were
  not stored; that was wrong for these four rows).
- **STUB:** three signatures with `todo!()`; `cargo check -p
  parseltongue-syntax-span-extractor` finished with 4 dead-code warnings.
- **RED:** `test_qualified_path_parts_read` and `test_qualified_member_trait_match`
  panicked at `not yet implemented: REQ-RUST-052.0 G14 step 10b`;
  `test_item_initializer_read_rows` and `test_declared_type_use_rows` failed on their row
  comparison (0 passed, 2 failed).
- **GREEN:** 47 of 47 itoa tests (142.51 s), 122 + 17 syntax crate tests, format, lint on
  the two touched crates.
- **Measured (3 release runs, executable `15a836e11ef8c635b505de6ff498d0b0f51f24338daca22a8ed3242655bbec71`):**

```text
nodes=103 containment=88 generates=45 expands=15 calls=197 resolved_calls=43 type_uses=106
implements=32 reads=78 reads_in_expanded_bodies=56 imports=4 edges=565 completeness=incomplete
run1 wall=21.87s maxrss=398096kB
run2 wall=18.86s maxrss=378808kB
run3 wall=16.74s maxrss=358516kB
store_bytes=18001920
```

- **Changed rows (4), totals unchanged:** 2 `reads` now on `<u64|i64 as
  Integer>::MAX_STR_LEN`, 2 `uses_type` now on `<u64|i64 as private::Sealed>::Buffer`.
  These are the expected graph's targets.
- **Rule changed after the first GREEN:** candidates are now only impls the path can name
  (not an inherent impl, not an impl of another stored trait). The fixture expectations
  for that case were changed with the rule and not seen failing first.
- **Weak evidence:** `test_qualified_path_member_evidence` was written after the code and
  passed on first run. Breaking the code makes it fail (below).
- **Deliberate breaks:** 11, all fail a test (7 on fixtures, 4 on itoa).
- **Slips:** no codebase-memory trace before the change. Second opinion for 10a, 10b and
  10c is being taken now and gets its own record.
- **Workspace gates:** not run; they run at the end of part 5d.
- **Next:** step 11.

### 2026-11-16 01:10 IST — Part 5d step 10c: declared and aliased types as `uses_type` sites (REQ-RUST-052.0 G7)

- **Milestone:** itoa full item graph, part 5d. No SOP step has passed. `completeness=incomplete`.
- **Built at** `84c4e6372`. Spec: part 5 section 10.16.
- **Cause, measured first:** name lookup was already asked for these type paths; the
  field table had no row for the declared type of a const or static or the aliased type of
  a type alias, so the `uses_type` producer never saw them.
- **STUB:** `read_declared_type_owner` with `todo!()`; `cargo check -p
  parseltongue-syntax-span-extractor` finished with one warning (`function
  read_declared_type_owner is never used`).
- **RED:** `test_declared_type_paths_listed` failed: left had 3 rows (`T: H`, `H`, `Q`),
  right 8. `test_declared_type_use_rows` failed: left `[]`, right 40 rows.
- **GREEN:** 46 of 46 itoa tests (144.55 s), 120 + 17 syntax crate tests, format, lint on
  the two touched crates.
- **Measured (3 release runs, executable `3532463e6eea5354921a8331ce0b2a1d8c916f5f22e20dcd577a39b86c9f4143`):**

```text
nodes=103 containment=88 generates=45 expands=15 calls=197 resolved_calls=43 type_uses=106
implements=32 reads=78 reads_in_expanded_bodies=56 imports=4 edges=565 completeness=incomplete
run1 wall=23.82s maxrss=385968kB
run2 wall=17.95s maxrss=356420kB
run3 wall=25.20s maxrss=374080kB
store_bytes=17985536
```

- **Prediction held:** 106 `uses_type` and 565 edges were written down before the code.
- **New rows (40):** 12 `MAX_STR_LEN -> usize`, 10 `Buffer -> MaybeUninit<u8>` (no
  target, in `core`), 10 `Buffer -> u8`, 6 from the two qualified aliases (the whole path
  on the trait's declaration, `u64`/`i64`, `private::Sealed`), `DECIMAL_PAIRS ->
  DecimalPairs`, `Integer::MAX_STR_LEN -> usize`.
- **Pins changed, each after reading the assertion:** edges 525 to 565, `uses_type` 66
  to 106, and the generated-signature test now skips declared-type rows (it asserts
  "copied from the definition", and `u64` is copied from the call; the new test pins the
  origin of all 40).
- **Deliberate breaks:** 7, all fail a test; one (a static labelled as a const) survived
  until the fixture also checked the item-kind field.
- **Defect of mine repaired:** the step 10a insert had split the doc comment of
  `test_cast_receiver_call_targets` in two. Put back in one piece.
- **Q40 closed:** the 10 `.ilog10()` calls in const initializers are already `calls` rows.
- **Slips:** no codebase-memory trace before the change. No second opinion yet; taken
  with 10b.
- **Workspace gates:** not run; they run at the end of part 5d.
- **Next:** 10b (2 `reads` and 2 `uses_type` qualified paths), then step 11.

### 2026-11-16 00:05 IST — Part 5d step 10a: consts, statics and types as `reads` sources (REQ-RUST-052.0 G7)

- **Milestone:** itoa full item graph, part 5d. No SOP step has passed. `completeness=incomplete`.
- **Built at** `ed55f4a63`. Spec: part 5 section 10.15.
- **Cause, measured first:** the stored site table already held the paths owned by
  generated consts and type aliases; the value-path request was only made inside a
  function body. One predicate changed, and the scope of local bindings with it.
- **Measured (3 release runs, executable `576bb0918faed1f1fa9daa14f6ff744de480cf025d7f6ca45cdb87c489cced1e`):**

```text
nodes=103 containment=88 generates=45 expands=15 calls=197 resolved_calls=43 type_uses=66
implements=32 reads=78 reads_in_expanded_bodies=56 imports=4 edges=525 completeness=incomplete
run1 wall=19.44s maxrss=385908kB
run2 wall=17.03s maxrss=391440kB
run3 wall=17.18s maxrss=371312kB
store_bytes=17358848
```

- **New rows (23):** 10 `uN::MAX` without a target, 10 `Self::MAX_STR_LEN` in generated
  `Buffer` aliases with the impl member as target, 2 `<u64|i64 as Integer>::MAX_STR_LEN`
  on the trait's declaration, 1 `i128::MAX_STR_LEN` in the array length of struct `Buffer`.
- **RED:** `test_item_initializer_read_rows` failed with 0 rows against 23;
  `test_item_initializer_paths_listed` failed on a struct in a function seeing the
  function's parameter. **GREEN:** 45 of 45 itoa tests (135.71 s), 119 + 17 syntax crate
  tests, format, lint on the two touched crates.
- **Prediction off:** I expected the 2 qualified-path rows to need a new rule before they
  had any target. Name lookup already gives the trait's declaration. I first wrote the
  test for the impl member (G14), then pinned the measured, less exact target and moved
  G14 to step 10b: it needs name requests that are not stored.
- **Pins changed, each after reading the assertion:** 12 in 7 older itoa tests, 2 in
  fixture tests. One was not a number to raise: an older test asserted that no generated
  `reads` row is left with `TraitSolverNeeded`. Ten are now (`uN::MAX`). I kept the
  reason and pinned the ten by text and lookup record, instead of renaming the reason
  to "external source missing", which is not proven.
- **Side effect:** 11 array-length paths left the `signature` request kind (204 to 193
  rows) and are value-path requests now.
- **Deliberate breaks:** 4, all fail a test; one survived until a file-attribute case
  was added to the fixture.
- **Slips:** no `todo!()` stub (no new signature). No codebase-memory trace result
  before the change: the first trace command failed on my parsing, and I did not rerun
  it. No second opinion yet; it is taken with 10b and 10c.
- **Workspace gates:** not run; they run at the end of part 5d.
- **Next:** 10c, then 10b, then step 11.

### 2026-11-15 23:20 IST — Part 5d step 9: the impl member when the receiver is a cast (REQ-RUST-052.0 G13)

Clock note: `date` on the machine reads 2026-11-15 23:20 IST. The entries below carry
stamps that run about five hours ahead of it; their order is right, their times are not.

- **Built.** `(x as T).name(..)` has the one stored impl member `name` for `T` as target
  (`dispatch_kind: cast_receiver_impl_method`) when `T` is a stored struct, enum, union or
  builtin and the member's trait is the trait of the impl the call is written in. The row
  keeps `cast_receiver_lookup_evidence`: the cast's type, the candidates, the member, the
  impl the call is in with its trait and type, and what was not read. New stored fields on
  every method call: `receiver_cast_type_evidence`, `receiver_member_body_flag`. New name
  lookup context: `body_cast_receiver_type`.
- **Measured on pinned itoa, three release runs:** 103 nodes, 502 edges (unchanged),
  `resolved_calls` 43 of 197 (was 41). 19.2 to 21.4 s, 350 to 359 MB peak, 16.8 MB store.
  The two rows: `<usize as private::Sealed>::write` to `<u64 as private::Sealed>::write`,
  and the `isize` to `i64` one. Still marked incomplete.
- **Counted first.** Of 141 stored method calls, 3 have a target in itoa (these two and
  `i.write(..)`, already resolved). The other 138 call into `core`.
- **RED.** `test_cast_receiver_call_targets`: no row had a cast receiver, 0 passed 1
  failed. **GREEN:** after one compile fix (a missing import) the two rows were right but
  the test still failed on text: expanded source reads `private ::Sealed`. The test now
  strips spaces; 1 passed.
- **No STUB phase.** The drafted code went in after RED without a `todo!()` signature
  step. A slip against the loop.
- **Caught by the full real-source run:** `test_generated_body_call_rows` (dispatch kinds;
  104 untargeted method calls are now 102 plus 2) and `test_sysroot_reached_names_reasoned`
  (202 path rows are 204, 164 resolved are 166). Both changed to measured values. One
  fixture pin, `test_capture_generic_receiver_evidence`, gained the two new fields.
- **Fixture tests, written after the code, all passed on first run**
  (`cast-receiver-member-tests.rs`): `test_cast_receiver_syntax_evidence`,
  `test_cast_receiver_path_request`, `test_cast_receiver_member_rules`,
  `test_cast_receiver_call_member`.
- **Mutations.** 16 deliberate breaks. 15 failed a fixture test at once. One passed
  ("any item" instead of "a function" as the call's nearest item); a top-level `const`
  case was added and it now fails (`/home/ubuntu/phase5d-work/s9-mutations.txt`).
- **Second opinion (GLM-5.3, built code, 324 s).** No wrong-target case found within read
  source. Adopted: fixture cases for a member that is not a graph node, a negative impl,
  and a call written `r#name`; the enclosing impl's trait and type on the row. Agreed with
  giving the target for a builtin with the unread `core` sources named on the row.
  Record: `docs/devin-research/1794784173-second-opinion-glm-5-3-max-part-5d-step-nine-cast-receiver.md`.
- **Gates.** Format; lint on the two touched crates; syntax crate 118 + 17 tests; all 44
  real-source itoa tests (127 s); release build. Workspace gates not run; they run at the
  end of part 5d.
- **codebase-memory.** `trace_path` on `extract_method_receiver_syntax`,
  `find_type_member_target` and `check_body_call_prefix` before the change; re-index
  after the push.
- **Refactor.** The first draft did lookup, proof and endpoint check in one function. It is
  split: `find_cast_receiver_member` (pure, on stored rows) and
  `resolve_cast_receiver_target` (type lookup and the graph-node check).
  `original-call-resolution-records.rs` is still far over the 200-line guide; the split is
  left for the end of part 5d.
- **Not built:** Q35 (a call in an associated const), Q36 (a trait in scope by `use` or
  prelude), Q37 (receiver types readable from itoa's source whose methods are in `core`).
- **Next.** Generated consts and type aliases as edge sources (G7), then the part 5d
  workspace gates, then part 6.

### 2026-11-16 04:10 IST — Part 5d step 8: the impl member when the first argument is the method's own `self` (REQ-RUST-052.0 G12)

Clock note: this time is read from the machine. The two entries below are stamped about
two and a half hours ahead of their commits (`79357012e` was committed at 03:38 IST).

- **Built.** `Trait::name(self, ..)` written in a method that takes `self` by value, in an
  impl for a stored type, has the one impl member of that trait for that type as target
  (`dispatch_kind: trait_path_impl_method`, outcome `single_impl_member_by_self`). The row
  keeps the trait's declaration, the candidate list, the chosen member, and the impl and
  type that proved it. New stored fields: `call_first_argument_self_flag` on call syntax,
  `member_self_parameter_kind` on impl member rows.
- **Measured on pinned itoa, three release runs:** 103 nodes, 502 edges, `resolved_calls`
  41 of 197, all unchanged; 5 rows moved from the trait's declaration to
  `<uN as Unsigned>::fmt` (u8, u16, u32, u64, u128). 14.8 to 18.1 s, 339 to 365 MB peak,
  16 MB store.
- **RED.** `test_trait_path_call_targets`: all ten rows read
  `trait_path_method trait_method_declaration false`, 0 passed 1 failed. **GREEN:** 1 passed
  (23 s), after one compile fix (`ast::HasArgList::arg_list(&call)`).
- **Caught by the full real-source run:** `test_generated_body_call_rows` pinned the set of
  dispatch kinds and failed (42 passed, 1 failed). The new kind is added; 43 pass.
- **Fixture tests, written after the code** (`self-argument-member-tests.rs`):
  `test_own_self_argument_flag`, `test_self_argument_member_rules`,
  `test_trait_call_self_member`. All three passed on first run.
- **Mutations.** 18 deliberate breaks (13 before the review changes, 5 after); each failed a
  fixture test (`/home/ubuntu/phase5d-work/s8-mutations.txt`).
- **Second opinion (GLM-5.3, built code).** Found one case that would have given a wrong
  member: `impl T<u16> for u8` beside `impl T for u8` left one candidate, because an impl
  whose trait has generic arguments is not stored as resolved. Fixed: every stored trait
  impl counts unless its trait or type is stored as another one. Pinned on header rows,
  not on real source. Also adopted: the row names the impl and the chosen member; `r#name`
  matches. Not adopted: an explicit guard on generic arguments in the callee (Q31).
  Record: `docs/devin-research/1794782315-second-opinion-glm-5-3-max-part-5d-step-eight-self-argument-member.md`.
- **Gates.** Format; lint on the two touched crates; syntax crate 114 + 17 tests; all 43
  real-source itoa tests (94 s); release build. Workspace gates not run; they run at the
  end of part 5d.
- **codebase-memory.** `search_graph` and `trace_path` on `resolve_callee_path_target`
  and `find_type_member_target` before the change; re-index and `trace_path
  find_self_argument_member` after it.
- **No refactor.** `original-call-resolution-records.rs` is 1,342 lines, far over the
  200-line guide; splitting it is left for the end of part 5d. `find_self_argument_member`
  and `check_own_self_argument` are three-word names after the verb; kept, since both are
  private and read as what they do.
- **Not built:** the five `Unsigned::fmt(self.unsigned_abs(), ..)` rows (Q33, needs
  `core`); cast receivers (Q34).
- **Next.** Method calls: read the stored rows of the 104 generated method calls by
  receiver shape, then the smallest rule that is provable from source.

### 2026-11-16 06:20 IST — Correction to the step 7 entry below: 5 and 5, not 6 and 4

- The part file said six of the 10 `Unsigned ::fmt` calls pass `self` and four pass
  `self.unsigned_abs()`. I had not counted them. Counted in the stored `calls` rows of the
  step 7 run: 5 are `Unsigned ::fmt (self , buf )`, 5 are
  `Unsigned ::fmt (self .unsigned_abs () , ..)`. The part file (10.12) is corrected.
- No code or test changed; no total changed.

### 2026-11-16 06:00 IST — Part 5d step 7: a trait's method through the trait; a tuple struct's constructor (REQ-RUST-052.0 G10, G11)

- **Built.** A callee qualified by a trait whose whole path stores one Function that is a
  graph node has the trait's declaration as target (`dispatch_kind: trait_path_method`),
  with `trait_method_lookup_evidence` listing every function of that name in a stored impl
  of that trait. A bare or module-qualified callee whose path stores one Struct that is a
  graph node has the struct as target (`tuple_struct_constructor`).
- **Measured on pinned itoa, three release runs:** 103 nodes, 502 edges (unchanged),
  `resolved_calls` 41 of 197 (was 30), 18.4 to 22.7 s, 354 to 369 MB peak, 16 MB store.
  The 10 `Unsigned ::fmt` rows each list 5 candidates (`u8`, `u16`, `u32`, `u64`, `u128`).
- **RED.** `test_trait_path_call_targets`: "`<u8 as private::Sealed>::write` calls
  Unsigned::fmt without a target", 0 passed 1 failed. **GREEN:** 1 passed, after I replaced
  a placeholder candidate list with the measured one.
- **Stale assertions changed to measured values:** `test_callee_path_function_targets`,
  `test_generated_body_call_rows`, and two fixture cases that pinned "no target" for a
  struct callee and a trait-qualified function.
- **Wrong expectation of mine:** a function through a struct qualifier stops at
  `member_lookup_unproven`, not `type_prefix_unresolved`.
- **Mutations.** Six deliberate breaks; each failed a fixture test at once
  (`/home/ubuntu/phase5d-work/s7-mutations.txt`).
- **Gates.** Format; lint on the two touched crates; syntax crate 111 + 17 tests; all 43
  real-source itoa tests (131 s); release build. Workspace gates not run; they run at the
  end of part 5d.
- **Second opinion.** None new: both designs were reviewed by GLM-5.3 in step 6 and are
  built as decided there.
- **codebase-memory.** Slip against Rule 4: re-index and `trace_path
  resolve_callee_path_target` (19 callees, among them `list_trait_method_candidates`,
  `read_single_target_identity`, `read_type_path_records`) were run after the code change,
  before the commit, not before the change.
- **No refactor.** `resolve_callee_path_target` grew one match and one helper; splitting
  it is left for the end of part 5d.
- **Not built:** the impl member as target for `Unsigned::fmt(self, buf)` (Q27); tuple enum
  variants as callees (Q28).
- **Next.** Q27, then method calls.

### 2026-11-16 04:30 IST — Part 5d step 6: a call whose callee path names one function (REQ-RUST-052.0 G9)

- **Built.** The whole callee path of each call is one more stored path request
  (`body_call_callee_path`, Value namespace, the call's own scope). A call gets the function
  as target when that lookup stores exactly one Function that is a graph node and the path
  is one name or is qualified by a module.
- **Measured on pinned itoa, three release runs:** 103 nodes, 502 edges (unchanged),
  `resolved_calls` 30 of 197 (was 3), 16.8 to 19.1 s, 357 to 376 MB peak, 16 MB store.
  Path table 258 rows (was 202): the 56 callee paths.
- **RED.** `test_callee_path_function_targets`: "`<i8 as private::Sealed>::write` calls
  slice_buffer_to_str without a target", 0 passed 1 failed. **GREEN:** 1 passed.
- **Stale assertions changed to measured values:** `test_generated_body_call_rows` (the 18
  generated `divmod100` / `slice_buffer_to_str` rows now have a target; a third dispatch
  kind), `test_sysroot_reached_names_reasoned` (callee paths split out of the path total),
  `test_request_original_body_prefix` (2 requests for `Buffer::new()`, was 1).
- **Two wrong expectations of mine in fixtures:** an anchored callee `<u8 as T>::x()` already
  has a Type request for its anchor; a trait qualifier's row keeps reason
  `TraitSolverNeeded`, and a builtin qualifier's `MissingTarget`. Tests pin the measured
  behaviour.
- **Finding.** `str::from_utf8_unchecked`: the qualifier's Type lookup says builtin `str`
  though `use core::str` makes it a module. The whole path's lookup is right (external
  source missing) and the row takes it. The qualifier lookup is not fixed (part 5 Q24).
- **GLM-5.3 review** (pi, max reasoning, read-only), recorded in
  `docs/devin-research/1794779312-second-opinion-glm-5-3-max-callee-path-step-six.md`.
  Adopted: keep the lookup record on qualified rows that go to the type-prefix lookup;
  narrow the exception to a builtin qualifier, so a struct qualifier's member lookup is
  never displaced; tests for two stored targets, the block scope of the request, a module
  qualifier without a target. Its answers on `Unsigned::fmt` and `DecimalPairs(..)` are
  recorded as decisions for the next step in the PRD.
- **Mutations.** Nine deliberate breaks; seven failed a test at once; two (ignore the
  shadowing flag; an ambiguous qualified path decides) did not, and each got a test.
- **Gates.** Format; lint on the two touched crates; syntax crate 109 + 17 tests; all 42
  real-source itoa tests (110.6 s; 667 s single-threaded). Workspace gates not run; they
  run at the end of part 5d.
- **Not covered:** `crate::f`, `self::f`, `super::f` callees and generic arguments on a
  non-final segment (Q25). itoa has none.
- **Next.** `Unsigned ::fmt(..)` and `DecimalPairs(..)`, then method calls.

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
