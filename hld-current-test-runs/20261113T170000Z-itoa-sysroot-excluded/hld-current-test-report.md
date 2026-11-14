# itoa with sysroot scope excluded — diagnostic run, 2026-11-13

Not an SOP PASS and not a milestone. A local-only run under PRD AD-1 interim option B
(SPEC REQ-RUST-038.0). It shows what the pipeline publishes for pinned itoa today.

```text
 source          dtolnay/itoa 1.0.18, commit af77385d0daf4d0e949e81f2588be2e44f69f086
                 (qualification copy with a generated 53-package Cargo.lock)
 binary          target/release/parseltongue, sha256
                 c8679fff0b3860973c8d27ba73a0a35ede1eda8f7e1295ea33fec97caf44c9a6
                 built from an uncommitted working tree
 command         parseltongue map-workspace-full-run --workspace-root-path-value <copy>
                 --workspace-store-file-path <db> --worker-thread-count-value 4
                 --sysroot-source-scope-kind excluded
 declared        expected 60 s, kill at 240 s; no memory cap
 exit            0
 wall            11.55 s
 peak RSS        362,708 kB (354 MiB), parent process only (time -v); not a tree sample
 generation      200527fb15e4e88d064c12943ffaaaa0accaa121bc08fffe76a89fab5cf6339a
 artifacts       ~/hld-current-test-artifacts/20261113T170000Z-itoa-sysroot-excluded/
```

## Stages

```text
 crate-graph, source-ingest, syntax-spans, macro-expansion, name-resolution   complete, published
 signature-lowering, call-resolution, proc-macro-shapes, visibility-hld      not yet implemented
```

## Published rows (read with export-published-stage-rows)

```text
 crate-dependency-graph-table    1 unit (itoa lib, edition 2021); 0 sysroot units
 span-reference-text-table       148 spans: 41 impl, 32 fn, 27 const, 17 type alias,
                                 15 macro call, 5 use, 3 macro_rules, 3 trait, 2 module,
                                 2 struct, 1 static
 macro-expanded-item-table       15
 module-scope-binding-table      140
 path-resolution-site-records    202
 inactive (cfg) item records     22
```

## Path sites: 202

```text
 resolved                              164   all in signatures (usize, u8, Integer, str,
                                             private::Sealed, Buffer, Unsigned, ...)
 unresolved, external_source_missing     5   the `use core::...` imports
 unresolved, no reason                  33
    type-relative, not built            16   Self::MAX_STR_LEN x10, Self::Buffer x5,
                                             i128::MAX_STR_LEN x1          (REQ-RUST-036.0)
    sysroot through an import           13   MaybeUninit<u8>               (REQ-RUST-038.0, owed)
    prelude names                        4   Copy x2, Default, Clone       (REQ-RUST-038.0, owed)
 path sites inside function bodies       0   none are produced            (REQ-RUST-034.0)
```

## Reading

- The first checkpoint, `<Buffer as Default>::default -> Buffer::new`: expected in the draft
  target; actual: absent. No path site exists for `Buffer::new()` because body paths are not
  resolved. First blocking stage: body path resolution, then impl headers and type-relative lookup.
- No full-graph table (`item-effective-visibility-records`, `edge-evidence-record-table`) is
  written yet.
- Not compared against the target dataset: nothing here is scored.
