# v1.7.3 — pt04 Bidirectional Workflow: Compiler Truth + LLM Judgment

**Date**: 2026-02-15
**Status**: Architectural Thesis
**Depends On**: RESEARCH-v173-rustanalyzer-semantic-supergraph.md, PARSELTONGUE_V2_BIDIRECTIONAL_LLM_ENHANCEMENT.md
**Key Insight**: rust-analyzer gives us GROUND TRUTH where the bidirectional research proposed LLM GUESSING. The LLM only handles what requires JUDGMENT.

---

## The Uncomfortable Realization

The bidirectional LLM-CPU research (Feb 2026) proposed this architecture:

```
LLM reads code → extracts domain concepts → passes as hints to CPU algorithms
CPU computes with hints → returns enriched results
```

The LLM was doing two jobs:
1. **Type-level semantics**: "What type is this? Which trait does this implement? Is this a trait dispatch?" — questions with CORRECT ANSWERS
2. **Business judgment**: "Is this code critical? Is this cycle intentional? Should we fix this?" — questions requiring INTERPRETATION

pt04 (rust-analyzer as a library) gives us compiler-grade answers to job #1. Not guesses. Not 88% confidence. 100% ground truth.

The bidirectional architecture doesn't become obsolete. It becomes a **three-layer** system:

```
Layer 1: pt04 (rust-analyzer)     → GROUND TRUTH about types, traits, dispatch, visibility
Layer 2: LLM                       → JUDGMENT about business context, naming, priorities
Layer 3: CPU graph algorithms      → FAST computation over the enriched graph

What changes: The LLM stops guessing at things the compiler already knows.
What stays: The LLM still provides judgment the compiler can't.
```

---

## Rubber Duck Debugging: Why Three Layers?

**Duck**: "Can't the LLM just read the code and figure out the types?"

**Me**: It can. At 88% accuracy. Taking 45 seconds. And hallucinating the other 12%.

rust-analyzer resolves `authenticate(req)` to `AuthService::authenticate(req: &Request<Body>) -> Result<User, AuthError>` in zero extra time — it already computed this during type-checking. The LLM would read the source, follow some imports, probably get the type right, and definitely get the generic bounds wrong.

**Duck**: "So pt04 replaces the LLM entirely?"

**Me**: No. pt04 replaces the LLM for QUESTIONS WITH CORRECT ANSWERS:
- "What type does this function return?" → compiler knows
- "Which trait does this method dispatch through?" → compiler knows
- "Is this function async?" → compiler knows
- "What does this closure capture?" → compiler knows

The LLM is STILL needed for QUESTIONS REQUIRING JUDGMENT:
- "Is this code revenue-critical?" → LLM reads comments/docs
- "Is this circular dependency intentional?" → LLM recognizes design patterns
- "How should we refactor this?" → LLM suggests strategies
- "What should we name this module?" → LLM understands domain

**Duck**: "Isn't that just more complexity? Two sources of truth instead of one?"

**Me**: No. It's LESS complexity. In the original bidirectional model, the LLM was doing both jobs. It was slow (45s for module detection) because it had to read all the source code to extract type information. With pt04, the LLM only handles judgment calls — 5-10 prompts instead of 230 file reads. The LLM call drops from 45s to 3s because it's doing less work.

```
Before (bidirectional):
  LLM (types + judgment): 45s
  CPU: 0.3s
  Total: 45.3s, 91% accuracy

After (three-layer):
  pt04 (types): 0s extra (already in CozoDB from ingestion)
  LLM (judgment only): 3s
  CPU: 0.3s
  Total: 3.3s, ~96% accuracy (ground truth types + LLM judgment)
```

---

## PART I: The Five Bidirectional Workflows, Rebuilt

### Workflow 1: Semantic Module Boundary Detection

**Bidirectional (original):**
1. LLM reads 230 files, extracts function names and comments
2. LLM guesses domain concepts: "Authentication", "Logging", "Cryptography"
3. LLM maps concepts to keywords: ["auth", "verify", "login", "session"]
4. CPU runs Leiden with keyword seeds
5. LLM labels results
6. **91% accuracy, 2.1s**

**Three-layer (with pt04):**
1. pt04 already ingested typed call edges and trait impls into CozoDB
2. CPU queries: "Which entities share trait dispatch targets?"

```
# CozoDB Datalog: Find entities connected through the same traits
?[trait_name, entities] :=
    *TypedCallEdges{from_key: e1, to_key: _, via_trait: t},
    *TypedCallEdges{from_key: e2, to_key: _, via_trait: t},
    t != "",
    trait_name = t,
    entities = [e1, e2]
```

3. CPU runs Leiden with TRAIT MEMBERSHIP as seeds (not keyword guesses)
   - Entities dispatching through `trait AuthService` → same cluster
   - Entities dispatching through `trait Hasher` → different cluster
   - Entities dispatching through `trait Logger` → different cluster
4. LLM labels the clusters: "Authentication Module", "Crypto Module", "Logging Module"
5. **~96% accuracy, 0.8s** (LLM only needed for naming, not discovery)

**Why this is better:**

The original approach guessed that `authenticate` and `hash_password` belong together because they share the keyword "auth" or because they're highly coupled in the call graph. Wrong. They're coupled because authentication CALLS hashing, but they serve different concerns.

pt04 KNOWS that `authenticate` dispatches through `trait AuthService` and `hash_password` dispatches through `trait Hasher`. Different traits = different domains. No guessing.

**What the LLM still does**: Labels. "Cluster containing AuthService dispatchers" becomes "Authentication & Session Management Module." The compiler can't name things meaningfully for humans.

---

### Workflow 2: Cycle Classification (Intentional vs Bug)

**Bidirectional (original):**
1. CPU runs Tarjan's SCC → finds 5 cycles
2. LLM reads code in each cycle, checks against design pattern knowledge
3. LLM guesses: "This looks like Observer pattern" (88% confidence)
4. LLM guesses: "This looks like a God Object cycle" (87% confidence)
5. **95% accuracy, 1.3s**

**Three-layer (with pt04):**
1. CPU runs Tarjan's SCC → finds 5 cycles
2. For each cycle, CPU queries pt04's TypedCallEdges:

```
# For cycle [A, B, C, A], what connects them?
?[from, to, call_kind, via_trait] :=
    *TypedCallEdges{from_key: from, to_key: to, call_kind, via_trait},
    from in ["A", "B", "C"],
    to in ["A", "B", "C"]
```

3. **Deterministic classification rules** (no LLM needed for most cases):

```
Rule 1: ALL edges in cycle are TraitMethod dispatch
  → Query SupertraitEdges to check if traits form a known hierarchy
  → If traits match Observer/Subject, Visitor/Element, Handler/Middleware patterns
  → Classification: INTENTIONAL_PATTERN (100% confidence — compiler proved it)

Rule 2: ALL edges are Direct calls, no trait dispatch
  → Classification: LIKELY_VIOLATION (high confidence)
  → Indicates tight coupling without abstraction layer

Rule 3: Mix of TraitMethod and Direct
  → Need LLM judgment: "Is this a partially-abstracted design pattern, or a mess?"

Rule 4: Any edge is ClosureInvoke
  → Closures creating cycles are almost always unintentional
  → Check capture kinds: MutableRef captures in cycles = definite bug
```

4. LLM only called for Rule 3 (ambiguous cases) — maybe 1 out of 5 cycles
5. **~99% accuracy on clear cases, ~93% on ambiguous cases, 0.4s average**

**The key difference**: The original approach sent ALL cycles to the LLM. With pt04, most cycles classify themselves. The compiler already KNOWS whether the connection is through trait dispatch (likely intentional) or direct calls (likely violation). LLM is only needed for edge cases.

**Shreyas would say**: "You went from 5 LLM calls to 1 LLM call. That's an 80% reduction in the slow part of the pipeline."

---

### Workflow 3: Complexity Classification (Essential vs Accidental)

**Bidirectional (original):**
1. CPU calculates McCabe complexity: function has 15 branches
2. LLM reads source code, determines if function has single responsibility
3. LLM guesses: "Essential complexity" or "Accidental complexity"
4. **93% accuracy, 2.8s**

**Three-layer (with pt04):**
1. CPU calculates McCabe complexity: 15 branches
2. CPU queries pt04's TypedCallEdges for the function body:

```
# How many distinct trait interfaces does this function consume?
?[trait_count, traits] :=
    *TypedCallEdges{from_key: "rust:fn:process_request", via_trait: t},
    t != "",
    traits = collect(t),
    trait_count = count(t)
```

3. **Quantitative single-responsibility check**:

```
trait_count = 1  → Single responsibility (essential complexity)
trait_count = 2  → Probably fine (calls auth + processes)
trait_count >= 4 → Multiple responsibilities (accidental complexity)
                   Each trait interface = a different concern

Also check: How many MODULES do the callees belong to?
callee_modules = 1  → Cohesive (essential)
callee_modules >= 4 → Scattered (accidental)
```

4. For borderline cases (trait_count = 2-3), ask LLM: "Is this justified?"
5. **~95% accuracy on clear cases (0.1s), ~90% on borderline (3s)**

**Why this is better:**

The bidirectional approach asked the LLM: "Does this function have single responsibility?" That's a judgment call. The LLM reads the code and... interprets vibes.

pt04 turns it into a measurement: "This function dispatches through 5 different trait interfaces from 4 different modules." That's not a vibe. That's a number. If you consume 5 different abstractions, you have 5 responsibilities. The compiler counted them.

**What the LLM still does**: For the borderline case (a function that dispatches through 2 traits and they're closely related), the LLM decides if it's justified. "This function calls AuthService and Logger — logging auth events is a single responsibility that naturally spans two interfaces." That's judgment. The compiler can't make that call.

---

### Workflow 4: Business-Aware Tech Debt Scoring

**Bidirectional (original):**
1. CPU runs SQALE → raw debt scores
2. LLM reads comments, git history, docs → classifies business criticality
3. CPU recomputes: `Score = SQALE x Business_Weight x Churn x Centrality`
4. **89% correct prioritization, 4.2s**

**Three-layer (with pt04):**
1. CPU runs SQALE → raw debt scores
2. pt04 adds QUANTITATIVE debt signals the compiler can see:

```
# Memory waste: padding bytes as tech debt
?[entity, size, padding, waste_ratio] :=
    *TypeLayouts{ISGL1_key: entity, size_bytes: size, padding_bytes: padding},
    waste_ratio = padding / size,
    waste_ratio > 0.25  # More than 25% padding = field ordering debt

# Visibility bloat: public items only used internally
?[entity, visibility] :=
    *SemanticTypes{ISGL1_key: entity, visibility: "pub"},
    not *TypedCallEdges{from_key: external, to_key: entity},
    # No external callers → pub is unnecessary

# Unused trait implementations
?[impl_entity, trait_name] :=
    *TraitImpls{impl_key: impl_entity, trait_name},
    not *TypedCallEdges{to_key: impl_entity}
    # Nobody calls through this trait impl
```

3. **Enhanced formula**:

```
Score = SQALE
      x Business_Weight (LLM provides)
      x Churn (git provides)
      x Centrality (graph algorithm provides)
      x (1 + padding_waste_ratio)     ← pt04 NEW
      x (1 + unnecessary_pub_count)   ← pt04 NEW
      x (1 + unused_impl_count)       ← pt04 NEW
```

4. LLM still classifies business criticality (can't be automated)
5. **~92% correct prioritization, 2.5s** (fewer LLM calls, more compiler data)

**What pt04 adds that the LLM couldn't see:**
- 40 bytes of padding in a hot-path struct = performance debt
- 12 `pub` functions that nobody outside the module calls = API surface debt
- 3 trait impls that nothing dispatches through = dead abstraction debt

These are invisible to the LLM because they require type-checking the entire crate. But rust-analyzer already did that.

---

### Workflow 5: Refactoring Suggestions

**Bidirectional (original):**
1. CPU detects: high coupling (23 dependencies), low cohesion (0.34)
2. LLM analyzes code structure, identifies refactoring pattern
3. LLM suggests: "Split God Object", "Extract Interface", "Dependency Inversion"
4. LLM generates pseudocode
5. **91% helpful, cost = many LLM tokens**

**Three-layer (with pt04):**
1. CPU detects: high coupling (23 dependencies), low cohesion (0.34)
2. pt04 provides EVIDENCE for specific refactorings:

```
# "Extract Interface" evidence:
# Multiple structs already implement the same set of methods
?[method_set, structs] :=
    *TraitImpls{self_type: s1, trait_name: ""},  # inherent impls
    *Function{parent: s1, name: method},
    *TraitImpls{self_type: s2, trait_name: ""},
    *Function{parent: s2, name: method},
    s1 != s2,
    method_set = collect(method),
    structs = [s1, s2]
# → "StripeProcessor and PayPalProcessor both have process(), refund(), authorize()"
# → Extract trait PaymentProcessor

# "Split God Object" evidence:
# Entity dispatches through many unrelated traits
?[entity, trait_count, traits] :=
    *TypedCallEdges{from_key: entity, via_trait: t},
    t != "",
    traits = collect(t),
    trait_count = count(t),
    trait_count > 4
# → "RequestHandler dispatches through AuthService, Logger, Cache, FeatureFlag, Processor"
# → Split into 5 structs, one per responsibility

# "Dependency Inversion" evidence:
# Direct calls that could be trait dispatches
?[from, to] :=
    *TypedCallEdges{from_key: from, to_key: to, call_kind: "Direct"},
    *TypedCallEdges{from_key: from2, to_key: to, call_kind: "TraitMethod"},
    from != from2
# → "Some callers use the trait, others bypass it with direct calls"
# → "Make all callers go through the trait"
```

3. LLM receives the compiler evidence and writes the human-readable suggestion:
   - "RequestHandler dispatches through 5 traits: split into AuthHandler (AuthService), LogHandler (Logger), CacheHandler (Cache), FlagHandler (FeatureFlag), CoreProcessor (Processor)"
   - Evidence-based, not pattern-matched
4. **~95% helpful, faster LLM calls (evidence provided, not source code)**

**The shift**: The LLM goes from "read 500 lines of source code and guess the pattern" to "here's what the compiler found, write the recommendation in English." Cheaper, faster, more accurate.

---

## PART II: Every Current Endpoint — How pt04 Enriches It

This section walks through ALL 26 HTTP endpoints. For each: what it does today (tree-sitter only), what pt04 adds (rust-analyzer semantics), and what the LLM companion gains.

### STEP 1 — ORIENT Endpoints

#### 1. `/server-health-check-status`

**Today**: Returns `"healthy"`, `file_watcher_active`, `database_connected`.

**With pt04**: Add `rust_analyzer_loaded: true/false`, `semantic_relations_count: 12450`, `workspace_crates: 4`. The health check tells the LLM whether compiler semantics are available — because pt04 is Rust-only, the LLM needs to know whether it can trust typed edges or must fall back to tree-sitter-only analysis.

```json
{
  "status": "healthy",
  "file_watcher_active": true,
  "database_connected": true,
  "semantic_layer": {
    "rust_analyzer_loaded": true,
    "typed_edges_count": 12450,
    "trait_impls_count": 340,
    "workspace_crates": 4,
    "last_semantic_sync": "2026-02-15T10:30:00Z"
  }
}
```

**LLM gains**: The LLM knows upfront whether to ask typed questions ("what trait does this dispatch through?") or stick to syntactic questions ("who calls this?"). No wasted queries.

---

#### 2. `/codebase-statistics-overview-summary`

**Today**: `code_entities_total_count: 755`, `dependency_edges_total_count: 4055`, `languages_detected_list`.

**With pt04**: Add semantic depth metrics:

```json
{
  "code_entities_total_count": 755,
  "dependency_edges_total_count": 4055,
  "languages_detected_list": ["rust", "javascript"],
  "semantic_enrichment": {
    "typed_call_edges": 3200,
    "untyped_call_edges": 855,
    "trait_implementations": 340,
    "async_functions": 47,
    "unsafe_blocks": 12,
    "generic_functions": 89,
    "closure_captures": 156,
    "pub_api_surface": 210,
    "pub_crate_internal": 145,
    "private_items": 400
  }
}
```

**LLM gains**: "This codebase has 47 async functions, 12 unsafe blocks, and 89 generics" — the LLM immediately knows the architectural flavor. High async count = need to trace async chains. Unsafe blocks = need safety audit. High generics = monomorphization cost concerns. None of this is visible from tree-sitter entity counts.

**New insight for the LLM**: The ratio `typed_call_edges / total_edges` = 79%. That means 79% of the call graph has compiler-verified dispatch information. The other 21% are cross-language or external calls. The LLM knows exactly how much to trust the typed analysis.

---

#### 3. `/folder-structure-discovery-tree`

**Today**: L1/L2 folder tree with entity counts per folder.

**With pt04**: Add crate boundaries and module visibility:

```json
{
  "folders": [
    {
      "path": "crates/parseltongue-core",
      "entity_count": 340,
      "crate_root": true,
      "pub_exports": 85,
      "crate_dependencies": ["serde", "cozo", "tree-sitter"],
      "internal_modules": ["parser", "storage", "entities", "isgl1"],
      "reexports_count": 12
    }
  ]
}
```

**LLM gains**: The LLM sees CRATE BOUNDARIES, not just folders. In Rust, `crates/parseltongue-core/` isn't just a folder — it's a compilation unit with a public API surface (`pub_exports: 85`) and internal modules. The LLM knows that changes to `pub` items in `parseltongue-core` affect all downstream crates, while `pub(crate)` items are safe to refactor internally.

**Workflow impact**: When the LLM does "New Codebase Orientation," it now sees the Rust-specific architecture: which crates depend on which, where the API boundaries are, which folders are internal modules vs standalone crates. Tree-sitter treats every `.rs` file the same — pt04 understands crate-level structure.

---

#### 4. `/api-reference-documentation-help`

**Today**: Self-documenting list of all 26 endpoints with descriptions.

**With pt04**: Add a new section documenting semantic query capabilities:

```json
{
  "standard_endpoints": [...],
  "semantic_query_capabilities": {
    "description": "When rust_analyzer_loaded is true, these additional query parameters are available",
    "typed_edge_filters": ["call_kind=Direct|TraitMethod|DynDispatch|ClosureInvoke", "via_trait=TraitName"],
    "type_queries": ["return_type=Result", "param_type=&str", "is_async=true", "is_unsafe=true"],
    "visibility_filters": ["visibility=pub|pub_crate|pub_super|private"],
    "trait_queries": ["implements=Display", "supertrait_of=Handler"]
  }
}
```

**LLM gains**: The LLM discovers new query dimensions dynamically. When the `/api-reference` tells it "you can filter edges by `call_kind=TraitMethod`," the LLM immediately knows to ask more precise questions. Self-documenting semantic capabilities = LLM doesn't need to be pre-programmed with pt04 knowledge.

---

### STEP 2 — FIND ENTITIES Endpoints

#### 5. `/code-entities-list-all`

**Today**: Returns entities with `key`, `name`, `entity_type`, `language`, `file_path`, `line_range`.

**With pt04**: Enrich each entity with compiler-resolved metadata:

```json
{
  "key": "rust:fn:authenticate:src_auth_rs:T1701234567",
  "name": "authenticate",
  "entity_type": "function",
  "language": "rust",
  "file_path": "src/auth.rs",
  "line_range": [10, 50],
  "semantic": {
    "return_type": "Result<User, AuthError>",
    "params": [{"name": "req", "type": "&Request<Body>"}],
    "is_async": true,
    "is_unsafe": false,
    "visibility": "pub(crate)",
    "generic_params": [],
    "trait_impls": ["AuthService"],
    "outgoing_dispatch_kinds": {"Direct": 3, "TraitMethod": 2, "ClosureInvoke": 1}
  }
}
```

**LLM gains**: Massive. Today the LLM sees `authenticate` is a function in `auth.rs`. With pt04, the LLM sees:
- It returns `Result<User, AuthError>` — it can fail, and the error type is domain-specific
- It takes `&Request<Body>` — it's an HTTP handler, not a generic auth function
- It's `async` — needs tokio runtime, can't be called from sync code
- It's `pub(crate)` — safe to refactor without affecting external consumers
- It implements `AuthService` — part of a trait-based architecture
- It makes 2 trait method dispatches — it's not a monolith, it delegates

**Today the LLM would need to**: Read the source file, parse the signature, follow imports to resolve `User` and `AuthError`, check if it's async, check visibility. That's 3-5 tool calls and token-expensive source reading. With pt04, it's one entity in the list response.

---

#### 6. `/code-entities-search-fuzzy?q=PATTERN`

**Today**: Fuzzy name matching. `?q=auth` returns entities with "auth" in the name.

**With pt04**: Add TYPE-BASED SEARCH:

```bash
# Search by name (existing)
curl "http://localhost:7777/code-entities-search-fuzzy?q=auth"

# NEW: Search by return type
curl "http://localhost:7777/code-entities-search-fuzzy?returns=Result<User>"

# NEW: Search by parameter type
curl "http://localhost:7777/code-entities-search-fuzzy?accepts=Request<Body>"

# NEW: Search by trait implementation
curl "http://localhost:7777/code-entities-search-fuzzy?implements=Display"

# NEW: Search by visibility
curl "http://localhost:7777/code-entities-search-fuzzy?visibility=pub"

# NEW: Combined
curl "http://localhost:7777/code-entities-search-fuzzy?q=handle&is_async=true&returns=Result"
```

**LLM gains**: The LLM can ask: "Find all functions that accept a `&Database` parameter" — that's a question about the DATABASE LAYER, not about naming conventions. Today, `?q=database` might find `database_init` and `database_config` but miss `execute_query(db: &Database)` and `run_migration(conn: &Database)` because their names don't contain "database."

Type-based search finds functions by what they DO (accept/return types) not what they're CALLED. This is the difference between syntactic search and semantic search.

**New LLM workflow**: "I'm debugging an auth issue. Show me all functions that return `AuthError`." → pt04 finds every function where `AuthError` appears in the return type, regardless of name. Tree-sitter can't do this because `Result<User, AuthError>` is just a string in the source — only the compiler knows that `AuthError` is the error variant of the Result.

---

#### 7. `/code-entity-detail-view?key=X`

**Today**: Full entity metadata + source code.

**With pt04**: Add resolved type context that the source code alone doesn't reveal:

```json
{
  "key": "rust:fn:process_batch:src_pipeline_rs:T1701234567",
  "source_code": "pub async fn process_batch<T: Serialize + Send>(items: Vec<T>, db: &Database) -> Result<BatchResult, PipelineError> { ... }",
  "semantic_detail": {
    "fully_resolved_signature": {
      "generic_params": [{"name": "T", "bounds": ["Serialize", "Send"]}],
      "params": [
        {"name": "items", "type": "Vec<T>", "resolved_bounds": "T must be Serialize + Send"},
        {"name": "db", "type": "&Database", "is_reference": true, "mutability": "immutable"}
      ],
      "return_type": {
        "outer": "Result",
        "ok": "BatchResult",
        "err": "PipelineError"
      }
    },
    "trait_dispatch_targets": [
      {"callee": "serde::Serialize::serialize", "call_kind": "TraitMethod", "via_trait": "Serialize"},
      {"callee": "Database::execute", "call_kind": "Direct"}
    ],
    "closure_captures": [
      {"variable": "db", "capture_kind": "SharedRef", "in_closure_at_line": 35}
    ],
    "unsafe_operations": [],
    "impl_block": null,
    "containing_module": "pipeline",
    "visibility_effective": "pub (re-exported from crate root)"
  }
}
```

**LLM gains**: When the LLM reads entity detail today, it sees source code and must PARSE it mentally. With pt04, the LLM gets pre-parsed compiler analysis:
- Generic bounds are resolved (the LLM doesn't need to chase `where` clauses)
- Every outgoing call is classified (the LLM knows which calls are trait dispatches)
- Closures and their captures are explicit (the LLM doesn't need to read closure bodies)
- Effective visibility accounts for re-exports (just because it's `pub(crate)` in source doesn't mean it's not re-exported as `pub` at the crate root)

**Key shift**: Today, `/code-entity-detail-view` gives the LLM source code to READ. With pt04, it gives the LLM compiler ANALYSIS to REASON about. Reading is slow and error-prone. Reasoning over pre-computed facts is fast and accurate.

---

### STEP 3 — TRACE DEPENDENCIES Endpoints

#### 8. `/dependency-edges-list-all`

**Today**: Returns edges with `from_key`, `to_key`, `edge_type` (Calls, Uses, Implements, Extends, Contains).

**With pt04**: Enrich every edge with dispatch semantics:

```json
{
  "edges": [
    {
      "from_key": "rust:fn:handle_request:...",
      "to_key": "rust:fn:authenticate:...",
      "edge_type": "Calls",
      "semantic": {
        "call_kind": "TraitMethod",
        "via_trait": "AuthService",
        "receiver_type": "Box<dyn AuthService>",
        "is_async_call": true,
        "generic_instantiation": null
      }
    },
    {
      "from_key": "rust:fn:handle_request:...",
      "to_key": "rust:fn:log_access:...",
      "edge_type": "Calls",
      "semantic": {
        "call_kind": "Direct",
        "via_trait": null,
        "receiver_type": null,
        "is_async_call": false,
        "generic_instantiation": null
      }
    },
    {
      "from_key": "rust:fn:process_items:...",
      "to_key": "rust:fn:transform:...",
      "edge_type": "Calls",
      "semantic": {
        "call_kind": "ClosureInvoke",
        "via_trait": "FnMut",
        "receiver_type": null,
        "is_async_call": false,
        "closure_captures": ["items: SharedRef", "config: Move"]
      }
    }
  ]
}
```

**LLM gains**: TODAY: "handle_request calls authenticate" — that's a fact, but it doesn't tell the LLM HOW. WITH PT04: "handle_request calls authenticate via trait dispatch through Box<dyn AuthService>" — now the LLM knows:
- This is a polymorphic call. At runtime, ANY type implementing AuthService could be called.
- Changing `authenticate`'s signature requires updating the TRAIT, not just the function.
- The `Box<dyn>` means dynamic dispatch — there's a vtable lookup at runtime.
- Adding a new AuthService impl means adding a new possible path through this edge.

**Edge filtering unlocked**:
```bash
# Only trait dispatches (architectural boundaries)
curl "http://localhost:7777/dependency-edges-list-all?call_kind=TraitMethod"

# Only closure invocations (potential capture bugs)
curl "http://localhost:7777/dependency-edges-list-all?call_kind=ClosureInvoke"

# Only async calls (async chain analysis)
curl "http://localhost:7777/dependency-edges-list-all?is_async=true"
```

---

#### 9. `/reverse-callers-query-graph?entity=X`

**Today**: "Who calls entity X?" Returns a flat list of caller keys.

**With pt04**: Classify callers by HOW they call:

```json
{
  "entity": "rust:fn:authenticate:...",
  "callers": {
    "direct_callers": [
      {"from_key": "rust:fn:test_auth:...", "context": "direct call in test"}
    ],
    "trait_dispatch_callers": [
      {"from_key": "rust:fn:handle_request:...", "via_trait": "AuthService", "receiver_type": "Box<dyn AuthService>"},
      {"from_key": "rust:fn:middleware_chain:...", "via_trait": "AuthService", "receiver_type": "Arc<dyn AuthService>"}
    ],
    "closure_callers": [],
    "total_callers": 3,
    "polymorphic_callers": 2,
    "direct_callers_count": 1
  }
}
```

**LLM gains**: "2 out of 3 callers use trait dispatch" — this tells the LLM that `authenticate` is primarily consumed through its trait interface, not directly. Refactoring implication: changing the trait method signature is HIGH IMPACT (affects polymorphic callers), but changing the internal implementation is LOW IMPACT (callers go through the trait boundary).

**Today the LLM would need to**: Read source code of each caller to determine if they call through a trait or directly. With pt04, it's in the response.

**New risk classification**:
- ALL callers are Direct → Low refactoring risk (just update call sites)
- ALL callers are TraitMethod → Medium risk (update trait definition)
- Mix of Direct + TraitMethod → HIGH risk (some callers bypass the trait boundary, they might break independently)

---

#### 10. `/forward-callees-query-graph?entity=X`

**Today**: "What does entity X call?" Returns flat list of callee keys.

**With pt04**: Enrich with dispatch semantics and dependency classification:

```json
{
  "entity": "rust:fn:handle_request:...",
  "callees": {
    "by_dispatch_kind": {
      "Direct": [
        {"to_key": "rust:fn:log_access:...", "module": "logging"}
      ],
      "TraitMethod": [
        {"to_key": "rust:fn:authenticate:...", "via_trait": "AuthService", "module": "auth"},
        {"to_key": "rust:fn:rate_limit:...", "via_trait": "RateLimiter", "module": "throttle"}
      ],
      "DynDispatch": [
        {"to_key": "rust:fn:render:...", "via_trait": "Renderer", "module": "ui"}
      ],
      "ClosureInvoke": [
        {"to_key": "rust:fn:map_result:...", "captures": ["ctx: SharedRef"]}
      ]
    },
    "unique_traits_consumed": ["AuthService", "RateLimiter", "Renderer"],
    "unique_modules_touched": ["auth", "throttle", "ui", "logging"],
    "responsibility_score": 4,
    "total_callees": 5
  }
}
```

**LLM gains**: `unique_traits_consumed: 3` + `unique_modules_touched: 4` = the LLM immediately sees this function has 4 responsibilities. It doesn't need to read the source code to detect a God Object — the forward callees response already measures it.

**New refactoring signal**: If `responsibility_score >= 4`, the LLM can suggest splitting the function BEFORE reading any source code. The compiler already proved it touches too many concerns.

---

#### 11. `/blast-radius-impact-analysis?entity=X&hops=N`

**Today**: Returns `total_affected`, `direct_callers`, `transitive` counts, and `affected_entities` list.

**With pt04**: Add TYPED blast radius — distinguish architectural impact from implementation impact:

```json
{
  "focus_entity": "rust:trait:AuthService:...",
  "blast_radius": {
    "hop_1": 15,
    "hop_2": 47,
    "total_impacted": 62
  },
  "typed_blast_radius": {
    "trait_boundary_crossings": 3,
    "affected_via_trait_dispatch": 42,
    "affected_via_direct_call": 20,
    "affected_crates": ["parseltongue-core", "pt01-folder-to-cozodb-streamer", "pt08-http-code-query-server"],
    "sealed_by_visibility": 8,
    "safety_assessment": {
      "can_change_internally": true,
      "can_change_signature": false,
      "reason": "2 external crates dispatch through this trait"
    }
  }
}
```

**LLM gains**: Today, blast radius says "62 things are affected." With pt04, it says:
- "42 of those 62 are affected THROUGH TRAIT DISPATCH" — they'll break if the trait signature changes, but NOT if the implementation changes
- "20 are affected by direct calls" — they break on any change
- "3 external crates are affected" — this is a cross-crate API change
- "8 entities are sealed by visibility" — they CAN'T be affected because they're private/pub(crate) in a different module

**The key insight**: Not all blast radius is equal. Changing a trait method's implementation (body) affects 0 callers. Changing a trait method's signature affects 42 callers. Changing a private helper function affects 3 callers. Today's blast radius conflates all three. pt04 separates them.

**New LLM workflow**: "Is it safe to refactor this?" →
1. Check `can_change_internally` — if true, body changes are safe
2. Check `affected_crates` — if only current crate, signature changes are contained
3. Check `sealed_by_visibility` — some "affected" entities are actually unreachable

---

### STEP 4 — ANALYZE ARCHITECTURE Endpoints

#### 12. `/circular-dependency-detection-scan`

**Today**: Returns cycles found, paths.

**With pt04**: Auto-classify each cycle (replaces LLM guessing):

```json
{
  "cycles_found": 3,
  "classified_cycles": [
    {
      "path": ["A", "B", "C", "A"],
      "all_edge_kinds": ["TraitMethod", "TraitMethod", "TraitMethod"],
      "all_via_traits": ["Observer", "Subject", "EventHandler"],
      "classification": "INTENTIONAL_PATTERN",
      "pattern_match": "Observer/Subject",
      "confidence": "COMPILER_VERIFIED",
      "action": "NONE_REQUIRED"
    },
    {
      "path": ["X", "Y", "X"],
      "all_edge_kinds": ["Direct", "Direct"],
      "all_via_traits": [null, null],
      "classification": "LIKELY_VIOLATION",
      "confidence": "HIGH",
      "action": "EXTRACT_INTERFACE"
    },
    {
      "path": ["P", "Q", "R", "P"],
      "all_edge_kinds": ["TraitMethod", "Direct", "ClosureInvoke"],
      "all_via_traits": ["Handler", null, "FnMut"],
      "classification": "AMBIGUOUS",
      "confidence": "NEEDS_LLM_JUDGMENT",
      "action": "REVIEW_REQUIRED"
    }
  ]
}
```

**LLM gains**: 2 out of 3 cycles auto-classified. The LLM only needs to reason about the AMBIGUOUS one. Today, the LLM reads source code for ALL cycles. With pt04, the endpoint does the classification work, and the LLM focuses judgment on edge cases.

---

#### 13. `/complexity-hotspots-ranking-view?top=N`

**Today**: Returns entities ranked by `in_degree + out_degree = total_coupling`.

**With pt04**: Decompose coupling into semantic categories:

```json
{
  "hotspots": [
    {
      "entity": "rust:fn:execute_query:...",
      "total_coupling": 35,
      "coupling_breakdown": {
        "direct_call_in": 8,
        "direct_call_out": 12,
        "trait_dispatch_in": 4,
        "trait_dispatch_out": 5,
        "closure_in": 0,
        "closure_out": 6
      },
      "unique_traits_consumed": 5,
      "unique_traits_provided": 2,
      "complexity_kind": "JUNCTION_NODE",
      "explanation": "High coupling through 5 different trait interfaces — probable God Object"
    }
  ]
}
```

**LLM gains**: "35 total coupling" is a number. "5 different trait interfaces consumed" is a diagnosis. The LLM now KNOWS the coupling is across trait boundaries (God Object pattern), not within a single module (high cohesion, potentially acceptable).

**New `complexity_kind` values**:
- `JUNCTION_NODE`: High coupling across many traits → God Object
- `HUB_NODE`: High coupling within one trait → Central utility (often intentional)
- `CLOSURE_HEAVY`: High coupling from closures → Callback hell pattern
- `BRIDGE_NODE`: High betweenness, low degree → Bottleneck/gateway (often intentional)

---

#### 14. `/semantic-cluster-grouping-list`

**Today**: Groups entities by file path / namespace.

**With pt04**: Group by TRAIT MEMBERSHIP (shared trait dispatch targets):

```json
{
  "clusters": [
    {
      "id": 0,
      "label": "AuthService dispatchers",
      "trait_anchor": "AuthService",
      "members": ["rust:fn:handle_login:...", "rust:fn:handle_logout:...", "rust:fn:handle_refresh:..."],
      "cohesion": 0.92,
      "internal_dispatch_kind": "TraitMethod"
    },
    {
      "id": 1,
      "label": "Database consumers",
      "trait_anchor": "DatabaseOps",
      "members": ["rust:fn:execute_query:...", "rust:fn:run_migration:...", "rust:fn:seed_data:..."],
      "cohesion": 0.88,
      "internal_dispatch_kind": "Direct"
    }
  ]
}
```

**LLM gains**: Today's clusters are file-based: "everything in `auth.rs` is the auth cluster." That's folder structure, not architecture. With pt04, clusters are TRAIT-based: "everything dispatching through `AuthService` is the auth cluster." This is the ACTUAL semantic boundary — functions that share a trait interface are architecturally coupled, regardless of which file they live in.

---

#### 15. `/strongly-connected-components-analysis`

**Today**: Returns SCCs with `id`, `size`, `members`, `risk_level`.

**With pt04**: Add edge-type analysis per SCC (same as cycle detection enrichment):

```json
{
  "sccs": [
    {
      "id": 0,
      "size": 3,
      "members": ["D", "E", "F"],
      "risk_level": "HIGH",
      "edge_analysis": {
        "internal_edges": 4,
        "trait_dispatch_edges": 3,
        "direct_call_edges": 1,
        "dominant_traits": ["Observer", "EventHandler"],
        "classification": "INTENTIONAL_PATTERN",
        "pattern": "Observer pattern cycle"
      }
    }
  ]
}
```

**LLM gains**: Same as `/circular-dependency-detection-scan` enrichment — the SCC comes pre-classified. The compiler already determined whether the cycle is through trait boundaries (likely intentional architecture) or direct calls (likely coupling bug).

---

#### 16. `/technical-debt-sqale-scoring`

**Today**: ISO 25010 SQALE debt based on CK metrics (CBO, LCOM, WMC thresholds).

**With pt04**: Add COMPILER-DETECTED debt categories invisible to tree-sitter:

```json
{
  "entities": [
    {
      "entity": "rust:struct:AppState:...",
      "total_debt_hours": 14.0,
      "violations": [
        {"type": "HIGH_COUPLING", "metric": "CBO", "value": 12, "threshold": 10, "remediation_hours": 4.0},
        {"type": "VISIBILITY_BLOAT", "metric": "unnecessary_pub", "value": 8, "threshold": 0, "remediation_hours": 2.0, "detail": "8 pub methods with 0 external callers"},
        {"type": "PADDING_WASTE", "metric": "padding_ratio", "value": 0.35, "threshold": 0.25, "remediation_hours": 1.0, "detail": "35% of struct size is padding — reorder fields"},
        {"type": "DEAD_TRAIT_IMPL", "metric": "unused_impls", "value": 2, "threshold": 0, "remediation_hours": 1.0, "detail": "Implements Display and Debug but neither is dispatched through"},
        {"type": "EXCESSIVE_GENERICS", "metric": "generic_params", "value": 4, "threshold": 3, "remediation_hours": 2.0, "detail": "4 generic parameters — consider trait objects"},
        {"type": "UNSAFE_SURFACE", "metric": "unsafe_blocks", "value": 3, "threshold": 0, "remediation_hours": 4.0, "detail": "3 unsafe blocks — each needs safety invariant documentation"}
      ],
      "severity": "HIGH"
    }
  ]
}
```

**LLM gains**: Today's SQALE catches coupling and complexity. With pt04, it also catches:
- **Visibility bloat**: `pub` items nobody external calls → shrink API surface
- **Padding waste**: struct field ordering causes memory waste → reorder fields
- **Dead trait impls**: `impl Display` that nothing dispatches through → remove dead code
- **Excessive generics**: 4 type parameters → consider type erasure (trait objects)
- **Unsafe surface area**: unsafe blocks = audit burden = maintenance debt

These are things a Rust developer would catch in code review. pt04 automates that review.

---

#### 17. `/kcore-decomposition-layering-analysis`

**Today**: Entities classified as CORE/MID/PERIPHERAL by degree-based k-core.

**With pt04**: Use TYPED edges for more accurate layering:

```json
{
  "entities": [
    {
      "entity": "rust:trait:StorageBackend:...",
      "coreness": 8,
      "layer": "CORE",
      "typed_coreness": {
        "trait_dispatch_degree": 12,
        "direct_call_degree": 3,
        "is_trait_definition": true,
        "impl_count": 4,
        "reason": "Core trait with 4 implementations and 12 trait-dispatch callers"
      }
    },
    {
      "entity": "rust:fn:format_output:...",
      "coreness": 1,
      "layer": "PERIPHERAL",
      "typed_coreness": {
        "trait_dispatch_degree": 0,
        "direct_call_degree": 2,
        "is_trait_definition": false,
        "impl_count": 0,
        "reason": "Leaf utility with 2 direct callers, no trait involvement"
      }
    }
  ]
}
```

**LLM gains**: A trait definition with 4 implementations is CORE infrastructure — changing it ripples through every impl. A function with 8 callers might have high k-core but they're all in the same module (safe to change). pt04 distinguishes "core because of trait architecture" from "core because of call volume."

---

#### 18. `/centrality-measures-entity-ranking?method=pagerank`

**Today**: PageRank and betweenness centrality on untyped edges.

**With pt04**: Run centrality on TYPED edge subgraphs:

```bash
# Overall PageRank (existing, now uses typed edges for weight)
curl "http://localhost:7777/centrality-measures-entity-ranking?method=pagerank"

# NEW: PageRank on trait-dispatch edges only (architectural importance)
curl "http://localhost:7777/centrality-measures-entity-ranking?method=pagerank&edge_filter=TraitMethod"

# NEW: Betweenness on direct-call edges only (coupling bottlenecks)
curl "http://localhost:7777/centrality-measures-entity-ranking?method=betweenness&edge_filter=Direct"
```

```json
{
  "method": "pagerank",
  "edge_filter": "TraitMethod",
  "entities": [
    {
      "entity": "rust:trait:StorageBackend:...",
      "score": 0.087,
      "rank": 1,
      "insight": "Highest trait-dispatch PageRank — this trait is the most architecturally central abstraction"
    }
  ]
}
```

**LLM gains**: PageRank on ALL edges finds "most called functions." PageRank on TRAIT edges only finds "most architecturally central abstractions." These are different entities with different implications:
- High PageRank on all edges → hot path, performance-critical
- High PageRank on trait edges → central abstraction, architecture-critical
- High betweenness on direct edges → coupling bottleneck, refactoring target

---

#### 19. `/entropy-complexity-measurement-scores`

**Today**: Shannon entropy over edge types (Calls, Uses, Implements — only 3 types, max H = 1.585).

**With pt04**: Entropy over TYPED edge categories (Direct, TraitMethod, DynDispatch, ClosureInvoke, Uses, Implements, Extends, Contains — 8 types, max H = 3.0):

```json
{
  "entities": [
    {
      "entity": "rust:fn:orchestrate:...",
      "entropy": 2.4,
      "max_possible_entropy": 3.0,
      "complexity": "HIGH",
      "edge_distribution": {
        "Direct": 5,
        "TraitMethod": 4,
        "DynDispatch": 2,
        "ClosureInvoke": 3,
        "Uses": 2,
        "Implements": 1
      },
      "interpretation": "Highly diverse interaction patterns — uses 6 different edge types"
    }
  ]
}
```

**LLM gains**: Today, entropy measures "does this entity only Call things, or does it also Use and Implement things?" — a coarse 3-type distinction. With pt04, entropy measures "does this entity use direct calls, trait dispatches, dynamic dispatch, AND closures?" — an 8-type distinction that captures the REAL complexity of interaction patterns.

A function with entropy 2.4/3.0 uses almost every kind of interaction the language supports. That's genuinely complex — not because it has many edges, but because it has many KINDS of edges.

---

#### 20. `/coupling-cohesion-metrics-suite`

**Today**: CBO, LCOM, RFC, WMC based on edge counts.

**With pt04**: Enrich CK metrics with typed edge decomposition:

```json
{
  "entities": [
    {
      "entity": "rust:struct:RequestHandler:...",
      "cbo": 15,
      "cbo_breakdown": {
        "via_trait_dispatch": 8,
        "via_direct_call": 5,
        "via_type_usage": 2
      },
      "lcom": 0.85,
      "lcom_detail": {
        "method_count": 6,
        "shared_trait_dispatches": 1,
        "distinct_trait_targets": 5,
        "interpretation": "6 methods, but they dispatch through 5 different traits — low cohesion confirmed"
      },
      "rfc": 42,
      "rfc_breakdown": {
        "direct_reachable": 25,
        "trait_dispatch_reachable": 17
      },
      "wmc": 23,
      "health_grade": "F",
      "typed_diagnosis": "God Object: 5 distinct trait interfaces consumed. Split by trait responsibility."
    }
  ]
}
```

**LLM gains**: CBO = 15 tells the LLM "high coupling." CBO breakdown `via_trait_dispatch: 8` tells the LLM "most coupling is through architectural boundaries." LCOM detail `distinct_trait_targets: 5` tells the LLM EXACTLY how many concerns this entity mixes.

The `typed_diagnosis` field gives the LLM a pre-computed refactoring suggestion: "Split by trait responsibility." The compiler proved this entity dispatches through 5 unrelated traits — that's 5 responsibilities in one struct.

---

#### 21. `/leiden-community-detection-clusters`

**Today**: Leiden clustering on unweighted, untyped edges. Returns `community_count`, `modularity`, communities with members.

**With pt04**: Use TRAIT DISPATCH as clustering signal:

```json
{
  "community_count": 8,
  "modularity": 0.58,
  "clustering_method": "leiden_with_trait_seeds",
  "communities": [
    {
      "id": 0,
      "size": 45,
      "anchor_traits": ["StorageBackend", "CozoDbOps"],
      "dominant_dispatch_kind": "TraitMethod",
      "label_suggestion": "Storage & Database Layer",
      "internal_cohesion": 0.89,
      "external_coupling": 0.12,
      "members": ["rust:fn:execute_query:...", "rust:fn:insert_entity:...", "..."]
    },
    {
      "id": 1,
      "size": 32,
      "anchor_traits": ["TreeSitterParser", "LanguageSupport"],
      "dominant_dispatch_kind": "TraitMethod",
      "label_suggestion": "Parsing & Language Support",
      "internal_cohesion": 0.91,
      "external_coupling": 0.08,
      "members": ["rust:fn:parse_rust:...", "rust:fn:parse_python:...", "..."]
    }
  ]
}
```

**LLM gains**: Today's Leiden finds communities by edge density. With pt04, Leiden uses TRAIT MEMBERSHIP as a signal: entities dispatching through the same traits cluster together. The result is communities aligned with architectural boundaries (traits), not just call frequency.

The `anchor_traits` field tells the LLM: "This community exists because these entities share the `StorageBackend` and `CozoDbOps` trait interfaces." That's an EXPLANATION of why the community exists, not just a list of members.

**Modularity improvement**: Trait-seeded Leiden typically achieves 0.58 modularity vs 0.42 without seeds (in the UserJourney test). Higher modularity = cleaner community boundaries = more accurate architecture recovery.

---

### STEP 5 — CONTEXT FOR LLM

#### 22. `/smart-context-token-budget?focus=X&tokens=N`

**Today**: Selects related entities within a token budget based on graph distance.

**With pt04**: Prioritize by SEMANTIC relevance, not just graph distance:

```json
{
  "focus_entity": "rust:fn:authenticate:...",
  "selection_strategy": "semantic_relevance",
  "included_entities": [
    {
      "entity": "rust:trait:AuthService:...",
      "relevance": "TRAIT_DEFINITION",
      "reason": "Focus entity implements this trait",
      "tokens": 120
    },
    {
      "entity": "rust:fn:verify_token:...",
      "relevance": "TRAIT_SIBLING",
      "reason": "Also implements AuthService — same contract",
      "tokens": 80
    },
    {
      "entity": "rust:struct:AuthError:...",
      "relevance": "ERROR_TYPE",
      "reason": "Return type error variant",
      "tokens": 45
    },
    {
      "entity": "rust:fn:handle_request:...",
      "relevance": "TRAIT_DISPATCH_CALLER",
      "reason": "Calls focus entity through trait dispatch",
      "tokens": 150
    }
  ],
  "total_tokens_used": 395,
  "token_budget": 500,
  "entities_included": 4,
  "excluded_low_relevance": [
    {"entity": "rust:fn:log_access:...", "reason": "Direct utility call — low architectural relevance"}
  ]
}
```

**LLM gains**: Today, smart context includes the N closest entities by graph hops. With pt04, it prioritizes:
1. **TRAIT_DEFINITION**: The trait this entity implements (the contract)
2. **TRAIT_SIBLING**: Other implementations of the same trait (alternative behaviors)
3. **ERROR_TYPE**: The error types in the return signature (error handling context)
4. **TRAIT_DISPATCH_CALLER**: Entities that call through the trait (the consumers)

These are semantically IMPORTANT entities. A direct-call utility like `log_access` is nearby in the graph but architecturally irrelevant. pt04 knows the difference.

**Token efficiency**: With semantic prioritization, the LLM gets the ARCHITECTURAL context (trait definition + siblings + callers) instead of the INCIDENTAL context (nearby utility functions). Same token budget, much higher reasoning value.

---

### STEP 6 — DIAGNOSTICS Endpoints

#### 23. `/file-watcher-status-check`

**Today**: File watcher status (active, events processed).

**With pt04**: Add semantic sync status:

```json
{
  "file_watcher_active": true,
  "events_processed": 142,
  "semantic_sync": {
    "last_ra_analysis": "2026-02-15T10:30:00Z",
    "pending_semantic_updates": 0,
    "ra_analysis_lag_ms": 0,
    "stale_typed_edges": false
  }
}
```

**LLM gains**: The LLM knows whether typed edges are stale. If `pending_semantic_updates > 0`, the LLM knows to use tree-sitter edges (current) rather than typed edges (potentially stale). Semantic analysis is slower than tree-sitter re-parse, so there can be a lag.

---

#### 24. `POST /incremental-reindex-file-update?path=X`

**Today**: Reindexes one file (tree-sitter re-parse).

**With pt04**: Trigger both tree-sitter re-parse AND rust-analyzer re-analysis:

```json
{
  "file": "src/auth.rs",
  "tree_sitter_reindex": {
    "entities_updated": 5,
    "edges_updated": 12,
    "time_ms": 7
  },
  "semantic_reanalysis": {
    "typed_edges_updated": 8,
    "trait_impls_changed": 1,
    "new_async_functions": 0,
    "time_ms": 150,
    "affected_crates": ["parseltongue-core"]
  }
}
```

**LLM gains**: The LLM sees that editing `auth.rs` updated 8 typed edges and changed 1 trait impl. If the trait impl changed, downstream crates may need reanalysis. The LLM can proactively warn: "You changed the `AuthService` trait impl — this affects `pt08-http-code-query-server`."

---

#### 25. `/ingestion-coverage-folder-report?depth=N`

**Today**: Per-folder parse coverage (total, eligible, parsed files).

**With pt04**: Add semantic coverage depth:

```json
{
  "folders": [
    {
      "folder_path": "crates/parseltongue-core/",
      "total_files": 62,
      "eligible_files": 62,
      "parsed_files": 62,
      "coverage_pct": 100.0,
      "semantic_coverage": {
        "files_with_typed_edges": 58,
        "files_without_typed_edges": 4,
        "semantic_coverage_pct": 93.5,
        "reason_for_gaps": "4 files are proc-macro definitions — rust-analyzer can't resolve inside proc macros"
      }
    }
  ]
}
```

**LLM gains**: Tree-sitter parsed 100% of files, but rust-analyzer only resolved types in 93.5%. The LLM knows that 4 proc-macro files lack typed edges — if a bug is in proc-macro code, the LLM should rely on source reading, not typed analysis.

---

#### 26. `/ingestion-diagnostics-coverage-report`

**Today**: Word coverage, test exclusions, ignored files.

**With pt04**: Add semantic analysis diagnostics:

```json
{
  "summary": {
    "total_entities": 755,
    "typed_entities": 680,
    "untyped_entities": 75,
    "typing_coverage_pct": 90.1,
    "unresolvable_reasons": {
      "proc_macros": 30,
      "cfg_gated_code": 15,
      "build_rs_generated": 10,
      "external_crate_boundary": 20
    }
  }
}
```

**LLM gains**: "90.1% of entities have compiler-resolved types. The 9.9% that don't are proc macros (30), cfg-gated (15), build.rs-generated (10), and cross-crate boundary (20)." The LLM knows exactly where to trust typed analysis and where to fall back to source reading.

---

## PART III: Every Workflow Pattern — How pt04 Enriches It

### README Workflow 1: New Codebase Orientation (5 queries)

**Today**:
```bash
curl .../codebase-statistics-overview-summary    # Scale
curl .../circular-dependency-detection-scan      # Health
curl ".../complexity-hotspots-ranking-view?top=10" # Hotspots
curl .../semantic-cluster-grouping-list          # Modules
curl ".../ingestion-coverage-folder-report?depth=2" # Coverage
```

**After these 5 queries, the LLM knows**: size, health, risk areas, module boundaries, parse confidence.

**With pt04 (same 5 queries, richer answers)**:

After these 5 queries, the LLM NOW knows:
1. **Scale**: 755 entities, 3200 typed edges, 47 async functions, 12 unsafe blocks, 89 generics
2. **Health**: 3 cycles — 1 intentional (Observer pattern via trait dispatch), 1 violation (direct call cycle), 1 ambiguous
3. **Hotspots**: Top entity dispatches through 5 different traits (God Object confirmed by compiler)
4. **Modules**: 8 communities anchored by traits like StorageBackend, TreeSitterParser — not just file paths
5. **Coverage**: 100% tree-sitter, 93.5% semantic (4 proc-macro files lack typed edges)

**The LLM's orientation is now ARCHITECTURAL, not just statistical.** It doesn't just know "755 entities." It knows "755 entities, 47 of which are async, organized around 8 trait-based communities, with 1 genuine coupling violation to fix."

---

### README Workflow 2: Bug Hunting (trace backward from symptom)

**Today**:
```bash
curl ".../code-entities-search-fuzzy?q=FUNCTION_NAME"  # Find
curl ".../code-entity-detail-view?key=KEY"              # Read
curl ".../reverse-callers-query-graph?entity=KEY"       # Callers
curl ".../blast-radius-impact-analysis?entity=KEY&hops=2" # Depth
```

**With pt04**:

```bash
# 1. Find — but now search by TYPE, not just name
curl ".../code-entities-search-fuzzy?returns=AuthError"
# → Finds ALL functions that can produce AuthError, not just ones named "auth"

# 2. Read — but now with compiler-resolved context
curl ".../code-entity-detail-view?key=KEY"
# → Shows: returns Result<User, AuthError>, is async, dispatches through 2 traits

# 3. Callers — but now classified by dispatch kind
curl ".../reverse-callers-query-graph?entity=KEY"
# → 5 callers: 3 via TraitMethod (safe boundary), 2 Direct (potential bypass)

# 4. Blast radius — but now TYPED
curl ".../blast-radius-impact-analysis?entity=KEY&hops=2"
# → 42 affected via trait dispatch (contained), 20 via direct call (risky)
```

**New pt04-only bug hunting step**:
```bash
# 5. NEW: Trace the async chain
curl ".../forward-callees-query-graph?entity=KEY&is_async=true"
# → Shows the async call chain — find where .await yields, where the bug might be a race condition

# 6. NEW: Check closure captures at the call site
curl ".../dependency-edges-list-all?from=KEY&call_kind=ClosureInvoke"
# → Shows closures created by this function and what they capture — MutableRef captures in async = likely bug
```

**LLM gain for bug hunting**: The LLM can trace "which callers bypass the trait boundary" (potential source of misuse), "which closures capture mutable references in async code" (potential race conditions), and "which functions return this error type" (root cause candidates). None of these are possible with tree-sitter alone.

---

### README Workflow 3: Safe Refactoring (quantify risk before changing)

**Today**:
```bash
curl ".../reverse-callers-query-graph?entity=KEY"       # Direct callers
curl ".../blast-radius-impact-analysis?entity=KEY&hops=3" # Transitive
curl .../circular-dependency-detection-scan              # Cycle check
curl ".../smart-context-token-budget?focus=KEY&tokens=8000" # Context
```

**With pt04**:

The refactoring risk assessment becomes PRECISE:

```bash
# 1. Callers — classified by dispatch kind
curl ".../reverse-callers-query-graph?entity=KEY"
# → "3 trait-dispatch callers, 2 direct callers"
# → Refactoring the function body: safe (trait callers don't care)
# → Refactoring the signature: risky (must update trait definition + all impls)

# 2. Blast radius — TYPED
curl ".../blast-radius-impact-analysis?entity=KEY&hops=3"
# → "can_change_internally: true, can_change_signature: false"
# → "2 external crates affected via trait dispatch"

# 3. Cycle check — auto-classified
curl .../circular-dependency-detection-scan
# → "Entity is in an INTENTIONAL_PATTERN cycle (Observer via trait dispatch)"
# → "Safe to refactor impl, but don't break the trait contract"

# 4. Smart context — semantic relevance
curl ".../smart-context-token-budget?focus=KEY&tokens=8000"
# → Includes trait definition, sibling impls, error types — not random nearby utilities
```

**New pt04-only refactoring step**:
```bash
# 5. NEW: Visibility check — can we reduce API surface?
curl ".../code-entities-search-fuzzy?q=KEY_MODULE&visibility=pub"
# → "12 pub items in this module, 8 have 0 external callers"
# → "8 items can safely become pub(crate) — reduces blast radius"

# 6. NEW: Trait compatibility check
# If adding a new method to a trait, check how many types implement it
curl ".../dependency-edges-list-all?edge_type=Implements&to_key=TRAIT_KEY"
# → "4 types implement this trait — all 4 need the new method"
```

**LLM gain for refactoring**: The safety assessment goes from "62 things affected" (today's number) to "you can safely change the function body (0 breaking callers), but adding a parameter requires updating the trait definition and all 4 implementations across 2 crates" (pt04's typed analysis). That's the difference between "maybe safe" and "here's exactly what needs to change."

---

### README Workflow 4: Architecture Review (7 queries)

**Today**: 7 graph analysis endpoints in parallel (SCC, SQALE, K-Core, PageRank, Entropy, CK, Leiden).

**With pt04**: Same 7 queries, but each response now includes typed decompositions. The architecture review summary becomes:

```
ARCHITECTURE REVIEW — Parseltongue Codebase
============================================

SCCs:        3 cycles found
             1 INTENTIONAL (Observer trait pattern)
             1 VIOLATION (direct call cycle in parser module)
             1 AMBIGUOUS (needs human review)

SQALE:       42 total debt hours
             NEW: 8 visibility bloat violations (pub items with 0 external callers)
             NEW: 3 padding waste violations (>25% padding in hot-path structs)
             NEW: 2 dead trait impls (Display/Debug impls nobody dispatches through)

K-Core:      Max coreness = 8
             Core trait: StorageBackend (12 trait-dispatch callers)
             NOTE: High coreness due to TRAIT ARCHITECTURE, not just call volume

PageRank:    Top entity: StorageBackend trait (architectural PageRank)
             NOTE: Different from call-volume PageRank (where execute_query leads)

Entropy:     3 entities with H > 2.0 (use 6+ edge types — genuinely complex)
             NOTE: Entropy now over 8 edge types (was 3), much more discriminating

CK Metrics:  2 entities grade F
             Both have unique_traits_consumed >= 5 (God Objects confirmed by compiler)
             typed_diagnosis: "Split by trait responsibility"

Leiden:      8 communities, modularity 0.58
             Trait-anchored: StorageBackend community, TreeSitterParser community
             NOTE: 0.58 modularity vs 0.42 without trait seeds — cleaner boundaries
```

**LLM gain**: The LLM produces an architecture review with COMPILER-VERIFIED claims, not statistical inferences. "This is a God Object" is backed by "5 distinct trait interfaces consumed" — not just "high CBO." "This cycle is intentional" is backed by "all edges are trait method dispatches" — not just "looks like Observer pattern."

---

### UserJourney Pattern 1: LLM-Powered Code Review

**Today**:
```bash
ENTITY=$(curl -s .../code-entities-search-fuzzy?q=authenticate | jq -r '.data.entities[0].key')
curl -s ".../blast-radius-impact-analysis?entity=$ENTITY&hops=2"
curl -s ".../reverse-callers-query-graph?entity=$ENTITY"
curl -s ".../smart-context-token-budget?focus=$ENTITY&tokens=2000"
```

**With pt04**: The code review prompt to the LLM becomes:

```
Review this function:
- Returns Result<User, AuthError>, is async, visibility pub(crate)
- Implements AuthService trait
- 3 callers via trait dispatch (safe boundary), 2 direct callers (potential bypass)
- Dispatches through 2 traits (AuthService, Logger) — acceptable responsibility count
- 1 closure captures db: MutableRef — check for async safety
- Blast radius: 42 via trait (contained), 20 via direct (risky)
- Smart context includes: AuthService trait definition, 2 sibling impls, AuthError enum
```

**vs today's prompt**:

```
Review this function:
- Source code: [500 lines of raw code]
- 5 callers
- Blast radius: 62 affected
- Smart context: [8 nearby entities by graph distance]
```

**The LLM gets ANALYSIS to reason about, not CODE to read.** The review is faster, more focused, and catches things like "closure captures mutable reference in async context" that would require expert-level Rust knowledge to spot in raw source.

---

### UserJourney Pattern 2: Pre-Refactoring Safety Check

**Today**: Check cycles, check hotspots, check blast radius.

**With pt04**: Add these NEW pre-refactoring checks:

```bash
# NEW: Trait contract check — does refactoring break any trait?
curl ".../dependency-edges-list-all?from_key=$TARGET&call_kind=TraitMethod"
# → Lists all traits this entity participates in — changing the entity may break the contract

# NEW: Visibility audit — can we tighten before refactoring?
curl ".../code-entities-search-fuzzy?q=MODULE_NAME&visibility=pub"
# → "8 pub items could be pub(crate)" — tighten FIRST, then refactor with smaller blast radius

# NEW: Generic bound check — does this entity participate in generic constraints?
curl ".../code-entity-detail-view?key=$TARGET"
# → Shows generic params and their bounds — refactoring may invalidate bounds on downstream generics
```

**LLM gain**: The pre-refactoring checklist goes from "is it safe?" to "here's exactly what would break and what to tighten first." The LLM can suggest: "Before refactoring, change these 8 pub items to pub(crate). That reduces blast radius from 62 to 34. Then proceed."

---

### UserJourney Pattern 3: Real-Time File Watching

**Today**: Edit a file → graph updates in ~7ms.

**With pt04**: Edit a file → tree-sitter updates in ~7ms → rust-analyzer re-analysis in ~150ms. Two-phase update:

```
Phase 1 (7ms):   Tree-sitter entities + syntactic edges updated → queries work immediately
Phase 2 (150ms): Typed edges + trait impls updated → semantic queries accurate

Between Phase 1 and Phase 2: Typed edges may be stale.
Solution: /file-watcher-status-check shows pending_semantic_updates count.
```

**LLM gain**: The LLM can query immediately after a file save (Phase 1 is fast enough). If it needs typed analysis, it checks `pending_semantic_updates` first. No wasted queries on stale data.

---

### UserJourney Pattern 4: Progressive Root Cause Analysis

**Today**:
```bash
curl .../code-entities-search-fuzzy?q=login_handler       # Find
curl .../centrality-measures-entity-ranking?method=betweenness  # Bottleneck?
curl ".../coupling-cohesion-metrics-suite?entity=$ENTITY"  # God Object?
curl ".../technical-debt-sqale-scoring?entity=$ENTITY"     # Debt?
curl .../leiden-community-detection-clusters               # Community?
# LLM synthesizes
```

**With pt04**:

```bash
# 1. Find — and immediately see async/unsafe/visibility
curl .../code-entities-search-fuzzy?q=login_handler
# → is_async: true, visibility: pub, unique_traits_consumed: 4

# 2. Bottleneck check — on TRAIT edges specifically
curl ".../centrality-measures-entity-ranking?method=betweenness&edge_filter=TraitMethod"
# → Is it a trait-dispatch bottleneck (architectural) or just a call-volume bottleneck?

# 3. CK metrics — with typed diagnosis
curl ".../coupling-cohesion-metrics-suite?entity=$ENTITY"
# → typed_diagnosis: "4 distinct trait interfaces consumed — Split by trait responsibility"

# 4. SQALE — with compiler-detected debt
curl ".../technical-debt-sqale-scoring?entity=$ENTITY"
# → Includes: visibility_bloat, padding_waste, unsafe_surface, dead_trait_impls

# 5. Community — with trait anchors
curl .../leiden-community-detection-clusters
# → login_handler is in the "AuthService + SessionManager" community

# 6. LLM synthesizes with EVIDENCE:
# "login_handler is a God Object (4 traits, compiler-verified).
#  Split into: AuthHandler (AuthService), SessionHandler (SessionManager),
#  AuditHandler (AuditLogger), RateLimitHandler (RateLimiter).
#  8 pub methods could become pub(crate) — tighten first to reduce blast radius."
```

---

### UserJourney Pattern 5: Architecture Health Dashboard

**Today**: Run all 7 graph analysis endpoints in parallel.

**With pt04**: Same 7 endpoints, but the dashboard now reports typed metrics:

```
ARCHITECTURE HEALTH DASHBOARD
==============================

Cycles:       3 found (1 intentional, 1 violation, 1 ambiguous)
Tech Debt:    42 hours (8 visibility bloat, 3 padding waste, 2 dead impls)
Core Layer:   StorageBackend trait (12 trait-dispatch callers)
Top Entity:   StorageBackend (trait-dispatch PageRank = 0.087)
Complexity:   3 entities with H > 2.0 (6+ interaction types)
Coupling:     2 God Objects (5+ traits consumed, compiler-verified)
Communities:  8 (trait-anchored, modularity 0.58)

TYPED HEALTH SCORE: B+
  Deductions:
    -1: 1 direct-call cycle (violation)
    -1: 2 God Objects
    -0.5: 8 unnecessary pub items
    -0.5: 3 padding waste structs
  Bonuses:
    +1: 0 unsafe code in hot paths
    +0.5: 93.5% semantic coverage
    +0.5: 0.58 modularity (above 0.5 threshold)
```

---

## PART IV: pt04-Only Capabilities — Endpoints That Don't Exist Today

These are capabilities that ONLY rust-analyzer can provide — tree-sitter can't extract this information at any confidence level.

### NEW Endpoint: `/trait-hierarchy-graph-view`

```bash
curl "http://localhost:7777/trait-hierarchy-graph-view?trait=Handler"
```

```json
{
  "root_trait": "Handler",
  "supertraits": ["Send", "Sync"],
  "subtrait_tree": {
    "Handler": {
      "children": ["AuthHandler", "CacheHandler", "LogHandler"],
      "methods": ["handle", "on_error"],
      "implementors": ["DefaultHandler", "MockHandler"]
    },
    "AuthHandler": {
      "children": [],
      "methods": ["authenticate", "authorize"],
      "implementors": ["JwtAuthHandler", "OAuthHandler", "BasicAuthHandler"]
    }
  },
  "blanket_impls": ["impl<T: Handler + Send> Service for T"],
  "total_implementors": 5,
  "total_trait_methods": 4
}
```

**Why this matters for LLM companionship**: The LLM sees the TRAIT HIERARCHY — which is the Rust equivalent of an OOP class hierarchy but more powerful (composition over inheritance). When the LLM suggests "add a new handler," it knows the trait contract (2 required methods), the existing implementations (5), and the supertrait bounds (must be Send + Sync). Today, the LLM would have to read every file to piece this together.

---

### NEW Endpoint: `/async-call-chain-trace?entity=X`

```bash
curl "http://localhost:7777/async-call-chain-trace?entity=rust:fn:handle_request"
```

```json
{
  "root": "handle_request",
  "async_chain": [
    {"fn": "handle_request", "awaits": ["authenticate", "process_body", "send_response"]},
    {"fn": "authenticate", "awaits": ["verify_token", "lookup_user"]},
    {"fn": "verify_token", "awaits": ["fetch_signing_key"]},
    {"fn": "fetch_signing_key", "awaits": ["http_get"]}
  ],
  "max_await_depth": 4,
  "spawn_points": [
    {"fn": "process_body", "spawns": "tokio::spawn(process_chunk)", "captures": ["body: Move", "db: SharedRef"]}
  ],
  "potential_issues": [
    {
      "kind": "MUTABLE_CAPTURE_IN_SPAWN",
      "location": "process_body spawns task capturing db as SharedRef",
      "risk": "If db is &mut elsewhere in the chain, this is a data race",
      "severity": "WARNING"
    }
  ]
}
```

**Why this matters**: Async Rust bugs are notoriously hard to find. The LLM can trace the full async call chain, see every `.await` point, every `tokio::spawn`, and every closure capture. A mutable reference captured in a spawned task while the parent holds another reference? That's a race condition the LLM can flag BEFORE it becomes a runtime panic.

Tree-sitter can see `async fn` keyword but cannot resolve the full async chain or detect cross-task capture conflicts.

---

### NEW Endpoint: `/visibility-audit-report`

```bash
curl "http://localhost:7777/visibility-audit-report"
```

```json
{
  "summary": {
    "total_pub_items": 210,
    "externally_used_pub_items": 145,
    "internally_only_pub_items": 65,
    "tightenable_to_pub_crate": 50,
    "tightenable_to_private": 15,
    "api_surface_reduction_pct": 31.0
  },
  "items": [
    {
      "entity": "rust:fn:internal_helper:...",
      "current_visibility": "pub",
      "recommended_visibility": "pub(crate)",
      "reason": "Only called from within the same crate (3 internal callers, 0 external)",
      "blast_radius_reduction": "62 → 34 if tightened"
    }
  ]
}
```

**Why this matters**: Visibility bloat is silent tech debt. Every `pub` item is a promise to external consumers. pt04 can prove that 65 out of 210 `pub` items have ZERO external callers — they can safely be tightened. This reduces blast radius for every function that uses them, making all refactoring safer.

---

### NEW Endpoint: `/generic-instantiation-map?entity=X`

```bash
curl "http://localhost:7777/generic-instantiation-map?entity=rust:fn:process_batch"
```

```json
{
  "entity": "rust:fn:process_batch<T: Serialize + Send>",
  "generic_params": [{"name": "T", "bounds": ["Serialize", "Send"]}],
  "known_instantiations": [
    {"T": "Entity", "call_site": "rust:fn:export_entities:...:42"},
    {"T": "Edge", "call_site": "rust:fn:export_edges:...:55"},
    {"T": "AnalysisResult", "call_site": "rust:fn:export_results:...:78"}
  ],
  "instantiation_count": 3,
  "monomorphization_cost": "3 copies in binary",
  "implied_constraints": "All 3 types are Serialize + Send — no dynamic dispatch needed"
}
```

**Why this matters**: Generic functions create multiple copies in the binary (monomorphization). The LLM sees: "process_batch is instantiated 3 times — for Entity, Edge, and AnalysisResult." If the LLM is optimizing binary size, it can suggest: "Convert to `dyn Serialize + Send` for one dynamic-dispatch copy instead of 3 monomorphized copies."

---

### NEW Endpoint: `/unsafe-audit-report`

```bash
curl "http://localhost:7777/unsafe-audit-report"
```

```json
{
  "total_unsafe_blocks": 12,
  "unsafe_functions": 3,
  "unsafe_trait_impls": 1,
  "audit": [
    {
      "entity": "rust:fn:raw_pointer_cast:...",
      "unsafe_kind": "UNSAFE_BLOCK",
      "operations": ["ptr::read", "ptr::write"],
      "callers": 4,
      "blast_radius": 15,
      "safety_invariant_documented": false,
      "risk_level": "HIGH",
      "recommendation": "Document safety invariant or use safe alternative (e.g., slice::from_raw_parts)"
    }
  ]
}
```

**Why this matters**: Every `unsafe` block is a potential soundness hole. The LLM sees ALL unsafe code in the codebase with their call chains. Combined with blast radius, the LLM can prioritize: "This unsafe block has 4 callers and a blast radius of 15. It uses raw pointer operations. No safety invariant is documented. This is the highest-risk code in the codebase."

---

### NEW Endpoint: `/type-size-layout-analysis`

```bash
curl "http://localhost:7777/type-size-layout-analysis"
```

```json
{
  "types": [
    {
      "entity": "rust:struct:Entity:...",
      "size_bytes": 128,
      "alignment": 8,
      "padding_bytes": 24,
      "padding_pct": 18.75,
      "fields": [
        {"name": "key", "type": "String", "size": 24, "offset": 0},
        {"name": "is_active", "type": "bool", "size": 1, "offset": 24},
        {"name": "data", "type": "Vec<u8>", "size": 24, "offset": 32},
        {"name": "timestamp", "type": "u64", "size": 8, "offset": 56}
      ],
      "optimal_field_order": ["key", "data", "timestamp", "is_active"],
      "optimal_size_bytes": 112,
      "savings_bytes": 16,
      "hot_path": true,
      "recommendation": "Reorder fields to save 16 bytes per instance (12.5% reduction)"
    }
  ]
}
```

**Why this matters for high-performance codebases**: Struct padding is invisible in source code but costs real memory. If Entity is stored 100K times, that's 1.6MB of wasted padding. pt04 shows the exact field ordering to minimize padding. This is optimization the compiler knows but doesn't tell you.

---

### NEW Endpoint: `/closure-capture-analysis`

```bash
curl "http://localhost:7777/closure-capture-analysis"
```

```json
{
  "closures": [
    {
      "defined_in": "rust:fn:process_items:...:35",
      "captures": [
        {"variable": "db", "capture_kind": "MutableRef", "type": "&mut Database"},
        {"variable": "config", "capture_kind": "SharedRef", "type": "&Config"},
        {"variable": "counter", "capture_kind": "Move", "type": "u32"}
      ],
      "is_async": true,
      "is_send": false,
      "send_blocker": "db is &mut Database which is not Send",
      "risk_level": "HIGH",
      "explanation": "Async closure captures &mut Database — cannot be sent across threads"
    }
  ],
  "summary": {
    "total_closures": 156,
    "closures_with_mutable_captures": 23,
    "async_closures_not_send": 5,
    "potential_issues": 5
  }
}
```

**Why this matters**: Closures that capture `&mut` references in async contexts are a leading source of Rust compilation errors and potential bugs. The LLM sees all 5 problematic closures at once and can suggest fixes: "Convert &mut Database to Arc<Mutex<Database>> to make this closure Send."

---

## PART V: pt04-Only Workflows — New LLM Companionship Patterns

These workflows are IMPOSSIBLE with tree-sitter alone. They require compiler semantics.

### Workflow 6: Trait-Driven Architecture Discovery

```bash
# 1. What traits define the architecture?
curl "http://localhost:7777/centrality-measures-entity-ranking?method=pagerank&edge_filter=TraitMethod"

# 2. What's the trait hierarchy?
curl "http://localhost:7777/trait-hierarchy-graph-view?trait=StorageBackend"

# 3. Who implements the core traits?
curl "http://localhost:7777/dependency-edges-list-all?edge_type=Implements&to_key=TRAIT_KEY"

# 4. Are there missing implementations?
# (If a trait has 4 impls but one module doesn't provide one, it's likely missing)

# 5. LLM synthesizes: "The architecture is organized around 3 core traits:
#    StorageBackend (4 impls), Parser (12 impls), Handler (5 impls).
#    The Parser trait hierarchy has a missing impl for Swift — likely a TODO."
```

**This workflow doesn't exist today** because tree-sitter can't resolve trait hierarchies or list implementations.

---

### Workflow 7: Async Safety Audit

```bash
# 1. Find all async functions
curl "http://localhost:7777/code-entities-list-all?is_async=true"

# 2. Trace async call chains from entry points
curl "http://localhost:7777/async-call-chain-trace?entity=rust:fn:main"

# 3. Find problematic closure captures in async code
curl "http://localhost:7777/closure-capture-analysis"

# 4. Check for Send/Sync violations
# (closures capturing non-Send types in spawned tasks)

# 5. LLM produces safety report:
#    "5 async closures capture non-Send types. 2 are in spawned tasks (data race risk).
#     Recommendation: Wrap db references in Arc<Mutex<>> at these call sites."
```

---

### Workflow 8: API Surface Minimization

```bash
# 1. Audit current visibility
curl "http://localhost:7777/visibility-audit-report"

# 2. For each unnecessary pub item, check what would break
curl "http://localhost:7777/blast-radius-impact-analysis?entity=ITEM&hops=1"

# 3. Tighten visibility
# LLM produces a PR checklist:
#    "50 items can become pub(crate), 15 can become private.
#     This reduces total API surface by 31%.
#     No external consumers affected (verified by compiler).
#     Blast radius of remaining pub items decreases by average 22%."
```

---

### Workflow 9: Performance Optimization via Type Layout

```bash
# 1. Find hot-path structs (high PageRank + large size)
curl "http://localhost:7777/centrality-measures-entity-ranking?method=pagerank"
curl "http://localhost:7777/type-size-layout-analysis"

# 2. Cross-reference: high PageRank + high padding = optimization target
# LLM: "Entity struct is the #1 PageRank entity AND has 18.75% padding.
#        Reordering fields saves 16 bytes per instance.
#        At 100K entities, that's 1.6MB savings."

# 3. Check for excessive monomorphization
curl "http://localhost:7777/generic-instantiation-map?entity=rust:fn:process_batch"
# LLM: "process_batch has 3 monomorphized copies. Consider trait objects to reduce binary size."
```

---

### Workflow 10: Pre-Merge Safety Gate (CI/CD Integration)

```bash
# Run these checks on every PR:

# 1. Did blast radius increase beyond threshold?
curl ".../blast-radius-impact-analysis?entity=CHANGED_ENTITY&hops=2"
# Gate: total_affected < 100

# 2. Did any new cycles appear?
curl ".../circular-dependency-detection-scan"
# Gate: No new VIOLATION cycles (INTENTIONAL_PATTERN ok)

# 3. Did visibility get looser?
curl ".../visibility-audit-report"
# Gate: internally_only_pub_items didn't increase

# 4. Did unsafe surface area grow?
curl ".../unsafe-audit-report"
# Gate: total_unsafe_blocks didn't increase without safety documentation

# 5. Did God Object metrics worsen?
curl ".../coupling-cohesion-metrics-suite?entity=CHANGED_ENTITY"
# Gate: unique_traits_consumed didn't increase beyond 4
```

**This is CI/CD-grade code quality gating** backed by compiler truth, not linter heuristics.

---

## The Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    LLM JUDGMENT LAYER                    │
│                                                         │
│  Business classification: "Is this revenue-critical?"   │
│  Design intent: "Is this cycle intentional?"            │
│  Naming: "What should we call this module?"             │
│  Strategy: "How should we refactor this?"               │
│                                                         │
│  INPUT: compiler evidence + graph metrics               │
│  OUTPUT: labels, classifications, recommendations       │
│  COST: 3-5 LLM calls per analysis (was 50-200)         │
│                                                         │
├─────────────────────────────────────────────────────────┤
│                 pt04 COMPILER TRUTH LAYER                │
│                                                         │
│  Types: fully resolved, no guessing                     │
│  Traits: hierarchy, impls, blanket impls                │
│  Calls: Direct vs TraitMethod vs DynDispatch            │
│  Closures: captures with kinds (SharedRef/Move)         │
│  Visibility: pub/pub(crate)/pub(super)/private          │
│  Layout: size, alignment, padding                       │
│  Async: call chains, spawn points, Send/Sync analysis   │
│  Unsafe: block locations, operations, blast radius      │
│  Generics: bounds, instantiation sites, mono cost       │
│                                                         │
│  INPUT: Cargo workspace (one-time ingestion)            │
│  OUTPUT: CozoDB relations (TypedCallEdges, TraitImpls)  │
│  COST: 1-3 min at ingestion, 0s at query time           │
│                                                         │
├─────────────────────────────────────────────────────────┤
│              CPU GRAPH ALGORITHM LAYER                   │
│                                                         │
│  Leiden clustering (with trait seeds, not keyword seeds) │
│  Tarjan SCC (with edge-type classification)             │
│  SQALE scoring (with compiler-detected debt signals)    │
│  McCabe complexity (with trait-dispatch counting)        │
│  PageRank/Betweenness (with typed edge subgraph filter) │
│  Shannon entropy (over 8 edge types, not 3)             │
│  K-Core (with trait-dispatch degree weighting)           │
│  CK Metrics (with typed coupling decomposition)         │
│                                                         │
│  INPUT: CozoDB graph (syntax + semantic layers)         │
│  OUTPUT: metrics, clusters, rankings                    │
│  COST: 0.1-1s per query                                 │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## What pt04 Replaces vs What It Doesn't

| Question | Before (LLM guesses) | After (pt04 knows) | Still needs LLM? |
|---|---|---|---|
| What type does `authenticate` return? | LLM reads source: "probably Result<User, Error>" | `Result<User, AuthError>` (exact) | No |
| Is this a trait dispatch or direct call? | LLM infers from naming conventions | `TraitMethod via AuthService` (exact) | No |
| Which trait hierarchy connects A and B? | LLM guesses from impl blocks it can see | `A: Handler -> Service -> Debug + Send` (exact) | No |
| What does this closure capture? | LLM usually can't see this at all | `[db: &mut Conn (MutableRef), config: Arc<Config> (SharedRef)]` | No |
| Is this function async? | LLM checks for `async fn` keyword | `is_async: true` (handles desugared cases too) | No |
| How many distinct traits does X consume? | LLM reads all callees manually | `unique_traits_consumed: 5` (exact count) | No |
| Is this closure Send? | LLM almost never gets this right | `is_send: false, blocker: &mut Database` | No |
| How much padding does this struct have? | LLM can't compute this | `padding_bytes: 24, waste: 18.75%` | No |
| How many monomorphizations exist? | LLM can't track this | `3 instantiations: Entity, Edge, AnalysisResult` | No |
| What's the effective visibility? | LLM sees `pub(crate)`, misses re-exports | `pub (re-exported at crate root)` | No |
| Is this code revenue-critical? | N/A — compiler can't know this | N/A | **Yes** |
| Is this cycle a design pattern? | Compiler provides evidence (dispatch types) | LLM applies judgment | **Sometimes** |
| What should we name this module? | N/A | N/A | **Yes** |
| Should we prioritize fixing this? | N/A | N/A | **Yes** |
| How should we explain this to a developer? | N/A | N/A | **Yes** |

**Pattern**: pt04 handles WHAT IS. LLM handles WHAT IT MEANS and WHAT TO DO ABOUT IT.

---

## Shreyas Doshi Critique: Am I Over-Engineering This?

Let me apply LNO to my own ideas.

### LEVERAGE: What ships first and delivers 80% of the value?

**TypedCallEdges. One CozoDB relation.**

```
TypedCallEdges {
    from_key: String,
    to_key: String =>
    call_kind: String,      # Direct, TraitMethod, DynDispatch, ClosureInvoke
    via_trait: String?,      # Which trait, if any
    receiver_type: String?,  # The concrete receiver type
}
```

This single relation unlocks:
- ALL 26 existing endpoints get richer responses (PART II above)
- Cycle classification (Workflow 2)
- Module boundaries via trait seeds (Workflow 1)
- Complexity analysis via trait counting (Workflow 3)
- Refactoring evidence (Workflow 5)
- Typed PageRank/betweenness (Workflow 4)
- Typed entropy (8 edge types vs 3)
- Forward/reverse caller classification
- Blast radius typing

**One relation. ALL existing endpoints improved. That's leverage.**

### NEUTRAL: What's helpful but not critical?

**TraitImpls + SupertraitEdges.** Phase 2.
- Enables: `/trait-hierarchy-graph-view` (new endpoint)
- Improves: cycle classification (supertrait pattern matching)
- Unlocks: "which types implement trait X?" queries

**SemanticTypes (return types, params, visibility, async/unsafe).** Phase 3.
- Enables: `/visibility-audit-report`, `/unsafe-audit-report` (new endpoints)
- Enables: type-based search (`?returns=Result<User>`)
- Improves: every entity response with resolved metadata

### OVERHEAD: What's research theater?

**Building ALL endpoints in PART IV before proving Phase 1 works.** Classic over-engineering. Ship TypedCallEdges, enrich the existing 26 endpoints, see if LLM companionship actually improves, THEN build new endpoints.

**TypeLayouts.** Cool for performance optimization workflows but only matters for codebases doing systems programming. Not a general-purpose feature. Build if someone asks.

**ClosureCaptures as a separate endpoint.** The data should be IN TypedCallEdges (`call_kind: "ClosureInvoke"` with capture metadata). Not a standalone endpoint.

**Generic instantiation maps.** Niche. Build after everything else.

### The Honest Ship Order

```
Phase 1: TypedCallEdges ONLY
  - One CozoDB relation
  - All 26 existing endpoints get semantic.* fields
  - Maybe 300 lines of new Rust
  - MEASURE: Does Leiden modularity improve? Do LLM code reviews catch more issues?

Phase 2: TraitImpls + SupertraitEdges (if Phase 1 proves valuable)
  - Two more CozoDB relations
  - /trait-hierarchy-graph-view endpoint
  - Better cycle classification
  - Maybe 200 lines

Phase 3: SemanticTypes (if Phase 2 ships)
  - Resolved signatures, visibility, async/unsafe flags
  - /visibility-audit-report, /unsafe-audit-report endpoints
  - Type-based search
  - Maybe 250 lines

Phase 4: Everything else (when someone has a concrete use case)
  - TypeLayouts, ClosureCaptures standalone, generic maps
  - Build on demand, not on speculation
```

---

## Performance: Three-Layer vs Two-Layer vs One-Layer

| Operation | CPU Only | Bidirectional (LLM+CPU) | Three-Layer (pt04+LLM+CPU) |
|---|---|---|---|
| Module Detection (1K entities) | 0.3s, 67% | 2.1s, 91% | 0.8s, ~96% |
| Cycle Classification (10 cycles) | 0.1s, 0% | 1.3s, 95% | 0.4s, ~99% |
| Complexity Analysis (50 functions) | 0.2s, 0% | 2.8s, 93% | 0.3s, ~95% |
| Tech Debt Scoring (100 files) | 0.8s, 64% | 4.2s, 89% | 2.5s, ~92% |
| Refactoring Suggestions | N/A | 5.0s, 91% | 3.5s, ~95% |
| Code Review Context | N/A | 8.0s, many tokens | 1.5s, minimal tokens |
| Architecture Orientation | 1.0s, data only | 5.0s, interpreted | 1.2s, pre-analyzed |
| Async Safety Audit | N/A | N/A (impossible) | 2.0s, compiler-verified |
| Visibility Audit | N/A | N/A (impossible) | 0.5s, compiler-verified |
| Pre-Merge Safety Gate | N/A | 10s, LLM-based | 0.8s, deterministic |

**Pattern**: Three-layer is faster than two-layer (fewer LLM calls) AND more accurate (compiler truth, not LLM guesses).

**Limitation**: pt04 is Rust-only. For Python/JS/Go, the original bidirectional approach with LLM-as-type-guesser remains the best option. The two architectures can coexist — pt04 enriches Rust entities, tree-sitter handles everything else.

---

## How pt04 Changes What pt01, pt02, pt03 Do

### pt01 (Ingestion): `pt01-folder-to-cozodb-streamer`

**Today**: tree-sitter parses source → extracts entities + edges → writes to CozoDB.

**With pt04**: pt01 runs tree-sitter FIRST (fast, all 12 languages), then pt04 runs rust-analyzer SECOND (Rust-only, adds typed edges).

```
pt01 pipeline:
  Phase 1: tree-sitter → entities + syntactic edges (all 12 languages, ~1.4s)
  Phase 2: pt04/rust-analyzer → typed edges + trait impls (Rust only, ~60-180s)

The result in CozoDB:
  - All entities from tree-sitter (Python, JS, Go, etc.)
  - Syntactic edges for all languages (Calls, Uses, Implements)
  - PLUS: TypedCallEdges for Rust (Direct/TraitMethod/DynDispatch/ClosureInvoke)
  - PLUS: TraitImpls for Rust (which types implement which traits)
  - PLUS: SemanticTypes for Rust (resolved signatures, visibility, async/unsafe)
```

**Key decision**: pt04 is a SECOND PASS on Rust files. It doesn't replace tree-sitter — it ENRICHES tree-sitter's output. Non-Rust files are unaffected. This means pt04 is backward-compatible: the same 26 endpoints work with or without pt04 data. When pt04 data is present, responses include `semantic.*` fields. When absent, they don't.

### pt02 (Snapshot Export): `pt02-cozodb-to-file-exporter`

**Today**: Exports CozoDB data to MessagePack file (slim model: entities + edges, no code bodies).

**With pt04**: The snapshot includes typed edges and trait impls:

```
Slim snapshot (without pt04): ~504 MB for 1.6M edges
  - Entity addresses (ISGL1 keys)
  - Syntactic edges (from_key, to_key, edge_type)

Enriched snapshot (with pt04): ~650 MB for 1.6M edges
  - Everything in slim
  - TypedCallEdges (from_key, to_key, call_kind, via_trait, receiver_type)
  - TraitImpls (impl_key, trait_name, self_type, items)
  - SemanticTypes (resolved signatures, visibility, async/unsafe)
```

**Format flag**: `parseltongue pt02 --format slim` (no pt04 data) vs `parseltongue pt02 --format enriched` (with pt04 data). Default is `enriched` if pt04 data exists in CozoDB.

### pt03 (Format): Becomes `--format` flag on pt02

**Per the Shreyas critique**: pt03 as a separate crate is overhead. It's a `--format` flag on pt02. The flag controls whether to include pt04 semantic data in the export.

### pt08 (HTTP Server): `pt08-http-code-query-server`

**Today**: Serves 26 endpoints from CozoDB.

**With pt04**: Same 26 endpoints, but responses include `semantic.*` fields when TypedCallEdges/TraitImpls/SemanticTypes relations exist in CozoDB. Plus new endpoints (PART IV) that only work when pt04 data is present.

**Graceful degradation**: If the server loads a snapshot WITHOUT pt04 data, all 26 endpoints work normally (tree-sitter data only). The `semantic.*` fields are simply absent. New pt04-only endpoints return `{"error": "semantic layer not available", "hint": "Re-ingest with pt04 to enable typed analysis"}`.

---

## The One Sentence Version

**pt04 gives every existing Parseltongue endpoint compiler-verified type information — making the LLM companion faster (fewer source reads), more accurate (ground truth types), and capable of new analyses (trait hierarchies, async safety, visibility audits) that are impossible with tree-sitter alone.**

---

## Open Questions (Honest)

1. **Does TypedCallEdges actually improve Leiden clustering?** Theory says yes (trait membership = better seeds). But we haven't tested it. Phase 1 should include a before/after comparison on a real codebase.

2. **Is the accuracy improvement worth the ingestion cost?** pt04 adds 1-3 minutes to ingestion for a 50-crate workspace. If the user only runs basic graph queries, they paid that cost for nothing. Should pt04 be opt-in (`parseltongue pt01-folder-to-cozodb-streamer . --semantic`)?

3. **Can we get TypedCallEdges from tree-sitter instead?** Partially. Tree-sitter can detect `.method()` syntax but can't resolve which impl block it dispatches to. The resolution is the valuable part. So: no.

4. **Will rust-analyzer's API stay stable?** It won't. rust-analyzer is an actively developed project that refactors its internals regularly. Pinning to a specific commit and updating quarterly is the realistic approach. This is ongoing maintenance cost.

5. **Is the three-layer architecture too complex to explain to users?** Users don't see layers. They see endpoints. The endpoint `/strongly-connected-components-analysis` either returns cycle classifications or it doesn't. The three layers are an implementation detail. But we should document it for contributors.

6. **Should pt04-only endpoints exist, or should everything be enrichment of existing endpoints?** The Shreyas answer: enrich existing endpoints FIRST. Only create new endpoints when the capability genuinely can't fit into an existing response. `/visibility-audit-report` is new because no existing endpoint covers visibility. `/async-call-chain-trace` is new because no existing endpoint traces async chains. But "typed blast radius" is just enrichment of `/blast-radius-impact-analysis`.

7. **How do we handle mixed-language codebases?** Rust entities get typed analysis. Non-Rust entities get tree-sitter analysis. Edges BETWEEN Rust and non-Rust entities (e.g., Rust calling C via FFI) get `call_kind: "FFI"` if pt04 can detect it, or `call_kind: "Unknown"` if it can't. The LLM should know which entities have semantic data and which don't.

8. **Should there be a `/semantic-coverage-status` endpoint?** Yes. The LLM needs to know: "73% of entities have typed analysis, 27% are tree-sitter only (15% non-Rust languages, 8% proc-macros, 4% cfg-gated)." This tells the LLM where to trust compiler data and where to fall back to source reading.

---

## What This Document IS and ISN'T

**IS**: A comprehensive architectural thesis showing how pt04's compiler semantics transform EVERY existing Parseltongue endpoint and workflow for higher-quality LLM companionship. Covers all 26 endpoints (PART II), all workflow patterns from README and UserJourney (PART III), new pt04-only capabilities (PART IV), and new pt04-only workflows (PART V).

**ISN'T**: A build plan. The build plan is Phase 1: ship TypedCallEdges, enrich the existing 26 endpoints, measure the improvement, decide on Phase 2. If TypedCallEdges doesn't measurably improve Leiden clustering or cycle classification on a real codebase, the thesis is wrong and we should stop.

**Shreyas would say**: "Ship Phase 1. Measure. Then decide if this document was right or wrong. Don't build Phases 2-4 based on a thesis. Build them based on data."

---

**Last Updated**: 2026-02-15
**Key Principle**: Compiler truth is cheaper, faster, and more accurate than LLM guessing. Use the LLM for judgment, not facts.
**Next Step**: Prototype TypedCallEdges extraction on Parseltongue's own codebase (eat your own dog food).
**Scope**: All 26 endpoints + 10 workflow patterns + 6 new endpoints + 5 new workflows.
