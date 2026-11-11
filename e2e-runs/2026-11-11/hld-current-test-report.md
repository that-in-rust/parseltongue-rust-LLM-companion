# M1: implementation gates and measured corpus evidence

Status: M1 S1–S3 measured on all five; M2 semantic agreement not yet implemented. Date: 2026-11-11.
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

Completed tools/corpus-production-stage-measurements.py with the unchanged release binary from
2c461fd80, four workers and isolated Cargo discovery caches. Artifacts are in each repository's
m1-prepared-final-measurements directory. Each repository ran three times: untraced measurement,
separate exec/connect/process-ancestry audit, then separate filtered openat audit.
Elapsed/RSS below come ONLY from the first run, not either audit. RSS is sampled process-tree VmRSS
in GiB, including Cargo/rustc descendants but excluding supervisors; it is not a kernel high-water mark.
The sampler traverses child/thread relationships rather than scanning the host's entire /proc.

All five mapper, status, process-audit and target-audit commands exited zero. Every tiling identity
holds; every measured peak is below 2 GiB; all process audits have zero violations.

```text
rust-analyzer
  files: 5702; spans: 120127; fields: 256207; body sites: 655973
  tiling: PASS; untraced wall: 228.278 s; sampled peak tree RSS: 0.669 GiB
  exec/connect: PASS; filtered target access: PASS; sampler CPU: 5.413 s
ripgrep
  files: 1039; spans: 21434; fields: 40231; body sites: 106389
  tiling: PASS; untraced wall: 32.615 s; sampled peak tree RSS: 0.166 GiB
  exec/connect: PASS; filtered target access: PASS; sampler CPU: 0.878 s
tokio
  files: 1239; spans: 8632; fields: 18581; body sites: 27492
  tiling: PASS; untraced wall: 10.886 s; sampled peak tree RSS: 0.108 GiB
  exec/connect: PASS; filtered target access: PASS; sampler CPU: 0.239 s
serde
  files: 354; spans: 4150; fields: 5848; body sites: 19735
  tiling: PASS; untraced wall: 4.594 s; sampled peak tree RSS: 0.099 GiB
  exec/connect: PASS; filtered target access: PASS; sampler CPU: 0.101 s
cargo-modules
  files: 5597; spans: 114015; fields: 258302; body sites: 636893
  tiling: PASS; untraced wall: 222.112 s; sampled peak tree RSS: 0.485 GiB
  exec/connect: PASS; filtered target access: PASS; sampler CPU: 5.063 s
```

Counts include mapped dependency/sysroot contexts; files include byte-only Rust files outside the
active module closure, whereas spans/fields/body sites come from active mapped contexts.
The exec audit preserves clone/fork events solely to validate ancestry and the approved Cargo
discovery grammar. No compilation, build-script executable or network connect was accepted.
The target path filter is exact and NOT recursive: a clean filtered trace is not evidence of a
recursive directory audit. Source guards separately reject traversal of target descendants.
The raw audit result explicitly records this limitation.

### Performance findings and remaining budget miss

The broad process/openat/network trace and its mapper were killed; its runtime/RSS are rejected.
The old sampler consumed approximately 54.1% CPU scanning unrelated processes. The replacement
used 5.413 CPU seconds over rust-analyzer's 228.278-second run (approximately 2.4% of one CPU).

The old final-generation publication rewrote every output row in one MVCC transaction and reached
approximately 2.35 GiB observed mapper RSS while still running. Discovery now closes source inputs,
prepares the final generation before syntax outputs, and writes bounded file units directly into it.
Publication is a manifest-pointer transaction, not another corpus-wide rewrite.

```text
rust-analyzer measured stage profile
  crate graph: 0.338 s; initial source ingest: 12.198 s
  source closure: 15.504 s; generation preparation: 2.553 s
  extraction: 2612 units; 1136982 total output records
  extraction load: 0.492 s; pure parse: 7.493 s; store writes: 182.383 s
  syntax extraction total: 190.742 s; final publication: 0.001 s
cargo-modules measured extraction
  2535 units; 1107775 output records; store writes: 179.166 s
  final publication: 0.002 s
```

Writes are NOT one transaction per row: each file unit uses one Concurrent transaction, with
multi-row INSERT statements of up to 128 records and one completion marker. Status aggregation
streams large tables and uses SQL COUNT rather than retaining all output payloads.
The rust-analyzer and cargo-modules wall times remain above the approximate three-minute goal.
The remaining measured hot stage is persistence, not parsing or final publication.

Release-mode synthetic profiles (12288 rows, 1024-byte opaque payloads, in-memory Turso) tested
whether larger statements or transactions improve this. These are measured microbenchmarks,
not corpus timings or projected corpus speedups:

```text
bound VALUES, rows per statement → wall seconds
  32: 0.616; 64: 0.639; 128: 0.614; 256: 0.688; 512: 0.612; 1024: 0.602
JSON json_each, rows per statement → wall seconds
  128: 1.230; 1024: 0.768
bound VALUES, rows per atomic unit (fixed 128-row statements) → wall seconds
  64: 0.681; 256: 0.565; 1024: 0.605; 4096: 0.631
```

Increasing statement size gives no substantial win in this sample; JSON batching is slower.
Larger transactions do not show a monotonic benefit. Those candidates were not enabled in
production, and file atomicity was not weakened. Optional per-unit setup/cleanup/insert/commit
timing and ignored reproducible profiling tests were added for deeper corpus diagnosis.
M2/S4/S5 semantic exports and agreement percentages remain unmeasured; S1–S3 do not imply them.

The subsequent untraced per-unit profile (m1-write-phase-profile, diagnostics enabled) exited zero:
192.941 s wall, 0.664 GiB sampled tree RSS. Its 2612 extraction file units wrote 1136982 records;
summed setup was 0.460 s, cleanup 0.846 s, INSERT 41.243 s, and Concurrent commit 106.921 s.
The measured remaining hot operation is the pinned Turso commit, not per-row transactions or
DELETE cleanup. This separate diagnostic run is NOT substituted for the all-five measurements
above and its faster wall time is not attributed to the instrumentation as a performance fix.
