# Part 15 — `cast`: a bound on the lang item `pointee_sized` restricts no type

Back to the [index](README.md).

Status on 2026-11-18: **specified, not built.** Picked by the user on 2026-11-17, last
in the order, with the question whether it needs a trait solver.

**It does not need a trait solver.** The run already finds the one declaration of
`cast`, in `impl<T: PointeeSized> *mut T`. It stops for two reasons, found in this
order while building: the impl has a bound, and a bound may exclude the receiver's type
(REQ-RUST-062.0 C7); and the receiver is the result `*mut T` of `[T]::as_mut_ptr`, whose
pointee the run did not read as a sized type (P7). This bound
excludes nothing, and that follows from one rule of the language, not from solving a
trait: `PointeeSized` is the lang item `pointee_sized`, the root of the sized hierarchy,
which every type meets and no code may implement.

```text
the call      src/lib.rs line 107       self.bytes.as_mut_ptr().cast::<u8>()
the impl      core ptr/mut_ptr.rs:8     impl<T: PointeeSized> *mut T {
the method    core ptr/mut_ptr.rs:31    pub const fn cast<U>(self) -> *mut U {
the bound     core marker.rs:181-190    #[lang = "pointee_sized"] ... #[rustc_deny_explicit_impl]
                                        pub trait PointeeSized {
```

**What this part adds on pinned itoa.** The call gets the target `*mut T::cast`, the
target the frozen targets file and the compile check of 2026-11-16 already give. The
expected graph of a `declarations` run gains that node (173 nodes), and no call is left
without a target (197 of 197).

**The limit, stated.** "Every type meets `PointeeSized`" is a rule of the compiler. The
source of `core` shows the attribute that marks the trait, not an impl for every type.
The rule is therefore tied to that attribute as read from the frozen `core` files: if
`core` stops marking the trait, the rule names nothing. `MetaSized` is not covered: an
extern type does not meet it.

## 1. Rust work mode

Spec Mode and Delivery Mode. Component: the declaration reader of `core`
(`sysroot-member-declaration-records.rs`) and the member probe
(`receiver-type-probe-records.rs`), both in `parseltongue-syntax-span-extractor`.

Found with codebase-memory and read in source on 2026-11-18:

- `check_impl_bound_restriction` sets `impl_bound_restriction_flag` on every member row
  of an impl: true for a where clause or any bound other than `?Sized`.
- The member probe marks a candidate "presence unproven" when that flag is not false,
  and such a candidate gives no target.
- `build_type_name_row` stores a row for each struct, enum, union, alias and rename of
  the read files. A trait has no row, so its attributes are not stored.

## 2. Executable requirements: REQ-RUST-073.0

```text
P1  WHEN a trait is declared in a read file of core THEN a declaration row SHALL store
    its name, its attributes and whether it is inside a function body; a trait inside a
    function body SHALL never satisfy P3

P2  WHEN an impl of core has members THEN each member row SHALL store the text of every
    bound on the impl's type parameters and whether the impl has a where clause

P3  WHEN a bound is a single name N, exactly one stored trait row is named N, that row
    carries the attribute #[lang = "pointee_sized"], no stored type-name row is named N,
    and N is not a generic parameter of the impl THEN the bound SHALL restrict no type

P4  WHEN an impl has no where clause and each of its bounds is ?Sized or restricts no type
    by P3 THEN its members SHALL be candidates like those of an impl without a bound

P5  WHEN any bound of an impl is anything else, or the impl has a where clause THEN its
    members SHALL stay unproven and the call SHALL keep the reason TraitSolverNeeded

P6  WHEN two stored trait rows are named N, or a type-name row is named N THEN P3 SHALL
    not hold for N

P7  WHEN the return type of a member of core is a raw pointer or a reference, and its
    pointee is written as a generic parameter of the member's owner that is proven Sized
    (REQ-RUST-062.0 C4) THEN the pointee SHALL be a type proven Sized, as a direct generic
    argument of a returned type already is

A1  WHEN the mapper runs on pinned itoa with scope declarations THEN the call `cast` at
    line 107 SHALL have the target *mut T::cast, no call SHALL be without a target, and
    the stored graph SHALL equal the re-frozen expected graph
```

## 3. Design (L1 / L2 / L3)

- **L2, the declaration reader.** A new row kind `trait_name_declaration` beside
  `type_name_declaration`, built the same way, with the trait's attributes. Member rows
  of an impl gain `impl_bound_text_values` and `impl_where_clause_flag`. The existing
  flag keeps its meaning.
- **L2, the return type shape.** `parse_return_type_shape` gives the pointee of a
  returned raw pointer or reference the shape "proven Sized" when it is written as a
  sized generic parameter (P7).
- **L2, the member probe.** One pure function decides P3 to P6 from a member row and the
  declaration rows. "Presence unproven" becomes: conditional, or the flag is not false
  and that function does not discharge it.
- **L1, L3.** Nothing changes: no new type, table, stage or error.
- **Expected data.** `*mut T::cast` leaves the list of unmerged labels of the
  declarations-graph builder. Both freeze records are written again.

## 4. Verification matrix

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| P1 P2 | `test_trait_and_bound_rows` | unit, extractor | rows of a fixture: a lang trait, a plain trait, a body-local trait; bound texts and where flag of three impls | 0 wrong |
| P3 P4 P5 P6 | `test_pointee_sized_bound_rules` | unit, extractor | discharged: the lang bound alone, with `?Sized`; not discharged: another trait, two bounds one of them plain, a where clause, the name declared twice, the name also a type, the name a generic parameter, the attribute absent | 0 wrong |
| P7 | `test_pointer_return_sized_pointee` | unit, extractor | `*mut T`, `*const T`, `&T`, `&mut Self::Item` with a sized name give a sized pointee; an unproven name and `*const [T]` do not; `cast` on the result reaches `*mut T` | 0 wrong |
| A1 | `test_itoa_core_member_targets` | real source | 133 of 133 calls into `core` equal their frozen rows; 0 without a target | 0 wrong |
| A1 | `test_itoa_declarations_graph_equality` | real source | stored graph equals the re-frozen expected graph | 0 differing rows |

## 5. Implementation plan

1. Expected graph: merge `*mut T::cast`; re-freeze; expected-data tests.
2. See `test_itoa_declarations_graph_equality` fail on that one row.
3. Declaration rows (P1, P2), unit test first.
4. The rule (P3 to P6), unit test first; the real-source tests pass.
5. Gates, a timed run, the documents and the log.

## 6. Quality gate results

Not run: nothing is built.

## 7. Open questions

- **Other bounds.** Every other bound on an impl of `core` still needs a trait solver.
  None is met on pinned itoa after this part.
- **The graph's `incomplete` mark.** With no call left without a target, whether the run
  marks the graph complete is read from the code in step 2 and recorded here.
