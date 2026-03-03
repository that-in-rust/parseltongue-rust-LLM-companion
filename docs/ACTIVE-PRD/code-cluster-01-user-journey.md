# Parseltongue: Replace Grep for Code

**Version**: v256 Positioning
**Date**: 2026-03-03
**Core Thesis**: CPU-only, zero GPU, zero LLM intervention

---

## The Positioning

> **Grep returns files. Parseltongue returns understanding.**

| Tool | Returns |
|------|---------|
| grep | Lines matching pattern |
| IDE search | Files containing text |
| Embedding search | Similar code chunks (guessed) |
| **Parseltongue** | Compiler-verified clusters + deep context |

---

## The 7-Event User Journey

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 1: QUERY                                                             │
│                                                                             │
│  LLM sends a short query (~7 words)                                         │
│                                                                             │
│  Example: "authentication flow in this codebase"                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 2: SEARCH                                                            │
│                                                                             │
│  Parseltongue finds 4 candidate entities using RRF fusion                   │
│                                                                             │
│  Retrievers:                                                                │
│    - Symbol trie (exact matches)                                            │
│    - Trigram index (fuzzy matches)                                          │
│    - Git history (recent edits)                                             │
│                                                                             │
│  Output: [auth::login, AuthProvider, authentication module, oauth]          │
│                                                                             │
│  Token cost: ~30 tokens                                                     │
│  Time: <10ms (pure CPU)                                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 3: ANCHOR                                                            │
│                                                                             │
│  For each candidate, find the public API boundary                           │
│                                                                             │
│  Algorithm: BFS upward until public function/trait found                    │
│                                                                             │
│  For auth::login (private):                                                 │
│    auth::login → auth::session::create → api::handlers::login_route (PUB)  │
│                                                                             │
│  For AuthProvider (public trait):                                           │
│    Already public → anchor is itself                                        │
│                                                                             │
│  Output: Public interface + module path + immediate neighbors               │
│                                                                             │
│  Token cost: ~100 tokens                                                    │
│  Time: <50ms (graph traversal in CozoDB)                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 4: CLUSTER                                                           │
│                                                                             │
│  Build ego network (distance=1) for each anchored entity                    │
│                                                                             │
│  Cluster = anchor + callers + callees + implementations                     │
│                                                                             │
│  Each cluster compressed to max 3000 tokens                                 │
│                                                                             │
│  Token cost: ~3000 tokens per cluster (4 clusters = 12000 tokens internal)  │
│  Time: <100ms                                                               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 5: ASK                                                               │
│                                                                             │
│  Present 4 candidate-info-clusters to LLM                                   │
│                                                                             │
│  "I found 4 clusters for 'authentication':                                  │
│                                                                             │
│   [1] API HANDLER - login_route (HTTP endpoint, calls auth::login)          │
│   [2] AUTH TRAIT - AuthProvider (abstraction, 2 impls: JWT, OAuth)          │
│   [3] MODULE - authentication (folder with 12 files)                        │
│   [4] EXTERNAL - oauth (third-party integration)                            │
│                                                                             │
│   Which cluster? [1] [2] [3] [4] [none]"                                    │
│                                                                             │
│  Token cost: ~200 tokens for LLM to read                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
                        LLM chooses: [1]
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 6: CHOICE                                                            │
│                                                                             │
│  LLM responds with:                                                         │
│    - A number [1-4] → proceed to deep dive                                  │
│    - "none" → no relevant cluster, try different query                      │
│    - Quit → end session                                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
                        If [1-4] chosen:
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 7: DEEP DIVE                                                         │
│                                                                             │
│  Return full context for chosen cluster (up to 20k tokens)                  │
│                                                                             │
│  Includes:                                                                  │
│    - Complete code for anchor + ego network                                 │
│    - Control flow graph (branching, loops, error paths)                     │
│    - Data flow (where data comes from, where it goes)                       │
│    - Type signatures (compiler-verified)                                    │
│    - Git history (what changes with this code)                              │
│                                                                             │
│  Plus, suggest non-traditional queries for next step:                       │
│    - blast_radius() → who will break if I change this?                      │
│    - complexity() → how hard is this code?                                  │
│    - test_coverage() → what's untested?                                     │
│    - type_flow() → trace data from input to output                          │
│    - call_slice() → minimal executable path                                 │
│                                                                             │
│  Token cost: Up to 20,000 tokens                                            │
│  Time: <500ms (all pre-computed via rust-analyzer)                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Why This Wins

### For LLMs

| Need | How Parseltongue Helps |
|------|------------------------|
| Fast | All CPU, milliseconds not seconds |
| Accurate | Compiler-verified, zero hallucination |
| Efficient | Only pay tokens for what you choose |
| Transparent | Logs show exactly why results bubbled up |
| Flexible | Can always ask for more depth |

### For Humans

| Need | How Parseltongue Helps |
|------|------------------------|
| Simple | Single endpoint, no upfront questions |
| Trust | See the reasoning, not a black box |
| Control | You pick the cluster, not the system |
| Iterative | Drill down step by step |

---

## Token Economics

| Stage | Tokens (Internal) | Tokens (to LLM) |
|-------|-------------------|-----------------|
| Event 1: Query | 0 | 7 words |
| Event 2: Search | 30 | - |
| Event 3: Anchor | 100 | - |
| Event 4: Cluster | 12,000 | - |
| Event 5: Ask | - | 200 |
| Event 7: Deep Dive | - | Up to 20,000 |

**Key insight**: LLM only sees ~200 tokens before making choice, then pays 20k for deep dive on ONE cluster (not 80k for all 4).

---

## CPU-Only Guarantee

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   NO GPU                                                        │
│   NO EMBEDDING MODEL                                            │
│   NO LLM IN THE MIDDLE                                          │
│                                                                 │
│   Everything is:                                                │
│     - Symbol trie lookup (O(k) where k = query length)          │
│     - Trigram index scan (O(n) but highly optimized)            │
│     - Graph traversal in CozoDB (Datalog, compiled queries)     │
│     - rust-analyzer type information (pre-computed)             │
│                                                                 │
│   Transparency: Full logs of why each result ranked             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Next Queries After Deep Dive

Once LLM has the deep dive, it can ask follow-ups:

```
"blast radius for auth::login"
  → Returns: 23 functions affected, 3 critical paths

"complexity of this cluster"
  → Returns: cyclomatic 47, nesting 5, high risk

"test coverage"
  → Returns: 62% covered, auth::session untested

"type flow from Request to Response"
  → Returns: data transformation path with types at each step
```

Each follow-up is also CPU-only, milliseconds, compiler-verified.

---

## Summary

**Parseltongue replaces grep for code by returning understanding, not lines.**

The 7-event journey:
1. **QUERY** - LLM sends 7 words
2. **SEARCH** - RRF finds 4 candidates
3. **ANCHOR** - Find public API boundary
4. **CLUSTER** - Build ego network, compress to 3000 tokens each
5. **ASK** - Present [1][2][3][4][none]
6. **CHOICE** - LLM picks one
7. **DEEP DIVE** - Full context + next query options

All CPU. All transparent. All compiler-verified.
