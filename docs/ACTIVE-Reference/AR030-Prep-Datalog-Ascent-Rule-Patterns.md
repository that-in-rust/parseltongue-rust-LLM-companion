# Prep: Datalog Ascent Rule Patterns for Parseltongue v2.0.0

**Date**: 2026-02-16
**Context**: Deep research into the Ascent crate (Rust Datalog engine) for `rust-llm-04-reasoning-engine`. Covers Ascent syntax, base relations for code analysis, 18 derived rule patterns, CodeQL-inspired patterns, alternatives comparison, and limitations with workarounds.

**Relationship to other docs**:
- Prep-Doc-V200.md Section 4 ("Ascent for Typed Datalog Reasoning") -- this document EXTENDS that section
- PRD-v200.md -- architectural context for rust-llm-04-reasoning-engine
- THESIS-taint-analysis-for-parseltongue.md -- taint-specific rules overlap

---

## Table of Contents

1. [Ascent Crate Deep Dive](#1-ascent-crate-deep-dive)
2. [Base Relations for Code Analysis](#2-base-relations-for-code-analysis)
3. [Derived Rules Catalog (18 Rules)](#3-derived-rules-catalog-18-rules)
4. [CodeQL Rule Patterns to Learn From](#4-codeql-rule-patterns-to-learn-from)
5. [Ascent vs Alternatives](#5-ascent-vs-alternatives)
6. [Limitations and Workarounds](#6-limitations-and-workarounds)

---

## 1. Ascent Crate Deep Dive

### 1.1 What Ascent Is

Ascent is a logic programming language (similar to Datalog) embedded in Rust via procedural macros. It was published by Arash Sahebolamri (Syracuse University) with an academic paper at CC 22 (ACM SIGPLAN International Conference on Compiler Construction) titled "Seamless Deductive Inference via Macros." A follow-up OOPSLA paper covers BYODS (Bring Your Own Data Structures to Datalog).

**Key identity**: Ascent compiles Datalog rules into Rust code at compile time. No query parser at runtime. No optimizer. The Rust compiler IS the optimizer. The "database" after `prog.run()` is just `Vec<(T1, T2, ...)>` tuples in memory.

**Current version**: 0.8.0 (as of late 2025). Actively maintained. 531+ GitHub stars.

### 1.2 The ascent! Macro Syntax

#### Relation Declarations

Relations are declared with the `relation` keyword. Any type implementing `Clone + Eq + Hash` can be a column type.

```rust
ascent! {
    struct MyProgram;

    // Simple relations
    relation edge(i32, i32);
    relation path(i32, i32);

    // Relations with complex types
    relation entity(String, EntityInfo);          // String key + custom struct
    relation edge_typed(String, String, EdgeKind); // Enum column
    relation node(i32, Rc<Vec<i32>>);             // Reference-counted collection
}
```

#### Rules

Rules use `<--` (not `:-` like classic Datalog). Head on the left, body conditions on the right. Body clauses are separated by commas (conjunction). Multiple rules for the same head relation = disjunction.

```rust
ascent! {
    struct TransitiveClosure;

    relation edge(i32, i32);
    relation path(i32, i32);

    // Base case: direct edge is a path
    path(x, y) <-- edge(x, y);

    // Recursive case: transitive closure
    path(x, z) <-- edge(x, y), path(y, z);
}
```

#### Body Conditions: Iteration and Filtering

Rules can include Rust expressions in the body via `for` and `if`:

```rust
ascent! {
    struct GraphExpand;

    relation node(i32, Rc<Vec<i32>>);
    relation edge(i32, i32);

    // Iterate over neighbors and filter
    edge(x, y) <-- node(x, neighbors), for &y in neighbors.iter(), if x != y;
}
```

#### Negation (Stratified)

Negation uses the `!` prefix before a relation in the body. Internally, Ascent desugars `!relation(args)` into `agg () = ::ascent::aggregators::not() in relation(args)`. Negation is stratified: the negated relation must be fully computed in a lower stratum before use.

```rust
ascent! {
    struct DeadCode;

    relation function(String);
    relation reachable(String);
    relation dead(String);

    // Dead = declared but not reachable
    dead(f) <-- function(f.clone()), !reachable(f);
}
```

#### Aggregation

Ascent provides built-in aggregators: `sum`, `min`, `max`, `count`, `mean`. Custom aggregators are also supported. Syntax uses `agg` keyword.

```rust
ascent! {
    struct Metrics;

    relation course_grade(String, String, f64);   // student, course, grade
    relation avg_grade(String, f64);               // student, avg

    // Compute average grade per student
    avg_grade(s, avg) <--
        agg avg = mean(g) in course_grade(s, _, g);
}
```

Aggregates can also be simulated through lattices. Negation, sum, and count are achievable via a `Set` lattice. Min and max are cheaper by using `Dual` type or integers directly.

#### Lattice Support

Lattices enable fixed-point computations not expressible in standard Datalog. Declared with `lattice` keyword. The final column must implement the `Lattice` trait. When a new fact `(v1, ..., vn)` is discovered and `(v1, ..., v_n_prime)` already exists, `vn` and `v_n_prime` are `join`ed to produce a single merged fact.

```rust
use ascent::Dual;

ascent! {
    struct ShortestPath;

    relation edge(i32, i32, u32);                   // from, to, weight
    lattice shortest_path(i32, i32, Dual<u32>);      // from, to, min-distance

    // Base: direct edge
    shortest_path(x, y, Dual(w)) <-- edge(x, y, w);

    // Recursive: extend via intermediate node
    shortest_path(x, z, Dual(d1 + d2)) <--
        shortest_path(x, y, Dual(d1)),
        edge(y, z, d2);
}
```

`Dual<u32>` wraps `u32` to reverse ordering, making the lattice join compute the minimum. This is the canonical shortest-path-via-Datalog example.

### 1.3 ascent_run! vs ascent!

Both are proc-macros that expand into Rust code at compile time. Both evaluate at runtime. The difference is ergonomic.

| Feature | `ascent!` | `ascent_run!` |
|---|---|---|
| Generates | A reusable struct with `.run()` | Inline evaluation, returns results directly |
| Evaluation | Explicit: create struct, populate, call `.run()` | Immediate: runs at the point of macro invocation |
| Local variable access | No -- must set fields on struct | Yes -- local variables are in scope |
| Use case | Library APIs, reusable analysis programs | One-off computations, quick queries |
| Parallel variant | `ascent_par!` | `ascent_run_par!` |

**ascent! pattern** (reusable struct):
```rust
ascent! {
    struct CodeAnalysis;
    relation entity(String, String);
    relation edge(String, String);
    relation reachable(String, String);

    reachable(x, y) <-- edge(x.clone(), y.clone());
    reachable(x, z) <-- edge(x.clone(), y), reachable(y.clone(), z.clone());
}

fn analyze(entities: Vec<(String, String)>, edges: Vec<(String, String)>) {
    let mut prog = CodeAnalysis::default();
    prog.entity = entities;
    prog.edge = edges;
    prog.run();
    // Results in prog.reachable: Vec<(String, String)>
}
```

**ascent_run! pattern** (inline, captures locals):
```rust
fn transitive_closure(r: Vec<(i32, i32)>, reflexive: bool) -> Vec<(i32, i32)> {
    let result = ascent_run! {
        relation edge(i32, i32) = r;  // captures local variable r
        relation tc(i32, i32);

        tc(x, y) <-- edge(x, y);
        tc(x, z) <-- edge(x, y), tc(y, z);
        tc(x, x) <-- if reflexive, edge(x, _);  // captures local `reflexive`
    };
    result.tc
}
```

**For rust-llm-04**: Use `ascent!` (struct form). The reasoning engine is a persistent struct that gets populated from extractors and queried by the HTTP/MCP servers. `ascent_run!` is for ad-hoc one-off analyses.

### 1.4 Modular Composition

#### Internal Macros

Macros defined inside `ascent!` blocks expand to sequences of body or head clauses. Syntax inspired by Rust macros 2.0. Identifiers bound in macro bodies are scoped (private to invocation).

```rust
ascent! {
    struct Compiler;

    type Lang = &'static str;
    type CompilerName = &'static str;

    relation compiler(CompilerName, Lang, Lang);
    relation bad_compiler(CompilerName);
    relation can_compile_to(Lang, Lang);

    // Macro: match compiler but exclude bad ones
    macro compiler($from: expr, $to: expr) {
        compiler(name, $from, $to), !bad_compiler(name)
    }

    // Use the macro
    can_compile_to(a, b) <-- compiler!(a, b);

    // Scoped binding: two invocations don't clash on `name`
    relation compiles_in_two_steps(Lang, Lang);
    compiles_in_two_steps(a, c) <-- compiler!(a, b), compiler!(b, c);
}
```

#### Cross-Module Composition (ascent_source! / include_source!)

Reusable Ascent code modules that can be included in multiple programs:

```rust
mod analysis_rules {
    ascent::ascent_source! {
        reachability_rules:
        relation edge(String, String);
        relation reachable(String, String);

        reachable(x, y) <-- edge(x.clone(), y.clone());
        reachable(x, z) <-- edge(x.clone(), y), reachable(y.clone(), z.clone());
    }
}

// Include in a single-threaded program
ascent! {
    struct Analysis;
    include_source!(analysis_rules::reachability_rules);
}

// Include in a parallel program (same source, different execution)
ascent_par! {
    struct ParallelAnalysis;
    include_source!(analysis_rules::reachability_rules);
}
```

**For rust-llm-04**: This is how we organize rules into modules. Base relations in one source, security rules in another, architecture rules in a third. Compose them into the final `CodeAnalysis` struct.

### 1.5 Performance Characteristics

#### Semi-Naive Evaluation

Ascent walks over the SCC (Strongly Connected Component) graph of rule dependencies in topologically-sorted order. For each SCC, rules are evaluated using semi-naive evaluation. This means:

1. In each iteration, only **new** facts from the previous iteration are used to derive **new** facts
2. Facts derived in earlier iterations are not re-processed
3. This avoids re-deriving known facts, dramatically reducing work

#### Benchmark Results (CC 22 Paper -- Polonius Borrow Checker)

The paper benchmarks Ascent against Souffle (compiled to C++) on the Polonius Rust borrow checker rules. Ascent performs competitively with compiled Souffle. The key advantage is not raw speed but integration: no separate toolchain, no I/O for fact exchange, no FFI boundary. Facts flow as Rust types.

#### Relation Storage

- Relations are backed by `Vec<Tuple>` internally
- Column types require `Clone + Eq + Hash`
- Modern Datalog engines (including Ascent) maintain multiple indices per relation based on usage patterns in rules
- After `.run()`, each relation is accessible as a public field: `prog.reachable` returns `Vec<(String, String)>`

#### Code Generation Size

The generated code for a rule with N dynamic relations was reduced from O(N * 2^N) to O(N^2) in recent versions, dramatically improving compile times for programs with many rules.

#### Parallel Execution

`ascent_par!` and `ascent_run_par!` use Rayon for data-parallel evaluation. Column types must be `Send + Sync`. Parallelism controlled via `RAYON_NUM_THREADS` or `ThreadPoolBuilder`. Note: parallel mode shows variable performance in some cases due to threading overhead and hash table non-determinism.

### 1.6 BYODS (Bring Your Own Data Structures)

Relations can be backed by custom data structures via `#[ds(...)]` attributes. The `ascent-byods-rels` crate provides:

- **trrel_uf**: Union-find-based reflexive transitive relations. Dramatically speeds up transitive closure on large graphs.
- **eqrel_uf**: Union-find-based equivalence relations.

```rust
use ascent_byods_rels::trrel_uf;

ascent! {
    struct FastReachability;

    relation edge(Node, Node);

    #[ds(trrel_uf)]
    relation path(Node, Node);

    path(x, y) <-- edge(x, y);
    // trrel_uf automatically computes full transitive closure
}
```

**For rust-llm-04**: BYODS is critical for reachability and equivalence relations. The default `Vec<Tuple>` storage works for most relations, but call-graph reachability over thousands of entities benefits from `trrel_uf`.

### 1.7 Additional Features

- **measure_rule_times**: `#![measure_rule_times]` attribute that measures execution time of individual rules. Essential for profiling which rules are expensive.
- **WASM support**: Feature flag for WebAssembly environments.
- **Generic type parameters**: Generated structs can have type parameters and lifetime constraints.

---

## 2. Base Relations for Code Analysis

These are the INPUT facts populated by `rust-llm-01-fact-extractor`, `rust-llm-02-cross-lang-edges`, and `rust-llm-03-rust-analyzer`. They go INTO the Ascent program before `.run()`.

### 2.1 Entity Facts

```rust
relation entity(
    /* key */       String,     // e.g. "rust:fn:my_crate::handlers::handle_request"
    /* name */      String,     // e.g. "handle_request"
    /* kind */      EntityKind, // Enum: Fn, Struct, Trait, Class, Module, Interface, ...
    /* file */      String,     // e.g. "src/handlers.rs"
    /* line_start */ u32,
    /* line_end */   u32,
);

relation is_pub(/* key */ String);
relation is_pub_crate(/* key */ String);
relation is_pub_super(/* key */ String);
relation is_async(/* key */ String);
relation is_unsafe(/* key */ String);
relation is_const(/* key */ String);
relation is_static_method(/* key */ String);

relation has_attribute(
    /* key */       String,
    /* attr_name */ String,     // e.g. "test", "derive", "wasm_bindgen", "pyfunction"
);

relation attribute_arg(
    /* key */       String,
    /* attr_name */ String,
    /* arg */       String,     // e.g. "Clone", "Debug"
);
```

### 2.2 Type Facts

```rust
relation return_type(
    /* key */         String,
    /* type_string */ String,   // e.g. "Result<Response, AuthError>"
);

relation param(
    /* fn_key */    String,
    /* param_name */String,     // e.g. "req"
    /* type_string*/String,     // e.g. "HttpRequest"
    /* position */  u32,
);

relation field(
    /* struct_key */String,
    /* field_name */String,
    /* type_string*/String,
);

relation generic_param(
    /* key */       String,
    /* param_name */String,     // e.g. "T"
    /* position */  u32,
);

relation trait_bound(
    /* key */       String,
    /* param_name */String,     // e.g. "T"
    /* trait_name */String,     // e.g. "Display", "Send", "Clone"
);
```

### 2.3 Edge Facts

```rust
relation edge(
    /* from_key */  String,
    /* to_key */    String,
    /* edge_kind */ EdgeKind,   // Enum: Calls, Uses, Implements, Inherits, ...
);

relation in_module(
    /* key */       String,
    /* module_path*/String,     // e.g. "my_crate::handlers"
);

relation trait_impl(
    /* type_key */  String,
    /* trait_name */String,
);

relation supertrait(
    /* trait_name */String,
    /* super_name*/String,
);

relation imports(
    /* file */      String,
    /* path */      String,
);
```

### 2.4 Cross-Language Facts

```rust
relation cross_lang_edge(
    /* from_key */  String,
    /* to_key */    String,
    /* mechanism */ CrossLangMechanism, // FFI, WASM, PyO3, JNI, gRPC, MessageQueue
);

relation ffi_export(
    /* key */       String,
    /* extern_name*/String,
    /* abi */       String,     // "C", "system", etc.
);

relation string_literal(
    /* fn_key */    String,
    /* value */     String,     // e.g. "/api/v1/analyze", "user-events"
);
```

### 2.5 Semantic Facts (from rust-analyzer, Rust only)

```rust
relation resolved_type(
    /* key */       String,
    /* type_string*/String,
);

relation closure_captures(
    /* closure_key */String,
    /* captured_var*/String,
    /* capture_kind*/CaptureKind, // ByRef, ByMut, ByValue, ByMove
);

relation macro_expands_to(
    /* call_site */ String,
    /* macro_name */String,
    /* expanded_entities */ String,
);
```

### 2.6 Entry Point Facts

```rust
relation entry_point(
    /* key */       String,
    /* kind */      EntryKind,  // Main, Test, HttpHandler, GrpcHandler, Lambda, ...
);

relation taint_source(
    /* key */       String,
    /* source_kind*/String,     // "http_param", "file_read", "env_var", ...
);

relation taint_sink(
    /* key */       String,
    /* sink_kind */ String,     // "sql_exec", "shell_exec", "file_write", ...
);
```

---

## 3. Derived Rules Catalog (18 Rules)

These are COMPUTED by Ascent from the base relations above. Each rule is written in actual Ascent syntax.

### Rule 1: Transitive Trait Hierarchy

**Purpose**: Given `trait A: B` and `trait B: C`, derive that `A` transitively requires `C`.

```rust
relation all_supers(String, String);

all_supers(t, s) <-- supertrait(t.clone(), s.clone());
all_supers(t, s) <--
    all_supers(t.clone(), mid),
    supertrait(mid.clone(), s.clone());
```

**Use case**: When a type implements trait A, it must also implement all transitive supertraits. This rule computes the full obligation set.

### Rule 2: Unsafe Call Chain Propagation

**Purpose**: If function F calls function G, and G is unsafe (or transitively calls unsafe), then F participates in an unsafe chain.

```rust
relation unsafe_chain(String);

unsafe_chain(f) <-- is_unsafe(f.clone());
unsafe_chain(f) <--
    edge(f.clone(), g, kind),
    if *kind == EdgeKind::Calls,
    unsafe_chain(g.clone());
```

**Use case**: Security audit. Find all functions that transitively touch unsafe code, even if they are not themselves marked `unsafe`. Inspired by the PLDI 20 Rust safety study finding that 17/21 buffer overflow bugs follow the pattern: error in safe code, access in unsafe code.

### Rule 3: Taint Analysis (Source to Sink)

**Purpose**: Track flow of untrusted data from taint sources to taint sinks.

```rust
relation tainted(String);
relation taint_violation(String, String);

tainted(f) <-- taint_source(f.clone(), _);
tainted(g) <--
    tainted(f),
    edge(f.clone(), g.clone(), kind),
    if *kind == EdgeKind::Calls;

taint_violation(src, sink) <--
    taint_source(src.clone(), _),
    tainted(sink.clone()),
    taint_sink(sink.clone(), _);
```

**Use case**: Find paths from HTTP request handlers to SQL execution without sanitization. The classic source-sink pattern used by CodeQL, Semgrep, and every SAST tool. The Datalog formulation is 3 rules vs hundreds of lines of imperative code.

### Rule 4: Reachability (Can A Reach B?)

**Purpose**: Compute full transitive reachability over the call graph.

```rust
#[ds(trrel_uf)]
relation reachable(String, String);

reachable(a, b) <--
    edge(a.clone(), b.clone(), kind),
    if *kind == EdgeKind::Calls;
// trrel_uf automatically computes full transitive closure
```

Without BYODS, the explicit recursive version:

```rust
relation reachable(String, String);

reachable(a, b) <-- edge(a.clone(), b.clone(), kind), if *kind == EdgeKind::Calls;
reachable(a, c) <-- reachable(a.clone(), b), reachable(b.clone(), c.clone());
```

**Use case**: Foundation for dead code detection, blast radius, and impact analysis.

### Rule 5: Dead Code Detection

**Purpose**: Find entities unreachable from any entry point.

```rust
relation reachable_from_entry(String);
relation dead_code(String);

reachable_from_entry(e) <-- entry_point(e.clone(), _);
reachable_from_entry(g) <--
    reachable_from_entry(f),
    edge(f.clone(), g.clone(), kind),
    if *kind == EdgeKind::Calls;

dead_code(f) <--
    entity(f.clone(), _, kind, _, _, _),
    if *kind == EntityKind::Fn,
    !reachable_from_entry(f);
```

**Use case**: Identify functions that can be safely deleted. Uses stratified negation -- `reachable_from_entry` must be fully computed before `dead_code` is evaluated.

### Rule 6: Layer Violation Detection

**Purpose**: Detect when a handler/controller directly accesses database code, violating layered architecture.

```rust
relation in_layer(String, String);
relation layer_violation(String, String, String, String);

layer_violation(f, fl, g, gl) <--
    edge(f.clone(), g.clone(), kind),
    if *kind == EdgeKind::Calls,
    in_layer(f.clone(), fl),
    in_layer(g.clone(), gl),
    if fl != gl,
    if !is_allowed_transition(fl, gl);
```

**Use case**: Enforce architectural boundaries. "Handlers should call services, services call repositories, repositories call the database. A handler calling the database directly is a violation."

### Rule 7: Async Boundary Detection

**Purpose**: Find places where async code calls sync code (potential blocking).

```rust
relation async_boundary(String, String);
relation sync_in_async_chain(String);

async_boundary(f, g) <--
    edge(f.clone(), g.clone(), kind),
    if *kind == EdgeKind::Calls,
    is_async(f.clone()),
    !is_async(g);

sync_in_async_chain(g) <--
    async_boundary(_, g.clone());
sync_in_async_chain(h) <--
    sync_in_async_chain(g),
    edge(g.clone(), h.clone(), kind),
    if *kind == EdgeKind::Calls,
    !is_async(h);
```

**Use case**: Detect blocking calls in async contexts. A sync function that does I/O, called from an async context, will block the async runtime. This is one of the most common Tokio/async-std bugs.

### Rule 8: Circular Dependency Detection via Datalog

**Purpose**: Find circular dependencies between modules using reachability.

```rust
relation module_edge(String, String);
relation module_reaches(String, String);
relation circular_dep(String, String);

module_edge(m1, m2) <--
    edge(e1.clone(), e2.clone(), _),
    in_module(e1.clone(), m1),
    in_module(e2.clone(), m2),
    if m1 != m2;

module_reaches(a, b) <-- module_edge(a.clone(), b.clone());
module_reaches(a, c) <-- module_edge(a.clone(), b), module_reaches(b.clone(), c.clone());

circular_dep(a, b) <--
    module_reaches(a.clone(), b),
    module_reaches(b.clone(), a.clone()),
    if *a < *b;  // Avoid duplicate pairs
```

**Use case**: Architecture health. Circular module dependencies make codebases impossible to understand or refactor.

### Rule 9: API Surface Analysis

**Purpose**: Find all public entities and their transitive internal dependencies.

```rust
relation api_surface(String);
relation api_dependency(String, String);

api_surface(e) <--
    entity(e.clone(), _, _, _, _, _),
    is_pub(e.clone());

api_dependency(api, dep) <--
    api_surface(api),
    edge(api.clone(), dep.clone(), _);

api_dependency(api, dep2) <--
    api_dependency(api.clone(), dep1),
    edge(dep1.clone(), dep2.clone(), _);
```

**Use case**: Understand the true cost of a public API. Changing a public function may transitively affect many internal functions.

### Rule 10: Coupling Metrics via Datalog

**Purpose**: Compute Coupling Between Objects (CBO) using Datalog instead of imperative counting.

```rust
relation depends_on_module(String, String);
relation cbo(String, usize);

depends_on_module(m1, m2) <--
    edge(e1.clone(), e2.clone(), _),
    in_module(e1.clone(), m1),
    in_module(e2.clone(), m2),
    if m1 != m2;

cbo(m, count) <--
    agg count = count() in depends_on_module(m, _);
```

**Use case**: Replace the imperative CBO calculation in v1.x. The Datalog version is declarative and composable.

### Rule 11: Cross-Language Edge Joining (FFI Name Matching)

**Purpose**: Match Rust FFI declarations with their C/C++ implementations by name.

```rust
relation ffi_match(String, String, String);

ffi_match(rust_key, c_key, name.clone()) <--
    ffi_export(rust_key.clone(), name, abi),
    if abi == "C",
    entity(c_key.clone(), name.clone(), kind, _, _, _),
    if *kind == EntityKind::Fn;
```

**Use case**: Cross-language dependency tracking. When Rust calls `rocksdb_put` via FFI, this rule links the Rust declaration to the C implementation.

### Rule 12: Closure Capture Analysis

**Purpose**: Track what variables closures capture in unsafe contexts.

```rust
relation closure_captures_unsafe(String, String);

closure_captures_unsafe(closure, parent) <--
    closure_captures(closure.clone(), var, capture_kind),
    if *capture_kind == CaptureKind::ByMove,
    in_module(closure.clone(), parent_mod),
    in_module(parent.clone(), parent_mod.clone()),
    unsafe_chain(parent.clone());
```

**Use case**: Security audit for closures in unsafe code. Closures that capture by move in unsafe contexts are especially dangerous.

### Rule 13: Error Propagation Chains

**Purpose**: Track how Result/Option types flow through call chains.

```rust
relation returns_result(String);
relation error_chain(String, String);
relation unhandled_error(String);

returns_result(f) <--
    return_type(f.clone(), rt),
    if rt.contains("Result") || rt.contains("Option");

error_chain(a, b) <--
    edge(a.clone(), b.clone(), kind),
    if *kind == EdgeKind::Calls,
    returns_result(a.clone()),
    returns_result(b.clone());

error_chain(a, c) <--
    error_chain(a.clone(), b),
    error_chain(b.clone(), c.clone());

unhandled_error(e) <--
    entry_point(e.clone(), kind),
    if *kind == EntryKind::Main,
    returns_result(e.clone());
```

**Use case**: Find long error propagation chains where a low-level error surfaces without meaningful context.

### Rule 14: Module Cohesion Analysis

**Purpose**: Measure how interconnected entities within a module are.

```rust
relation same_module_edge(String, String, String);
relation module_entity_count(String, usize);
relation module_internal_edges(String, usize);

same_module_edge(e1, e2, m.clone()) <--
    edge(e1.clone(), e2.clone(), _),
    in_module(e1.clone(), m),
    in_module(e2.clone(), m.clone());

module_entity_count(m, count) <--
    agg count = count() in in_module(_, m);

module_internal_edges(m, count) <--
    agg count = count() in same_module_edge(_, _, m);
```

**Use case**: Low cohesion suggests the module should be split. High cohesion means the module is well-designed. Replaces v1.x LCOM metric.

### Rule 15: Test Coverage Gap Detection

**Purpose**: Find public functions that have no corresponding test.

```rust
relation is_test(String);
relation tested_fn(String);
relation untested_pub_fn(String);

is_test(f) <--
    has_attribute(f.clone(), attr),
    if attr == "test";

tested_fn(f) <-- is_test(t), edge(t.clone(), f.clone(), _);
tested_fn(f) <-- tested_fn(g), edge(g.clone(), f.clone(), _);

untested_pub_fn(f) <--
    entity(f.clone(), _, kind, _, _, _),
    if *kind == EntityKind::Fn,
    is_pub(f.clone()),
    !tested_fn(f);
```

**Use case**: CI quality gate. "Every public function must be reachable from at least one test."

### Rule 16: Derive Macro Dependency Inference

**Purpose**: When `#[derive(Clone, Debug, Serialize)]` is used, infer trait implementation edges.

```rust
relation derive_impl(String, String);

derive_impl(key, trait_name.clone()) <--
    attribute_arg(key.clone(), attr, trait_name),
    if attr == "derive";
```

**Use case**: Tree-sitter sees `#[derive(Serialize)]` but does not create a `type implements Serialize` edge. This rule materializes those implicit edges.

### Rule 17: God Object Detection

**Purpose**: Find entities with excessively high fan-in + fan-out.

```rust
relation fan_out(String, usize);
relation fan_in(String, usize);
relation god_object(String, usize, usize);

fan_out(f, count) <-- agg count = count() in edge(f, _, _);
fan_in(f, count) <-- agg count = count() in edge(_, f, _);

god_object(f, out_count, in_count) <--
    fan_out(f.clone(), out_count),
    fan_in(f.clone(), in_count),
    if *out_count > 20 && *in_count > 20;
```

**Use case**: v1.x found `runtime/db.rs` has CBO=236. This rule finds such entities automatically. Thresholds are configurable.

### Rule 18: Strongly Connected Component Membership

**Purpose**: Identify which entities belong to the same SCC (mutual reachability).

```rust
relation mutual_reach(String, String);
relation in_scc(String, String);

mutual_reach(a, b) <--
    reachable(a.clone(), b),
    reachable(b.clone(), a.clone()),
    if *a <= *b;

in_scc(a, rep.clone()) <--
    mutual_reach(rep, a.clone());
in_scc(rep, rep.clone()) <--
    mutual_reach(rep.clone(), _);
```

**Use case**: Circular dependency detection at the entity level. The SCC representative (lexicographically smallest key) serves as a group identifier.

---

## 4. CodeQL Rule Patterns to Learn From

CodeQL is GitHub's semantic code analysis engine. It treats code as a database and uses a query language (QL) to find vulnerabilities and quality issues. While CodeQL uses its own QL language (not Datalog), many of its patterns map to Datalog rules.

### 4.1 Categories of CodeQL Queries

CodeQL organizes queries by metadata tags:

| Tag | Description | Ascent Equivalent |
|---|---|---|
| `security` | Vulnerabilities (injection, XSS, etc.) | Taint analysis rules (Rule 3) |
| `correctness` | Logic bugs, incorrect behavior | Error chain analysis (Rule 13) |
| `maintainability` | Hard-to-change patterns | Coupling metrics (Rule 10), God objects (Rule 17) |
| `readability` | Confusing code patterns | N/A (requires AST-level analysis) |
| `useless-code` | Unused functions, dead code | Dead code detection (Rule 5) |
| `complexity` | Cyclomatic complexity, unclear control flow | Module cohesion (Rule 14) |
| `reliability` | Runtime failure patterns | Unhandled errors (Rule 13) |
| `performance` | Inefficient algorithms | N/A (requires execution analysis) |
| `concurrency` | Race conditions, deadlocks | Async boundary detection (Rule 7) |
| `error-handling` | Uncaught exceptions, missing checks | Error propagation chains (Rule 13) |

### 4.2 CodeQL Source-Sink Pattern (Taint Tracking)

CodeQL's most powerful pattern is the source-sink taint tracking model:

1. **Source**: Where untrusted data enters (HTTP params, file reads, env vars)
2. **Propagator**: How data flows (assignments, function calls, field access)
3. **Sanitizer**: Where data is cleaned (validation, escaping, type conversion)
4. **Sink**: Where untrusted data is dangerous (SQL exec, shell exec, file write)

**CodeQL (QL language)**:
```ql
class SqlInjectionConfig extends TaintTracking::Configuration {
  override predicate isSource(DataFlow::Node source) {
    source instanceof RemoteFlowSource
  }
  override predicate isSink(DataFlow::Node sink) {
    exists(SqlExecution exec | sink = exec.getInput())
  }
}
```

**Ascent equivalent** (Rule 3, extended with sanitizers):
```rust
relation tainted(String);
relation sanitized(String);
relation violation(String, String);

tainted(f) <-- taint_source(f.clone(), _);
tainted(g) <--
    tainted(f),
    edge(f.clone(), g.clone(), kind),
    if *kind == EdgeKind::Calls,
    !sanitized(g);

violation(src, sink) <--
    taint_source(src.clone(), _),
    tainted(sink.clone()),
    taint_sink(sink.clone(), _);
```

The key difference: CodeQL operates on data-flow graphs with fine-grained node tracking (expression-level). Ascent operates on call-graph edges (function-level). CodeQL is more precise; Ascent is faster and simpler for architectural analysis. Both find real bugs.

### 4.3 CodeQL Path Queries

CodeQL "path queries" show the full data-flow path from source to sink. We can approximate this in Ascent by recording the chain:

```rust
relation taint_path(String, String, u32);

taint_path(src, src.clone(), 0) <-- taint_source(src.clone(), _);
taint_path(src, g.clone(), hops + 1) <--
    taint_path(src.clone(), f, hops),
    edge(f.clone(), g.clone(), kind),
    if *kind == EdgeKind::Calls,
    if *hops < 20;  // prevent infinite paths
```

### 4.4 CodeQL Dead Code Patterns

| CodeQL Query | Language | What It Finds | Our Ascent Rule |
|---|---|---|---|
| `cpp/dead-code-goto` | C/C++ | Code after goto/break | N/A (needs CFG) |
| `cpp/unused-local-variable` | C/C++ | Unused locals | N/A (needs CFG) |
| `java/unused-reference-type` | Java | Unused classes | Rule 5 (dead code) |
| `java/unused-container` | Java | Collections never read | N/A (needs data flow) |
| `java/unused-parameter` | Java | Useless function params | N/A (needs data flow) |
| `js/unused-local-variable` | JS | Unused vars/imports | Rule 5 adapted |

Our Rule 5 operates at function granularity. CodeQL also operates at expression/variable granularity. For v2.0.0, function-level dead code is the priority.

### 4.5 CodeQL Architectural Rule Enforcement

CodeQL can enforce architectural rules via custom queries. Example: "REST controllers must not directly access DAO classes."

**CodeQL custom query** (Java):
```ql
from MethodAccess call, Class controller, Class dao
where
  controller.getAnAnnotation().getType().hasName("RestController") and
  dao.getAnAnnotation().getType().hasName("Repository") and
  call.getEnclosingCallable().getDeclaringType() = controller and
  call.getMethod().getDeclaringType() = dao
select call, "Direct access from controller to repository layer"
```

**Ascent equivalent** (Rule 6):
```rust
layer_violation(f, "handler", g, "database") <--
    edge(f.clone(), g.clone(), kind),
    if *kind == EdgeKind::Calls,
    has_attribute(f.clone(), attr_f),
    if attr_f == "get" || attr_f == "post" || attr_f == "put",
    in_module(g.clone(), mod_g),
    if mod_g.contains("repository") || mod_g.contains("dao");
```

### 4.6 Patterns We Can Express That CodeQL Cannot Easily

| Pattern | Why Ascent Excels |
|---|---|
| Cross-language taint flow | CodeQL is single-language per database; we join multiple extractors |
| Trait hierarchy transitivity | Native Datalog recursion; CodeQL needs explicit recursive predicates |
| Module-level coupling metrics | Ascent aggregation + lattices; CodeQL queries return findings, not metrics |
| Custom lattice-based analysis | Ascent has first-class lattices; CodeQL has limited lattice support |
| Rule composition | `include_source!` + Ascent macros; CodeQL has library imports but no macro system |

---

## 5. Ascent vs Alternatives

### 5.1 Comparison Matrix

| Feature | **Ascent** | **DDlog** | **Datafrog** | **Crepe** |
|---|---|---|---|---|
| **Approach** | Rust proc macro | Standalone lang -> Rust | Rust library (imperative) | Rust proc macro |
| **Declarative syntax** | Yes | Yes (own .dl files) | No (manual joins) | Yes |
| **Incremental** | No (batch) | Yes (differential dataflow) | No (batch) | No (batch) |
| **Lattice support** | Yes (user-defined) | Limited | No | No |
| **Parallel execution** | Yes (rayon) | Yes (timely dataflow) | No | No |
| **Negation** | Stratified | Stratified | Limited | Stratified |
| **Aggregation** | Yes (sum/min/max/count/mean/custom) | Yes | No | No |
| **BYODS** | Yes | No | No | No |
| **Rust integration** | Excellent (proc macro) | Moderate (separate compilation) | Excellent (pure library) | Excellent (proc macro) |
| **Active maintenance** | Yes (v0.8.0, 2025) | No (archived) | Low | Low |
| **Academic backing** | CC 22 + OOPSLA | CEUR-WS | None | Inspired by Souffle |
| **Production use** | Research + industry | VMware NSX | Rust borrow checker | Unknown |
| **Module composition** | ascent_source! / include_source! | Modules + imports | N/A | N/A |
| **Rule-time profiling** | Yes | Yes | Manual | No |

### 5.2 Why Ascent Wins for Our Use Case

1. **Lattices are mandatory**. We need shortest-path computations (blast radius with hop counts), coupling metrics as fixed-point lattice values, and data-flow analysis (abstract interpretation over lattices). Only Ascent and DDlog have lattices. DDlog is archived.

2. **Rust integration is mandatory**. Our facts come from Rust structs (tree-sitter, rust-analyzer). Ascent operates directly on `Clone + Eq + Hash` Rust types. No serialization boundary.

3. **Aggregation is mandatory**. Coupling metrics, cohesion scores, coverage counts all need aggregation. Only Ascent has built-in aggregators.

4. **Declarative syntax is mandatory**. We want 30+ rules that non-compiler-experts can read and modify. Datafrog requires manual join iteration code. Ascent rules read like specifications.

5. **Active maintenance matters**. DDlog is archived. Crepe and Datafrog have minimal activity. Ascent has recent releases and academic investment.

6. **BYODS is a bonus**. For transitive closure over large call graphs (thousands of entities), `trrel_uf` provides algorithmic speedup.

### 5.3 What We Lose by Not Choosing DDlog

The ONE thing DDlog has that Ascent does not: **incremental computation**. When a file changes, DDlog can recompute only the affected derived facts. Ascent must recompute everything from scratch.

**Impact for us**: On a codebase with ~3,000 entities and ~15,000 edges (CozoDB size from dogfooding), full Ascent re-evaluation takes milliseconds. Even 10x that (30K entities) is sub-second. Incremental matters at 100K+ entities. We do not need it for v2.0.0.

**Workaround if needed later**: Re-run Ascent on subgraph (only the changed modules and their transitive dependents). This is manual incremental but good enough.

### 5.4 What About Souffle?

Souffle is the gold standard for Datalog-based program analysis (used by DOOP, DDISASM, Gigahorse, Securify). It compiles Datalog to parallel C++ and is extremely fast.

**Why not Souffle**: Souffle is a standalone compiler with its own language. Facts must be serialized to TSV files, passed to the Souffle executable, and results read back. This is exactly the CozoDB pattern we are trying to eliminate. Ascent keeps everything in-process as Rust types.

Souffle has a component system (generic reusable rules) that Ascent lacks. But Ascent's `ascent_source!` + Rust macros approximate this, and the tight Rust integration more than compensates.

---

## 6. Limitations and Workarounds

### 6.1 No Incremental Computation

**Limitation**: Ascent recomputes all derived facts from scratch on each `.run()`. Retracting/deleting input facts requires full recomputation.

**Workaround for v2.0.0**:
1. Full re-evaluation is fast enough for our scale (sub-second for 30K entities)
2. On file change, only re-extract facts for changed files, then re-run Ascent with full fact set
3. If scale becomes a problem: partition by module, run Ascent per module, join results

### 6.2 No Component/Generic Abstraction

**Limitation**: Souffle has a component system where you can write generic rules parameterized over relation names. Ascent macros only substitute expressions, not entire relations.

**Workaround**:
1. Use `ascent_source!` modules for reusable rule sets
2. Use Rust's own macro system to generate `ascent!` blocks programmatically

```rust
macro_rules! define_reachability {
    ($struct_name:ident, $edge_rel:ident, $reach_rel:ident) => {
        ascent! {
            struct $struct_name;
            relation $edge_rel(String, String);
            relation $reach_rel(String, String);
            $reach_rel(a, b) <-- $edge_rel(a.clone(), b.clone());
            $reach_rel(a, c) <-- $edge_rel(a.clone(), b), $reach_rel(b.clone(), c.clone());
        }
    };
}

define_reachability!(EntityReach, entity_edge, entity_reachable);
define_reachability!(ModuleReach, module_edge, module_reachable);
```

### 6.3 No Control Flow Graph (CFG) Support

**Limitation**: Ascent reasons over facts (entities, edges). It does not have a control flow graph. Cannot detect dead code within a function, cannot do expression-level data flow, cannot detect unused local variables.

**Workaround**:
1. Function-level analysis covers 80% of practical use cases
2. CFG analysis could be added later via rust-analyzer integration
3. For variable-level analysis, use dedicated lints (clippy, rust-analyzer diagnostics)

### 6.4 String-Based Key Matching

**Limitation**: Our entity keys are strings. `String::clone()` allocates.

**Workaround**:
1. Use `Arc<str>` instead of `String`. `Arc::clone()` is a reference count bump, not a heap allocation.
2. Consider interning: map string keys to `u32` IDs before populating Ascent.
3. **Recommendation**: Start with `Arc<str>`. Profile. Switch to interning only if needed.

### 6.5 Compile Time for Large Rule Sets

**Limitation**: Programs with many rules (30+) and many relations (20+) can have long compile times.

**Workaround**:
1. Split rules into multiple `ascent!` programs (by analysis category: security, architecture, coupling)
2. Run programs sequentially, feeding derived facts from one as input to the next
3. Use `#![measure_rule_times]` to identify expensive rules

### 6.6 No Persistence

**Limitation**: Ascent is in-memory only. After `.run()`, results are `Vec<Tuple>`.

**Workaround**: This is BY DESIGN for us. `rust-llm-05-knowledge-store` handles persistence. Ascent computes; the store persists. Clean separation.

```
rust-llm-01 (extract facts)
    |
    v
rust-llm-04 (Ascent: compute derived facts)
    |
    v
rust-llm-05 (persist: HashMaps + MessagePack + secondary indices)
    |
    v
rust-llm-06 (serve: HTTP API queries the store)
```

### 6.7 No Built-in Provenance/Explanation

**Limitation**: When Ascent derives a fact, it does not record WHY (which rules and input facts produced it).

**Workaround**: Manually track provenance by adding "path" columns to rules:

```rust
relation taint_with_path(String, String, u32);

taint_with_path(f, f.clone(), 0) <-- taint_source(f.clone(), _);
taint_with_path(g, src.clone(), hops + 1) <--
    taint_with_path(f, src, hops),
    edge(f.clone(), g.clone(), kind),
    if *kind == EdgeKind::Calls,
    if *hops < 20;
```

### 6.8 Hash Table Non-Determinism in Parallel Mode

**Limitation**: `ascent_par!` results may vary between runs due to HashMap iteration order.

**Workaround**:
1. Sort results after `.run()` if deterministic output is needed
2. Use single-threaded `ascent!` for CI/testing
3. Use `ascent_par!` only for production workloads

### 6.9 No Disjunction in Rule Heads

**Limitation**: Cannot write `A(x) OR B(x) <-- C(x)`. Each rule derives into exactly one relation.

**Workaround**: Write separate rules:
```rust
A(x) <-- C(x.clone()), if is_a(x);
B(x) <-- C(x.clone()), if is_b(x);
```

### 6.10 No Recursion Through Negation

**Limitation**: Stratified negation means you cannot have mutually recursive rules where one negates the other.

**Workaround**: Redesign rules to avoid negation cycles. If needed, use a lattice with three-valued logic (True/False/Unknown) instead of negation.

---

## Appendix A: Complete Ascent Program Skeleton for rust-llm-04

```rust
use ascent::ascent;
use ascent_byods_rels::trrel_uf;

mod security_rules {
    ascent::ascent_source! {
        taint_rules:
        relation tainted(String);
        relation taint_violation(String, String);

        tainted(f) <-- taint_source(f.clone(), _);
        tainted(g) <--
            tainted(f),
            edge(f.clone(), g.clone(), kind),
            if *kind == EdgeKind::Calls,
            !sanitized(g);
        taint_violation(src, sink) <--
            taint_source(src.clone(), _),
            tainted(sink.clone()),
            taint_sink(sink.clone(), _);
    }
}

ascent! {
    struct CodeAnalysis;

    // === BASE RELATIONS (populated before .run()) ===
    relation entity(String, String, EntityKind, String, u32, u32);
    relation edge(String, String, EdgeKind);
    relation in_module(String, String);
    relation is_pub(String);
    relation is_async(String);
    relation is_unsafe(String);
    relation trait_impl(String, String);
    relation supertrait(String, String);
    relation has_attribute(String, String);
    relation attribute_arg(String, String, String);
    relation return_type(String, String);
    relation param(String, String, String, u32);
    relation entry_point(String, EntryKind);
    relation taint_source(String, String);
    relation taint_sink(String, String);
    relation sanitized(String);
    relation ffi_export(String, String, String);
    relation closure_captures(String, String, CaptureKind);

    // === DERIVED RELATIONS (computed by Ascent) ===

    include_source!(security_rules::taint_rules);

    #[ds(trrel_uf)]
    relation reachable(String, String);
    reachable(a, b) <-- edge(a.clone(), b.clone(), kind), if *kind == EdgeKind::Calls;

    relation reachable_from_entry(String);
    relation dead_code(String);
    reachable_from_entry(e) <-- entry_point(e.clone(), _);
    reachable_from_entry(g) <--
        reachable_from_entry(f), edge(f.clone(), g.clone(), _);
    dead_code(f) <--
        entity(f.clone(), _, kind, _, _, _),
        if *kind == EntityKind::Fn,
        !reachable_from_entry(f);

    relation unsafe_chain(String);
    unsafe_chain(f) <-- is_unsafe(f.clone());
    unsafe_chain(f) <--
        edge(f.clone(), g, kind),
        if *kind == EdgeKind::Calls,
        unsafe_chain(g.clone());

    relation all_supers(String, String);
    all_supers(t, s) <-- supertrait(t.clone(), s.clone());
    all_supers(t, s) <-- all_supers(t.clone(), m), supertrait(m.clone(), s.clone());

    relation async_boundary(String, String);
    async_boundary(f, g) <--
        edge(f.clone(), g.clone(), kind),
        if *kind == EdgeKind::Calls,
        is_async(f.clone()),
        !is_async(g);

    relation module_edge(String, String);
    module_edge(m1, m2) <--
        edge(e1.clone(), e2.clone(), _),
        in_module(e1.clone(), m1),
        in_module(e2.clone(), m2),
        if m1 != m2;

    // ... remaining rules from catalog ...
}
```

## Appendix B: Ascent Crate Dependencies for Cargo.toml

```toml
[dependencies]
ascent = "0.8"
ascent-byods-rels = "0.8"  # For trrel_uf, eqrel_uf
```

No other dependencies needed. No runtime. No query parser. No storage engine. Just Rust.

---

## References

### Ascent
- Ascent Homepage: https://s-arash.github.io/ascent/
- Ascent GitHub: https://github.com/s-arash/ascent
- Ascent docs.rs: https://docs.rs/ascent/latest/ascent/
- Ascent crates.io: https://crates.io/crates/ascent
- CC 22 Paper: https://thomas.gilray.org/pdf/seamless-deductive.pdf
- OOPSLA BYODS Paper: https://dl.acm.org/doi/abs/10.1145/3622840
- Ascent MACROS.MD: https://github.com/s-arash/ascent/blob/master/MACROS.MD
- ascent-byods-rels: https://docs.rs/ascent-byods-rels/latest/ascent_byods_rels/
- Ascent vs Souffle Discussion: https://github.com/s-arash/ascent/discussions/72

### Alternatives
- DDlog GitHub: https://github.com/vmware-archive/differential-datalog
- Datafrog GitHub: https://github.com/rust-lang/datafrog
- Crepe GitHub: https://github.com/ekzhang/crepe
- Souffle Homepage: https://souffle-lang.github.io/
- Souffle Examples: https://souffle-lang.github.io/examples

### CodeQL
- CodeQL Homepage: https://codeql.github.com/
- CodeQL Data Flow: https://codeql.github.com/docs/writing-codeql-queries/about-data-flow-analysis/
- CodeQL Taint Tracking: https://codeql.github.com/codeql-standard-libraries/cpp/semmle/code/cpp/dataflow/TaintTracking.qll/module.TaintTracking.html
- CodeQL Metadata Guide: https://github.com/github/codeql/blob/main/docs/query-metadata-style-guide.md
- CodeQL Dead Code: https://codeql.github.com/codeql-query-help/cpp/cpp-dead-code-goto/
- CodeQL SQL Injection: https://codeql.github.com/codeql-query-help/python/py-sql-injection/

### Datalog for Program Analysis
- Souffle for Program Analysis: https://www.javacodegeeks.com/2025/10/building-lightning-fast-program-analysis-with-souffle-and-datalog.html
- Philip Zucker Datalog Notes: https://www.philipzucker.com/notes/Languages/datalog/
- Datalog in Rust Discussion: https://lobste.rs/s/btlkeb/datalog_rust
- Semgrep Taint Analysis: https://semgrep.dev/docs/writing-rules/data-flow/taint-mode/overview
- CMU Dataflow Analysis: https://www.cs.cmu.edu/~janh/courses/411/17/lec/17-dataflow.pdf

### Rust Safety Research
- PLDI 20 Rust Safety Study: https://cseweb.ucsd.edu/~yiying/RustStudy-PLDI20.pdf
- Error Propagation Bugs: https://pages.cs.wisc.edu/~liblit/dissertations/crubio.pdf
