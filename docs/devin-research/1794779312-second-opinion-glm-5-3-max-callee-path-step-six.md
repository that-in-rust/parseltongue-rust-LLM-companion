# Second opinion: GLM-5.3 (pi, max reasoning, read-only) on part 5d step 6, callee paths

Date: 2026-11-16. Reviewed: the uncommitted working tree of part 5d step 6 (rule G9), before
the review-driven changes listed at the end.

## Prompt (verbatim)

```text
You are a read-only reviewer of uncommitted Rust changes in this repository (branch research/tiny-rust-code-mapper). Do not edit files. Review rule G9 ("a call whose callee path names one function") as built, for correctness against Rust name resolution and against the repository rule "never guess a target; keep one row per site with a reason".

Read these files (uncommitted working tree):
- crates/parseltongue-syntax-span-extractor/src/original-call-resolution-records.rs : functions resolve_callee_path_target, check_body_call_callee, extract_original_call_syntax, build_original_call_edge
- crates/parseltongue-syntax-span-extractor/src/syntax-namespace-input-extraction.rs : the block that emits path requests with site_path_context_kind "body_call_callee_path" (search for check_body_call_callee)
- crates/parseltongue-syntax-span-extractor/src/body-value-read-records.rs : check_local_value_binding
- crates/parseltongue-syntax-span-extractor/src/callee-path-call-tests.rs (new fixture tests)
- crates/parseltongue-syntax-span-extractor/src/namespace-legacy-source-fixtures.rs : test_request_call_callee_path
- docs/phase-20261113/SPEC-itoa-full-item-graph/part-5-macro-generated-items.md section 10 (rules G1..G8, steps)

What the change does:
1. The whole callee path of a call expression (`f(..)`, `f::<T>(..)`, `m::f(..)`, `Trait::f(..)`, `Type::f(..)`) is now sent to the stored name lookup as a Value-namespace request in the call's lexical scope, context kind body_call_callee_path. Not sent: a callee with a `<T as Trait>` anchor, a single name bound by a parameter or pattern in the function (check_local_value_binding), a parenthesised callee.
2. resolve_callee_path_target gives the call the function as target only when: the stored row is resolved with exactly one target symbol, of kind Function; the function is a graph node; and, for a qualified callee, the qualifier's own stored Type lookup is exactly one Module. A bare callee without such a target keeps one unresolved row with a reason and a detail (callee_path_unresolved, callee_path_ambiguous, callee_is_not_function, callee_target_not_node). A qualified callee whose qualifier is a trait, a type or unresolved is left to the older type-prefix lookup, with one exception: when the qualifier's Type lookup is stored as resolved but the whole path is stored unresolved WITH a stored reason, the row takes the whole path's reason (case in pinned itoa: `str::from_utf8_unchecked` under `use core::str;` — the Type lookup of `str` says builtin, the Value lookup of the whole path says external_source_missing).
3. dispatch metadata: dispatch_kind "callee_path_function" plus callee_path_lookup_evidence {state, stored reason, target symbol kinds, outcome}.

Measured on pinned itoa: of 56 callee paths, name lookup stores Function for divmod100 x12, slice_buffer_to_str x10, div_rem_1e16 x2, enc_16lsd::<..> x2, u128_ext::mulhi x1 (27 calls gain a target), Function for Unsigned::fmt x10 (NOT taken: qualifier is a trait; planned next rule), Struct for DecimalPairs x1 (tuple struct constructor; stays unresolved, detail callee_is_not_function).

Questions. Answer each with file:line evidence, and say plainly when you could not verify something.
Q1. Is there a Rust program where rule 2 gives a WRONG target (not merely no target)? Consider: a function-local `fn` item or `use` in a block shadowing an outer function; a local closure `let f = |..|` (check_local_value_binding); a tuple-struct or enum-variant constructor; a `const F: fn() = ..` or static holding a fn pointer; macro-generated bodies where the callee token comes from the macro definition (hygiene: can a macro_rules body's bare function name resolve differently at the definition site than in the call's scope as we look it up?); glob imports and prelude names; a callee path with generic arguments on a non-final segment.
Q2. Is the Module-qualifier requirement right, or too narrow / too wide? What about `crate::f`, `self::f`, `super::f` (how does the stored qualifier row look for those — is there one at all, and does the code then refuse them)? An enum-qualified path `E::V(..)`?
Q3. The str exception: is "qualifier resolved + whole path unresolved with stored reason => take the whole path's reason" sound? Is there a case where it replaces a correct prefix-flow outcome with a worse one? Separately, is the stored Type lookup of `str` (builtin, although `use core::str` imports a module named str into the Type namespace) a real defect in the request for the qualifier, and where would the fix belong?
Q4. For the 10 generated `Unsigned::fmt(..)` calls: name lookup stores the trait's method declaration as a Function target. Under the repository's maximum-information rule (give a coding agent the most explicit facts, never a guessed one), what should the row's target be: the trait method declaration `Unsigned::fmt`, the impl member chosen by the argument's type (`<u8 as Unsigned>::fmt` — needs the receiver type `self as u8` / `Self`), or the declaration as target plus the candidate impl members as evidence? State what Rust semantics make certain here and what would be a guess.
Q5. For `DecimalPairs(..)` (tuple struct constructor called as a function): should the calls row target the struct node, stay unresolved, or not be a calls row at all? Same lens as Q4.
Q6. Any defect in the tests (a test that cannot fail, a wrong expectation, a missing case that would let a wrong target through)?
Keep the answer under 900 words. Number the answers Q1..Q6.
```

## Output (verbatim)

**Q1 — Can rule 2 produce a wrong target?**
Within the resolver itself, no: it takes a target only from one stored row naming exactly one `Function` symbol that is an endpoint (original-call-resolution-records.rs:403-455), and every non-Function kind stops (`callee_is_not_function`). Closures, `let f = ..`, params, match/for/closure-param bindings are excluded because `check_local_value_binding` covers all `IDENT_PAT`s of the enclosing `fn` (body-value-read-records.rs:449-470; scope coverage 476+). A body-local `fn`/`const`/`use` shadowing an outer `fn` is *not* excluded there; correctness is delegated to the namespace engine's scope keys — block scopes exist (test_value_path_block_scope, namespace-legacy-source-fixtures.rs:425-465), but the callee-path tests inject rows at the seam (callee-path-call-tests.rs:32-91) and never assert the request's scope key, so **a block-local `fn helper` shadowing a module `fn helper` resolving to the outer (node) function would pass every test here; I could not verify the engine's scoping — unverified**. Tuple-struct/variant constructors resolve to Struct/Variant kinds → refused (Q5). Module-level `const F: fn()` → `Const` kind → refused. Hygiene: `macro_rules` paths resolve at the invocation site for item names (only locals/labels are def-site hygienic), so requesting in the expanded body's scope is semantically right; hygienic locals are excluded by construction. Generic args on a non-final segment (`m::<u8>::f()`): requested (only `TYPE_ANCHOR` is excluded, original-call-resolution-records.rs:783-797) but no fixture pins it and I could not verify the engine's segment handling — flag as untested. Note `test_shadowed_qualifier_blocks_callee` (callee-path-call-tests.rs:273-291) refuses a target Rust would actually give (locals don't shadow type-namespace qualifiers in multi-segment paths) — conservative, not wrong.

**Q2 — Module-qualifier requirement.**
Right as a guard: the Value lookup of the whole path is the real authority; the qualifier check (original-call-resolution-records.rs:432-446) only prevents trait/type members. Too narrow in two ways: (a) `crate::`/`self::`/`super::` qualifiers depend on whether the engine stores a Type-lookup row resolving to Module for keyword segments (`crate` becomes `_crate`, syntax-namespace-input-extraction.rs:1624+; `self`/`super` unverified) — if no row exists, `module_prefix` is false and the call is refused; (b) `E::V(..)` (enum qualifier) is refused and, worse, its whole-path evidence record is discarded because the `stop` closure returns `None` for qualified callees (~line 380-400): a stored `Variant` fact is thrown away, contradicting the max-information rule. Not too wide: Module+Function can't be wrong.

**Q3 — The `str` exception.**
Mostly sound: it fires only when the engine itself stored a typed reason (guard `!stored.is_null()`, ~line 415-420), so local shapes with no stored reason still fall through to the inherent-member lookup. Residual risk: a whole path stored unresolved-with-reason where the qualifier is a *local* type whose member the old flow could extract preempts `resolve_inherent_member_target` — I can't construct one on itoa (the two extracted `type_path_associated_function` rows survive only if their paths are stored resolved or without reason), but the exception outranks the only extraction path for struct-qualified calls. Narrowing to qualifier kinds Module/Builtin/Trait, or running it only when the old flow yields no target, would remove the risk. On `str`: yes, real precedence gap — `use core::str;` binds a module `str` in the type namespace, which should shadow the shy primitive; storing `Builtin` names the wrong binding, though on itoa both targets sit in unread `core` so no outcome changes. The fix belongs in the namespace engine's binding precedence (or storing both candidates), not in `resolve_callee_path_target`.

**Q4 — `Unsigned::fmt(..)`.**
Certain from the path alone: the callee names the trait method declaration `Unsigned::fmt` (function item taking `&self`). Choosing `<u8 as Unsigned>::fmt` requires the receiver's type — proven only when the call site's first-argument type is stored; guessing it violates Rule 1. Under Rule 6: **target the trait method declaration, carry the candidate impl members (and their receiver types, when stored) as dispatch evidence**, upgrading to an impl-member target only when exactly one candidate's receiver type is proven — which is what part-5 Q21 already plans (part-5-macro-generated-items.md:577-583).

**Q5 — `DecimalPairs(..)`.**
Rust is certain: the callee is the struct's constructor function; the stored Struct target names it. Target the **struct node** with a distinct dispatch detail (e.g. `tuple_struct_constructor`) and the Struct-kind evidence — a coding agent learns "this call constructs DecimalPairs". Staying unresolved discards a proven fact; dropping the row loses the site. Same for enum variants if variants are nodes (graph rules don't say — decide in the PRD). Caveat: this must match the expected dataset or be recorded as a divergence.

**Q6 — Test defects.**
(1) `outcome(&[])` in test_bare_callee_untargeted_reasons (callee-path-call-tests.rs:165-174) tests pre-change behavior, not G9. (2) No test asserts the callee request's *lexical scope key* (block-local `fn`/`use` shadowing is the wrong-target hole of Q1). (3) `(_, [_,_,..])` → `callee_path_ambiguous` arm untested for a "resolved" state with two targets. (4) No case for prefix=Module + path unresolved *without* reason (falls to old flow). (5) No test for non-final generic args. (6) `rfind`-positioned fixture rows work but are fragile (`"m"` happens to be last in source); fine today. Nothing that cannot fail outright, but (2) and (3) would let a wrong target through.

## What I did with it

Checked in source or by test, and adopted:

- **Q2(b), evidence discarded on qualified rows.** True as reviewed. The lookup record is now
  kept on every call whose callee path has a stored lookup, whether or not the callee path
  decides the row. Pinned by `test_callee_path_function_targets` (56 records on itoa).
- **Q3, the exception could displace a struct qualifier's member lookup.** True as reviewed.
  Narrowed to a qualifier stored as a builtin type. Pinned by
  `test_struct_qualifier_keeps_member`; widening it again fails that test.
- **Q6(2), no test of the request's scope.** Added `test_callee_path_block_scope`: the request
  for `h` in `fn caller() { fn h() {} h(); }` is made in the block's scope, where exactly
  one of the two `h` bindings lives. It passed on first run, so it describes existing
  behaviour; it does not run the name lookup itself.
- **Q6(3), Q6(4).** Added the cases: stored as resolved with two targets (ambiguous, no
  target); a module qualifier whose whole path has no target (older outcome stands).
- **Q6(1).** Kept the case and labelled it as the unchanged older behaviour.

Recorded, not built:

- **Q2(a)** `crate::` / `self::` / `super::` qualifiers, and **Q1** generic arguments on a
  non-final segment: part 5 Q25. Neither occurs in itoa; each can only lose a target.
- **Q3** the `str` precedence gap in name lookup: part 5 Q24.
- **Q4** and **Q5**: taken as the design for the next step and written into the PRD
  (trait method declaration as target with candidates as evidence; the struct node as the
  target of a tuple-struct constructor call).

Not verified by me: the reviewer's statement on `macro_rules` hygiene for item names. It
matches the Rust reference as I know it; no test here depends on it.
