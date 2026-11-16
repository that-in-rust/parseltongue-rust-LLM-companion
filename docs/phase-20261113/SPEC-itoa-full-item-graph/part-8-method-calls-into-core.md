# Part 8 — method calls into `core`

Status on 2026-11-16: **step 8.1 in progress.** REQ-RUST-055.0 and REQ-RUST-056.0 are
reserved and get their test rows when their steps start.

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

## 5. Open questions

- **Q1. The label of a method on a slice, an array or a raw pointer.** `core` writes
  `impl<T> [T]` and `impl<T> *mut T`. Proposed: the impl's self type as written, without
  its generic names bound: `[T]::len`, `*mut T::cast`.
- **Q2. A prelude trait of `core` that declares the same name** (`ExactSizeIterator::len`
  for `buf.len()` on `&mut [T; N]`). Proposed: name the target and record the assumption
  that the trait is not implemented for the earlier types; the alternative is no target.

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
