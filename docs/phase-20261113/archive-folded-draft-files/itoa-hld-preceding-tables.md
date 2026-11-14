# itoa HLD-current — preceding tables, worked backwards to the spans table

Date: 2026-11-13. Phase folder: `docs/phase-20261113/`.
Benchmark: `dtolnay/itoa` 1.0.18, commit `af77385d0daf4d0e949e81f2588be2e44f69f086`
(pin: `docs/phase-20261111/phase-one-benchmark-pin.json`).

Revised 2026-11-13: levels 1 and 2 together are now the full dependency graph, a product in
its own right, and level 0 is derived from it (`Table-Specs-01.md`). The target dataset for
itoa is therefore levels 1 and 2 only; levels 3 to 7 are pipeline internals.

This file is a thinking document. It lists which tables must exist before the
itoa HLD tables can be produced, starting at the HLD outputs and walking back
to `span-reference-text-table`. Nothing here has been built or run on itoa.

## How to read the status words

```text
 REGISTERED+PRODUCED   table is in the Turso stage registry and a pipeline stage writes it
 PARTIAL               a stage writes it, but the stage is not qualified (M2 native run timed out on sysroot)
 REGISTERED ONLY       table name is in the registry; no source producer exists in crates/
 NOT RUN ON ITOA       applies to every row below: the pipeline has never been run on pinned itoa
```

Sources for the status words (checked 2026-11-13):

```text
 registry   crates/parseltongue-turso-store-adapter/src/stage-table-store-backend.rs  STAGE_OWNED_TABLE_NAMES
 CLI        crates/parseltongue-cli-dispatcher-binary/src/main.rs  stage_implementation_status_values
            -> signature-lowering, call-resolution, visibility-hld: "not yet implemented"
 grep       the four semantic table names occur only in the registry and one test file
```

## The chain, top (HLD) to bottom (spans)

```text
 LEVEL 0 — HLD OUTPUTS                                   owner: hld-current-materialization-stage
   hld-current-interface-nodes
   hld-current-dependency-edges
   hld-edge-evidence-paths
   hld-file-level-rollup
   hld-folder-level-rollup
   status: DuckDB builder passes on typed fixtures; Turso publication not GREEN (9 passed, 2 failed)
        ^
        |  DuckDB: filter nodes, collapse private paths (never through contains or
        |  imports edges), keep proof paths, roll up. Adds no fact.
        |  The builder does NOT read tables. It takes HldMaterializationInputRecord:
        |  provenance records + node records + edge records. Something must assemble
        |  those records from levels 1 and 2. That assembler does not exist.
        |
 LEVEL 1 — FULL GRAPH EDGES                              owner: semantic-edge-evidence-stage
   edge-evidence-record-table                            REGISTERED ONLY
   one row per relation, private -> private included, never collapsed: source
   occurrence, resolved target, relation kind (calls / uses_type / implements /
   reexports / contains / derives / imports / reads), status, dispatch +
   substitution metadata, site occurrence, evidence range.
        ^
        |
 LEVEL 2 — FULL GRAPH NODES                              owner: item-effective-visibility-stage
   item-effective-visibility-records                     REGISTERED ONLY
   one row per item outside fn bodies (private included), per `use`, and one
   file-public-area-node per source file: semantic owner, effective visibility,
   body ancestry, external reachability. Every row is a full-graph node; the
   interface level filters on visibility.
        ^
        |
 LEVEL 3 — IMPLS AND TYPES                               owner: signature-type-lowering-stage
   impl-header-index-table                               REGISTERED ONLY
   impl occurrence, self type, trait target, substitutions, associated members.
   lowered-signature-type-table                          REGISTERED ONLY
   resolved parameter / return / self types, generic bounds, projections.
        ^
        |
 LEVEL 4 — NAME AND PATH RESOLUTION                      owner: name-resolution
   path-resolution-site-records                          PARTIAL
   name-resolution-scope-records                         PARTIAL
   module-scope-binding-table                            PARTIAL
   macro-expanded-item-table                             PARTIAL
        ^
        |
 LEVEL 5 — NAMESPACE INPUTS                              owner: syntax-namespace-input-records
   syntax-namespace-input-scopes
   syntax-namespace-input-items
   syntax-namespace-input-imports
   syntax-namespace-input-paths
   syntax-namespace-input-macros
   syntax-namespace-input-externs
   lang-item-definition-table
   legacy-macro-scope-records
   legacy-macro-source-events
   status: REGISTERED+PRODUCED
        ^
        |
 LEVEL 6 — SYNTAX FACTS CUT FROM SPANS                   owner: syntax-spans
   field-signature-detail-table                          REGISTERED+PRODUCED
   raw params, return, generics, bounds, impl trait/self text. Text, not resolved types.
   body-site-reference-table                             REGISTERED+PRODUCED
   raw call / path / method sites inside bodies. Stored pending, not resolved.
        ^
        |
 LEVEL 7 — SPANS  (stop here)                            owner: syntax-spans
   span-reference-text-table                             REGISTERED+PRODUCED
   one row per item span: digest id, kind, name, path, visibility, range, raw code, doc, cfg.
```

## Not on the semantic chain

These exist and are needed, but they carry context or control, not HLD meaning.

```text
 below the spans table
   file-source-text-table            source-ingest   raw file bytes the spans are cut from
   span-byte-ownership-ranges        syntax-spans    which span owns which bytes
   crate-dependency-graph-table      crate-graph     packages, targets, features, cfg
   workspace-config-state-records    crate-graph     workspace configuration, required stages

 work queues
   module-source-closure-requests    syntax-spans
   syntax-file-work-requests         syntax-spans

 control and publication
   stage-run-generation-records              completion markers per stage
   frozen-stage-publication-records          frozen snapshot revisions
   frozen-stage-unit-memberships             which units belong to a frozen revision
   workspace-generation-publication-manifest which generation is currently published

 not needed for HLD
   oracle-edge-agreement-table               offline comparison only
   analysis-resolution-receipt-records       name-resolution run receipts
```

## What each itoa relationship needs

```text
 <Buffer as Default>::default --calls--> Buffer::new
   span-reference-text-table          the two fn spans and the impl span
   body-site-reference-table          the `Buffer::new()` site inside default
   level 5 + level 4                  `Buffer` resolves to the struct occurrence
   impl-header-index-table            inherent impl of Buffer has member `new`;
                                      the caller's impl is Default for Buffer
   edge-evidence-record-table         the calls row with its evidence range
   item-effective-visibility-records  both ends are nodes, at both layers
   lowered-signature-type-table       NOT needed for this one

 Buffer::format<I: Integer> --calls--> private::Sealed::write
   everything above, plus
   lowered-signature-type-table       I: Integer, Integer: Sealed, so i.write() is Sealed::write
   must stay ONE generic edge; the twelve integer impls go in dispatch metadata

 <u128 as Sealed>::write -> <u128 as Unsigned>::fmt -> div_rem_1e16 -> u128_ext::mulhi
   everything above, plus
   macro-expanded-item-table          the impls come from itoa's declarative macros
   item-effective-visibility-records  Unsigned::fmt and div_rem_1e16 are private:
                                      full-graph nodes, dropped and collapsed at the
                                      interface level by the HLD builder
```

## The gap in one line

```text
 levels 7, 6, 5     produced today
 level 4            produced, not qualified
 levels 3, 2, 1     four tables, zero producers   <-- the missing work
 assembler          tables -> HldMaterializationInputRecord, missing
 level 0            builder works on fixtures only
```

`body-site-reference-table` holds the text `Buffer::new()`. Turning that text into
the occurrence `Buffer::new` is the job of levels 4 and 3. No amount of work at
level 0 produces it.

## About "LLM parsing"

Production rows at levels 1 to 3 must come from deterministic Rust and SQL.
An LLM-parsed copy of these tables for itoa is allowed only as separate
benchmark data to compare against. It must never be fed to the production
builder, and it has not been created.
