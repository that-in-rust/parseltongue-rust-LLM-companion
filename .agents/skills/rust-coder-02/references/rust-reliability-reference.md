---
name: rust-coder-02
description: reliability-first idiomatic rust patterns, expanded edition
model: sonnet
color: orange
---

# Rust Coder 02

> A larger, stricter, and more relevant successor to `rust-coder-01.md`
>
> Built from `Notes2026` with one bias above all others: write Rust that keeps working after the happy path.

## Premise Check

The first draft of `rust-coder-02` was too compressed.

That made it cleaner, but not strong enough for the role you want it to play.

If `rust-coder-01.md` is the broad baseline, then `rust-coder-02.md` should be:

- more selective about what matters
- more explicit about why patterns prevent bugs
- broader in failure-heavy areas
- deeper in tradeoffs
- clearer about what is repo-local convention versus generally strong Rust practice

That is what this revised version is trying to be.

## What rust-coder-01 Gets Right

`rust-coder-01.md` already has several strong instincts:

- tests should shape implementation, not merely verify it later
- requirements should be executable and measurable
- boundaries and layers matter
- naming consistency reduces confusion
- architecture should be designed for testability, not patched for it later

Those are worth keeping.

## Where rust-coder-01 Is Thin

For production-reliable Rust, the biggest gaps are not syntax-level idioms. They are failure-boundary idioms.

The missing or underweighted areas are:

- public API evolution without semver traps
- typed error design versus app-level aggregation
- byte-safe diagnostics when UTF-8 assumptions break
- async cancellation, backpressure, and structured shutdown
- cancel-safe future design inside `select!`
- concurrency verification beyond “it passed once”
- panic-free boundary code for slicing, offsets, and external input
- compile-fail tests, fuzzing, and richer contract testing
- operational patterns like tracing, secret handling, and supply-chain gates
- advanced type patterns that make incorrect states unrepresentable

## Source Pool

This file is synthesized mainly from:

- `Notes2026/RustLibraryResearch20260123/S77-IdiomaticRustPatterns.md`
- `Notes2026/ripgrep-idiomatic-patterns/*`
- `Notes2026/MDWisdom202601/rust-analyzer-analysis/10-SUPER-HQ-IDIOMATIC-PATTERNS.md`

Those sources are useful together because:

- `S77` gives the wide idiom surface
- ripgrep gives battle-tested production patterns with operational realism
- rust-analyzer contributes the type-system and correctness-heavy advanced moves

## Selection Thesis

The rule here is not “include the smartest Rust tricks.”

The rule is:

> Prefer patterns that reduce bug classes, misuse, and ambiguous failure behavior over patterns that merely look advanced.

That means a pattern scores highly when it:

- prevents a mistake at compile time
- narrows API misuse
- produces better diagnostics when things still fail
- behaves well under async, concurrency, and cancellation
- shows up in serious production code rather than only in toy examples

## Reliability Rubric

Each pattern is scored out of `100`:

- `35` points: bug-prevention power
- `20` points: misuse resistance
- `15` points: diagnosability
- `15` points: concurrency or async safety
- `15` points: breadth and repeatability

Only patterns above `80` are kept.

## Coverage Claim

This file is now intended to be comprehensive enough for most LLM-authored Rust work in:

- libraries
- CLIs
- async services
- application backends
- moderately low-level systems code
- FFI wrappers with contained unsafe

It is still not a full specialist manual for:

- compiler internals
- kernel-grade lock-free systems
- deeply embedded no-std firmware
- extremely advanced proc-macro internals

That is a boundary claim, not a weakness claim.

## Repo-Local Versus General Rust

This distinction matters.

Strong repo-local conventions from `rust-coder-01`:

- four-word naming
- executable requirements format
- one-feature-per-version discipline
- Mermaid-only documentation rules

Strong general Rust conventions:

- typed boundaries and newtypes
- typed errors in libraries
- no blocking in async contexts
- cancel-safe `select!`
- `loom`, `proptest`, doctests, compile-fail tests
- `#[non_exhaustive]`, `Cow`, `OnceLock`, `ExitCode`, `tracing`

`rust-coder-02` is mainly about the second list.

## Pattern Map

| ID | Score | Pattern |
| --- | ---: | --- |
| `RC2-01` | `93` | Accept borrowed inputs in public APIs |
| `RC2-02` | `98` | Newtypes and parse-don't-validate |
| `RC2-03` | `89` | `From` and `TryFrom` instead of ad hoc constructors |
| `RC2-04` | `84` | Option and Result combinators over nested matching |
| `RC2-05` | `83` | Trait objects versus generics as an explicit boundary decision |
| `RC2-06` | `82` | Return `impl Iterator` from APIs when laziness matters |
| `RC2-07` | `90` | Named outcome enums over booleans and sentinels |
| `RC2-08` | `85` | Blanket impls for `&T`, `&mut T`, and `Box<T>` where ergonomic |
| `RC2-09` | `83` | Flat public API facades with `pub use` |
| `RC2-10` | `88` | `PhantomData<fn() -> T>` invariance for typed handles |
| `RC2-11` | `92` | Type-state plus DropBomb for must-complete protocols |
| `RC2-12` | `97` | `thiserror` in libraries, `anyhow` or `eyre` in binaries |
| `RC2-13` | `95` | Opaque public errors with private kind representation |
| `RC2-14` | `92` | `#[non_exhaustive]` on public enums and structs that will grow |
| `RC2-15` | `94` | Actionable diagnostics with exact failure location and repair hint |
| `RC2-16` | `87` | Binary-safe diagnostics for non-UTF-8 inputs and outputs |
| `RC2-17` | `90` | Error context layering at I/O and orchestration boundaries |
| `RC2-18` | `87` | `ExitCode` and error-chain inspection at CLI boundaries |
| `RC2-19` | `84` | FFI unwind boundaries with `catch_unwind` and explicit contracts |
| `RC2-20` | `97` | No blocking and no locks across `.await` |
| `RC2-21` | `96` | Bounded channels, backpressure, and cancellation tokens |
| `RC2-22` | `94` | Cancel-safe `tokio::select!` and future ownership discipline |
| `RC2-23` | `88` | Structured concurrency with `JoinSet` and task scopes |
| `RC2-24` | `89` | Concurrency model checking with `loom` |
| `RC2-25` | `85` | Atomic ordering hygiene plus verification, not superstition |
| `RC2-26` | `83` | Thread naming, join discipline, and `move` closure hygiene |
| `RC2-27` | `84` | `OnceLock`, `LazyLock`, and one-time initialization patterns |
| `RC2-28` | `91` | Doctests as executable contracts |
| `RC2-29` | `88` | Compile-fail UI tests for misuse contracts |
| `RC2-30` | `90` | Property-based tests for invariants and round trips |
| `RC2-31` | `88` | Fuzzing for parser, decoder, and protocol frontiers |
| `RC2-32` | `83` | Snapshot tests with `expect-test` where textual behavior matters |
| `RC2-33` | `84` | Test infrastructure should optimize failure readability and isolation |
| `RC2-34` | `89` | `cargo clippy -D warnings` as a correctness gate |
| `RC2-35` | `82` | `rustfmt` as non-negotiable style infrastructure |
| `RC2-36` | `84` | MSRV policy and toolchain pinning |
| `RC2-37` | `84` | Supply-chain policy with `audit`, `deny`, or `vet` |
| `RC2-38` | `88` | Structured observability with `tracing` |
| `RC2-39` | `86` | Secret handling with `secrecy` |
| `RC2-40` | `85` | SQLx compile-time query checking and pooling discipline |
| `RC2-41` | `90` | Panic-free arithmetic and slicing |
| `RC2-42` | `86` | Builder plus frozen config struct |
| `RC2-43` | `84` | Accumulate-then-build and incremental error collection |
| `RC2-44` | `84` | Prepare-once query-many precomputation |
| `RC2-45` | `90` | Interior mutability versus synchronization primitives |
| `RC2-46` | `88` | Borrow-checker-friendly API design and two-phase borrow awareness |
| `RC2-47` | `87` | Iterator contracts, fallible pipelines, and owned/borrowed iteration |
| `RC2-48` | `85` | Collection choice discipline |
| `RC2-49` | `86` | Pin, pin projection, and async trait boundaries |
| `RC2-50` | `84` | HRTBs, GATs, and object-safety-aware trait design |
| `RC2-51` | `88` | Feature flags additive, sealed traits, and coherence hygiene |
| `RC2-52` | `83` | Macro and proc-macro hygiene, spans, and diagnostics |
| `RC2-53` | `95` | Unsafe encapsulation with minimal surface, SAFETY docs, and Miri |
| `RC2-54` | `89` | MaybeUninit, aliasing invariants, and RAII resource guards |
| `RC2-55` | `88` | FFI layout, DST/wide pointers, and explicit nullability |
| `RC2-56` | `83` | NonZero niches, ZSTs, and uninhabited types |
| `RC2-57` | `85` | Source-annotated diagnostics with miette and color-eyre |
| `RC2-58` | `84` | Coverage as a gate with cargo-llvm-cov |
| `RC2-59` | `84` | Channel and synchronization choice matrix |
| `RC2-60` | `85` | Workspace, toolchain, and additive-feature discipline |

## 1. API And Type Design

### `RC2-01` Accept borrowed inputs in public APIs - `93/100`

Why it matters:

- `&[T]`, `&str`, `AsRef<T>`, and `Borrow<T>` reduce caller friction and needless allocation.
- The API commits to semantics rather than to a specific backing container.
- Borrow-friendly signatures also make hot paths cheaper by default.

Use when:

- the function reads from input rather than owning it
- callers may naturally have slices, arrays, `Vec`, `String`, `&str`, or borrowed views

Avoid when:

- the function must take ownership for long-term storage or mutation semantics

Source basis:

- `S77` `A.2`

### `RC2-02` Newtypes and parse-don't-validate - `98/100`

Why it matters:

- Turning “raw data plus repeated checks” into a validated type eliminates entire classes of downstream bugs.
- Newtypes also prevent accidental mixing of offsets, IDs, units, and domain concepts that are all primitives underneath.
- This is one of the highest-leverage Rust patterns because it converts runtime discipline into compile-time discipline.

Use when:

- IDs, offsets, handles, units, validated strings, and FFI wrapper types exist

Avoid when:

- the wrapper adds no semantic distinction at all and only makes the API noisier

Source basis:

- `S77` `A.5`
- rust-analyzer `Pattern 1.7` for defensive newtype usage

### `RC2-03` `From` and `TryFrom` instead of ad hoc constructors - `89/100`

Why it matters:

- Conversions become composable and discoverable.
- Infallible and fallible conversions are separated cleanly.
- Pipelines become easier to read because call sites rely on standard Rust conversion semantics instead of custom one-off method names.

Use when:

- the operation is fundamentally a conversion between two domain types

Avoid when:

- the operation does real work beyond conversion and deserves a named constructor

Source basis:

- `S77` `A.4`

### `RC2-04` Option and Result combinators over nested matching - `84/100`

Why it matters:

- Many Rust bugs are not logic bugs but readability bugs: too much nested matching hides the true control flow.
- `map`, `and_then`, `transpose`, `ok_or_else`, and `try_fold` keep linear flows linear.

Use when:

- the flow is straightforward transformation or chaining

Avoid when:

- a full `match` would make branching semantics clearer than stacked combinators

Source basis:

- `S77` `A.7`
- `S77` `A.90`

### `RC2-05` Trait objects versus generics as an explicit boundary decision - `83/100`

Why it matters:

- Static dispatch and dynamic dispatch are both valid. Problems appear when the choice is accidental.
- Hot paths often want generics.
- Plugin points, heterogeneous collections, and boundary seams often want `dyn Trait`.

Use when:

- flexibility and type-erasure are worth the dispatch cost or code-size savings

Avoid when:

- a tight loop or heavily inlined path would pay for unnecessary dynamic dispatch

Source basis:

- `S77` `A.9`
- `S77` `A.115`

### `RC2-06` Return `impl Iterator` from APIs when laziness matters - `82/100`

Why it matters:

- It preserves laziness and avoids committing public APIs to messy concrete adapter types.
- It gives callers cheap pipelines without `Box<dyn Iterator>` overhead in common cases.

Use when:

- the returned pipeline should stay lazy and the concrete type is an implementation detail

Avoid when:

- you need heterogeneity, object safety, or long-lived trait objects

Source basis:

- `S77` `A.41`

### `RC2-07` Named outcome enums over booleans and sentinels - `90/100`

Why it matters:

- A boolean tells you something happened.
- An enum tells you which action the caller must take next.
- This is especially powerful in control-flow helpers, search engines, state machines, and CLI parsing.

Use when:

- the return value controls caller behavior rather than merely conveying a fact

Avoid when:

- there are truly only two obvious semantic states and both are already universally understood

Source basis:

- ripgrep searcher core `Pattern 1`
- ripgrep main entry `Pattern 2`

### `RC2-08` Blanket impls for `&T`, `&mut T`, and `Box<T>` where ergonomic - `85/100`

Why it matters:

- Good traits are easier to use when references and boxed forms work naturally.
- This reduces wrapper boilerplate and makes shared access patterns feel native.

Use when:

- a trait is read-only, object-safe, or naturally delegating

Avoid when:

- the trait’s object-safety or ownership semantics make these impls misleading

Source basis:

- ripgrep trait design `Patterns 2 and 4`
- ripgrep type system `Patterns 5 and 6`

### `RC2-09` Flat public API facades with `pub use` - `83/100`

Why it matters:

- Internal file layout and public API shape should not be welded together.
- Re-export facades let a crate reorganize internals without punishing downstream callers.

Use when:

- a crate has several internal modules but wants one stable import surface

Avoid when:

- you are merely hiding useful structure rather than simplifying it

Source basis:

- ripgrep searcher toplevel `Pattern 2`
- `S77` `A.92`

### `RC2-10` `PhantomData<fn() -> T>` invariance for typed handles - `88/100`

Why it matters:

- Advanced handle types, arena indices, and lifetime-sensitive tokens can become unsound if variance is too permissive.
- Invariance is a subtle but real correctness lever in systems code.

Use when:

- the type parameter carries lifetime or aliasing implications that must not be coerced

Avoid when:

- ordinary covariance is actually correct and the type does not model a handle discipline

Source basis:

- rust-analyzer `Pattern 1.1`

### `RC2-11` Type-state plus DropBomb for must-complete protocols - `92/100`

Why it matters:

- Some APIs are wrong if partially used.
- Type-state makes legal transitions explicit.
- A debug-time bomb catches “forgot to complete or abandon” programmer errors immediately.

Use when:

- parser markers, staged builders, transactions, or protocol objects must be finalized explicitly

Avoid when:

- the state machine is trivial and the extra type surface would overwhelm the gain

Source basis:

- rust-analyzer `Pattern 1.4`

## 2. Error Design And Diagnostics

### `RC2-12` `thiserror` in libraries, `anyhow` or `eyre` in binaries - `97/100`

Why it matters:

- Libraries need structured errors callers can inspect.
- Binaries need ergonomic propagation, context, and display.
- Mixing those two concerns produces mushy error boundaries.

Use when:

- writing public crates or internal libraries with machine-handled errors

Avoid when:

- you are tempted to export `anyhow::Error` from a library because it feels easier

Source basis:

- `S77` `A.6`, `A.29`, `A.31`
- ripgrep error handling `Pattern 2`

### `RC2-13` Opaque public errors with private kind representation - `95/100`

Why it matters:

- A public error struct wrapping a private or controlled kind representation keeps semver room while still allowing inspection.
- It prevents downstream construction of states your crate alone should own.

Use when:

- your library error space is public and likely to evolve

Avoid when:

- the error really is a fixed small enum with no realistic future growth

Source basis:

- ripgrep error handling `Pattern 1`

### `RC2-14` `#[non_exhaustive]` on public enums and structs that will grow - `92/100`

Why it matters:

- Future-proofing is not cosmetic. It is a real API reliability pattern.
- Many Rust crates accidentally promise “this will never grow” just by omitting the attribute.

Use when:

- the type is public and the domain obviously has room to evolve

Avoid when:

- the type is intentionally sealed and extra variants would change the semantics fundamentally

Source basis:

- ripgrep regex patterns `Pattern 1`
- `S77` `A.141`
- `S77` `A.174`

### `RC2-15` Actionable diagnostics with exact failure location and repair hint - `94/100`

Why it matters:

- Good error messages shorten debugging loops dramatically.
- Telling a user the byte offset and a valid workaround is more valuable than another abstract variant name.

Use when:

- parsing user-facing input, formats, flags, sizes, patterns, and configuration

Avoid when:

- the information is internal only and would confuse rather than help the caller

Source basis:

- ripgrep error handling `Patterns 6 and 7`

### `RC2-16` Binary-safe diagnostics for non-UTF-8 inputs and outputs - `87/100`

Why it matters:

- Paths and byte-oriented data often are not valid UTF-8.
- Converting too early to `String` either loses data or panics in the wrong place.

Use when:

- handling file paths, regex patterns, command stderr, protocol bytes, or search output

Avoid when:

- the domain guarantees valid UTF-8 and the proof is local and strong

Source basis:

- ripgrep regex patterns `Pattern 2`
- ripgrep printer patterns `Pattern 9`
- `S77` `A.3`

### `RC2-17` Error context layering at I/O and orchestration boundaries - `90/100`

Why it matters:

- `?` alone preserves propagation.
- It does not preserve enough local meaning unless you add context where the operation is still understandable.

Use when:

- opening files, issuing requests, decoding payloads, or crossing service boundaries

Avoid when:

- adding context at every tiny helper layer would only create noise and repeat the same message

Source basis:

- `S77` `A.29`
- `S77` `A.60`

### `RC2-18` `ExitCode` and error-chain inspection at CLI boundaries - `87/100`

Why it matters:

- CLIs need semantically correct exit behavior, not just “print error and die.”
- Broken pipes and other wrapped I/O failures often deserve special handling.
- Returning `ExitCode` also keeps destructors running.

Use when:

- writing binaries, command runners, and Unix-style tools

Avoid when:

- a library is trying to own process exit semantics from underneath its caller

Source basis:

- ripgrep main entry `Pattern 1`
- ripgrep CLI core patterns
- `S77` `A.112`

### `RC2-19` FFI unwind boundaries with `catch_unwind` and explicit contracts - `84/100`

Why it matters:

- Panics crossing FFI boundaries are not “just another error.”
- They can be undefined behavior or at least an integration hazard unless explicitly handled.

Use when:

- exporting or importing C-facing APIs or working across language boundaries

Avoid when:

- assuming the other side “probably handles it”

Source basis:

- `S77` `A.32`
- `S77` `A.143`

## 3. Async And Concurrency Reliability

### `RC2-20` No blocking and no locks across `.await` - `97/100`

Why it matters:

- This is a top-tier async correctness rule.
- Violating it causes latency spikes, deadlocks, starvation, and impossible-to-reason-about performance cliffs.

Use when:

- writing anything on Tokio or another async runtime

Avoid when:

- pretending a short synchronous call is harmless without proving it is non-blocking

Source basis:

- `S77` `A.11`

### `RC2-21` Bounded channels, backpressure, and cancellation tokens - `96/100`

Why it matters:

- Unbounded queues hide problems until memory pressure becomes the symptom.
- Backpressure makes throughput limits visible.
- Cancellation tokens make shutdown explicit and cooperative.

Use when:

- building pipelines, workers, fan-out systems, and long-running async services

Avoid when:

- using unbounded channels as a convenience default in high-volume paths

Source basis:

- `S77` `A.12`
- `S77` `A.74`
- ripgrep CLI patterns around coordinated stopping

### `RC2-22` Cancel-safe `tokio::select!` and future ownership discipline - `94/100`

Why it matters:

- A future dropped by `select!` loses its internal state immediately.
- If the partially consumed state lives inside the future itself, cancellation becomes data loss or protocol corruption.

Use when:

- multiplexing reads, writes, shutdown signals, timers, and work streams

Avoid when:

- branches own half-read buffers or half-advanced parsers internally

Source basis:

- `S77` `A.25`
- `S77` `A.52`

### `RC2-23` Structured concurrency with `JoinSet` and task scopes - `88/100`

Why it matters:

- Detached tasks are easy to start and hard to reason about.
- Structured concurrency makes ownership, shutdown, and error propagation visible again.

Use when:

- a parent operation owns a family of subtasks and must know when they finish or fail

Avoid when:

- spawning fire-and-forget background work with no plan for lifecycle or cancellation

Source basis:

- `S77` `A.76`
- `S77` `A.87`

### `RC2-24` Concurrency model checking with `loom` - `89/100`

Why it matters:

- Passing one test run proves almost nothing about concurrency.
- `loom` systematically explores interleavings for small models and catches races ordinary tests miss.

Use when:

- validating lock-free pieces, ordering-sensitive state machines, or subtle synchronization

Avoid when:

- trying to throw the whole production system into loom without carving out a focused model

Source basis:

- `S77` `A.28`
- `S77` `A.183`

### `RC2-25` Atomic ordering hygiene plus verification, not superstition - `85/100`

Why it matters:

- Many atomic bugs come from cargo-culting `SeqCst` everywhere or, worse, using `Relaxed` everywhere.
- Ordering should be justified by what memory relation is needed, then tested where possible.

Use when:

- atomics exist for more than a boolean flag or the ordering story is part of correctness

Avoid when:

- choosing orderings by folklore instead of by data and invariants

Source basis:

- `S77` `A.121`
- `S77` `A.139`

### `RC2-26` Thread naming, join discipline, and `move` closure hygiene - `83/100`

Why it matters:

- Systems get easier to debug when thread purpose is visible and thread lifecycle is explicit.
- `move` closures also make ownership across concurrency boundaries less magical.

Use when:

- spawning threads or coordinating background worker lifetimes

Avoid when:

- leaking threads or relying on implicit shutdown

Source basis:

- `S77` `A.43`
- `S77` `A.136`
- `S77` `A.130`

### `RC2-27` `OnceLock`, `LazyLock`, and one-time initialization patterns - `84/100`

Why it matters:

- Global initialization bugs are often hidden until startup races or double-init assumptions show up.
- The standard one-time primitives make intent explicit and remove hand-rolled synchronization.

Use when:

- expensive shared configuration, lookup tables, regexes, or lazily initialized global state exist

Avoid when:

- using global mutable state when plain ownership would have been simpler

Source basis:

- `S77` `A.133`
- ripgrep ignore patterns `Pattern 10`

## 4. Testing And Contract Verification

### `RC2-28` Doctests as executable contracts - `91/100`

Why it matters:

- Examples rot fast when they are not executable.
- Doctests turn API explanation into a regression boundary.

Use when:

- the example communicates expected behavior, not just syntax

Avoid when:

- examples are so large they belong in integration tests instead

Source basis:

- `S77` `A.16`
- `S77` `A.89`
- ripgrep searcher toplevel `Pattern 1`

### `RC2-29` Compile-fail UI tests for misuse contracts - `88/100`

Why it matters:

- If user misuse should fail with a specific compiler message or type error, that is part of the public contract.
- This is especially important for macros, builders, and type-level APIs.

Use when:

- diagnostics themselves are a deliverable

Avoid when:

- testing unstable compiler wording too precisely without a durable assertion strategy

Source basis:

- `S77` `A.17`

### `RC2-30` Property-based tests for invariants and round trips - `90/100`

Why it matters:

- Example-based tests often prove only that one manually chosen case works.
- Property tests expose invariant cracks across broad input space and shrink failures to something understandable.

Use when:

- serializers, parsers, formatters, normalizers, and algebraic invariants are central

Avoid when:

- the state space is tiny and examples already cover it completely

Source basis:

- `S77` `A.18`

### `RC2-31` Fuzzing for parser, decoder, and protocol frontiers - `88/100`

Why it matters:

- Untrusted input is where “works on my machine” stops mattering.
- Fuzzing complements properties by exploring malformed and chaotic inputs no human would write.

Use when:

- parsing bytes, external protocols, compressed data, or attacker-influenced input

Avoid when:

- failing to pair fuzzing with sanitizers, corpus hygiene, and timeout discipline

Source basis:

- `S77` `A.19`
- `S77` `A.154`

### `RC2-32` Snapshot tests with `expect-test` where textual behavior matters - `83/100`

Why it matters:

- Parsers, formatters, diagnostics, and macro expansions often have output whose whole shape matters.
- Snapshot tests are strong when output readability matters and changes should be reviewed holistically.

Use when:

- the textual surface is part of the contract

Avoid when:

- snapshots become lazy golden files that reviewers stop reading

Source basis:

- `S77` `A.71`

### `RC2-33` Test infrastructure should optimize failure readability and isolation - `84/100`

Why it matters:

- A test suite is only as fast as its failure diagnosis loop.
- Dedicated temp dirs, diff-friendly assertions, explicit command wrappers, and flaky-OS retries are not fluff. They are engineering leverage.

Use when:

- integration tests exercise real filesystems, commands, or multi-line outputs

Avoid when:

- debugging ergonomics are sacrificed because “the test already fails”

Source basis:

- ripgrep testing patterns `Patterns 1 through 6`

## 5. Tooling, Release Gates, And Operations

### `RC2-34` `cargo clippy -D warnings` as a correctness gate - `89/100`

Why it matters:

- Clippy is not just style policing. A meaningful subset of lints catches correctness, needless allocation, and API-footgun patterns early.
- Treating warnings as optional turns those findings into background noise.

Use when:

- maintaining serious library or application code

Avoid when:

- suppressing large lint classes without understanding why they fire

Source basis:

- `S77` `A.13`
- `S77` `A.114`

### `RC2-35` `rustfmt` as non-negotiable style infrastructure - `82/100`

Why it matters:

- Consistent formatting removes review churn and keeps discussions about logic rather than whitespace.
- It also makes generated diffs and blame easier to parse.

Use when:

- working in any shared codebase

Avoid when:

- hand-tuning style in ways that fight the formatter

Source basis:

- `S77` `A.14`
- `S77` `A.30`

### `RC2-36` MSRV policy and toolchain pinning - `84/100`

Why it matters:

- If your crate implicitly depends on “whatever latest stable has,” your compatibility story is accidental.
- Explicit MSRV and pinned toolchains reduce surprise breakage for contributors and downstream users.

Use when:

- shipping reusable crates or coordinating across CI and local environments

Avoid when:

- changing MSRV casually without documenting it

Source basis:

- `S77` `A.15`
- `S77` `A.148`

### `RC2-37` Supply-chain policy with `audit`, `deny`, or `vet` - `84/100`

Why it matters:

- Dependency risk is part of software correctness in the real world.
- Security advisories, license constraints, and yanked crates all affect whether code is safe to ship.

Use when:

- maintaining a dependency tree large enough that trust can no longer be informal

Avoid when:

- assuming Cargo.lock alone is a risk management strategy

Source basis:

- `S77` `A.21`

### `RC2-38` Structured observability with `tracing` - `88/100`

Why it matters:

- Async and concurrent systems become opaque fast.
- Spans, fields, and correlation IDs make failures reconstructable without turning logs into undifferentiated text noise.

Use when:

- async services, workers, or complex workflows need post-failure visibility

Avoid when:

- logging secrets or dumping whole payloads without filtering

Source basis:

- `S77` `A.22`

### `RC2-39` Secret handling with `secrecy` - `86/100`

Why it matters:

- Leaking secrets in logs or debug output is rarely caught by type-checking alone.
- Wrapping secret values in types that resist accidental display materially reduces risk.

Use when:

- handling API keys, tokens, passwords, and similar credentials

Avoid when:

- passing secrets around as plain `String` because “it’s just internal”

Source basis:

- `S77` `A.23`

### `RC2-40` SQLx compile-time query checking and pooling discipline - `85/100`

Why it matters:

- Database bugs often survive until runtime because strings and schemas drift apart.
- Compile-time verified queries plus disciplined pool usage cut a large failure surface.

Use when:

- SQL and schema validity are part of the core behavior

Avoid when:

- dynamic SQL generation obscures the benefits or holding pooled connections across unnecessary `.await`s

Source basis:

- `S77` `A.24`
- `S77` `A.127`

## 6. Performance Without Sacrificing Correctness

### `RC2-41` Panic-free arithmetic and slicing - `90/100`

Why it matters:

- External input plus unchecked indexing is one of the oldest ways to get boundary bugs.
- `checked_*`, `saturating_*`, and `.get()` are small tools with outsized reliability value.

Use when:

- ranges, offsets, terminators, suffix stripping, and parser cursor movement exist

Avoid when:

- relying on informal reasoning like “we already proved this above” in code that will change later

Source basis:

- ripgrep searcher toplevel `Pattern 5`
- related ripgrep matcher and lines patterns

### `RC2-42` Builder plus frozen config struct - `86/100`

Why it matters:

- Mutable configuration during construction and immutable configuration during execution are different phases.
- A builder plus a cheap, frozen config struct makes that split explicit and keeps runtime state simpler.

Use when:

- a component has many orthogonal knobs and a stable execution phase

Avoid when:

- the builder starts accumulating runtime state or expensive artifacts that should live elsewhere

Source basis:

- ripgrep builder patterns `Pattern 1`
- ripgrep regex config patterns

### `RC2-43` Accumulate-then-build and incremental error collection - `84/100`

Why it matters:

- Some systems need to collect a batch of declarations or patterns before an optimized representation can be built.
- Collecting non-fatal build-time errors separately from final construction keeps the API honest about what can be recovered and when.

Use when:

- building pattern sets, registries, ignore lists, or aggregated configuration objects

Avoid when:

- forcing eager construction on every incremental addition when the optimized shape only makes sense globally

Source basis:

- ripgrep builder patterns `Patterns 3 and 11`

### `RC2-44` Prepare-once query-many precomputation - `84/100`

Why it matters:

- Some of the best performance wins are not algorithm swaps but repeated-work elimination.
- Precompute normalized or derived views once, then reuse them across many checks.
- This improves both speed and local reasoning because the derivation becomes a named value type.

Use when:

- matching, routing, filtering, or repeated comparison against one input occurs

Avoid when:

- precomputation costs exceed reuse or hide freshness requirements

Source basis:

- ripgrep performance patterns `Pattern 9`
- ripgrep globset patterns around `Candidate`

## 7. Ownership, Borrowing, And Iterator Sharp Edges

### `RC2-45` Interior mutability versus synchronization primitives - `90/100`

Why it matters:

- `Cell` and `RefCell` solve single-thread logical mutability.
- `Mutex` and `RwLock` solve cross-thread synchronization.
- Mixing those mental models is a frequent source of needless locking or hidden runtime panics.

Use when:

- choosing between single-threaded interior mutability and multi-threaded shared state

Avoid when:

- using a mutex in single-threaded code just to quiet the borrow checker
- carrying `RefCell` or lock guards across async suspension points

Source basis:

- `S77` `A.8`
- `S77` `A.117`

### `RC2-46` Borrow-checker-friendly API design and two-phase borrow awareness - `88/100`

Why it matters:

- Many awkward Rust APIs are not hard because Rust is hard. They are hard because the API shape prolongs borrows unnecessarily.
- Smaller scopes, temporaries, and borrow-friendly method structure reduce E0502-class friction without cloning everything.

Use when:

- an API mixes read and write phases or nested `&mut self` calls

Avoid when:

- relying on subtle borrow-checker behavior that only works because of current compiler lowering rather than clear structure

Source basis:

- `S77` `A.81`
- `S77` `A.95`
- `S77` `A.109`
- `S77` `A.110`

### `RC2-47` Iterator contracts, fallible pipelines, and owned/borrowed iteration - `87/100`

Why it matters:

- Iterator APIs become much more reusable when they support owned and borrowed forms correctly and preserve error short-circuiting.
- `size_hint`, `ExactSizeIterator`, `try_fold`, `try_for_each`, and `FromIterator` are not trivia. They define what consumers can safely assume.

Use when:

- designing iterable custom types or error-aware pipelines

Avoid when:

- claiming exact iterator properties you cannot actually uphold
- manually unwrapping inside iterator chains that should short-circuit with `Result`

Source basis:

- `S77` `A.40`
- `S77` `A.50`
- `S77` `A.90`

### `RC2-48` Collection choice discipline - `85/100`

Why it matters:

- `Vec`, `VecDeque`, `HashMap`, `BTreeMap`, `IndexMap`, and `BinaryHeap` encode different operational promises.
- Defaulting to the wrong one creates performance or determinism bugs that feel “incidental” until they are everywhere.

Use when:

- choosing queue, ordered map, stable iteration, priority scheduling, or dense append-heavy storage

Avoid when:

- using `LinkedList` as a generic default
- assuming `HashMap` iteration order is meaningful

Source basis:

- `S77` `A.38`
- `S77` `A.39`
- `S77` `A.54`
- `S77` `A.57`

## 8. Trait, Async, And Macro Boundary Discipline

### `RC2-49` Pin, pin projection, and async trait boundaries - `86/100`

Why it matters:

- Pinned values carry movement invariants, and violations here are not normal logic bugs.
- Safe projection helpers and cautious async-trait boundaries prevent LLMs from cargo-culting `Pin::new_unchecked` into unsafe territory they do not actually control.

Use when:

- implementing custom futures, self-referential state machines, or pinned wrapper types

Avoid when:

- hand-projecting pinned fields without a proof story
- reaching for `async-trait` or AFIT without understanding object-safety and allocation tradeoffs

Source basis:

- `S77` `A.10`
- `S77` `A.80`
- `S77` `A.84`

### `RC2-50` HRTBs, GATs, and object-safety-aware trait design - `84/100`

Why it matters:

- Some APIs need callbacks over any lifetime, lending-style iteration, or precise object-safety boundaries.
- These tools are powerful, but only when used to model a real borrowing problem rather than to look sophisticated.

Use when:

- ordinary trait methods cannot express borrowed output or “for any lifetime” callback semantics cleanly

Avoid when:

- adding HRTBs or GATs where simple ownership would be clearer
- publishing traits that look dyn-friendly but are not actually object-safe

Source basis:

- `S77` `A.61`
- `S77` `A.62`
- `S77` `A.63`
- `S77` `A.79`
- `S77` `A.93`
- `S77` `A.94`

### `RC2-51` Feature flags additive, sealed traits, and coherence hygiene - `88/100`

Why it matters:

- Public API stability is not only about type shapes. It is also about whether features compose and whether downstream crates can safely implement your traits.
- Additive features and sealed/coherent extension points keep ecosystems from becoming semver traps.

Use when:

- stabilizing library APIs, feature matrices, and trait extension surfaces

Avoid when:

- creating mutually exclusive features
- exposing traits for external impls when you still need internal freedom to evolve them

Source basis:

- `S77` `A.47`
- `S77` `A.88`
- `S77` `A.113`
- `S77` `A.156`

### `RC2-52` Macro and proc-macro hygiene, spans, and diagnostics - `83/100`

Why it matters:

- Macro bugs are amplified bugs because they replicate bad behavior at every expansion site.
- Absolute paths, disciplined spans, and compile-time diagnostics keep macros from becoming opaque failure generators.

Use when:

- writing `macro_rules!`, derives, attribute macros, or function-like proc macros

Avoid when:

- panicking inside proc macros for user errors
- relying on local imports or implicit hygiene assumptions in expansions

Source basis:

- `S77` `A.77`
- `S77` `A.101`
- `S77` `A.102`
- `S77` `A.103`
- `S77` `A.104`
- `S77` `A.150`

## 9. Unsafe And FFI Containment

### `RC2-53` Unsafe encapsulation with minimal surface, SAFETY docs, and Miri - `95/100`

Why it matters:

- Unsafe Rust is not wrong by default, but undocumented unsafe is.
- The safest pattern is not “avoid unsafe forever.” It is: keep unsafe small, private where possible, justified with `SAFETY` comments, and tested with tools like Miri.

Use when:

- raw pointers, FFI, aliasing-sensitive optimizations, or manual memory invariants are truly required

Avoid when:

- spreading unsafe across wide public surfaces without a single owned invariant boundary

Source basis:

- `S77` `A.86`
- `S77` `A.91`

### `RC2-54` MaybeUninit, aliasing invariants, and RAII resource guards - `89/100`

Why it matters:

- Manual initialization and raw-pointer writes are where memory-safety mistakes cluster.
- The reliable path is to separate “uninitialized but allocated,” “safely initialized,” and “resource must be released” into explicit phases and guards.

Use when:

- performance or FFI requires manual initialization or manual resource acquisition

Avoid when:

- creating references to uninitialized memory
- forgetting that `&mut` implies uniqueness and that RAII should end cleanup ambiguity

Source basis:

- `S77` `A.83`
- `S77` `A.85`
- `S77` `A.97`
- `S77` `A.111`

### `RC2-55` FFI layout, DST/wide pointers, and explicit nullability - `88/100`

Why it matters:

- Rust-native reference shapes like `&str`, `&[u8]`, and `&dyn Trait` are not FFI-safe contracts.
- Explicit `repr` choices, `(ptr, len)` structs, and `Option<NonNull<T>>` style nullability keep ABI assumptions honest.

Use when:

- crossing C or other language boundaries, defining wire-adjacent layouts, or documenting layout-sensitive invariants

Avoid when:

- exporting Rust fat pointers directly over FFI
- assuming default Rust layout is stable enough for external consumers

Source basis:

- `S77` `A.59`
- `S77` `A.67`
- `S77` `A.140`
- `S77` `A.143`
- `S77` `A.172`

### `RC2-56` NonZero niches, ZSTs, and uninhabited types - `83/100`

- Use `NonZero*` and `Option<NonZero*>` when the niche optimization is real and semantically clean.
- Use ZST marker types and `PhantomData` when compile-time state should cost nothing at runtime.
- Use uninhabited types to encode impossible states rather than inventing sentinel values.

Use when:

- modeling impossible states, nullable handles, and zero-cost markers

Avoid when:

- using these tricks merely because the layout looks clever instead of because the model is clearer

Source basis:

- `S77` `A.99`
- `S77` `A.171`
- `S77` `A.173`

## 10. Diagnostics, Coverage, And Release Discipline

### `RC2-57` Source-annotated diagnostics with miette and color-eyre - `85/100`

Why it matters:

- Some failures are best explained with context snippets and labeled spans, not plain strings.
- Rich diagnostics are especially valuable in binaries and CLIs, where the developer is also the user of the feedback loop.

Use when:

- end-user parse or config errors need precise, human-friendly reports

Avoid when:

- leaking rich diagnostic types into library APIs that callers need to keep lightweight and stable

Source basis:

- `S77` `A.27`
- `S77` `A.135`

### `RC2-58` Coverage as a gate with cargo-llvm-cov - `84/100`

Why it matters:

- Coverage is imperfect, but for critical modules it is a useful regression gate when used thoughtfully.
- Branch coverage especially helps where line coverage gives false confidence.

Use when:

- critical modules need a floor against silent testing regressions

Avoid when:

- chasing arbitrary 100 percent numbers instead of meaningful test depth

Source basis:

- `S77` `A.20`
- `S77` `A.114`
- `S77` `A.153`

### `RC2-59` Channel and synchronization choice matrix - `84/100`

Why it matters:

- Concurrency bugs often begin one decision earlier than expected, at primitive selection time.
- `std::mpsc`, `crossbeam-channel`, `flume`, Tokio channels, `Mutex`, `RwLock`, and `parking_lot` are not interchangeable defaults.

Use when:

- choosing thread or task coordination primitives and lock behavior

Avoid when:

- mixing channel ecosystems casually
- using the default std channel where MPMC, select, deadlines, or rendezvous semantics are actually needed

Source basis:

- `S77` `A.117`
- `S77` `A.184`

### `RC2-60` Workspace, toolchain, and additive-feature discipline - `85/100`

Why it matters:

- Large Rust codebases are stabilized as much by workspace and toolchain discipline as by code style.
- Additive features, pinned toolchains, and clear workspace layering make LLM-authored changes less likely to introduce accidental build drift.

Use when:

- maintaining multi-crate repos or public crates with evolving feature surfaces

Avoid when:

- letting workspace config become implicit tribal knowledge

Source basis:

- `S77` `A.88`
- `S77` `A.148`
- `S77` `A.149`

## 11. How An LLM Should Use This File

An LLM should not treat this as a buffet.

It should use it as a decision ladder:

1. classify the code being written:
   library, binary, async service, FFI wrapper, proc-macro, or low-level systems component
2. choose the ownership surface:
   borrowing, owned values, shared ownership, or type-state
3. choose the error surface:
   typed library error, app-level aggregation, or rich binary diagnostics
4. choose the concurrency surface:
   sync, async, channels, cancellation, or structured task ownership
5. choose the public API stability surface:
   `#[non_exhaustive]`, additive features, sealed traits, coherence-safe extension points
6. choose the verification mix:
   unit, doctest, compile-fail, property, fuzz, loom, coverage gates
7. if unsafe remains necessary:
   minimize the unsafe block, document the invariant, add Miri or equivalent verification

The core meta-rule is:

> Reach for the simplest pattern that preserves correctness boundaries, not the most impressive pattern in the file.

## High-Value Anti-Patterns

Reject these by default:

- public APIs that accept `&Vec<T>` or `&String`
- library APIs that return `anyhow::Error`
- public enums likely to grow but missing `#[non_exhaustive]`
- `unwrap()` or `expect()` in production paths without an explicit invariant story
- blocking work directly on async executors
- holding locks or mutable borrows across `.await`
- unbounded channels in high-throughput paths
- textual diagnostics that discard exact failure location
- byte-processing code that assumes everything is UTF-8
- detached tasks with no lifecycle ownership
- tests that only cover examples but not invariants
- concurrency code that has never been modeled or stress-tested

## Patterns I Intentionally Did Not Elevate

Some source patterns are powerful but were not promoted to the top tier here because they are either niche or too easy to cargo-cult badly:

- allocator overrides as a general default
- `ManuallyDrop` for compile-time or micro-optimization purposes
- specialized proc-macro internals beyond hygiene, spans, and diagnostics
- lock-free building blocks without a verification plan
- advanced pinning tricks beyond safe projection and clear containment
- specialization-related designs in public crates

These are not bad. They are just not the first reliability bets for most teams.

## Default Reliability Stack

If you do not know what to prioritize, start here:

1. model domain invariants with newtypes, enums, and explicit conversions
2. keep library errors typed and application errors contextual
3. make async code cancel-safe, bounded, and non-blocking
4. remove panic edges from parsing, slicing, and boundary arithmetic
5. turn examples, invariants, and misuse cases into tests
6. make operational visibility and secret discipline part of the design

## A Better Thesis For rust-coder-02

The best updated Rust guide is not simply longer than `rust-coder-01`.

It is better because it is more exact about where correctness is won or lost:

- in public API commitments
- in error boundary design
- in async cancellation and concurrency semantics
- in executable verification
- in operational reality after code leaves the editor

That is the real bet here.
