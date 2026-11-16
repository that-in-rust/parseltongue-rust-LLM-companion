# Second opinion — GLM-5.3 (thinking max) — part 6, named external targets

- Asked: 2026-11-16, epoch 1794796183, at commit 8afa22fe8. Read-only run (`~/bin/zpi`), 369 s, exit 0.
- Why: the design for REQ-RUST-053.0 was a draft; the review was taken before the work packet was written.
- The reviewer could not compile or run code. Its claims about code were checked against the files before use.

## Prompt (verbatim)

```text
You are a read-only design reviewer for this repository (branch research/tiny-rust-code-mapper, HEAD 8afa22fe8). Do not edit files. You cannot compile or run code; say so and answer from reading.

Context. The product is a deterministic, source-backed dependency graph of a Rust crate for an external coding agent. Rule: maximum information for the agent, never a guessed fact. The current loop is pinned itoa 1.0.18, run with the sysroot source excluded (core is not read). The stored graph has 103 node rows (88 local items, 2 file areas, 13 builtin primitive types) and 585 edge rows, of which 205 are stored without a target.

Read:
- docs/phase-20261113/SPEC-itoa-full-item-graph/part-6-targets-in-core.md (the user's decision: a named external target, as far as itoa's source proves the name, plus the crate and a flag that the target's source was not read)
- docs/phase-20261113/ARCH-itoa-full-item-graph-pipeline.md rules F2, F3, F4, F11
- crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs : HldTargetIdentityValue, HldNodeInputCategory (has ExternalDefinition and Builtin), HldEdgeResolutionValue, HldUnresolvedReasonKind, and the validation of node and edge records (how a Builtin node and an Extracted target are checked)
- crates/parseltongue-core-row-types/src/name-resolution-stage-records.rs : NameSymbolTargetKey
- crates/parseltongue-syntax-span-extractor/src/use-import-edge-records.rs, impl-header-implements-records.rs, signature-type-edge-records.rs, body-value-read-records.rs, original-call-resolution-records.rs : where each relation gives ExternalSourceMissing / TraitSolverNeeded / MissingTarget today

The 205 rows without a target, measured from the stored edge table:
  imports 4      use core::hint; use core::mem::{self, MaybeUninit}; use core::str            ExternalSourceMissing
  uses_type 13   MaybeUninit<u8> (the name came through the use item)                         ExternalSourceMissing
  uses_type 1    Copy (a bound; no item or import of that name in the crate)                  ExternalSourceMissing
  implements 3   impl Clone / Copy / Default for Buffer                                       ExternalSourceMissing
  calls 4        mem::size_of::<Self>()                                                       ExternalSourceMissing
  calls 1 each   hint::unreachable_unchecked(), str::from_utf8_unchecked(..), MaybeUninit::<u8>::uninit()
  calls 9        u128::from(x)                                                                MissingTarget (no stored impl of u128 declares from)
  reads 30       u8::MAX, i8::MIN, u8::BITS ... (10 integer types x MAX, MIN, BITS)           TraitSolverNeeded
  calls 138      method calls whose receiver type comes from core: .write 49, .get_unchecked 39, .try_into 13, .ilog10 10, .expect 8, .len 6, .unsigned_abs 5, .unwrap 5, .as_mut_ptr, .cast, .rev

My draft design (not decided):
 D1. Representation: a new target identity variant for a named external item (crate name + item path as proven + how it was proven), stored as Extracted, with one node row of category ExternalDefinition per distinct named item (the same pattern as the 13 builtin node rows). Alternative: a new resolution variant that carries the name and is not a node.
 D2. What each shape proves:
   - use core::mem::MaybeUninit            -> core::mem::MaybeUninit, "path as written"; the definition path inside core (it may be a re-export) is not read
   - MaybeUninit<u8> through that use      -> the same named item
   - mem::size_of::<Self>() through use core::mem -> core::mem::size_of
   - MaybeUninit::<u8>::uninit()           -> core::mem::MaybeUninit::uninit, "member through an external type"; inherent or trait member is not proven
   - u128::from(x), u8::MAX                -> "u128::from", "u8::MAX": a member of a builtin type that no stored impl in this crate declares; which impl (inherent, or which trait) is not proven
   - Clone, Copy, Default                  -> the bare name, "not declared or imported in read source, so it is a prelude name"; the path inside core is not read
   - the 138 method calls                  -> stay without a target, with the method name and the reason (the user's decision)
 D3. Expected effect: 67 rows gain a named external target, 138 stay unresolved, about 42 new node rows.

Questions. Answer each with a verdict and the reasoning; cite file and line where you rely on code.
 Q1. D1: which representation is better for an agent that reads the node and edge tables, and which has the smaller risk given the validation code as written? Name every place in the code that would have to change for your choice.
 Q2. For each shape in D2: is the claimed name really proven by itoa's source alone? Give a concrete Rust program where the claim would be WRONG (a wrong target, not a missing one). In particular: (a) a use of a path that is a re-export; (b) "u8::MAX" when a trait in scope with an impl in another crate declares MAX; (c) "u128::from" when the crate itself declares a trait with a method from and a blanket impl; (d) a bare name Default when a glob import or a macro-generated item defines Default; (e) #![no_std] versus std prelude, and a crate with #![no_implicit_prelude].
 Q3. What should the crate field be for a builtin member (u8::MAX) and for a prelude name (Default) when core was not read? Is "core" a proven fact or a guess? Propose the wording that is true.
 Q4. Should a row with a named external target be Extracted, or a distinct status? What would an agent wrongly conclude from Extracted?
 Q5. Should two rows that name u8::MAX share one node, and should u8::MAX and i8::MAX be separate nodes? What identity key makes rows comparable without claiming more than is proven?
 Q6. Anything in the draft that guesses, or anything the source proves that the draft leaves out (for example the trait of try_into through the prelude, or the callee of a method call on a literal-typed receiver).
 Q7. The smallest safe build order: which shape first, and what each step's real-source test on itoa should assert.

Keep the answer under 1500 words. Plain sentences.
```

## Answer (verbatim)

I have read the SPEC part file, ARCH rules F2/F3/F4/F11, the row types and validation, the five producer files, and the pinned itoa source. I cannot compile or run anything; everything below is from reading. Review of HEAD 8afa22fe8 as asked.

## Q1 — Representation

**Verdict: a node, but only through a new name-keyed identity variant. Do not reuse `ExternalDefinition` as it is validated today.**

The draft says "the same pattern as the 13 builtin node rows", but the code has two different patterns. A `Builtin` node is name-keyed (`NameSymbolTargetKey::Builtin(name)`, name-resolution-stage-records.rs:60; validation at hld-materialization-core-records.rs:959-975; `read_hld_target_unit` returns `None` for it at :1071, which also skips the unit-provenance check at :755-762). An `ExternalDefinition` node is span-keyed: it must be `Definition(occurrence_key)` and passes `validate_hld_occurrence_identity` (:938-951), which rebuilds a real file, byte range and digest in a unit that must be in the provenance records (:634-641). Core was never ingested, so no such span exists; inventing one violates ARCH I1 ("no invented sysroot spans"). So option A needs a new variant, e.g. `NameSymbolTargetKey::ExternalNamed { crate_name, member_path, proof_kind }`, plus a new arm in `validate_hld_node_input` mirroring the `Builtin` arm (no item kind, owner, visibility; qualified path non-empty, :593).

The alternative (a resolution variant carrying the name, no node) touches less code but hides the name inside each edge row: two rows cannot share an identity, and an agent cannot ask "what in itoa uses `MaybeUninit`?". Rule 6 (maximum information) picks the node. The not-read flag and proof kind should live inside the variant, because `HldNodeInputRecord` has no metadata field and adding one touches every codec.

Places that change for the node choice: `NameSymbolTargetKey` (name-resolution-stage-records.rs:56-61); `validate_hld_target_key`, `read_hld_target_unit`, `read_hld_target_path` (hld-materialization-core-records.rs:1057-1097); the `ExternalDefinition` arm of `validate_hld_node_input` (:937-951); the `From`/`PartialEq` impls (:96-127, generic, likely no change); the producers that return `ExternalSourceMissing` today (use-import-edge-records.rs:250-288, impl-header-implements-records.rs:114-125, signature-type-edge-records.rs:440-485, body-value-read-records.rs:299-311, original-call-resolution-records.rs:246-300 and :1027-1077); and the Turso/DuckDB codecs plus the export verb for both enums.

## Q2 — Is each claimed name proven?

**(a) `use core::mem::MaybeUninit` → "core::mem::MaybeUninit".** Proven as *the written path*: the import machinery already stores the segments per binding (`NameImportRequestRow.import_path_segment_rows`, name-resolution-stage-records.rs:107-116; the fixture at use-import-edge-tests.rs:281-291 shows "core::mem" and "core::mem::MaybeUninit" per binding, two rows for `{self, MaybeUninit}`). Wrong-target counterexample: `use std::fmt::Display;` — written "std::fmt::Display", definition `core::fmt::Display`. So the node must claim "the name this use binds", never "the definition location". Same for `mem::size_of`, `str::from_utf8_unchecked`, `hint::unreachable_unchecked` (itoa lib.rs:38-41, :212, :252, :110): written paths, proven, unification with the import nodes is legitimate.

**(b) `u8::MAX`.** The written member path on a builtin is proven (lib.rs:144, :160, :138-140). Which impl declares it is not: inherent precedence only saves you because core happens to define an inherent `MAX`, a fact not in read source. Counterexample: `trait Wide { const MAX: Self; } impl Wide for u8 { const MAX: u8 = 200; }` with the trait in scope — for a primitive *without* an inherent member of that name, `u8::MAX` would be the trait's. Store (builtin `u8`, member `MAX`); claim nothing about impl or crate.

**(c) `u128::from` (u128_ext.rs:14-21, nine sites).** The draft stores "member of a builtin that no stored impl in this crate declares" — that gate is proven by read source (u128_ext.rs declares no trait at all). Counterexample where a crate claim would be a *wrong target*: `trait Widen { fn from(v: u64) -> Self; } impl Widen for u128 { ... }` in scope — then `u128::from(x_lo)` resolves to the local trait. So keep the gate dynamic: a fixture where a local trait declares `from` must resolve locally, never to the external node. Do not store `core::convert::From`.

**(d) bare `Default`/`Clone`/`Copy`.** Proven: no item, import or macro-generated binding of these names exists in read source (lib.rs has only the four `use` lines, one cfg-inactive; item macros are expanded to a fixed point by ARCH step 5). Counterexample where "prelude" is wrong: `use helper::*;` where the glob binds `Default`, or a macro that expands to `pub struct Default;`. The gate must name all three sources (item, import, macro expansion) before the row may say prelude.

**(e) prelude kind.** itoa is `#![no_std]` (lib.rs:36); the implicit prelude of a no_std crate is `core::prelude::v1` — a language rule, not read source. Counterexample: `#![no_implicit_prelude]` plus `use core::default::Default;` — then the name is an explicit import, not a prelude name. Helpfully, the name-resolution stage already models this: `scope_implicit_prelude_key` (name-resolution-stage-records.rs:33-40) and `NameBindingPriorityKind::Prelude` (:63-69). Use that priority as the proof, not a textual heuristic.

## Q3 — The crate field

For written-path items, crate = "core" is proven *as the path root written in itoa's source*. For `u8::MAX` and `u128::from`, crate "core" is a guess: nothing in read source says a builtin's member lives in core. For prelude names, "core" is doubly inferred (no_std prelude, then its contents). Make the field an enum, not a string: `WrittenAsRoot("core")`, `InferredPrelude("core")`, `Unknown`. True wordings: "crate named by the path root in itoa's source; definition not read"; "member MAX of builtin u8; no impl in read source declares it; defining crate not read"; "name bound by the implicit prelude (priority Prelude); prelude not read; crate core inferred".

## Q4 — Extracted or a distinct status

Keep `Extracted`, because the uncertainty then lives in exactly one place: the node's category, not-read flag and proof kind. Today `Extracted` means "target is a node with a real occurrence key" (:697-702), and an agent reading a bare `Extracted` could wrongly conclude the target's span is real evidence. That is only safe if every reader joins to the node — so also mirror the proof kind into the edge's dispatch metadata (the field already exists, `HldEdgeInputRecord.edge_dispatch_metadata_text`). A second status variant would duplicate information the node already carries.

## Q5 — Identity

One node per distinct written name, shared by all rows: the nine `u128::from` sites share one node; `u8::MAX` and `i8::MAX` are separate nodes (different members, and an agent asking "which bounds does this crate read" wants them apart). The key must include the proof kind, so a path-proven `core::mem::MaybeUninit` never unifies with a hypothetical prelude `MaybeUninit`, and a member-of-builtin `u128::from` never unifies with a written-path `core::…::from` if both ever exist. Proposed key: (identity kind, crate field, full member path as written, proof kind).

## Q6 — Guesses and omissions

Guesses: crate "core" on member-of-builtin rows (Q3); and `MaybeUninit::<u8>::uninit` stored as "core::mem::MaybeUninit::uninit" over-claims — a trait in scope could declare `uninit` (itoa proves none is, but the stored form should be "member `uninit` of the item named core::mem::MaybeUninit", not a merged definition path). Omissions: the `.try_into` rows could carry the prelude trait name `TryInto` as *inferred metadata* (same proof class as `Default`), not as a target — optional, cheap information. The draft correctly keeps `.write`, `.get_unchecked`, `.len` etc. unresolved; note the bound-method machinery already records `missing_source_traits` lists (original-call-resolution-records.rs:1027-1077, test at original-call-resolution-tests.rs:1024-1028), which is where those 138 rows keep their names.

## Q7 — Build order

1. **Imports (4 rows).** Name fully written; machinery exists. Test: production run on pinned itoa, excluded sysroot; read back `edge-evidence-record-table`; assert exactly four `imports` rows, `Extracted`, targeting nodes "core::hint", "core::mem", "core::mem::MaybeUninit", "core::str", each with not-read flag and proof "path as written"; assert two rows for `{self, MaybeUninit}`; assert file, range, digest per site.
2. **Written-path calls (6 rows).** Same proof kind; assert `mem::size_of` and `str::from_utf8_unchecked` target the *same nodes* as step 1 (tests unification).
3. **`uses_type` 13 + `MaybeUninit::uninit`.** Assert all point at the one `MaybeUninit` node; the member row carries "member through external type".
4. **Prelude names (implements 3, uses_type 1).** Gate on binding priority `Prelude`; crate field `InferredPrelude`.
5. **Member-of-builtin (9 + 30).** Last, weakest identity. The test must include a fixture where a local trait declares the member and assert the row then resolves locally or stays `MissingTarget` — never the external node.

Each step keeps its `itoa-source-acceptance-tests` row-exact and re-measures the stored counts; 67 target-bearing rows and ~42 new node rows are then measurements, not arithmetic.

## What was done with it

- Adopted: a node row per named external item; a name-keyed identity (the span-keyed `ExternalDefinition` arm cannot hold an unread item); "path as written", never a definition path; no crate claimed for a builtin member or a prelude name; the gates against a local trait member, a shadowing import and a glob; `Extracted` with the proof mirrored on the edge; the build order.
- Changed: the new variant goes on `HldTargetIdentityValue`, not on `NameSymbolTargetKey`. Name lookup never produces it, so fewer places change.
- Changed: the identity holds the kind of name (`crate_rooted_path`, `prelude_name`, `builtin_member`); how one site reached the name is edge metadata. With the reviewer's key, `use core::mem::MaybeUninit` and `MaybeUninit<u8>` would be two nodes for one written path.
- Changed: `MaybeUninit::<u8>::uninit` is stored as the written path `core::mem::MaybeUninit::uninit` under the claim "outside read source", gated on no trait in read source declaring `uninit`. Read source does not prove whether `core::mem` or `MaybeUninit` is a module or a type, so "member of" is not claimed either.
- Not planned: the prelude trait name on `.try_into()` rows (Q4 of the part file).
- Checked in code: the `ExternalDefinition` arm requires `Definition(occurrence_key)` (hld-materialization-core-records.rs:937-951); `read_hld_target_unit` returns none for a builtin (:1071); `NameBindingPriorityKind::Prelude` exists. Not checked: the line numbers it gives in the five producer files.
