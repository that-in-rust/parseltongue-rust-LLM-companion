# v173-z3-tradeoff-analysis: Why Parseltongue Skips Z3

**Generated**: 2026-02-15
**Context**: Taint analysis architecture decision for Parseltongue v1.7.3
**Decision**: Do NOT use Z3. Use CozoDB Datalog reachability instead.
**Status**: DECIDED

---

## What Is Z3?

Z3 is a Satisfiability Modulo Theories (SMT) solver from Microsoft Research, written in C++. It takes logical constraints and determines whether they can be satisfied simultaneously. code-scalpel (our primary competitor for taint analysis) uses Z3 to symbolically reason about whether tainted data can reach a security sink through feasible code paths.

code-scalpel's implementation: `src/code_scalpel/security/analyzers/taint_tracker.py` — 2,466 lines of Python using Z3 to encode Python semantics as symbolic constraints.

---

## Problem 1: C++ Binary Dependency Breaks Our Zero-Deps Property

Parseltongue today:

```
cargo build
pure Rust
compiles anywhere
no system deps
~30s build
single binary output (~15MB)
```

With Z3:

```
cargo build
... needs libz3.so / libz3.dylib / z3.dll
... needs Z3 C++ headers
... needs cmake
... needs python (for Z3 binding generation)
... platform-specific linking (Linux/macOS/Windows/ARM)
... build breaks on CI without z3-sys setup
... binary goes from ~15MB to ~80MB+
```

Z3 is a 500K+ LoC C++ project. The Rust bindings (`z3` crate on crates.io) require either:
- System-installed Z3 with headers (user burden), or
- Vendored Z3 source compiled from C++ at build time (10+ minute builds)

Parseltongue currently has zero non-Rust compile-time dependencies. tree-sitter grammars are Rust-generated C compiled via `cc` crate (lightweight, hermetic). Z3 is a different magnitude of dependency — it's an entire theorem prover runtime.

**Architectural principle violated**: Parseltongue is designed to be embeddable like SQLite. Adding Z3 makes it embeddable like PostgreSQL — technically possible, practically painful.

---

## Problem 2: Doesn't Scale to 12 Languages

Z3 does symbolic reasoning by encoding language semantics as logical constraints. code-scalpel does this for Python:

```
Python:  if len(username) > 0:
         Z3 encodes:  username.length > 0
         Z3 reasons:  length check != sanitization, still tainted
```

This encoding is language-specific. Every language has different:
- Truthiness rules (Python: `0`, `""`, `[]` are falsy; Rust: no implicit truthiness)
- Type coercion (JavaScript: `"5" + 3 = "53"`; Python: TypeError)
- String handling (Go: immutable strings; Ruby: mutable)
- Null semantics (Java: NullPointerException; Rust: Option<T>; JavaScript: undefined vs null)
- Control flow (Go: goroutines; Rust: ownership/borrowing; JavaScript: async/await + Promises)

code-scalpel's 2,466 lines of Z3 encoding cover Python only. Estimated effort per language:

| Language | Z3 Encoding Effort | Unique Challenges |
|----------|-------------------|-------------------|
| Python | Done (code-scalpel) — 2,466 lines | Dynamic types, duck typing |
| JavaScript | ~2,500 lines | Prototype chain, coercion, async |
| TypeScript | ~2,000 lines (extends JS) | Type narrowing, generics |
| Rust | ~3,000 lines | Ownership, borrowing, lifetimes, traits |
| Go | ~2,000 lines | Goroutines, interfaces, defer |
| Java | ~2,500 lines | Inheritance, generics erasure, checked exceptions |
| C/C++ | ~3,500 lines | Pointers, undefined behavior, macros |
| Ruby | ~2,000 lines | Open classes, method_missing, blocks |
| PHP | ~2,000 lines | Type juggling, superglobals |
| C# | ~2,000 lines | Properties, LINQ, nullable reference types |
| Swift | ~2,000 lines | Optionals, ARC, protocol extensions |

**Total estimated: ~25,000+ lines of Z3 encoding logic** to match code-scalpel's precision across Parseltongue's 12 supported languages. That's a multi-year effort with per-language PhD-level knowledge of formal semantics.

The registry approach (Layer 2 in our architecture) replaces all of this with ~50 entries per language in a TOML file — function names classified as source, sink, or sanitizer. No semantic encoding required.

---

## Problem 3: Category Mismatch — SMT Solver vs Graph Reachability

Z3 solves: "Is it **mathematically possible** for taint to reach this sink given all path constraints?"

Parseltongue needs: "Does taint **flow through the code graph** from a source to a sink without passing through a sanitizer?"

```
Z3 approach (what code-scalpel does):

    if role == "admin":
        query(user_input)       <-- Z3 asks: can role == "admin"?
                                    Z3 explores symbolic state space
                                    Z3 proves: yes, this path is feasible
                                    Result: VULNERABLE
                                    Time: seconds per function

Datalog approach (what we'd do):

    source(user_input)  --edge-->  query(user_input)
                                    Datalog asks: is there a path?
                                    Datalog follows edges
                                    Result: VULNERABLE
                                    Time: milliseconds per codebase
```

The difference:
- Z3 eliminates **infeasible paths** (paths that can never execute due to contradictory conditions). This reduces false positives.
- Datalog follows **all edges** regardless of path feasibility. This overapproximates, producing more false positives but never missing a real vulnerability.

Complexity comparison:
- Z3/SMT solving: **NP-hard** (exponential worst case per function)
- Datalog reachability: **polynomial** (linear in graph size with semi-naive evaluation)

For a function with N branches, Z3 explores up to 2^N symbolic states. Datalog follows O(E) edges where E is the number of data-flow connections. On a codebase with 19,431 entities and 144,137 edges, Datalog finishes in milliseconds. Z3 would need seconds per function, minutes to hours for the full codebase.

---

## The Precision Trade-Off (Quantified)

From the academic literature and tool benchmarks:

| Approach | False Positive Rate | Speed | Languages |
|----------|-------------------|-------|-----------|
| Z3 symbolic (code-scalpel) | ~10% | Seconds/function | 1 (Python) |
| Datalog reachability (proposed) | ~15-25% | Milliseconds/codebase | 12 |
| Type-based (SFlow/TaintTyper) | ~15% | Fast | 1-2 |
| Context-sensitive (RustGuard) | ~8% | Moderate | 1 (Rust) |
| Whole-program (CodeQL kernel) | ~90% | Minutes | Many |

The ~10% → ~20% FP increase means: for every 10 findings, Z3 flags 1 false alarm while Datalog flags 2. Both require human review. The practical difference is one extra "not a real bug" per 10 findings.

For Parseltongue's use case (developer-facing analysis tool, not CI gate), this is acceptable. The literature confirms:
- SFlow achieves 15% FP with type-based checking (no symbolic reasoning)
- CFTaint achieves <7% FP with compositional analysis (no Z3)
- AdaTaint reduces FP by 43.7% using LLM augmentation (future option for Parseltongue)

---

## What Z3 Buys That We'd Lose

To be honest about what we're giving up:

**1. Path feasibility pruning**

```python
# Z3 catches that this is unreachable:
x = input()
if False:
    query(x)    # Z3 knows: dead code, no vulnerability

# Datalog reports it as: SOURCE -> SINK path exists (false positive)
```

**2. Constraint-based sanitizer verification**

```python
# Z3 can verify the sanitizer is sufficient:
x = input()
if len(x) < 100 and x.isalnum():
    query(x)    # Z3 proves: constrained input, may be safe

# Datalog sees: no sanitizer function call, reports vulnerable
```

**3. Alias analysis through symbolic state**

```python
# Z3 tracks that a and b are the same object:
a = input()
b = a
query(b)    # Z3 traces through alias

# Datalog handles this IF we emit DataFlowEdge(a, b) from assignment
# (which we do in Layer 1 — so we actually get this one)
```

Items 1 and 2 are genuine precision losses. Item 3 we handle through data-flow edge extraction. The net precision loss is real but bounded.

---

## Why The Alternatives Are Worse Than "No Z3"

| Option | Problem |
|--------|---------|
| Vendor Z3 source | 10+ minute builds, 500K LoC C++ in tree, ARM/Windows pain |
| Use z3 crate with system Z3 | User must install Z3. Breaks "cargo install parseltongue" story |
| Use a pure-Rust SMT solver | None exist at Z3's maturity. egg/varisat are SAT-only |
| Use WASM-compiled Z3 | Possible but: 80MB+ WASM blob, 10x slower than native |
| Build our own symbolic engine | Multi-year effort, reinventing Z3 |

---

## The Decision

**Use CozoDB Datalog reachability. Do not integrate Z3.**

Rationale:
1. Zero new dependencies (tree-sitter + CozoDB already in stack)
2. 12-language support from day one (registry, not per-language encoding)
3. Millisecond query speed (polynomial, not NP-hard)
4. ~15-25% FP rate is acceptable for developer-facing tool
5. Future FP reduction via LLM augmentation (AdaTaint pattern) without Z3
6. Preserves "cargo install parseltongue" single-command install
7. Preserves embeddable-like-SQLite architectural property

If we ever need Z3-level precision for a single language (e.g., a "deep scan" mode for Python), it can be added as an optional feature behind a cargo feature flag with the z3-sys crate. But it should never be required.

---

## References

- code-scalpel taint_tracker.py: 2,466 lines, 27 SecuritySink types, 80+ sanitizers, Z3 symbolic
- Denning 1976: Lattice model maps to Datalog relations (2,079 citations)
- DOOP/Souffle: Prove Datalog is 15x faster than hand-written for points-to/taint
- SFlow: 15% FP with type-based checking, no symbolic reasoning
- CFTaint: <7% FP with compositional analysis, no Z3
- RustGuard: 91.67% precision with context-sensitive analysis, no Z3
- AdaTaint: 43.7% FP reduction via LLM augmentation (future path)
- CWE Top 25 (2025): 5/10 top CWEs are taint-detectable, justifying the investment
- Aalborg University Thesis: Rust ownership improves taint precision without symbolic reasoning

---

*Generated 2026-02-15. Architecture decision for Parseltongue v1.7.3 taint analysis.*
