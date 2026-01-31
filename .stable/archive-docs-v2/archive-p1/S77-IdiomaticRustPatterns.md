========================================
IDIOMATIC RUST PATTERNS
========================================


A. Curated Idioms (Deep Dives)
------------------------------

A.1 Expression-Oriented Code
- Use when: a function naturally computes a value; prefer returning the last expression without a trailing semicolon.
- Context: prefer `if/else` and `match` as expressions; use `let` bindings for clarity; reduce mutable state.
- Avoid/Anti-pattern: unnecessary `mut` accumulators when `map/fold` suffice; stray semicolons converting expressions to `()`.

A.2 Accept Slices and Traits in Public APIs
- Use when: callers can pass many input types; prefer `&[T]`, `&str`, `AsRef<T>`, `Borrow<T>` over concrete containers.
- Context: stable, flexible APIs; avoid needless cloning, enable zero-cost conversion.
- Avoid/Anti-pattern: taking `&Vec<T>`/`&String` in signatures; over-generic bounds that hurt inference.

```rust path=null start=null
fn process<S: AsRef<str>>(s: S) {
    let s: &str = s.as_ref();
    // ...
}
```

A.3 Clone-on-Write (Cow<'_, str/[u8]>)
- Use when: sometimes borrow, sometimes need owned/modified data; avoid eager clones on hot paths.
- Context: config, cache keys, deserialized data that is usually borrowed.
- Avoid/Anti-pattern: using `Cow` when data is always owned or always borrowed; hidden `.to_string()` in tight loops.

A.4 From/TryFrom/TryInto for Conversions
- Use when: modeling infallible/fallible conversions instead of ad-hoc constructors.
- Context: ergonomic API surfaces; composable conversions in pipelines.
- Avoid/Anti-pattern: exposing `Result` for infallible cases; implementing for foreign types (orphan rule) instead of a newtype.

A.5 Newtype Pattern (+ repr(transparent) for FFI)
- Use when: enforce domain invariants and give semantic meaning beyond primitives.
- Context: type-safe IDs, units, opaque handles, and FFI boundaries.
- Avoid/Anti-pattern: `type` alias where a distinct type is needed; forgetting `#[repr(transparent)]` for FFI.

```rust path=null start=null
#[repr(transparent)]
struct UserId(String);
impl UserId { fn parse(s: &str) -> Option<Self> { (!s.is_empty()).then(|| Self(s.to_owned())) } }
```

A.6 Error Boundaries: thiserror (libs) vs anyhow (apps)
- Use when: libraries define precise `Error` enums; applications aggregate with context.
- Context: `?` with `.context()` / `.with_context()`; stable public APIs.
- Avoid/Anti-pattern: exposing `anyhow::Error` in public APIs; panicking on recoverable errors.

A.7 Option/Result Combinators over Nested Matching
- Use when: simple transformations/chaining suffice (`map`, `and_then`, `transpose`).
- Context: linear flows; interop between `Option` and `Result`.
- Avoid/Anti-pattern: `.unwrap()`/`.expect()` in production; deep nested matches.

A.8 Interior Mutability vs Synchronization Primitives
- Use when: `Cell/RefCell` for single-thread logical mutability; `Mutex/RwLock` for cross-thread.
- Context: minimize lock scope; short-lived borrows.
- Avoid/Anti-pattern: holding `RefCell` borrows or locks across `.await`; mutex in single-threaded code.

A.9 Trait Objects vs Generics
- Use when: heterogenous collections or dynamic dispatch at API boundaries (`Box<dyn Trait>`).
- Context: use generics for hot paths; trait objects for flexibility and reduced code size.
- Avoid/Anti-pattern: trait objects in tight loops; over-generic public APIs causing monomorphization bloat.

A.10 Pin/Unpin and Self-Referential Types
- Use when: implementing `Future`/state machines that must not move after pinning.
- Context: use `Pin` smartly; maintain invariants; prefer safe APIs that encapsulate pin.
- Avoid/Anti-pattern: `Pin::new_unchecked` without proof; moving pinned fields.

A.11 Async Hygiene: No Blocking and No Locks Across await
- Use when: CPU-bound work inside async contexts (`spawn_blocking`) and short critical sections.
- Context: avoid scheduler starvation; minimize `.await` inside critical regions.
- Avoid/Anti-pattern: blocking I/O/CPU directly on the runtime; holding locks across `.await`.

```rust path=null start=null
let data = Arc::new(Mutex::new(vec![]));
// Anti-pattern:
// let mut guard = data.lock().unwrap();
// async_call().await; // lock held across await
// Correct:
{
    let mut guard = data.lock().unwrap();
    guard.push(1);
}
async_call().await;
```

A.12 Bounded Channels, Backpressure, and Cancellation
- Use when: limit memory growth and propagate shutdown.
- Context: `tokio::sync::mpsc` with capacity; `tokio_util::sync::CancellationToken` for cooperative cancellation.
- Avoid/Anti-pattern: unbounded channels in high-throughput systems; ad-hoc boolean flags for cancellation.

```rust path=null start=null
use tokio::{sync::mpsc, time::{timeout, Duration}};
let (tx, mut rx) = mpsc::channel(1024);
let send = async move { tx.send(item).await.ok(); };
let recv = async move { while let Some(v) = rx.recv().await { /* ... */ } };
let _ = timeout(Duration::from_secs(1), recv).await; // apply backpressure/timeouts
```

A.13 Clippy as a Gate (-D warnings)
- Use when: enforce correctness and idioms across all targets and features in CI.
- Context: run locally and in CI; gate merges on a clean clippy run.
- Avoid/Anti-pattern: allowing warnings in CI; running clippy on a subset of targets only.

```sh path=null start=null
cargo clippy --all-targets --all-features -- -D warnings
```

A.14 rustfmt as Non-negotiable Style
- Use when: enforce consistent formatting across the workspace.
- Context: run `cargo fmt --check` in CI; auto-format locally.
- Avoid/Anti-pattern: hand-tuned formatting; ignoring fmt drift.

```sh path=null start=null
cargo fmt --all -- --check
```

A.15 MSRV Policy (rust-version in Cargo.toml)
- Use when: communicating minimum supported Rust version to users and CI.
- Context: set `package.rust-version`; test on that toolchain; document policy.
- Avoid/Anti-pattern: breaking MSRV silently; relying on nightly-only features in libraries.

```toml path=null start=null
[package]
rust-version = "1.76"
```

A.16 Doctests as Executable Contracts
- Use when: examples express API semantics and invariants.
- Context: small, focused examples; run with `cargo test`.
- Avoid/Anti-pattern: examples that don't compile or assert nothing.

```rust path=null start=null
/// Adds two numbers.
/// ```
/// assert_eq!(my_crate::add(2, 2), 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 { a + b }
```

A.17 Compile-fail UI Tests (trybuild)
- Use when: macros and type errors are part of your public contract.
- Context: maintain `tests/ui/*.rs` cases; assert specific diagnostics.
- Avoid/Anti-pattern: relying on unstable error messages; coupling to nightly-only spans.

```rust path=null start=null
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
```

A.18 Property-based Testing (proptest)
- Use when: invariants must hold across large input spaces.
- Context: seed failures for reproducibility; shrink to minimal counterexamples.
- Avoid/Anti-pattern: only example-based tests for complex parsers/serializers.

```rust path=null start=null
use proptest::prelude::*;
proptest! {
  #[test]
  fn roundtrip(x in any::<u32>()) {
    let enc = encode(x);
    prop_assert_eq!(decode(&enc).unwrap(), x);
  }
}
```

A.19 Fuzz Frontiers (cargo-fuzz/libFuzzer)
- Use when: parsing/untrusted input, protocol boundaries, decoders.
- Context: minimal harness; CI nightly jobs; keep corpus and minimize.
- Avoid/Anti-pattern: running fuzzers without ASan/UBSan; ignoring timeouts.

```rust path=null start=null
#![no_main]
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| { let _ = my_crate::parse(data); });
```

A.20 Coverage as a Gate (cargo-llvm-cov)
- Use when: prevent coverage regressions on critical modules.
- Context: gate on line/branch coverage thresholds; exclude generated code.
- Avoid/Anti-pattern: chasing 100% blindly; ignoring branch coverage.

```sh path=null start=null
cargo llvm-cov --workspace --branch --fail-under-lines 75
```

A.21 Supply-chain Policies (cargo-audit/deny/vet)
- Use when: enforce vuln and license policies before merge/release.
- Context: nightly job for advisory DB; strict allowlists.
- Avoid/Anti-pattern: pinning yanked/vulnerable crates without justification.

```toml path=null start=null
# deny.toml
[licenses]
allow = ["MIT", "Apache-2.0"]

[advisories]
db-path = "~/.cargo/advisory-db"
```

A.22 Structured Observability (tracing)
- Use when: correlate logs, metrics, and spans in async services.
- Context: `#[instrument]` on boundaries; propagate span fields (trace_id, user_id).
- Avoid/Anti-pattern: println!-style logs in libraries; logging secrets.

```rust path=null start=null
#[tracing::instrument(skip(req))]
pub async fn handle(req: Request) -> Result<Response, Error> {
    tracing::info!(method = %req.method(), path = %req.uri().path());
    // ...
    Ok(resp)
}
```

A.23 Secret Handling (secrecy)
- Use when: hold API keys/passwords safely in memory/logs.
- Context: use `SecretString`/`SecretVec`; avoid Display/Debug.
- Avoid/Anti-pattern: formatting secrets into logs; cloning secrets widely.

```rust path=null start=null
use secrecy::{ExposeSecret, SecretString};
let api_key = SecretString::new(load_key());
// tracing::info!(key = %api_key); // anti-pattern
tracing::info!(has_key = api_key.expose_secret().len() > 0);
```

A.24 SQLx Compile-time Queries and Pooling
- Use when: ensure query validity at build time and safe pooling.
- Context: `query!`/`query_as!` with offline mode; timeouts and max connections.
- Avoid/Anti-pattern: building SQL strings dynamically; holding connections across `.await` unnecessarily.

```rust path=null start=null
#[derive(sqlx::FromRow)]
struct User { id: i64, name: String }
let row = sqlx::query_as!(User, "SELECT id, name FROM users WHERE id = $1", id)
    .fetch_one(&pool).await?;
```

A.25 Cancellation-safe select! and Future design
- Use when: multiplexing operations with `tokio::select!`.
- Context: futures in `select!` should not own partially-consumed state; borrow from a shared buffer or encapsulate state in an owner outside the futures.
- Avoid/Anti-pattern: futures that drop with half-consumed state; mixing blocking I/O in selected branches.

```rust path=null start=null
use tokio::select;
async fn run(mut stream: BufferedStream) {
    loop {
        select! {
            biased;
            maybe = stream.next() => { if maybe.is_none() { break; } },
            _ = shutdown_token.cancelled() => { break; }
        }
    }
}
```

A.26 Panic Strategy: abort in release binaries
- Use when: CLIs/embedded where unwind is undesirable; minimize binary size and fail-fast.
- Context: keep unwind in debug for better backtraces; never expose panic behavior from libraries.
- Avoid/Anti-pattern: setting `panic = "abort"` in libraries; aborting in tests.

```toml path=null start=null
[profile.release]
panic = "abort"
```

A.27 Rich Diagnostics for Applications (color-eyre/miette)
- Use when: human-friendly error reports and panics are required.
- Context: install a global reporter on startup; prefer `eyre::Result` in bin targets only.
- Avoid/Anti-pattern: using these types in public library APIs.

```rust path=null start=null
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // ...
    Ok(())
}
```

A.28 Concurrency Model Testing with loom
- Use when: validating Send/Sync invariants and race-freedom.
- Context: model small critical sections; assert invariants across all interleavings.
- Avoid/Anti-pattern: testing production code paths wholesale; keep models minimal.

```rust path=null start=null
use loom::{sync::Mutex, thread};
loom::model(|| {
    let m = std::sync::Arc::new(Mutex::new(0));
    let m2 = m.clone();
    let t = thread::spawn(move || { *m2.lock().unwrap() += 1; });
    *m.lock().unwrap() += 1;
    t.join().unwrap();
    assert_eq!(*m.lock().unwrap(), 2);
});
```

A.29 Result Ergonomics: context and mapping
- Use when: propagate errors with additional context.
- Context: libraries: `thiserror` + `#[from]` + `map_err`; apps: `anyhow::Context`.
- Avoid/Anti-pattern: `.unwrap()` in non-test code; swallowing sources.

```rust path=null start=null
use anyhow::Context;
fn load(path: &str) -> anyhow::Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("reading {}", path))
}
```

A.30 rustfmt configuration as code
- Use when: enforce project-wide style to avoid review churn.
- Context: commit `rustfmt.toml` and gate with `cargo fmt --check`.
- Avoid/Anti-pattern: per-contributor styles leading to unstable diffs.

```toml path=null start=null
edition = "2024"
max_width = 100
group_imports = "StdExternalCrate"
```

A.31 Library Error Design: thiserror vs snafu vs eyre/anyhow
- Use when: choosing error modeling for libraries vs applications.
- Context: libraries prefer typed enums (`thiserror`) or context-rich enums (`snafu`) when callsites need structured handling; binaries can use `anyhow`/`eyre` for ergonomic aggregation.
- Avoid/Anti-pattern: returning `anyhow::Error` from library public APIs; losing source error context.

```rust path=null start=null
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
  #[error("invalid header: {0}")]
  InvalidHeader(String),
  #[error(transparent)]
  Io(#[from] std::io::Error),
}
```

A.32 FFI Unwind Safety and extern "C-unwind"
- Use when: calling into or being called from C/C++/other languages where exceptions/longjmp may cross boundaries.
- Context: mark Rust FFI with `extern "C"`; never let Rust panics cross FFI; catch with `std::panic::catch_unwind` and convert to error codes. Consider `extern "C-unwind"` where appropriate.
- Avoid/Anti-pattern: unwinding across FFI without explicit contracts; relying on UB-prone longjmp behavior.

```rust path=null start=null
#[no_mangle]
pub extern "C" fn api_do(ptr: *const u8, len: usize) -> i32 {
  let res = std::panic::catch_unwind(|| unsafe { do_work(std::slice::from_raw_parts(ptr, len)) });
  match res { Ok(Ok(())) => 0, Ok(Err(_e)) => -2, Err(_panic) => -1 }
}
```

A.33 IntoIterator/iter/iter_mut Semantics
- Use when: designing APIs that accept collections generically or implementing iteration for your types.
- Context: prefer bounds like `impl<T: IntoIterator<Item = X>>` for flexibility; use `iter()/iter_mut()` when you need borrow semantics; implement `IntoIterator` for owned iteration of custom collections.
- Avoid/Anti-pattern: surprising ownership moves via `into_iter()` on owned containers when a borrow suffices; ambiguous trait bounds hurting inference.

A.34 Iterator Laziness and Adapter vs Consumer Separation
- Use when: building pipelines; compose with adapters (`map/filter/take`) and drive with consumers (`collect/sum/for_each`).
- Context: prefer iterator combinators over manual loops for clarity and fusion; be mindful of allocation in `collect`.
- Avoid/Anti-pattern: building intermediate `Vec`s unnecessarily; relying on side-effects in adapters.

A.35 Choosing Fn/FnMut/FnOnce Bounds
- Use when: accepting closures; pick the narrowest bound that fits call semantics.
- Context: `Fn` for repeated read-only calls (often Send/Sync friendly); `FnMut` when mutation is required; `FnOnce` when the closure may consume captures.
- Avoid/Anti-pattern: overconstraining to `Fn` when `FnOnce` is needed; accidentally requiring `mut` when not needed.

A.36 DoubleEndedIterator and FusedIterator
- Use when: bi-directional traversal and predictable exhaustion semantics matter.
- Context: implement `DoubleEndedIterator` when reverse iteration is natural; use `.fuse()` or rely on `FusedIterator` for repeated-None guarantees.
- Avoid/Anti-pattern: assuming fused behavior without `.fuse()`; incorrectly implementing `next_back` semantics.

A.37 FromIterator and Collect Patterns
- Use when: constructing collections from iterators; offer `FromIterator` for custom containers.
- Context: `collect::<Vec<_>>()` for materialization; `FromIterator` for explicit types; streaming transforms with minimal allocations.
- Avoid/Anti-pattern: collecting when a lazy pipeline suffices; turbofish misuse obscuring types.

A.38 Vec vs VecDeque vs LinkedList
- Use when: choosing a sequence container.
- Context: default to `Vec` for most cases; `VecDeque` for efficient queue/deque semantics; avoid `LinkedList` unless specific O(1) splice patterns dominate and cache effects are acceptable.
- Avoid/Anti-pattern: using `LinkedList` for general workloads; premature `VecDeque` when simple front-removals can be amortized differently.

A.39 HashMap vs BTreeMap
- Use when: key-value storage; choose `HashMap` for average-case O(1), `BTreeMap` for ordering and predictable iteration.
- Context: determinism requirements, range queries, memory locality; consider worst-case hash DoS and feature `hashbrown`/fxhash only with care in non-adversarial contexts.
- Avoid/Anti-pattern: relying on hash iteration order; using BTreeMap for hot random-access without ordering needs.

A.40 Implement IntoIterator for Owned and Borrowed
- Use when: custom collections should work with `for` over owned and borrowed forms.
- Context: implement `IntoIterator` for `T` and for `&T`/`&mut T` to support all use-sites.
- Avoid/Anti-pattern: only implementing owned iteration, breaking `for &item in &collection` ergonomics.

```rust path=null start=null
#[derive(Debug)]
struct MyCollection(Vec<i32>);
impl IntoIterator for MyCollection {
  type Item = i32;
  type IntoIter = std::vec::IntoIter<i32>;
  fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
impl<'a> IntoIterator for &'a MyCollection {
  type Item = &'a i32;
  type IntoIter = std::slice::Iter<'a, i32>;
  fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}
```

A.41 Return impl Iterator from APIs
- Use when: expose lazy pipelines without committing to concrete adaptor types.
- Context: stable, efficient APIs that can be optimized by the compiler; document when evaluation occurs.
- Avoid/Anti-pattern: returning concrete iterator types in public APIs; boxing iterators on hot paths.

A.42 Fused Iteration and Reversal
- Use when: consumers may call `next()` after exhaustion; add `.fuse()` to guarantee repeated-None semantics.
- Context: use `.rev()` when `DoubleEndedIterator` is available; implement `next_back` correctly for custom iterators.
- Avoid/Anti-pattern: assuming fusion without `.fuse()`; mixing forward/backward iteration state incorrectly.

A.43 Closure Capture and move in Concurrency
- Use when: spawning threads/tasks; prefer `move` closures; ensure captured values are `Send + 'static` where required.
- Context: choose `Fn`/`FnMut`/`FnOnce` bounds precisely; prefer immutable captures for `Fn` when possible.
- Avoid/Anti-pattern: capturing non-`Send` across thread boundaries; overusing `Arc<Mutex<_>>` when channels suffice.

A.44 Arrays, Slices, and IntoIterator Nuances
- Use when: writing generic iteration; prefer `&[T]` bounds and `iter()` for clarity; be aware of array `IntoIterator` semantics on modern Rust.
- Context: avoid surprises from `into_iter()` moving owned arrays; use `iter_mut()` for in-place transforms.
- Avoid/Anti-pattern: ambiguous `IntoIterator` bounds that hinder inference; relying on edition-specific behavior.

A.45 Raw Identifiers to Bridge Editions
- Use when: identifiers conflict with keywords across editions (`r#try`, `r#match`).
- Context: interop with older crates or generated code.
- Avoid/Anti-pattern: renaming public symbols that must stay stable for interop without using raw identifiers.

A.46 Trait Object Upcasting (dyn upcast)
- Use when: upcast `dyn SubTrait` to `dyn SuperTrait` where stable; simplifies API layering without custom `as_super` shims.
- Context: prefer static dispatch in hot loops; use trait objects for dynamic plugin points.
- Avoid/Anti-pattern: constructing invalid vtables in unsafe; leaking raw trait-object pointers.

A.47 Coherence and Orphan Rule Hygiene
- Use when: designing trait extension points; ensure either the trait or the self type is local.
- Context: provide extension traits for foreign types; avoid overlapping impls.
- Avoid/Anti-pattern: blanket impls that would conflict downstream; violating E0117/E0210 orphan rules.

A.48 Specialization Status and min_specialization
- Use when: constrained cases in std or internal crates; avoid in general libraries until fully stabilized.
- Context: prefer explicit trait design over specialization; document semantics if used under feature flags.
- Avoid/Anti-pattern: relying on unsound specialization behavior in public crates.

A.49 Fn/FnMut/FnOnce Bound Selection Patterns
- Use when: picking callable trait bounds; mirror call semantics (read-only, mutable, consuming).
- Context: function pointers implement all three; prefer `Fn` for concurrency-friendly code.
- Avoid/Anti-pattern: over-constraining to `Fn` when mutation is required; capturing by move accidentally when borrowing suffices.

A.50 Iterator Contracts: size_hint and ExactSizeIterator
- Use when: consumers can benefit from pre-allocation or exact length.
- Context: implement `ExactSizeIterator` only when length is truly exact; expose meaningful `size_hint`.
- Avoid/Anti-pattern: relying on `size_hint` correctness in unsafe code; claiming `ExactSizeIterator` without invariants.

A.51 Panic Strategy per Profile
- Use when: balancing diagnostics (unwind) vs size/perf (abort) per binary profile.
- Context: `panic = 'abort'` for release CLIs; `unwind` for tests/debug; be explicit in `Cargo.toml`.
- Avoid/Anti-pattern: panics crossing FFI; assuming platform supports unwinding.

A.52 Cancel-Safe select! Patterns
- Use when: designing async multiplexing; ensure losing branches don’t own partial state.
- Context: encapsulate state in owner types (channels/buffers); use borrowed futures in `select!` arms.
- Avoid/Anti-pattern: storing partial buffers inside futures that may be dropped.

A.53 Collection Capacity Planning
- Use when: heavy pushes known; prefer `with_capacity` and amortized growth.
- Context: understand doubling strategies; avoid frequent reallocations in tight loops.
- Avoid/Anti-pattern: per-push reallocations; ignoring memory locality vs LinkedList.

A.54 BinaryHeap and Priority Queues
- Use when: need top-k, scheduling, or priority work queues.
- Context: `BinaryHeap` from Vec in O(n); beware ascending push pathological cases.
- Avoid/Anti-pattern: using `BTreeMap` to emulate a heap for max operations.

A.55 SSO and SmallVec Trade-offs
- Use when: small fixed upper bound per hot path; reduce heap traffic.
- Context: prefer `SmallVec<[T; N]>` with profiling; watch for spill costs.
- Avoid/Anti-pattern: assuming SSO always wins; ignoring spill hotspots like `spilled()`.

A.56 HashMap Implementation and Load Factors
- Use when: tuning map performance; understand SwissTable/hashbrown behavior.
- Context: expected O(1) with SIMD probing; tune hasher (aHash) only in non-adversarial contexts.
- Avoid/Anti-pattern: custom hashers in hostile inputs; mutating keys in-place.

A.57 IndexMap for Stable Insertion Order
- Use when: deterministic iteration order is required.
- Context: iteration over dense storage; removal semantics affecting order.
- Avoid/Anti-pattern: assuming order preservation after removals without `shift_remove`.

A.58 Pattern Matching: Guards and Ergonomics (2024)
- Use when: refine matches with guards; be explicit under Rust 2024 ergonomics.
- Context: avoid relying on implicit ref/mut; use `cargo fix --edition` for migrations.
- Avoid/Anti-pattern: ambiguous patterns relying on old match ergonomics.

A.59 FFI Layout and repr Choices
- Use when: interop and ABI stability required; choose `repr(C)`/`repr(transparent)` appropriately.
- Context: `Option<non-null>` for nullable pointer optimizations across FFI.
- Avoid/Anti-pattern: assuming Rust default layout across FFI; mixing unwinding across FFI.

A.60 Error Layering Revisited: ? and Context
- Use when: propagate errors ergonomically.
- Context: favor `?` and combinators; attach context at boundaries.
- Avoid/Anti-pattern: deep nested matches; panicking for recoverable errors.

A.61 HRTBs for Callback APIs (for<'a>)
- Use when: generic callbacks borrow data with any lifetime.
- Context: `for<'a> Fn(&'a T) -> U` in trait bounds; avoid leaking concrete lifetimes.
- Avoid/Anti-pattern: requiring 'static unnecessarily; encoding lifetimes as type params when HRTB suffices.

A.62 GATs for Streaming and Lending Patterns
- Use when: associated types depend on lifetimes (e.g., iterator yielding borrowed data).
- Context: use GATs to model lending iterators or parser cursors.
- Avoid/Anti-pattern: forcing owned allocations because associated outputs can’t borrow.

A.63 Dyn Compatibility and Object Safety
- Use when: designing traits for dynamic dispatch.
- Context: avoid methods using `Self` in returns or generics; ensure receivers are object-safe (`&self`, `&mut self`, `Box<Self>`, etc.).
- Avoid/Anti-pattern: non-dyn-compatible APIs in public traits; surprise `impl Trait` in returns.

A.64 Never Type (!) and Diverging APIs
- Use when: functions that never return (process abort, loop forever) or exhaustive matches.
- Context: use `!` for type-level modeling; improves control-flow typing.
- Avoid/Anti-pattern: encoding never-return via panics in library code without documenting.

A.65 Raw Identifiers and Edition Interop
- Use when: cross-edition compatibility requires using keywords as identifiers.
- Context: prefer `r#ident` to avoid breaking API surfaces.
- Avoid/Anti-pattern: renaming public items and causing churn when `r#` solves it.

A.66 Trait Bounds via where for Readability
- Use when: long bounds clutter signatures; prefer `where` for clarity.
- Context: move complex bounds off the main signature.
- Avoid/Anti-pattern: unreadable nested bounds inline.

A.67 Nullable Pointer Optimization in FFI
- Use when: model nullable pointers using `Option<NonNull<T>>` or `Option<extern "C" fn()>`.
- Context: leverage guaranteed representation for `Option<non-null>`.
- Avoid/Anti-pattern: using magic sentinel integers; UB with invalid values.

A.68 Deterministic APIs with IndexMap
- Use when: stable iteration order matters (caches, diffs, tests).
- Context: choose `IndexMap` for dense iteration; document order semantics.
- Avoid/Anti-pattern: relying on HashMap’s nondeterministic order.

A.69 BinaryHeap Top-k and Scheduling
- Use when: maintain rolling top-k or priority scheduling.
- Context: build from Vec in O(n); pop/push O(log n).
- Avoid/Anti-pattern: full sort when only top-k required.

A.70 Async Polling Semantics and Waker
- Use when: building executors or custom futures.
- Context: respect `Poll::Pending` contract; wake with `Waker` when progress possible.
- Avoid/Anti-pattern: busy-loop polling; waking without state change.

A.71 Snapshot Testing with expect-test
- Use when: you want robust, readable golden tests that update intentionally.
- Context: `expect-test` stores expected output inline and updates via a flag; great for parsers, formatters, analyzers.
- Avoid/Anti-pattern: brittle string asserts scattered across tests; manual golden files without tooling.

```rust path=null start=null
use expect_test::expect;

fn render(x: &[u8]) -> String { format!("{:?}", x) }

#[test]
fn snapshot() {
    let got = render(&[1,2,3]);
    expect![["[1, 2, 3]"]].assert_eq(&got);
    // Run with `EXPECT=overwrite` to update.
}
```

A.72 Bit-level Collections with bitvec
- Use when: pack booleans/bitfields tightly or model protocol layouts.
- Context: choose order (`Msb0`/`Lsb0`) and store type (`u8/u32`); supports slicing and iteration.
- Avoid/Anti-pattern: `Vec<bool>` or ad-hoc bit-twiddling without clear layout semantics.

```rust path=null start=null
use bitvec::prelude::*;
let mut bits: BitVec<u8, Msb0> = bitvec![0; 16];
bits.set(3, true);
assert!(bits[3]);
```

A.73 Read-Optimized Concurrent Maps (evmap)
- Use when: reads dominate and must be lock-free; eventual consistency is acceptable.
- Context: writers update a handle and `refresh()` to publish; readers see snapshots without locking.
- Avoid/Anti-pattern: `Mutex<HashMap>` on hot read paths; assuming linearizability.

```rust path=null start=null
let (read, mut write) = evmap::new();
write.insert("k", 1);
write.refresh();
assert_eq!(read.get_and("k", |v| v.iter().copied().collect::<Vec<_>>()), Some(vec![1]));
```

A.74 Backpressure with Bounded/Rendezvous Channels
- Use when: producers must slow when consumers lag; avoid unbounded memory growth.
- Context: `crossbeam_channel::bounded` or `flume::bounded`; use timeouts/deadlines.
- Avoid/Anti-pattern: unbounded queues in high-throughput services.

```rust path=null start=null
let (tx, rx) = flume::bounded::<u32>(1024);
// send with timeout
if tx.send_timeout(42, std::time::Duration::from_millis(50)).is_err() {
    // apply backpressure strategy
}
```

A.75 Choosing an Async Runtime (tokio/smol/glommio)
- Use when: selecting the execution model for async I/O.
- Context: tokio for general-purpose, ecosystem-rich; smol for lightweight single-threaded simplicity; glommio for io_uring and sharded executors.
- Avoid/Anti-pattern: mixing runtimes in one process without clear isolation.

A.76 Structured Concurrency via Task Scopes
- Use when: spawned tasks must complete before scope exit and errors must be contained.
- Context: prefer `tokio::task::scope` (or `task_scope` crate) to avoid detached tasks and leaks.
- Avoid/Anti-pattern: fire-and-forget tasks that outlive owners; manual join bookkeeping.

```rust path=null start=null
use tokio::task;
async fn parent() {
    task::scope(|s| async move {
        let h1 = s.spawn(async { /* child */ });
        let h2 = s.spawn(async { /* child */ });
        let _ = (h1.await, h2.await);
    }).await;
}
```

A.77 Macro Hygiene with $crate and Absolute Paths
- Use when: writing `macro_rules!` or proc-macros that refer to crate items robustly.
- Context: use `$crate::path` or `::std::...` to avoid name resolution surprises; prefer `proc_macro_error` for diagnostics.
- Avoid/Anti-pattern: relying on local imports inside expansions; unstable identifiers that clash.

```rust path=null start=null
#[macro_export]
macro_rules! myvec {
    ($($x:expr),* $(,)?) => {{
        let mut v = ::std::vec::Vec::new();
        $(v.push($x);)*
        v
    }};
}
```

A.78 Build Script Codegen and include!(OUT_DIR)
- Use when: generate bindings or code at build time (e.g., bindgen, version info).
- Context: write artifacts under `OUT_DIR` and `include!` them; declare `cargo::rerun-if-changed` inputs.
- Avoid/Anti-pattern: writing outside `OUT_DIR`; non-deterministic generation that breaks reproducibility.

```rust path=null start=null
// build.rs
use std::{env, fs, path::PathBuf};
fn main() {
    let out: PathBuf = env::var_os("OUT_DIR").unwrap().into();
    fs::write(out.join("generated.rs"), "pub const V: &str = \"1.0\";").unwrap();
    println!("cargo::rerun-if-changed=build.rs");
}
```

```rust path=null start=null
// lib.rs
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
```

A.79 Trait Objects: Dispatchable vs Non-Dispatchable Methods
- Use when: designing traits for dynamic dispatch while keeping rich APIs.
- Context: object safety requires dispatchable receivers (`&self`, `&mut self`, `Box<Self>`, `Rc<Self>`, `Arc<Self>`, `Pin<P>`). Mark helpers `where Self: Sized` to keep them off trait objects.
- Avoid/Anti-pattern: generics on methods, `Self` in returns, `impl Trait` returns on object-safe traits.

```rust path=null start=null
trait DynOk {
  fn do_it(&self);
  // not on dyn objects; callable only on concrete types
  fn into_inner(self) where Self: Sized; 
}
```

A.80 Async in Traits: AFIT vs async-trait
- Use when: declaring async behavior in traits.
- Context: prefer stable `async fn` in traits for static dispatch; for dynamic dispatch, return `Pin<Box<dyn Future<Output = T> + Send>>` or use `async-trait` with the boxing cost; be explicit about `Send` bounds.
- Avoid/Anti-pattern: assuming async trait methods are dyn-compatible; mixing boxed and inlined futures silently.

```rust path=null start=null
use core::future::Future;
trait Fetch {
  async fn get(&self, k: &str) -> Vec<u8>; // static dispatch
}
// dynamic dispatch alternative
trait FetchDyn {
  fn get(&self, k: &str) -> Pin<Box<dyn Future<Output = Vec<u8>> + Send + '_>>;
}
```

A.81 Two-Phase Borrowing Awareness (Nested &mut calls)
- Use when: APIs perform nested calls like `vec.push(vec.len())`.
- Context: a mutable borrow can be reserved then activated; ensure intermediate shared borrows die before activation; avoid keeping references alive across the activation point.
- Avoid/Anti-pattern: relying on UB-y patterns that assume immediate uniqueness.

```rust path=null start=null
let mut v = vec![1];
// OK: len() borrow ends before push activates the &mut
v.push(v.len());
```

A.82 Unsafe Aliasing Models: Stacked vs Tree Borrows (mental model)
- Use when: writing unsafe/FFI code involving raw pointers/reborrows.
- Context: `&mut` implies uniqueness; reads/writes via unrelated pointers can invalidate references; prefer setting up raw pointers before writes; use Miri to validate.
- Avoid/Anti-pattern: aliasing writes through raw pointers while shared refs exist.

A.83 MaybeUninit Correctness
- Use when: manual initialization for performance or FFI.
- Context: never create refs to uninitialized memory; write with `ptr::write`, call `assume_init` only after full init; consider `MaybeUninit<[T; N]>` for uninit arrays.
- Avoid/Anti-pattern: `assume_init_mut` followed by reads before initialization.

```rust path=null start=null
use std::{mem::MaybeUninit, ptr};
let mut buf: MaybeUninit<[u8; 4]> = MaybeUninit::uninit();
let p = buf.as_mut_ptr() as *mut u8;
unsafe { ptr::write(p, 1); ptr::write(p.add(1), 2); ptr::write(p.add(2), 3); ptr::write(p.add(3), 4); }
let arr = unsafe { buf.assume_init() };
```

A.84 Pin Projection and pin-project-lite
- Use when: types need pinned self with pinned fields.
- Context: use `pin-project-lite` (or `pin-project`) to safely project pinned fields; keep invariants when implementing `Future::poll`.
- Avoid/Anti-pattern: manual `Pin::new_unchecked` field projection without proof.

```rust path=null start=null
use pin_project_lite::pin_project;
pin_project! {
  struct Task { #[pin] fut: impl Future<Output = ()> }
}
```

A.85 Drop Check and Generic Drop Soundness
- Use when: implementing `Drop` for generic types.
- Context: ensure generic params outlive `self` on drop; avoid holding references into shorter-lived data in structs with `Drop`.
- Avoid/Anti-pattern: hidden self-referential borrows that make `dropck` unsound.

A.86 SAFETY Documentation and Miri in CI
- Use when: authoring `unsafe`.
- Context: document invariants with `// SAFETY:`; run `cargo +nightly miri test` in CI for UB detection; prefer smallest unsafe blocks, encapsulated behind safe APIs.
- Avoid/Anti-pattern: sprawling `unsafe` with no justification; unsound public `unsafe fn` contracts.

A.87 Structured Concurrency with JoinSet
- Use when: run and manage many tasks that must end with the parent.
- Context: `tokio::task::JoinSet` owns children; dropping it cancels outstanding tasks; loop `join_next().await` to drain.
- Avoid/Anti-pattern: detached tasks leaking beyond scope.

```rust path=null start=null
use tokio::task::JoinSet;
let mut set = JoinSet::new();
for i in 0..10 { set.spawn(async move { i }); }
while let Some(res) = set.join_next().await { let _ = res?; }
```

A.88 Feature Flags Must Be Additive
- Use when: designing Cargo features.
- Context: avoid mutually-exclusive features; document effects; use `dep:` to decouple names; guard with `#[cfg(feature = "foo")]`.
- Avoid/Anti-pattern: feature combos that break due to removal/incompatibility.

```toml path=null start=null
[features]
serde = ["dep:serde"]
fast = []
```

A.89 Doctest Contracts: Errors/Panics/Safety Sections
- Use when: documenting public APIs.
- Context: include `# Errors`, `# Panics`, `# Safety` where applicable; ensure examples compile and assert behavior.
- Avoid/Anti-pattern: undocumented panics or unsafe preconditions.

```rust path=null start=null
/// Parses input.
/// # Errors
/// Returns Err if input is invalid.
pub fn parse(_s: &str) -> Result<(), Error> { Ok(()) }
```

A.90 Fallible Iterator Pipelines (try_collect/try_fold)
- Use when: error-aware transforms.
- Context: `Iterator<Item = Result<T,E>>` -> `Result<Vec<T>,E>` via `collect`; use `try_fold`/`try_for_each` to short-circuit on first error.
- Avoid/Anti-pattern: manual unwraps inside maps.

```rust path=null start=null
let items: Vec<Result<u32, &'static str>> = vec![Ok(1), Ok(2)];
let ok: Result<Vec<_>, _> = items.into_iter().collect();
```

A.91 Unsafe Encapsulation with Minimal Surface + Miri
- Use when: providing safe wrappers over unsafe code/FFI.
- Context: make unsafe internals private; add `// SAFETY:` comments; validate with Miri/sanitizers.
- Avoid/Anti-pattern: exposing raw pointers directly in public APIs.

A.92 API Prelude Re-exports
- Use when: improve ergonomics for frequent traits/types.
- Context: expose `prelude` with `pub use` of common items; keep internal modules private.
- Avoid/Anti-pattern: deep import paths for essential traits.

```rust path=null start=null
pub mod prelude { pub use crate::{MyTrait, KeyType}; }
```

A.93 Lifetime Elision Rules in Public APIs
- Use when: writing function signatures with references.
- Context: rely on elision for simple cases; add explicit lifetimes when outputs cannot be inferred or when multiple inputs exist; avoid returning references detached from inputs.
- Avoid/Anti-pattern: ambiguous signatures like `fn frob(s: &str, t: &str) -> &str` without lifetimes.

A.94 Trait Object Lifetimes and Default Bounds
- Use when: using `dyn Trait` references.
- Context: understand default object lifetime bound; specify `&'a dyn Trait` when needed; avoid accidental `'static` in trait objects.
- Avoid/Anti-pattern: assuming trait object lifetimes are inferred as desired without annotation.

A.95 Borrow Conflict Hygiene (E0502)
- Use when: mixing shared and mutable borrows.
- Context: shorten borrows with inner scopes; create temporaries so shared borrows end before `&mut` activation; split struct borrows when needed.
- Avoid/Anti-pattern: long-lived shared borrows that overlap upcoming `&mut` operations.

A.96 Dyn Vtable Invariants and Raw Trait Objects
- Use when: interfacing with `dyn Trait` and raw pointers.
- Context: never fabricate vtables; avoid leaking raw trait-object pointers; prefer safe coercions and upcasting where available.
- Avoid/Anti-pattern: constructing invalid fat pointers or transmuting between unrelated `dyn` types.

A.97 OBRM (RAII) Resource Guards
- Use when: acquiring OS/file/lock resources.
- Context: acquire in constructor; release in `Drop`; keep guards short-lived; model ownership explicitly.
- Avoid/Anti-pattern: manual free/close without RAII; sharing guards beyond necessary scope.

A.98 Language/Std Prelude Awareness
- Use when: relying on items in scope by default.
- Context: know what the language/std prelude brings; avoid shadowing common items; re-export crate preludes judiciously.
- Avoid/Anti-pattern: redundant `use` of prelude items; conflicting names hiding standard traits.

A.99 Niche Optimization with NonZero and Option
- Use when: optimize enum sizes.
- Context: use `NonZero*` and `Option<NonZero*>` or `Option<NonNull<T>>` to keep size equal to primitive/pointer; document layouts only when required.
- Avoid/Anti-pattern: sentinel integers in FFI or home-grown nullable encodings when the type system provides niches.

A.100 Match and Pattern Ergonomics
- Use when: matching references; prefer clear bindings.
- Context: be explicit with `ref`/`ref mut` as needed; beware edition changes; keep patterns readable over overly clever ergonomics.
- Avoid/Anti-pattern: relying on implicit ref patterns that obscure mutability and ownership.

A.101 Choosing Macro Flavors (derive/attribute/function-like)
- Use when: determining macro type for a task.
- Context: prefer derive for generating impls from type structure; attribute macros to transform or wrap items; function-like for DSLs/codegen where call-site granularity is desired.
- Avoid/Anti-pattern: using proc-macros when macro_rules! suffices; heavy proc-macro logic that can be done in build.rs.

A.102 TokenStreams, Spans, and Hygiene Discipline
- Use when: building procedural macros.
- Context: operate on TokenStreams; attach meaningful Spans to preserve error locations; use absolute paths (`::std::...`) to avoid name resolution surprises; unique internal names.
- Avoid/Anti-pattern: leaking local imports into expansions; panicking instead of emitting compile_error!.

A.103 syn + quote + proc-macro2 Pipeline
- Use when: parsing and generating Rust in proc-macros.
- Context: parse with syn, construct tokens with quote!, and target proc-macro2 to test outside proc-macro crates; keep AST transforms minimal and predictable.
- Avoid/Anti-pattern: manual string concatenation to build code; relying on unstable compiler internals.

A.104 Robust Macro Error Reporting
- Use when: signaling misuse or invalid input.
- Context: prefer `compile_error!` with helpful messages; attach Spans to the exact offending tokens; avoid `panic!` which surfaces as opaque errors.
- Avoid/Anti-pattern: cryptic errors without spans; swallowing errors and generating broken code.

A.105 macro_rules! Repetition and Fragment Specs
- Use when: writing declarative macros.
- Context: use `$( ... )*`/`+` with separators; select correct fragment spec (`expr`, `ty`, `path`, `ident`, `tt`); keep matchers simple and layered.
- Avoid/Anti-pattern: over-greedy matchers; ambiguous patterns that require deep backtracking.

A.106 Build Scripts: OUT_DIR + include!
- Use when: compile-time codegen/bindings.
- Context: write artifacts to `OUT_DIR`, print `cargo::rerun-if-changed=...` for inputs, and include with `include!(concat!(env!("OUT_DIR"), "/file.rs"))`.
- Avoid/Anti-pattern: writing outside `OUT_DIR`; missing rerun hints causing unnecessary or missing rebuilds.

A.107 Formatting and Macros
- Use when: large macro bodies or DSLs impact formatting.
- Context: configure rustfmt options (format_macro_bodies/matchers, skip lists) judiciously; prefer readable macro expansions and tests.
- Avoid/Anti-pattern: disabling all formatting; relying on formatting-sensitive macro bodies.

A.108 Macro Security and Performance
- Use when: designing macro-heavy APIs.
- Context: be mindful that proc-macros run at compile time with filesystem access; constrain surface area; document costs; avoid quadratic token growth.
- Avoid/Anti-pattern: hidden heavy codegen surprising users; reading arbitrary files without rerun guards.

A.109 Borrow-Checker-Friendly API Design
- Use when: exposing methods that mix reads and writes.
- Context: structure code so shared borrows end before `&mut` activation; create temporaries; use smaller scopes to avoid E0502 conflicts.
- Avoid/Anti-pattern: long-lived shared borrows overlapping imminent mutable operations.

A.110 Two-Phase Borrow Patterns in Public APIs
- Use when: methods take `&mut self` and perform nested calls.
- Context: ensure intermediate borrows do not escape to callers; document expectations; avoid relying on subtle two-phase details across crate boundaries.
- Avoid/Anti-pattern: exposing APIs that require specific compiler lowering to be sound.

A.111 Unsafe Aliasing Invariants for `&`/`&mut`
- Use when: writing unsafe code around pointers and references.
- Context: `&mut` implies uniqueness; `&` implies immutability (outside `UnsafeCell`); set up raw pointers before writes; validate with Miri.
- Avoid/Anti-pattern: writing through raw pointers while shared refs are live.

A.112 CLI Exit Codes and Termination
- Use when: designing CLIs that must signal failure.
- Context: use `Result<(), E>` return from `main` (Termination), `std::process::ExitCode`, or `std::process::exit`; print helpful errors.
- Avoid/Anti-pattern: panicking for expected user errors; inconsistent nonzero codes.

A.113 API Guidelines: Naming, Features, and Sealed Traits
- Use when: stabilizing public APIs.
- Context: follow C-CASE conventions; make features additive (no `no-foo`), avoid `use-`/`with-` prefixes; use sealed traits to control external impls.
- Avoid/Anti-pattern: breaking SemVer via trait changes; mutually exclusive features.

A.114 CI Quality Gates: fmt, clippy, doctests, coverage
- Use when: enforcing quality in CI.
- Context: `cargo fmt --check`, `cargo clippy -D warnings`, run doctests, use `nextest` and `llvm-cov`/`grcov`; add `cargo-deny` and SemVer checks.
- Avoid/Anti-pattern: allowing warnings; untested examples; unaudited dependencies.

A.115 Dynamic vs Static Dispatch Trade-offs
- Use when: choosing generics vs `dyn Trait`.
- Context: prefer generics on hot paths; use trait objects for heterogenous collections or plugin boundaries; document `Send/Sync` as needed.
- Avoid/Anti-pattern: premature boxing/dynamic dispatch; monomorphization bloat from excessive generics.

A.116 Avoid Unnecessary Cloning; Prefer Borrowing and Shared Ownership Wisely
- Use when: eliminating needless allocations.
- Context: borrow where possible; choose `Rc/Arc` when multiple owners are required; avoid “clone to satisfy the borrow checker”.
- Avoid/Anti-pattern: cloning large structures gratuitously; using `Arc` uniformly instead of designing ownership.

A.117 Fast Synchronization with parking_lot
- Use when: you need smaller, faster locks than std on contended paths.
- Context: `parking_lot::{Mutex,RwLock,Condvar,Once}` offer compact types, fast uncontended paths, adaptive spinning, and optional deadlock detection; semantics differ slightly (e.g., poisoning behavior) from std.
- Avoid/Anti-pattern: mixing `std::sync` and `parking_lot` guards; assuming identical poisoning/fairness semantics; using on platforms where std primitives are required by dependencies.

```rust path=null start=null
use parking_lot::{Mutex, RwLock};
static DATA: Mutex<Vec<u8>> = Mutex::new(Vec::new());
let mut g = DATA.lock();
g.push(1);
```

A.118 io_uring-backed Runtimes (monoio/tokio-uring)
- Use when: Linux (>=5.11) high-throughput, I/O-bound servers benefit from kernel async I/O.
- Context: `monoio` favors thread-per-core and non-`Send` tasks; `tokio-uring` integrates with Tokio-backed resources; benchmark on your workload and gate behind features.
- Avoid/Anti-pattern: mixing runtimes without isolation; assuming portability to non-Linux; using when epoll/kqueue suffices.

```rust path=null start=null
// monoio example
ofn main() -> monoio::Result<()> {
    monoio::RuntimeBuilder::new().enable_timer().build()?.block_on(async move {
        // use monoio::net for io_uring-backed sockets/files
        Ok(())
    })
}
```

A.119 no_std Panic Handling
- Use when: building embedded/`no_std` binaries where unwind behavior is undefined.
- Context: define a single `#[panic_handler]` to choose reset/log/LED-blink, etc.; do not unwind; keep handler minimal and non-allocating.
- Avoid/Anti-pattern: relying on default panic in `no_std`; performing blocking I/O without watchdog considerations.

```rust path=null start=null
#![no_std]
use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // signal fault, then halt
    loop {}
}
```

A.120 Native dyn Upcasting (Rust 1.86+)
- Use when: a trait has a supertrait and you need `&dyn Sub` -> `&dyn Super` (also works for Box/Arc/raw pointers).
- Context: prefer built-in upcasting over custom `as_super()` shims; remove redundant helper methods; never fabricate raw trait-object pointers or vtables—Miri flags invalid metadata.
- Avoid/Anti-pattern: manual transmute/coercions for upcasting; leaking raw trait-object pointers.

```rust path=null start=null
trait Super {}
trait Sub: Super {}
fn take_super(x: &dyn Sub) -> &dyn Super { x }
```

A.121 Atomic Ordering Hygiene + Loom
- Use when: writing lock-free code with atomics; verify interleavings deterministically.
- Context: default to Acquire/Release for hand-over-hand patterns; use Relaxed only with proven invariants; fences sparingly; model with `loom::model` to explore schedules (note Loom treats SeqCst ops as AcqRel, but `fence(SeqCst)` is supported).
- Avoid/Anti-pattern: SeqCst everywhere; assuming a particular schedule; non-determinism in loom tests.

```rust path=null start=null
use loom::{sync::{Arc, atomic::{AtomicUsize, Ordering::*}}, thread};
loom::model(|| {
  let n = Arc::new(AtomicUsize::new(0));
  let a = n.clone(); thread::spawn(move || { let cur = a.load(Acquire); a.store(cur+1, Release); });
  let b = n.clone(); thread::spawn(move || { let cur = b.load(Acquire); b.store(cur+1, Release); });
});
```

A.122 Lock-free Building Blocks (crossbeam-epoch, AtomicCell)
- Use when: implementing concurrent data structures without locks.
- Context: use epoch-based reclamation (`crossbeam-epoch`) to defer frees safely; `AtomicCell<T: Copy>` for simple shared mutable slots; document hazards and MSRV.
- Avoid/Anti-pattern: manual hazard-pointer schemes; ad-hoc reclamation; mixing GC domains.

A.123 Managing Monomorphization Bloat
- Use when: generics inflate compile times or binary size.
- Context: prefer generics on hot paths; use `dyn Trait` at API boundaries and for rarely-used code paths; consider feature flags to switch dispatch; measure impact.
- Avoid/Anti-pattern: blanket generics across public APIs without need; premature boxing in inner loops.

A.124 Procedural Macro Diagnostics (proc-macro-error)
- Use when: writing proc-macros that must report errors precisely.
- Context: prefer `proc_macro_error` to emit span-rich diagnostics; avoid `panic!`; combine with `syn`/`quote` pipeline.
- Avoid/Anti-pattern: cryptic panics; missing spans; swallowing multiple errors.

A.125 DataFrames with Polars (lazy + streaming)
- Use when: columnar analytics over large datasets.
- Context: prefer `polars-lazy` for query planning and parallel execution; use streaming/out-of-core where memory-bound; tune via documented env vars only when needed.
- Avoid/Anti-pattern: eager DataFrame ops on huge tables; relying on env knobs instead of fixing query plans.

```rust path=null start=null
use polars::prelude::*;
let df = df!("a" => &[1,2,3], "b" => &[4,5,6])?;
let out = df.lazy()
    .groupby([col("a")])
    .agg([col("b").sum()])
    .collect()?;
```

A.126 Caching: moka (concurrent) vs cached (memoization)
- Use when: reduce recomputation or expensive I/O.
- Context: `moka::Cache` for concurrent TTL/LRU caches; `cached` macros for function-level memoization; bound size/TTL and observe memory use.
- Avoid/Anti-pattern: unbounded caches; caching non-deterministic results; stale reads without TTL.

```rust path=null start=null
use moka::sync::Cache;
let cache = Cache::builder().max_capacity(10_000).build();
cache.insert("k", 1);
if let Some(v) = cache.get("k") { /* ... */ }
```

```rust path=null start=null
#[cached::cached(size=128)]
fn fib(n: u64) -> u64 { /* ... */ n }
```

A.127 SQLx CLI and Offline Mode
- Use when: manage migrations and build offline `query!`.
- Context: use `sqlx migrate add/run`; set `DATABASE_URL` for CLI; enable offline preparation in CI to avoid DB at build time.
- Avoid/Anti-pattern: ad-hoc SQL without migrations; building with online macros in hermetic CI.

```sh path=null start=null
sqlx migrate add init
sqlx migrate run --database-url=${DATABASE_URL}
```

A.128 postcard for no_std serialization
- Use when: serde in embedded/no_std contexts.
- Context: prefer fixed buffers with `to_slice`/`from_bytes`; avoid heap where possible; validate sizes.
- Avoid/Anti-pattern: allocating in tight ISR paths; trusting unverified input.

```rust path=null start=null
#![no_std]
use postcard::{to_slice, from_bytes};
let mut buf = [0u8; 64];
let written: &mut [u8] = to_slice(&my_struct, &mut buf)?;
let round: My = from_bytes(written)?;
```

A.129 Rayon for CPU data parallelism
- Use when: CPU-bound loops over large collections.
- Context: replace hot sequential loops with `into_par_iter()`; keep work items independent/pure; measure speedups.
- Avoid/Anti-pattern: parallelizing tiny tasks; holding locks inside parallel iterators.

```rust path=null start=null
use rayon::prelude::*;
let sum: i64 = (0..1_000_000i64).into_par_iter().sum();
```

A.130 Scoped Threads for Non-'static Borrows
- Use when: you need threads to temporarily borrow data without `'static` lifetimes.
- Context: use `std::thread::scope` to spawn threads that borrow from the parent; all threads must finish before the scope exits.
- Avoid/Anti-pattern: forcing `Arc<Mutex<_>>` solely to satisfy `'static` when simple borrowing suffices; leaking threads beyond scope.

```rust path=null start=null
use std::thread;
let mut v = vec![1, 2, 3];
thread::scope(|s| {
    s.spawn(|| { /* read-only borrow is fine */ });
    s.spawn(|| { /* can borrow &mut v in disjoint regions via interior logic */ });
    s.spawn(|| println!("len = {}", v.len()));
}); // all threads joined here; v is still valid
```

A.131 MPMC with crossbeam-channel and select!
- Use when: need fast multi-producer multi-consumer channels and multiplexing across events.
- Context: choose `bounded` for backpressure or `unbounded` for buffering; use `select!` to await on multiple receivers/timeouts; handle disconnection.
- Avoid/Anti-pattern: relying on `std::sync::mpsc` for complex concurrent topologies; unbounded channels in high-throughput pipelines without limits.

```rust path=null start=null
use crossbeam_channel::{bounded, select, tick};
let (tx, rx) = bounded::<u32>(1024);
let ticker = tick(std::time::Duration::from_millis(10));
loop {
    select! {
        recv(rx) -> msg => if let Ok(x) = msg { /* handle */ } else { break },
        recv(ticker) -> _ => { /* periodic work */ },
        default(Duration::from_millis(1)) => { /* do other work */ }
    }
}
```

A.132 Rendezvous Backpressure with sync_channel
- Use when: producer must block until consumer receives (rendezvous) or when a small bounded buffer is required.
- Context: `std::sync::mpsc::sync_channel(0)` for strict rendezvous; use small capacities to throttle producers.
- Avoid/Anti-pattern: large unbounded queues that hide overload; busy-waiting when blocking semantics are desired.

```rust path=null start=null
use std::sync::mpsc;
let (tx, rx) = mpsc::sync_channel::<String>(0); // rendezvous
std::thread::spawn(move || { tx.send("hello".into()).ok(); });
let msg = rx.recv().unwrap();
```

A.133 One-time Initialization: OnceLock, LazyLock, and once_cell
- Use when: initialize globals or expensive state exactly once without races.
- Context: prefer `std::sync::OnceLock`/`LazyLock` on stable; `once_cell` for older MSRV or extra APIs; expose getters returning `&'static T`.
- Avoid/Anti-pattern: `lazy_static!` when `OnceLock`/`LazyLock` suffices; re-initializing via interior mutability.

```rust path=null start=null
use std::sync::OnceLock;
static CONFIG: OnceLock<String> = OnceLock::new();
fn config() -> &'static str { CONFIG.get_or_init(|| "default".to_string()) }
```

A.134 Fine-grained Visibility and Prelude Control
- Use when: structuring large libraries with clear internal boundaries or avoiding implicit imports.
- Context: use `pub(crate)`, `pub(super)`, and `pub(in path)` to scope items precisely; apply `#![no_implicit_prelude]` in modules that require explicit paths (e.g., macro internals).
- Avoid/Anti-pattern: defaulting to `pub` broadly; relying on implicit preludes in macros leading to hygiene issues.

```rust path=null start=null
pub mod outer { 
    pub(in crate::outer) fn only_outer() {}
}
```

A.135 Source-annotated Diagnostics with miette
- Use when: you need rich, span-labeled error reports for end users or CLI tools.
- Context: derive `Diagnostic` on your error types, attach `#[source_code]` and `#[label]`/`#[related]` to emit snippets; keep this in binaries, not libraries.
- Avoid/Anti-pattern: using `miette` types in public library APIs; emitting unlabeled, context-free errors.

```rust path=null start=null
use miette::{Diagnostic, Result, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("parse error")]
struct ParseErr {
  #[source_code]
  src: String,
  #[label("here")] span: SourceSpan,
}

fn parse(_s: &str) -> Result<()> { Err(ParseErr { src: "x+y".into(), span: (2..3).into() }.into()) }
```

A.136 Thread Naming and Join Discipline
- Use when: spawning OS threads that must be tracked and debuggable.
- Context: prefer capturing `JoinHandle` and calling `join()`; use `thread::Builder::name` for observability; avoid truly detached threads.
- Avoid/Anti-pattern: spawning and forgetting threads; relying on process exit to clean up work.

```rust path=null start=null
use std::thread;
let h = thread::Builder::new().name("worker-1".into()).spawn(|| { /* work */ })?;
let res = h.join();
```

A.137 Prefer fn Pointers When No Capture Is Needed
- Use when: an API accepts a callable that never needs to capture environment.
- Context: taking a parameter of type `fn(T) -> U` is simpler and avoids extra monomorphizations; callers can pass function items or non-capturing closures.
- Avoid/Anti-pattern: generic `F: Fn(T) -> U` when you only need a function pointer; overusing generics inflates compile times and binary size.

```rust path=null start=null
fn apply<T,U>(x: T, f: fn(T) -> U) -> U { f(x) }
fn double(n: i32) -> i32 { n*2 }
let y = apply(21, double);
```

A.138 FromIterator for Explicit Collection Construction
- Use when: you want to avoid turbofish on `collect` and make target type explicit.
- Context: call `FromIterator::from_iter(iter)` directly, especially in generic contexts; implement `FromIterator` for your types.
- Avoid/Anti-pattern: opaque `collect::<Vec<_>>()` in codebases that prefer explicit constructors.

```rust path=null start=null
use std::iter::FromIterator;
let it = [1,2,3].into_iter().map(|x| x * 2);
let v = Vec::<i32>::from_iter(it);
```

A.139 Atomics and Fences: Don’t Order Non-atomics
- Use when: building low-level synchronization like spinlocks.
- Context: use Acquire/Release on atomics; `fence(SeqCst)` only orders other atomic ops and does not make plain memory accesses visible across threads.
- Avoid/Anti-pattern: relying on fences to synchronize non-atomic loads/stores; mixing non-atomic shared mutability across threads.

```rust path=null start=null
use std::sync::atomic::{AtomicBool, Ordering, fence};
static LOCK: AtomicBool = AtomicBool::new(false);
fn lock() { while LOCK.swap(true, Ordering::Acquire) {} }
fn unlock() { fence(Ordering::Release); LOCK.store(false, Ordering::Release); }
```

A.140 Representation Choices: repr(C), repr(transparent), and enum reprs
- Use when: FFI or layout-dependent code requires a stable ABI.
- Context: use `#[repr(C)]` for FFI structs/enums; `#[repr(transparent)]` for newtype wrappers crossing FFI; for enums, use primitive reprs like `#[repr(u8)]` only when required and safe; default `repr(Rust)` is best for optimization.
- Avoid/Anti-pattern: assuming default enum discriminant layout; reading discriminants of non-`repr` enums; casual use of `repr(packed)` creating unaligned references.

```rust path=null start=null
#[repr(C)]
struct CTimeSpec { tv_sec: i64, tv_nsec: i64 }
#[repr(transparent)]
struct Fd(i32);
#[repr(u8)]
enum Kind { A, B(u32) }
```

A.141 Stabilizing Public APIs with #[non_exhaustive]
- Use when: you plan to add fields/variants later without breaking SemVer.
- Context: mark exported structs/enums or enum variants `#[non_exhaustive]`; external code must use `..` in patterns and cannot construct directly.
- Avoid/Anti-pattern: exposing exhaustively-matchable enums in public APIs that will grow.

```rust path=null start=null
#[non_exhaustive]
pub enum Error { Io(std::io::Error), Parse }

fn handle(e: Error) {
    match e { Error::Io(_)| Error::Parse | _ => {} }
}
```

A.142 Rust 2024 Pattern Ergonomics: Prefer Fully Explicit Patterns
- Use when: migrating to 2024 or writing clear matches.
- Context: avoid redundant `ref`/`ref mut`; `mut` on bindings requires fully explicit patterns; use `cargo fix --edition` to rewrite.
- Avoid/Anti-pattern: relying on implicit reference binding modes that change under 2024 rules.

```rust path=null start=null
// Before (implicit, may lint under 2024)
match &opt { Some(x) => {/*...*/}, _ => {} }
// After (explicit)
match opt.as_ref() { Some(x) => {/*...*/}, None => {} }
```

A.143 FFI Nullability and Unwinding Contracts
- Use when: modeling nullable pointers and crossing language boundaries.
- Context: prefer `Option<NonNull<T>>` or `Option<extern "C" fn()>` over sentinel integers; mark FFI with `extern "C"`; use `extern "C-unwind"` when exceptions/panics may cross; keep `unsafe` at the boundary and wrap with safe APIs.
- Avoid/Anti-pattern: null `&T`/`&mut T`; unwinding across FFI without `-unwind` ABI.

```rust path=null start=null
use core::ptr::NonNull;
#[no_mangle]
pub extern "C" fn api(cb: Option<extern "C" fn(i32)>) -> i32 { if let Some(f) = cb { f(1); } 0 }
// Nullable pointer field
type MaybeBuf = Option<NonNull<u8>>;
```

A.144 Choosing Collections and Planning Capacity
- Use when: selecting data structures and avoiding accidental slow paths.
- Context: default to `Vec`; use `VecDeque` for queue/deque ends; avoid `LinkedList` for general use; build heaps with `BinaryHeap::from(Vec)`; `HashMap` uses SwissTable (hashbrown) with expected O(1); use `IndexMap` for stable order; preallocate with `with_capacity`.
- Avoid/Anti-pattern: modifying keys inside `HashMap`; pushing ascending into `BinaryHeap` repeatedly; relying on unbounded growth without `reserve`.

```rust path=null start=null
let mut v = Vec::with_capacity(1024);
use indexmap::IndexMap; let mut im = IndexMap::new();
use std::collections::BinaryHeap; let h = BinaryHeap::from(vec![3,1,4]);
```

A.145 Diesel: Migrations and Type-safe Queries
- Use when: you need compile-time checked SQL and schema migrations.
- Context: use `diesel_cli` for migrations; derive/query DSL ensures type safety; keep schema.rs generated in sync; prefer explicit connections/pools.
- Avoid/Anti-pattern: ad-hoc SQL strings without migrations; ignoring compile-time errors by falling back to runtime stringly queries.

```rust path=null start=null
// diesel setup
// cargo install diesel_cli
// diesel migration generate create_users
#[derive(Queryable)]
struct User { id: i32, name: String }
```

A.146 Build Scripts: OUT_DIR, rerun-if-*, and Link Directives
- Use when: binding native libs or generating code.
- Context: write outputs to `OUT_DIR`; narrow rebuilds with `cargo::rerun-if-changed`/`cargo::rerun-if-env-changed`; link with `cargo::rustc-link-lib`/`-search`; gate custom cfgs with `cargo::rustc-check-cfg`.
- Avoid/Anti-pattern: touching files outside `OUT_DIR`; re-running on every build due to missing rerun hints.

```rust path=null start=null
// build.rs
println!("cargo::rerun-if-changed=wrapper.h");
println!("cargo::rustc-link-lib=static=foo");
println!("cargo::rustc-check-cfg=cfg(has_foo)");
```

A.147 The -sys Crate Pattern and links
- Use when: exposing FFI for a native library with a safe wrapper.
- Context: name the low-level crate `foo-sys`, set `package.links = "foo"`, emit link metadata; high-level crate depends on `foo-sys` and exposes safe APIs.
- Avoid/Anti-pattern: mixing safe wrapper and raw FFI in one crate; duplicating `links` across multiple crates.

```toml path=null start=null
# Cargo.toml
[package]
links = "foo"

[build-dependencies]
cc = "*"
```

A.148 Pin Toolchains per Project (rustup)
- Use when: ensuring reproducible builds across environments.
- Context: add `rust-toolchain.toml` with `channel = "stable"` or a pinned version/date; use overrides for local custom toolchains; document MSRV.
- Avoid/Anti-pattern: relying on developer-global toolchains; unspecified editions.

```toml path=null start=null
# rust-toolchain.toml
[toolchain]
channel = "1.80.1"
components = ["rustfmt", "clippy"]
```

A.149 Cargo Workspaces and Config Layering
- Use when: managing multi-crate repos and CI builds.
- Context: define a workspace root for shared `Cargo.lock`/`target`; use `.cargo/config.toml` for aliases (`cargo c = check`), profiles, offline mode; run `cargo check --workspace`.
- Avoid/Anti-pattern: per-crate divergent profiles/locks; networked builds without `--offline` in hermetic CI.

```toml path=null start=null
# .cargo/config.toml
[alias]
c = "check --workspace"
[net]
offline = true
```

A.150 Procedural Macro Discipline: Hygiene, Spans, Diagnostics
- Use when: writing proc-macros (derive/attribute/function-like).
- Context: use absolute paths (e.g., `::std::...`), generate unique identifiers, attach precise `Span`s; prefer `compile_error!` or `proc-macro-error` over panic; mark crate `[lib]
proc-macro = true`.
- Avoid/Anti-pattern: assuming hygiene; leaking local imports; panicking with opaque messages.

```toml path=null start=null
[lib]
proc-macro = true
```

A.151 Choosing a Global Allocator (jemalloc/mimalloc)
- Use when: allocator behavior is a bottleneck; benchmark-driven.
- Context: set `#[global_allocator]` to `tikv_jemallocator::Jemalloc` or `mimalloc::MiMalloc`; measure throughput, latency, and RSS; consider security modes.
- Avoid/Anti-pattern: assuming universal wins; ignoring memory footprint/regressions.

```rust path=null start=null
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

A.152 Build-time Optimizations: LTO, PGO, and Inlining
- Use when: squeezing runtime performance for hot binaries.
- Context: enable ThinLTO or Fat LTO in release; use cargo-pgo for profile-guided optimization; annotate tiny hot wrappers with `#[inline]` judiciously.
- Avoid/Anti-pattern: blanket `inline(always)`; enabling FatLTO without measuring compile-time/size trade-offs.

```toml path=null start=null
[profile.release]
lto = "thin"
codegen-units = 1
```

A.153 Coverage with cargo-llvm-cov (CI + local)
- Use when: track line/branch coverage and gate regressions.
- Context: run `cargo llvm-cov --workspace` locally; in CI, emit lcov/codecov JSON and upload; ignore generated/build dirs via `--ignore-filename-regex`.
- Avoid/Anti-pattern: mixing tools without consistent flags; relying on flaky nightly-only options unless required.

```sh path=null start=null
cargo llvm-cov --all-features --workspace --branch --fail-under-lines 75
```

A.154 Fuzzing and Property Testing Together
- Use when: hardening parsers/decoders and complex invariants.
- Context: add `cargo-fuzz` targets for libFuzzer; use `proptest` or `quickcheck` for properties; seed RNG/env for determinism in CI; minimize failing inputs.
- Avoid/Anti-pattern: running fuzzers without sanitizers; ignoring corpus minimization.

```sh path=null start=null
cargo fuzz init && cargo fuzz add parse && cargo fuzz run parse
```

A.155 Specialization: Avoid in Public APIs
- Use when: you control all crates and understand soundness limits.
- Context: prefer explicit trait design over specialization; if you must, use `min_specialization` behind feature flags for internal crates only; document invariants.
- Avoid/Anti-pattern: relying on full specialization in libraries; overlapping impls that risk unsoundness.

A.156 Coherence and Orphan Rules in Practice
- Use when: designing extension traits and impls across crates.
- Context: either the trait or the self type must be local; use newtype wrappers to implement foreign traits for foreign types; avoid overlapping impls; consider sealed traits.
- Avoid/Anti-pattern: blanket impls that could overlap downstream; violating E0117/E0210.

A.157 repr modifiers: align and packed
- Use when: aligning data for cache lines or external protocols; packing only when absolutely necessary.
- Context: `#[repr(align(N))]` to raise alignment; `#[repr(packed(n))]` strips padding but causes unaligned fields—access via raw pointers only; never mix with assumptions of alignment.
- Avoid/Anti-pattern: taking references to packed fields; combining `align` and `packed`.

```rust path=null start=null
#[repr(align(64))]
struct CacheLine([u8; 64]);
```

A.158 Enum Discriminants and repr(u8)
- Use when: FFI or stable wire formats require fixed discriminant size.
- Context: `#[repr(u8)]` for field-less or C-like enums; safe discriminant inspection via `mem::discriminant`; do not read discriminants for `repr(Rust)` enums.
- Avoid/Anti-pattern: mixing `repr(Rust)` assumptions with raw reads; over-constraining repr without need.

A.159 Associated Types for Clearer Traits
- Use when: trait outputs are tied to the implementor and improve readability.
- Context: prefer `type Output;` over extra generic parameters when appropriate; combine with where-clauses for clarity; consider GATs when lifetimes depend on methods.
- Avoid/Anti-pattern: proliferating type params where associated types fit better.

A.160 Arrow + DataFusion for In-memory Analytics
- Use when: you need a Rust-native, columnar analytics engine with SQL on Arrow memory.
- Context: use Arrow arrays/RecordBatches; build queries with DataFusion SQL or logical plans; leverage vectorized execution and zero-copy between processes.
- Avoid/Anti-pattern: row-based ad-hoc processing for large analytics; converting Arrow to row structs prematurely.

A.161 Layered Config: config + dotenvy + envy (serde)
- Use when: applications need hierarchical configuration from files, env, and defaults.
- Context: define defaults, merge files (TOML/YAML/JSON), load .env with dotenvy in dev, override with env vars via envy into typed structs; document precedence.
- Avoid/Anti-pattern: hand-parsing env; scattering config reads without a central builder.

```rust path=null start=null
#[derive(serde::Deserialize)]
struct Settings { db_url: String, workers: usize }
let mut cfg = config::Config::builder()
    .set_default("workers", 4)?
    .add_source(config::File::with_name("Config").required(false))
    .add_source(config::Environment::default())
    .build()?;
let s: Settings = cfg.try_deserialize()?;
```

A.162 Embedded KV Choices: RocksDB vs Sled vs LMDB
- Use when: selecting a local key-value store.
- Context: RocksDB (LSM, high write throughput, tuned space/compaction), Sled (pure Rust, lock-free B+ tree, watch prefixes, may use more space), LMDB (memory-mapped, great read throughput, multi-process, write-once txn).
- Avoid/Anti-pattern: defaulting without measuring space/write amp; multi-process with Sled; LMDB for heavy concurrent writers.

A.163 Messaging Clients: NATS vs Kafka vs Pulsar
- Use when: building message-driven systems.
- Context: NATS (async-nats; simple pub/sub, JetStream for at-least-once), Kafka (rdkafka/librdkafka; partitions, consumer groups, backpressure), Pulsar (pure Rust clients; topic/partitioning, async API).
- Avoid/Anti-pattern: unbounded consumers; ignoring idempotency and exactly-once myths; mixing at-most/at-least-once semantics.

A.164 no_std Embedded Patterns: RTIC, embedded-hal, Panic, Debugging
- Use when: Cortex-M/embedded development.
- Context: `#![no_std]`, provide `#[panic_handler]`; use `embedded-hal` traits and platform HALs; structure apps with RTIC; debug/flash with probe-rs; beware global alloc (linked_list_allocator) if using `alloc`.
- Avoid/Anti-pattern: unwinding panics in no_std; blocking busy-waits without timers; tight coupling to a single MCU.

A.165 NATS JetStream At-least-once Messaging
- Use when: you need durable streams, retention, and at-least-once delivery on NATS.
- Context: create streams, use durable consumers with explicit acks; size ack wait/timeouts; apply backpressure; idempotent handlers (dedupe keys/ids).
- Avoid/Anti-pattern: forgetting to ack; assuming exactly-once; unbounded pull batches; ignoring durable consumer state.

```rust path=null start=null
// sync nats crate example (JetStream)
let nc = nats::connect("nats://127.0.0.1:4222")?;
let js = nats::jetstream::new(nc);
js.add_stream(nats::jetstream::StreamConfig {
    name: "events".into(), subjects: vec!["events.*".into()], ..Default::default()
})?;
let mut sub = js.subscribe("events.foo")?; // durable can be configured via consumer APIs
js.publish("events.foo", "hello")?;
if let Some(msg) = sub.next() { msg.ack()?; }
```

A.166 Kafka Clients with rdkafka: Producers, Consumers, Commits
- Use when: you need Kafka’s partitions, consumer groups, and backpressure.
- Context: prefer `StreamConsumer` to integrate with async; use `FutureProducer` for backpressure-aware sends; choose manual/auto commit explicitly.
- Avoid/Anti-pattern: ignoring rebalance events; synchronous commits on hot path; assuming exactly-once without idempotent/transactional config.

```rust path=null start=null
use rdkafka::{ClientConfig, consumer::{CommitMode, StreamConsumer}, message::BorrowedMessage};
use futures::StreamExt;
let consumer: StreamConsumer = ClientConfig::new()
  .set("group.id", "g1").set("bootstrap.servers", "localhost:9092")
  .create()?;
consumer.subscribe(&["input"])?;
let mut stream = consumer.stream();
while let Some(Ok(m)) = stream.next().await {
    handle(&m.payload().unwrap_or(&[]));
    consumer.commit_message(&m, CommitMode::Async)?;
}
```

A.167 Redis Connection Pooling with deadpool-redis
- Use when: many tasks share Redis connections under async.
- Context: create a pool, get short-lived connections, configure timeouts/max size; prefer commands via `redis::cmd` and release promptly.
- Avoid/Anti-pattern: holding connections across long `.await` chains; unbounded pools; blocking operations with pooled conns.

```rust path=null start=null
use deadpool_redis::{Config, Runtime};
use redis::AsyncCommands;
let cfg = Config::from_url("redis://127.0.0.1/");
let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
let mut conn = pool.get().await?;
conn.set::<_, _, ()>("k", "v").await?;
let v: String = conn.get("k").await?;
```

A.168 Search Clients: OpenSearch/Elasticsearch in Rust
- Use when: indexing/searching documents from Rust services.
- Context: use official clients (`opensearch`, `elasticsearch`) with typed request builders; prefer `serde_json::json!`/typed structs; handle 429/backoff.
- Avoid/Anti-pattern: string-concatenated JSON; ignoring HTTP errors/timeouts; bulk without chunking.

```rust path=null start=null
use opensearch::{OpenSearch, http::transport::Transport};
use serde_json::json;
let transport = Transport::single_node("http://localhost:9200")?;
let client = OpenSearch::new(transport);
let resp = client.index(opensearch::IndexParts::IndexId("logs", "1"))
  .body(json!({"msg":"hello","ts":123}))
  .send().await?;
```

A.169 Compact Binary with rmp-serde (MessagePack)
- Use when: compact, fast serialization with Serde is needed.
- Context: prefer `rmp-serde` encode/decode; cap message sizes; validate inputs; consider schema/version fields.
- Avoid/Anti-pattern: assuming map key order; unbounded nested structures; deserializing to untrusted types.

```rust path=null start=null
use rmp_serde::{encode::to_vec_named, decode::from_slice};
#[derive(serde::Serialize, serde::Deserialize)]
struct Event { id: u64, msg: String }
let bytes = to_vec_named(&Event { id: 1, msg: "hi".into() })?;
let evt: Event = from_slice(&bytes)?;
```

A.170 Refutable vs Irrefutable Patterns: match, if let, while let
- Use when: choosing control flow to destructure values safely and clearly.
- Context: use `match` for exhaustive handling; `if let` for refutable single-branch matches; `while let` for iterator-like loops; prefer irrefutable patterns in `let` bindings.
- Avoid/Anti-pattern: using `if let` where multiple cases need handling; non-exhaustive `match` on public enums without `_`/`..` or `#[non_exhaustive]` awareness.

```rust path=null start=null
// if let: handle one case
if let Some(x) = maybe { use_x(x); }

// while let: drain until pattern fails
while let Some(item) = iter.next() { process(item); }

// match: exhaustive handling
match res {
    Ok(v) => use_v(v),
    Err(e) => handle(e),
}
```

A.171 Zero-Sized Types (ZST) and Unit () for Markers and States
- Use when: encode type-level information without runtime cost.
- Context: define marker structs/enums with no fields; use `PhantomData<T>` to express ownership/variance; rely on ZST size 0, align 1 (unit) semantics.
- Avoid/Anti-pattern: storing ZSTs in large collections expecting memory usage; depending on layout of ZST-containing structs beyond guarantees.

```rust path=null start=null
use core::marker::PhantomData;
struct ReadOnly;
struct Buffer<T, Mode> { data: Vec<T>, _mode: PhantomData<Mode> }
let _buf: Buffer<u8, ReadOnly> = Buffer { data: vec![], _mode: PhantomData };
```

A.172 DST and Wide Pointers: slices, str, and dyn Trait (FFI-safe forms)
- Use when: handling dynamically sized types and trait objects.
- Context: `&[T]`, `&str`, and `&dyn Trait` are wide pointers (ptr+len or ptr+vtable); never expose them directly over FFI—use `(ptr,len)` or concrete reprs.
- Avoid/Anti-pattern: `extern "C" fn foo(s: &str)`; passing `&dyn Trait` across FFI; assuming layout of fat pointers.

```rust path=null start=null
#[repr(C)]
pub struct SliceU8 { ptr: *const u8, len: usize }
#[no_mangle]
pub extern "C" fn take_bytes(s: SliceU8) { /* safe FFI shape for &[u8] */ }
```

A.173 Uninhabited Types: empty enums and never type
- Use when: model impossible states or functions that never return.
- Context: `!` is the canonical uninhabited type; empty enums can model uninhabited generics (e.g., `Result<T, Void>`); convert `Result<T, Void>` -> `T` safely.
- Avoid/Anti-pattern: constructing values of empty enums; exposing uninhabited types in FFI.

```rust path=null start=null
enum Void {}
fn into_ok<T>(r: Result<T, Void>) -> T { match r { Ok(v) => v, Err(e) => match e {} } }
```

A.174 Wildcard '_' vs Rest '..' Patterns (forward-compat)
- Use when: destructuring while ignoring fields and preserving forward compatibility.
- Context: use `_` for a single field; `..` to ignore the rest (especially with `#[non_exhaustive]` structs/enums); pair with explicit bindings you need.
- Avoid/Anti-pattern: listing all fields in public patterns that may change; relying on private fields without `..`.

```rust path=null start=null
struct S { a: u32, b: u32, c: u32 }
let s = S { a: 1, b: 2, c: 3 };
let S { a, .. } = s; // ignore b, c
let S { b: _, c, .. } = s; // ignore b explicitly, bind c
```

A.175 RTT + defmt Logging for Embedded Debugging
- Use when: need low-overhead logging over SWD/JTAG without UART.
- Context: use `defmt` with `probe-rs` RTT; add `panic-probe` for panic reporting; keep logs concise; strip in release if needed.
- Avoid/Anti-pattern: semihosting in production; blocking prints in interrupts; verbose logs in time-critical paths.

```rust path=null start=null
// Cargo.toml (bins)
// defmt = "*", rtt-target = "*", panic-probe = { version = "*", features = ["print-defmt"] }
#[defmt::panic_handler]
fn panic() -> ! { panic_probe::hard_fault() }
#[defmt::timestamp]
fn ts() -> u64 { 0 }
```

A.176 Flash/Debug Cycle with probe-rs and cargo-flash
- Use when: flashing and running firmware quickly without vendor IDEs.
- Context: prefer `cargo-flash` for one-step build+flash; use `probe-rs run` for RTT logs; select chip via `--chip`.
- Avoid/Anti-pattern: manual openocd+gdb for simple workflows; relying on unstable scripts.

```sh path=null start=null
cargo flash --chip STM32F401RETx --release --example blinky
probe-rs run --chip STM32F401RETx target/thumbv7em-none-eabihf/release/app
```

A.177 Cross-compiling to Cortex-M (thumbv7em-none-eabihf)
- Use when: building no_std firmware for ARM Cortex-M4F/M7F.
- Context: `rustup target add thumbv7em-none-eabihf`; set target in `.cargo/config.toml`; ensure linker script `memory.x`; use `cortex-m-rt`/`rtic`/`embassy` as needed.
- Avoid/Anti-pattern: building without specifying target; missing `memory.x` causing link errors; unwinding panics on bare metal.

```toml path=null start=null
# .cargo/config.toml
[build]
target = "thumbv7em-none-eabihf"
```

A.178 Async Embedded with Embassy Executor
- Use when: cooperative async on MCUs with timers/interrupts.
- Context: use `embassy-executor` with `#[embassy_executor::main]`; never block; use embassy time drivers; integrate interrupts via `#[interrupt]`/`Spawner`.
- Avoid/Anti-pattern: blocking delays inside async tasks; mixing multiple executors without care.

```rust path=null start=null
#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    // spawn tasks, use embassy_time::Timer
}
```

A.179 Linker Scripts and Memory Layout (memory.x)
- Use when: placing code/data in specific regions and defining stacks/ISRs.
- Context: provide `memory.x` and reference it via `linker = "flip-link"` or build script; define FLASH/RAM, stack, and vector table; use `#[link_section]` sparingly.
- Avoid/Anti-pattern: relying on default layouts; hardcoding addresses in code; missing sections leading to undefined behavior.

```ld path=null start=null
/* memory.x */
MEMORY { FLASH : ORIGIN = 0x08000000, LENGTH = 512K; RAM : ORIGIN = 0x20000000, LENGTH = 128K }
PROVIDE(_stack_start = ORIGIN(RAM) + LENGTH(RAM));
```

A.180 Work-stealing Schedulers with crossbeam-deque
- Use when: distributing tasks across threads with minimal contention.
- Context: use a global Injector and per-thread Worker queues (LIFO for cache locality); idle threads steal from others’ tails via Stealer; batch stealing reduces contention.
- Avoid/Anti-pattern: single global queue bottlenecks; pushing all tasks to injector under load; heavy locking around queues.

```rust path=null start=null
use crossbeam_deque::{Injector, Worker, Stealer, Steal};
let injector = Injector::new();
let workers: Vec<_> = (0..num_cpus::get()).map(|_| Worker::new_lifo()).collect();
let stealers: Vec<Stealer<_>> = workers.iter().map(|w| w.stealer()).collect();
// spawn workers: pop local, else steal batch from injector or others
```

A.181 Epoch-based Reclamation (crossbeam-epoch) Pin/Collect Pattern
- Use when: building lock-free structures that remove nodes safely.
- Context: pin the epoch when accessing shared pointers; retire removed nodes to be reclaimed when no thread holds a pin; prefer default Collector unless you need custom domains.
- Avoid/Anti-pattern: freeing memory without retire; accessing pointers outside a pin; mixing GC domains.

```rust path=null start=null
use crossbeam_epoch as epoch;
let guard = epoch::pin();
let shared: epoch::Atomic<usize> = epoch::Atomic::null();
let ptr = shared.load(epoch::Acquire, &guard);
// ... remove node ...
unsafe { guard.defer_destroy(ptr) };
```

A.182 Unsafe Send/Sync: Design and Manual impl Rules
- Use when: declaring a type safe to send/share across threads beyond compiler inference.
- Context: Only `unsafe impl Send/Sync for T {}` when you can prove thread-safety invariants (no non-thread-safe interior mutability; raw pointers protected by atomics/locks; lifetimes not leaked). Prefer composing Send/Sync fields.
- Avoid/Anti-pattern: blanket unsafe impls; using `Cell/RefCell` across threads; ignoring drop ordering and aliasing.

```rust path=null start=null
unsafe impl<T: Send> Send for MyType<T> {}
unsafe impl<T: Sync> Sync for MyType<T> {}
// Justify invariants in docs: why concurrent access is safe.
```

A.183 Deterministic Concurrency Testing with loom
- Use when: verifying lock-free invariants and ordering under all interleavings.
- Context: model with loom types (Arc, Atomic*); keep tests fully deterministic; note SeqCst ops are treated as AcqRel (use fence(SeqCst) if needed); cap state space.
- Avoid/Anti-pattern: randomness/timers/I/O in loom tests; relying on OS scheduler; ignoring false positives from SeqCst downgrades.

```rust path=null start=null
loom::model(|| {
    use loom::{sync::{Arc, atomic::{AtomicUsize, Ordering::*}}, thread};
    let n = Arc::new(AtomicUsize::new(0));
    let a = n.clone(); let b = n.clone();
    let t1 = thread::spawn(move || { let cur = a.load(Acquire); a.store(cur+1, Release); });
    let t2 = thread::spawn(move || { let cur = b.load(Acquire); b.store(cur+1, Release); });
    t1.join().unwrap(); t2.join().unwrap();
    assert_eq!(n.load(Relaxed), 2);
});
```

A.184 Channel Choices: std::mpsc vs crossbeam-channel vs flume
- Use when: choosing message-passing primitives for threads.
- Context: std::mpsc is baseline (MPSC, simple); crossbeam-channel adds MPMC, select!, performance; flume offers MPMC with async support, deadlines, rendezvous, and Selector.
- Avoid/Anti-pattern: unbounded queues in high-throughput paths; blocking sends without timeouts; mixing channel ecosystems without need.

```rust path=null start=null
// flume rendezvous + select-like API
let (tx1, rx1) = flume::bounded::<u32>(0); // rendezvous
let (tx2, rx2) = flume::bounded::<u32>(1024);
flume::Selector::new()
    .recv(&rx1, |v| println!("r1: {}", v))
    .recv(&rx2, |v| println!("r2: {}", v))
    .wait();
```

0A. WORKSPACE AND DEPENDENCY MANAGEMENT
--------------------------------
0A.1. Workspace-level dependency declaration for version consistency
0A.2. Module-level re-exports via lib.rs/mod.rs for clean public APIs
0A.3. Feature flags for optional dependencies
0A.4. Shared dependency versioning through workspace inheritance
0A.5. Path-based local dependencies for monorepo development
0A.6. Public API organization through prelude modules
0A.7. Conditional compilation with cfg attributes
0A.8. Dependency groups by feature sets
0A.9. Version compatibility through semver
0A.10. Cross-crate type sharing via workspace-common modules
0A.11. Clean Build Pattern
      - Regular execution of 'cargo clean' and './mach clean'
      - Clean before switching branches or major dependency changes
      - Clean when encountering mysterious build or dependency errors
      - Clean when updating workspace-level dependency configurations
      - Verify clean build state before running critical tests

1. OWNERSHIP AND BORROWING PATTERNS
----------------------------------
1.1. Clone-on-Write (Cow) for optional data ownership
1.2. Passing references instead of moving values
1.3. Using Arc for shared ownership in concurrent contexts
1.4. Implementing Clone selectively
1.5. Taking owned values in constructors
1.6. Borrowing in method arguments
1.7. Using Box<dyn Trait> for trait objects
1.8. Smart pointer patterns (Rc, Arc, Box)
1.9. Temporary ownership with mem::replace
1.10. Moving out of collections safely

2. ERROR HANDLING PATTERNS
-------------------------
2.1. Custom error types with thiserror
2.2. Using anyhow for application errors
2.3. Question mark operator chaining
2.4. Context addition with .context() or .with_context()
2.5. Custom Error type with From implementations
2.6. Result wrapping for fallible operations
2.7. Nested error handling with map_err
2.8. Error source chaining
2.9. Using Option for nullable values
2.10. Fallback patterns with unwrap_or_else

3. BUILDER PATTERNS
------------------
3.1. Builder pattern for complex object construction
3.2. Fluent interfaces
3.3. Default trait implementation
3.4. Type-state builders
3.5. Validate-before-build pattern
3.6. Optional field builders
3.7. Consuming builders
3.8. Generic builders
3.9. Builder with phantom types
3.10. Nested builders

4. RESOURCE MANAGEMENT
---------------------
4.1. RAII pattern
4.2. Drop trait implementation
4.3. Guard patterns
4.4. Cleanup in reverse order
4.5. Temporary resource allocation
4.6. Resource pools
4.7. Connection management
4.8. File handle management
4.9. Memory management patterns
4.10. Resource limitation patterns

5. CONCURRENCY PATTERNS
----------------------
5.1. Actor pattern
5.2. Message passing
5.3. Mutex guard pattern
5.4. RwLock patterns
5.5. Channel patterns (mpsc)
5.6. Thread pool implementations
5.7. Async/await patterns
5.8. Future combinators
5.9. Tokio runtime patterns
5.10. Parking_lot synchronization

6. TRAIT PATTERNS
----------------
6.1. Extension traits
6.2. Marker traits
6.3. Associated type patterns
6.4. Trait bounds composition
6.5. Conditional trait implementation
6.6. Sealed traits
6.7. Auto traits
6.8. Trait objects
6.9. Generic traits
6.10. Default trait implementations

7. TYPE SYSTEM PATTERNS
----------------------
7.1. Newtype pattern
7.2. Phantom data
7.3. Type-state programming
7.4. Zero-sized types
7.5. Marker types
7.6. Type-level programming
7.7. Generic type parameters
7.8. Associated types
7.9. Type aliases
7.10. Const generics

8. MEMORY OPTIMIZATION
---------------------
8.1. Small string optimization
8.2. Stack allocation preferences
8.3. Arena allocation
8.4. Memory pooling
8.5. Zero-copy parsing
8.6. Packed structures
8.7. Cache-friendly data layouts
8.8. Memory mapping
8.9. Custom allocators
8.10. Slice optimization

9. API DESIGN PATTERNS
---------------------
9.1. Into/From conversions
9.2. TryFrom/TryInto for fallible conversions
9.3. AsRef/AsMut traits
9.4. IntoIterator implementation
9.5. Display and Debug implementations
9.6. Visitor pattern
9.7. Command pattern
9.8. Factory pattern
9.9. Strategy pattern
9.10. Adapter pattern

10. MACRO PATTERNS
-----------------
10.1. Declarative macros
10.2. Procedural macros
10.3. Derive macros
10.4. Attribute macros
10.5. Function-like macros
10.6. Internal rule patterns
10.7. Recursive macros
10.8. Token manipulation
10.9. Custom syntax extensions
10.10. Hygiene patterns

11. TESTING PATTERNS
-------------------
11.1. Unit test organization
11.2. Integration test patterns
11.3. Property-based testing
11.4. Test fixtures
11.5. Mock objects
11.6. Parameterized tests
11.7. Benchmark patterns
11.8. Test utilities
11.9. Assert macro patterns
11.10. Test harnesses

12. SAFETY PATTERNS
------------------
12.1. Safe wrapper types
12.2. Bounds checking
12.3. Panic guards
12.4. Memory safety patterns
12.5. Thread safety patterns
12.6. Safe abstractions over unsafe code
12.7. Invariant maintenance
12.8. Permission systems
12.9. Capability patterns
12.10. Validation chains

13. PERFORMANCE PATTERNS
-----------------------
13.1. Zero-cost abstractions
13.2. Static dispatch
13.3. Dynamic dispatch optimization
13.4. Lazy initialization
13.5. Caching patterns
13.6. Batch processing
13.7. SIMD optimization
13.8. Memory prefetching
13.9. Lock-free algorithms
13.10. Compile-time computation

14. ASYNC PATTERNS
-----------------
14.1. Stream processing
14.2. Async trait patterns
14.3. Futures composition
14.4. Async resource management
14.5. Backpressure handling
14.6. Timeout patterns
14.7. Rate limiting
14.8. Circuit breaker pattern
14.9. Async initialization
14.10. Error propagation in async

15. COLLECTIONS PATTERNS
-----------------------
15.1. Custom iterators
15.2. Collection transformations
15.3. Efficient searching
15.4. Sorting strategies
15.5. Custom collection types
15.6. Thread-safe collections
15.7. Specialized containers
15.8. Index access patterns
15.9. Collection views
15.10. Cursor patterns

16. MODULE ORGANIZATION
----------------------
16.1. Public API design
16.2. Internal module structure
16.3. Feature flagging
16.4. Conditional compilation
16.5. Platform-specific code
16.6. Library organization
16.7. Dependency management
16.8. Version compatibility
16.9. Documentation organization
16.10. Example code structure

17. SERIALIZATION PATTERNS
-------------------------
17.1. Serde implementations
17.2. Custom serialization
17.3. Versioned serialization
17.4. Binary formats
17.5. Text formats
17.6. Schema evolution
17.7. Validation during deserialization
17.8. Efficient serialization
17.9. Format conversion
17.10. Type-driven serialization

18. NETWORKING PATTERNS
----------------------
18.1. Connection pooling
18.2. Protocol implementations
18.3. Async networking
18.4. Request/response patterns
18.5. Streaming protocols
18.6. Connection management
18.7. Retry mechanisms
18.8. Load balancing
18.9. Service discovery
18.10. Protocol buffers

19. FFI PATTERNS
---------------
19.1. C API wrappers
19.2. Memory management
19.3. Error handling
19.4. Callback patterns
19.5. Type conversion
19.6. String handling
19.7. Array handling
19.8. Function exports
19.9. Platform specifics
19.10. Safety boundaries

20. OPTIMIZATION PATTERNS
------------------------
20.1. Compile-time optimization
20.2. Runtime optimization
20.3. Memory optimization
20.4. CPU cache optimization
20.5. Algorithm selection
20.6. Data structure choice
20.7. Parallel processing
20.8. Resource pooling
20.9. Load distribution
20.10. Bottleneck elimination

21. ASYNC RUNTIME INTERNALS
--------------------------
21.1. Task scheduler implementation
21.2. Waker implementation patterns
21.3. Reactor patterns
21.4. Poll function optimization
21.5. Future state machines
21.6. Task queue management
21.7. Work-stealing schedulers
21.8. Timer wheel implementation
21.9. IO event notification systems
21.10. Task cancellation mechanisms

22. ZERO-COST ABSTRACTION PATTERNS
--------------------------------
22.1. Compile-time dispatch tables
22.2. Static virtual dispatch
22.3. Const generics optimization
22.4. Enum optimization patterns
22.5. Monomorphization strategies
22.6. Inline assembly integration
22.7. SIMD abstraction layers
22.8. Branch prediction hints
22.9. Memory alignment optimization
22.10. Dead code elimination patterns

23. ASYNC MIDDLEWARE PATTERNS
---------------------------
23.1. Tower layer implementation
23.2. Service trait patterns
23.3. Middleware chaining
23.4. Request/response transformation
23.5. Async interceptors
23.6. Filter chains
23.7. Middleware state management
23.8. Cross-cutting concerns
23.9. Conditional middleware
23.10. Middleware composition

24. RUNTIME REFLECTION PATTERNS
-----------------------------
24.1. Type ID manipulation
24.2. Dynamic type registration
24.3. Type metadata handling
24.4. Runtime type checking
24.5. Dynamic dispatch tables
24.6. Type erasure techniques
24.7. Trait object manipulation
24.8. Virtual method tables
24.9. Dynamic loading patterns
24.10. Type reconstruction

25. ADVANCED MACRO PATTERNS
-------------------------
25.1. Token tree manipulation
25.2. Macro hygiene management
25.3. Recursive macro expansion
25.4. Custom syntax parsing
25.5. Macro debugging patterns
25.6. Cross-platform macros
25.7. Conditional compilation
25.8. Code generation patterns
25.9. Macro export patterns
25.10. Macro documentation

26. ASYNC IO PATTERNS
-------------------
26.1. Zero-copy IO operations
26.2. Buffered IO abstractions
26.3. Async file operations
26.4. Network buffer management
26.5. IO completion ports
26.6. Scatter-gather IO
26.7. Direct memory access
26.8. IO uring integration
26.9. Async IO queues
26.10. IO prioritization

27. LOCK-FREE PATTERNS
--------------------
27.1. CAS operations
27.2. Memory ordering
27.3. Atomic reference counting
27.4. Lock-free queues
27.5. Wait-free algorithms
27.6. Memory barriers
27.7. ABA problem solutions
27.8. Lock-free data structures
27.9. Hazard pointers
27.10. Epoch-based reclamation

28. ASYNC STREAM PATTERNS
-----------------------
28.1. Back-pressure implementation
28.2. Stream buffering
28.3. Stream transformation
28.4. Stream composition
28.5. Stream splitting
28.6. Stream multiplexing
28.7. Stream rate limiting
28.8. Stream windowing
28.9. Stream error handling
28.10. Stream cancellation

29. PLATFORM ABSTRACTION
----------------------
29.1. OS API abstraction
29.2. System call wrapping
29.3. Platform-specific features
29.4. Conditional compilation
29.5. Feature detection
29.6. ABI compatibility
29.7. Cross-platform IO
29.8. Platform-specific optimization
29.9. Syscall abstraction
29.10. Platform capability detection

30. ADVANCED TYPE SYSTEM
----------------------
30.1. Higher-kinded types simulation
30.2. GATs implementation
30.3. Type-level computation
30.4. Type state machines
30.5. Dependent type patterns
30.6. Type-level integers
30.7. Type families
30.8. Associated type constructors
30.9. Type-level proofs
30.10. Type inference helpers

31. OPTION AND NULL SAFETY PATTERNS
--------------------------------
31.1. Combinators Over Matching
     - Use .map() when transforming Some values
     - Use .and_then() for chaining Option-returning operations
     - Use .or_else() for fallback computations
     - Use .unwrap_or_else() for lazy default values

31.2. Collection Operations
     - Use .filter_map() instead of filter().map()
     - Use .and_then() for flattening nested Options
     - Use .zip() to combine two Options

31.3. Early Returns and Guards
     - Return None early in functions
     - Use if let Some(x) for single-case matching
     - Chain .ok_or()/.ok_or_else() when converting to Result

31.4. Default Values
     - Use .unwrap_or(default) for simple defaults
     - Use .unwrap_or_else(|| expensive_computation()) for lazy defaults
     - Use .unwrap_or_default() for types implementing Default

31.5. Pattern Matching Best Practices
     - Match on multiple Options using tuple patterns
     - Use @ bindings to reference matched values
     - Prefer if let over match for single patterns

31.6. Option Construction
     - Use Some(val) explicitly for clarity
     - Use None::<Type> when type inference fails
     - Convert from nullable types using .map(|x| Some(x))

31.7. Composition Patterns
     - Chain .as_ref() for borrowing Option contents
     - Use .as_mut() for mutable borrowing
     - Combine with Result using .ok() and .transpose()

31.8. When to Use Each Pattern:
     ┌────────────────────┬──────────────────────────────────────┐
     │ Pattern            │ When to Use                          │
     ├────────────────────┼──────────────────────────────────────┤
     │ .map()             │ Transform Some value without nesting  │
     │ .and_then()        │ Chain operations that return Option   │
     │ .filter()          │ Conditionally keep Some values       │
     │ .or()/.or_else()   │ Provide fallback Options            │
     │ if let Some()      │ Single-case pattern matching        │
     │ match              │ Multiple cases or complex logic      │
     │ .unwrap_or()       │ Simple default values               │
     │ .unwrap_or_else()  │ Expensive default computations      │
     └────────────────────┴──────────────────────────────────────┘

31.9. Anti-patterns to Avoid
     - Avoid .unwrap() in production code
     - Don't use .expect() unless truly impossible
     - Avoid nested match statements on Options
     - Don't use if x.is_some() { x.unwrap() }

31.10. Testing Patterns
     - Use assert_eq!(Some(expected), result)
     - Test None cases explicitly
     - Use Option::as_ref() in assertions

32. ASYNC CHANNEL PATTERNS
------------------------
32.1. Multi-producer channels
32.2. Bounded channel implementation
32.3. Priority channels
32.4. Channel selection
32.5. Channel composition
32.6. Channel broadcasting
32.7. Channel filtering
32.8. Channel transformation
32.9. Channel monitoring
32.10. Channel cleanup

33. UNSAFE CODE PATTERNS
----------------------
33.1. Safe abstraction boundaries
33.2. Pointer manipulation
33.3. Raw memory management
33.4. FFI boundary safety
33.5. Undefined behavior prevention
33.6. Memory mapping safety
33.7. Platform-specific unsafe
33.8. Atomic operation safety
33.9. Exception safety
33.10. Invariant maintenance

34. ASYNC EXECUTOR PATTERNS
-------------------------
34.1. Task spawning
34.2. Executor shutdown
34.3. Task prioritization
34.4. Resource limits
34.5. Executor metrics
34.6. Task grouping
34.7. Executor composition
34.8. Thread pool management
34.9. Work stealing
34.10. Task locality

35. ADVANCED TRAIT PATTERNS
-------------------------
35.1. Trait specialization
35.2. Trait aliases
35.3. Trait composition
35.4. Negative trait bounds
35.5. Conditional trait impl
35.6. Trait object safety
35.7. Associated type defaults
35.8. Trait upcasting
35.9. Trait downcasting
35.10. Trait coherence

36. ASYNC NETWORKING PATTERNS
---------------------------
36.1. Protocol implementation
36.2. Connection management
36.3. TLS integration
36.4. Proxy patterns
36.5. Network timeouts
36.6. Connection pooling
36.7. Protocol negotiation
36.8. Network error handling
36.9. Keep-alive management
36.10. Connection backoff

37. COMPILE-TIME VALIDATION
-------------------------
37.1. Type-level constraints
37.2. Const evaluation
37.3. Static assertions
37.4. Build-time checks
37.5. Compile-time verification
37.6. Type system proofs
37.7. Const generics validation
37.8. Macro-time validation
37.9. Link-time optimization
37.10. Dead code detection

38. ASYNC STATE MANAGEMENT
------------------------
38.1. State machine implementation
38.2. Shared state access
38.3. State synchronization
38.4. State transition validation
38.5. State persistence
38.6. State recovery
38.7. State snapshot
38.8. State migration
38.9. State replication
38.10. State consistency

39. ADVANCED MEMORY PATTERNS
--------------------------
39.1. Custom allocator implementation
39.2. Memory pool management
39.3. Garbage collection
39.4. Reference counting
39.5. Memory fence patterns
39.6. Cache line optimization
39.7. Memory prefetching
39.8. Stack vs heap decisions
39.9. Memory compaction
39.10. Memory defragmentation

40. ASYNC TESTING PATTERNS
------------------------
40.1. Async test harness
40.2. Mock async services
40.3. Async assertions
40.4. Time manipulation
40.5. Race condition testing
40.6. Async property testing
40.7. Network simulation
40.8. Async benchmarking
40.9. Fault injection
40.10. Concurrency testing

41. LIBRARY API DESIGN
--------------------
41.1. Versioning strategies
41.2. Breaking change management
41.3. API stability guarantees
41.4. Feature flagging
41.5. Documentation generation
41.6. Error type design
41.7. Type system ergonomics
41.8. Builder pattern design
41.9. Extension trait design
41.10. Conditional compilation

Each of these patterns represents advanced techniques commonly used in building production-grade async Rust libraries like Tokio and Axum. They focus on performance, safety, and maintainability while providing powerful abstractions for users.
