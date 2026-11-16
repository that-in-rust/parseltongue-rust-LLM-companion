# Part 8 — method calls into `core`

Status on 2026-11-16: **steps 8.1 to 8.3b-2a built; 8.3b-2b open (one real-source test
fails). The order changed by user decision: the expected target of all 133 calls is written
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
  for that site, keyed by file and byte range. No other site has a row.
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
- T5. Each row SHALL list its assumptions; an empty list means none. A call that a prelude
  trait could also answer SHALL carry `prelude_trait_impl_unrefuted` (decided, Q2). A
  declaration written inside a `macro_rules!` body of `core` SHALL carry
  `core_macro_body_declaration` and name the macro.
- T6. The file SHALL record the compiler version (`rustc -vV`) and one sha256 over the path
  and bytes of every `.rs` file under `library/core/src`, in path order. WHEN a cited
  declaration line does not hold the cited text in those files THEN the data-set test SHALL
  fail.
- T7. The file is frozen like the expected graph: its sha256 is in a freeze record, and a
  change needs a source citation and a new freeze. It is never a production input.
- T8. WHEN a run with scope `declarations` stores a target for one of the 133 sites THEN
  the real-source test SHALL fail unless label and assumptions equal the row. A site the
  run leaves without a target is counted, not failed, until step 8.6.
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
     write the builder and the 133 rows; data-set tests T1 to T6 pass
9.3  the user reads the rows by method group (11 groups) and rules on any I marked open
9.4  freeze (T7)
9.5  test_itoa_core_member_targets compares with the file (T8): STUB, RED, GREEN, REFACTOR
9.6  close step 8.3b-2b against the frozen rows, then 8.3b-3, 8.4, 8.5 in the order the
     missing count falls fastest
```

### Quality gate results

Not run. This section is a specification; 9.2 runs the data-set tests.

### Open questions

- Q1. **Trait methods** (`try_into` 13, `rev` 1): the label `TryInto::try_into` names the
  trait's declaration, as the PRD does for itoa's own traits. The impl that runs (`impl<T,
  U> TryInto<U> for T where U: TryFrom<T>`) is not named. Proposed: keep the trait label
  and cite the blanket impl in the proof.
- Q2. **`ilog10`** (10): `core` writes it inside `uint_impl!` and `int_impl!`, called from
  `impl u64 { .. }`. Proposed label: the builtin type, as the PRD does for `u8::MAX`
  (`u64::ilog10`), with `core_macro_body_declaration`.
- Q3. **`expect` and `unwrap`** (13): the receiver is the result of `try_into`, so its type
  comes from a `core` return type (`Result<U, U::Error>`). Proposed label
  `Result<T, E>::expect`, with the return type cited as a proof step.
- Q4. **Which compiler.** No file pins the compiler. T6 records the one read; the SOP run
  must use the same one or the freeze fails.
