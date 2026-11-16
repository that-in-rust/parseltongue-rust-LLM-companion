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
