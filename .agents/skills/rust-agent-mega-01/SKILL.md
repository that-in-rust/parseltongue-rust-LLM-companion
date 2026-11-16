---
name: rust-agent-mega-01
description: One reference for an agent that plans, writes, reviews or hardens Rust (executable specifications, architecture, testing, types, ownership, errors, async, unsafe, performance, quality gates). Use for any Rust change in this repository; prefer idiomatic functional Rust (iterator pipelines, pure functions, typed errors) wherever it is clearer than a loop.
---

# Rust Agent Mega 01

The whole guide is one file: [references/Rust-agent-mega-01.md](references/Rust-agent-mega-01.md)
(9,967 lines). It is a copy of `Rust-skills/Rust-agent-mega-01.md` from
`that-in-rust/agent-room-of-requirements` at commit `97a669e` (2026-11-16). Do not edit the
copy; update it by copying the file again.

## How to read it

Read section 1 first; it routes a task to the sections it needs. Do not load the whole file.

```text
section                                         starts at line
1   Operating principles and workflow                7
2   Work modes and output contract                 281
3   Executable specifications                      345
4   Architecture and boundaries                    509
5   Verification and test-first delivery         1,813
6   Types and API design                         2,404
7   Ownership, borrowing and data handling       3,513
8   Errors and recovery                          3,732
9   Async and concurrency                        4,132
10  Unsafe and FFI                               5,053
11  Performance                                  5,580
12  Naming, modules and documentation            6,524
13  Anti-patterns                                6,585
14  Rust web backends                            6,682
15  Tauri desktop apps                           7,646
16  Quality gates and commands                   9,068
17  Review checklist                             9,855
```

## Rules of this repository that come first

- `AGENTS.md` and the SOP of the current phase win over this guide where they differ.
- Write idiomatic functional Rust wherever possible: iterator pipelines and small pure
  functions instead of loops that push into accumulators, `Option` and `Result` combinators
  instead of flags, no mutation that a returned value can replace. Keep a loop when the
  control flow is heavy and a chain would be less clear (section 13, "Imperative accumulator
  soup").
- Sections 14 (web backends) and 15 (Tauri) are not used in this repository.
