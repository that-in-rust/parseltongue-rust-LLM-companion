# Phase 1 Evolution Log — Part 1

This is the evolving log of the path from pinned itoa source to the detailed full
dependency graph and evidence. Higher-level query abstractions are deferred in the
[backlog](phase1-abstractions-decision-backlog.md); older entries retain their historical scope.

## How to update this log

- **Newest entry first.** Insert each new dated entry immediately below this
  introduction and above the previous entry. Do not append updates at the bottom.
- Preserve older entries as historical snapshots. Correct a mistake in a new
  entry rather than silently rewriting history.
- Use IST timestamps and keep subpart IDs stable so progress can be compared.
- Each update records what changed, executable evidence and its scope, remaining
  failures/decisions, and the next source-backed acceptance target.
- Distinguish a decided rule, written code, a passing component test, and a
  passing real-source acceptance test. They are not interchangeable.
- This log does not replace the current PRD, SPEC, architecture, status board or
  append-only TDD journal. Historical entries are not new requirements.

---

## 2026-11-14 20:24 IST — First full-map call proven on the single VM

**Expected:** `<Buffer as Default>::default -> Buffer::new`, produced from pinned source,
persisted with exact evidence, then recovered from the compatible frozen full graph.
**Actual:** both new real-source tests pass. This is a bounded first-call checkpoint,
not complete-map, whole-itoa golden, all-five SOP or final 500 MB qualification.

The selected ten subparts now have integrated evidence:

1. Original active non-body items are inventoried from existing persisted source facts.
2. Canonical item/member identities retain exact ranges and digests.
3. Associated functions bridge to existing impl-header member occurrences.
4. Each original item receives one lexical file/module/trait source container.
5. Body-local descendants remain evidence, not graph nodes.
6. Item nodes persist without guessing visibility or reachability.
7. Containment stays separate from impl/self-type/trait ownership.
8. Existing namespace facts resolve the body's Buffer type prefix.
9. Existing impl records prove the unique non-generic inherent Buffer::new target.
10. The exact direct call survives frozen incomplete-input readback; strict HLD rejects it.

The retained run produced **45 nodes** (43 items + 2 file areas), **43 containment edges**,
and **61 calls, only 2 resolved**. Its **104 edge/evidence records include 59 unresolved calls**.
Full-map completeness remains explicitly `incomplete`; no HLD output is published by this stage.
Child-file ownership was corrected through behavioral RED/GREEN without changing containment.
The original 202 path-fact checks remain, alongside 29 explicitly separate body-prefix facts.

Verification: **11 source tests pass**, including the two new full-map checks; **415 bounded
workspace tests pass / 0 fail / 4 ignored / 1 excluded**. Formatting, warnings-denied release
Clippy and all-target/all-feature release build pass. Python SOP runner: **146 passed**.
The excluded native/sysroot test remains excluded under the prior stop rule.

Retained single CLI run: **14.810602s / 410,705,920 bytes** sampled whole-process-tree peak;
DuckDB reports 25.0 GiB effective limit and 2 threads, with no interim cap. Executable SHA-256:
`ec9d24df192152263ed5bf6cec0db0c3cc20b64a2498886bcc4a1b63fb85fcde`.
The bounded run report `hld-current-test-runs/20261114T140300Z-full-map-implementation/` (report removed from the tree on 2026-11-14; last present at commit a2e13a3bd)
indexes the frozen runtime source manifest/archive, retained database, exact pin and gate logs.

All recovered worker progress remains in this checkout; the three workers remain archived.
Everything after consolidation was implemented and verified on **this VM only**. No commit,
push or PR. Next source-backed target: **Buffer::format -> private::Sealed::write**, starting
with generic/supertrait lookup RED. Higher-level abstractions stay in the backlog.

---

## 2026-11-14 19:43 IST — Single-VM correction and preserved worker progress

The user clarified that parallelization must use **one VM only**. The earlier interpretation
as separate-VM child sessions was wrong. `AGENTS.md` now forbids separate-VM workers;
implementation, integration, builds and tests continue on this session VM only.

All three workers stopped and exported their current owned changes and evidence. Their
20 changed/new source files were applied without conflicts into the existing integration
checkout; all three reverse-patch checks passed, and parent files outside their ownership
were compared byte-for-byte against a pre-integration backup. Worker sessions were archived
after recovery. This is local consolidation, not a GitHub merge, commit or push.

**Incomplete work remains explicit:** original projection and the CLI facade are stubs;
the resolver's last frozen-snapshot correction is not yet reverified locally; graph storage
needs formatting and aggregate gates. Child-reported component passes are not a parent
acceptance pass. `default -> Buffer::new` remains unproven through the combined pipeline.

Recovery artifacts and checksums:
`~/hld-current-test-artifacts/20261114T140300Z-full-map-implementation/`.
Next: continue TDD on the single VM, finish projection/composition and run the real-itoa check.

---

## 2026-11-14 19:33 IST — Ten dependent implementation subparts

Active target: original item/member evidence followed by the exact
`<Buffer as Default>::default -> Buffer::new` call. This is bounded development of the full map,
not another HLD projection or a complete semantic qualification.

1. Inventory original non-body items (1.1).
2. Preserve canonical item/member identities and exact ranges (1.2).
3. Bridge members to existing impl-header records (1.3).
4. Assign one lexical source container (1.4).
5. Retain body-local descendants as evidence, not nodes (1.1/1.4).
6. Persist private/public item nodes without guessed visibility (1.1/1.9).
7. Persist containment separately from impl/self-type/trait ownership (1.4).
8. Resolve the body type prefix using existing namespace facts (1.6).
9. Select a uniquely proven associated function through impl records (1.7).
10. Persist the direct call and verify compatible frozen readback (1.8/1.10/4.5).

Delivery/reliability mode: reuse existing canonical core records, `StageTableStorePort`, impl
headers, source-evidence validation and Turso snapshot safeguards. Rust remains the semantic
authority; DuckDB performs relational source projection; no third database. REQ-RUST-045.0
specifies incomplete development checkpoints independently of the unchanged HLD completeness gate.

Parallel work partitions: original-item projection; Rust body/type-relative lookup; frozen
full-map storage/validation. The parent owns CLI integration, real-itoa acceptance, shared docs
and final gates. TDD is STUB -> RED -> GREEN -> REFACTOR, with phase evidence in the journal.
No implementation or new runtime pass is claimed in this planning entry. No commit/push authorized.

## 2026-11-14 18:19 IST — Name, current evidence and next implementation checkpoint

### Name: `full-item-dependency-graph`

Keep the existing four-word canonical name; no competing name or schema rename is needed.
It names the intended complete scoped map, not a claim that it is already complete.

- **full:** retain public/private facts and direct dependencies without HLD filtering or collapse.
- **item:** graph nodes follow the item contract; function-body descendants remain linked evidence,
  and impl blocks remain ownership records rather than graph nodes.
- **dependency:** include proven calls, type uses, reads, imports and other specified relations,
  not merely file containment. Unresolved sites retain explicit reasons, never guessed edges.
- **graph:** one logical graph across canonical node/edge tables and linked source, signature,
  body, resolution and expansion evidence; not one giant physical table.

Abstractions are later queries over this graph. Their work and unrelated decisions stay in the
[backlog](phase1-abstractions-decision-backlog.md).

### Where we stand — expected versus actual

**Expected next relationship:** `<Buffer as Default>::default --calls--> Buffer::new`, with
canonical endpoints, exact call-site evidence and compatible frozen full-map readback.
**Actual:** this semantic call edge is not implemented or verified. The first missing production
slice is original item/member node projection and source-container edges; body/type-relative
lookup follows. Planning is clearer, but the last scope/name updates added no runtime capability.

The last verified bounded itoa run established:

- 33 active impl spans have header rows; `Default`'s self type resolves to `Buffer`.
  The excluded core trait remains explicitly unresolved. This is header-path coverage, not
  complete signature lowering or call resolution.
- Original top-level structural persistence/publication and exact source-evidence checks pass;
  that diagnostic excludes methods and is not the full-map product.
- 9 source-acceptance tests and 362 bounded workspace tests passed; formatting, Clippy and
  release build passed. These are the earlier recorded results, not new tests. The native
  sysroot test was filtered; no complete full-map, all-five SOP or determinism qualification.

### What moves us next

1. **Behavioral RED:** add/discover a pinned-source regression for distinct canonical `default`
   and `new` method nodes. Assert exactly one lexical source container for each, separate impl
   ownership, and exclusion of body-local descendants from graph nodes.
2. **GREEN:** implement the minimum general-purpose original item/member producer and its
   source-container edges from persisted facts. No itoa-name special cases and no widening of
   the original-top-level diagnostic mode.
3. **First semantic call:** resolve the body path `Buffer::new` through impl/type-relative lookup;
   require exactly one extracted call at the `default` site, with exact endpoints and source bytes.
4. **Readback and verification:** check those nodes, edges and evidence in a compatible frozen
   full-map generation without HLD output. Specify any missing stage/marker contract before
   implementing it; do not bypass the existing publisher. Rerun affected tests and quality gates,
   recording executable/source identity, elapsed time and process-tree memory.

**Next checkpoint passes only when:** the method-node/source-container assertions and the direct
`default -> Buffer::new` call survive exact frozen readback. Node-only progress is reported as
node-only progress. Generic `format -> Sealed::write`, remaining dependency kinds, macro/core
coverage and independent whole-map comparison follow, not another abstraction layer.

Cargo targets were cleaned; the next build is cold. Keep build inputs stable, measure compilation
separately, and do not retry the blocked full-native path. This entry changes documentation only;
no production edits, runtime tests, commit or push.

---

## 2026-11-14 18:03 IST — Detailed full map only; abstractions moved to backlog

**Decision:** focus on the mega dependency map with all scoped semantic details.
Anything not needed to build, preserve or verify that graph belongs in the
[abstraction and decision backlog](phase1-abstractions-decision-backlog.md).

- The map is one logical graph: canonical nodes/edges plus linked source, signature,
  body, resolution and expansion evidence. Public/private facts remain unfiltered.
- Interface views, dependency slices, collapse and roll-ups are later queries over
  a compatible frozen generation, optionally materialized. They do not need separate
  semantic pipelines and cannot recover missing upstream facts. Intent is separate input.
- `original-top-level-containment` remains a bounded diagnostic regression. Treating
  it as the main product path was the planning mistake; it does not admit methods.
- PRD, SPEC, ARCH, SOP, AGENTS and the status board now use full-map acceptance.
  REQ-RUST-044.0 states the boundary; new test IDs are explicitly queued, not passing.
- Existing HLD implementation, compatibility contracts, rollback safeguards and old
  evidence are retained. Nothing is deleted or marked complete by this scope change.

| Earlier subparts | Current disposition |
| --- | --- |
| 1.1–1.10 | Active node/signature/body/call work; 1.10 ends at frozen full-map evidence, not HLD |
| 2.1–2.10 | Active semantic edges; only absent-from-itoa `pub use` policy moves to BD-05 |
| 3.1–3.10 | Active source/expansion/core semantics; 3.3's derived roll-up attribution is deferred, actual provenance is not |
| 4.1–4.5 | Active freeze/reference/full-map/evidence verification; HLD expected rows are deferred |
| 4.6–4.7 | Deferred interface/collapse/all-five-HLD-output work, BL-01–BL-06 |
| 4.8–4.10 | Active full-map generation safety, determinism and measured verification; not HLD prerequisites |

**Expected / actual / next:** `default -> Buffer::new` is still missing. The last
verified runtime snapshot has 33 impl headers, not a resolved semantic call. Next:
original item/member nodes and source-container edges, then body/type-relative lookup,
exact call evidence and frozen full-map readback. Generic `format -> Sealed::write` follows.

**Evidence scope:** documentation-only Spec/Review work. No production code, database,
build or runtime test was changed/run; the previous 362-test result is not a new result.
All 40 earlier subparts and dated entries remain below as history. No commit or push.

Documentation checks passed: links added by this change resolve, Markdown fences balance,
all 67 prior requirement definitions remain, and earlier log/journal bytes are preserved.
All 134 inventoried Rust/SQL/TOML source files have unchanged digests. These are document
integrity checks, not acceptance of the newly specified full-map behavior.

---

## 2026-11-14 16:24 IST — User-requested Cargo and temporary-file cleanup

- Ran `cargo clean --offline --locked` with explicit manifest and target paths
  for the integration workspace, research workspace and nested oracle harness.
  All three commands exited successfully; their target directories are gone.
- Removed **13 unused temporary paths**, approximately **339.7 MiB** of logical
  file contents: three abandoned test-fixture databases/workspaces, stale compiler
  assembly files, an installer extraction directory and a Node compilation cache.
  Checked process references before deletion; active service files were retained.
- Net disk reclaimed: **11.54 GiB**. Available space increased from **4.98 GiB**
  to **16.51 GiB**. These are filesystem measurements, not Cargo's logical totals.
- Preserved source, documentation, pinned repositories and run evidence. Before
  this log update, all **1,106 inventoried source/document files** and both Git
  status snapshots were unchanged. The temporary checkout with uncommitted source
  changes was deliberately retained. IDE processes were resumed after cleanup.
- Archived four executable artifacts with checked SHA-256 digests, including the
  last verified production executable and source-acceptance harness, outside Cargo
  targets. Cleanup inventories, command output, binaries and measurements live in
  `/home/ubuntu/hld-current-test-artifacts/20261114T105323Z-cargo-temp-cleanup/`.
- No production change, rebuild, test rerun, commit or push. Earlier verification
  remains historical evidence for the preserved executable; the next build will
  be cold. No native-sysroot retry or new semantic-acceptance claim.
- Next source-backed target remains **1.10**, `default -> Buffer::new`; missing
  member-node/signature/body-path producers remain unfinished.

---

## 2026-11-14 15:56 IST — Bounded verification recovered and passed

- Reclaimed only the inactive research checkout's rebuildable debug cache;
  source, release executables and run evidence were preserved. Free space reached
  **5.0 GiB**. IDE background processes paused for verification were resumed.
- On the unchanged production/source artifact, **all 9 source-acceptance tests
  passed** in **59.73 seconds**, including the impl-header test and the structural
  `area:src/lib.rs -> Buffer` publication test.
- Bounded workspace: **362 passed / 0 failed / 4 ignored / 1 filtered**, across
  **21 harness results**. Sampled wall time **133.375446 seconds**; sampled
  process-tree peak **512,983,040 bytes**. Memory was measured, not capped; this is
  not final memory qualification.
- The filtered test remains `test_lang_items_in_sysroot`. Thus the bounded run
  passes, but the unrestricted workspace/full-native/SOP gates do **not** pass.
- Fresh formatting, warnings-denied release Clippy and release build, all targets
  and features: **PASS**. The syntax-only final-stage regression is fixed and
  passes in the workspace run.
- Production executable SHA-256:
  `18ac2591bc0e973c880936eef51ae834eb71ee2030abcef6f15734d55afe352e`.
  Source manifest SHA-256 (121 build/test/measurement source files):
  `c2b4beb33718eb9e9eaa8245cb635b19bc1bf95cedae83e4a0772c751929cd12`.
  Both were checked unchanged across the recovered run.
- **Still not proven:** the `default -> Buffer::new` call, complete signature
  lowering, complete member-node containment, generic dispatch or whole-itoa HLD.
- Subpart **1.3** has real-source header-path evidence and recovered gates; its
  additional negative/ownership checks remain follow-up work. The next semantic
  relationship remains **1.10**, `default -> Buffer::new`.
- No commit or push. The 40-subpart baseline below is retained unchanged.

---

## 2026-11-14 15:53 IST — Bounded run blocked by disk exhaustion

- The syntax-only regression now passes. Fresh formatting, warnings-denied
  release Clippy and the release all-target/all-feature build pass.
- The bounded workspace test run stopped in the source-acceptance harness:
  **8 passed / 1 failed**. `test_impl_header_row_contents` passed again;
  `test_itoa_top_level_containment` failed when DuckDB could not write its WAL:
  `No space left on device`. The remaining workspace harnesses were not reached.
- This is an environmental failure, not a completed structural/semantic verdict.
  No full workspace PASS is claimed.
- Next: inspect storage, preserve source/evidence, reclaim only rebuildable
  caches and rerun the bounded checks. No native-sysroot retry.

---

## 2026-11-14 15:49 IST — Syntax-only regression isolated

- An existing source-backed CLI regression discovered a boundary error: requesting
  `syntax-spans` still invoked the new impl-header producer, which correctly
  rejected the absent name-resolution snapshot. One selected test failed; 29
  were filtered.
- The invocation is now gated on stages that include name resolution. Its fresh
  regression rerun and subsequent workspace gates are pending.
- Timestamp correction: the preceding Clippy recovery entry was mistakenly
  headed `15:50`; that gate result was already observed by `15:48 IST`. Its result
  remains accurate. Entries are retained below as originally recorded.
- No change to graph semantics, source-container policy or the active
  `default -> Buffer::new` relationship target.

---

## 2026-11-14 15:50 IST — Formatting and Clippy recovered

- Moved the impl-header test module after the production items without changing
  behavior or suppressing the lint.
- `cargo +stable fmt --all -- --check`: **PASS**.
- Warnings-denied release Clippy, all targets and features: **PASS**.
- Workspace test executables are being rebuilt. Bounded tests and a fresh source
  acceptance rerun are still outstanding; the prohibited native sysroot retry is
  not part of this gate.
- Subpart 1.3 remains partial: header-path coverage is not complete signature
  lowering or the `default -> new` edge.

---

## 2026-11-14 15:46 IST — Quality-gate follow-up

- The pinned-source impl-header acceptance test remains a narrow passing result.
- The subsequent warnings-denied release Clippy gate **failed** with
  `clippy::items_after_test_module` in the new impl-header producer. The test module
  must move after the production items; no lint suppression is intended.
- Formatting was applied before that Clippy run. The remaining fresh quality
  gates are not yet complete.
- Next: fix the module ordering, rerun the gate and continue header verification.
  This does not change the four-part breakdown below or establish `default -> new`.

---

## 2026-11-14 15:45 IST — Initial four-part breakdown

### Current evidence and scope

**The impl-header test passed:** all **33 active itoa impls** have header rows.
The test also verifies `Default`'s `Buffer` self-type, member source evidence and
the explicit missing-core reason. **This is header coverage, not the
`default -> Buffer::new` call.** Fresh quality gates remain outstanding.

Measured run: one selected test passed, eight filtered; **103.071636 seconds**
sampled elapsed and **434,118,656 bytes** sampled process-tree peak. Scope:
pinned itoa library, Rust 2021, x86_64-unknown-linux-gnu, no features,
`cfg(test)=false`, sysroot excluded. Commit:
`af77385d0daf4d0e949e81f2588be2e44f69f086`.

The earlier structural checkpoint produced **17 full nodes / 15 full edges**,
then **8 interface nodes / 6 HLD edges / 6 evidence paths / 2 file roll-ups /
1 folder roll-up**. It proved `area:src/lib.rs -> Buffer`, storage, derivation and
publication on that tested snapshot. Subsequent production edits require fresh
evidence; these numbers do not establish complete semantic coverage.

No whole-itoa or formal SOP milestone PASS is claimed. Nothing has been committed
or pushed for this implementation cycle.

### How this breakdown was grounded

Codebase-memory was used to locate production stages, trace the publication
facade in both directions and inspect the implementation boundaries. Its
integration-project index was dated `2026-11-14T04:11:02Z` and contained 26,147
nodes and 55,279 edges. Coverage was checked for 13 evidence paths and four crate
scopes. Changed/untracked source and relevant partially parsed SQL ranges were
read directly. The index is a navigation aid, not semantic acceptance evidence.

### Reading the tables

- **Clear:** the required behavior is known. This does **not** mean implemented.
- **Build:** implementation is missing or incomplete.
- **Verify:** supporting code exists, but whole-itoa proof is missing.
- **Decide:** the current specification still leaves a choice open.

These are **four parts with ten subparts each**, not 40 equally sized tasks.

### Part 1 — Complete the item graph and prove `default -> Buffer::new`

| ID | Subpart | What is clear | What remains |
| --- | --- | --- | --- |
| 1.1 | Item inventory | Include public/private non-body items, trait/impl members and non-body `use` declarations. Impl blocks are records, not nodes. | **Build:** extend beyond original top-level nodes. Field/variant policy remains separate and open. |
| 1.2 | Canonical item identities | Identity includes compilation unit, source range and expansion provenance, not just a name. | **Verify:** extend existing identity checks to every member and nested item. |
| 1.3 | Impl-header records | Record self type, trait, members, source container and unresolved reasons. | **Narrow PASS:** 33 headers. Refactor, negative tests and fresh gates remain; full signature lowering is not done. |
| 1.4 | Source containment | User decision: one lexical source container; impl ownership stays separate. | **Build:** emit member/nested-item containment. **Decide:** relationship between a file's module node and its area node. |
| 1.5 | Signature lowering | Parameters, return types, `Self`, generic parameters and bounds need semantic identities. | **Build:** turn syntax/signature facts into usable type records; headers alone are insufficient. |
| 1.6 | Body scopes and names | Respect parameters, local bindings and shadowing. Existing body-site text is not resolution. | **Build + decide AD-2:** how body-name lookup reuses SQL binding lookup while Rust handles lexical semantics. |
| 1.7 | `Type::member` lookup | `Buffer::new` must select the unique applicable associated item; ambiguity must not become a guess. | **Build:** inherent/trait candidate lookup backed by impl records. |
| 1.8 | Call-edge production | Each extracted call needs canonical endpoints and its exact call-site evidence. | **Build:** produce `default -> Buffer::new` in the full graph. |
| 1.9 | Effective visibility | Visibility and outside-crate reachability are separate; trait-impl members inherit trait visibility. | **Build + decide AD-3:** complete the visibility stage. Missing core trait facts cannot be treated as private or invented. |
| 1.10 | First semantic round trip | Verify the persisted full-graph call first; verify HLD inclusion when eligibility is established. | **Verify:** source -> call edge -> frozen readback -> HLD/evidence. This is the next relationship checkpoint. |

### Part 2 — Resolve the remaining kinds of dependency

| ID | Subpart | What is clear | What remains |
| --- | --- | --- | --- |
| 2.1 | Local/receiver typing | Calls need types for parameters, `self`, locals and intermediate expressions. | **Build + decide AD-4:** bounded inference scope versus wider Rust inference; unsupported cases need reasons. |
| 2.2 | Generic-bound records | Preserve facts such as `I: Integer` and their substitutions. | **Build:** semantic bound records consumed by lookup, not matching type-name strings. |
| 2.3 | Supertrait traversal | Follow `Integer: private::Sealed`; handle cycles without looping. | **Build:** canonical bound traversal with cycle/distractor tests. |
| 2.4 | Concrete method selection | Use receiver type and applicable impls; never select solely because the method name matches. | **Build:** receiver adjustments, argument/cast typing and concrete impl selection needed by itoa. |
| 2.5 | Generic method selection | `i.write(...)` targets **one `Sealed::write` declaration**, not every integer implementation. | **Build:** hardest bounded target; prove `format -> Sealed::write`, including ambiguity and rename tests. |
| 2.6 | Associated-type projections | Preserve types such as `I::Buffer` and their relationship to bounds. | **Build:** projection/substitution handling sufficient for the selected method signature. |
| 2.7 | Trait-implementation edges | An `implements` fact joins the actual self type and trait; primitive types do not get fabricated local declarations. | **Build:** emit resolved relations; retain missing-core cases as unresolved. |
| 2.8 | Type-use edges | Type dependencies target semantic items, not import lines. | **Build + decide OD-8:** which non-function positions—fields, associated types, array lengths—emit edges and who owns them. |
| 2.9 | Const/static reads | Actual reads of consts, statics and associated consts become `reads`, not calls. | **Build + decide OD-8:** non-function initializer ownership; function-body reads are already specified. |
| 2.10 | Imports and reexports | A `use` node imports resolved bindings; merely having access to an import creates no function dependency. | **Build:** import edges. **Decide later:** dual imports/reexports treatment of `pub use`; itoa has no `pub use`. |

### Part 3 — Apply those semantics to macros and core dependencies

| ID | Subpart | What is clear | What remains |
| --- | --- | --- | --- |
| 3.1 | Active expanded-item inventory | Cfg-active macro-generated items must be accounted for alongside original items. | **Verify + build:** expansion infrastructure exists; complete graph-node production is not proven. |
| 3.2 | Expansion identities/hygiene | Keep invocation ancestry and hygiene so repeated expansions cannot collide. | **Verify:** carry these identities through semantic endpoints, not only parser/namespace rows. |
| 3.3 | Generated-item file attribution | Expansion provenance must survive into containment and file/folder summaries. | **Decide:** which source file owns a generated item for those projections. |
| 3.4 | Generated impls/signatures | Reuse the same semantic producers; no macro-specific name shortcuts. | **Partial:** the header test includes expanded impls. Rich type/bound lowering still needs implementation. |
| 3.5 | Generated function bodies | Expanded bodies need the same scopes, typing and call resolution as original bodies. | **Build:** connect expanded body facts to semantic edge production. |
| 3.6 | The concrete `u128` chain | Prove `write -> fmt -> div_rem_1e16 -> mulhi` as three full-graph edges before HLD collapse. | **Build + verify:** exact concrete targets and every intermediate proof step. |
| 3.7 | Body-position macros | Supported expansions need body provenance; unsupported sites remain explicitly unresolved. | **Build/verify:** body expansion handling and complete unresolved-site accounting. |
| 3.8 | Core source ingestion | External identities must come from real source; sysroot exclusion is only a local diagnostic scope. | **Operational blocker:** native runs hit time/disk limits. A measured re-plan is needed before retrying or claiming complete scope. |
| 3.9 | Core-dependent typing | `Default`, `Clone`, primitive methods, pointers and slices can affect local target selection, not just external edges. | **Build/verify:** real trait/method signatures and relevant lang-item semantics. No canonical-path fabrication. |
| 3.10 | Coverage accounting | Every in-scope item/site needs a result, an unresolved reason or an explicit exclusion. | **Verify:** no silent gaps among expanded bodies, operators, body types or repeated same-line sites. |

### Part 4 — Prove the complete output is correct

| ID | Subpart | What is clear | What remains |
| --- | --- | --- | --- |
| 4.1 | Freeze the tested inputs | Pin source, configuration, toolchain, executable and generation. | **Verify:** record one fresh semantic acceptance artifact; old structural evidence cannot qualify new code. |
| 4.2 | Freeze an independent reference | Expected facts must never feed production. The current source-reviewed draft is not a verified oracle. | **Decide/verify AD-5:** review/oracle cross-check approach; close affected graph-shape choices and canonicalize expectations. |
| 4.3 | Full-node/full-edge comparison | Check missing, extra and wrong facts, including private-to-private dependencies. | **Build/verify:** exact whole-itoa comparison, not only a path-resolution comparator or row counts. |
| 4.4 | Unresolved/inactive comparison | Unresolved is acceptable only where the scoped contract permits it and records the reason. | **Verify:** match the entire expected inventory; never hide missing edges by relabelling them unresolved. |
| 4.5 | Backward source proof | Every edge must trace through persisted records to exact original/expanded source evidence. | **Verify:** extend the proven structural backward check to all semantic edges. |
| 4.6 | Interface selection and collapse | Preserve full-graph facts; collapse only allowed dependency paths—**never `contains` or `imports`**. | **Verify:** complete semantic inputs, visibility and evidence paths. No new calls/reads-only restriction is assumed. |
| 4.7 | All five HLD outputs | Compare interface nodes, dependency edges, evidence paths, file roll-ups and folder roll-ups. | **Verify:** whole-output equality. Existing structural/component passes are narrower. |
| 4.8 | Publication failure safety | Frozen inputs must stay consistent; cancellation/failure must preserve previous serving output. | **Verify:** existing component safeguards against the complete semantic pipeline and stale generations. |
| 4.9 | Worker determinism | Equivalent semantic output at 1/4/16 workers; operational attempt IDs are not semantic differences. | **Verify:** repeated whole-itoa runs and exact normalized comparisons. |
| 4.10 | Gates and measurements | Formatting, Clippy, tests, build, wall time, process-tree memory and disk usage are required evidence. | **Verify:** fresh gates and resource measurements. Full-native and formal SOP qualification remain unpassed. |

### What is unclear versus merely unfinished?

The overall architecture is clear: Rust produces semantic facts; Turso
persists/freezes/publishes them; DuckDB performs relational analysis and derives
HLD. We do not need another database or another architecture.

The largest unresolved areas are complete core handling, generated-item file
attribution, non-function dependency ownership, the remaining module/area
relationship, the body/visibility stage choices, and the independently verified
reference.

The hardest bounded implementation is **2.5, generic method lookup**. The largest
operational uncertainty is **3.8, core ingestion**.

```text
33 impl headers proven
        |
        v
member nodes + signatures + body/type-relative lookup
        |
        v
default -> new proven in the full graph
        |
        v
generic call + macro-generated concrete chain
        |
        v
complete reference comparison + all five HLD outputs
```

Execute that path in small source-backed slices, rather than building all 40
subparts independently and attempting one large integration at the end.

### Source and evidence pointers

- [Current PRD, including AD-1 through AD-5](PRD-itoa-full-item-graph-scope.md)
- [Current executable SPEC, including open graph contracts](SPEC-wider-scope-requirements-not-current.md)
- [Current pipeline architecture](ARCH-itoa-full-item-graph-pipeline.md)
- [Current status board](../phase-20261111/current-milestone-status-board.md)
- [Append-only TDD evidence journal](../phase-20261111/TDD-phase-20261111-progress-checkpoints.md)
- Previous structural checkpoint report `hld-current-test-runs/20261114T043000Z-structural-verification/` (report removed from the tree on 2026-11-14; last present at commit a2e13a3bd)
- [Production publication facade](../../crates/parseltongue-cli-dispatcher-binary/src/hld-source-publication-facade.rs)
- [Impl-header producer](../../crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs)
- [Source acceptance tests](../../crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs)
