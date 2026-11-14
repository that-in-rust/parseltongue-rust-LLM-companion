# Part 2 — the first call from source

**State: PARTLY MET.** One call is proven from source and read back frozen:
`<Buffer as Default>::default --calls--> Buffer::new`. 2 of 61 call sites resolve; 59 are
stored unresolved with a typed reason.

## Requirements

- **REQ-RUST-034.0 — paths inside bodies.** A path in a function body, const or static
  initializer resolves to an item or is stored unresolved with one reason. A name shadowed by
  a local binding never resolves to an item.
- **REQ-RUST-036.0 — `Type::member` lookup.** Resolve `Type`, search its inherent impls, then
  impls of traits in scope, and emit a target only when exactly one member matches. Zero or
  several matches are unresolved.

## What the code does today

`resolve_original_call_edges` in
`parseltongue-syntax-span-extractor/src/original-call-resolution-records.rs`:

- reads `CALL_EXPR` and `METHOD_CALL_EXPR` rows from `body-site-reference-table`, original
  source only (rows with an expansion chain are skipped);
- resolves the callee only when it is a path call `Type::member(...)` with no generic
  arguments and no shadowed head;
- finds `member` only in non-generic inherent impls of `Type`;
- stores everything else unresolved: method calls, trait impls, generic heads
  (`TraitSolverNeeded`), several candidates (`AmbiguousTarget`), no candidate
  (`MissingTarget`).

## Tests on pinned itoa

```text
034.0        test_default_new_call_roundtrip
036.0        test_default_new_call_roundtrip (doc comment does not name 036.0; add the tag)
```

## Still missing

- Method calls: [part 3](part-3-generic-call-format-write.md).
- The four other type-relative shapes itoa uses (`i128::MAX_STR_LEN`, `Self::MAX_STR_LEN`,
  `I::MAX_STR_LEN`, `<$primitive as Integer>::MAX_STR_LEN`) and type uses and reads in
  bodies: [part 4](part-4-signatures-other-edge-kinds.md).
- The split of the 59 unresolved calls by reason has not been measured.
