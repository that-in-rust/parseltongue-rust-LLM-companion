---
name: idiomatic-rust-coding
description: Write production-quality Rust code using layered architecture (L1 Core / L2 Standard / L3 External), 125+ curated idioms, structured error handling, concurrency patterns, and performance optimization. Covers the vital 20% of Rust patterns that enable 99% of production code with compile-first success.
---

# Idiomatic Rust Coding

## Overview

Use this skill when implementing Rust code. It distills 180+ curated idioms into a layered reference organized by when to use each pattern, the context it applies in, and the anti-patterns to avoid.

Core thesis: ~20% of Rust patterns enable writing 99% of production code with minimal bugs. Following these patterns achieves an average of 1.6 compile attempts vs 4.9 without them (67% faster development).

## When To Use

Use this skill when:
- Implementing a feature after specs are written (use `ai-native-spec-writing` for specs).
- Choosing between ownership strategies, error handling approaches, or concurrency models.
- Reviewing Rust code for idiomatic correctness.
- Deciding on data structures, smart pointers, or async patterns.
- Writing tests, benchmarks, or CI quality gates.

Do not use this skill for writing requirements or naming conventions (use `ai-native-spec-writing` instead).

## Layered Architecture: L1 / L2 / L3

Structure all systems in layers with clear boundaries. Never mix L3 dependencies into L1 core.

### L1 Core (no_std compatible)

Ownership, lifetimes, traits, Result/Option, RAII, newtype pattern.

```rust
// L1: Pure core, no external dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(u64);

impl EntityId {
    pub fn new(id: u64) -> Self { Self(id) }
    pub fn as_u64(self) -> u64 { self.0 }
}
```

### L2 Standard (stdlib idioms)

Collections, iterators, smart pointers (Rc/Arc), thread safety (Send/Sync).

```rust
// L2: Uses stdlib idioms
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct EntityRegistry {
    by_id: HashMap<EntityId, Arc<Entity>>,
    by_name: HashMap<String, EntityId>,
    implementations: HashSet<EntityId>,
}
```

### L3 External (ecosystem)

Async/await (Tokio), serialization (Serde), databases (CozoDB/SQLx), web frameworks (Axum).

```rust
// L3: External dependencies
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use axum::Json;

pub struct AppState {
    registry: Arc<RwLock<EntityRegistry>>,
}
```

## Dependency Injection for Testability

Every component depends on traits, not concrete types:

```rust
// Bad: Hard dependency
pub struct Service { database: PgPool }

// Good: Trait-based dependency
pub struct Service<D: Database> { database: Arc<D> }
type ProductionService = Service<PgDatabase>;
type TestService = Service<MockDatabase>;
```

## Error Handling

- **Libraries** (`parseltongue-core`): Use `thiserror` for structured error enums.
- **Applications** (CLI/tools): Use `anyhow` for context propagation.

```rust
// Library: thiserror
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("invalid header: {0}")]
    InvalidHeader(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

// Application: anyhow
pub async fn process_request(req: Request) -> anyhow::Result<Response> {
    let data = fetch_data(&req.id)
        .await
        .with_context(|| format!("Failed to fetch data for request {}", req.id))?;
    Ok(Response::new(data))
}
```

Never return `anyhow::Error` from library public APIs. Never use `.unwrap()` or `.expect()` in production code.

## RAII Resource Management

All resources automatically managed with Drop implementations:

```rust
pub struct ResourceManager {
    connection: Option<Connection>,
    _cleanup: CleanupGuard,
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            if let Err(e) = conn.close() {
                eprintln!("Failed to close connection: {}", e);
            }
        }
    }
}
```

## Curated Idiom Categories

The full idiom reference is in `references/idiom-catalog.md`. Below is the index organized by category.

### Expressions and Control Flow (A.1, A.7, A.170-A.174)

- Prefer expression-oriented code: `if/else` and `match` as expressions.
- Use Option/Result combinators (`map`, `and_then`, `transpose`) over nested matching.
- Understand refutable vs irrefutable patterns, wildcard `_` vs rest `..`.

### API Design (A.2-A.5, A.41, A.92-A.93, A.113)

- Accept slices and traits in public APIs: `&[T]`, `&str`, `AsRef<T>`.
- Use `Cow<'_, str>` for conditional ownership on hot paths.
- Implement `From`/`TryFrom` for conversions, newtype pattern for domain invariants.
- Return `impl Iterator` from APIs to expose lazy pipelines.
- Follow C-CASE naming conventions; use sealed traits to control external impls.

### Error Handling (A.6, A.29, A.31, A.60, A.90, A.112, A.27)

- `thiserror` for libraries, `anyhow` for applications.
- Propagate with `?` and `.context()` / `.with_context()`.
- Use `try_fold`/`try_for_each` for fallible iterator pipelines.
- Use `color-eyre`/`miette` for rich diagnostics in binaries only.

### Interior Mutability and Synchronization (A.8, A.117, A.122, A.131-A.133)

- `Cell`/`RefCell` for single-thread; `Mutex`/`RwLock` for cross-thread.
- `parking_lot` for smaller, faster locks on contended paths.
- `crossbeam-channel` for MPMC; `OnceLock`/`LazyLock` for one-time init.
- Never hold locks across `.await`.

### Trait Design (A.9-A.10, A.46-A.49, A.61-A.63, A.79-A.80)

- Generics for hot paths; trait objects for heterogeneous collections.
- Understand `Pin`/`Unpin` for self-referential types.
- Use HRTBs (`for<'a>`) for callback APIs; GATs for lending patterns.
- Async in traits: prefer stable `async fn` for static dispatch; `Pin<Box<dyn Future>>` for dynamic.

### Async Patterns (A.11-A.12, A.25, A.52, A.70, A.75-A.76, A.87)

- Never block in async context; use `spawn_blocking` for CPU work.
- Bounded channels with backpressure; `CancellationToken` for cooperative shutdown.
- Cancel-safe `select!`: futures must not own partially-consumed state.
- `JoinSet` for structured concurrency; drop cancels outstanding tasks.

### Iterators and Collections (A.33-A.44, A.50, A.53-A.58)

- Understand `IntoIterator`/`iter()`/`iter_mut()` semantics.
- Prefer iterator combinators over manual loops for clarity and fusion.
- Choose `Fn`/`FnMut`/`FnOnce` bounds precisely.
- Default to `Vec`; `VecDeque` for queues; `HashMap` for O(1); `BTreeMap` for ordering.
- Use `with_capacity` for known sizes; `IndexMap` for stable insertion order.

### Testing (A.16-A.20, A.71, A.28, A.154)

- Doctests as executable contracts.
- Property-based testing with `proptest` for large input spaces.
- Snapshot testing with `expect-test` for parsers and formatters.
- Concurrency model checking with `loom`.
- Fuzzing with `cargo-fuzz` for untrusted input boundaries.
- Coverage gating with `cargo-llvm-cov`.

### Macros (A.77, A.101-A.108, A.150)

- Prefer `macro_rules!` when sufficient; proc-macros for derive/attribute/function-like.
- Use `$crate::path` and absolute paths for hygiene.
- `syn` + `quote` + `proc-macro2` pipeline for proc-macros.
- Emit `compile_error!` with spans instead of panicking.

### Performance (A.3, A.55-A.56, A.116, A.123, A.126, A.151-A.153)

- `Cow` for conditional ownership; zero-allocation string processing.
- `SmallVec` for small fixed upper bounds on hot paths.
- Avoid unnecessary cloning; prefer borrowing and shared ownership.
- Manage monomorphization bloat with strategic trait objects.
- `moka` for concurrent caching; `jemalloc`/`mimalloc` for global allocator.
- LTO, PGO, and inlining for release builds.

### Unsafe and FFI (A.32, A.59, A.67, A.82-A.86, A.91, A.111, A.143)

- Document invariants with `// SAFETY:` comments.
- Run `cargo +nightly miri test` in CI for UB detection.
- `repr(C)`/`repr(transparent)` for FFI layout.
- `Option<NonNull<T>>` for nullable pointer optimization.
- Never let Rust panics cross FFI; use `catch_unwind`.

### CI and Quality (A.13-A.15, A.21-A.22, A.114, A.148-A.149)

- `cargo clippy --all-targets --all-features -- -D warnings` as gate.
- `cargo fmt --all -- --check` as non-negotiable.
- `cargo-audit`/`cargo-deny` for supply-chain policies.
- `tracing` with `#[instrument]` for structured observability.
- Pin toolchains per project with `rust-toolchain.toml`.

## Production Templates

### Axum Microservice

```rust
use axum::{extract::State, routing::{get, post}, Router};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(|| async { "OK" }))
        .route("/api/users", post(create_user))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(state)
}
```

### Actor Pattern for State Management

```rust
use tokio::sync::{mpsc, oneshot};

pub enum StateMessage<T> {
    Get { respond_to: oneshot::Sender<T> },
    Update {
        updater: Box<dyn FnOnce(&mut T) + Send>,
        respond_to: oneshot::Sender<Result<(), StateError>>,
    },
}
```

## LLM Response Contract

When implementing with this skill, ensure:
- Layer boundaries respected (L1/L2/L3 never mixed).
- Error handling uses the correct strategy (thiserror vs anyhow).
- All resources managed with RAII.
- Concurrency validated with stress tests.
- Performance claims backed by automated tests.
- All names follow 4WNC (see `ai-native-spec-writing`).

## Resources

- Full idiom catalog: `references/idiom-catalog.md`
- Source: `agent-room-of-requirements/agents-used-202512/rust-coder-01.md` (Sections A.1-A.184)
- Source: `agent-room-of-requirements/agents-used-202512/notes01-agent.md` (Parts VI, VII)
- Related skill: `ai-native-spec-writing` (for requirements and naming)
- Related skill: `context-engineering-agents` (for agent design patterns)
