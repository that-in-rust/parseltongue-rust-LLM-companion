# hld-current-test-report — 20261113T151000Z-itoa-unchanged-pipeline

Phase 1, slice 0: today's pipeline on pinned itoa, unchanged. Diagnostic measurement, not an SOP
PASS and not a milestone run.

```text
 source        dtolnay/itoa af77385d0daf4d0e949e81f2588be2e44f69f086, lib, no features
 input copy    git archive of the pin + a Cargo.lock generated once outside production
               (53 packages, sha256 965fa758128c03489e02550fbf8f7fb0e2b36e6e82163c3cf482900cfa8d019d)
 executable    target/release/parseltongue
               sha256 9971d1498800126b8f932bad1474b2c1ac203dc6b26aa94a1ab58d4d96ea0a01
               built from 4f499f708 plus uncommitted HLD publication changes; build 19m21s, exit 0
 command       map-workspace-full-run, 4 workers, final stage name-resolution
 declared      expected 1200 s, kill at 2400 s, no memory cap
 artifacts     ~/hld-current-test-artifacts/20261113T151000Z-itoa-unchanged-pipeline/
```

## Result

```text
 exit            124: stopped by the declared 2400 s limit; pipeline reported
                 NameResolutionCancelledError
 wall            40m21.67s
 peak RSS        8,725,256 kB (8.32 GiB), parent process only (time -v); not a tree sample
 syntax stage    619 units, 359,053 records, 44.3 s  -> sysroot crates were ingested
 name resolution NOT complete. Last marker: round 5 of frozen snapshot 7; highest round in any
                 snapshot: 11. No error event before the kill.
 published       nothing. map-stdout.json is empty; no itoa rows can be read.
```

## Per SOP step

```text
 S1-S2   not started
 S3      BLOCKED at name resolution (time limit)
 S4-S10  not started
```

## Phase 1 checkpoint

```text
 expected  <Buffer as Default>::default --calls--> Buffer::new   (target dataset, lib.rs:79)
 actual    none: no generation published
 first blocking stage  name resolution over itoa plus sysroot units
```

## What it decides

PRD decision AD-1 said: measure first; if the run does not finish, take option B as an explicit
interim. Option B: map itoa without sysroot units; edges into `core` are recorded unresolved
(external_source_missing) and comparisons are labelled local-only until option A works.
