# Idiomatic Rust Idiom Catalog

> 125 curated idioms indexed by category from a source catalog of 184. Each entry specifies when to use and anti-patterns to avoid. The full source contains additional idioms in the A.1-A.184 range.
> Source: `agent-room-of-requirements/agents-used-202512/rust-coder-01.md`

---

## Expressions and Control Flow

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.1 | Expression-Oriented Code | Function naturally computes a value; prefer returning last expression | Unnecessary `mut` accumulators when `map/fold` suffice |
| A.7 | Option/Result Combinators | Simple transformations/chaining (`map`, `and_then`, `transpose`) | `.unwrap()`/`.expect()` in production; deep nested matches |
| A.170 | Refutable vs Irrefutable Patterns | `match`, `if let`, `while let` for conditional matching | Using irrefutable patterns where refutable is needed |
| A.171 | Zero-Sized Types (ZST) | Markers and state machines using `()` and empty structs | Allocating for marker types |
| A.172 | DST and Wide Pointers | Slices, `str`, and `dyn Trait` at FFI boundaries | Assuming thin pointer layout for DSTs |
| A.173 | Uninhabited Types | Empty enums and never type for impossible states | Encoding never-return via panics without documenting |
| A.174 | Wildcard vs Rest Patterns | `_` for single fields, `..` for forward-compatible struct matching | Matching all fields when only some are needed |

## API Design

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.2 | Accept Slices and Traits | Callers can pass many input types; `&[T]`, `&str`, `AsRef<T>` | Taking `&Vec<T>`/`&String` in signatures |
| A.3 | Clone-on-Write (Cow) | Sometimes borrow, sometimes need owned data; hot paths | Using `Cow` when data is always owned or always borrowed |
| A.4 | From/TryFrom/TryInto | Infallible/fallible conversions instead of ad-hoc constructors | Exposing `Result` for infallible cases |
| A.5 | Newtype Pattern | Enforce domain invariants; type-safe IDs, units, opaque handles | `type` alias where a distinct type is needed |
| A.41 | Return impl Iterator | Expose lazy pipelines without committing to concrete types | Returning concrete iterator types in public APIs |
| A.45 | Raw Identifiers | Cross-edition compatibility (`r#try`, `r#match`) | Renaming public symbols when `r#` solves it |
| A.92 | API Prelude Re-exports | Improve ergonomics for frequent traits/types | Deep import paths for essential traits |
| A.93 | Lifetime Elision Rules | Simple function signatures with references | Ambiguous signatures without explicit lifetimes |
| A.94 | Trait Object Lifetimes | Using `dyn Trait` references with correct lifetime bounds | Assuming trait object lifetimes are inferred as desired |
| A.113 | API Guidelines | Stabilizing public APIs; C-CASE, additive features, sealed traits | Breaking SemVer via trait changes |

## Error Handling

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.6 | thiserror vs anyhow | Libraries: precise Error enums; Applications: aggregate with context | Exposing `anyhow::Error` in public APIs |
| A.27 | Rich Diagnostics (color-eyre/miette) | Human-friendly error reports in binaries | Using these types in public library APIs |
| A.29 | Result Ergonomics | Propagate errors with `?` and `.context()` | `.unwrap()` in non-test code; swallowing sources |
| A.31 | Library Error Design | Choosing error modeling for libraries vs applications | Returning `anyhow::Error` from library public APIs |
| A.60 | Error Layering | Propagate errors ergonomically with `?` and combinators | Deep nested matches; panicking for recoverable errors |
| A.90 | Fallible Iterator Pipelines | Error-aware transforms with `try_fold`/`try_for_each` | Manual unwraps inside maps |
| A.112 | CLI Exit Codes | CLIs that must signal failure with proper exit codes | Panicking for expected user errors |

## Interior Mutability and Synchronization

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.8 | Interior Mutability | `Cell/RefCell` single-thread; `Mutex/RwLock` cross-thread | Holding `RefCell` borrows or locks across `.await` |
| A.117 | parking_lot | Smaller, faster locks on contended paths | Mixing `std::sync` and `parking_lot` guards |
| A.122 | Lock-free (crossbeam-epoch) | Lock-free data structures with epoch-based reclamation | Premature lock-free when simple locks suffice |
| A.131 | MPMC (crossbeam-channel) | Multi-producer multi-consumer with `select!` | Unbounded channels in high-throughput systems |
| A.132 | Rendezvous Backpressure | Producers must slow when consumers lag | Unbounded memory growth |
| A.133 | One-time Init (OnceLock/LazyLock) | Global initialization that runs exactly once | Ad-hoc boolean flags for initialization |

## Trait Design

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.9 | Trait Objects vs Generics | Heterogeneous collections or dynamic dispatch at boundaries | Trait objects in tight loops |
| A.10 | Pin/Unpin | Implementing Future/state machines that must not move | `Pin::new_unchecked` without proof |
| A.46 | Trait Object Upcasting | Upcast `dyn SubTrait` to `dyn SuperTrait` (Rust 1.86+) | Constructing invalid vtables in unsafe |
| A.47 | Coherence and Orphan Rules | Designing trait extension points | Blanket impls that conflict downstream |
| A.48 | Specialization | Constrained cases in std/internal crates only | Relying on unsound specialization in public crates |
| A.49 | Fn/FnMut/FnOnce Selection | Picking callable trait bounds matching call semantics | Over-constraining to `Fn` when mutation is required |
| A.61 | HRTBs (for<'a>) | Generic callbacks borrowing data with any lifetime | Requiring 'static unnecessarily |
| A.62 | GATs | Associated types depending on lifetimes (lending iterators) | Forcing owned allocations because outputs can't borrow |
| A.63 | Dyn Compatibility | Designing traits for dynamic dispatch | Non-dyn-compatible APIs in public traits |
| A.64 | Never Type (!) | Functions that never return; exhaustive matches | Encoding never-return via panics without documenting |
| A.79 | Dispatchable Methods | Object safety with rich APIs | Generics on methods of object-safe traits |
| A.80 | Async in Traits | Declaring async behavior in traits | Assuming async trait methods are dyn-compatible |

## Async Patterns

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.11 | Async Hygiene | CPU-bound work in async (`spawn_blocking`); short critical sections | Blocking I/O directly on the runtime |
| A.12 | Bounded Channels | Limit memory growth and propagate shutdown | Unbounded channels; ad-hoc boolean flags for cancellation |
| A.25 | Cancel-safe select! | Multiplexing operations; futures must not own partial state | Futures that drop with half-consumed state |
| A.52 | Cancel-Safe Patterns | Async multiplexing with state encapsulation | Storing partial buffers inside futures that may be dropped |
| A.70 | Async Polling/Waker | Building executors or custom futures | Busy-loop polling; waking without state change |
| A.75 | Runtime Choice | tokio (general), smol (lightweight), glommio (io_uring) | Mixing runtimes in one process |
| A.76 | Structured Concurrency | Spawned tasks must complete before scope exit | Fire-and-forget tasks that outlive owners |
| A.87 | JoinSet | Run and manage many tasks that must end with parent | Detached tasks leaking beyond scope |
| A.118 | io_uring Runtimes | Linux high-throughput I/O-bound servers | Assuming portability to non-Linux |

## Iterators and Collections

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.33 | IntoIterator Semantics | Designing APIs that accept collections generically | Surprising ownership moves via `into_iter()` |
| A.34 | Iterator Laziness | Building pipelines with adapters and consumers | Building intermediate `Vec`s unnecessarily |
| A.35 | Fn Bounds for Closures | Accepting closures with narrowest bound | Overconstraining to `Fn` when `FnOnce` is needed |
| A.36 | DoubleEndedIterator | Bi-directional traversal and predictable exhaustion | Assuming fused behavior without `.fuse()` |
| A.37 | FromIterator/Collect | Constructing collections from iterators | Collecting when a lazy pipeline suffices |
| A.38 | Vec vs VecDeque vs LinkedList | Choosing sequence containers | Using `LinkedList` for general workloads |
| A.39 | HashMap vs BTreeMap | Key-value storage: O(1) avg vs ordering | Relying on hash iteration order |
| A.40 | IntoIterator for Owned/Borrowed | Custom collections working with `for` loops | Only implementing owned iteration |
| A.42 | Fused Iteration | Consumers calling `next()` after exhaustion | Assuming fusion without `.fuse()` |
| A.43 | Closure Capture and move | Spawning threads/tasks with `move` closures | Capturing non-`Send` across thread boundaries |
| A.44 | Arrays/Slices IntoIterator | Generic iteration with `&[T]` bounds | Relying on edition-specific behavior |
| A.50 | size_hint/ExactSizeIterator | Consumers benefiting from pre-allocation | Claiming `ExactSizeIterator` without invariants |
| A.53 | Collection Capacity Planning | Heavy pushes with known sizes; `with_capacity` | Per-push reallocations in tight loops |
| A.54 | BinaryHeap | Top-k, scheduling, priority work queues | Using `BTreeMap` to emulate a heap |
| A.55 | SmallVec | Small fixed upper bound per hot path | Assuming SSO always wins; ignoring spill costs |
| A.56 | HashMap Implementation | Tuning map performance; SwissTable/hashbrown | Custom hashers in hostile inputs |
| A.57 | IndexMap | Deterministic iteration order required | Assuming order preservation after removals |
| A.58 | Pattern Matching (2024) | Refine matches with guards; Rust 2024 ergonomics | Relying on implicit ref/mut |

## Testing

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.16 | Doctests | Examples express API semantics and invariants | Examples that don't compile or assert nothing |
| A.17 | Compile-fail Tests (trybuild) | Macros and type errors are part of public contract | Relying on unstable error messages |
| A.18 | Property-based (proptest) | Invariants across large input spaces | Only example-based tests for complex parsers |
| A.19 | Fuzzing (cargo-fuzz) | Parsing/untrusted input, protocol boundaries | Running fuzzers without ASan/UBSan |
| A.20 | Coverage (cargo-llvm-cov) | Prevent coverage regressions on critical modules | Chasing 100% blindly; ignoring branch coverage |
| A.28 | Concurrency (loom) | Validating Send/Sync invariants and race-freedom | Testing production code paths wholesale |
| A.71 | Snapshot (expect-test) | Robust golden tests for parsers, formatters | Brittle string asserts; manual golden files |
| A.154 | Fuzzing + Property Together | Complementary coverage for complex inputs | Relying on only one approach |

## Macros

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.77 | Macro Hygiene ($crate) | `macro_rules!` referring to crate items robustly | Relying on local imports inside expansions |
| A.78 | Build Script Codegen | Generate bindings/code at build time | Writing outside `OUT_DIR` |
| A.101 | Macro Flavors | Choosing derive/attribute/function-like | Using proc-macros when `macro_rules!` suffices |
| A.102 | TokenStreams and Spans | Building procedural macros with meaningful errors | Manual string concatenation to build code |
| A.103 | syn + quote Pipeline | Parsing and generating Rust in proc-macros | Relying on unstable compiler internals |
| A.104 | Macro Error Reporting | Signaling misuse with `compile_error!` and spans | Cryptic errors without spans |
| A.105 | macro_rules! Repetition | Declarative macros with fragment specs | Over-greedy matchers requiring deep backtracking |
| A.108 | Macro Security | Designing macro-heavy APIs with constrained surface | Hidden heavy codegen surprising users |
| A.150 | Proc-Macro Discipline | Hygiene, spans, diagnostics in proc-macros | Sprawling proc-macro logic |

## Performance

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.116 | Avoid Unnecessary Cloning | Eliminating needless allocations | Cloning large structures gratuitously |
| A.123 | Monomorphization Bloat | Strategic trait objects to reduce code size | Excessive generics causing binary bloat |
| A.126 | Caching (moka/cached) | Concurrent caching and memoization | Unbounded caches without eviction |
| A.151 | Global Allocator | jemalloc/mimalloc for allocation-heavy workloads | Switching allocator without benchmarking |
| A.152 | Build-time Optimizations | LTO, PGO, and inlining for release builds | Enabling LTO without measuring compile time impact |
| A.153 | Coverage in CI | cargo-llvm-cov for local and CI coverage | Coverage without branch analysis |

## Unsafe and FFI

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.32 | FFI Unwind Safety | Calling into/from C where exceptions may cross boundaries | Unwinding across FFI without contracts |
| A.59 | FFI Layout (repr) | Interop and ABI stability with `repr(C)`/`repr(transparent)` | Assuming Rust default layout across FFI |
| A.67 | Nullable Pointer Optimization | `Option<NonNull<T>>` for nullable pointers | Magic sentinel integers |
| A.82 | Unsafe Aliasing Models | Writing unsafe/FFI code with raw pointers | Aliasing writes through raw pointers while shared refs exist |
| A.83 | MaybeUninit | Manual initialization for performance or FFI | `assume_init` before full initialization |
| A.84 | Pin Projection (pin-project-lite) | Types with pinned self and pinned fields | Manual `Pin::new_unchecked` field projection |
| A.85 | Drop Check | Implementing Drop for generic types | Hidden self-referential borrows |
| A.86 | SAFETY Documentation + Miri | Authoring unsafe with documented invariants | Sprawling unsafe with no justification |
| A.91 | Unsafe Encapsulation | Safe wrappers over unsafe code/FFI | Exposing raw pointers in public APIs |
| A.99 | Niche Optimization | `NonZero*` and `Option<NonNull<T>>` for size optimization | Sentinel integers when type system provides niches |
| A.111 | Unsafe Aliasing Invariants | Writing unsafe code around pointers and references | Writing through raw pointers while shared refs are live |
| A.143 | FFI Nullability Contracts | Nullable pointers and unwinding contracts at FFI | Assuming non-null without validation |

## CI and Quality

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.13 | Clippy as Gate | Enforce correctness and idioms in CI | Allowing warnings in CI |
| A.14 | rustfmt Non-negotiable | Enforce consistent formatting | Hand-tuned formatting |
| A.15 | MSRV Policy | Communicating minimum supported Rust version | Breaking MSRV silently |
| A.21 | Supply-chain (cargo-audit/deny) | Enforce vuln and license policies | Pinning yanked/vulnerable crates |
| A.22 | Structured Observability (tracing) | Correlate logs, metrics, spans in async services | println!-style logs in libraries |
| A.23 | Secret Handling (secrecy) | Hold API keys/passwords safely | Formatting secrets into logs |
| A.24 | SQLx Compile-time Queries | Ensure query validity at build time | Building SQL strings dynamically |
| A.114 | CI Quality Gates | fmt, clippy, doctests, coverage in CI | Untested examples; unaudited dependencies |
| A.148 | Pin Toolchains (rustup) | Reproducible builds per project | Relying on system-wide toolchain |
| A.149 | Cargo Workspaces | Config layering with `.cargo/config.toml` | Inconsistent settings across workspace members |

## Concurrency Deep Patterns

| ID | Name | When To Use | Avoid |
|----|------|-------------|-------|
| A.95 | Borrow Conflict Hygiene | Mixing shared and mutable borrows | Long-lived shared borrows overlapping &mut |
| A.109 | Borrow-Checker-Friendly APIs | Methods that mix reads and writes | Long-lived shared borrows overlapping mutable operations |
| A.110 | Two-Phase Borrow Patterns | Methods taking `&mut self` with nested calls | Exposing APIs requiring specific compiler lowering |
| A.134 | Fine-grained Visibility | Controlling module visibility and prelude | Over-exposing internal types |
| A.136 | Thread Naming | Named threads for debugging and join discipline | Anonymous threads that are hard to debug |
| A.137 | fn Pointers | No capture needed; use function pointers | Closures when a plain function suffices |
| A.180 | Work-stealing (crossbeam-deque) | Custom thread pools with work-stealing | Reinventing scheduling when tokio/rayon suffice |
| A.181 | Epoch-based Reclamation | Lock-free data structures with safe memory reclamation | Manual memory management in concurrent code |
| A.182 | Unsafe Send/Sync | Manual impl when compiler cannot prove safety | Implementing without documenting invariants |
| A.183 | Deterministic Testing (loom) | Validating all interleavings of concurrent code | Testing production code paths wholesale in loom |
| A.184 | Channel Choices | Selecting between std::mpsc, crossbeam, flume | Using wrong channel type for workload |

---

*Catalog version: 1.0 -- 125 idioms indexed from rust-coder-01.md (source contains 184 total)*
