# Second opinion: GLM-5.3 (pi, max reasoning, read-only) on part 5d step 9, the impl member when the receiver is a cast

Date: 2026-11-15 (machine clock). Reviewed: commit `6e78660c0` (rule G13 as first built),
before the review-driven changes listed at the end. Run time 324 s; the error file was empty.

## Prompt (verbatim)

````text
You are a read-only reviewer of Rust changes in this repository (branch research/tiny-rust-code-mapper, HEAD commit 6e78660c0, not pushed). Do not edit files. Review rule G13 as built, for correctness against Rust's method lookup and against the repository rule "never guess a target; keep one row per site with a reason; give a coding agent the most source-proven information".

Read these files:
- crates/parseltongue-syntax-span-extractor/src/original-call-resolution-records.rs : functions read_cast_receiver_type, read_simple_type_path, check_cast_receiver_type, extract_method_receiver_syntax, resolve_cast_receiver_target, find_cast_receiver_member, build_original_call_edge
- crates/parseltongue-syntax-span-extractor/src/type-relative-member-records.rs : find_named_impl_member, find_type_member_target
- crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs : resolve_type, read_type_path_records
- crates/parseltongue-syntax-span-extractor/src/syntax-namespace-input-extraction.rs : the block that pushes NamePathRequestRow (search for body_cast_receiver_type)
- crates/parseltongue-syntax-span-extractor/src/cast-receiver-member-tests.rs (new fixture tests)
- crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs : test_cast_receiver_call_targets
- docs/phase-20261113/SPEC-itoa-full-item-graph/part-5-macro-generated-items.md section 10 (rules G9..G12, section 10.13, open questions Q31..Q34)
- crates/parseltongue-cli-dispatcher-binary/tests/pinned-itoa-source-tree/src/lib.rs (macro impl_Integer_size, trait private::Sealed)

Rule G13 as built. A method call whose receiver, through any parentheses, is a cast `(x as T).name(..)` gets a target when all of these hold:
1. T is one path without generic arguments or a type anchor. Its whole path is sent to name lookup as a Type request in the call's lexical scope (context kind body_cast_receiver_type), so a shadowed builtin name is handled by name lookup.
2. The stored lookup of T resolves to exactly one struct, enum, union or builtin (resolve_type; a type alias, type parameter or Self gives no target).
3. Among the stored impl headers whose self type is that type, or whose self type is not stored (generic, blanket, reference types), exactly one has a member named `name` (raw identifier prefix ignored).
4. That member is a function in a trait impl whose trait is stored, with no generic parameters and no negation, and it takes `self` or `&self`/`&mut self` (not a typed self, not no self).
5. The trait of that impl is the same stored trait as the trait of the impl the call is written in (the impl whose member list holds the call's owner item), that enclosing impl is not inside a function body, and the call is written directly in the member's body (its nearest enclosing item is a function that is not inside any block). This is the proof that the trait is in scope: rustc and rust-analyzer add the trait of the enclosing impl to the traits in scope.
6. The member is a graph node.
The row gets dispatch_kind cast_receiver_impl_method and cast_receiver_lookup_evidence: the cast's type text and identity, the candidates, the outcome, the chosen member, the enclosing impl, trait_scope_proof_kind, and sources_not_read (for a builtin: its inherent items, and trait impls for it in crates whose source was not read). When a condition fails the row has no target and unresolved_detail_kind says which: cast_type_not_stored, cast_member_unproven, cast_member_not_method, cast_trait_scope_unproven, cast_member_not_node.

Known and accepted: for a builtin such as u64, an inherent method of that name in core would win over the trait method; core is not read, so the row records "inherent items of builtin type u64" under sources_not_read and still gives the target. The same was accepted for `u8::MAX_STR_LEN` in rule G5.

Measured on pinned itoa: 2 rows, `(self as u64).write(buf)` in `<usize as private::Sealed>::write` -> `<u64 as private::Sealed>::write`, and the i64 one. resolved_calls 41 -> 43 of 197; nodes 103, edges 502 unchanged.

Answer these, each with a concrete Rust counter-example when you say something is wrong, and cite file and function:
A. Is there compiling Rust code for which conditions 1-6 hold and rustc dispatches the call to a different function than the chosen member (leaving aside methods in crates whose source is not read)? Consider: autoref and autoderef order, by-value vs by-reference candidates, impls for &T or &mut T, blanket impls, inherent impls of a struct in the same crate (inherent wins over trait), a second trait in scope with the same method name, supertraits, trait objects, casts that change the type through coercion, `as _`, macro hygiene of the cast type inside expanded text, shadowed type names, nested items and closures.
B. Condition 5: is it correct that the trait of an enclosing impl is in scope for method calls in that impl's method bodies, including inside closures? Is it also in scope inside a nested fn item or a const item written in the body (I exclude those)?
C. Is the accepted builtin caveat (unread inherent methods in core) consistent with "never guess a target"? Would you rather withhold the target, or keep it with the caveat recorded? Say which gives a coding agent more correct information.
D. Is anything in the evidence missing that an agent would need to trust or distrust the row?
E. Are the fixture tests in cast-receiver-member-tests.rs missing a case that would let a wrong implementation pass? Name the case.
F. The remaining 138 method calls on itoa all target core (MaybeUninit::write, get_unchecked, try_into, ilog10, expect, len, unwrap, unsigned_abs, as_mut_ptr, cast, rev). Core source is not read in this phase and the decided plan is "named core targets" (a row names the external item it most likely reaches only when provable from source text). For which of these receivers is the receiver type provable from itoa's source text alone, without reading core? Be specific per method.
Keep the answer under 900 words.
````

## Reply (verbatim)

````text
I've read the code (`find_cast_receiver_member`, `resolve_cast_receiver_target` in original-call-resolution-records.rs; `find_type_member_target` in type-relative-member-records.rs; `resolve_type` in impl-header-source-records.rs; the fixtures; the itoa test; spec §10.13; the pinned itoa source). Review answers:

## A — No counter-example within read source

Every rustc-priority alternative to the chosen member either becomes a stored candidate (→ `several_stored_impl_members`, no target) or lives in an unread crate (the accepted caveat):

- **Inherent method on T**: inherent impls of T can only be in T's defining crate, hence read; they are stored headers with unresolved trait (`cast-receiver-member-tests.rs`, `inherent` case → unproven; together with the trait impl → several).
- **Second in-scope trait with the same name**: its impl for T in this crate is a candidate → several. Supertrait case: if `Sealed: Other`, an `impl Other for u64 { fn write }` is stored → candidate → blocked; a chosen member in `impl Other for u64` while the call sits in `impl Sealed` fails `trait_of` equality → `cast_trait_scope_unproven` — correct, since Other would also be in scope via the supertrait.
- **`impl` for `&u64`/`&mut u64`, blanket impls**: self type is not a simple path → unresolved (`resolve_type`), so they count as candidates → several (fixture `blanket` case).
- **Autoderef**: step T already yields the chosen member (non-generic impl of exactly T; `(x as T)` is a value of exactly T), so deref steps are never probed. Inherent of a deref target loses the same way.
- **dyn, `as _`, generics, anchors**: excluded syntactically (`read_simple_type_path`); `Self`/aliases/params give no stored type.
- **Shadowed `u64`**: lookup runs at the call's scope (`check_cast_receiver_type` → `body_cast_receiver_type` request), so a local struct shadow wins there exactly as in rustc. Macro hygiene: the path request carries the site's scope and provenance chain; a mis-resolved name surfaces as `cast_type_not_stored`, not a guessed target.

The only silent mismatch class: a foreign trait in scope (prelude, `use ... as _`) whose unread impl for a builtin T is applicable at an *earlier* autoref step (e.g. takes `self` by value while the chosen member takes `&self`). That is precisely the `sources_not_read` class ("trait impls for u64 in crates whose source was not read"); for a non-builtin T a foreign non-blanket impl is impossible (orphan rule), and blanket is recorded.

## B — Condition 5 scope claims are right

`(self as $primitive).write(buf)` inside `impl private::Sealed for usize` compiles in pinned itoa with no import of `Sealed`, proving the impl's trait is in scope for arbitrary receivers in member bodies; closures inherit that lexical scope. Nested `fn`/`const` items in a body get module scope, so the trait is *not* in scope there — excluding them (`receiver_member_body_flag` in `extract_method_receiver_syntax`) is correct, not merely conservative:

```rust
mod m { pub trait T { fn g(&self) {} } }
struct S;
impl m::T for S { fn f(&self) { fn h(x: &S) { x.g(); } h(self) } } // E0599: T not in scope
```

Note: the same check also excludes an associated `const` written directly in the impl, where the trait *is* in scope — a conservative miss, not a wrong answer.

## C — Keep the target with the caveat

This is Rule 1's "Proven, gap" state exactly: one target found, the edge names the missing source. `sources_not_read` names the only sources that could overturn it. Withholding would delete a true edge (u64 has no inherent `write`) and give an agent strictly less information, against Rule 6. Keep, as built.

## D — Evidence is sufficient

Nothing essential is missing. Two fields are join-only: the enclosing impl's trait identity and the chosen impl's self-type text appear only as occurrence keys / prefix text; inlining them would let an agent judge the row without opening the impl table.

## E — Missing fixture cases

1. **`cast_member_not_node`**: the fixtures never drive `resolve_cast_receiver_target`'s endpoints check (G12 had the "member not a graph node" run; G13 lacks it). An implementation that skips the check passes all tests, including the itoa test — both real targets are nodes.
2. A single candidate with `impl_negative_implementation_flag == true` (negation is asserted nowhere in the G13 fixture).
3. Call-side `r#write` (only the member side is tested).

## F — Receiver types provable from itoa alone

**Provable**: `MaybeUninit::write` — `buf[offset]` on `&mut [MaybeUninit<u8>]` / `Self::Buffer = [MaybeUninit<u8>; N]`, element type from itoa's own signatures and associated-type alias; `get_unchecked` — same receivers plus `DECIMAL_PAIRS.0: [u8; 200]`; `len` — `buf.len()` on the array alias, and `string.len()` where `string: &str` is the return type of itoa's own `Sealed::write`; `unsigned_abs` — `self` of `impl private::Sealed for $Signed` = iN; `as_mut_ptr` — `self.bytes` field of itoa's `Buffer`.

**Not provable**: `try_into` — receivers are the untyped literals `999` / `1_00_00`; their types come from inference against `remain: Self` and `let scale: Self` via core's `PartialOrd`/`TryInto`; `expect`/`unwrap` — receiver is the `Result` of core's `TryInto`; `ilog10` — receiver `$Unsigned::MAX` is a const declared in core's impl; only the qualifier uN is in itoa; `cast` — receiver is `as_mut_ptr`'s return type, declared in core; `rev` — receiver `1..4` is core's `Range`, method needs `Iterator`.

(~870 words.)
````

## What I checked, and what I did with each point

```text
 point                                        checked how                                 result
 A  no wrong-target program in read source    the cases it lists against the fixture      agrees with the fixture cases: inherent impl,
                                              test test_cast_receiver_member_rules        second impl, blanket impl each give no member
 B  the enclosing impl's trait is in scope    pinned itoa compiles (self as u64).write    taken as the proof kind enclosing_impl_trait
    in member bodies and closures             with no import of private::Sealed
 B  a nested fn has module scope (its         NOT compiled here                           not relied on: the rule excludes nested
    example, E0599)                                                                       fn and const either way
 B  an associated const in the impl is        read extract_method_receiver_syntax:        true; recorded as open question Q35
    excluded although the trait is in scope   the nearest item must be a function
 C  keep the target for a builtin, with the   -                                           kept as built; decision recorded in the PRD
    unread core sources on the row
 D  inline the enclosing impl's trait and     -                                           adopted: enclosing_impl_trait_identity,
    the type                                                                              enclosing_impl_trait_source_text,
                                                                                          enclosing_impl_self_type_identity,
                                                                                          enclosing_impl_self_type_source_text
 E1 no fixture drives the graph-node check    read the fixtures: none removed the         adopted: test_cast_receiver_call_member;
                                              member's node                               a mutation that skips the check fails it
 E2 no negative-impl case                     read the fixture                            adopted in test_cast_receiver_member_rules
 E3 no call-side r#name case                  read the fixture                            adopted in test_cast_receiver_member_rules
 F  receiver types readable from itoa alone   NOT checked row by row                      recorded as open question Q37, as the
                                                                                          review's count, not mine
```

Limit on E1: the "check skipped" mutation was run only after the new test existed, where
it fails `test_cast_receiver_call_member`. That the older tests would have passed follows
from reading them, not from a run.

## Changes made after the review

Commit `a9e8cdbf1`: the four `enclosing_impl_*` fields on `cast_receiver_lookup_evidence`;
`test_cast_receiver_call_member`; the negative-impl and `r#name` cases. Measured again
after it: 103 nodes, 502 edges, 43 of 197 calls with a target, unchanged.
