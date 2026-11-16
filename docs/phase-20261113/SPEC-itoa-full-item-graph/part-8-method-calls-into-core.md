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

```text
test_inherent_member_row_fields      D1 D2   crates/parseltongue-syntax-span-extractor/src/sysroot-member-declaration-tests.rs
test_trait_member_row_fields         D1 D2   same file
test_macro_body_member_rows          D3 D5   same file
test_member_rows_offset_order        D4      same file
```
