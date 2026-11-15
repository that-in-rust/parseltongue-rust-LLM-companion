# Second opinion: part 4d `imports` design (GLM-5.3, pi, thinking max)

Run 2026-11-15, read-only (`~/bin/zpi`, tools read, grep, find, ls). About 5 minutes.
It changed one design choice: the import outcome is written by the analysis stage, not
re-derived in Rust.

## What I did with it

```text
claim                                                           status      effect
Q1 the SQL already computes absent-sysroot import names         confirmed   design changed: R1 of part 4d
   (namespace-final-path-products.sql:1-24)
Q1 imports renamed to _ create no binding                       confirmed   part 4d Q3
   (namespace-import-binding-rounds.sql:5-11)
Q2 one site can bind two targets; globs bind children           confirmed   R3, R9
   (namespace-import-binding-rounds.sql:5-30)                   (lines 5-14 read)
Q3 no stored link from an import request to its use item        not checked range containment kept (R2)
Q4 expected anchors mem::{self and MaybeUninit} are hand-cut    confirmed   part 4d Q1 (ranges read from the
                                                                            dataset and the exported store)
Q5 expected graph has 0 reexports and no glob rows              not checked R9, "not in this part"
Q6 validator accepts an unresolved imports row from a use node  not checked decides at step 4 when the
                                                                            facade test runs
Q7.1 195 edges is arithmetic                                    agreed      marked predicted
Q7.2 import_path_text is part of the contract                   agreed      R6
Q7.3 4d ends with stored rows, not a PASS                       agreed      part 4d Q4
Q7.4 resolve_use_import_edges is three words                    wrong       it is four; name kept
Q7.5 a builtin target for an import is dead code                agreed      dropped from R3
```

## Prompt (verbatim)

```text
You are a second reader with no stake in the plan. Read-only. Read the repository yourself and cite path:line for every claim. Be concrete and brief (under 900 words). Counts you give are arithmetic, not measurements; say so.

GOAL
Part 4d of docs/phase-20261113/SPEC-itoa-full-item-graph/: produce 'imports' edges for pinned itoa 1.0.18 (source at /home/ubuntu/repos/rust-tools/itoa, src/lib.rs lines 56-60). Scope is itoa only. Read AGENTS.md (Rules 1-5), part-4b-implements-from-impl-headers.md and part-4c-reads-of-consts-statics.md for the house style of a part, and crates/parseltongue-syntax-span-extractor/src/impl-header-implements-records.rs for the shape of an existing edge resolver.

MEASURED STATE (from an exported store of one run, sysroot source excluded)
- The 3 active 'use' items are graph nodes, kind UseStatement, byte ranges [1995,2010], [2011,2046], [2047,2061] in src/lib.rs. The two cfg-gated 'use no_panic::no_panic' items are inactive and are not nodes.
- Table syntax-namespace-input-imports has 8 rows. 4 have import_resolution_source_kind "Import": segments core::hint at [1999,2009]; core::mem::self at [2027,2031]; core::mem::MaybeUninit at [2033,2044]; core::str at [2051,2060]. The other 4 have kind "Signature" and glob flag true (trait bounds treated as glob imports by name resolution).
- No table stores the outcome of an import request. A successful import shows only as rows in module-scope-binding-table whose binding_import_evidence_keys contain the import site. For the 4 core imports there is no such binding and no stored reason.
- Path requests (a different table) get the reason external_source_missing by the SQL rule in crates/parseltongue-duckdb-analysis-adapter/src/namespace-final-path-products.sql lines 42-66: first name segment in (core, alloc, std, proc_macro, test) and the extern scope does not declare it.
- The draft expected graph (docs/phase-20261113/itoa-target-dataset/itoa-expected-graph-edges.json) has 4 imports rows, source the use node, targets external core::hint, core::mem, core::mem::MaybeUninit, core::str, with anchors 'core::hint' [1999,2009], 'mem::{self' [2021,2031], 'MaybeUninit}' [2033,2045], 'core::str' [2051,2060].
- User decision already made (docs/phase-20261113/BACKLOG-deferred-work-open-decisions.md): targets in core become named external targets in part 6; until then rows into core are stored unresolved with a reason, as parts 4a and 4b do.

MY DESIGN (lean), a new resolver resolve_use_import_edges in the syntax-span-extractor crate, same signature shape as resolve_impl_header_edges:
1. Iterate syntax-namespace-input-imports rows; keep kind "Import"; never "Signature".
2. Source = the one UseStatement node of the same unit and file whose byte range contains the import site and whose provenance chain is empty. No such node (use inside a function body, or macro-generated use): no row, counted (imports_without_source, imports_in_expanded_items).
3. Target = join module-scope-binding-table on binding_import_evidence_keys containing the site. One row per distinct target that is a graph node (or builtin): Extracted. Bindings exist but none is a node: Unresolved MissingTarget. No binding: ExternalSourceMissing when the first name segment is core/alloc/std/proc_macro/test and no unit of that name is in crate-dependency-graph-table (a Rust copy of the SQL rule); otherwise Unresolved MissingTarget.
4. Evidence site = the byte range the import request stores (whole path for a plain use, the leaf for a group member).
5. Metadata on every row: import_path_text (segments joined, trailing self dropped, so core::mem), alias, glob flag. This is the name part 6 needs to build the named external target.
6. Glob imports (use a::*): no row in 4d, counted as imports_glob_skipped, because the bindings name the children, not the module. itoa has none.
7. 'pub use' also gets an imports row; the reexports relation stays unproduced (itoa has none).
8. Deterministic order, cancellation per row, typed errors, scope check as in 4b.
Expected result on itoa: 4 imports rows, all Unresolved ExternalSourceMissing, 195 edges.

QUESTIONS
Q1. Is deriving ExternalSourceMissing in Rust (a second copy of the SQL rule) acceptable, or should the DuckDB name-resolution stage persist one outcome row per import request instead? Which is smaller and less likely to drift? Point at the code that would change.
Q2. Is the join in step 3 sound? Can one import site bind two different targets (type and value namespace), and can a binding carry import evidence without being that import's target (glob priority, re-export chains)? Cite the SQL that fills binding_import_evidence_keys.
Q3. Source lookup by byte-range containment: is there an existing stored key that links an import request to its use item (so no range search is needed)? Look in syntax-namespace-input-extraction.rs and the item rows.
Q4. Evidence range: production stores 'self' [2027,2031] and 'MaybeUninit' [2033,2044]; the expected graph has 'mem::{self' [2021,2031] and 'MaybeUninit}' [2033,2045]. Which side is right for a reader of the graph, and is the expected anchor 'MaybeUninit}' simply a defect?
Q5. Glob imports and pub use: is skipping globs and not producing reexports defensible for an itoa-only phase, or does it leave a silent hole? What is the smallest honest alternative?
Q6. Does the graph validator (crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs, validate_hld_materialization_input and the full-graph validation) accept an imports edge whose source is a UseStatement node and which is unresolved? Cite the checks.
Q7. Tell me what I did not ask and should have.
```

## Output (verbatim)

```text
I've read the repository (part files, the resolver, both SQL files, the validator, the dataset, the backlog decision, itoa source). Review below. All counts are arithmetic on numbers you gave or that I read from files, not measurements.

**Q1 — persist the outcome in DuckDB; do not copy the rule to Rust.** Precedent is against the Rust copy: parts 4a–4c read a reason the analysis stage already persisted (`read_stored_reason_kind`, `signature-type-edge-records.rs:371-378`; `original-call-resolution-records.rs:200-205`). The machinery for imports already exists: `absent-sysroot-import-names` (`namespace-final-path-products.sql:8-21`) computes exactly "non-glob import whose first name segment is core/alloc/std/proc_macro/test in an extern scope with no core/std unit". Worse, your Rust predicate ("no unit of that name in `crate-dependency-graph-table`") differs from the SQL predicate (no type-namespace declaration of `core`/`std` in the extern scope, `namespace-final-path-products.sql:3-7,42-44`) — a Rust copy is a third variant that can disagree with both. Smallest change: emit one outcome row per import request next to the binding emit in `namespace-final-output-records.rs:150-180`, then read it in the resolver like 4a/4c do. Also note `namespace-import-binding-rounds.sql:11` drops `_` aliases from bindings, so "no binding" cannot distinguish unresolved from resolved-but-suppressed; a persisted outcome row fixes that too.

**Q2 — the join is sound, with two caveats.** Direct imports seed one binding per resolved candidate *namespace* with evidence = the request key (`namespace-import-binding-rounds.sql:5-11`), so one site can legitimately bind two targets (type + macro). "One row per distinct target" is right; expect >1 row per site in general, and assert the binding's lexical scope equals the request's `import_lexical_scope_key` (the SQL's `importer_scope`) as a guard. Globs seed bindings for the module's *children* carrying the glob request as evidence (`namespace-import-binding-rounds.sql:13-30`), so an evidence-key join at a glob site returns children, never the module — your step 6 skip is consistent with the SQL. Re-export chains are fine: the binding target is the final resolved candidate, still this import's target.

**Q3 — no stored link exists.** `NameImportRequestRow` (`name-resolution-stage-records.rs:93-101`) holds only the UseTree occurrence, scope, segments, alias, glob, visibility, kind. Use nodes are built independently from `span-reference-text-table` (`hld-original-source-projection.rs:300-348`; category `LocalItem` at 348). Range containment on same unit + file + empty provenance chain is the only join available; it is deterministic, and both group-member sites sit inside the use node ([2027,2031] and [2033,2044] ⊂ [2011,2046]).

**Q4 — production is right; the dataset anchors are hand-cut and self-declared non-canical.** The dataset's own evidence keys are [1999,2009], [2021,2031], [2033,2045], [2051,2060] with `evidence_ref_kind: SYMBOLIC_SOURCE_SITE_REF_NOT_CANONICAL` (`itoa-blocked-rows-reasons.json:16286-16332`). [2021,2031] = `mem::{self` and [2033,2045] = `MaybeUninit}` span brace/prefix text — no syntax node has those ranges. Production's [2027,2031] `self` and [2033,2044] `MaybeUninit` are the UseTree nodes the extractor walks (`syntax-namespace-input-extraction.rs:1453-1475`), and the dataset's two plain-use keys already agree with production. `MaybeUninit}` is not a defect *inside* the dataset (anchor matches its range) but is not canonical: record it for part-7 reconciliation (as 4b Q2 did), don't bend production to it.

**Q5 — defensible for itoa only, and the numbers back it:** expected `reexports: 0` and no glob rows (`itoa-expected-graph-summary.json:41-48`); itoa's only extra use is cfg-inactive (`lib.rs:59-60`). The counter + part-file rule is the smallest honest form now; the alternative (one unresolved `Other` row per glob site) stores zero rows on itoa. Write the inactive-use and glob rules into the part file explicitly so part 7 can account for them.

**Q6 — yes, the validator accepts it.** `Imports` exists (`hld-materialization-core-records.rs:227`); `UseStatement` is a valid `LocalItem` kind (172; node checks at 743-780 don't restrict it under `FullSemantic`, and the resolver enforces that coverage kind — `impl-header-implements-records.rs:52-56`). Source category check (631-648) passes for `LocalItem`; unresolved rows skip the target endpoint check (651-658) and need only evidence identity (692-699: evidence source == edge source, relation match, both target refs None); site unit must equal source unit (712-719). No check requires the site range to lie inside the source node. One trap: `evidence_source_occurrence_key` must equal the *use node's* identity — evidence source is the node, site is the leaf.

**Q7 — what you didn't ask:**
1. "195 edges" is arithmetic (170 + 21 + 4); label it predicted in the part file (Rule 1).
2. `import_path_text` is load-bearing, not metadata: the expected rows are Extracted-to-external with names `core::hint`, `core::mem`, `core::mem::MaybeUninit`, `core::str` (blocked-rows 16286-16332) — part 6/7 can only match your ExternalSourceMissing rows to them via that text. Make it a requirement with a frozen rule (trailing `self` dropped gives `core::mem`, which matches).
3. Per AGENTS Rule 2, a sysroot-excluded run cannot PASS while the core decision is open — 4d's end state is stored rows, not SOP PASS; phrase it that way.
4. Naming: `resolve_use_import_edges` is three words; the house rule wants four (`resolve_use_statement_import_edges`).
5. Step 3's "(or builtin)" target is dead code for imports — builtins aren't importable; drop it or say why it's there.
```
