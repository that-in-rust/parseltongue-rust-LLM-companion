# Part 8 — method calls into `core`

Status on 2026-11-16: **108 of the 133 calls are named and equal their frozen rows, 25 have
no target (sections 10 to 12 built). The order changed by user decision: the expected target of all 133 calls is written
from source and frozen first (section 9, REQ-RUST-058.0), then the lookup is built to equal
it.** Counts are on the log's "Where we are" page.

## Requirement

A method call whose receiver type is declared in `core` gets a target when itoa's source
proves the receiver's type and the `core` source shows exactly one method that Rust's lookup
order can reach. Otherwise the row stays without a target and keeps its reason. A missed
target is better than a wrong one.

## 1. Measurement (2026-11-16, commit `ff01f59e1`, executable `43d999e1…`)

```text
sites  method          receiver in itoa                         declaration found in core (syntax scan)
49     .write          buf[i], an element of the byte buffer    impl<T> MaybeUninit<T>, (&mut self), mem/maybe_uninit.rs
39     .get_unchecked  DECIMAL_PAIRS.0 (38), buf: &[T] (1)      impl<T> [T], (&self), slice/mod.rs
13     .try_into       &mut buf[a..] (5), an integer literal (8) trait TryInto, convert/mod.rs
10     .ilog10         uN::MAX, iN::MAX                         inside macro_rules! uint_impl / int_impl
 8     .expect         the result of try_into                   impl<T, E> Result<T, E>, (self), result.rs
 5     .unwrap         the result of try_into                   impl<T, E> Result<T, E>, (self), result.rs
 5     .len            buf: &mut Self::Buffer                   impl<T> [T], (&self), slice/mod.rs
 1     .len            string: &str                             impl str, (&self), str/mod.rs
 1     .as_mut_ptr     self.bytes, an array field               impl<T> [T], (&mut self), slice/mod.rs
 1     .cast           the result of as_mut_ptr                 impl<T> *mut T, (self), ptr/mut_ptr.rs
 1     .rev            (1..4), a range                          trait Iterator, iter/traits/iterator.rs
```

- **Reading the whole sysroot fails.** Run with `--sysroot-source-scope-kind included`:
  exit 1 after 170.94 s, 1,555.9 MB peak, 1.9 GB store. Error:
  `Namespace(Copy { .. "Turso/DuckDB row-count or content-digest mismatch" })` after
  `syntax-namespace-input-paths` (135,793 rows). Not fixed here.
- **A syntax-only scan of `core` is cheap.** 288 files, 6,419,461 bytes, 2,133 impls,
  247 traits, 4,509 functions in impls and traits, 0 parse errors: 0.27 s, 22 MB
  (throwaway `/home/ubuntu/phase7-work/core-index-probe`, not in the repository).
- **Other declarations of the same names in `core`** (the scan lists them all): `write` 23
  (`Hasher`, `io::Write`, `PostPadding`), `get_unchecked` 46 (`SliceIndex` impls, `str`),
  `len` 41, `as_mut_ptr` 6, `cast` 6, `expect` 2 and `unwrap` 2 (`Option`, `Result`).
- **Why the directory changes the rows** (found the same day): the store's file identity is
  the absolute path, and `compute_workspace_generation_digest` hashes those rows. Open
  point, not part of this part.

## 2. What a proof needs

Rust looks a method up in this order: the receiver's type, then each type reached by
dereferencing, then the slice of an array. At each type it tries the value, then `&`, then
`&mut`. At each of those it takes a method declared on the type before a trait's method.

So a target is proven only when:

1. **The receiver's type is proven from itoa's source.** Written types only: a parameter, a
   `let` annotation, a field, a static or const, `Self::Buffer` through the one impl that
   defines it, and the element of an array or slice.
2. **`core` declares one method of that name at the first place the order reaches.**
3. **No earlier place has a method of that name.** A trait of itoa is ruled out when none
   of its impls is for the type (all its impls are in itoa). A trait of `core` in the
   prelude cannot be ruled out without solving trait bounds; the row then records the
   assumption, or stays without a target. Decided per method in its step.

## 3. Limits of a syntax-only scan (recorded on every row that uses it)

- A method written inside a `macro_rules!` body in `core` is seen as text, not as a member
  of a type (`ilog10`).
- A method whose name is a macro parameter is not seen.
- `cfg` and stability attributes are recorded, not evaluated.
- `alloc` and `std` are not scanned. itoa is `#![no_std]`.

## 4. Steps

```text
8.1  the scan as a production stage: one row per method declared in core            REQ-RUST-054.0
     new value for --sysroot-source-scope-kind; the SOP process audit allows reads
     under <sysroot>/lib/rustlib/src/rust/library/core/src
8.2  receiver type from itoa's source                                               REQ-RUST-055.0
8.3  the lookup order over 8.1 and 8.2; targets for write, get_unchecked, len,     REQ-RUST-056.0
     as_mut_ptr, cast (predicted 96 sites, not measured)
8.4  try_into (13), then expect and unwrap (13): need the declared return type
8.5  ilog10 (10): needs core's macro calls expanded; rev (1)
8.6  expected graph re-cut and re-frozen; new acceptance run
```

## 5. Questions, decided by the user on 2026-11-16

- **Q1. The label of a method on a slice, an array or a raw pointer.** `core` writes
  `impl<T> [T]` and `impl<T> *mut T`. Decided: the impl's self type as written, without
  its generic names bound: `[T]::len`, `*mut T::cast`.
- **Q2. A prelude trait of `core` that declares the same name** (`ExactSizeIterator::len`
  for `buf.len()` on `&mut [T; N]`). Decided: name the target and record the assumption
  that the trait is not implemented for the earlier types. This replaces "no target" in
  REQ-RUST-056.0 P5 for that case; the requirement text is revised in section 9.
- **The `core` read is an input of the run** (PRD, SOP Rule 2 and S1).

## 6. REQ-RUST-054.0 — the member declarations of `core` (step 8.1, added 2026-11-16)

One row per method that a `core` source file declares, from syntax only.

- D1. A row for each `fn` written directly in an impl or in a trait declaration.
- D2. The row holds: the file path relative to the scanned directory; the byte range of
  the name; the name; the owner kind (`inherent_impl`, `trait_impl`, `trait_declaration`);
  the impl's self type and generic parameters as written; the trait path of a trait impl
  or the name of the declaring trait; the self parameter kind (`value`, `reference`,
  `mutable_reference`, `typed`, `none`); the return type as written, or null; the
  attributes written on the function and on its owner; whether the owner is inside a
  function body.
- D3. A row with owner kind `macro_definition_body` for each `fn NAME` written in the body
  of a `macro_rules!` or `macro` definition, with the macro's name. `fn $name` gives a row whose
  name is the parameter text and whose parameter flag is true. No self type is claimed.
- D4. Rows are ordered by the byte offset of the name. The row key is the path and that
  offset.
- D5. Nothing is resolved: no path, no `cfg`, no macro call.
- D6. A run with `--sysroot-source-scope-kind declarations` writes those rows to the
  stage table `sysroot-member-declaration-records` for every `.rs` file under
  `<sysroot>/lib/rustlib/src/rust/library/core/src`. Files are taken in path order. The
  path in a row is relative to `library` (`core/src/mem/maybe_uninit.rs`); the row key is
  that path, `#`, and the name's byte offset.
- D7. That run adds no crate to the crate graph: like `excluded`, no unit is a sysroot
  unit, and the stored nodes and edges are those of an `excluded` run.
- D8. A run with `excluded` or `included` writes no row to that table.
- D9. A file that cannot be read as UTF-8 fails the run. Nothing is skipped silently.

```text
test_inherent_member_row_fields      D1 D2   crates/parseltongue-syntax-span-extractor/src/sysroot-member-declaration-tests.rs
test_trait_member_row_fields         D1 D2   same file
test_macro_body_member_rows          D3 D5   same file
test_member_rows_offset_order        D4      same file
test_core_directory_member_rows      D6 D9   crates/parseltongue-cli-dispatcher-binary/src/sysroot-member-declaration-tests.rs
test_core_member_rows_published      D6 D7 D8  crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs
```

## 7. Steps 8.2 and 8.3 — the work packet (spec first)

**Rust work mode:** spec and delivery. The risk is a wrong target, not speed.

### REQ-RUST-055.0 — the receiver's type, from written source only

- R1. WHEN the receiver of a method call is a name bound by a parameter whose type is
  written in the source THEN the site record SHALL carry that type's text and byte range,
  whatever its shape (`&mut [MaybeUninit<u8>]`, `&mut Self::Buffer`, `[u8; 200]`).
- R2. WHEN the receiver is `self.FIELD`, a path to a `static` or `const`, a tuple or
  named field of one of those, or an index of one of those THEN the record SHALL say which
  shape it is and name the field, item or indexed base.
- R3. WHEN the receiver is anything else (a call's result, a local without an annotation,
  a name a macro bound) THEN the record SHALL say `other` and carry no type. Nothing is
  inferred.
- R4. WHEN the receiver name may be rebound in the body THEN R1 and R2 SHALL not apply.
- R5. The extractor SHALL stay syntax-only: no path resolution, no `cfg`, no projection of
  `Self::Buffer`.

### REQ-RUST-056.0 — a target only when the declaration is proven

- P1. The receiver type is read from R1/R2 evidence: a parameter's written type, a field's
  declared type, a `static`'s declared type, the element type of an indexed array or slice.
  A projection (`Self::Buffer`) is followed only through an associated type written in a
  stored impl of this workspace.
- P2. The probe is Rust's: for the receiver type `T` the steps are `T`, each dereference of
  `T`, then the slice an array unsizes to. At each step the candidate receiver types are
  `S`, `&S`, `&mut S`, in that order.
- P3. A declaration fits a step when it is an inherent member of a `core` impl whose
  written self type unifies with `S` (the impl's own generic parameters are wildcards) and
  its `self` parameter kind gives that step's receiver type.
- P4. A target is named only when exactly one declaration fits at the first fitting step,
  and no method of that name declared by a trait in scope can fit earlier. Traits in scope
  are the traits this workspace declares or imports, plus the 2021 `core` prelude.
- P5. A trait method is refuted only from impls that are stored: this workspace's impls,
  and the `core` impls the declaration table holds. WHEN a trait in scope declares the
  name and its impls cannot all be read THEN the row SHALL stay without a target, with the
  reason `trait_solver_needed`.
- P6. The name of a named target is the impl's self type as `core` writes it, then `::`,
  then the method name (`MaybeUninit<T>::write`, `[T]::len`).
- P7. Every named target SHALL carry its proof: the receiver type, how it was proven, the
  step that fitted, the declaration's file and byte range, and every assumption.

### Design, by layer

```text
L1  the written-type model and the probe: pure functions over text and rows
    crates/parseltongue-syntax-span-extractor/src/receiver-type-probe-records.rs (new)
L2  the lookup over stored rows, through the existing store port
    original-call-resolution-records.rs, type-relative-member-records.rs
L3  no new effect; the rows come from the stage table built in 8.1
```

### Verification matrix

```text
req  test                                     type         assertion
055  test_parameter_written_type_rows         fixture      a written type of any shape is recorded
055  test_field_and_static_receivers          fixture      self.bytes, DECIMAL_PAIRS.0, buf[i]
055  test_unwritten_receiver_stays_other      fixture      a call's result carries no type
056  test_slice_member_probe_steps            fixture      &[T] finds [T]::get_unchecked at step 1
056  test_array_unsize_probe_order            fixture      [T; N] reaches [T]::len after the array
056  test_scope_trait_blocks_target           fixture      a trait in scope that may fit earlier wins
056  test_core_member_call_targets            real source  the itoa calls this slice names
```

### Plan

```text
8.2a  parameters with any written type (REQ 055 R1, R3, R4)
8.2b  fields, statics and index expressions (R2)
8.3a  the probe and the proof on slices and MaybeUninit (REQ 056 P1 to P7)
8.3b  Self::Buffer through this workspace's impls
```

### Open, and the limit I cannot pass

- The declaration table holds methods, not impl headers. A `core` impl that overrides no
  method (`impl ExactSizeIterator for X {}`) leaves no row, so P5 cannot refute it. Of the
  five names in this slice only `len` is declared by a prelude trait
  (`ExactSizeIterator::len`), so only the 5 `buf.len()` calls meet this limit.
- Q2 stands: either those 5 rows stay without a target, or they are named `[T]::len` with
  the assumption recorded. I build P5 (no target) and leave the assumption variant to you.

## 8. Step 8.3b-2 — a parameter typed `Self::NAME` (spec first, added 2026-11-16)

Work mode: Spec + Delivery + Reliability. Component: the L2 lookup of a library crate.
The main risk is a wrong target, so every rule below ends in "otherwise no target".

### Measured before writing (commit `07b95dfbc`, scope `declarations`)

```text
 124 calls without a target
  46 of them: the receiver is a parameter whose written type is "&mut Self::Buffer"
     reason stored today: receiver_type_name_unproven
 where itoa writes the type:
   trait Unsigned: Integer            fn fmt(self, buf: &mut Self::Buffer)
   trait Integer: private::Sealed
   trait Sealed: Copy                 type Buffer: 'static;  fn write(self, buf: &mut Self::Buffer)
   impl private::Sealed for uN / iN   type Buffer = [MaybeUninit<u8>; Self::MAX_STR_LEN];
   impl private::Sealed for usize     type Buffer = <u64 as private::Sealed>::Buffer;
   impl Unsigned for uN               fn fmt(self, buf: &mut Self::Buffer)   (no "type Buffer" here)
```

### The Rust rule this follows

In an impl of a trait, `Self::NAME` in type position names the associated type `NAME` of
the impl's trait or of one of its supertraits, for the impl's self type. Two traits of
that set declaring `NAME` is error E0221. In an inherent impl it is error E0223. So in
source that compiles, one trait of that set declares `NAME`, and the type is what that
trait's impl for the self type writes after `type NAME =`.

### REQ-RUST-057.0 — the type behind `Self::NAME` in an impl

- A1. WHEN an impl writes `type NAME = T;` THEN its stored member row SHALL carry the text
  of `T`, its byte range, and the byte range of each type path in `T` (paths under an
  expression, such as an array length, excluded). Built in step 8.3b-2a.
- A2. WHEN the receiver is a parameter whose written type, less its reference layers, is
  exactly `Self::NAME` THEN the receiver type SHALL be those reference layers around the
  type of A1, and only when A3 to A7 all hold.
- A3. The function is a member of a stored impl that is not inside a function body, and
  that impl's trait and self type both have a stored target.
- A4. Among the stored impls of that self type, and the stored impls whose self type has
  no stored target, exactly one member is named `NAME`; it is a `type` member of a trait
  impl with a stored trait, no generic parameter and no negation.
- A5. That impl is the function's own impl, or its trait is reached from the function's
  impl's trait through stored supertrait bounds. A supertrait bound that is not stored as
  resolved is not followed.
- A6. That impl's self type has the same stored target and the same written text as the
  function's impl's self type.
- A7. Every type path in the text of A1 is proven as REQ-RUST-056.0 proves a parameter's:
  a builtin, or a name of `core`. A path through `Self`, a qualified path
  (`<u64 as Sealed>::Buffer`) and a workspace type are not proven.
- A8. WHEN any of A3 to A7 fails THEN the row SHALL stay without a target, with the reason
  `associated_type_impl_unproven` (A3, A4, A6), `associated_type_trait_unproven` (A5) or
  `receiver_type_name_unproven` (A7).
- A9. A named target SHALL carry, besides the proof of REQ-RUST-056.0 P7: the proof kind
  `parameter_associated_type_alias`, the text of A1, the stored occurrence of the
  `type NAME` member and of its impl, and the assumption `source_compiles_one_associated_type`.
- A10. REQ-RUST-056.0 P2 to P6 apply unchanged to the receiver type of A2.

Not in this requirement: `Self::NAME` in a trait's own default body (the self type is not
known there), `Self::NAME<..>`, `T::NAME`, and a `type NAME` written by a macro call
inside the impl body.

### Design, by layer

```text
L1  no change: parse_written_type_shape and probe_core_member_declaration
    (receiver-type-probe-records.rs) take the alias text as they take a parameter's text
L2  core-member-target-records.rs
      find_alias_receiver_type      A2 to A8 over the stored impl header rows
      find_core_member_target       calls it when the written type is Self::NAME
    original-call-resolution-records.rs
      BoundMethodLookupIndex::check_trait_supertrait_reach   A5, over stored bound rows
    type-relative-member-records.rs
      ImplMemberLookupIndex::find_type_member_target         A4, used as it is
L3  no new effect and no new stored table
errors  a withheld target is a reason value on the row, not an error; store and decode
        failures stay OriginalCallResolutionError
```

### Verification matrix

```text
req     test                                 type         assertion
057 A1  test_alias_member_type_evidence      fixture      "type Buffer = [MaybeUninit<u8>; Self::LEN]" row: text, range, 2 paths
057 A2  test_alias_parameter_core_member     fixture      impl Sealed for u8 with type Store = [MaybeUninit<u8>; 3]:
        A9                                                buf[0].write(1) -> MaybeUninit<T>::write, proof kind and alias text
057 A5  test_alias_parameter_core_member     fixture      fn in impl Unsigned for u8, Unsigned: Sealed -> same target;
                                                          trait with no supertrait bound to Sealed -> associated_type_trait_unproven
057 A4  test_alias_parameter_core_member     fixture      two impls writing "type Store" -> associated_type_impl_unproven
057 A7  test_alias_parameter_core_member     fixture      type Store = <u8 as Other>::Store -> receiver_type_name_unproven
057     test_itoa_core_member_targets        real source  counts of named and unnamed calls on pinned itoa, by method
057     test_core_member_rows_published      real source  default scope still 151 nodes, 682 rows
```

Deliberate breaks after GREEN, each must fail a test: drop A5, drop A6, drop A7.

### Plan

```text
STUB      find_alias_receiver_type and check_trait_supertrait_reach with todo!(); cargo check
RED       test_alias_parameter_core_member fails; test_itoa_core_member_targets fails on new counts
GREEN     minimum code; both pass; commit
REFACTOR  combinators, shared helpers, or a "no refactor needed" note; fmt, clippy, tests; commit
gates     extractor crate tests, 60 real-source itoa tests, workspace tests, release build
```

### Open questions

- Q1. The real-source counts are not known before the run. The RED run of the real-source
  test records what production gives today (9 named); the GREEN run gives the new counts,
  and each newly named call is checked against the source before the counts are fixed.
- Q2. `buf.len()` on `&mut [MaybeUninit<u8>; N]` (5 calls): the prelude trait
  `ExactSizeIterator` declares `len`; REQ-RUST-056.0 P5 keeps those without a target.
- Q3. Inside `impl Sealed for iN`, `buf[offset].write(b'-')` (5 calls): the trait `Sealed`
  is in scope and declares `write`. P5 names the target only if the stored impls show
  `Sealed` is not implemented for `MaybeUninit<u8>`.

## 9. The expected target of every method call into `core` (spec first, added 2026-11-16)

### Rust work mode

Spec + Reliability. Component: the expected data set (Python, test side) and one
real-source test (Rust). No production code. The risk is a wrong expected target: the
author of the expected rows is also the author of the mapper.

### Executable requirements: REQ-RUST-058.0

- T1. WHEN the frozen expected graph lists a `calls` site without a target (133 sites, 55
  rows) THEN the file `itoa-expected-core-member-targets.json` SHALL hold exactly one row
  for that site, keyed by the calling item, the file and the byte range (a site inside a
  macro body has one row per item the macro generates). No other site has a row.
- T2. Each row SHALL give either a target label, or `no_target` with a reason. `no_target`
  is allowed only when Rust's lookup rules and the two read sources (pinned itoa, the
  frozen `core` files) cannot prove one declaration.
- T3. A target label SHALL be the owner as `core` writes it, `::`, the method name
  (decided, Q1): an inherent method takes the impl's self type (`MaybeUninit<T>::write`,
  `[T]::get_unchecked`, `*mut T::cast`, `str::len`); a trait's method takes the trait's name
  (`TryInto::try_into`, `Iterator::rev`).
- T4. Each row with a target SHALL cite its proof, so a reader can check it without a
  tool: the receiver's type; the itoa line that proves that type; the `core` file (relative
  to `library`), line and text of the declaration; the lookup step that reaches it (the
  type itself, a dereference, the slice of an array; by value, `&`, `&mut`).
- T5. Each row SHALL list its assumptions; an empty list means none. An assumption is
  something the row rests on that was not proven. A call that a prelude trait could also
  answer SHALL carry `prelude_trait_impl_unrefuted` (decided, Q2).
- T5a. A field SHALL have one meaning, so that an agent need not read prose (review of
  2026-11-16). A fact is not an assumption: a declaration written inside a `macro_rules!`
  body of `core` is `declaration_form_kind = core_macro_body`, with the macro and the impl
  that calls it cited. The crate is its own field (`core`). A trait label says
  `trait_declaration_impl_not_named`. A receiver type that no source writes is absent, with
  `receiver_type_source_kind = inferred_not_written`; this is allowed only where the name
  alone decides the target, and a test holds the list of such methods.
- T6. The file SHALL record the compiler version (`rustc -vV`) and one sha256 over the
  `sha256sum` listing of every `.rs` file under `library/core/src`, in path order. WHEN a cited
  declaration line does not hold the cited text in those files THEN the data-set test SHALL
  fail.
- T7. The file is frozen like the expected graph: its sha256 is in a freeze record, and a
  change needs a source citation and a new freeze. It is never a production input.
- T8. WHEN a run with scope `declarations` stores a target for one of the 133 sites THEN
  the real-source test SHALL fail unless the fields the file lists as `compared_fields`
  equal the row: the label, the crate, the owner kind, the declaration's file and line. The
  stored row SHALL carry every assumption the expected row lists. It may carry more: the
  assumptions of the mapper's own proof (today `core_macro_expansions_import_no_trait` and
  `source_compiles_one_associated_type`). The test fixes the set of those kinds, so a new
  kind fails until it is reviewed. A site the run leaves without a target is counted, not
  failed, until step 8.6. The test also fails when the `core` source of the compiler in
  use is not the one the rows were written from (T6).
- T9. Part 8 is DONE when the count of T8 is 133 of 133 equal (rows with `no_target`
  equal when the run stores no target and the same reason). Then the rows are merged into
  the expected graph, it is re-frozen, and S1 to S4 run with `declarations` (step 8.6).

### Design (L1 / L2 / L3)

```text
L1  itoa-expected-core-member-targets.json   the 133 rows: data, no logic
    itoa-expected-graph-rules.py             the vocabulary: labels, steps, assumptions
L2  itoa-expected-core-target-builder.py     reads the pinned source, the frozen graph and
                                             the core files; checks T1, T4, T6; writes L1
L3  none. Production never opens these files (SOP Rule 2).
test side (Rust, later step)
    itoa-source-acceptance-tests.rs          test_itoa_core_member_targets reads L1 and
                                             compares as T8; pure functions over two row
                                             lists, iterator pipelines, no mutable state
errors  a row that fails T1, T4 or T6 stops the builder with the row named; nothing is
        skipped
```

### Verification matrix

```text
req      test                                  type         assertion                                metric
058 T1   test_core_target_site_coverage        data set     rows == unresolved call sites of the     133 of 133
                                                            frozen graph, by file and byte range
058 T2   test_core_target_label_or_reason      data set     each row has a label xor a reason        133 rows
058 T3   test_core_target_label_shape          data set     label == cited owner text + "::" + name  rows with a label
058 T4   test_core_declaration_citation_text   data set     cited core line holds the cited text     rows with a label
058 T5   test_core_target_assumption_names     data set     assumptions are from the vocabulary;     133 rows
                                                            the 5 buf.len() rows carry the Q2 value
058 T6   test_core_source_digest_recorded      data set     version and digest equal the files read  1
058 T7   test_core_target_freeze_digest        data set     file sha256 == freeze record             1
058 T8   test_itoa_core_member_targets         real source  no stored target differs from its row;   equal of 133
                                                            prints equal, missing, wrong
```

### Implementation plan

```text
9.1  this section; decisions in PRD, SOP, ARCH, AGENTS.md           done (documents)
9.2  read each of the 55 rows' sites in the pinned source and the declaration in core;
     write the builder and the 133 rows; data-set tests T1 to T6 pass    done 2026-11-16:
     133 rows
9.3  review. The user asked for a self-review from the point of view of a coding agent
     that reads the row, and for the call to be taken. Six changes followed (T5a, T8 and
     three checks that replace prose claims); the log entry of 21:05 lists them    done
9.4  freeze (T7): itoa-expected-core-target-freeze.json, 11 tests pass               done
9.5  test_itoa_core_member_targets compares with the file (T8): STUB, RED, GREEN, REFACTOR
     done 2026-11-16: 17 stored targets, 17 equal, 0 wrong, 116 without a target
9.6  close step 8.3b-2b against the frozen rows, then 8.3b-3, 8.4, 8.5 in the order the
     missing count falls fastest
```

### Quality gate results

9.4, 2026-11-16: `python3 -m unittest test_itoa_expected_core_targets` runs 11 tests, all
pass, against rustc 1.99.1 and the vendored copy of the pinned source (same sha256 as the
pin). No Rust was written, so no cargo gate ran.

### Open questions

The user answered Q1 to Q3 on 2026-11-16 ("as long as you are moving towards a solution
that is more explicit"); the rows use the proposals. Each stays open to the row review (9.3).

- Q1. **Trait methods** (`try_into` 13, `rev` 1): the label `TryInto::try_into` names the
  trait's declaration, as the PRD does for itoa's own traits. The impl that runs (`impl<T,
  U> TryInto<U> for T where U: TryFrom<T>`) is not named. Used: the trait label, with the
  blanket impl cited in the proof.
- Q2. **`ilog10`** (10): `core` writes it inside `uint_impl!` and `int_impl!`, called from
  `impl u64 { .. }`. Proposed label: the builtin type, as the PRD does for `u8::MAX`
  (`u64::ilog10`), with `core_macro_body_declaration`.
- Q3. **`expect` and `unwrap`** (13): the receiver is the result of `try_into`, so its type
  comes from a `core` return type (`Result<U, U::Error>`). Proposed label
  `Result<T, E>::expect`, with the return type cited as a proof step.
- Q4. **Which compiler.** No file pins the compiler. T6 records the one read; the SOP run
  must use the same one or the freeze fails.

## 10. A receiver that is a `static`, then fields (spec first, added 2026-11-16)

### Rust work mode

Delivery Mode and Reliability Mode. Component: the Rust resolver of a batch mapper. The
main risk is a wrong target: a field read at the wrong position, or a type name taken for
a `core` type without proof.

### Measured before writing (commit `d1ab2f1ee`, scope `declarations`)

```text
 38  get_unchecked   receiver DECIMAL_PAIRS.0   reason receiver_type_not_determined
     the stored receiver place:  value_path "DECIMAL_PAIRS", then field_access "0"
     stored already (field-signature-detail-table):
       the declared type of the static   PATH_TYPE, STATIC, "DecimalPairs"
       the field of the struct           TUPLE_FIELD, "[u8; 200]", owner: the struct
```

Expected for the 38 (frozen rows): `[T]::get_unchecked`, `core/src/slice/mod.rs`, no
assumption.

### The Rust rule this follows

- A value path that names a `static` has the type the `static` declares.
- A field expression takes the field of the first type, in the order of automatic
  dereference, that has a field of that name. A struct's own field is found at step 0, or
  after the `&` and `&mut` layers of a reference are taken off. No `Deref` impl is
  consulted before that.
- A tuple field's name is its position among the fields that remain after `cfg`.

### Executable requirements: REQ-RUST-059.0

```text
S1  WHEN the first step of a receiver place is a value path
    THEN the lookup SHALL apply only if the stored value-path lookup at that range is
    resolved to exactly one symbol of kind Static with exactly one graph identity.
    Otherwise the lookup does not apply and the row keeps its reason.
S2  The type of the place SHALL start as the text of the one stored declared type whose
    owner is that static. None or several: the lookup does not apply.
S3  WHEN a field step follows
    THEN the type so far SHALL be one path without generic arguments, behind `&` and
    `&mut` only, that the stored lookup resolved to exactly one symbol of kind Struct with
    one graph identity; and the field SHALL be found among the stored fields of that
    struct: a tuple field by its position in source order, a record field by its name.
    The type of the place becomes the type that field writes.
    A stored field with an attribute, a field text that is not one field, a missing field,
    or a type that is not such a path SHALL withhold the name with
    receiver_field_type_unproven.
S4  Field steps SHALL come before index steps. Any other order: the lookup does not apply.
    A parameter followed by a field step: the lookup does not apply (not built).
S5  Every type path of the final written type SHALL be proven as in REQ-RUST-056.0 (a
    builtin, or a name of core), looked up in the file and expansion of the row that
    writes it. Otherwise receiver_type_name_unproven.
S6  The probe, the scope rules and the withheld reasons of REQ-RUST-056.0 then apply
    unchanged. The lookup record SHALL carry receiver_type_proof_kind
    (static_declared_type or static_declared_type_field), the declared type text, the
    path as written, and for each field step its name, its stored text and its type.
```

```text
S7  (added after S1 to S6 were built) WHEN the first step is `self` and a field step
    follows
    THEN the type of the place SHALL start as the self type of the stored impl that
    holds the function, only if: the function's `self` parameter is written `self`,
    `mut self`, `&self` or `&mut self`; the impl is not inside a function body; and its
    self type is one plain name, without generic arguments, resolved to a definition.
    The first field is found among the stored fields of that definition; S3 to S6 then
    apply. A `self` with a written type, a method a trait declares, or `self` with no
    field step: the lookup does not apply.
    receiver_type_proof_kind is self_declared_type_field.
```

A field that `cfg` removes has no stored row (the extractor skips inactive nodes), so the
position among stored fields is the position after `cfg`. An attribute on a stored field is
not read by this lookup, so S3 withholds: a missed target is preferable to a wrong one.

### Design (L1 / L2 / L3)

```text
L1  pure, in core-member-target-records.rs
      parse_stored_field_text        one stored field -> (name, written type)
      find_struct_field_type         fields of one struct -> the field of a step
      find_static_declared_type      the one declared type of a static
      build_row_path_evidence        a type path of a stored row -> its lookup key
      read_record_definition_identity  a stored path lookup -> one definition of a kind
      find_field_step_type, find_static_place_type   S1 to S5, a fold over the field steps
      find_parameter_place_type      the existing parameter rule, moved, unchanged
L2  read_core_member_index reads field-signature-detail-table into the index
L3  none added
```

No new stage, table or dependency. Typed errors are the existing
`OriginalCallResolutionError`.

### Verification matrix

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| S1, S2 | test_static_field_receiver_member (plain) | unit, fixture | a static with no field step gives the target | proof kind static_declared_type |
| S3 | same (tuple, record, removed) | unit, fixture | position 1 of two, a named field behind `pub(crate)` and `&`, position after a removed field | `[T]::get_unchecked` |
| S3 | same (gated, nested) | unit, fixture | attribute on a stored field; tuple type | receiver_field_type_unproven |
| S4 | same (indexed) | unit, fixture | field then index | `MaybeUninit<T>::write` |
| S5 | same (unproven) | unit, fixture | field type names a workspace struct | receiver_type_name_unproven |
| S7 | test_self_field_receiver_member | unit, fixture | `self`, `&self`, `&mut self` then a field; `self: &Self`; a trait's own method | `[T]::get_unchecked`; no target; no target |
| S1 to S7 | test_itoa_core_member_targets | real source | each named call equals its frozen row | wrong 0 |

### Open questions

- A receiver rooted at a parameter with a field step is not built.

## 11. A prelude trait that declares the same name (spec first, added 2026-11-16)

### Rust work mode

Delivery Mode and Reliability Mode. This builds decision Q2 of section 5. The risk is a
target that rests on something not proven, so the row must say so.

### Measured before writing (commit `3a4827d01`, scope `declarations`)

```text
  5  len   receiver buf, typed &mut Self::Buffer = &mut [MaybeUninit<u8>; N]
           reason scope_trait_may_fit
```

`ExactSizeIterator` is exported by the prelude of `core` and declares `len(&self)`. The
probe reaches `&mut [T; N]` taken by `&` and `[T; N]` taken by `&` before the slice. The
impls of a trait of `core` are not read, so the probe cannot show that neither type
implements it. Expected for the 5 (frozen rows): `[T]::len`, with the assumption
`prelude_trait_impl_unrefuted`.

### Executable requirements: REQ-RUST-060.0

```text
U1  WHEN at one receiver form no inherent member fits, and every trait in scope that may
    fit is a trait of core that is in scope through the prelude only
    THEN the probe SHALL go on to the next form and remember the names of those traits.
U2  "Through the prelude only" SHALL mean: the trait's name, or a name core renames it
    to, is exported by a prelude file of core, and none of those names is written in the
    workspace's read source.
U3  WHEN the probe then names a declaration
    THEN the lookup record SHALL carry the assumption prelude_trait_impl_unrefuted and
    the remembered trait names.
U4  WHEN the probe then names nothing, or withholds for any reason
    THEN the reason SHALL be scope_trait_may_fit, as before.
U5  A trait of the workspace, a trait of core whose name is written in the workspace,
    and a trait whose name is not known SHALL withhold at once, as before
    (REQ-RUST-056.0 P4, P5).
U6  A row that names its target at a form where an inherent member fits SHALL NOT carry
    the assumption.
```

This replaces "no target" in REQ-RUST-056.0 P5 for the case of U1 only. The assumption is
that the trait is not implemented for the types of the earlier forms; nothing in the run
proves it.

### Design (L1 / L2 / L3)

```text
L1  receiver-type-probe-records.rs
      ScopeTraitMethodValue        gains the prelude trait's name
      decide_probe_form_outcome    returns decided, assumed (names) or passed
      probe_core_member_declaration  folds the forms, carrying the assumed names
    core-member-target-records.rs
      list_core_trait_names, check_prelude_only_scope   U2
      find_core_member_target      writes U3 into the lookup record
L2, L3  nothing added
```

### Verification matrix

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| U1, U3 | test_scope_trait_blocks_target | unit | a prelude trait before the slice | `[T]::len`, names the trait |
| U4 | same | unit | prelude trait, no later member | scope_trait_may_fit |
| U5 | same | unit | prelude trait and an unread other trait | scope_trait_may_fit |
| U1 to U3 | test_parameter_receiver_core_member (length) | unit, fixture | an array parameter | `[T]::len` with the assumption |
| U2, U5 | same (imported) | unit, fixture | the trait's name written in the workspace | scope_trait_may_fit |
| U6 | same (direct) | unit, fixture | a slice parameter | no assumption |
| U1 to U6 | test_itoa_core_member_targets | real source | 5 `[T]::len` equal their frozen rows; no other row carries the assumption | wrong 0 |

## 12. A method name that one trait of `core` declares (spec first, added 2026-11-16)

### Rust work mode

Delivery Mode and Reliability Mode. The risk is a second declaration of the name that the
run did not see.

### Measured before writing (commit `3a4827d01`, scope `declarations`)

```text
 13  try_into   receivers: a literal (8), `(&mut buf[offset..])` (5)   receiver_type_not_determined
  1  rev        receiver `(1..4)`                                      receiver_type_not_determined
 stored declarations of core (6,528 rows):
   try_into   1 trait declaration (TryInto, `self` by value), 1 impl of that trait
   rev        1 trait declaration (Iterator)
```

Expected (frozen rows): `TryInto::try_into` and `Iterator::rev`, owner kind
`trait_declaration_impl_not_named`, no assumption. The receiver's type is written nowhere
in itoa for these calls, and the label does not depend on it.

### The Rust rule this follows

A method call that compiles resolves to a method of that name. The methods that exist for
a `no_std` unit without dependencies are those of `core` and of the workspace. When
exactly one method of the name is declared in both, the call is a call of it, whatever the
receiver's type. The row names the trait's declaration, not the impl that runs.

### Executable requirements: REQ-RUST-061.0

```text
M1  WHEN no reason withholds every call of the run (REQ-RUST-056.0 P1: no_std, no
    dependency, no alloc or std, no glob import, the prelude read)
    AND exactly one trait declaration of core declares the method name, with a `self`
    parameter, and that trait is in scope (through the prelude, or its name is written in
    the workspace)
    AND no inherent impl of core declares the name
    AND no macro of core may declare an inherent member or a trait member of the name
    AND every impl member of the name in core is in an impl of that trait
    AND no item of the workspace has the name
    THEN the call SHALL be named `<Trait>::<method>`, crate core, with the file and
    offsets of the trait's declaration of the method.
M2  The row SHALL say that it names a trait's declaration and no impl:
    dispatch_kind core_trait_declared_member, target_owner_kind_value trait_declaration,
    receiver_type_proof_kind method_name_sole_declaration.
M3  The row SHALL carry the assumption source_compiles_method_resolves.
M4  WHEN any condition of M1 does not hold THEN this rule SHALL name nothing, and the
    lookup by receiver type applies as before.
```

### Design (L1 / L2 / L3)

```text
L1  core-member-target-records.rs
      CoreMemberLookupIndex.workspace_item_leaf_names   the last name of every graph node
      find_sole_trait_member                            M1 to M3
      find_core_member_target                           asks it first (M4)
    original-call-resolution-records.rs                 dispatch_kind (M2)
L2, L3  nothing added
```

### Verification matrix

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| M1 to M3 | test_sole_trait_method_name | unit, fixture | a literal receiver | `TryInto::try_into`, offsets of the declaration |
| M1, M4 | same | unit, fixture | a workspace fn of the name; a name an inherent impl also declares; a trait out of scope; a unit that is not no_std | no target |
| M1 | same | unit, fixture | the trait's name written in the workspace | `Lone::lone_call` |
| M1 to M4 | test_itoa_core_member_targets | real source | 13 `TryInto::try_into`, 1 `Iterator::rev` equal their frozen rows | wrong 0 |

## 13. A receiver that is itself a method call (spec first, added 2026-11-16)

### Rust work mode

Delivery Mode and Reliability Mode. This is step 8.4 of section 4: the receiver's type is
the return type that `core` declares for the inner call. The risk is reading a return type
whose names are generic parameters as if they were types of `core`.

### Measured before writing (commit `4ea40b960`, scope `declarations`)

```text
  8  expect   receiver `LITERAL.try_into()`               receiver_type_not_determined
  5  unwrap   receiver `(&mut buf[offset..]).try_into()`  receiver_type_not_determined
  1  cast     receiver `self.bytes.as_mut_ptr()`          receiver_type_not_determined
 stored declarations of core:
   TryInto::try_into   trait declaration, returns `Result<T, Self::Error>`
   expect, unwrap      2 inherent rows each: `Option<T>`, `Result<T, E>`; no bound on the impl
   [T]::as_mut_ptr     returns `*mut T`
   cast                4 inherent rows; `impl<T: PointeeSized> *mut T` has a bound on the impl
```

Expected (frozen rows): `Result<T, E>::expect` (`core/src/result.rs`), `Result<T, E>::unwrap`,
`*mut T::cast` (`core/src/ptr/mut_ptr.rs`); owner kind `inherent_impl`; no assumption.

### What is missing today

- The call's stored evidence says nothing about a receiver that is a call.
- `T` in `Result<T, Self::Error>` is a generic parameter of the trait `TryInto<T>`. A trait
  declaration's row stores no generic names, so `T` reads as an unproven type name.
- `Self::Error` is a path through `Self`; the shape model does not read it.
- An impl's own `T` is implicitly `Sized`, so the probe does not let it fit a type it cannot
  read. Here both arguments are `Sized` by declaration: a generic parameter of the trait
  without `?Sized`, and an associated type the trait declares without `?Sized`.

### Executable requirements: REQ-RUST-062.0

```text
C1  The stored evidence of a method call whose receiver is a method call (through
    parentheses only) SHALL hold the evidence of that inner call: its method name and its
    own receiver evidence, read by the same rules.
C2  WHEN the inner call is named by a declaration of core (REQ-RUST-056.0 to 061.0)
    AND that declaration's row stores a return type
    THEN the receiver's type SHALL be that return type, read by C3 to C5. Otherwise the
    lookup does not apply, or keeps the inner call's withheld reason.
C3  The row of a trait declaration's member SHALL store the names of the trait's generic
    parameters, the names of the associated types the trait declares, and for each
    whether a bound that takes the implicit `Sized` away is written for it (`?Sized`,
    `PointeeSized`, `MetaSized`; the text of C4 says `?Sized` for all three). The row of
    any member SHALL store the names of the member's own generic parameters.
C4  In the return type, a name that is a generic parameter of the declaring trait or
    impl, and a path `Self::NAME` where NAME is an associated type the trait declares,
    SHALL be read as "a type not named, proven Sized" when no `?Sized` is written for it
    and it stands as a generic argument of a named type. Anywhere else, or with `?Sized`,
    or a generic parameter of the member itself, it SHALL be read as a type the model
    does not read.
C5  "A type not named, proven Sized" SHALL fit a generic parameter of an impl and
    nothing else.
C6  The probe, the scope rules and the withheld reasons then apply unchanged. The lookup
    record SHALL carry receiver_type_proof_kind call_result_declared_type, the inner
    call's target label, its declaration file and offsets, and the return type text.
C7  `cast` on `*mut T` stays without a target in this step: the impl has a bound
    (`T: PointeeSized`) and nothing stored proves that the bound restricts no type.
```

### Measured after C1 to C6 were built (2026-11-17, working tree after `3d43abb7c`)

The fixture names `Result<T, E>::expect`. The run on pinned itoa names none of the 13:

```text
 13  expect, unwrap   receiver_type_name_unproven
 stored declarations of core named `Result`:
   enum         core/src/result.rs
   type_alias   core/src/fmt/mod.rs
   type_alias   core/src/io/error.rs
```

The probe reads a type name as one type only when `core` declares it once. `Result` is
declared three times, so the name alone does not say which type `try_into` returns. Rust
decides it by the scope of the file that writes the name. Read by hand in `core` of rustc
1.99.1: `convert/mod.rs` binds no `Result` at file level, has no glob import and no macro
call there; `lib.rs` line 198 to 200 is `#[prelude_import] use prelude::rust_2024::*;`;
the only `Result` of the prelude files is `crate::result::Result` (`prelude/v1.rs` line
50); `lib.rs` line 314 is `pub mod result;`; `result.rs` line 558 declares the enum.

### Executable requirements: REQ-RUST-062.0, continued (added 2026-11-17, spec first)

```text
C8   A type name in the return type that core gives to exactly one struct, enum or union
     and to no alias or rename is read as before. Any other type name that is not a
     builtin type SHALL be read as one type only when C10 and C11 prove which declaration
     it is. Otherwise the lookup is withheld: receiver_type_name_unproven.
C9   Stored rows (file_scope_name_binding). For every direct child of a core source file
     that may bind a name in the type namespace, the run SHALL store one row with its
     form and the attribute texts of its item:
       item          struct, enum, union, trait, trait alias, type alias (its kind stored)
       module        inline or file
       extern_crate  the name it binds
       use_leaf      the name it binds and the full path it names (`self` binds the name
                     before it; `as _` binds nothing and gives no row)
       use_glob      the path it globs, no name
       unread_form   a macro call or an extern block: its names are not read, no name
     A member row SHALL store whether its trait or impl is a direct child of its file.
     A prelude row (REQ-RUST-054.0 D10) SHALL store the name it binds and its full path.
C10  The name N written in the return type or impl header of a member in file F stands
     for the declaration D when the member's trait or impl is a direct child of F, N is
     not a generic parameter of that owner or of the member, and one of:
     (a) F has exactly one row named N, an item of kind struct, enum or union: D is it;
     (b) F has no row named N, no use_glob and no unread_form row; no extern crate of
         core binds N; and the prelude gives N the declaration D (C11).
     A row used as a proof step SHALL have no attribute text that contains `cfg`.
C11  The prelude gives N the declaration D when all of these hold:
     - exactly one use row of core has an attribute text containing `prelude_import`; it
       is a use_glob in core/src/lib.rs and its path starts with `prelude::`;
     - lib.rs has exactly one row named `prelude`: a file module with no `#[path`;
     - every prelude row that binds N names the same path, and it is `crate::M::N`;
     - lib.rs has exactly one row named M: a file module with no `#[path`;
     - exactly one of core/src/M.rs and core/src/M/mod.rs has stored rows, and in that
       file N stands for D by C10 (a).
C12  Every inherent impl and trait impl row of a member with the probed method name,
     whose self type text writes N as a name, SHALL have N proven by C10 to stand for the
     same D in its own file. When one does not, the lookup is withheld:
     receiver_type_name_unproven.
C13  The lookup record SHALL carry, for each name read by C10: the name, the file that
     writes it, the proof kind (file_scope_declaration | prelude_import), the file and
     offsets of D, and for prelude_import the files and offsets of the prelude import and
     of the prelude row.
```

What C10 and C11 rest on: `core` is the compiled crate of the pinned toolchain, so a name
it writes resolves, and two items of one name in one module would not compile. An item
written in a file wins over a glob import. A file module's scope holds its own items and
imports, then the extern crates, then the prelude; it does not hold the names of its
parent module.

### Design (L1 / L2 / L3)

```text
L1  sysroot-member-declaration-records.rs   C3: three new fields on member rows
    original-call-resolution-records.rs     C1: extract_method_receiver_syntax
    receiver-type-probe-records.rs          C4, C5: a new shape variant and its unification
    core-member-target-records.rs           C2, C6: a third kind of receiver base
    sysroot-member-declaration-records.rs   C9: file_scope_name_binding rows, one flag
    core-file-scope-name-records.rs (new)   C10, C11: pure lookups over the stored rows
    core-member-target-records.rs           C8, C12, C13: names proven before the probe
    receiver-type-probe-records.rs          C8: the probe accepts a name proven by C10
L2, L3  nothing added
```

### Verification matrix

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| C3 | sysroot declaration row test | unit | a trait with a generic, an associated type and `?Sized` | the three fields |
| C4, C5 | probe test | unit | `Result<T, Self::Error>` with T and Error proven Sized; with `?Sized` | `Result<T, E>::expect`; inherent_member_unproven |
| C1, C2, C6 | fixture | unit | `999.try_into().expect("x")`; inner call not named | `Result<T, E>::expect`; no target |
| C9 | sysroot declaration row test | unit | one file with each form; `self` leaf; `as _` | the rows |
| C10, C11 | file scope name test | unit | declared in the file; from the prelude; a glob, a macro call, a second binding, a `cfg`, a `#[path]` | D; not proven |
| C8, C12, C13 | fixture | unit | a second and third `Result` in the fixture `core`; an impl on `Result` in the alias file | `Result<T, E>::expect`; receiver_type_name_unproven |
| C7 | test_itoa_core_member_targets | real source | `cast` stays without a target | reason stored |
| C1 to C6 | test_itoa_core_member_targets | real source | 8 expect and 5 unwrap equal their frozen rows | wrong 0 |
