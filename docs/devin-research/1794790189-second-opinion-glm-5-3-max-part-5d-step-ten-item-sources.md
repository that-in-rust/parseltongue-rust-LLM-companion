# Second opinion: GLM-5.3 (pi, max reasoning, read-only) on part 5d steps 10a, 10b and 10c

Date: 2026-11-16 (machine clock). Reviewed: commit `3e4957cbe` (rules G7 and G14 as built).
Run time 511 s, exit 0. The reviewer could not compile or run code; every answer is from reading.

## Prompt (verbatim)

````text
You are a read-only reviewer of committed Rust changes in this repository (branch research/tiny-rust-code-mapper, HEAD 3e4957cbe). Do not edit files. Review three rules of part 5d of the itoa full item graph: step 10a (consts, statics, structs and type aliases as `reads` sources), step 10c (declared and aliased types as `uses_type` sites) and step 10b / rule G14 (the impl member for a qualified path `<A as T>::NAME`).

Read:
- docs/phase-20261113/SPEC-itoa-full-item-graph/part-5-macro-generated-items.md sections 10.15, 10.16, 10.17 (the rules as built, measured rows, open questions Q38..Q46)
- crates/parseltongue-syntax-span-extractor/src/type-relative-member-records.rs : read_qualified_path_parts, find_qualified_path_member, find_qualified_member_target, find_admitted_member_target
- crates/parseltongue-syntax-span-extractor/src/type-relative-member-tests.rs : test_qualified_path_parts_read, test_qualified_member_trait_match
- crates/parseltongue-syntax-span-extractor/src/body-value-read-records.rs : resolve_read_path_target (the G14 block before `let walk`), find_type_relative_const, check_local_value_binding and the predicate that decides which items are value-path owners
- crates/parseltongue-syntax-span-extractor/src/signature-type-edge-records.rs : read_declared_type_owner, check_signature_row_node, build_type_path_edge (the `qualified` block), resolve_type_path_target
- crates/parseltongue-syntax-span-extractor/src/signature-type-edge-tests.rs : test_declared_type_paths_listed
- crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs : resolve_type, read_type_path_records
- crates/parseltongue-syntax-span-extractor/src/syntax-namespace-input-extraction.rs : check_exported_signature_path (which paths get a stored lookup)
- crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs : test_item_initializer_read_rows, test_declared_type_use_rows, test_qualified_path_member_evidence
- crates/parseltongue-cli-dispatcher-binary/tests/pinned-itoa-source-tree/src/lib.rs (the real source)

The repository rule: an edge target must be proven from stored source evidence; a row that cannot be proven keeps a less exact target or no target, with a record of why. A wrong target is a defect; a missed target is not.

Questions. Answer each with file:line evidence, and say plainly when you could not verify something.
Q1 (G14 soundness). Is there a Rust program, compiling on stable, where rule G14 gives a WRONG impl member for `<A as T>::NAME` (not merely no member)? Consider: `A` a type alias; `A` or `T` shadowed by a generic parameter or a local item; `T` reached through a `use .. as`; a trait with generic parameters (`impl T<u8> for A` and `impl T<u16> for A`); a default const/type in the trait that the impl does not override; a supertrait's member named through the subtrait (`<A as Sub>::N` where N is declared in Super); an impl inside a function body or another module; cfg-disabled impls; a macro-generated impl; specialization-free blanket impls `impl<X: B> T for X`.
Q2 (G14 text parse). read_qualified_path_parts parses the path's source text. Find a text it accepts but should not, or where the byte offsets it returns would point the stored lookup at the wrong path (comments inside the path, raw identifiers, `as` inside an identifier, leading `::`, `crate::`/`self::`/`super::` segments, unicode).
Q3 (G14 anchors). The anchor is looked up with resolve_type(.., false), which accepts only Struct, Enum, Union or Builtin. Is refusing a type alias anchor correct, and is there any anchor kind accepted that should not be?
Q4 (10c). read_declared_type_owner makes the type node after the colon of a const/static, or after `=` of a type alias, one signature row. Is there a syntax form where it lists a path that is not a type use of that item, or misses one (a const in a trait with a default, an associated type with bounds and a default, `type A<T> where T: B = ..;`, a const generic default, `static mut`, an extern static, `impl Trait` in a type alias, an array length path)? Is one `uses_type` row for the whole qualified path plus one per inner path the right inventory for a coding agent?
Q5 (10a). Value paths written in a const or static initializer, a struct field's array length or a type alias's array length are now `reads` sites of that item, and local bindings are looked for only up to the nearest item. Find a case where a local binding is wrongly taken for an item read, or an item read is wrongly dropped as a local (closures in a const initializer, a `const` block expression, patterns in `match` inside an initializer, a nested fn inside a const initializer, const generic parameters used in an array length).
Q6 (tests). Any test above that cannot fail, has a wrong expectation, or a missing case that would let a wrong target through?
Q7 (next step). Step 11 will read the arguments of builtin expression macros such as `assert!($Signed::MIN < 0, "..")` in generated `const _: () = { .. };` blocks, where the macro call's token tree is not parsed as expressions. What is the soundest way to get value paths out of such a token tree without expanding the builtin macro, and what must it refuse?
Keep the answer under 1100 words. Number the answers Q1..Q7.
````

## Answer (verbatim)

````text
Review of G14 (step 10b), 10c and 10a at HEAD 3e4957cbe. I read every file listed; I could not compile or run anything, so each answer below is code-reading, not an executed proof.

**Q1 — G14 soundness.** I found no compiling stable program that gets a *wrong* member; every risky input degrades to no member plus a typed record. Anchor as type alias: `resolve_type(.., false)` (impl-header-source-records.rs:375–378) accepts only Struct/Enum/Union/Builtin, so an alias anchor is `anchor_type_not_stored` and the declaration stands. Shadowing: the anchor and trait are read from *stored* lookups at the site (type-relative-member-records.rs:89–131 builds the sub-ranges; find via `read_type_path_records`, impl-header-source-records.rs:284), which are scope-aware; a TypeParam or local shadowing resolves to a rejected kind. `use .. as`: the stored lookup resolves the renamed path; if it doesn't, `unknown_receiver` → declaration (miss only). Generic traits: `impl T<u8> for A` gets an unresolved trait resolution (impl-header-source-records.rs:161–163, `resolve_type(.., true)` on a path with generic args), stays a candidate, and `proven` (type-relative-member-records.rs:263–269) demands a resolved trait, so one or two such impls give `single_member_unproven`/`several_stored_impl_members` — no member. Trait defaults not overridden: the impl doesn't declare NAME, zero candidates, target stays the declaration, which is where the default lives — correct. Supertrait `<A as Sub>::N`: `impl Super for A` is excluded by the `admit` closure as "another stored trait" (type-relative-member-records.rs:183–189), so no member; the whole-path declaration (Super::N) survives. Blanket `impl<X: B> T for X`: self type unresolved, `proven` requires `same(header)`, no member. cfg-disabled impls: `persist_impl_header_rows` errors on an inactive span (impl-header-source-records.rs:126–128) and inactive sites are skipped when rows are written (lib.rs:521–522), so an inactive impl cannot be a silent candidate. Two soft spots: (a) a fn-local impl's member can be "proven" then dropped by the `endpoints.contains` filter (body-value-read-records.rs:318–320; signature-type-edge-records.rs:343–346), which silently saves the row while the record still says `impl_member_of_named_trait`; (b) that filter is the only guard for (a) and has no test (spec Q46, part-5 10.17). Verified: itoa's four rows match the expected targets (itoa tests, lines 3786–4026).

**Q2 — text parse.** No wrong-offset or false-accept found. `split_once(" as ")` cannot split inside an identifier (identifiers have no spaces); `<A as B as C>::N` fails `plain_path` on the second part; comments, generics, `&`/`[` anchors, `N::M`, `::<` leading forms all fail the char check (type-relative-member-records.rs:61–86). Offsets are byte offsets of trimmed slices into `text`, added to the site start — correct for the spaced expanded form (pinned by `test_qualified_path_parts_read` on `"< i64 as private :: Sealed > :: Buffer"`, type-relative-member-tests.rs:290–294, and by the passing itoa run). Cosmetic: `plain_path` allows `#` anywhere and `is_alphanumeric` accepts non-XID unicode, but such text cannot be a real path site, so a false lookup key finds no stored row (`unknown_receiver`) — a miss, not a wrong target.

**Q3 — anchors.** Refusing a type alias anchor is correct-but-lossy: `<Alias as T>::N` names the same member as the underlying type, so the row keeps the declaration (miss only, permitted by the repo rule). Under the max-information rule, resolving through the alias's stored aliased type would gain rows; that is a backlog choice, not a defect. No accepted anchor kind is unsound: Struct/Enum/Union/Builtin identities match exactly (type-relative-member-records.rs:225–229), and a trait as anchor (`<Sub as Super>::N`, valid Rust) is refused — again a miss only.

**Q4 — 10c.** `read_declared_type_owner` (signature-type-edge-records.rs:44–53) is right for the listed forms: trait consts with defaults (type row + initializer reads), `type P: Q = R` (Q on the bound row, R the alias row, nearest-row rule at signature-type-edge-records.rs:92–103), `static mut` and extern statics (one `STATIC` kind), const-generic defaults (expression paths not listed). `impl Trait`/`dyn Trait` aliases do get a row (bound paths on the bound row; Q41's missing owner-kind field is open in the spec). A macro-call aliased type is also a row: `ast::Type` includes `MacroType` (rust-analyzer nodes.rs:2382–2389) — not covered by a fixture. I could not construct a listed path that is not a type use. One row per qualified path plus one per inner path is the right inventory under Rule 6: the qualified row is what the type *is* (and can be refined by G14), the inner rows name the mentioned items; cost (3 rows per qualified type) is already recorded in 10.16.

**Q5 — 10a.** The nearest-item rule handles nested fns, closures and const/static items inside functions correctly (`check_local_value_binding`, body-value-read-records.rs:480–509, plus `test_item_initializer_paths_listed`). One real miss: a **const block in a function** — `let N = 1; const { N }` with a module const `N`. The nearest item is the fn; the outer `let N` is an IDENT_PAT under it whose scope covers the site, so the item read is dropped as a local. But a const block cannot capture locals, so `N` there names the item. Not verified by compiling rustc (I could not run code); the direction is only a dropped row, never a wrong target.

**Q6 — tests.** All three itoa tests can fail on a wrong target (exact labels, `label()` panics on an unknown target). Gaps: the G14 endpoint filter (Q46) is untested and is what saves fn-local impl members; no fixture for a cfg-inactive or fn-local impl as candidate; no multi-byte-anchor offset pin; `test_qualified_member_trait_match` does not cover a negative impl in the *qualified* path (covered only via the shared G5 code).

**Q7 — step 11.** Do not pattern-match token runs by hand. Enumerate the builtin macros with a fixed argument structure (`assert!`, `assert_eq!`, `debug_assert!`, `debug_assert_eq!`); for `assert!` take only the top-level comma-group 0 of the call's token tree; **re-parse that slice as an expression with the pinned rust-analyzer parser** (wrap in `fn _() { … }`, parse, walk `PathExpr`s) — parsing is Rust's owned layer, not expansion, and it resolves the `<` comparison-vs-turbofish ambiguity that a token scanner cannot (`$Signed::MIN < 0`). Then reuse `check_body_value_path` unchanged. It must refuse: any parse error; any macro not on the list; the message arguments and format-string implicit captures (their name binding is expansion semantics); a path that is a callee, a local, or a generic parameter; and every refusal keeps a typed reason row, never a guessed target.
````

## What was done with it (Devin)

```text
Q1, Q2  no wrong target and no wrong offset found in G14            nothing to change
Q3      a type alias as anchor keeps the trait's declaration        recorded as Q53 (part 5 section 10.18)
Q4      an aliased type that is a macro call has no fixture         recorded in Q54
Q5      const block in a function: an item read hidden by an        CHECKED AND REJECTED. rustc 1.99.1 refuses
        outer let of the same name                                  `const N: usize = 7; fn f() { let N = 1usize; .. }`
                                                                    with E0005 (N is a constant pattern), so a let
                                                                    cannot hide a const. Probe: two files compiled
                                                                    by hand, both rejected. Not measured for a static.
Q6      test gaps (endpoint filter, cfg-inactive and fn-local       recorded as Q46 (existing) and Q54; not built
        impl candidates, multi-byte anchor, negative impl)
Q7      step 11: parse the arguments again with the pinned parser   ADOPTED (rule G15, section 10.18)
        refuse the name of a named format argument                  ADOPTED after a failing fixture case
        refuse every message argument                               NOT TAKEN: a positional message argument is an
                                                                    expression the macro evaluates, so it is a read
        refuse implicit captures in the format string               already so: a string literal holds no path (Q48)
```
