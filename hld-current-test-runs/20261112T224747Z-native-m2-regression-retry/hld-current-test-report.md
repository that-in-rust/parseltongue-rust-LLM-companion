# Native M2 regression retry report — 2026-11-12

This is one bounded native-test attempt on the clean integrated source identity. It is diagnostic evidence only, not a full SOP runner run, S3 acceptance, production comparison, corpus qualification, or release freeze. The earlier timeout and forensic reports are preserved and were not overwritten.

## Gate status

| SOP step | Status | Evidence |
|---|---|---|
| S1–S2 | NOT STARTED | No corpus-source qualification or coverage inventory was run in this retry. |
| S3 | BLOCKED / incomplete | The isolated test reached the 2,400-second supervisor deadline and exited 124. No assertion failure or test-result line was emitted. The supervisor completed process-group cleanup. |
| S4–S10 | NOT STARTED | No production exports/comparisons, corpus run, or qualification invocation occurred. |

All five corpus runs remain NOT STARTED. No all-five PASS, production qualification, or frozen release is claimed.

## Authorized disk cleanup

Only the exact authorized regenerable directory was removed:

~~~text
/home/ubuntu/repos/parseltongue-rust-LLM-companion/tools/oracle-semantic-export-harness/target/debug
~~~

Immediately before removal, canonical path equaled the authorized path, the final component was not a symlink, it was a directory, and lsof reported zero open paths. No other target or artifact was removed.

Persistent /dev/root free space immediately before cleanup was 3,192,228 KiB; immediately after it was 19,760,256 KiB. The latest post-cleanup sample before launch was 19,760,076 KiB, above the 12 GiB requirement (12,582,912 KiB). After the retry and retained fixture writes, 15,958,480 KiB remained available.

Nine protected release executable/archive paths were rehashed after cleanup and matched their pre-cleanup values. This includes the shared CLI and test harness, oracle harness release binaries, and bundled DuckDB archive. The shared target/release, oracle target/release, and shared target/debug remained present. Full before/after df and hashes are in the external preservation record:

~~~text
/home/ubuntu/hld-current-test-artifacts/20261112T224747Z-native-m2-regression-retry/disk-cache-preservation-record.md
~~~

During post-cleanup verification, lsof showed rust-analyzer PID 1805010 with open handles to shared target/debug library paths flagged DEL, including libtypetag_impl-318c7ab49a189c87.so, libthiserror_impl-22d2ebaa93385b1c.so, and libturso_macros-d1d2eb01d8cdf145.so. The shared target/debug directory remained present; this cleanup command did not target it. The unlink cause is unknown, and no repair or cleanup was attempted.

## Run plan and execution

Source identity at launch and after the run:

~~~text
HEAD:       825d760e8685d8d4d95ab8acd2267edf3687847b
tree:       e3dc9a270177b5b47e0611ec89165c6d1780c282
production source diff under crates/: clean
~~~

Cargo 1.99.1 (5f94df478 2026-08-27), rustc 1.99.1 (58cf8f9c3 2026-10-13), active stable-x86_64-unknown-linux-gnu.

Exact command:

~~~text
cargo test -p parseltongue-cli-dispatcher-binary --test workspace-cli-contract-tests --all-features --release --locked --offline -- test_lang_items_in_sysroot --exact --nocapture
~~~

Expected duration was 1,200 seconds; maximum was 2,400 seconds; one attempt only. No external RAM cap was set. The whole Cargo/test/CLI process tree was sampled nominally every 0.2 seconds; the sampler itself was outside that tree and excluded. The recorded environment was:

~~~text
PATH=/opt/.devin/package/custom_binaries:/home/ubuntu/.cargo/bin:/home/ubuntu/.local/bin:/home/ubuntu/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/snap/bin
CARGO_TARGET_DIR=/home/ubuntu/repos/parseltongue-rust-LLM-companion/target
CARGO_BUILD_JOBS=2
CARGO_PROFILE_DEV_DEBUG=0
CXXFLAGS=-g0
RUST_LOG=info
PARSELTONGUE_NATIVE_FIXTURE_DIRECTORY=/home/ubuntu/hld-current-test-artifacts/20261112T224747Z-native-m2-regression-retry/fixtures
CARGO_GATE_ARTIFACTS=/home/ubuntu/hld-current-test-artifacts/20261112T224747Z-native-m2-regression-retry
CARGO_HOME=<unset>
RUSTUP_TOOLCHAIN=<unset>
RUSTUP_HOME=<unset>
RUSTFLAGS=<unset>
CARGO_ENCODED_RUSTFLAGS=<unset>
CARGO_INCREMENTAL=<unset>
CARGO_TERM_COLOR=<unset>
~~~

The finite supervisor enforced the configured 2,400-second deadline, then terminated and checked the process group. The test command's elapsed time including bounded cleanup was 2,410.252 seconds; the supervisor wrapper total was 2,410.271 seconds. Exit status was 124, timed_out=true, suite_results=[], and memory_cap=null. Peak process-tree RSS was 9,234,448 KiB across 4 processes. The final 10.252 seconds beyond the test deadline were bounded termination/cleanup, not extra test runtime.

~~~text
COMMAND_RESULT={"elapsed_seconds":2410.252,"exit_status":124,"timed_out":true,"cpp_rebuild_guard":false,"peak_process_tree_rss_kb":9234448,"peak_process_count":4,"process_group_cleanup":{"complete":true,"process_group":2123736,"remaining":[],"sigkill_sent":true,"sigterm_sent":true},"suite_results":[]}
~~~

The test harness source file SHA-256 is 2696ce5b42393f84e9baa8a39382469a6376e36909f2f0f3350e9db7ee8147e5. Executable identities after the run:

~~~text
CLI target/release/parseltongue
  b36a6d972d106ad42499fd9517adfd9bc7c186457752f34429f9a00954f5aa73
Test harness workspace_cli_contract_tests-f13681de9efffe60
  0a33b182d458340364c2a0a7374103eb09e5d18c9536dfe93ec46a4b49c973ca
Bundled DuckDB archive
  a753cf5c643d8bfe15e58fbf6f7e53c280ddf52b4bf811a791600ea7f71272ba
~~~

The C++ rebuild guard did not trip, Cargo reused the release test harness, and the DuckDB archive hash stayed unchanged. DuckDB phase diagnostics reported the engine as v2.0.0, threads=2, and its internal memory_limit=25.0 GiB. That application setting is distinct from the absent external/process memory cap.

## Live stderr observations

The parent Cargo log is retained at:

~~~text
/home/ubuntu/hld-current-test-artifacts/20261112T224747Z-native-m2-regression-retry/native-m2-sysroot-regression.log
~~~

Parent log SHA-256: 66cfe53890fd56aa818084ca68ef4ead1e22aced2462fb040204a0d5c5f914aa

It ends with the test running for over 60 seconds and has no assertion failure or test-result line. The retained child stderr is:

~~~text
/home/ubuntu/hld-current-test-artifacts/20261112T224747Z-native-m2-regression-retry/fixtures/.tmpPST9At/native-child-stderr-log-LxQdvS
~~~

Its final size is 118,777 bytes; SHA-256 is 8704f4920772fb891206f8c9a032bd37b0acd9581c25764b7d5e9f6adb2816f0. The sibling child stdout capture is retained at fixtures/.tmpPST9At/native-child-stdout-log-iOUAyR (0 bytes; SHA-256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855).

The live output recorded a namespace SQL fixed-point line for 221,714 source rows, 14 rounds, 458.399 seconds. It also recorded completed namespace-copy and namespace-normalization stages (for example, copy 29.769 seconds and normalization 41.272 seconds), then subsequent resolution steps. These are observations from this retry's live test logger, not queries against the earlier timed-out database.

At the 16:26 checkpoint, stderr showed namespace-resolution event=finished (390.992 seconds), the fixed-point line above, a frozen-input snapshot, copy and normalization complete, then a new namespace-resolution sequence. Later snapshots showed repeated completed copy/normalization events followed by path-candidate and import-binding rounds. The last retained event at timeout was:

~~~text
stage=namespace-resolution-step round=3 step=import-bindings event=started
~~~

No completion follows this marker. The current active internal operation and exact stalled SQL query are unknown; no root cause is inferred. Full checkpoint timing, process-tree RSS, limits, and observed lines are in:

~~~text
/home/ubuntu/hld-current-test-artifacts/20261112T224747Z-native-m2-regression-retry/native-m2-phase-observations.md
~~~

## Retained evidence and state

- Plan recorded before launch: /home/ubuntu/hld-current-test-artifacts/20261112T224747Z-native-m2-regression-retry/native-m2-run-plan.md
- Supervisor source and SHA-256: /home/ubuntu/hld-current-test-artifacts/20261112T224747Z-native-m2-regression-retry/native-m2-regression-supervisor.py; c8293dc900172702fefe52f0ed325dd2eb304b3d49c4b40ba92a50ad87870c2b
- Machine summary: /home/ubuntu/hld-current-test-artifacts/20261112T224747Z-native-m2-regression-retry/native-m2-summary.json
- Machine summary SHA-256: acbfa6b410f0a4b14e858fb6ac68ad9490a40eddd15f3f818ba144c3944ea976
- Cleanup and preservation record: /home/ubuntu/hld-current-test-artifacts/20261112T224747Z-native-m2-regression-retry/disk-cache-preservation-record.md
- Process-group cleanup summary reported complete, and a post-run scan found no process remaining in group 2123736.
- The older 2026-11-12 timeout/forensic report and the disk-blocked preflight report remain unchanged. Their evidence is not overwritten by this separate retry.

No source implementation, corpus, lock, pin, memory policy, or qualification state was changed. No further retry is authorized or started. No commit or push was made.
