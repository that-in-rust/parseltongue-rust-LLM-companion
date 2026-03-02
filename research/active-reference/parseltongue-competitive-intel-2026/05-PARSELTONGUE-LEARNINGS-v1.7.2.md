# Parseltongue Learnings from Sourcegraph
**Version Context**: Parseltongue v1.7.2 (Windows ingestion fix)
**Analysis Date**: 2026-02-28
**Source**: `sourcegraph-public-snapshot` + existing competitive intel

---

## Executive Summary

Sourcegraph is a **server-side enterprise platform** requiring 8+ services. Parseltongue is a **local CPU binary** with zero deployment. These are fundamentally different markets.

**Key insight**: Sourcegraph's moat is SCIP (precise type-aware indexing). Parseltongue's moat is **graph-native queries** (blast radius, complexity hotspots).

---

## What Parseltongue Already Does Better

| Feature | Sourcegraph | Parseltongue |
|---------|-------------|--------------|
| **Deployment** | 8+ services, Postgres, object storage | Single binary |
| **Speed** | Minutes to hours for indexing | Milliseconds to seconds |
| **Blast radius** | Indirect (reference count aggregation) | Direct (CozoDB Datalog query) |
| **API simplicity** | Complex GraphQL schema | 26 flat REST endpoints |
| **Offline** | Requires server | Fully local |
| **Complexity metrics** | None exposed | Cyclomatic, nesting, function length |
| **Graph queries** | Multiple Postgres round-trips | Single recursive Datalog query |

---

## High-Priority Features to Steal

### 1. RRF (Reciprocal Rank Fusion) - **50 lines, HIGH IMPACT**

Sourcegraph combines multiple retrievers using RRF. When you have:
- Embedding similarity results
- Jaccard similarity results
- LSP graph results
- Recent edits results

RRF fuses them:

```rust
// Pseudo-code for Parseltongue
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

**Implementation location**: Add to context selection module
**Source**: `cody/vscode/src/completions/context/reciprocal-rank-fusion.ts`

---

### 2. Jaccard Similarity with Stemming - **200 lines, NO ML REQUIRED**

Sourcegraph's primary local retriever for autocomplete is NOT embeddings - it's **Jaccard on stemmed words**:

```rust
// Pipeline:
// 1. Tokenize text
// 2. Break camelCase/snake_case -> constituent words
// 3. Remove stop words
// 4. Stem (Porter stemmer)
// 5. Build word frequency map
// 6. Jaccard = intersection / union
```

**Why this matters**: Works without any ML model, without network, without GPU. Pure algorithmic matching that's surprisingly effective for code.

**Rust crates**: `rust-stemmers` for Porter stemmer, `unicode-segmentation` for word breaking

**Implementation**: Add as alternative to embedding search
**Source**: `cody/vscode/src/completions/context/retrievers/jaccard-similarity/bestJaccardMatch.ts`

---

### 3. Semantic Chunking - **100 lines**

Sourcegraph splits at function/declaration boundaries, not arbitrary character counts:

```rust
// Splittable line prefixes (Rust-specific):
const SPLITTABLE_PREFIXES: &[&str] = &[
    "//", "#", "/*",           // comments
    "fn", "pub", "async",       // function starts
    "const", "static", "let",   // declarations
    "struct", "enum", "trait", "impl",  // type definitions
    "mod", "use",               // module items
];

// Parameters:
// - Hard limit: 800 tokens per chunk (never exceed)
// - Soft limit: 400 tokens (split here if on good boundary)
// - Minimum file size: 128 chars (embed whole file if smaller)
```

**Implementation**: Modify entity extraction to respect chunk boundaries
**Source**: `sourcegraph/internal/codeintel/context/split.go`

---

### 4. Source Provenance - **Schema change, LOW EFFORT**

Every result should include where it came from:

```rust
#[derive(Serialize)]
pub enum ContextSource {
    User,        // Explicitly @-mentioned
    Search,      // From vector/keyword search
    Graph,       // From dependency graph traversal
    BlastRadius, // From transitive dependency closure
    RecentEdit,  // Recently modified
    Initial,     // Default context
}

#[derive(Serialize)]
pub struct ContextItem {
    pub entity: Entity,
    pub source: ContextSource,
    pub confidence: f64,      // 0.0 to 1.0
    pub distance: Option<u32>, // Hops from query entity
}
```

**Why matters**: LLM agents can reason about context quality when they know provenance.

**Source**: `cody/lib/shared/src/codebase-context/messages.ts` - `ContextItemSource` enum

---

### 5. cl100k Token Counting - **Use `tiktoken-rs` crate**

Sourcegraph uses real tokenization (cl100k_base, same as GPT-4/Claude) for accuracy:

```rust
// Use tiktoken-rs crate
use tiktoken_rs::cl100k_base;

fn count_tokens(text: &str) -> usize {
    let bpe = cl100k_base().unwrap();
    bpe.encode_with_special_tokens(text).len()
}

// Warning: tokens don't sum!
// count_tokens(a) + count_tokens(b) != count_tokens(a + b)
// Always count AFTER concatenation
```

**Current Parseltongue**: Uses `CHARS_PER_TOKEN = 4` approximation
**Problem**: Can be off by 4x+ for non-Latin languages (Japanese, Chinese)

**Source**: `cody/lib/shared/src/token/counter.ts`

---

## Medium-Priority Features

### 6. Async Job Pattern for Expensive Queries

Blast radius queries on large codebases can take 10+ seconds. Use the async pattern:

```
POST /api/blast-radius
  -> 202 Accepted, { "job_id": "abc123" }

GET /api/jobs/abc123
  -> { "status": "processing", "progress": 0.45 }
  -> { "status": "completed", "result": {...} }
  -> { "status": "failed", "error": "..." }
```

**Recommended timeout**: 300 seconds (5 minutes)
**Poll interval**: 10 seconds

**Source**: Deep Search API pattern from `amp-contrib/deep_search/`

---

### 7. File Exclusion Patterns

```rust
const DEFAULT_EXCLUDED_PATTERNS: &[&str] = &[
    ".*ignore",           // .gitignore, .eslintignore
    ".gitattributes",
    ".mailmap",
    "*.csv", "*.svg", "*.xml",
    "__fixtures__/",
    "node_modules/",
    "testdata/",
    "mocks/",
    "vendor/",
    "target/",            // Rust-specific
    "*.lock",             // Lockfiles
];
```

**Implementation**: Add to ingestion pipeline
**Source**: `sourcegraph/internal/embeddings/embed/files.go`

---

### 8. Auto-Generated File Detection

Check first 1024 bytes for patterns:

```rust
const AUTOGENERATED_HEADERS: &[&str] = &[
    "autogenerated file",
    "auto-generated",
    "lockfile",
    "generated by",
    "do not edit",
    "code generated",
    "@generated",
];

fn is_autogenerated(content: &str) -> bool {
    let header = &content[..content.len().min(1024)];
    AUTOGENERATED_HEADERS.iter().any(|h| header.to_lowercase().contains(h))
}
```

**Source**: `sourcegraph/internal/embeddings/embed/files.go`

---

### 9. Two-Tier Token Budgets

```rust
pub const FAST_MODEL_TOKEN_BUDGET: usize = 4096;      // haiku, small models
pub const SMART_MODEL_TOKEN_BUDGET: usize = 7000;     // sonnet, large models
pub const EXTENDED_MODEL_TOKEN_BUDGET: usize = 30000; // opus, huge context

// As API parameter
GET /api/context?query=auth&token_budget=7000
```

**Source**: `cody/lib/shared/src/token/constants.ts`

---

### 10. RetrieverStat Metadata

Return stats about HOW context was retrieved:

```rust
#[derive(Serialize)]
pub struct RetrieverStat {
    pub name: String,           // "embedding", "jaccard", "graph"
    pub suggested_items: usize, // Items this retriever found
    pub retrieved_items: usize, // Items that made it into final context
    pub chars: usize,           // Total characters contributed
    pub duration_ms: u64,       // Time taken
}
```

**Source**: `cody/vscode/src/completions/context/context-mixer.ts`

---

## NOT Worth Replicating

| Feature | Why Skip |
|---------|----------|
| **SCIP** | Requires compiler per language (scip-go needs Go toolchain, etc.) |
| **Zoekt** | Trigram search is overkill for single-codebase local tool |
| **Cross-repo** | Parseltongue is single-codebase focused |
| **Enterprise auth** | No auth is a feature for local tools |
| **Postgres storage** | CozoDB is better for graph queries |
| **Embedding hosting** | Local tools should use local embeddings or none |

---

## Unique Parseltongue Angles to Double Down On

### 1. Blast Radius as First-Class API

Sourcegraph has NO concept of blast radius. They'd need multiple GraphQL queries to compute transitive closure.

With CozoDB, this is one recursive query:

```datalog
?[affected_entity, distance] :=
    changed[entity],
    *depends_on{dependent: affected_entity, dependency: entity},
    distance = 1
;

?[affected_entity, distance] <-
    ?[affected_entity, distance + 1] :=
    ?[affected_entity, distance],
    *depends_on{dependent: further_affected, dependency: affected_entity}
;
```

**Action**: Make blast radius the hero feature. Add `/api/blast-radius/{entity}` endpoint with visualization support.

---

### 2. Complexity Hotspots

Sourcegraph ranks by reference count (PageRank analog). Parseltongue can offer:

- **Cyclomatic complexity**: How many code paths
- **Nesting depth**: How deeply nested
- **Function length**: Lines of code
- **Parameter count**: How many arguments
- **Cognitive complexity**: Human readability score

**Action**: Add `/api/complexity-hotspots` endpoint returning top-N most complex entities.

---

### 3. Graph-Native Queries

CozoDB Datalog can express queries that would be complex SQL:

```datalog
// Find all cycles in dependency graph
?[cycle_member] <-
    ?[a, b] := *depends_on{dependent: a, dependency: b},
    ?[b, a] := *depends_on{dependent: b, dependency: a}
;

// Find shortest path between modules
?[path] := path[start, end, path] <-
    algs.shortest_path_len[]
;
```

**Action**: Document and expose graph query endpoints clearly.

---

## Implementation Priority

| Priority | Feature | Effort | Impact |
|----------|---------|--------|--------|
| P0 | RRF for context ranking | 1 day | High |
| P0 | Source provenance | 1 day | Medium |
| P1 | Jaccard with stemming | 2 days | High |
| P1 | Semantic chunking | 1 day | Medium |
| P1 | cl100k token counting | 0.5 days | Medium |
| P2 | File exclusion patterns | 0.5 days | Low |
| P2 | Auto-generated detection | 0.5 days | Low |
| P2 | Async job pattern | 2 days | Medium |
| P2 | Two-tier token budgets | 0.5 days | Low |

---

## API Design Recommendations

### Enhanced Context Response

```json
{
  "query": "authentication",
  "context": [
    {
      "entity": {
        "name": "login",
        "type": "function",
        "file_path": "src/auth.rs",
        "line_start": 45,
        "line_end": 89,
        "signature": "pub fn login(user: &str, pass: &str) -> Result<Token, Error>"
      },
      "source": "search",
      "confidence": 0.92,
      "distance": null
    },
    {
      "entity": { "name": "AuthProvider", "type": "trait", ... },
      "source": "blast_radius",
      "confidence": 0.85,
      "distance": 2
    }
  ],
  "stats": {
    "total_tokens": 2847,
    "retrievers": [
      { "name": "embedding", "suggested": 15, "retrieved": 8, "duration_ms": 45 },
      { "name": "graph", "suggested": 5, "retrieved": 3, "duration_ms": 12 }
    ]
  }
}
```

---

## References

- `sourcegraph-public-snapshot/` - Cloned to `competitor-research/`
- `01-AMP-SOURCEGRAPH.md` - Detailed 1219-line analysis
- Parseltongue v1.7.2 - https://github.com/that-in-rust/parseltongue-rust-LLM-companion/releases/tag/v1.7.2
