# M1 preflight: corpus qualification and process-policy blocker

Status: BLOCKED before STUB. Date: 2026-11-11. No M1 implementation or S1–S3 success is claimed.
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

## Blocking process trace

REQ 001 requires cargo metadata --offline and limits the complete production process tree to that
command plus rustc --print cfg and rustc --print sysroot. The mandated metadata command on the
rust-analyzer checkout itself executed:

```text
rustc -vV
rustc - --crate-name ___ --print=file-names --crate-type bin --crate-type rlib
  --crate-type dylib --crate-type cdylib --crate-type staticlib --crate-type proc-macro
  --print=sysroot --print=split-debuginfo --print=crate-name --print=cfg -Wwarnings
```

This was observed with strace -f -e trace=execve, not inferred from Cargo source.
Full trace: rust-analyzer/metadata-process-preflight-trace.log.
The same command on the already warm Parseltongue workspace did not execute rustc, so relying on
warm caches can conceal the incompatibility. No production M1 path exists yet; this demonstrates
the required subprocess contract needs a decision, not that a completed mapper has failed.

Recommendation: allowlist Cargo's narrowly defined compiler-discovery descendants without allowing
compilation or build-script execution; keep the mapper's direct command list unchanged.
No such exception has been applied.

## Unexecuted gates

S1 index, S2 coverage, S3 mapper: NOT RUN; blocked before implementation.
File/span/field/body-site counts, tiling identity, mapper time and peak RSS: NOT MEASURED.
No TDD phase commits, stage publication, concurrency or cancellation results for M1.
