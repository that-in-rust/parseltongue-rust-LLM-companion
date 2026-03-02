# Code Cluster 01 Workflow - The 5-Phase Graph-Based Query Flow

**Version**: v256 Design
**Date**: 2026-03-02
**Status**: Core Architecture Pattern

---

## Executive Summary

This document defines the **canonical query flow** for Parseltongue v256. The goal is **super-high-value per token** with **zero hallucination** - every piece of context is compiler-verified.

**Key insight**: Progressive disclosure via **graph algorithms**, not embedding similarity.

---

## The 5-Phase Flow with Graph Algorithms

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  USER: "authentication"                                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 1: SEARCH (Token Budget: ~50 tokens)                                 │
│                                                                             │
│  ALGORITHM: RRF (Reciprocal Rank Fusion)                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Retriever 1: Symbol trie (exact matches)                             │    │
│  │ Retriever 2: Trigram index (fuzzy matches)                            │    │
│  │ Retriever 3: Recent edits (git history)                                │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                              ↓ RRF Fusion                                    │
│  OUTPUT: 4 candidates with scores                                           │
│    [auth::login: 0.89, AuthProvider: 0.72, authentication: 0.68, oauth: 0.41]│
│                                                                             │
│  TOKEN COST: 4 entity names + 4 scores = ~30 tokens                         │
│  TRUTH: ✅ All entities exist in rust-analyzer symbol table                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 2: ANCHOR (Token Budget: ~100 tokens)                               │
│                                                                             │
│  ALGORITHM: BFS UPWARD + API BOUNDARY DETECTION                             │
│                                                                             │
│  For "auth::login" (private function):                                      │
│                                                                             │
│      auth::login (private)                                                  │
│           ↓ calls                                                            │
│      auth::session::create (private)                                        │
│           ↓ calls                                                            │
│      api::handlers::login_route (PUBLIC) ← ANCHOR FOUND                    │
│                                                                             │
│  For "AuthProvider" (public trait):                                         │
│      AuthProvider ← already public, ANCHOR IS ITSELF                        │
│                                                                             │
│  OUTPUT: 2 anchored public entities                                         │
│    [api::handlers::login_route, AuthProvider]                              │
│                                                                             │
│  TOKEN COST: 2 paths × 20 tokens = ~40 tokens                              │
│  TRUTH: ✅ Compiler-verified call graph                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 3: CLUSTER (Token Budget: ~200 tokens)                              │
│                                                                             │
│  ALGORITHM: EGO NETWORK + PERSONALIZED PAGERANK                             │
│                                                                             │
│  CLUSTER 1: api::handlers::login_route                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Ego Network (distance ≤ 2):                                           │    │
│  │   • auth::login (callee, distance=1)                                  │    │
│  │   • middleware::auth_check (sibling, distance=1)                      │    │
│  │   • api::handlers::refresh_route (sibling, distance=1)                │    │
│  │                                                                       │    │
│  │ Module: crate::api::handlers                                           │    │
│  │ Signature: fn login_route(req: Request) -> Response                   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  CLUSTER 2: AuthProvider                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Ego Network (distance ≤ 2):                                           │    │
│  │   • JwtAuthProvider (impl, distance=1)                                │    │
│  │   • OAuthProvider (impl, distance=1)                                  │    │
│  │   • auth::login (uses via trait, distance=2)                          │    │
│  │                                                                       │    │
│  │ Module: crate::auth::provider                                         │    │
│  │ Signature: trait AuthProvider { fn login(&self, ...) -> Result; }     │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  OUTPUT: 2 clusters with ego networks                                       │
│                                                                             │
│  TOKEN COST: 2 clusters × 80 tokens = ~160 tokens                          │
│  TRUTH: ✅ Compiler-verified call graph + type hierarchy                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 4: DISAMBIGUATE (LLM decides)                                        │
│                                                                             │
│  PRESENT TO LLM:                                                           │
│                                                                             │
│  "I found 2 clusters for 'authentication':                                 │
│                                                                             │
│   [1] API HANDLER LAYER                                                     │
│       api::handlers::login_route                                            │
│       → HTTP endpoint, calls auth::login                                    │
│       Module: crate::api::handlers                                          │
│                                                                             │
│   [2] AUTH ABSTRACTION LAYER                                                │
│       AuthProvider trait                                                    │
│       → Has 2 implementations (JWT, OAuth)                                  │
│       Module: crate::auth::provider                                         │
│                                                                             │
│   Which are you interested in?"                                             │
│                                                                             │
│  TOKEN COST: ~60 tokens for LLM to read                                     │
│  TRUTH: ✅ All context is compiler-verified                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
                        LLM chooses: [1]
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 5: DEEP DIVE (Token Budget: ~2000 tokens)                           │
│                                                                             │
│  ALGORITHM: SUBGRAPH EXTRACTION + TYPE FLOW + CONTROL FLOW                 │
│                                                                             │
│  DELIVER:                                                                   │
│  1. Full code for api::handlers::login_route                               │
│  2. Complete call graph (transitive closure to depth 3)                    │
│  3. Type signatures for all involved entities                               │
│  4. Control flow highlights (error paths, async points)                    │
│  5. Related changes (git history - what changes with this file)            │
│                                                                             │
│  TOKEN COST: ~1500-2000 tokens                                              │
│  TRUTH: ✅ 100% compiler-verified, zero hallucination                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## The Key Graph Algorithms

| Phase | Algorithm | Purpose |
|-------|-----------|---------|
| **SEARCH** | RRF (Reciprocal Rank Fusion) | Combine symbol + fuzzy + git retrievers |
| **ANCHOR** | BFS upward + API boundary | Find public interface for private impl |
| **CLUSTER** | Ego Network + PageRank | Extract immediate neighborhood with importance |
| **DISAMBIGUATE** | Token-efficient summary | Let LLM choose |
| **DEEP DIVE** | Subgraph + Type flow + CFG | Full compiler-verified context |

---

## Why This Is "Super-High-Value Per Token"

### Traditional Embedding Search
```
"authentication" → 50 similar code chunks → 10,000 tokens of guessed relevance
```

### Parseltongue 5-Phase
```
Phase 1: 4 candidates   →   30 tokens (RRF-ranked)
Phase 2: 2 anchors      →   40 tokens (call graph verified)
Phase 3: 2 clusters     →  160 tokens (ego networks)
Phase 4: LLM picks      →   60 tokens (disambiguation)
Phase 5: Deep dive      → 1500 tokens (full context for ONE thing)
                           ─────────────────
                           TOTAL: ~1,800 tokens for PRECISE truth
```

### Compression Math
```
Traditional: 10,000 tokens
Parseltongue: 1,800 tokens
Compression: 82% token reduction
Truth: 100% compiler-verified, zero hallucination
```

---

## RRF (Reciprocal Rank Fusion) Implementation

From Sourcegraph learnings - combine multiple retrievers without ML:

```rust
const RRF_K: i32 = 60;

fn reciprocal_rank_fusion(
    retriever_results: Vec<HashMap<EntityId, i32>>, // rank per retriever
) -> Vec<(EntityId, f64)> {
    let mut scores: HashMap<EntityId, f64> = HashMap::new();

    for ranks in retriever_results {
        for (entity, rank) in ranks {
            *scores.entry(entity).or_default() += 1.0 / (RRF_K as f64 + rank as f64);
        }
    }

    let mut sorted: Vec<_> = scores.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    sorted
}
```

---

## Ego Network Extraction

Extract the immediate neighborhood of an entity:

```datalog
# CozoDB query for ego network (distance ≤ 2)

?[entity, distance, relation] :=
    *call_graph{caller: anchor, callee: entity},
    distance = 1,
    relation = "callee";

?[entity, distance, relation] :=
    *call_graph{caller: entity, callee: anchor},
    distance = 1,
    relation = "caller";

?[entity, distance, relation] :=
    *call_graph{caller: anchor, callee: intermediate},
    *call_graph{caller: intermediate, callee: entity},
    distance = 2,
    relation = "transitive_callee";

?[entity, distance, relation] :=
    *impls{trait: anchor, impl: entity},
    distance = 1,
    relation = "impl";
```

---

## BFS Upward for Anchor Detection

Find the public interface that exposes a private implementation:

```rust
fn find_anchor(entity: EntityId, call_graph: &CallGraph) -> Option<EntityId> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    queue.push_back((entity, 0));
    visited.insert(entity);
    
    while let Some((current, depth)) = queue.pop_front() {
        // Check if this is a public API boundary
        if is_public_api(&current) {
            return Some(current);
        }
        
        // BFS upward (find callers)
        for caller in call_graph.callers(&current) {
            if !visited.contains(&caller) {
                visited.insert(caller);
                queue.push_back((caller, depth + 1));
            }
        }
    }
    
    None
}

fn is_public_api(entity: &EntityId) -> bool {
    matches!(entity.visibility, Visibility::Public) &&
    matches!(entity.kind, EntityKind::Function | EntityKind::Trait)
}
```

---

## Token Budget Per Phase

| Phase | Budget | Purpose |
|-------|--------|---------|
| SEARCH | ~50 tokens | Narrow down candidates |
| ANCHOR | ~100 tokens | Find API boundaries |
| CLUSTER | ~200 tokens | Show neighborhoods |
| DISAMBIGUATE | ~100 tokens | Let LLM pick |
| DEEP DIVE | ~2000 tokens | Full context |
| **TOTAL** | **~2500 tokens** | Complete flow |

---

## The Meta Pattern: Truth-Based Context Compression

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   LLM says:   "I think this function is used here..."           │
│                      ↓                                          │
│   PARSLETONGUE SAYS:   "No. Here's the ACTUAL call graph."      │
│                      ↓                                          │
│   OUTPUT:   Compiler-verified context (zero hallucination)      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**The core insight**: It's not about MORE context. It's about THE RIGHT context, delivered progressively, verified by the compiler.

---

## Next Steps for v256 Implementation

1. **Phase 1 (SEARCH)**: Implement RRF with 3 retrievers (symbol, trigram, git)
2. **Phase 2 (ANCHOR)**: BFS upward with visibility detection
3. **Phase 3 (CLUSTER)**: Ego network extraction in CozoDB
4. **Phase 4 (DISAMBIGUATE)**: Token-efficient cluster summaries
5. **Phase 5 (DEEP DIVE)**: Full subgraph + type flow + CFG

---

## References

- `parseltongue-competitive-intel-2026/05-PARSELTONGUE-LEARNINGS-v1.7.2.md` - RRF implementation
- `parseltongue-competitive-intel-2026/06-PARSELTONGUE-EXISTING-TOOLS-RESEARCH.md` - Axon, Flowistry analysis
- `parseltongue-conversational-mcp-simulation.md` - Full UX simulation
