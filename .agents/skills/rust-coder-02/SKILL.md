---
name: rust-coder-02
description: Reliability-first Rust skill for libraries, CLIs, async services, app backends, and FFI wrappers. Use when Rust work needs strong boundary types, typed errors, async and cancellation safety, concurrency verification, compile-fail or property testing, unsafe containment, or operational release gates.
---

# Rust Reliability

## Overview

Use this skill when the task is primarily about writing, refactoring, or reviewing Rust that keeps working after the happy path. It biases toward patterns that prevent misuse, improve diagnostics, survive async and concurrency stress, and keep unsafe or FFI edges tightly contained.

## When To Use

- Libraries, CLIs, async services, backends, parsers, protocol code, or contained FFI wrappers
- Public API design, error surface design, async cancellation, concurrency, or release-hardening work
- Code reviews where the real question is whether the Rust will stay safe and diagnosable under change
- Rust tasks that need more production reliability guidance than syntax or style guidance

## Workflow

1. Read [references/reference-map.md](references/reference-map.md), then load only the relevant sections in [references/rust-reliability-reference.md](references/rust-reliability-reference.md).
2. Default to the strongest patterns first: parse-don't-validate with newtypes, typed library errors, no blocking or locks across `.await`, bounded concurrency and cancellation, doctests and misuse tests, and minimal unsafe surfaces.
3. Use the skeptical question throughout: what fails here under malformed input, cancellation, backpressure, non-UTF-8 data, concurrency, or API evolution?
4. Validate with the right gates for the boundary you touched: `cargo test`, doctests, compile-fail tests, property or fuzz tests, `clippy -D warnings`, and any concurrency verification the code deserves.
5. Finish by reading the anti-patterns and default reliability stack again before you hand off the result.

## Companion Use

- Bring in `idiomatic-rust-coder-01` when the task also needs executable specs, TDD-first planning, four-word naming discipline, or explicit L1/L2/L3 architecture planning.
- Use this skill as the reliability playbook and `idiomatic-rust-coder-01` as the delivery process when both are relevant.

## Reference Use

- Use [references/reference-map.md](references/reference-map.md) first for quick routing.
- Use [references/rust-reliability-reference.md](references/rust-reliability-reference.md) for the full scored patterns, anti-patterns, and default reliability stack.
- For large-file navigation, prefer heading search such as `rg '^##|^###' references/rust-reliability-reference.md`.

## Output Expectations

- Prefer APIs that make misuse harder than correct use.
- Keep error surfaces actionable and boundary-aware.
- Treat async, cancellation, and concurrency as correctness problems, not just performance concerns.
- Fence unsafe code tightly and document the invariants it relies on.
