# Part 6 — targets in `core`

**State: IN PROGRESS. Decided 2026-11-15; work packet written 2026-11-16; no step built yet.**

## The answer

An edge whose target is outside the source that was read gets a **named external target**
when the read source proves the name, and stays unresolved with a reason when it does not.
On pinned itoa, 67 of the 205 rows stored without a target today are predicted to gain one;
the other 138 are method calls and stay unresolved.

## Decision

The user chose what is most explicit for a coding agent that reads the graph: a named
external target, with the sysroot source left out of the run.

- An edge into `core` names its target as far as itoa's source proves it, plus the crate and
  a flag that the target's source was not read (ARCH F3).
- `use core::mem::MaybeUninit` proves the full path. `u128::from(x)` proves `u128::from`, not
  which `From` impl.
- A method call on a value whose type comes from `core` (`.len()`, `.get_unchecked(..)`)
  proves only the method name. It stays unresolved, with that name and a reason.
- REQ-RUST-038.0 is revised to allow an excluded-sysroot run to pass on these terms.

Options not taken: running with the sysroot source (the one measured run did not finish name
resolution in 2,400 s; it loaded `core`, `alloc` and `std`, and a `core`-only run was never
measured), and storing every such edge unresolved with no target.

## 1 — Work mode

Change to existing behaviour across four crates: core row types, the DuckDB analysis adapter
(name lookup), the syntax span extractor (five edge producers), and the Turso store adapter
(codec). Real-source loop: pinned itoa 1.0.18, sysroot source excluded. One step per site
shape; each step is stub, failing test, passing test, measure, log.

## 2 — Executable requirement

**REQ-RUST-053.0 — named external targets.**

- **N1 What a named external target is.** A target identity with three fields: the kind of
  name, the crate name when the source writes it, and the item path as the source writes it.
  It is a node row of category `ExternalDefinition` with no file, no range, no item kind and
  no visibility. No span is invented for it.
- **N2 What it claims.** "The item this written path names is outside the source that was
  read." It never claims where the item is defined (a written path may be a re-export), and
  never which impl declares a member.
- **N3 Kinds of name.**

```text
kind                proof in read source                                    crate field   example
crate_rooted_path   the path starts at a crate name with no unit in this    the root      core::mem::MaybeUninit
                    run, written in full or composed through one use item   segment       core::mem::size_of
prelude_name        one segment; no scope on the lexical chain declares     none          Default
                    or imports it; the chain ends at the implicit prelude
builtin_member      a member reached through a builtin type; no stored      none          u8::MAX, u128::from
                    impl and no trait in read source declares that member
```

- **N4 The row.** The edge is stored `Extracted` with the external node as target. Its
  metadata names how this site reached the name (`path_root_names_crate`,
  `use_import_names_path`, `implicit_prelude_name`, `builtin_type_member`) and what was not
  read. Rows that write the same name share one node.
- **N5 When no name is given.** The row stays unresolved with its reason when any of these
  holds:
  - a nearer scope than the `use` item declares or imports the same first name, or holds a
    glob import;
  - for a prelude name, any scope on the chain imports the name or holds a glob import;
  - the path has two or more segments and a trait in read source declares a member with the
    name of the last segment (a local impl could be the target);
  - the site is a method call (the receiver's type is not proven).
- **N6 Method calls.** A method call whose receiver type is not determined stays unresolved,
  with the method name and the reason, as it is stored today.
- **N7 No rule names itoa, `core` items or a fixed list of members.** The crate names
  recognised without a unit are the five sysroot crates the name lookup already lists.
- **N8 Completeness.** The published graph stays `incomplete` in this part.

## 3 — Design by layer

Measured before design, on the store of commit `8afa22fe8` (103 nodes, 585 edges):

```text
rows  relation    site                                           stored today
4     imports     core::hint, core::mem (self), core::mem::MaybeUninit, core::str     ExternalSourceMissing
13    uses_type   MaybeUninit<u8>                                ExternalSourceMissing
1     uses_type   Copy (a bound)                                 ExternalSourceMissing
3     implements  Clone, Copy, Default for Buffer                ExternalSourceMissing
4     calls       mem::size_of::<Self>()                         ExternalSourceMissing
3     calls       hint::unreachable_unchecked(), str::from_utf8_unchecked(..),
                  MaybeUninit::<u8>::uninit()                    ExternalSourceMissing
9     calls       u128::from(x)                                  MissingTarget
30    reads       uN::MAX, uN::MIN, uN::BITS, iN::...            TraitSolverNeeded
138   calls       method calls: write 49, get_unchecked 39, try_into 13, ilog10 10, expect 8,
                  len 6, unsigned_abs 5, unwrap 5, as_mut_ptr 1, cast 1, rev 1
205   total
```

The name lookup stores only the reason `external_source_missing` for such a path; it does not
store the crate or the path it proved. That is the first producer gap.

- **Core row types (L1).** `HldTargetIdentityValue` gains a variant for a named external
  item. Node validation gains an arm for it, shaped like the builtin arm. The existing
  `ExternalDefinition` arm needs a real occurrence key, which an unread crate cannot give.
  The variant is on the graph identity, not on `NameSymbolTargetKey`: name lookup never
  produces it, so fewer places change.
- **DuckDB analysis adapter.** `namespace-final-path-products.sql` already decides
  `external_source_missing` by three proofs (crate name at the path root; first name bound by
  a `use` whose path starts at such a crate; one-segment name ending at the prelude). It
  gains the name each proof gives and the stricter conditions of N5. The name is stored as a
  new field on `path-resolution-site-records` and `import-resolution-site-records`.
- **Syntax span extractor.** The five producers (`use-import-edge-records`,
  `impl-header-implements-records`, `signature-type-edge-records`, `body-value-read-records`,
  `original-call-resolution-records`) read that field. Each returns the external node rows
  it targets, as `signature-type-edge-records` does for builtin nodes today.
- **Turso store adapter and export.** The identity codec gains the variant.

## 4 — Verification matrix

Each step adds its tests and its row here. Built so far: steps 6.1 to 6.4.

```text
rule       test (exists)                                              where
N1 N3 N4   test_itoa_imports_row_counts                               itoa-source-acceptance-tests.rs
N1 N3 N4   test_itoa_external_path_calls (7 calls, 8 external nodes)  itoa-source-acceptance-tests.rs
N6         same test: no method call row carries a proof record       itoa-source-acceptance-tests.rs
N1 N3      test_external_import_names_target, test_external_record_fields_checked   use-import-edge-tests.rs
N1 N4 N5   test_external_callee_names_target                          callee-path-call-tests.rs
N1 N3 N4   test_itoa_external_type_uses (13 rows, no new node)        itoa-source-acceptance-tests.rs
N1         test_signature_external_type_named                         itoa-source-acceptance-tests.rs
N1 N4 N5   test_external_type_names_target                            signature-type-edge-tests.rs
N5         test_written_path_last_name                                signature-type-edge-tests.rs
N1         test_self_unresolved_header_reason (Self takes no name)    signature-type-edge-tests.rs
N1 N3 N4   test_itoa_prelude_name_targets (4 rows, 3 new nodes)       itoa-source-acceptance-tests.rs
N1         test_buffer_prelude_traits_named                           itoa-source-acceptance-tests.rs
N1 N4      test_external_trait_names_target                           impl-header-implements-tests.rs
N3 N5      test_external_name_proof_rows (a bound is not a glob import or a binder)   published-generation-copy-tests.rs
```

Planned at the start of the part:

```text
rule   real-source test on pinned itoa (planned)          fixture test (planned)
N1 N4  step 6.1: the four imports target external nodes   an external node row passes validation; one with a file or a kind fails
N2 N3  one per step: node identity and edge metadata      the name each proof gives, from stored lookup rows
N5     none (itoa has no such site)                       a shadowing import, a glob, a local trait member: no name
N6     the 138 method calls keep name and reason          none
N8     test_itoa_pipeline_reads_published (exists)        none
```

## 5 — Implementation plan

Predicted on itoa before any code. Edges stay 585 in every step.

```text
step  shape                                   rows gaining a target   new nodes   nodes after   unresolved after
6.1   imports                                 4                       4           107           201
6.2   calls by a written path                 7                       4           111           194
6.3   uses_type MaybeUninit<u8>               13                      0           111           181
6.4   prelude names (implements 3, bound 1)   4                       3           114           177
6.5   builtin members (calls 9, reads 30)     39                      31          145           138
```

Measured after each built step (pinned itoa, release run):

```text
step  rows gaining a target   new nodes   nodes after   unresolved after   as predicted
6.1   4                       4           107           201                yes
6.2   7                       4           111           194                yes
6.3   13                      0           111           181                yes
6.4   4                       3           114           177                yes
```

Step 6.4 as built: an `implements` row whose trait is left unresolved with
`ExternalSourceMissing` takes the name its stored lookup proved; the impl header row carries
that name as `external_name_record`. The third condition of N5 is not applied to the trait
of an impl header: it is never a member reached through a type. The bound `Copy` needed a
correction to the proof in SQL, not a new rule: a bound's request for its trait is stored
beside the `use` requests as a glob, and the proof counted it as a glob import of the scope.
Only `use` requests are glob imports or binders now.

Step 6.3 as built: the third condition of N5 for a type path reads the names of the written
path outside `<..>` and `(..)` groups; a leading `<A as T>` counts as one name. `Self`
takes no name from a lookup stored at its site.

Step 6.2 as built: the row's `dispatch_kind` is `external_written_path`. The third
condition of N5 is applied in the call producer from the node rows; a one-segment callee is
not blocked by it. A blocked row says `external_member_name_in_local_trait`.

## 6 — Quality gate results

Per step: syntax crate tests, the real-source itoa suite, format and clippy on the touched
crates (see the log). Workspace gates and timed runs: not run yet, taken at the end of the part.

## 7 — Open questions

- **Q1** The expected graph's 119 external rows carry full definition paths such as
  `<u128 as core::convert::From<u64>>::from` and `core::num::<impl i128>::MAX`. Read source
  does not prove them. They are cut back to the written name in part 7.
- **Q2** The five signed `write` calls `Unsigned::fmt(self.unsigned_abs(), ..)` need a
  `core` return type to choose the impl member. They stay on the trait's declaration.
- **Q3** An import through another import (`use core::mem; use mem::MaybeUninit;`) is not
  covered by the stored proof. It stays unresolved. itoa has none.
- **Q4** The trait of `.try_into()` could be stored as a prelude name on the unresolved row.
  Not planned.
- **Q5** A crate that is not a sysroot crate and has no unit in the run is not recognised.

## Second opinion

GLM-5.3 reviewed the draft design before this packet was written:
`docs/devin-research/1794796183-second-opinion-glm-5-3-max-part-6-external-targets.md`.
Adopted: a node, not a name inside each edge row; a name-keyed identity, not the span-keyed
`ExternalDefinition` arm; no crate claimed for a builtin member or a prelude name; the
conditions of N5. Changed: the variant sits on the graph identity, and the kind of proof for
one site is edge metadata, so an import and a use of the same written path share one node.
