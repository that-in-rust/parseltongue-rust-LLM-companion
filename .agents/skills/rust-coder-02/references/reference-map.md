# Reference Map

Use this file to route quickly before loading the full Rust reference.

## Jump Points

- `Pattern Map`: fast triage of the full scored surface
- `1. API And Type Design`: public API shape, newtypes, typestate, and ownership-friendly boundaries
- `2. Error Design And Diagnostics`: typed errors, context layering, diagnostics, and CLI/FFI boundaries
- `3. Async And Concurrency Reliability`: cancellation, backpressure, structured concurrency, and verification
- `4. Testing And Contract Verification`: doctests, compile-fail tests, property tests, fuzzing, and test readability
- `5. Tooling, Release Gates, And Operations`: clippy, MSRV, supply chain, tracing, secrets, and SQLx gates
- `9. Unsafe And FFI Containment`: minimal unsafe surfaces, `MaybeUninit`, layout, pointers, and nullability
- `High-Value Anti-Patterns`: mistakes to reject before shipping
- `Default Reliability Stack`: compact final checklist

## Section Search

- `rg '^##|^###' references/rust-reliability-reference.md`
- `rg 'Pattern Map|High-Value Anti-Patterns|Default Reliability Stack|How An LLM Should Use This File' references/rust-reliability-reference.md`

## Use Order

1. Read `Pattern Map`.
2. Open the sections that match the boundary you are touching.
3. Finish with `High-Value Anti-Patterns` and `Default Reliability Stack`.
