# Can Lucene/Elasticsearch Help with Data Flow/Control Flow Analysis?
## Research for Parseltongue Architecture Decisions

**Research Date**: 2026-02-28
**Question**: Can search engine algorithms (Lucene, Elasticsearch, Zoekt) benefit code understanding tools like Parseltongue for data flow and control flow analysis?

---

## Executive Summary

**Short Answer**: **No, not directly.** Lucene/ES algorithms are fundamentally designed for **text retrieval**, not **graph traversal**.

| Analysis Type | Search Engines | Graph Databases |
|---------------|----------------|-----------------|
| Find "function named login" | ✅ Excellent | ✅ Good |
| Find "code like authentication" | ✅ Semantic search | ❌ Not designed |
| Compute blast radius | ❌ Not possible | ✅ Native |
| Track data from input to DB | ❌ Not possible | ✅ Path queries |
| Find all paths through code | ❌ Not possible | ✅ Recursive queries |

**Long Answer**: Search engines can help with the **retrieval phase** (finding candidate code), but NOT with the **analysis phase** (computing data/control flow).

---

## Part 1: What Lucene/Elasticsearch Actually Do

### 1.1 Core Algorithms

#### Inverted Index
```
Document: "function login(user, password) { ... }"

Tokenized: ["function", "login", "user", "password"]

Inverted Index:
  "function" → [doc1, doc5, doc12, ...]
  "login"    → [doc1, doc7, doc23, ...]
  "user"     → [doc1, doc2, doc3, ...]
```

**Purpose**: Map terms → documents for fast lookup

**Limitation**: No structural relationships. Cannot express "function X calls function Y."

#### BM25 Ranking
```
Score(D, Q) = Σ IDF(qi) × (f(qi, D) × (k1 + 1)) / (f(qi, D) + k1 × (1 - b + b × |D|/avgdl))
```

Where:
- `f(qi, D)` = frequency of term qi in document D
- `IDF(qi)` = inverse document frequency (rarity)
- `|D|` = document length
- `k1, b` = tuning parameters (typically k1=1.2, b=0.75)

**Purpose**: Rank documents by relevance

**Limitation**: Based on term frequency, not code semantics. A long comment about "login" scores higher than the actual `login()` function.

#### Trigram Indexing (Zoekt)
```
Code: "function login"

Trigrams: ["fun", "unc", "nct", "cti", "tio", "ion", "on ", "n l", " lo", "log", "ogi", "gin"]

Query: "login"
→ Look up trigrams: ["log", "ogi", "gin"]
→ Find documents containing all three
→ Verify positions are adjacent
```

**Purpose**: Fast substring/regex search in code

**Limitation**: Still text-based. Cannot express "find all functions called within this loop."

### 1.2 What ES Can Do for Code

| Task | Works? | How |
|------|--------|-----|
| Find files containing "login" | ✅ | Inverted index |
| Regex search `fn \w+\(.*auth.*\)` | ✅ | Trigram + verification |
| Semantic search "authentication code" | ✅ | Vector embeddings + cosine |
| Find function definition | ⚠️ Partial | Needs AST metadata |
| Find callers of function X | ❌ | No call graph |
| Compute transitive dependencies | ❌ | No graph traversal |
| Track variable through code | ❌ | No data flow |

---

## Part 2: What Data Flow/Control Flow Analysis Requires

### 2.1 Core Algorithms

#### Control Flow Graph (CFG)
```
                    ┌─────────────┐
                    │   entry     │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │  if (x > 0) │
                    └──────┬──────┘
                    ┌──────┴──────┐
                    │             │
             ┌──────▼──────┐ ┌────▼────┐
             │  x = x + 1  │ │  x = 0  │
             └──────┬──────┘ └────┬────┘
                    │             │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │   return x  │
                    └─────────────┘
```

**Required Operations**:
- Graph traversal (DFS, BFS)
- Path enumeration
- Dominance analysis
- Loop detection

**ES Capability**: ❌ Cannot represent graphs natively

#### Worklist Algorithm (Data Flow)
```
1. Initialize: worklist = all nodes, in[n] = ⊥ for all n
2. While worklist not empty:
   a. Remove node n from worklist
   b. in[n] = ⨆ out[p] for all predecessors p of n
   c. out[n] = transfer_function(n, in[n])
   d. If out[n] changed:
      Add all successors of n to worklist
3. Return in[] and out[] for all nodes
```

**Required Operations**:
- Predecessor/successor lookups
- Lattice operations (meet, join)
- Fixed-point iteration
- Monotonic transfer functions

**ES Capability**: ❌ Cannot iterate over graph structure

#### Taint Analysis (Security)
```
Sources (untrusted input):
  - request.POST['data']
  - socket.recv()
  - file.read()

Sinks (dangerous operations):
  - db.execute(query)
  - eval(code)
  - os.system(cmd)

Analysis: Track all paths from sources to sinks
```

**Required Operations**:
- Path queries (source → sink)
- Variable assignment tracking
- Function call resolution
- Cross-file analysis

**ES Capability**: ❌ Cannot track paths or relationships

### 2.2 Comparison Table

| Operation | Lucene/ES | Graph DB | Parseltongue (CozoDB) |
|-----------|-----------|----------|----------------------|
| Full-text search | ✅ Native | ⚠️ Possible | ⚠️ Possible |
| Regex search | ✅ Native | ❌ | ❌ |
| Vector similarity | ✅ Native | ❌ | ❌ |
| Find node by ID | ✅ Fast | ✅ Fast | ✅ Fast |
| Find neighbors | ❌ | ✅ Native | ✅ Native |
| Shortest path | ❌ | ✅ Native | ✅ Native |
| Transitive closure | ❌ | ✅ Recursive | ✅ Recursive |
| Cycle detection | ❌ | ✅ Algorithms | ✅ Algorithms |
| Fixed-point iteration | ❌ | ⚠️ Custom | ⚠️ Custom |

---

## Part 3: Where Search Engines CAN Help Parseltongue

### 3.1 Two-Phase Architecture

```
┌────────────────────────────────────────────────────────────┐
│                    USER QUERY                               │
│              "Where is user authentication?"                │
└────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────────┐
│              PHASE 1: RETRIEVAL (Search Engine)             │
│                                                             │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐       │
│  │ BM25 Search │   │ Vector Search│   │ Regex Search│       │
│  │ "auth"      │   │ "auth code" │   │ fn.*login.* │       │
│  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘       │
│         │                 │                 │               │
│         └────────────┬────┴────────────────┘               │
│                      ▼                                      │
│              ┌───────────────┐                              │
│              │  RRF Fusion   │                              │
│              │  (Top 20)     │                              │
│              └───────┬───────┘                              │
└──────────────────────┼──────────────────────────────────────┘
                       │ Candidate entities
                       ▼
┌────────────────────────────────────────────────────────────┐
│              PHASE 2: ANALYSIS (Graph Database)             │
│                                                             │
│  ┌─────────────────────────────────────────────────┐       │
│  │  For each candidate:                             │       │
│  │    - Look up in CozoDB graph                     │       │
│  │    - Compute blast radius                        │       │
│  │    - Find related entities                       │       │
│  │    - Check data flow paths                       │       │
│  └─────────────────────────────────────────────────┘       │
│                      │                                      │
│                      ▼                                      │
│              ┌───────────────┐                              │
│  │  Enriched results│                              │
│              │  with context  │                              │
│              └───────────────┘                              │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────────┐
│                    FINAL RESULTS                            │
│  Entity + file + lines + blast_radius + dependencies       │
└────────────────────────────────────────────────────────────┘
```

### 3.2 What to Add to Parseltongue

**Option A: Add BM25/Jaccard (Low Effort, High Value)**

```rust
// Pure Rust, no external service
struct LocalSearchIndex {
    inverted_index: HashMap<String, Vec<EntityId>>,
    entity_signatures: HashMap<EntityId, String>,
}

impl LocalSearchIndex {
    fn search(&self, query: &str, top_k: usize) -> Vec<(EntityId, f64)> {
        let query_terms = tokenize(query);
        let mut scores: HashMap<EntityId, f64> = HashMap::new();

        for term in query_terms {
            if let Some(entities) = self.inverted_index.get(&term) {
                for entity_id in entities {
                    *scores.entry(*entity_id).or_default() += 1.0;
                }
            }
        }

        // BM25 scoring...
        let mut ranked: Vec<_> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ranked.into_iter().take(top_k).collect()
    }
}
```

**Effort**: ~200 lines of Rust
**Value**: Fast keyword search without embedding model

**Option B: Add Vector Embeddings (Medium Effort)**

```rust
use candle_core::Tensor;
use tokenizers::Tokenizer;

struct EmbeddingIndex {
    embeddings: HashMap<EntityId, Vec<f32>>,
    model: EmbeddingModel,  // e.g., all-MiniLM-L6-v2
}

impl EmbeddingIndex {
    fn search(&self, query: &str, top_k: usize) -> Vec<(EntityId, f64)> {
        let query_embedding = self.model.embed(query);
        let mut scores: Vec<_> = self.embeddings.iter()
            .map(|(id, emb)| (*id, cosine_similarity(&query_embedding, emb)))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores.into_iter().take(top_k).collect()
    }
}
```

**Effort**: ~500 lines + model loading (~50MB)
**Value**: Semantic search ("auth" finds "login", "verify_credentials")

**Option C: Keep Graph-Only, Steal RRF (Low Effort)**

Parseltongue already has:
- Entity extraction (tree-sitter)
- Dependency graph (CozoDB)
- API endpoints (26 REST)

Add only:
- RRF for combining multiple ranking signals
- Jaccard similarity for local matching

---

## Part 4: What Sourcegraph Does (Lessons)

### 4.1 Sourcegraph's Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     SOURCEGRAPH STACK                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │    Zoekt    │  │  Embeddings │  │    SCIP     │         │
│  │  (Trigram)  │  │  (Vectors)  │  │  (Precise)  │         │
│  │  Search     │  │  Search     │  │  Index      │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                 │
│         └────────────────┼────────────────┘                 │
│                          ▼                                  │
│                  ┌───────────────┐                          │
│                  │  RRF Fusion   │                          │
│                  │  + Reranking  │                          │
│                  └───────┬───────┘                          │
│                          │                                  │
│                          ▼                                  │
│                  ┌───────────────┐                          │
│                  │   Postgres    │                          │
│                  │   (Storage)   │                          │
│                  └───────────────┘                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Key Insight**: Sourcegraph uses THREE different systems:
1. **Zoekt** for fast text/regex search
2. **Embeddings** for semantic search
3. **SCIP** for precise code intelligence

They don't try to make one system do everything.

### 4.2 What Parseltongue Already Does Better

| Feature | Sourcegraph | Parseltongue |
|---------|-------------|--------------|
| Blast radius query | Multiple SQL queries | Single Datalog query |
| Transitive dependencies | N round trips | Recursive query |
| Complexity hotspots | Not available | Native metrics |
| Local deployment | Requires servers | Single binary |

### 4.3 What Parseltongue Should Steal

1. **Zoekt's trigram indexing** → Add for regex search
2. **RRF** → For combining retrieval signals
3. **Jaccard with stemming** → For local keyword matching

**NOT worth stealing**:
- Postgres (CozoDB is better for graphs)
- SCIP (requires compiler per language)
- Zoekt server (overkill for single codebase)

---

## Part 5: Concrete Recommendations for Parseltongue

### 5.1 Don't Add Lucene/ES

| Reason | Explanation |
|--------|-------------|
| **Wrong data model** | ES = documents, Parseltongue = graph |
| **Wrong query model** | ES = keyword matching, DF/CF = path traversal |
| **Deployment complexity** | ES requires JVM, clusters, maintenance |
| **No path queries** | Cannot express "find all paths from A to B" |
| **No fixed-point** | Cannot iterate until convergence |

### 5.2 DO Add These (From Search Engine Research)

#### 1. BM25 for Entity Ranking (100 lines)
```rust
fn bm25_score(
    entity: &Entity,
    query_terms: &[String],
    avg_doc_length: f64,
    k1: f64,  // 1.2
    b: f64,   // 0.75
) -> f64 {
    let doc_length = entity.signature.len() as f64;
    let mut score = 0.0;

    for term in query_terms {
        let tf = entity.term_frequency(term) as f64;
        let idf = entity.inverse_doc_frequency(term);

        let numerator = tf * (k1 + 1.0);
        let denominator = tf + k1 * (1.0 - b + b * doc_length / avg_doc_length);

        score += idf * numerator / denominator;
    }

    score
}
```

#### 2. Jaccard Similarity with Stemming (200 lines)
```rust
fn jaccard_similarity(text1: &str, text2: &str) -> f64 {
    let tokens1: HashSet<String> = tokenize_and_stem(text1);
    let tokens2: HashSet<String> = tokenize_and_stem(text2);

    let intersection = tokens1.intersection(&tokens2).count();
    let union = tokens1.union(&tokens2).count();

    if union == 0 { 0.0 } else { intersection as f64 / union as f64 }
}

fn tokenize_and_stem(text: &str) -> HashSet<String> {
    text.split_whitespace()
        .flat_map(|word| {
            // Break camelCase
            split_camel_case(word)
        })
        .filter(|w| !is_stop_word(w))
        .map(|w| stem(w))  // Porter stemmer
        .collect()
}
```

#### 3. RRF for Combining Signals (50 lines)
```rust
const RRF_K: i32 = 60;

fn reciprocal_rank_fusion(
    rankings: Vec<Vec<(EntityId, f64)>>,
    top_k: usize,
) -> Vec<(EntityId, f64)> {
    let mut scores: HashMap<EntityId, f64> = HashMap::new();

    for ranking in rankings {
        for (rank, (entity_id, _)) in ranking.iter().enumerate() {
            *scores.entry(*entity_id).or_default() +=
                1.0 / (RRF_K as f64 + rank as f64 + 1.0);
        }
    }

    let mut fused: Vec<_> = scores.into_iter().collect();
    fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    fused.into_iter().take(top_k).collect()
}
```

### 5.3 Architecture Recommendation

```
┌─────────────────────────────────────────────────────────────┐
│                     PARSELTONGUE v2                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                 RETRIEVAL LAYER                      │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐         │   │
│  │  │  BM25     │ │  Jaccard  │ │  Graph    │         │   │
│  │  │  Search   │ │ Similarity│ │  Lookup   │         │   │
│  │  └─────┬─────┘ └─────┬─────┘ └─────┬─────┘         │   │
│  │        │             │             │                │   │
│  │        └─────────────┼─────────────┘                │   │
│  │                      ▼                              │   │
│  │              ┌───────────────┐                      │   │
│  │              │   RRF Fusion  │                      │   │
│  │              └───────┬───────┘                      │   │
│  └──────────────────────┼──────────────────────────────┘   │
│                         │                                   │
│                         ▼                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                ANALYSIS LAYER (CozoDB)               │   │
│  │                                                      │   │
│  │  • Blast radius (recursive query)                    │   │
│  │  • Dependency paths (shortest path)                  │   │
│  │  • Complexity hotspots (aggregation)                 │   │
│  │  • Data flow patterns (pattern matching)             │   │
│  │                                                      │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Part 6: Summary

### The Answer

| Question | Answer |
|----------|--------|
| Can Lucene/ES help with data flow? | **No** - wrong data model |
| Can Lucene/ES help with control flow? | **No** - cannot traverse graphs |
| Can Lucene/ES help with code search? | **Yes** - that's what they're built for |
| Should Parseltongue use Lucene/ES? | **No** - add local search instead |
| What should Parseltongue add? | BM25 + Jaccard + RRF (350 lines) |

### Key Insight

> **Search engines find THINGS. Graph databases find RELATIONSHIPS.**

Data flow and control flow are fundamentally about **relationships** (what calls what, where data flows). This is graph territory, not search engine territory.

### Implementation Priority

| Priority | Feature | Effort | Impact |
|----------|---------|--------|--------|
| P0 | RRF for combining signals | 1 day | High |
| P1 | Jaccard with stemming | 2 days | High |
| P1 | BM25 ranking | 1 day | Medium |
| P2 | Vector embeddings (optional) | 3 days | Medium |
| ❌ | Lucene/Elasticsearch | Weeks | Negative |

---

## References

- Lucene/Elasticsearch: Inverted index, BM25, trigram search
- Zoekt (Sourcegraph): Trigram-based code search
- Data flow analysis: Worklist algorithm, monotone frameworks, fixed-point iteration
- Control flow: CFG construction, dominance, path analysis
- Graph databases for code: Neo4j, CozoDB, Code Property Graph
- RRF: Reciprocal Rank Fusion from Sourcegraph Cody
