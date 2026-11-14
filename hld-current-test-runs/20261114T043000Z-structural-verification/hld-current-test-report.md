# Structural source-to-HLD verification

**Phase 1 development checkpoint: PASS. Full SOP milestone: NOT PASS.**

## Scope and identity

- Rust Work Mode: Delivery + Reliability; original-source structural projection only.
- Pinned itoa: `af77385d0daf4d0e949e81f2588be2e44f69f086`, library target, Rust 2021,
  `x86_64-unknown-linux-gnu`, no features, `cfg(test)=false`, sysroot excluded.
- Repository base: `4f499f708a3f8d4e82fa2954ed42e774891e9914`, with uncommitted changes.
- Compiler: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
- Current release CLI SHA-256:
  `4991262d573cdc462973033d82d860835096fc2aa3bbb2cb5fabdf12ce2bbe54`.
- Source-test executable SHA-256:
  `9a976538a4061c2090f30c9f1e2a9f7a92e0128d6d078330f9b1731b55f322fa`.
- Cargo.lock SHA-256:
  `c0d6fab89ac1a0b183f9fb31074d138dc629515aacfd224e590a3aa3a7888b47`.
- The 117-file workspace Rust/SQL/Cargo input manifest SHA-256:
  `8955b37266d12d20e18fb8c89dd4a4ba71cb2f5db35c4b254555efcd49ed2eaa`.
  It is a workspace input inventory, not a digest of all external compiler/dependency sources.
- Raw evidence: `/home/ubuntu/hld-current-test-artifacts/20261114T043000Z-structural-verification/`.
  `artifact-sha256-manifest.json` records log/measurement/source-export hashes.
- No commit or push was made for this work.

## Proven real-source path

```text
pinned source bytes + spans + frozen namespace ownership
  -> source-derived full graph inputs
  -> persisted Turso graph inputs
  -> frozen readback
  -> DuckDB HLD derivation
  -> atomic Turso publication
  -> published-row and exact-source readback
```

`area:src/lib.rs --contains--> Buffer` passes on the current executable.
The exact focused run selected **1 test**, with **7 filtered**, and passed:

```text
stage=original-top-level-source-graph units=1 nodes=17 edges=15
stage=original-top-level-hld interface_nodes=8 dependency_edges=6 evidence_paths=6
observed-hld-rows interface_nodes=8 dependency_edges=6 evidence_paths=6 file_rollups=2 folder_rollups=1
test test_itoa_top_level_containment ... ok
```

Measured elapsed **17.364 seconds**, sampled process-tree peak **422,227,968 bytes**.
The source projection reports an effective DuckDB limit of **25.0 GiB**; no interim
correctness memory cap was added. These values concern the structural slice, not full semantic itoa.

## Regressions and corrections

- Visibility is read from structured namespace context, not a `pub(...)` text prefix.
  Public, crate, ancestor and private-equivalent restrictions—including comments/spacing—are tested.
- Child-file owners and qualified paths come from namespace ownership; same-named items remain distinct.
- Private-equivalent nodes remain in the full graph; ordinary HLD interface selection excludes them.
  The explicit top-level-use exception remains.
- Missing, conflicting, stale and malformed namespace contexts fail closed. The strict nonempty
  import-range check is covered by `test_source_namespace_input_validation`.
  Exact selected run: **1 passed / 149 filtered**, **0.242 seconds / 74,842,112 bytes**.
- The apparent missing-reason defect was a **test matcher defect**, not a production defect.
  All 22 expected `external_source_missing` rows already existed. Ten expanded
  `MaybeUninit <u8 >` spellings did not match the raw `MaybeUninit<u8>` assertion.
  Matching now ignores whitespace for these token-path assertions, while all **202 path texts**
  are checked byte-for-byte against their persisted source files and ranges.
  The unchanged totals remain **164 resolved / 38 unresolved / 22 external-source reasons**.
  The other **16 type-relative sites remain unreasoned** at this stage; semantic completeness is not claimed.
  Exact selected run: **1 passed / 7 filtered**, **15.410 seconds / 390,660,096 bytes**.

The current source-acceptance target passes **8 tests**: pinned-source intake/containment,
scope exclusion/reasons, plus scope, visibility, module-owner and mutation fixtures.

## Quality gates on this source

Commands used `CARGO_BUILD_JOBS=2`, `CXXFLAGS=-g0`, stable, release, locked and offline.

- Formatting: **PASS**, `cargo +stable fmt --all -- --check`.
- Clippy: **PASS**, `cargo +stable clippy --release --locked --offline --all-targets --all-features -- -D warnings`.
  **0.492 seconds / 339,062,784 bytes**. Cargo still emits upstream unused-manifest-key notices;
  the Rust lint command exits successfully.
- Build: **PASS**, `cargo +stable build --release --locked --offline --all-targets --all-features`.
  **1.251 seconds / 914,939,904 bytes**.
- Bounded workspace tests: **PASS**, **360 passed / 0 failed / 4 ignored / 1 filtered**, across 21 harness results.
  **77.805 seconds / 848,207,872 bytes**:

```sh
cargo +stable test --release --locked --offline --all-targets --all-features \
  --no-fail-fast -- --test-threads=2 --skip test_lang_items_in_sysroot
```

Test discovery logs verify both exact selections and the workspace suite. Compile-only output is
not counted as behavior evidence. Budgets were E60/stop120 for exact tests and E120/stop240
for the final bounded workspace/gate commands. Peaks are sampled process-tree RSS, not parent-only RSS.

## Full gate failure and exclusions

The earlier unrestricted workspace command **failed** in two targets:
the raw-text matcher and `test_lang_items_in_sysroot`, which hit Turso
`pwrite: no storage space` after extracting 618 units / 358,595 syntax records.

Including that full-native test was a process error: the earlier native stop rule was still in force.
Its evidence remains in `workspace-test.log`; it was **not retried**. Only inactive, rebuildable
debug/build and debug/incremental caches from the research checkout were removed, freeing 4.4 GiB
(1.2 to 5.6 GiB available). No source or run evidence was deleted. The matcher was fixed and the
bounded suite rerun with the native test explicitly excluded. **This is not a full-workspace PASS.**

## Formal SOP status on this artifact

```text
S1      NOT STARTED — formal current-artifact corpus run
S2      NOT STARTED
S3      BLOCKED — full native qualification; bounded structural itoa progress only
S4      NOT STARTED
S5      NOT STARTED
S6      NOT STARTED
S7      NOT STARTED
S8      NOT STARTED
S9      NOT STARTED
S10     NOT STARTED
```

Codebase-memory was refreshed: **26,147 nodes / 55,279 edges**, **6 partial SQL parses**,
**0 unusable / 0 skipped**. Relevant coverage was checked and source inspected directly;
this index is navigation evidence, not semantic qualification.

## Next frontier

The structural checkpoint is closed, not the whole skeleton or whole itoa graph.
Impl headers, complete nested/associated node coverage, imports/dependency edges, body lookup and
generic method resolution remain. `default -> Buffer::new` and
`format -> private::Sealed::write` are **not implemented or claimed passing**.

SPEC §8 / PRD OD-7 explicitly leave impl-member containment undecided. Before extending the
skeleton, choose its source container. Recommendation: one lexical source container per node
(file area for file-level impl members; enclosing inline module for nested impls), with the
impl/self-type/trait link in owner metadata—not an additional containment parent. No
`contains`/`imports` collapse is proposed. Generated ownership and non-function dependency
decisions remain separate. The six proposed SOP/AGENTS process corrections are still unapplied.
