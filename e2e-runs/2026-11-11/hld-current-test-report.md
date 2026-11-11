# M1: implementation gates and measured corpus evidence

Status: GREEN and REFACTOR pushed; S3 corpus measurements in progress. Date: 2026-11-11.
Pins and commands: docs/phase-20261111/corpus-source-pin-manifest.json.

## Measured corpus prerequisites

Each repo ran cargo fetch once and cargo check --workspace once, outside Parseltongue.
All ten commands exited zero. Cargo-reported check durations, not total fetch/check wall time:

```text
rust-analyzer: fetch exit 0; check exit 0; Finished dev profile in 1m 19s
ripgrep:       fetch exit 0; check exit 0; Finished dev profile in 34.12s
tokio:         fetch exit 0; check exit 0; Finished dev profile in 11.08s
serde:         fetch exit 0; check exit 0; Finished dev profile in 5.79s
cargo-modules: fetch exit 0; check exit 0; Finished dev profile in 59.63s
```

Evidence: <repo>/corpus-fetch-output-log.txt and <repo>/corpus-check-output-log.txt.
Build outputs: /home/ubuntu/m1-corpus-build-targets/<repo>/, outside source inputs.
Matching toolchain rust-src library sources exist. Real PRIMARY semantic exports are not yet implemented;
successful checks alone do not establish complete P3 oracle/proc-macro availability.

## Resolved process-policy amendment

The amended REQ 001 retains the three direct commands and permits Cargo's rustc version and print-only
discovery descendants. The required metadata command on the rust-analyzer checkout itself executed:

```text
rustc -vV
rustc - --crate-name ___ --print=file-names --crate-type bin --crate-type rlib
  --crate-type dylib --crate-type cdylib --crate-type staticlib --crate-type proc-macro
  --print=sysroot --print=split-debuginfo --print=crate-name --print=cfg -Wwarnings
```

This was observed with strace -f -e trace=execve, not inferred from Cargo source.
Full trace: rust-analyzer/metadata-process-preflight-trace.log.
The same command on the already warm Parseltongue workspace did not execute rustc, so relying on
warm caches can conceal the incompatibility. This was pre-implementation policy evidence.

The user approved the six fixed crate-type selector pairs and -Wwarnings only within the stdin ___
discovery probe. SPEC/SOP and the auditor now implement that grammar. Unknown flags, output/emit
arguments, real crate inputs, build-script binaries and network remain forbidden. Positive and negative
auditor contract tests pass. A metadata-created .rustc_info.json is compiler discovery provenance;
only that exact target-cache basename is permitted, not saved build outputs.

## Commits and TDD evidence

- Prep: f5aa564d8d689db780ebe9b4ceb5142080e3ce29.
- STUB: d91ac40694a618c74816637773af5d97013e46cd.
  cargo check --workspace: Finished dev profile in 29.70s.
- RED: 63b5406d4b05ebecfeaabc42189a6e95d27ff361.
  Graph 0 passed/6 failed; syntax 0/6; scheduler 0/2; store 0/9; CLI 0/1.
  These 24 Rust failures are expected todo!() panics. Python RED: 2 NotImplementedError errors.
  The existing offline dependency gate separately hit uncached objc2; cargo fetch --locked provisioned it.
- GREEN: 96a22ac38. Production implementations and 31 passing Rust tests.
- REFACTOR: ab958d89c. Formatted implementations, allocation/condition cleanup and production driver.
- Python memory sampler RED: NotImplementedError; then Ran 3 tests in 0.115s; OK.

Final gate output:

```text
cargo fmt --all --check: exit 0
cargo clippy --all-targets -- -D warnings:
  Finished dev profile [unoptimized + debuginfo] target(s) in 0.29s
cargo test --workspace:
  test result: ok. 1 passed; 0 failed;   CLI
  test result: ok. 3 passed; 0 failed;   core
  test result: ok. 6 passed; 0 failed;   graph
  test result: ok. 2 passed; 0 failed;   scheduler
  test result: ok. 6 passed; 0 failed;   syntax
  test result: ok. 1 passed; 0 failed;   fake store
  test result: ok. 9 passed; 0 failed;   Turso stage store
  test result: ok. 3 passed; 0 failed;   build gates
cargo build --release -p parseltongue-cli-dispatcher-binary:
  Finished release profile [optimized] target(s) in 4.32s
```

Implementation failures found and fixed: optional publication decoding needed a Turso error type
annotation; the CLI fixture's handwritten lock was noncanonical and correctly rejected by the lock
guard (fixture now qualifies its lock offline before mapping); the fixture inventory was four files,
not three, because build.rs is bytes-only input too. Clippy rejected test unwraps and unnecessary
allocations/nested conditions; only tests explicitly allow unwrap, and production warnings remain denied.

## S1 and S2 measured inventories

Fresh index commands exited zero for all five. Saved index statuses are ready; graph schemas and
list_projects output are also saved. Each corpus-index-coverage-summary.json records its generation.
All scope and missed-File pages are saved, including output-budget continuation of missed graph pages.

```text
rust-analyzer
  nodes: 38467; edges: 229944
  partial files: 137; unusable files: 148
  scope findings: 294 in 15 pages; missed Files: 285 in 2 pages
  amended S2 complete-inventory gate: PASS (294 accumulated == scope_total)
ripgrep
  nodes: 4855; edges: 22699
  partial files: 3; unusable files: 0
  scope findings: 6 in 1 page; missed Files: 3 in 1 page
  literal S2 final-page gate: PASS
tokio
  nodes: 12569; edges: 64034
  partial files: 0; unusable files: 0
  scope findings: 2 in 1 page; missed Files: 0 in 1 page
  literal S2 final-page gate: PASS
serde
  nodes: 3299; edges: 14352
  partial files: 0; unusable files: 0
  scope findings: 16 in 1 page; missed Files: 0 in 1 page
  literal S2 final-page gate: PASS
cargo-modules
  nodes: 1572; edges: 3580
  partial files: 0; unusable files: 0
  scope findings: 15 in 1 page; missed Files: 0 in 1 page
  literal S2 final-page gate: PASS
```

The old SOP required both has_more=false and scope_truncated=false.
Rust-analyzer's last response (page 14, offset 280) instead reports:

```json
{"scope_total":294,"scope_returned":14,"scope_truncated":true,"has_more":false,"next_offset":null}
```

The approved amendment gates stable generation, advancing offsets, has_more=false and accumulated
entries == scope_total. Per-page scope_truncated is informational. The inventory driver now exits
zero for all five; all 294 rust-analyzer findings remain saved without filtering.
CM-GAP remains missing structural cross-check evidence only, never an excuse for an oracle error.

## S3 production measurements

Running tools/corpus-production-stage-measurements.py with the release binary, four workers, isolated
Cargo discovery caches, strace ancestry/full argv auditing and nominal 10 ms VmRSS tree samples.
The external Python sampler substitutes for the SOP's planned memory-sampler wrapper; supervisors
are excluded from mapper-tree RSS. Per-repo summary JSON, raw samples, trace, stderr and JSON status
are ignored artifacts under each repository directory. No corpus success is implied by fixture gates.
