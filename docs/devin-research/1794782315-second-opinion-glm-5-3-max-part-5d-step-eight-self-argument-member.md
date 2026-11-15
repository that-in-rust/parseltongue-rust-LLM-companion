# Second opinion: GLM-5.3 (pi, max reasoning, read-only) on part 5d step 8, the impl member by the method's own `self`

Date: 2026-11-16. Reviewed: the uncommitted working tree of part 5d step 8 (rule G12), before
the review-driven changes listed at the end. Run time about 7 minutes; the error file was empty.

## Prompt (verbatim)

```text
You are a read-only reviewer of uncommitted Rust changes in this repository (branch research/tiny-rust-code-mapper). Do not edit files. Review rule G12 as built, for correctness against Rust's trait method selection and against the repository rule "never guess a target; keep one row per site with a reason; give a coding agent the most source-proven information".

Read these files (uncommitted working tree):
- crates/parseltongue-syntax-span-extractor/src/original-call-resolution-records.rs : functions check_own_self_argument, extract_original_call_syntax, find_self_argument_member, list_trait_method_candidates, resolve_callee_path_target (its last 60 lines), build_original_call_edge
- crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs : extract_impl_header_syntax (member rows; new field member_self_parameter_kind) and resolve_type
- crates/parseltongue-syntax-span-extractor/src/self-argument-member-tests.rs (new fixture tests)
- crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs : test_trait_path_call_targets
- docs/phase-20261113/SPEC-itoa-full-item-graph/part-5-macro-generated-items.md section 10 (rules G9..G11, step 7, open question Q27)
- crates/parseltongue-cli-dispatcher-binary/tests/pinned-itoa-source-tree/src/lib.rs (the itoa source: macro impl_Integer, trait Unsigned, trait private::Sealed)

Rule G12 as built. A call `Trait::name(self, ..)` already targets the trait's declaration of `name` and lists the stored impl members that could run (G10). G12 replaces that target by one impl member when ALL of these hold:
1. Syntax: the call's first argument is exactly the path `self`; the nearest enclosing `fn` declares `self` by value with no type (`self` or `mut self`), and that `fn` has no BLOCK_EXPR ancestor (so it is not in an impl written inside a body).
2. The call site's owner item is a member (FN, self by value) of a stored impl header H1 that has no generic parameters or where clause, is not negative, is not inside a function body, and whose self type is a plain path (no generic arguments) with a stored resolved identity X.
3. Among stored impl headers with the same properties, with self type identity equal to X and trait identity equal to the trait named by the call's qualifier, exactly one member is an FN named `name` that takes `self` by value.
4. That member is a graph node.
Then target = that member, dispatch_kind = "trait_path_impl_method", and the row's trait_method_lookup_evidence keeps the candidate list, adds receiver_type_stored_flag true, receiver_self_type_identity, and trait_method_declaration_identity. Otherwise the G10 row is unchanged.

On pinned itoa this changes 5 rows (`Unsigned::fmt(self, buf)` inside `<uN as private::Sealed>::write` for u8,u16,u32,u64,u128 -> `<uN as Unsigned>::fmt`). The 5 calls `Unsigned::fmt(self.unsigned_abs(), ..)` keep the trait declaration.

Questions. Answer each with a verdict (sound / unsound / cannot tell) and the exact source lines you relied on.
Q1. Is there any compiling Rust program in which conditions 1-4 hold and the function that actually runs is NOT the chosen member? Consider: trait generic parameters (`impl T<A> for X` and `impl T<B> for X`), type aliases as self type, a plain-path self type with defaulted generic parameters, `Self` shadowing, closures and async blocks between the call and the `fn`, macro-generated text (the impls here are in macro expansions), specialization or default methods, a trait method that takes `self` by value where the argument is coerced, auto traits, and two impls made distinct only by cfg.
Q2. Is the BLOCK_EXPR guard in check_own_self_argument sufficient and not too strict? Is there a legal position of a method (e.g. inside a const initializer block, inside a trait's default method body) where the owner item of the call site and the nearest fn differ and the guard does not catch it?
Q3. The evidence record: is anything a coding agent needs missing or misleading (for example, should the row also keep the trait declaration as a second target, or is trait_method_declaration_identity in the record enough)? The edge model has one target per row.
Q4. Tests: name any rule in find_self_argument_member or check_own_self_argument that no assertion in self-argument-member-tests.rs would catch if removed.
Q5. The next step is the 5 calls `Unsigned::fmt(self.unsigned_abs(), ..)` (return type is in core, unread) and 104 method calls in generated bodies whose receiver type is not determined. Given the code as it is, what is the smallest sound next rule that gives the most rows a target without guessing? Be specific about what stored evidence it would read.
Keep the answer under 900 words.
```

## Output (verbatim)

## Review of G12 (read-only)

**Q1 — verdict: unsound in one corner; sound for every shape pinned itoa has.**

The corner: callee paths whose trait qualifier carries explicit generic arguments. `find_self_argument_member` and its call site (`original-call-resolution-records.rs:618-627`) never consult `call_supported_path_flag` or reject generic arguments; `check_body_call_callee` blocks only TYPE_ANCHOR (`:984-987`) and `check_body_call_prefix` (`:946-967`) has no generic-args filter, and the extractor submits such prefixes (`syntax-namespace-input-extraction.rs:1485`). So for `trait T2<A = u8> { fn f(self); }`, `impl T2<u16> for u8`, `impl T2 for u8`, a call `T2::<u16>::f(self)` in a plain by-value method: the u16 impl is excluded from candidates because its trait source `T2<u16>` fails `type_simple_path_flag` (`impl-header-source-records.rs:479-495`, `resolve_type` `:363-366`), leaving exactly one candidate — the *defaulted* impl — which G12 targets, while the program runs the u16 member. Missing guard: require no generic arguments on the callee path and its qualifier. Whether today's namespace stage actually stores such a prefix row as `Trait` I cannot tell from these files; the guard is needed regardless.

Everything else listed is safe: generic impls/negative/body-nested impls are excluded by `plain()` (`:436-443`); type-alias self types either resolve to the alias (excluded: kinds allowed are Struct/Enum/Union/Builtin, `impl-header-source-records.rs:388-398`) or resolve through to the same type both sides (then the chosen member is the one that runs); `Self` is a keyword, unshadowable; closures/async blocks introduce no nearer `ast::Fn`, so `self`'s type stays the impl's Self; cfg-distinct impls cannot both be active in a compiling run (coherence) and inactive impl spans are rejected at persist (`impl-header-source-records.rs:110-113`); a default method with no override yields zero candidates → G10 fallback (the declaration holds the default body); UFCS by-value receivers get no autoref/deref, and `self`-vs-`&self` receiver mismatches don't compile; macro-generated impls are distinguished by provenance-bearing occurrence keys.

**Q2 — verdict: sufficient, slightly too strict (loses only).**

The guard is load-bearing: `impl_function_body_ancestry_flag` only tracks `Fn` ancestors (`impl-header-source-records.rs:458-461`), so an impl inside a module-level const block passes `plain()`; the BLOCK_EXPR check (`:1007-1013`) is what rejects calls inside it. Const/static initializers cannot legally reference `self`, and nested free fns cannot declare `self`, so no legal position makes the flag true with a nearest-fn/owner mismatch. A trait default method (module level) gets flag true but no owner in impl headers (`:452-460`) → G10, correct. Overly strict: const-block impls lose targets — acceptable.

**Q3 — verdict: adequate; two gaps.**

One target per row is the model; keeping the declaration as `trait_method_declaration_identity` plus the full candidate list (`:630-643`) is the right encoding — a second target row would duplicate sites. Gaps: (a) the record never names the *owner* impl header whose self type justified the choice — the row's source names the method, not the impl; (b) member-name matching does not trim `r#` (`:464-466`, same in `list_trait_method_candidates` `:391-393`) while the call name is trimmed (`:1058-1060`) — a `fn r#fn` member silently loses the target; inconsistent with `resolve_inherent_member_target` (`:883-887`), which trims.

**Q4 — untested rules.**

1. `path.qualifier().is_none()` (`:999`): no compiling syntax reaches it (a `self` segment can't be qualified), and no fixture pins it — remove it and no assertion fails.
2. The `r#` asymmetry above: no test.
3. Owner lookup takes the *first* plain header containing the key (`find`, `:452-460`): no fixture has two plain impls sharing a member key (impossible by construction, but unpinned). Everything else — plain flags, self/trait identities, by-value kinds, exactly-one, first-argument position, BLOCK_EXPR guard, nearest-fn semantics — has a killing assertion (`self-argument-member-tests.rs:41-67, 70-172, 289-313`).

**Q5 — smallest sound next rule: cast receivers (ARCH E5's second shape).**

`(self as $primitive).write(buf)` (`lib.rs:211`) — on a 64-bit run two rows, whose true targets are itoa nodes. Rule: a method call whose receiver is a cast expression whose written target type is a simple stored type → receiver identity from the stored path rows for that type text; then exactly one FN member of that name in stored plain impls for that identity (reuse G12's header predicates; method-call receivers may be any self kind — require uniqueness across all receiver kinds, else ambiguous). Evidence read: `method_receiver_evidence` (add a cast shape in `extract_method_receiver_syntax`, `:1088-1108`, storing the cast type text), `path-resolution-site-records`, `impl-header-index-table`, endpoints. The other ~100 rows and the 5 `Unsigned::fmt(self.unsigned_abs(), ..)` calls have core-typed receivers/targets (`unsigned_abs`, `try_into`, `len`, `get_unchecked`, `MaybeUninit::write`); they cannot gain an itoa target without core, and the 5 already carry the G10 declaration target — leave them at E4's open decision rather than guess.

## What I did with it

Line numbers in the output refer to the tree as reviewed; they have moved since.

Checked in source or by test, and adopted:

- **Q1, an impl of the trait with generic arguments is left out of the candidates.** I checked
  the mechanism in the code as reviewed: the candidate filter required the impl's trait to be
  stored as resolved, and a trait written with generic arguments is not stored as resolved.
  So `impl T<u16> for u8` beside `impl T for u8` left one candidate. Changed: every stored
  trait impl counts unless its trait or its type is stored as another one; exactly one
  candidate is required, and it must be plain, of the stored trait, and take `self` by
  value. Pinned on header rows by `test_self_argument_member_rules` (removing either
  "not stored counts" rule fails it). Not reproduced on real source text.
- **Q3(a), the row does not name the impl that justified the choice.** Added
  `receiver_impl_header_occurrence`, `receiver_self_type_source_text` and
  `selected_member_occurrence_key`. Pinned by `test_trait_call_self_member` and, for the
  type text, by `test_trait_path_call_targets` on pinned itoa.
- **Q3(b), `r#` on the member name.** The member name is trimmed in G12 now; pinned by a
  case in `test_self_argument_member_rules`. The same asymmetry in
  `list_trait_method_candidates` (G10's evidence list) is not changed.
- **Q4(1), the qualifier check on the `self` path could not be reached.** Removed.

Not adopted:

- **Q1, an explicit guard on generic arguments in the callee path.** With the candidate rule
  above, a call can get a member only when one stored impl may be of this trait for this
  type, and then generic arguments cannot select another. Recorded as part 5 Q31.

Recorded, not built:

- **Q2**, an impl inside a module-level `const` block loses the member: part 5 Q32.
- **Q5**, cast receivers as the next rule: part 5 Q34. The reviewer's count ("two rows on a
  64-bit run") and its statement that the other method calls all need `core` are its
  reading of the source, not measurements. I have not checked either in the stored rows.

Not verified by me: the reviewer's statements that a `self` path segment cannot be
qualified, that inactive impl spans are rejected when impl headers are stored, and that
`impl_function_body_ancestry_flag` tracks only `fn` ancestors. No decision here depends on
the first two; the third is why the block rule in `check_own_self_argument` is kept.
