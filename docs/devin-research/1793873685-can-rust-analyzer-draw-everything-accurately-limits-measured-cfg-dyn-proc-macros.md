# Can rust-analyzer draw everything accurately? No — here is exactly what it draws, what it abstracts, and what it cannot see

*Written 2026-11-05. Branch `research/tiny-rust-code-mapper`. Follows `1793864191-can-an-llm-parse-rust-better-than-the-compiler-…md`, which argued that rust-analyzer (not `rustc`) should be the extraction substrate for levels L1–L4 of the Rust graph. This note is the honest list of its limits, each one checked on this machine (rust-analyzer 1.99.1, cargo 1.99.1 stable, Linux, 8 cores / 31 GB).*

**One-line answer:** rust-analyzer draws the structural graph of *the one build configuration you load* almost exactly; it cannot draw all configurations, it deliberately abstracts dynamic/generic dispatch, it needs a crate graph and (for proc macros) a real build step, and it knows nothing about meaning. Those are the places where the rest of the toolchain, or the model, has to step in.

---

## 1. What it gets right [measured]

| Check | Result |
|---|---|
| cargo-semver-checks (11 crates, 37 822 expressions), proc macros + build scripts **on** | **9 unknown types (0.02%)**, 21.9 s CPU, 2.3 GB. The build step (`cargo check` for proc-macro crates / `OUT_DIR`) was already warm: 0.36 s |
| same, proc macros + build scripts **off** | 83 unknown types (0.2%), 29.0 s — so proc macros are the main source of gaps when the build step is skipped |
| cargo-geiger (depends on the whole `cargo` crate, 15 419 expressions), proc macros on | **0 unknown types**, 26.7 s, of which **14.8 s was the `cargo check` build step** for proc macros / build scripts |
| rust-code-analysis (15 crates, 51 893 expressions), proc macros off | 55 unknown (0.1%), 35.0 s |
| broken file injected into cargo-public-api (bogus `use`, misplaced inner attribute, garbage statement) | analysis completes for all 14 crates; syntax errors reported with positions; other crates' type errors still reported; broken file still indexed |
| `macro_rules!`-generated function (fixture below) | the call `generated()` resolves, to the invocation site `make_fn!(generated);` ✓ |
| concrete method call `Circle(1.0).area()` | resolves to the body in `impl Shape for Circle` ✓ |

## 2. The fixture

A 20-line crate built to probe the known corners (`/tmp/rafix`, `rust-analyzer lsif .`, then follow `textDocument/definition` edges):

```rust
pub trait Shape { fn area(&self) -> f64; }
pub struct Circle(pub f64);
pub struct Square(pub f64);
impl Shape for Circle { fn area(&self) -> f64 { 3.14 * self.0 * self.0 } }
impl Shape for Square { fn area(&self) -> f64 { self.0 * self.0 } }

pub fn total(shapes: &[Box<dyn Shape>]) -> f64 { shapes.iter().map(|s| s.area()).sum() }   // line 7
pub fn total_generic<S: Shape>(shapes: &[S]) -> f64 { shapes.iter().map(|s| s.area()).sum() } // line 8
pub fn concrete() -> f64 { Circle(1.0).area() }                                                // line 9

#[cfg(windows)] pub fn os_only() -> u8 { 1 }                                                   // line 11
#[cfg(unix)]    pub fn os_only() -> u8 { 2 }                                                   // line 12
pub fn uses_os() -> u8 { os_only() }                                                           // line 13

#[cfg(feature = "extra")] pub fn only_with_feature() -> u8 { 9 }
pub fn uses_feature() -> u8 { #[cfg(feature = "extra")] { only_with_feature() } #[cfg(not(feature = "extra"))] { 0 } }

macro_rules! make_fn { ($n:ident) => { pub fn $n() -> u8 { 7 } }; }
make_fn!(generated);                                                                           // line 19
pub fn uses_generated() -> u8 { generated() }                                                  // line 20
```

What the LSIF said:

```text
line  7 `area`      -> definition at line  1: pub trait Shape { fn area(&self) -> f64; }        (dyn:     trait decl, not Circle/Square)
line  8 `area`      -> definition at line  1: pub trait Shape { fn area(&self) -> f64; }        (generic: trait decl)
line  9 `area`      -> definition at line  4: impl Shape for Circle { fn area(&self) ... }      (concrete: the impl)   ✓
line 13 `os_only`   -> definition at line 12: #[cfg(unix)] pub fn os_only() -> u8 { 2 }         (windows variant does not exist)
        `only_with_feature` call          -> not indexed at all                                  (inactive cfg block is invisible)
line 20 `generated` -> definition at line 19: make_fn!(generated);                               ✓
```

## 3. Where it is accurate but *abstract* (by design)

**Dynamic and generic dispatch resolve to the trait method, not to implementers.** `s.area()` on `dyn Shape` or on `S: Shape` → `Shape::area` (the declaration). This is the *correct* static answer; the set of possible targets (`Circle::area`, `Square::area`, plus any impl in any downstream crate) is a different question (`goto_implementation` for the local set; whole-program monomorphisation — Präzi/LLVM — for the real set). For the graph: `CALLS` to a trait method must be typed `MAY_CALL(dispatch=dyn|generic)` with the locally known implementer set attached as a hint, never expanded into N concrete edges as if they were certain.

## 4. Where it is blind

1. **One `cfg` at a time.** The windows `os_only` does not exist; the `feature = "extra"` call is not indexed. rust-analyzer analyses one target × feature set per load (it even warns `cyclic deps: cfg(...)` on the rust-analyzer repo). Nothing in the Rust toolchain yields the whole cfg matrix in one pass — `rustc`, rustdoc and MIR tools have the same property. For the graph this is a design decision: load N configurations and union/diff the item trees (cheap: item trees are per file and cacheable), or record `cfg` predicates syntactically at L2 (tree-sitter sees the attribute even if rust-analyzer drops the item) and let L3 confirm one branch at a time. Recommendation: both — L2 keeps every cfg'd item with its predicate; L3/L4 edges carry the configuration they were verified under.
2. **Proc macros and `build.rs` output need a real build step.** rust-analyzer shells out to `cargo check` to compile proc-macro crates and populate `OUT_DIR` (14.8 s of geiger's 26.7 s). Without it, derives, `#[tokio::main]`, `#[derive(Serialize)]`, `include!(concat!(env!("OUT_DIR"), …))` are holes — degraded gracefully (items unresolved, not crashes), but holes. This is the one place where "no compiler" is not fully true: the *proc-macro crates* (not your crate) must build once; after that the server is reused.
3. **Needs a crate graph.** `cargo metadata` (dependencies must resolve: network or vendored registry) or a `rust-project.json` for Buck/Bazel/other build systems. A loose directory of `.rs` files, a diff, a snippet, a crate whose `Cargo.toml` is broken → syntax only. This is the regime for the tree-sitter and model fallback (L2 exact, L3–L4 as tagged guesses).
4. **Type-system corners.** Its trait solver is not rustc's; specialization, some HRTB/async/closure cases, const-generic evaluation and unstable features produce `{unknown}`. Measured at 0.02–0.2% of expressions; non-zero, and it can disagree with rustc about whether code *compiles*. Fine for a graph; not a semver or soundness verdict — that is cargo-semver-checks / rustdoc JSON / `cargo check`.
5. **No dataflow or information flow.** It lowers MIR (for the const evaluator and a few hints), but "which statements influence this value", lock order, aliasing and `Send`/`Sync` boundary reasoning are Flowistry / `rustc_mir_dataflow` territory: nightly and a borrowck-clean build. Level L5 is on-demand only.
6. **No versioned public-API artefact.** It has all the facts, but the stable, format-versioned contract of a crate is rustdoc JSON's job (and cargo-public-api's rendering of it). For spec-vs-code reflexion checks, generate rustdoc JSON when the crate builds and reconcile ids with the rust-analyzer graph.
7. **Scale and robustness.** 2–3 GB and 20–35 s for a 42k-LOC workspace, ~80% of it in std and dependency crates (identical across projects, cacheable). It panicked on its own 593k-LOC repository inside the stats printer (`str` char-boundary on a `│` in a test fixture) — a tooling bug, but a reminder that at 10 M LOC you load per crate against a shared std/registry index and never the whole workspace in one process.
8. **Structure, not meaning.** It cannot say what a module is for, which of 40 public items are the real entry points, which path is hot, or which invariant an `unsafe` block depends on. That is the model's layer (L6) and the only place where a model is not competing with an exact tool.

## 5. Consequences for the graph design

```text
level  rust-analyzer gives it?      gap                          who fills the gap
L1     via cargo metadata           non-cargo projects           Cargo.toml parse / rust-project.json / tree-sitter
L2     yes (ItemTree)               inactive-cfg items dropped   tree-sitter keeps every item + cfg predicate
L3     yes, one cfg at a time       other cfgs; proc-macro items  load N cfgs and union; cargo check once for proc macros
L4     yes; dyn/generic → trait     implementer fan-out          MAY_CALL + implementer hint; MIR/LLVM when exact needed
L5     no                           information flow             Flowistry (nightly, built crates), on demand
L6     no                           meaning                      the model
```

Every record therefore carries `cfg` (which configuration it was verified under), `dispatch` (static | generic | dyn) on call edges, `substrate` (ra-hir | tree-sitter | rustdoc-json | mir | llm) and `confidence`. "Accurate" is then a per-record statement rather than a promise about the whole picture — which is the only honest way to say "98%".

## 6. Provenance

Runs: `rust-analyzer analysis-stats --skip-mir-stats --skip-const-eval --skip-data-layout .` with and without `--disable-build-scripts --disable-proc-macros`, in the clones under `~/repos/rust-tools/`; `rust-analyzer lsif .` on the fixture crate above and on the deliberately broken copy of cargo-public-api; definitions followed through LSIF `next` → `textDocument/definition` → `item` edges with a 30-line Python script. Single runs, ±20% on times. The "unknown types" counts are rust-analyzer's own `??ty`/`?ty` statistics.
