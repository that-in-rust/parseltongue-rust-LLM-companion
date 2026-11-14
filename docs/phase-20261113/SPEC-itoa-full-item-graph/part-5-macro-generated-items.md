# Part 5 — macro-generated items as graph nodes

**State: NOT STARTED. Stub: the work packet is written when this part is picked.**

## Requirement

- **REQ-RUST-046.0 — macro-generated items as nodes.** Items produced by `macro_rules`
  expansion are nodes with their own identity and the invocation they came from; their
  signatures and bodies are analysed like original ones.

itoa: three item macros (`impl_Integer`, `impl_Integer_size`, `impl_Unsigned`) generate 28 of
the 33 impl blocks. 45 of the 73 expected nodes come from them. The expander and its
projection exist and have 10 passing tests (REQ-RUST-006.2 in the wider-scope SPEC), but no
generated item reaches the graph, and the call resolver skips every site with an expansion
chain.

First itoa target:
`<u128 as Sealed>::write -> <u128 as Unsigned>::fmt -> div_rem_1e16 -> u128_ext::mulhi`.

## Blocked on or waiting for

- Decision OD-2: identity and source container of a generated item. It blocks 279 expected
  edges and 45 expected nodes ([backlog](../BACKLOG-deferred-work-open-decisions.md)).
- Parts 3 and 4, for calls with a concrete receiver inside generated impls.
