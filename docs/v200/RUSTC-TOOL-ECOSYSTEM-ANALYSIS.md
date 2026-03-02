# Rust Compiler Tool Ecosystem: Comprehensive Use Case Analysis

**Analysis Date:** 2026-03-01
**Target User Segment:** Rust open source contributors (accuracy-focused, not speed-focused)
**Purpose:** Map all tools using Rust compiler internals for Parseltongue's design

---

## Executive Summary

**45+ tools** were analyzed from the Rust compiler internals ecosystem. The key findings:

| Category | Tools | Primary Use Case | API Pattern |
|----------|-------|------------------|-------------|
| **Formal Verification** | 9 | Prove correctness properties | MIR → SMT/theorem prover |
| **Static Analysis** | 12 | Find bugs without execution | MIR/HIR analysis |
| **Linting** | 4 | Style and correctness checks | AST/HIR traversal |
| **Codegen Backends** | 7 | Alternative compilation targets | MIR → native code |
| **Information Flow** | 3 | Security and privacy | MIR dataflow |
| **Education/Debugging** | 3 | Understand Rust behavior | MIR interpretation |
| **Infrastructure** | 5 | Support other tools | Frameworks and libraries |

**Critical insight for Parseltongue:** 80% of tools operate at the **MIR level** via `TyCtxt` after `after_analysis`. This is the sweet spot for semantic analysis.

---

## Part 1: The Five Integration Patterns

### Pattern 1: Direct rustc_private (30+ tools)

The most common pattern - directly using `#![feature(rustc_private)]` and depending on `rustc_*` crates.

**Entry point:** `rustc_driver::Callbacks`
**Hook point:** `after_analysis` callback provides `TyCtxt<'tcx>`

```rust
#![feature(rustc_private)]
extern crate rustc_driver;

struct MyCallbacks;
impl rustc_driver::Callbacks for MyCallbacks {
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &Compiler,
        tcx: TyCtxt<'tcx>
    ) -> Compilation {
        // Full access to MIR, HIR, types, traits
        Compilation::Stop
    }
}
```

**Pros:** Maximum power, access to everything
**Cons:** Nightly-only, breaks frequently, complex setup

**Users:** Kani, Miri, Creusot, Flux, Prusti, Rudra, RAPx, lockbud, rustc_codegen_*

---

### Pattern 2: rustc_plugin Framework (4 tools)

Brown University's framework generalizing Clippy's pattern.

**Components:**
- `rustc_plugin`: Cargo integration, CLI scaffolding
- `rustc_utils`: Extension traits for MIR, HIR, spans

**Users:**
| Tool | Version | Nightly |
|------|---------|---------|
| Flowistry | 0.14.2 | 2025-08-20 |
| Aquascope | 0.12.0 | 2024-12-15 |
| Paralegal | 0.12.0 | 2024-12-15 |
| Argus | 0.14.1 | 2025-08-20 |

**Key benefit:** Framework handles boilerplate, you focus on analysis

---

### Pattern 3: Charon/LLBC Decoupling (5+ tools)

Charon extracts MIR to a language-agnostic JSON format (LLBC), decoupling tools from compiler entirely.

**Pipeline:**
```
Rust source → rustc → MIR → Charon → LLBC (JSON) → Downstream tools
```

**Users:**
| Tool | Purpose | Backend |
|------|---------|---------|
| Aeneas | Functional verification | F*, Coq, HOL4, Lean |
| Eurydice | C code generation | C compiler |
| Kani (partial) | Model checking | CBMC via LLBC |
| Hax | Cryptographic verification | F*, Rocq |

**Key benefit:** Tools don't break when rustc changes
**Trade-off:** Loss of incremental analysis, must re-extract

---

### Pattern 4: Stable MIR / rustc_public (1 tool migrating)

Official rust-lang project to provide SemVer-stable MIR API.

**Architecture:** Two crates like `proc-macro2`
- `rustc_public`: Public stable API (to be on crates.io)
- `rustc_smir`: Internal translation layer

**Status:** v0.1 not yet published, Kani is pioneering adopter

**Key benefit:** Eventually works on stable Rust
**Current state:** Still nightly-only, incomplete

---

### Pattern 5: No Compiler Dependency (5+ tools)

Some tools avoid compiler internals entirely.

**Users:**
| Tool | Approach |
|------|----------|
| cargo-scan | Uses `cargo` crate for metadata |
| Polonius | Standalone Datalog engine |
| cargo-semver-checks | rustdoc JSON + Trustfall |
| cargo-modules | ra_ap_* crates |
| Dylint | Dynamic library loading |

---

## Part 2: Verification and Formal Methods (9 tools)

### Tier 1: Production-Ready

#### Kani (AWS)
- **Backend:** CBMC (C Bounded Model Checker) → SAT/SMT (bitwuzla, cvc5, z3)
- **Nightly:** 2025-12-03
- **Use cases:**
  - Prove absence of panics
  - Verify unsafe code safety
  - Function contracts and invariants
  - Bit-precise verification
- **Unique:** Pioneering Stable MIR migration, has Charon/LLBC backend too
- **Code patterns:** `kani::any()` for non-deterministic inputs, `#[kani::proof]` for harnesses

#### Miri (rust-lang)
- **Backend:** Direct MIR interpretation
- **Nightly:** Tracks latest continuously
- **Use cases:**
  - Detect undefined behavior
  - Test aliasing models (Stacked Borrows, Tree Borrows)
  - Find data races in concurrent code
  - Validate unsafe code
- **Unique:** Core lives in `rustc_const_eval`, Miri repo adds runtime features
- **Code patterns:** Interprets each MIR statement, tracks provenance

### Tier 2: Research-Active

#### Creusot (INRIA)
- **Backend:** Why3 → Alt-Ergo/Z3/CVC5
- **Nightly:** 2026-02-27
- **Use cases:**
  - Deductive verification
  - Pearlite specification language
  - Reasoning about mutable borrows with `^` (final value)
- **Unique:** Quantifier support, prophecy variables for borrows

#### Flux (UC San Diego)
- **Backend:** Liquid types → Horn constraint solving
- **Nightly:** 2025-11-25
- **Use cases:**
  - Refinement type checking
  - Value-range analysis
  - Tock OS verification
- **Unique:** Exploits Rust ownership for strong updates

#### Prusti (ETH Zurich)
- **Backend:** Viper → Silicon/Carbon → Z3
- **Nightly:** 2023-09-15 (frozen for stable)
- **Use cases:**
  - Specification-based verification
  - Pre/post conditions via proc macros
  - Client-server caching architecture
- **Unique:** Java dependency for Viper backend

### Tier 3: Decoupled via Charon

#### Aeneas + Charon (INRIA/MSR)
- **Backend:** F*/Coq/HOL4/Lean
- **Languages:** OCaml (Aeneas) + Rust (Charon)
- **Use cases:**
  - Functional translation of Rust
  - Proof assistant integration
  - HACL-Rust verified crypto
- **Unique:** Ownership becomes functional updates

#### Hax (Cryspen)
- **Backend:** F*/Rocq
- **Nightly:** 2025-11-08
- **Use cases:**
  - High-assurance crypto
  - Specification extraction
- **Unique:** Shares `hax-frontend-exporter` with Charon

---

## Part 3: Static Analysis and Bug Finding (12+ tools)

### Concurrency Bugs

#### lockbud
- **Nightly:** 2025-10-02
- **Use cases:**
  - Double-lock detection
  - Conflicting lock order
  - Atomicity violations
- **Method:** Track MutexGuard/RwLockGuard lifetimes through MIR
- **Found bugs in:** openethereum, grin, lighthouse

### Memory Safety

#### RAPx (Fudan University)
- **Nightly:** 2025-12-06
- **Use cases:**
  - Use-after-free detection
  - Memory leak detection
  - Extensible analysis platform
- **Architecture:** Core algorithms + application detectors
- **Unique:** 881+ commits, actively maintained

#### Rudra (Georgia Tech)
- **Nightly:** 2021-10-21 (FROZEN)
- **Use cases:**
  - Panic safety in unsafe code
  - Send/Sync invariant violations
  - Higher-order invariant bugs
- **Results:** 264 memory bugs → 76 CVEs → 98 RustSec advisories
- **Status:** Stale, Charon reimplementation being explored

### Type Confusion

#### TypePulse
- **Use cases:**
  - Misalignment detection
  - Inconsistent layout bugs
  - Mismatched scope bugs
- **Results:** 71 new type confusion bugs in top 3000 crates

### Side Effect Auditing

#### cargo-scan (UCSD)
- **Rust:** Stable 1.85.0 (no rustc_private!)
- **Use cases:**
  - OS call auditing
  - Filesystem access tracking
  - Unsafe block detection
- **Unique:** Uses `cargo` crate, not compiler internals

---

## Part 4: Information Flow and Security (3 tools)

#### Flowistry (Brown)
- **Nightly:** 2025-08-20
- **Framework:** rustc_plugin 0.14.2
- **Use cases:**
  - Modular information flow
  - Dependency analysis
  - Taint tracking
- **Method:** Polonius facts → dependency sets
- **Properties:** Flow-sensitive, field-sensitive, modular
- **Published:** PLDI 2022

#### Paralegal (Brown)
- **Nightly:** 2024-12-15
- **Framework:** rustc_plugin 0.12.0
- **Use cases:**
  - Privacy policy enforcement
  - Security policy checking
- **Method:** PDG (Program Dependence Graph) analysis
- **Annotations:** `#[paralegal::marker(...)]`

---

## Part 5: Education and Debugging (3 tools)

#### Aquascope (Brown CEL)
- **Nightly:** 2024-12-15
- **Use cases:**
  - Ownership visualization
  - Borrow checking explanation
- **Method:** Polonius → permissions + Miri for execution
- **Published:** OOPSLA 2023

#### Argus (Brown CEL)
- **Nightly:** 2025-08-20
- **Use cases:**
  - Trait error debugging
  - Inference tree exploration
- **Method:** Intercept trait solver, capture search tree
- **Results:** 2.2× better fault localization
- **Published:** PLDI 2025

---

## Part 6: Linting Frameworks (4 tools)

#### Clippy (rust-lang)
- **700+ lints** across AST, HIR, and MIR
- **Pattern:** Custom driver + lint passes

#### Dylint (Trail of Bits)
- **Version:** 5.x
- **Pattern:** Dynamic library loading (`cdylib`)
- **Unique:** Multi-version driver management

#### Marker (rust-marker)
- **Nightly:** 2023-12-28
- **Pattern:** Driver-independent stable API
- **Status:** Early stage

---

## Part 7: Codegen Backends (7 tools)

| Backend | Target | Status |
|---------|--------|--------|
| **cranelift** | Native code | Very active |
| **gcc** | GCC platforms | Active |
| **SPIR-V** | GPU shaders | Active |
| **CLR** | .NET CIL | Very active |
| **JVM** | JVM bytecode | Active |
| **C** | C code | Experimental |
| **NVVM** | CUDA | Lower activity |

---

## Part 8: Infrastructure Projects (5 tools)

#### Polonius (rust-lang)
- **Purpose:** Next-generation borrow checker
- **Method:** Datalog on MIR borrowcheck facts
- **Engine:** `datafrog` crate

#### KMIR (Runtime Verification)
- **Purpose:** Operational semantics for MIR in K framework
- **Uses:** Stable MIR JSON serialization

#### compiletest_rs
- **Purpose:** Extract rustc's test harness
- **Users:** Dylint, Prusti, many others

---

## Part 9: Use Case Taxonomy for Parseltongue

### Use Case 1: "Where is X defined?"
**Tools that solve this:** All of them - fundamental operation
**Data needed:** HIR → DefId → Span
**Accuracy requirement:** 100% - must be exact

### Use Case 2: "What calls X?"
**Tools:** Flowistry, Paralegal, lockbud, RAPx
**Data needed:** MIR call graph
**Accuracy requirement:** 100% - cannot miss callers

### Use Case 3: "What does X depend on?"
**Tools:** Flowistry, Paralegal, cargo-scan
**Data needed:** MIR dataflow analysis
**Accuracy requirement:** High - missing dependencies = incomplete context

### Use Case 4: "What can go wrong here?"
**Tools:** Kani, Miri, lockbud, Rudra, RAPx
**Data needed:** Full semantic analysis + domain knowledge
**Accuracy requirement:** Very high for safety-critical

### Use Case 5: "What types implement trait T?"
**Tools:** Argus, all tools using TyCtxt
**Data needed:** Trait solver output
**Accuracy requirement:** 100% - cannot miss implementations

### Use Case 6: "What's the ownership/borrow situation?"
**Tools:** Aquascope, Miri, Flowistry
**Data needed:** Polonius borrowcheck facts
**Accuracy requirement:** Very high - wrong borrow info = confusion

### Use Case 7: "What would break if I change X?"
**Tools:** cargo-semver-checks, Paralegal
**Data needed:** Cross-crate references, trait impls
**Accuracy requirement:** High - false positives are annoying

### Use Case 8: "Is this code correct?"
**Tools:** Kani, Prusti, Creusot, Flux
**Data needed:** Full semantic model + specifications
**Accuracy requirement:** Maximum - proving correctness

---

## Part 10: API Surface Analysis

### What These Tools Actually Use

| API Surface | Users | Purpose |
|-------------|-------|---------|
| **TyCtxt** | All rustc_private tools | Global context, queries |
| **MIR bodies** | 80% of tools | Control flow, data flow |
| **HIR** | 40% of tools | Syntactic analysis |
| **Trait solver** | Argus, verification tools | Type resolution |
| **Polonius facts** | Flowistry, Aquascope | Borrow checking |
| **DefId** | All tools | Unique identifier |
| **Span** | All tools | Source location |

### Key Types for Parseltongue

```rust
// From rustc_middle
TyCtxt<'tcx>      // Global context - query everything
DefId             // Unique definition identifier
LocalDefId        // Definition in local crate
Span              // Source location (file, line, column)
Ty<'tcx>          // Type representation
Instance<'tcx>    // Monomorphized function

// From rustc_hir
HirId             // HIR node identifier
ItemKind          // What kind of item
FnSig             // Function signature

// From rustc_middle::mir
Body<'tcx>        // MIR body (control flow graph)
BasicBlock        // CFG node
Statement         // MIR statement
Terminator        // CFG edge (calls, branches)
Place             // Memory location
Operand           // Value (copy or move)
Rvalue            // Right-hand side expression
```

---

## Part 11: Nightly Version Analysis

### Version Distribution

| Era | Nightly Range | Tools | Notes |
|-----|---------------|-------|-------|
| **Ancient** | Pre-2022 | Rudra | Frozen, unusable |
| **Old** | 2022-2023 | Prusti, Marker | Still functional |
| **Recent** | 2024 | Aquascope, Paralegal | 1-2 years old |
| **Current** | 2025+ | Kani, Creusot, Flux | Actively maintained |
| **Bleeding** | Latest | Miri, codegen_clr | Tracks nightly |

### Update Frequency

| Tool Type | Update Cadence | Why |
|-----------|---------------|-----|
| **Miri** | Weekly | Must track compiler |
| **Kani** | Monthly | AWS team, dedicated |
| **Framework tools** | Quarterly | Framework absorbs breakage |
| **Research tools** | Yearly or frozen | Academic timelines |
