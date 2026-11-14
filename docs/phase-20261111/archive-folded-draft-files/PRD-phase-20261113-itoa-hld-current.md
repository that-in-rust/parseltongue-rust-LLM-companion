# PRD — phase-20261113: HLD-current for itoa

Status: draft; §6.1 decided 2026-11-13, §6.2 and §6.3 open. Owner: human-architect-intent-author.
Parent: `docs/phase-20261111/PRD-phase-20261111-compiling-hld.md`. That PRD is not replaced.
This document narrows its scope to one repository and lists the architecture decisions
that must be made before code. Where the two disagree, the parent wins until a decision
below is marked decided. The decisions in §6.1 are also written into the parent PRD (§17)
and SPEC (REQ-RUST-032.0, 019.2, 030.1).

Companions in this folder:

```text
 itoa-hld-preceding-tables.md      the tables, HLD back to spans, with what exists
 itoa-hld-build-architecture.md    the stages that build them, with code evidence
 Table-Specs-01.md                 the graph and level rules, with the itoa example
 itoa-target-dataset/              LLM-parsed draft of the full graph for itoa: nodes, edges,
                                   impl blocks, expected-unresolved sites, derived interface
                                   level, and the script that checks every cited line
```

AGENTS.md canonical paths still point at phase-20261111. They are not changed by this draft.

## 1. Objective

Produce the full-item-dependency-graph for one pinned repository, from its real source,
through the production pipeline; derive the five HLD-current tables from it; and compare
both row by row against an independently reviewed reference.

```text
 repository   dtolnay/itoa 1.0.18
 commit       af77385d0daf4d0e949e81f2588be2e44f69f086
 target       lib, edition 2021, x86_64-unknown-linux-gnu, no features, no build script
 size         src/lib.rs 466 lines, src/u128_ext.rs 22 lines
 full graph   item-effective-visibility-records (nodes), edge-evidence-record-table (edges)
 derived      hld-current-interface-nodes, hld-current-dependency-edges,
              hld-edge-evidence-paths, hld-file-level-rollup, hld-folder-level-rollup
```

Why itoa first: the whole library can be read and checked by a person, so every expected
row can be inspected. The five-repository qualification of the parent PRD comes after.

## 2. What stays exactly as in the parent PRD

```text
 storage       Turso for durable rows and publication; DuckDB for analysis; no third database
               (parent §16 governs; it supersedes the older "Turso is the only engine" line in §4)
 production    CPU tools only; no LLM at runtime; no ra_ap_hir*, ra_ap_ide*, ra_ap_base_db, salsa
 nodes         D6 as revised in parent §17: the full graph holds every item outside fn
               bodies, private included; the interface level keeps the non-private ones plus
               file-public-area-nodes and use nodes; effective visibility, not the `pub` token
 edges         extraction only; a generic or dynamic call targets the trait method,
               never a guessed implementation; unknowns are recorded with a reason;
               relation kinds as in the parent, with imports and reads added
 identity      full semantic occurrence keys; no display names, no invented sysroot spans
 publication   frozen generation in, atomic publish out, completion markers per stage
 method        TDD stub -> red -> green -> refactor with the journal; four-word names
 measurement   wall time and whole-process-tree RSS, measured and reported; no interim cap
```

## 3. What is narrower in this phase

```text
 one repository instead of five
 one compilation unit plus whatever `core` handling decision AD-1 selects
 not needed by itoa, so not built now: proc macros, build scripts, features,
 third-party dependencies, async, dyn dispatch, the optional trait solver (parent 7c)
```

## 4. Acceptance

The target dataset is two tables, full-graph nodes and full-graph edges, plus the list of
sites expected to stay unresolved. The five HLD tables are derived from those two by the
level rules and are compared as well. The preceding pipeline tables
(`itoa-hld-preceding-tables.md`) are internals, not targets.

```text
 A1  the production binary runs on the pinned source with no pre-resolved input
 A2  every expected full-graph node and edge is present with its full identity and
     evidence range, private items and private -> private edges included
 A3  no extra extracted edge is present
 A4  unresolved sites are listed with a reason; none is silently dropped
 A5  run time and peak process-tree RSS are recorded
 A6  every interface-level node, edge and evidence step exists in the full graph; the
     levels add nothing
```

Relationships the reference must contain (from source review):

```text
 <Buffer as Default>::default --calls--> Buffer::new
 <Buffer as Clone>::clone     --calls--> Buffer::new
 Buffer::format               --calls--> private::Sealed::write     one generic edge, not twelve
 Integer                      --uses_type--> private::Sealed
 <u128 as Sealed>::write --> <u128 as Unsigned>::fmt --> div_rem_1e16 --> u128_ext::mulhi
     three edges in the full graph; at the interface level collapsed to visible
     endpoints, private steps kept as proof
 Buffer::new                  --reads--> <i128 as Integer>::MAX_STR_LEN
 Buffer::format               --calls--> core::hint::unreachable_unchecked
     the `use core::hint` node is proof, not the target
 use nodes of lib.rs          --imports--> core::hint, core::mem, core::mem::MaybeUninit, core::str
 area node of lib.rs, area node of u128_ext.rs  --contains--> their top-level nodes
```

Targets inside `core` depend on AD-1; under its option B those edges are recorded unresolved.

Passing this phase is not a pass of the parent PRD's M8 or of the five-repository SOP.

## 5. Where the pipeline stands (read from the code, 2026-11-13)

```text
 parent stage             state                                   tables
 0-1  graph, ingest       built                                   crate graph, file source text
 2/3  syntax walker       built                                   spans, fields, body sites, namespace inputs
 4+5  names + macros      built, not qualified                    bindings, scopes, path sites, macro items
 6    signature lowerer   not built                               lowered signatures, impl headers
 7    body inferencer     not built                               edge evidence
 -    visibility          not built                               effective visibility; this table
                                                                  is also the full-graph node list
 9    HLD builder         built on fixtures only; nothing feeds   five HLD tables
                          it from tables; publish tests not green
```

No run on itoa has happened yet. Not in the code either: the `file_public_area` item kind,
the `imports` and `reads` relation kinds, and any writer of the two full-graph tables.

## 6. Architecture decisions

### 6.1 Decided on 2026-11-13

Details and the itoa example are in `Table-Specs-01.md`.

```text
 DD-1  Build the full dependency graph first; every HLD level is derived from it in DuckDB
       and adds no fact.
 DD-2  Private items outside fn bodies are full-graph nodes. The interface level drops them
       and keeps them as proof steps on collapsed edges.
 DD-3  One file-public-area-node per source file, separate from the module node, always
       kept. `src/u128_ext.rs` gets one although `mod u128_ext` is private.
 DD-4  Every `use` is a node, with an `imports` edge per resolved binding.
 DD-5  `mod private { }` is an ordinary module node with its own scope.
 DD-6  New relation kind `reads` for const, static and associated-const reads.
 DD-7  Calls and type uses target the real item; the `use` line is proof. No edge from a
       function to an import it does not use.
 DD-8  The full graph is the two existing tables, item-effective-visibility-records and
       edge-evidence-record-table. No new table.
 DD-9  Collapse never walks a `contains` or `imports` edge. So the lib.rs area node gets no
       collapsed calls edges from the private functions it contains. The builder's SQL
       walks every relation kind today and must change.
```

DD-6 settles the associated-const question raised in `itoa-architecture-haze-ratings.md`.

### 6.2 Open, raised by the decisions above

None blocks slices 0 to 3. OD-1 was decided and is now DD-9. OD-7 and OD-8 were found in the
line-by-line review of 2026-11-13.

```text
 OD-2  Which file owns an item generated by `impl_Integer!`, for `contains` and roll-ups.
 OD-3  Does the agent query the two full-graph tables, or a published copy with path and
       signature text.
 OD-4  Enum variants and struct fields as nodes. itoa's public surface has neither.
 OD-5  `pub use`: an imports edge, a reexports edge, or both. itoa has no `pub use`.
 OD-6  Private items visible to several files, shown at the interface level or not.
 OD-7  Containment detail. Impl blocks are not nodes, so which node contains
       `<Buffer as Default>::default`? And `private::Sealed` sits in a private module that the
       interface level drops, so under DD-9 the lib.rs area node no longer contains it there.
       Lean: one container per node (the area node for top-level items and impl members), and
       a contains-only chain may collapse through a dropped module.
 OD-8  May `reads` and `uses_type` start at a non-function item? itoa needs it for
       `const MAX_STR_LEN: usize = <u64 as Integer>::MAX_STR_LEN` (isize, usize) and for the
       array length in `Buffer`'s field. Lean: yes, the source is the item that holds the
       expression.
```

OD-7 and OD-8 change rows of the itoa target dataset, so they block freezing it.

### 6.3 Open, pipeline choices

Each one states what the code does today, the options, and what each option costs.
A recommendation is given where the evidence supports one. None of AD-1 to AD-5 is decided.

### AD-1  How itoa's references into `core` are handled

```text
 today      a file that mentions `core`, `Clone` and similar makes the pipeline ingest and
            parse the sysroot crates as extra units. itoa's lib.rs does. The latest native run
            through that path (not itoa) stopped at 2,400 s with 8.81 GiB peak RSS.
            Whether itoa plus sysroot finishes is not known.

 option A   keep sysroot ingestion (parent PRD as written)
            + targets into core get real identities
            - first itoa result waits on the unqualified name-resolution run

 option B   first itoa pass without sysroot units
            + most itoa-to-itoa edges are buildable now
            - not all: the signed `write` calls `Unsigned::fmt(self.unsigned_abs(), ..)`; which
              impl that reaches depends on the return type of a core method
            - edges into core (trait impls of Default / Clone / Copy, calls on MaybeUninit,
              pointers, slices) are recorded unresolved, reason: external source missing
            - the comparison must be labelled local-only until A is done

 option C   name core targets by canonical path without parsing the sysroot
            - conflicts with "no invented sysroot identities"; needs a parent PRD revision

 recommend  measure first: run today's pipeline on itoa once (slice 0). If it finishes
            within a stated time, take A. If it does not, take B as an explicit interim.
```

### AD-2  Who resolves paths inside function bodies

```text
 today      the syntax walker stores body paths as text; name resolution takes only
            import paths and signature paths, and skips anything inside a fn body.

 option A   send body paths to DuckDB as a third kind of path site, resolved in one SQL
            pass after the name fixed point has finished
            + reuses the existing path-candidate SQL; joins stay in DuckDB (parent §16)
            - more rows in the analysis copy; needs a SPEC requirement revision

 option B   resolve them in Rust inside the body inferencer, reading binding rows
            + matches the parent PRD's stage 7 wording
            - a second path resolver to keep consistent with the SQL one

 recommend  A. Looking up an item path is a join the SQL already does; the Rust
            inferencer then only drops names that a local variable shadows.
```

### AD-3  Where effective visibility is computed

```text
 today      the Turso registry and the HLD publisher already name a separate stage,
            item-effective-visibility-stage, with its own table and required marker.
            The parent PRD §5 lists no separate visibility stage; its §17 now shows the
            registry's stage names without deciding where the work is computed.

 option A   separate stage, DuckDB SQL only (what the code already expects)
 option B   inside the builder; the registry and the publisher then have to change

 note       under DD-8 this table holds every full-graph node, not only selected ones, so
            the stage must also write private items, use nodes and file-public-area-nodes.

 recommend  A, and add one line to the parent PRD. It is the smaller change.
```

### AD-4  How much body inference the first version has

```text
 itoa needs  receiver types from parameters, `self`, and simple `let` bindings;
             inherent methods (parent 7a); trait methods through a direct bound (parent 7b)

 option A    exactly that; anything else is recorded unresolved with a reason
 option B    port the wider inference first (closures, match ergonomics, autoderef chains)

 recommend   A. B is needed for the larger repositories, not for itoa.
```

### AD-5  What the pipeline's output is compared against

```text
 today      the reference is a source-review draft: 48 selected occurrences, 197 call-site
            candidates, status DRAFT_NOT_FROZEN. The rust-analyzer oracle export is incomplete.

 option A   reference tables written by LLM parsing of the pinned source, then reviewed
            by a person, kept outside production, each row marked observed / inferred / unresolved
 option B   reference exported by the rust-analyzer oracle harness run on itoa
 option C   A as the reference, B as a cross-check where the harness can export

 note       A is what was asked for on 2026-11-13. In every option the reference is
            comparison data only and is never read by the production pipeline.
```

## 7. Build order

One real-source test per slice. Expected rows live only in the test.

```text
 slice 0   run today's pipeline on itoa unchanged; record time, peak RSS, last stage reached
           -> settles AD-1
 slice 1   <Buffer as Default>::default --calls--> Buffer::new
 slice 2   <Buffer as Clone>::clone --calls--> Buffer::new; implements and uses_type edges
 slice 3   Buffer::format --calls--> private::Sealed::write (one generic edge)
 slice 4   the u128 chain through macro-generated impls: three full-graph edges, then collapsed
 slice 5   use nodes, imports edges, file-public-area-nodes, reads edges
 slice 6   whole library compared against the frozen reference (A1 to A6)
```

A slice passes on the full-graph row first. The interface-level row is checked once the
visibility stage exists.

Each slice adds only the general behaviour that relationship needs. Nothing is keyed on
itoa names.

## 8. Out of scope

```text
 serde, rust-analyzer, ripgrep, tokio, cargo-modules
 the native sysroot timeout, except as far as AD-1 option A requires it
 CLI query verbs, coverage banner, intent tables
 memory or speed optimisation
```

## 9. What this document does not claim

```text
 no stage after name resolution exists for real source
 no itoa run has been made
 the reference is a draft
 the recommendations above are design judgement, not measurements
 the decisions in §6.1 are written down, not implemented
```
