# Part 7 — whole-graph acceptance

**State: IN PROGRESS. Measured 2026-11-16: 267 of the draft's 368 expected edges and 72 of
its 73 nodes match what production stores. The expected graph is an unreviewed draft, so no
SOP step has passed.**

## Requirement

- **REQ-RUST-044.0 — whole-graph acceptance.** The exported nodes, edges, evidence ranges and
  unresolved sites equal the frozen expected graph, with no extra, missing or wrong row; the
  output is identical at 1, 4 and 16 workers; source, configuration and executable digests,
  time, peak memory and disk are recorded.

Test to write: `test_itoa_whole_graph_equality`. Procedure:
[SOP](../SOP-itoa-full-item-graph-acceptance.md) S1 to S4.

## 1. First measurement (2026-11-16, commit `a8fd18503`)

Compared by label with a throwaway script (`/home/ubuntu/phase7-work/compare.py`), one row
per distinct source, relation and target. Production rows are from a release run of the
executable `82eab262…` on pinned itoa. This is a diagnostic, not a test.

```text
relation    expected  matched  not matched  production rows the draft lacks
contains          70       70            0  18  (macro calls and definitions)
imports            4        4            0   0
implements        32       29            3   3
uses_type         98       96            2   5
reads             62       32           30  30
calls            102       36           66  30  (23 of them have no target)
generates          0        0            0  45
expands            0        0            0  15
total            368      267          101
```

The 101 unmatched expected rows:

```text
rows  what differs
35    the name of a core target: the draft uses the defining path
      (core::num::<impl u8>::MAX, core::clone::Clone, core::marker::Copy,
      <u128 as core::convert::From<u64>>::from); production stores the written name
      (u8::MAX, Clone, Copy, u128::from), which is the rule decided 2026-11-15
60    method calls into core; the draft names a defining path, production has no target:
      11 MaybeUninit::write, 10 ilog10, 9 TryInto::try_into, 7 get_unchecked, 5 slice len,
      5 Result::unwrap, 5 unsigned_abs, 4 Result::expect, 1 str len, 1 as_mut_ptr,
      1 pointer cast, 1 Iterator::rev
5     Unsigned::fmt(self.unsigned_abs(), ..): the draft says <uN as Unsigned>::fmt,
      production says the trait's declaration (unsigned_abs is in core)
1     Buffer::format --uses_type--> private::Sealed::Buffer, at cast::<I::Buffer>() on
      line 107: a type written in a function body. Production makes no uses_type row from
      a function body. A production miss.
```

Nodes: the draft's crate-root node `itoa` has no production node. Production has 73 nodes
the draft lacks: 42 external names, 15 macro calls, 13 builtin types, 3 macro definitions.

## 2. Steps

```text
7.1  the comparison test: test_itoa_whole_graph_equality reads the dataset files as test
     input, maps production labels to dataset labels, and reports every differing row
7.2  re-cut the dataset from the pinned source to the decided rules: written names for
     targets outside itoa, macro definition and macro call nodes, generates and expands
     rows, builtin nodes, one row per site, method calls without a provable target as
     expected unresolved rows
7.3  production misses the comparison finds: uses_type from function bodies; the
     crate-root node
7.4  method calls whose receiver type itoa's own source proves
7.5  freeze the dataset digests; SOP S2 (process audit), S3 (equality), S4 (1, 4, 16 workers)
```

The dataset is never edited to match production: each change cites the source line.

## 3. Open questions

- **Q1 (asked 2026-11-16, not answered).** The SOP says a run marked incomplete cannot
  pass, and also that a site whose target source cannot prove matches an expected
  unresolved row. With method calls into unread `core` the graph stays incomplete, so the
  two rules cannot both hold.
- **Q2 (decided 2026-11-16 from PRD graph rule 1).** The crate root is a node: every module
  is. It has no edge until OD-7 (file module and file area) is decided.
- **Q3 (decided 2026-11-16 from the maximum-information rule).** Every type written in
  an expression is a `uses_type` site. Rule: REQ-RUST-018.3 R10 below.
- **Q4 (decided 2026-11-16 from "a missed target is better than a wrong one").**
  `string.len()` at `src/lib.rs:109` is an expected row without a target; the draft's
  `str::len` is withdrawn. Source proves the receiver is a `&str` (the declared return
  type of `private::Sealed::write`). It does not prove where `len` is declared: a method
  called on a `&str` value is either a `&self` method of `str` or a by-value method
  declared for `&str`, as `clone` and `into` are. A rule that named `str::NAME` would
  name `str::clone` wrongly; telling the two apart needs `core`. `buf.len()` on a
  `&[T]` parameter was already expected without a target for the same reason. Not built:
  the row does not record the proven receiver type (backlog).

## 5. REQ-RUST-043.1 — the crate-root node (step 7.3c, added 2026-11-16)

- **C1.** WHEN a mapped unit's root file is read, THEN the graph SHALL hold one node for
  the unit's root module. Its identity is the unit; its label is the unit's target name
  (`itoa`). It is of the crate-root category and carries no file, kind, owner or
  visibility: the root module has no declaration of its own.
- **C2.** The node has no edge. The file's top-level items stay contained by the file
  area (REQ-RUST-043.0). Which row links a module to the file area of its body is not
  decided (OD-7); until then neither `itoa` nor `u128_ext` has such a row.
- **C3.** A unit whose root file is not among the files read gets no crate-root node.

## 4. REQ-RUST-018.3 — types written in an expression (step 7.3, added 2026-11-16)

REQ-RUST-018.2 R1 to R9 stay as written, except that R8 no longer excludes macro
expansions (part 5). One rule is added.

- **R10.** WHEN a type is written inside an expression of an item that is a graph node (a
  function body, the initializer of a const or static, an array length), THEN each type
  path in it SHALL be a `uses_type` site of that item under R1 to R5 and R7. The forms are
  the type of an `as` cast, a generic argument (`cast::<T>()`, `Name::<T>::new()`), a
  `let` annotation, a closure's parameter and return types, and the type and the trait of
  `<T as Trait>::NAME`. The row's metadata says `type_use_kind = "expression"`; a
  signature row says `"signature"`.
- **R10a.** WHEN the path starts at a type parameter of the item (`I::Buffer`), THEN the
  target SHALL be given only if name lookup stores exactly one member for it: a member of
  a trait the parameter is bound by in its declaration, or of a supertrait of one.
  Otherwise the row is unresolved with `TraitSolverNeeded`. Not built: a bound written in
  a `where` clause is not walked (the row is unresolved), and the row does not list the
  bounds walked.
- **R10c.** WHEN the type names an item declared in a function body, THEN the row is
  unresolved with `MissingTarget`; such an item is not a node.
- **R10d.** A path in the header of an impl written in a function body is not requested:
  the stored header of such an impl keeps neither its type nor its trait resolved
  (REQ-RUST-053.0 N5 depends on it).
- **R10b.** A struct literal (`Buffer { bytes }`), a pattern and the qualifier of a value
  path (`u128::from`, `MaybeUninit::<u8>::uninit`) are not types in the grammar and give
  no `uses_type` row. The call or read row of the value path already names its target.

## 6. REQ-RUST-044.0 — what the byte range of a site is (step 7.5, added 2026-11-16)

The expected graph states each site's range from the Rust grammar and the pinned bytes.
It is never copied from a production run.

- **E1 `calls`.** The call expression: from the first token of the callee, or of the
  receiver of a method call, to the closing parenthesis of its arguments.
- **E2 `reads`.** The path expression that names the const or static
  (`DECIMAL_PAIRS`, not `DECIMAL_PAIRS.0`).
- **E3 `uses_type`.** The type path with its generic arguments (`MaybeUninit<u8>`), and
  nothing around it: no `->`, `:`, `as`, `&` or brackets. A qualified path
  `<T as Trait>::NAME` is one site for `NAME`; `T` and `Trait` inside it are sites of
  their own.
- **E4 `imports`.** The leaf of the use tree that brings the name in
  (`core::hint`; `self` and `MaybeUninit` in `core::mem::{self, MaybeUninit}`).
- **E5 `generates`, `expands`.** The macro call item with its closing `;`.
- **E6 a site written in a macro definition.** The range is given by its first and its
  last token, each with where it is written: in the macro definition, or in the macro
  call when the token is an argument substituted for a macro parameter.
- **E7 `contains`, `implements`.** Not compared by range: the row is identified by its
  source, relation and target (one row each).

## 7. REQ-RUST-044.1 — which node fields are compared (step 7.6, added 2026-11-16)

- **N1.** A node is compared by its label, its category (crate root, file area, local
  item, external name, builtin type), its item kind, the kind of its owner (crate root,
  module, impl, trait), its file, the macro call that generates it, and whether it is
  declared inside a function body. A node without a value has none on both sides.
- **N2.** Not compared, because production stores no value: effective visibility,
  reachability from outside the crate, and signature text. The expected graph keeps the
  visibility and reachability it reads from the source. The test fails when production
  starts to store one of the three, so that it joins the comparison then.
- **N3.** The owner of a top-level item of the root file is the crate root; of another
  file, the module that file is the body of (`mod u128_ext;`). A file area and the crate
  root have no owner (OD-7).

- **N4 (step 7.9).** A node written directly in a source file is also compared by its byte
  range in that file. It starts at the first comment or attribute line written directly
  above the item with no blank line between, and ends after the item's last token. A node
  written by a macro expansion, a file area, the crate root, a builtin type and a name
  outside itoa have no such range; the expected value is `none`. On pinned itoa 43 nodes
  have a range.
- **N5 (step 7.9).** A name outside itoa is also compared by its kind: `crate_rooted_path`
  (8), `prelude_name` (3) or `builtin_member` (36).
- **N6 (step 7.9).** An expected row with a different number of sites and ranges fails the
  test before any comparison.

## 8. REQ-RUST-044.2 — which record of a site is compared (step 7.8, added 2026-11-16)

- **H1.** A `calls` site is compared by how its target was chosen:
  - `callee_path_function`: the callee is a path that names a function in read source.
  - `type_path_associated_function`: `Type::name(..)` where an impl of the type declares it.
  - `tuple_struct_constructor`: the callee names a tuple struct.
  - `trait_path_impl_method`: `Trait::method(arg, ..)`, and the argument's type picks the impl.
  - `trait_path_method`: `Trait::method(..)`, and the target is the trait's declaration.
  - `generic_bound_trait_method`: a method on a value whose type is a generic parameter;
    the target is the declaration in the bound's trait.
  - `cast_receiver_impl_method`: `(x as T).method(..)`, and the cast names the type.
  - `external_written_path`: the callee is a path into source that was not read.
  - `external_builtin_member`: the callee is a member of a builtin type.
  - `undetermined`: no target. The site also states what stopped the proof; on pinned
    itoa that is `receiver_type_not_determined` for all 133.
- **H2.** A `uses_type` site is compared by whether it is written in a `signature` or in
  an `expression`.
- **H3.** A `reads` site is compared by whether it is written in a body
  (`body_value_path`) or in the arguments of a macro (`macro_argument_value_path`).
- **H4.** Not compared: the lookup evidence a row keeps (candidate impls, the `use` item a
  name went through, the proof of an external name). The feature tests of parts 5 and 6
  check those.
