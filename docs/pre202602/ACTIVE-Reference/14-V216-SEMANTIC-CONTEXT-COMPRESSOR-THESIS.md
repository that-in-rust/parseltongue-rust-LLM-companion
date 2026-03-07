# Parseltongue v216: The Semantic Context Compressor Thesis

**Analysis Date:** 2026-01-29
**Method:** Rubber Duck Debugging + Rust-Analyzer Architecture Synthesis
**Conclusion:** The 3-layer model maps directly to rust-analyzer's HIR layer

---

## Executive Summary

This document synthesizes a thesis for Parseltongue v216 through rubber duck debugging against rust-analyzer's documented architecture. The conclusion: **rust-analyzer already IS the graph database we need — we just need to serialize what it computes.**

---

## The Raw Thesis Input

> **Layer 1**: LLMs → code search with random text from non-keyword words in codebase
> **Layer 2**: Find entity that wraps that code  
> **Layer 3**: Find related entities → under 10k tokens of high-context code including control/data flow

---

## 🦆 Rubber Duck Debugging Session

### Question 1: What problem are we actually solving?

**Claim:** "random text from non-keyword words in codebase"

**Duck asks:** Why non-keywords? What's wrong with searching for `HashMap` or `parse`?

**Answer emerges:** Keywords are ambiguous. `HashMap` appears 500 times. But `authenticate_user` or `websocket_frame_handler` — those are **unique identifiers of semantic concepts**. Non-keyword words (function names, variable names, type names) are the **semantic fingerprints** of code regions.

**The search isn't finding text — it's finding semantic anchors.** The user's query text gets matched against entity names, not arbitrary substrings. This is the first filtering layer.

---

### Question 2: "Find entity which wraps that code" — what entity?

**Claim:** Find the entity that wraps the matched code.

**Duck asks:** What's an "entity" in this model?

**Looking at rust-analyzer's hierarchy:**
```
Crate
  └── Module
       └── Struct/Enum/Trait
            └── Impl block
                 └── Method/Function
                      └── [matched code is here]
```

The entity that "wraps" code is the **innermost semantic container**. For a line inside a function, it's the function. For a struct field definition, it's the struct. For a trait bound, it's the trait or the type parameter.

**Key insight from hir-def patterns:** The `ItemTree` already knows containment. Every item has a parent in the module tree. This is O(1) lookup.

---

### Question 3: "Find related entities" — how do we define related?

**Claim:** Find related entities to that entity.

**Duck asks:** What makes entities "related"?

**This is where rust-analyzer's goldmine lives:**

| Relationship Type | Source | What it tells the LLM |
|-------------------|--------|----------------------|
| **Calls** | Function body → callee resolution | "To understand this code, you need to know what it calls" |
| **Called-by** | Reverse of calls | "To change this safely, know what depends on it" |
| **Implements** | `trait_impls_in_crate` | "This type has these capabilities" |
| **Implemented-by** | Reverse of implements | "These types satisfy this interface" |
| **Contains** | Module tree, struct fields | Hierarchical context |
| **Contained-by** | Parent modules | Scope resolution |
| **Uses-type** | Signatures, generic bounds | Data flow hints |
| **Visibility** | DefMap + ItemScope | What can see what |

**"Related" isn't one thing.** It's a **graph traversal** from the anchor entity with configurable depth and edge-type filters.

---

### Question 4: "Under 10k tokens" — what goes in that budget?

**Claim:** High-context-per-unit-token including control/data flow.

**Duck asks:** What has the highest information density for an LLM?

**The answer from rust-analyzer patterns:**

1. **Signatures over bodies** — ItemTree already gives you this
   ```rust
   fn process_request(req: Request) -> Result<Response, Error>
   // vs 47 lines of implementation
   ```
   The signature tells the LLM 90% of what it needs. The body is detail.

2. **Type relationships over type definitions** — Trait implementations
   ```rust
   impl Serialize for User { ... }
   impl Debug for User { ... }
   // vs the full User struct with 20 fields
   ```
   Knowing what a type *does* (traits) beats knowing what it *contains* (fields).

3. **Module structure over file contents** — DefMap
   ```
   auth/
     ├── service.rs      (AuthService, authenticate, validate_token)
     ├── middleware.rs   (AuthMiddleware, check_header)
     └── models.rs       (Credentials, Token, Session)
   ```
   This "map of the territory" orients the LLM in 50 tokens.

4. **Call graph slice over full call graph** — Blast radius
   ```
   authenticate → validate_token → decode_jwt → [crypto functions]
                 → check_permissions → [database query]
   ```
   Only the path relevant to the anchor entity.

---

### Question 5: Why 3 layers specifically?

**Duck's analysis:**

| Layer | Purpose | Information Type |
|-------|---------|------------------|
| **Layer 1: Search** | Find the entry point | Keyword → Entity mapping |
| **Layer 2: Anchor** | Establish context | Containment, scope |
| **Layer 3: Expand** | Gather related context | Graph traversal |

This maps directly to how humans understand code:
1. "Where is that thing?" (Search)
2. "Where does it live?" (Containment)
3. "What does it touch?" (Relationships)

**The 3-layer structure IS the minimal viable context pipeline.**

---

## 🎯 The Synthesized Thesis

### Thesis Statement

> **Parseltongue v216 is a semantic context compressor.**
> 
> It takes a free-form text query from an LLM and produces a maximally information-dense context window by:
> 1. **Mapping** query text to semantic entities via name matching
> 2. **Anchoring** to the entity's position in the code structure
> 3. **Expanding** through typed relationships to related entities
> 
> The output is sub-10k tokens of signatures, types, and relationships that encode the "understanding" of the codebase relevant to the query.

---

## The Architecture Thesis

```
┌─────────────────────────────────────────────────────────────────┐
│                     THE 3-LAYER PIPELINE                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  INPUT: LLM query (natural language or code fragment)          │
│         "how does auth work"                                    │
│         "authenticate_user function"                            │
│         "impl Serialize for Session"                            │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ LAYER 1: SEMANTIC SEARCH                                │   │
│  │                                                          │   │
│  │   Query → Extract nouns/identifiers                     │   │
│  │   Match against entity names in graph                   │   │
│  │   Output: Candidate entity IDs                          │   │
│  │                                                          │   │
│  │   Entity matching: "authenticate_user" → FunctionId(42) │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           ↓                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ LAYER 2: ENTITY ANCHORING                               │   │
│  │                                                          │   │
│  │   For each candidate:                                   │   │
│  │     - Resolve to containing module                      │   │
│  │     - Get entity signature (no body)                    │   │
│  │     - Get source location (file, lines)                 │   │
│  │                                                          │   │
│  │   Output: Anchor entity + immediate context             │   │
│  │                                                          │   │
│  │   FunctionId(42) → {                                     │   │
│  │     name: "authenticate_user",                           │   │
│  │     module: "auth::service",                             │   │
│  │     signature: "fn(Credentials) -> Result<Token, Error>",│   │
│  │     file: "crates/auth/src/service.rs:42-89",            │   │
│  │   }                                                       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           ↓                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ LAYER 3: RELATIONSHIP EXPANSION                         │   │
│  │                                                          │   │
│  │   From anchor, traverse edges by priority:              │   │
│  │                                                          │   │
│  │   Priority 1 (Essential - always include):              │   │
│  │     - Calls (what this function invokes)                │   │
│  │     - Uses-type (types in signature)                    │   │
│  │     - Implements (traits on this type)                  │   │
│  │                                                          │   │
│  │   Priority 2 (Context - include if budget allows):      │   │
│  │     - Called-by (who depends on this)                   │   │
│  │     - Contains (module structure, struct fields)        │   │
│  │     - Same-module (siblings)                            │   │
│  │                                                          │   │
│  │   Priority 3 (Deep context - for complex queries):      │   │
│  │     - Transitive calls (call graph slice)               │   │
│  │     - Trait hierarchy (supertraits, blanket impls)      │   │
│  │                                                          │   │
│  │   Output: Related entity set with token budget          │   │
│  │                                                          │   │
│  │   Expansion result:                                      │   │
│  │     authenticate_user [anchor, 150 tokens]              │   │
│  │     ├── validate_token [called, 80 tokens]              │   │
│  │     ├── Credentials [uses-type, 40 tokens]              │   │
│  │     ├── Token [uses-type, 30 tokens]                    │   │
│  │     ├── AuthError [uses-type, 30 tokens]                │   │
│  │     ├── AuthService [contains, 60 tokens]               │   │
│  │     └── impl Serialize for Token [implements, 50 tokens]│   │
│  │                                                          │   │
│  │   Total: 440 tokens for "understanding" auth            │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           ↓                                     │
│  OUTPUT: Compressed context (<10k tokens)                       │
│          Ready for LLM consumption                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## The Token Budget Thesis

### Why 10k tokens?

**Duck's calculation:**

| Context Type | Tokens per Entity | Count | Subtotal |
|--------------|-------------------|-------|----------|
| Anchor entity (full) | 150 | 1 | 150 |
| Related signatures | 80 | 10 | 800 |
| Type definitions (summary) | 40 | 15 | 600 |
| Module structure | 50 | 5 | 250 |
| Trait implementations | 30 | 10 | 300 |
| Call graph slice | 20 | 20 | 400 |
| **Overhead (formatting)** | — | — | 500 |
| **Total** | | | **~3000 tokens** |

**Key insight:** 10k tokens is **way more than needed** for most queries. A well-structured 3-layer extraction lands at 2-4k tokens for complex queries, 500-1000 for simple ones.

**The real constraint isn't tokens — it's relevance.** Every token should answer a question the LLM has.

---

## The Information Density Formula

**Thesis:** Context quality = (Information relevant to query) / (Total tokens)

To maximize this ratio:

1. **Signatures over bodies** — 10x density increase
2. **Relationships over implementations** — 5x density increase  
3. **Types over values** — 3x density increase
4. **Module maps over file listings** — 4x density increase

**The compression pipeline:**
```
Source file (10k tokens)
    ↓ [extract signature only]
Function signature (100 tokens) = 100x compression
    ↓ [add type relationships]
Signature + impls (200 tokens) = 50x compression with 2x information
    ↓ [add module context]
Signature + impls + module (250 tokens) = 40x compression with 3x information
```

---

## The "Random Text" Thesis — Why Non-Keywords Work

**Claim:** "random text from selection of non-keyword words"

**Duck's analysis:**

Rust keywords: `fn`, `let`, `impl`, `struct`, `use`, `pub`, etc. (40 words)

Rust identifiers in a typical codebase: 10,000+ unique names

**The insight:** Keywords are the **grammar**. Identifiers are the **semantics**.

When an LLM asks "how does authentication work", the semantic anchors are:
- `authenticate`
- `login`
- `session`
- `token`
- `credentials`

NOT:
- `fn` (appears everywhere)
- `impl` (appears everywhere)
- `let` (appears everywhere)

**The search algorithm:**
```rust
fn semantic_search(query: &str, graph: &Graph) -> Vec<EntityId> {
    // 1. Tokenize query
    let tokens = tokenize(query);
    
    // 2. Filter to non-keywords (semantic anchors)
    let anchors: Vec<&str> = tokens
        .into_iter()
        .filter(|t| !RUST_KEYWORDS.contains(t))
        .collect();
    
    // 3. Match anchors against entity names
    let mut candidates = Vec::new();
    for anchor in anchors {
        // Exact match first
        if let Some(id) = graph.entity_by_name(anchor) {
            candidates.push(id);
        }
        // Fuzzy match (substring, camelCase split)
        candidates.extend(graph.fuzzy_match_entities(anchor));
    }
    
    // 4. Rank by relevance (module depth, usage count)
    rank_candidates(candidates)
}
```

---

## The Control/Data Flow Thesis

**Claim:** "including some kind of control flow or data flow"

**Duck asks:** What flow information is useful to an LLM?

### Control Flow (What calls what)

```
For authenticate_user:
  Control flow slice (outgoing):
    authenticate_user → validate_token → decode_jwt
                     → check_permissions → query_database
                     → log_attempt → write_log
```

**LLM question answered:** "What happens when this runs?"

### Data Flow (What data moves where)

```
For authenticate_user:
  Data flow (input → output):
    Credentials (input) → validated → Token (output)
    Credentials.username → database lookup
    Credentials.password → hash comparison
```

**LLM question answered:** "What data does this transform?"

### Combined: The "Understanding Graph"

```
┌─────────────────┐
│ Credentials     │ [data input]
└────────┬────────┘
         │
         ▼
┌─────────────────┐      ┌─────────────────┐
│ authenticate    │─────▶│ validate_token  │ [control flow]
│ _user           │      └────────┬────────┘
└────────┬────────┘               │
         │                        ▼
         │               ┌─────────────────┐
         │               │ decode_jwt      │
         │               └─────────────────┘
         ▼
┌─────────────────┐
│ Token           │ [data output]
└─────────────────┘
```

**This is what fits in 500 tokens and gives the LLM "understanding."**

---

## The v216 Deliverable

### What We're Building

A binary that:
1. **Loads** a Rust workspace via `ra_ap_*` crates
2. **Extracts** the graph (ItemTree + DefMap + TraitImpls)
3. **Persists** to storage
4. **Exposes** 3-layer queries via MCP

That's it. No type inference. No macro expansion. No LSP server. Just extraction, persistence, and query.

### The `ra_ap_*` Crates Needed

| Crate | Purpose |
|-------|---------|
| `ra_ap_load_cargo` | Load workspace |
| `ra_ap_hir_def` | ItemTree + DefMap queries |
| `ra_ap_hir_ty` | trait_impls_in_crate, inherent_impls_in_crate |
| `ra_ap_hir` | High-level API wrapping all the above |
| `ra_ap_ide_db` | Symbol search integration (optional) |

---

## The Value Proposition

**Before Parseltongue:**
```
LLM: "How does authentication work?"
Human: [copies 5 files, 3000 lines of code]
LLM: [processes 50k tokens, still confused about module structure]
```

**After Parseltongue:**
```
LLM: "How does authentication work?"
Parseltongue: [Layer 1: match "authenticate"]
              [Layer 2: anchor to authenticate_user]
              [Layer 3: expand to auth module + related types]
              [Output: 800 tokens of signatures + relationships]
LLM: [processes 800 tokens, understands auth flow]
```

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Context compression ratio** | 50x | Source tokens / Output tokens |
| **Query accuracy** | 95% | Correct entity match rate |
| **LLM task success** | 2x improvement | Completion rate with Parseltongue vs raw code |
| **Response latency** | <100ms | Time from query to context output |

---

## The Extraction Points

### Extraction Point 1: ItemTree (The Gold Mine)

From hir-def Pattern 4:
> *"ItemTree is a simplified AST containing only items (no bodies), stored per HirFileId. It's designed as an 'invalidation barrier' — changes inside function bodies don't invalidate the ItemTree."*

ItemTree gives us:
- All struct/enum/trait/function **signatures** (without bodies)
- Generic parameters and bounds
- Visibility information
- Attribute flags
- Import declarations

**v216 action:** Write an ItemTree walker that extracts every item and its relationships.

### Extraction Point 2: DefMap + PerNs (The Relationship Map)

From hir-def Pattern 6 (DefMap) and Pattern 8 (PerNs):
- **Module tree and what's visible where**
- **Resolved name → definition mappings across all namespaces**
- **Visibility of each item**

**v216 action:** Query DefMap for each crate, walk the module tree, extract ItemScope contents with visibility.

### Extraction Point 3: Type Relationships (Selective)

From hir-ty Pattern 1:
```rust
fn trait_impls_in_crate(&self, krate: CrateId) -> Arc<TraitImpls>;
fn inherent_impls_in_crate(&self, krate: CrateId) -> Arc<InherentImpls>;
```

These give:
- **Every trait implementation in the workspace**
- **Every inherent impl block**

**v216 action:** Query these for each crate. Serialize type → trait relationships.

---

## What NOT To Build for v216

**Skip full type inference (hir-ty infer/).** The LLM doesn't need to know "the inferred type of expression on line 47." It needs structural relationships.

**Skip macro expansion details.** ItemTree already contains the **results** of macro expansion. You get derive implementations for free.

**Skip the event loop / LSP integration.** Parseltongue v216 runs as a batch process: load workspace → extract graph → persist → serve via MCP.

**Skip the Snapshot mechanism.** The persistence layer handles concurrent reads natively.

---

## Key Insight: Rust-Analyzer IS the Graph Database

From the data flow analysis:
> **What's Persisted:** None (in-memory only)
> **Why No Persistence:** Salsa queries are deterministic

The query dependency chain:
```
file_text → parse → item_tree → def_map → resolve_path → infer → IDE features
```

**For Parseltongue v216, we don't rebuild any of this.** We intercept at specific layers and serialize what's already computed. The rust-analyzer pattern knowledge is the **map** — now we know exactly where to drill.

---

## Conclusion

The 3-layer model maps directly to:
1. **How humans understand code** (search → anchor → relate)
2. **How rust-analyzer structures code** (names → containment → relationships)
3. **How LLMs process code** (keywords → signatures → patterns)

**The missing piece we already have:** Rust-analyzer's HIR layer already computes all of this. We're not building a compiler — we're building a **serialization layer** that captures what rust-analyzer knows and formats it for LLM consumption.

---

# V2: Three Production Upgrades

The base 3-layer model is sound. Three upgrades make it production-grade:

## Upgrade 1: Rerank Formula

```
score = lexical + semantic + graph_proximity + entity_type_intent + freshness
```

**What this is:** A **multi-signal ranking function** combining orthogonal relevance signals.

### Signal Decomposition

| Signal | Source | Range | When it matters most |
|--------|--------|-------|---------------------|
| **lexical** | String matching (exact, substring, fuzzy) | 0.0 - 1.0 | Query is a direct name reference |
| **semantic** | Embedding similarity, synonym expansion | 0.0 - 1.0 | Query is conceptual ("auth", "logging") |
| **graph_proximity** | Distance from other matched entities | 0.0 - 1.0 | Multiple entities found, need to cluster |
| **entity_type_intent** | Query pattern → entity type preference | 0.0 - 1.0 | "how do I" → functions, "what is" → types |
| **freshness** | Recency of modification | 0.0 - 1.0 | Codebase with active development |

### Weighted Combination with Query-Type Detection

```rust
struct RerankWeights {
    lexical: f32,
    semantic: f32,
    graph_proximity: f32,
    entity_type_intent: f32,
    freshness: f32,
}

impl RerankWeights {
    fn for_query_type(query_type: QueryType) -> Self {
        match query_type {
            QueryType::HowDoI => Self {
                lexical: 0.25,
                semantic: 0.15,
                graph_proximity: 0.20,
                entity_type_intent: 0.30,  // Boost functions/methods
                freshness: 0.10,
            },
            QueryType::WhatIs => Self {
                lexical: 0.20,
                semantic: 0.20,
                graph_proximity: 0.15,
                entity_type_intent: 0.25,  // Boost types/structs
                freshness: 0.20,
            },
            QueryType::WhereIs => Self {
                lexical: 0.40,  // Exact name matters
                semantic: 0.10,
                graph_proximity: 0.20,
                entity_type_intent: 0.15,
                freshness: 0.15,
            },
            QueryType::WhyDoes => Self {
                lexical: 0.20,
                semantic: 0.25,  // Conceptual understanding
                graph_proximity: 0.25,  // Context matters
                entity_type_intent: 0.15,
                freshness: 0.15,
            },
        }
    }
}

fn compute_score(entity: &Entity, query: &Query, weights: &RerankWeights) -> f32 {
    let lexical = lexical_score(&query.tokens, &entity.name);
    let semantic = semantic_score(&query.embedding, &entity.embedding);
    let graph_proximity = graph_proximity_score(entity, &query.matched_entities);
    let entity_type_intent = intent_score(query.type_hint(), entity.kind);
    let freshness = freshness_score(entity.last_modified, query.now);
    
    weights.lexical * lexical
        + weights.semantic * semantic
        + weights.graph_proximity * graph_proximity
        + weights.entity_type_intent * entity_type_intent
        + weights.freshness * freshness
}
```

### Creative Extension: Cluster-Aware Ranking

The `graph_proximity` signal enables **cluster-aware ranking** — entities that form coherent subgraphs get boosted together:

```
Query: "authentication flow"

Initial matches:
  - authenticate_user (fn, auth/service.rs)     lexical=0.95
  - AuthConfig (struct, auth/config.rs)         lexical=0.60
  - User (struct, models/user.rs)               lexical=0.40
  - authenticate (fn, oauth/provider.rs)        lexical=0.85

Graph proximity calculation:
  authenticate_user ─── calls ───> validate_token
         │
         └── uses ───> Credentials
                            │
                            └── contains ───> User
  
  Cluster 1: {authenticate_user, validate_token, Credentials, User}
    → authenticate_user is CENTROID
    → User gets proximity boost
  
  Cluster 2: {authenticate, AuthConfig, OAuthProvider}
    → Different cluster, no cross-boost

Final ranking:
  1. authenticate_user (0.95 + 0.80 proximity = 1.75)
  2. User (0.40 + 0.60 proximity = 1.00)  ← proximity rescued it
  3. authenticate (0.85 + 0.10 proximity = 0.95)
  4. AuthConfig (0.60 + 0.10 proximity = 0.70)
```

### Creative Extension: Learning Weights from Feedback

```rust
// When LLM successfully uses context, record positive signal
fn record_positive_feedback(query: &Query, selected_entity: &Entity) {
    // Boost weights that contributed to this match
    // Learning-to-rank with implicit feedback
}

// When LLM asks follow-up clarification, record uncertainty
fn record_uncertainty_feedback(query: &Query, candidates: &[Entity]) {
    // Adjust thresholds, boost underweighted signals
}
```

---

## Upgrade 2: Fail-Safe Branches

**Core insight:** Low confidence ≠ no answer. Low confidence = structured uncertainty.

### Confidence Gate

```
┌─────────────────────────────────────────────────────┐
│ CONFIDENCE GATE                                      │
│                                                      │
│  top_score >= 0.80?                                  │
│    → HIGH CONFIDENCE → Proceed to Layer 2           │
│                                                      │
│  top_score >= 0.50 && (top - second) >= 0.15?       │
│    → MEDIUM CONFIDENCE → Proceed with uncertainty   │
│                                                      │
│  top_score < 0.50 || (top - second) < 0.15?         │
│    → LOW CONFIDENCE → Fail-safe response            │
└─────────────────────────────────────────────────────┘
```

### Fail-Safe Response Structure

```json
{
  "status": "uncertain",
  "confidence": 0.42,
  "candidates": [
    { "name": "authenticate_user", "score": 0.48, "why": "lexical match on 'authenticate'" },
    { "name": "Authenticator", "score": 0.45, "why": "semantic match on 'auth'" },
    { "name": "auth_middleware", "score": 0.41, "why": "substring match" }
  ],
  "uncertainty_reasons": [
    "Query 'auth' matches 12 entities across 4 modules",
    "No exact match for 'auth flow'",
    "Top 3 candidates are semantically close (Δ < 0.07)"
  ],
  "suggested_clarifications": [
    "Are you referring to the login flow? (auth/service)",
    "Or the OAuth integration? (auth/oauth)",
    "Or the middleware chain? (middleware/auth)"
  ],
  "partial_context": {
    "modules_involved": ["auth", "middleware", "models"],
    "module_summary": "..."
  }
}
```

### Graceful Degradation Pyramid

```
┌─────────────────────────────────────────────────────────────┐
│               GRACEFUL DEGRADATION PYRAMID                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Level 1: Full Context (confidence >= 0.80)                 │
│    → Return complete 3-layer context                        │
│                                                             │
│  Level 2: Annotated Context (confidence >= 0.50)            │
│    → Return context with uncertainty annotations            │
│    → "I'm 70% sure this is what you want. If not, see..."  │
│                                                             │
│  Level 3: Candidate Set (confidence >= 0.30)                │
│    → Return top 3 candidates with summaries                 │
│    → "Found 3 possibilities. Please clarify:"              │
│                                                             │
│  Level 4: Module Map (confidence < 0.30)                    │
│    → Return module structure only                           │
│    → "Not sure what you mean. Here's the codebase map:"    │
│                                                             │
│  Level 5: Acknowledge (no matches at all)                   │
│    → Return nothing but log for learning                    │
│    → "I don't have context for that. Can you rephrase?"    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### SearchResult Enum

```rust
enum SearchResult {
    HighConfidence {
        entity: Entity,
        context: Context,
    },
    MediumConfidence {
        entity: Entity,
        context: Context,
        uncertainty: UncertaintyInfo,
    },
    LowConfidence {
        candidates: Vec<Candidate>,
        clarification_questions: Vec<String>,
        partial_context: PartialContext,
    },
    NoMatch {
        suggestions: Vec<String>,  // "Did you mean...?"
        nearest_entities: Vec<Entity>,
    },
}
```

---

## Upgrade 3: Token-Budget Packing Stages

**Budget allocation:**
- Core entity: 40%
- Direct neighbors: 35%
- Evidence snippets: 25%

**This is a knapsack problem.** Maximize value (information density) within budget (10k tokens).

### Multi-Stage Packing

```
┌─────────────────────────────────────────────────────────────┐
│                TOKEN BUDGET PACKER                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  BUDGET: 10,000 tokens                                      │
│                                                             │
│  STAGE 1: CORE ENTITY (40% = 4,000 tokens)                 │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ Priority queue:                                      │    │
│  │ 1. Entity signature           [150 tokens] ✓        │    │
│  │ 2. Generic bounds             [80 tokens] ✓         │    │
│  │ 3. Documentation              [200 tokens] ✓        │    │
│  │ 4. Source location            [30 tokens] ✓         │    │
│  │ 5. Module path context        [50 tokens] ✓         │    │
│  │                                                      │    │
│  │ Subtotal: 510 tokens                                 │    │
│  │ Overflow: 3,490 tokens → carry to Stage 2           │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
│  STAGE 2: DIRECT NEIGHBORS (35% + overflow = 6,990 tokens) │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ Essential (always include):                          │    │
│  │ 1. Calls (functions this calls)                      │    │
│  │    - validate_token signature [80 tokens] ✓         │    │
│  │    - check_permissions signature [70 tokens] ✓      │    │
│  │                                                      │    │
│  │ 2. Uses-type (types in signature)                    │    │
│  │    - Credentials struct summary [40 tokens] ✓       │    │
│  │    - Token struct summary [30 tokens] ✓             │    │
│  │                                                      │    │
│  │ 3. Implements (trait impls on this type)             │    │
│  │    - impl Serialize for Token [30 tokens] ✓         │    │
│  │                                                      │    │
│  │ Contextual (include if budget):                      │    │
│  │ 4. Called-by (who depends on this)                   │    │
│  │    - login_handler signature [80 tokens] ✓          │    │
│  │                                                      │    │
│  │ 5. Same-module siblings                              │    │
│  │    - AuthService summary [60 tokens] ✓              │    │
│  │                                                      │    │
│  │ Subtotal: 640 tokens                                 │    │
│  │ Overflow: 6,350 tokens → carry to Stage 3           │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
│  STAGE 3: EVIDENCE SNIPPETS (25% + overflow = 8,850 tokens)│
│  ┌─────────────────────────────────────────────────────┐    │
│  │ Evidence types (by value density):                   │    │
│  │                                                      │    │
│  │ 1. Test code (usage examples) - HIGH VALUE           │    │
│  │    - test_authenticate_success [200 tokens] ✓       │    │
│  │    - test_authenticate_invalid [180 tokens] ✓       │    │
│  │                                                      │    │
│  │ 2. Doc examples - MEDIUM VALUE                       │    │
│  │    - Example from docs [150 tokens] ✓               │    │
│  │                                                      │    │
│  │ 3. Error paths - MEDIUM VALUE                        │    │
│  │    - Error handling in caller [120 tokens] ✓        │    │
│  │                                                      │    │
│  │ 4. Implementation snippets - LOW VALUE               │    │
│  │    - Key lines from body [100 tokens] ✓             │    │
│  │                                                      │    │
│  │ Subtotal: 750 tokens                                 │    │
│  │ Remaining: 8,100 tokens                              │    │
│  │                                                      │    │
│  │ BACKFILL: Expand to more distant relations          │    │
│  │ - Transitive call graph (2 hops) [2000 tokens]      │    │
│  │ - Trait hierarchy [500 tokens]                       │    │
│  │ - Module cousins [1000 tokens]                       │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
│  FINAL OUTPUT: ~4,500 tokens of HIGH-DENSITY context       │
│  Well under 10k cap, leaving room for LLM reasoning        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Value-Per-Token Optimization

```rust
struct ContextPiece {
    content: String,
    token_count: usize,
    value_score: f32,  // 0.0 - 1.0
    category: ContextCategory,
}

impl ContextPiece {
    fn value_density(&self) -> f32 {
        self.value_score / self.token_count as f32
    }
}

fn pack_context(pieces: Vec<ContextPiece>, budget: usize) -> Vec<ContextPiece> {
    // Greedy knapsack: sort by value density, take highest first
    let mut sorted: Vec<_> = pieces.into_iter().collect();
    sorted.sort_by(|a, b| {
        b.value_density().partial_cmp(&a.value_density()).unwrap()
    });
    
    let mut packed = Vec::new();
    let mut used = 0;
    
    for piece in sorted {
        if used + piece.token_count <= budget {
            packed.push(piece);
            used += piece.token_count;
        }
    }
    
    packed
}
```

### Progressive Disclosure via MCP

```rust
// Initial response is compact
struct CompactContext {
    core: CoreEntity,           // Always included
    neighbors_summary: String,  // Names only, no details
    token_count: usize,
}

// LLM can request expansion
fn expand_context(&self, focus: ExpansionFocus) -> ExpandedContext {
    match focus {
        ExpansionFocus::Calls => self.expand_calls(),
        ExpansionFocus::Types => self.expand_types(),
        ExpansionFocus::Tests => self.expand_tests(),
        ExpansionFocus::All => self.expand_all(),
    }
}
```

---

## V2 Integrated Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                 V216 CONTEXT COMPRESSOR v2                              │
│              (The Three Upgrades Integrated)                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │ LAYER 1: SEMANTIC SEARCH + RERANK                                 │  │
│  │                                                                    │  │
│  │   Query → Match entities → Rerank formula → Confidence gate       │  │
│  │                                                                    │  │
│  │   Rerank: lexical + semantic + graph_proximity + intent + fresh   │  │
│  │                                                                    │  │
│  │   Confidence levels:                                               │  │
│  │   HIGH (≥0.80)    → Proceed to Layer 2                            │  │
│  │   MEDIUM (≥0.50)  → Proceed with uncertainty annotation           │  │
│  │   LOW (<0.50)     → FAIL-SAFE: return candidates + clarify        │  │
│  │                                                                    │  │
│  │   [FAIL-SAFE BRANCH #1]                                            │  │
│  │   If low confidence:                                               │  │
│  │     - Return top 3 candidates with scores                         │  │
│  │     - Include why each matched                                    │  │
│  │     - Generate clarification questions                            │  │
│  │     - Provide partial context (module map only)                   │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                              ↓                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │ LAYER 2: ENTITY ANCHORING                                         │  │
│  │                                                                    │  │
│  │   For high/medium confidence entities:                             │  │
│  │     - Resolve containing module                                   │  │
│  │     - Extract signature (no body)                                 │  │
│  │     - Get source location (file:lines)                            │  │
│  │     - Attach confidence score if medium                           │  │
│  │                                                                    │  │
│  │   [FAIL-SAFE BRANCH #2]                                            │  │
│  │   If entity resolution fails:                                      │  │
│  │     - Return module context only                                  │  │
│  │     - Explain what couldn't be resolved                           │  │
│  │     - Suggest next steps                                          │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                              ↓                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │ LAYER 3: RELATIONSHIP EXPANSION + TOKEN PACKING                   │  │
│  │                                                                    │  │
│  │   Budget: 10,000 tokens hard cap                                  │  │
│  │                                                                    │  │
│  │   STAGE 1: Core Entity (40%)                                      │  │
│  │     → Signature, generics, docs, location                         │  │
│  │     → Overflow carries to Stage 2                                 │  │
│  │                                                                    │  │
│  │   STAGE 2: Direct Neighbors (35%)                                 │  │
│  │     → Calls, uses-type, implements                                │  │
│  │     → Called-by (if budget)                                       │  │
│  │     → Same-module siblings (if budget)                            │  │
│  │     → Overflow carries to Stage 3                                 │  │
│  │                                                                    │  │
│  │   STAGE 3: Evidence Snippets (25%)                                │  │
│  │     → Test code (usage examples) - HIGH priority                  │  │
│  │     → Doc examples - MEDIUM priority                              │  │
│  │     → Error paths - MEDIUM priority                               │  │
│  │     → Implementation snippets - LOW priority                      │  │
│  │     → Backfill with transitive relations if budget remains        │  │
│  │                                                                    │  │
│  │   [FAIL-SAFE BRANCH #3]                                            │  │
│  │   If expansion produces >10k tokens:                               │  │
│  │     - Truncate at hard cap                                        │  │
│  │     - Note what was truncated                                     │  │
│  │     - Offer expansion endpoint for more                           │  │
│  │                                                                    │  │
│  │   Output: Packed context with value-density optimization          │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                              ↓                                          │
│  OUTPUT:                                                                │
│                                                                         │
│  {                                                                      │
│    "status": "high_confidence",  // or "medium", "low", "no_match"     │
│    "confidence": 0.87,                                                  │
│    "context": {                                                         │
│      "core": { "name": "authenticate_user", ... },                      │
│      "neighbors": [...],                                                │
│      "evidence": [...],                                                 │
│    },                                                                   │
│    "token_count": 4823,                                                 │
│    "expansion_available": true,  // LLM can request more               │
│    "truncated": ["transitive_calls", "trait_hierarchy"],                │
│  }                                                                      │
│                                                                         │
│  OR (low confidence):                                                   │
│                                                                         │
│  {                                                                      │
│    "status": "low_confidence",                                          │
│    "confidence": 0.42,                                                  │
│    "candidates": [                                                      │
│      {"name": "authenticate_user", "score": 0.48, "why": "..."},        │
│      {"name": "Authenticator", "score": 0.45, "why": "..."},            │
│    ],                                                                   │
│    "clarification_questions": [                                         │
│      "Are you referring to the login flow?",                            │
│      "Or the OAuth integration?",                                       │
│    ],                                                                   │
│    "partial_context": { "module_map": "..." },                          │
│  }                                                                      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## V2 Summary: From Linear to Resilient

| Aspect | V1 (Base) | V2 (With Upgrades) |
|--------|-----------|-------------------|
| **Ranking** | Single lexical match | Multi-signal rerank formula |
| **Confidence** | Binary (found/not) | Graduated with fail-safe branches |
| **Token budget** | Best-effort | Staged packing with overflow |
| **Output** | One context | Status-adaptive response |
| **LLM interaction** | One-shot | Progressive disclosure |

**V1:** Search → Anchor → Expand (linear pipeline)

**V2:** Search + Rerank → Confidence-gated Anchor → Staged Expansion (resilient, optimized pipeline)

---

## Creative Extensions Summary

1. **Cluster-aware ranking** — Graph proximity as entity clustering
2. **Learning weights from feedback** — Implicit learning-to-rank
3. **Interactive clarification loop** — LLM can ask itself or user
4. **Progressive disclosure via MCP** — Lazy loading of context
5. **Knapsack-style value optimization** — Maximize information density

---

*Thesis compiled: 2026-01-29*
*V2 Upgrades added: 2026-01-29*
*Method: Rubber Duck Debugging + Rust-Analyzer Architecture Analysis*
*Sources: 14 rust-analyzer pattern documents + user thesis input*
