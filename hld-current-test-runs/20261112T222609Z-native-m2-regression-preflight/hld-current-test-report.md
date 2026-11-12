# Native M2 regression preflight report

**Verdict: S3 BLOCKED/incomplete before launch.** The requested native test was
not started because the persistent filesystem did not have enough free space
for the retained fixture footprint measured in the prior run. This is neither
a timeout nor an assertion failure, and it provides no acceptance result.

## Requested gate

```text
cargo test -p parseltongue-cli-dispatcher-binary --test workspace-cli-contract-tests --all-features --release --locked --offline -- test_lang_items_in_sysroot --exact --nocapture
```

- Expected: 20 minutes / 1,200 seconds.
- Maximum: 40 minutes / 2,400 seconds; unchanged from the previous run.
- Memory cap: none.
- Planned whole-process-tree RSS sampling: nominal 0.2-second interval,
  including Cargo, test, and CLI children; sampler excluded.
- Planned fixture root:
  `/home/ubuntu/hld-current-test-artifacts/20261112T222609Z-native-m2-regression-preflight/fixtures`.
- Integrated source HEAD: `825d760e8685d8d4d95ab8acd2267edf3687847b`; worktree
  was clean.

## Preflight stop

All persistent run paths—the repo, warmed target, and external artifact root—
are on `/dev/root`. It had **3,213,532 KiB available (98% used)**. The original
timed-out native fixture is being preserved and occupies **4.0G**; its largest
files are a 2,430,873,600-byte analysis DuckDB and a 1,824,514,048-byte
`store.db`. Free persistent space was below that previously measured fixture
footprint. `/dev/shm` is temporary and was not substituted. The required stop
condition was met before launch.

Process preflight passed: 372 processes total, `ulimit -u` 128356, and no stale
Cargo/native-test/CLI process. The exact command, environment, toolchain,
source/binary SHA-256s, and preflight evidence are in
`/home/ubuntu/hld-current-test-artifacts/20261112T222609Z-native-m2-regression-preflight/native-m2-run-plan.md`
and `native-m2-preflight-evidence.md`.

The planned supervisor is the previously used finite release-gate wrapper at
`/home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T202154Z-release-profile-tests/measure-cargo-release-gates.py`.
It was not started. No Cargo compilation or test process was launched, so no
DuckDB C++ rebuild occurred. No phase began; live stderr, phase duration,
DuckDB limits, assertion output, elapsed runtime, and peak process-tree RSS are
unavailable for this attempt. No timeout or descendant cleanup result is
claimed.

## Separate historical evidence

The prior native attempt remains preserved at
`hld-current-test-runs/20261112T203325Z-native-m2-regression/hld-current-test-report.md`.
That earlier run timed out at 2,400.11 seconds with exit 124 and no assertion
result; its fixture and forensic evidence were not overwritten or deleted.

The separate CLI contract regression completed before this preflight with 27
passed, 0 failed, 1 ignored (`test_child_fixture_waits_release`), and 1 filtered
(`test_lang_items_in_sysroot`). The filtered native test is not among those 27.

All corpus runs and S4–S10 remain NOT STARTED. This preflight report is not a
native test result, a full SOP run, or a qualification report. No source change,
binary freeze, commit, or push was made.

## Artifacts

- Plan: `.../20261112T222609Z-native-m2-regression-preflight/native-m2-run-plan.md`
- Preflight evidence: `.../20261112T222609Z-native-m2-regression-preflight/native-m2-preflight-evidence.md`
- Machine summary: `.../20261112T222609Z-native-m2-regression-preflight/native-m2-run-summary.json`
