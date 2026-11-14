# itoa HLD-current — rubber-duck pass and haze ratings

Date: 2026-11-13. Reviews `itoa-hld-build-architecture.md` and `PRD-phase-20261113-itoa-hld-current.md`
by walking each stage against the code and the pinned itoa source. Nothing was run.

## 1. What the walk-through found wrong or missing

```text
 E1  Wrong example in my own documents.
     itoa's `default` and `clone` bodies call `Buffer::new()`, not `Self::new()`.
     So slice 1 does not need a `Self` rule. It needs a type-relative lookup:
     resolve `Buffer` to the struct, then find `new` among Buffer's inherent impls.
     Corrected in the three documents.

 E2  Type-relative paths are a stage of their own, and nothing does them.
     The name-resolution SQL handles module-scope paths. The SPEC defers
     type-relative and associated paths. itoa uses five shapes:
       Buffer::new                             struct, inherent impl
       i128::MAX_STR_LEN                       primitive, trait impl
       Self::MAX_STR_LEN                       `Self` inside an impl
       I::MAX_STR_LEN                          generic parameter, through the bound I: Integer
       <$primitive as Integer>::MAX_STR_LEN    fully qualified, inside a macro template

 E3  No relation kind fits a reference to an associated const.
     The relation kinds are calls, uses_type, implements, reexports, contains, derives.
     `Buffer::new` reads `i128::MAX_STR_LEN`, and the 12 MAX_STR_LEN impls are HLD nodes.
     The architecture does not say which edge, if any, that produces.

 E4  "Without sysroot, all local edges still work" was false.
     The signed `write` calls `Unsigned::fmt(self.unsigned_abs(), ..)`. Choosing
     `<u8 as Unsigned>::fmt` needs the return type of `i8::unsigned_abs`, a core method.
     So some itoa-to-itoa edges depend on core. Corrected in the PRD (AD-1 option B).

 E5  Two call shapes with a concrete receiver are not covered.
     `Unsigned::fmt(self, buf)` inside `impl Sealed for u8` must reach `<u8 as Unsigned>::fmt`,
     not the bodyless trait method. That needs the argument's type and an impl pick.
     `(self as $primitive).write(buf)` in the isize/usize impls is a method call on a
     cast to a primitive; it must reach `<i64 as Sealed>::write`. That needs the cast's
     type and a trait-method lookup on a primitive.
     `Sealed` is not imported there. A rustc check on a four-line copy of the pattern
     showed the call compiles inside `impl private::Sealed for ..` and fails outside it:
     the trait being implemented is in scope in its own impl. The lookup must copy that rule.
     The architecture only described `recv.name(..)` on `self` and on a generic receiver.

 E6  Macro-generated items: where is their evidence?
     Expanded impls are parsed from a virtual source file, so their spans point into
     generated text. The HLD needs them attributed to src/lib.rs for evidence ranges
     and file roll-ups. Token-origin rows exist; the mapping rule is not written.

 E7  Body-position macros are not connected, but itoa barely depends on them.
     itoa has 15 assert expansions (three `assert!` lines in a macro template) and one
     `debug_assert!`. Their arguments name only primitive consts and a local variable,
     so no itoa-to-itoa edge comes from them. They are defined in core, and body macros
     are not yet joined to body sites (open task from the earlier phase). For itoa
     they only need to be recorded as unresolved sites, not expanded.

 E8  Visibility is not "SQL over scopes only".
     A trait impl member takes its visibility from the trait (`Sealed` public,
     `Unsigned` private). So the visibility stage reads impl headers too.

 E9  The assembler has unstated outputs.
     Node rows need a qualified path text and a signature text
     (for example `<u8 as private::Sealed>::write`), a crate-root node and `contains` edges.
     The architecture file does not say who renders or emits them.
```

## 2. Ratings

Scale: 1 = clear, could start now. 3 = approach known, details open.
5 = not understood well enough to start; needs a decision or an experiment.

```text
 haze  task                                               what is still unclear
 ----  -------------------------------------------------  ------------------------------------------------
  1    run today's pipeline on itoa, unchanged            nothing; it is a measurement
  1    fix the two failing publication fixtures           nothing; both causes are known
  1    HLD builder (collapse, roll-ups)                   nothing for itoa; passes on fixtures
  2    real-source test harness for slice 1               where the pinned source lives in the test
  2    impl-header rows for hand-written impls            row columns to be fixed in the SPEC
  2    body-position macros (E7)                          only: record them unresolved, add no edges
  3    body paths, first segment (AD-2)                   local shadowing; generic-parameter heads
  3    lowered signature types                            how deep the type tree goes for itoa
  3    uses_type and implements edge rules                which signature positions emit an edge
  3    method call on a generic receiver (i.write)        walking the supertrait Integer: Sealed
  3    effective visibility                               join with impl headers (E8)
  3    assembler into builder records                     path text, signature text, contains (E9)
  4    type-relative lookup (E2)                          five shapes, primitives, bounds; no code today
  4    calls with a concrete receiver (E5)                argument and cast typing; impl selection
  4    macro-generated identity and evidence (E6)         mapping generated spans back to lib.rs
  4    freezing the reference                             waits on E3, E5, E6 being decided
  5    associated const references (E3)                   no relation kind exists; needs a product decision
  5    core handling (AD-1, E4)                           unmeasured; and how exact edges into core must be
```

## 3. What this changes

```text
 - Slice 1 is still the right first slice, but its real content is type-relative lookup (haze 4),
   not a `Self` rule.
 - Two items need your decision before a reference can be frozen: E3 and AD-1.
 - One item needs a measurement before anything else: the unchanged itoa run.
```

## 4. Update after the decisions of 2026-11-13

The ratings above are kept as written. What changed since:

```text
 E3   decided: a `reads` relation exists (SPEC REQ-RUST-030.1). Haze 5 -> 2. Left open: may
      the source be a const initializer or a type, not only a function (itoa PRD OD-8).
 E9   partly decided: file-public-area-nodes and use nodes exist, `contains` and `imports`
      are stored, and collapse never walks them. Left open: one container per node, impl
      members, items inside a dropped private module (itoa PRD OD-7).
 new  the full graph is the target: private items are nodes and private -> private edges are
      rows, so the reference must list them all. "Freezing the reference" stays haze 4 and
      now also waits on OD-7 and OD-8.
 new  the builder needs three changes (two node kinds admitted, two relation kinds, collapse
      skipping contains and imports). Its haze goes from 1 to 2.
 same core handling (AD-1) is still haze 5 and still unmeasured.
```
