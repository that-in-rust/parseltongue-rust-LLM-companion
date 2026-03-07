# Deep Research: Entity Search & Context Retrieval for LLM Code Understanding

**Research Date**: 2026-02-28
**Sources**: Warp deconstruction, Sourcegraph analysis, rust-analyzer patterns, dependency graph research

---

## The Problem Statement

When an LLM needs to understand code, the workflow is:
1. Find top-N candidates via some search method
2. Map candidates to precise **entities** (interface-level units)
3. Return entity context (file_path, file_name, line_start, line_end) + dependency graph

The question: **How do you efficiently search and retrieve the right entity context?**

---

## What This Repository Shows Works

### 1. Entity Boundary Format (Already Partially Solved)

From `idiomatic-patterns/rust-analyzer/`:
```markdown
**File:** `crates/hir-ty/src/db.rs` (lines 30-216)
**Category:** Salsa/Query-Based Architecture
**Description:** [pattern description]
**Code Example:**
```rust
// actual code
```
```

This format captures:
- File path
- Line range (start, end)
- Entity type/category
- Rich context

### 2. Dependency Graph Structure

From `rust-analyzer-analysis/internal-crate-dependency-graph.json`:
```json
{
  "forward_dependencies": {
    "hir-def": ["base-db", "cfg", "edition", "hir-expand", ...],
    "hir-ty": ["base-db", "edition", "hir-def", ...]
  },
  "reverse_dependencies": {
    "base-db": ["hir", "hir-def", "hir-expand", "hir-ty", ...],
    "syntax": ["base-db", "cfg", "hir", "hir-def", ...]
  }
}
```

Entity-level graphs would extend this to function/struct/trait granularity.

### 3. Entity Metadata Schema

From `rust-analyzer-analysis/crate-analysis.json`:
```json
{
  "hir-ty": {
    "struct_count": 250,
    "enum_count": 116,
    "trait_count": 21,
    "function_count": 244,
    "method_count": 2051,
    "impl_count": 183,
    "module_count": 81,
    "total_entities": 814
  }
}
```

---

## Search Methods Comparison

| Method | Best For | Speed | Precision | Index Cost |
|--------|----------|-------|-----------|------------|
| **Grep/Ripgrep** | Exact names, patterns | Fastest | High (exact) | None |
| **Vector/Cosine** | Semantic queries | Medium | Medium | High |
| **SCIP/LSIF** | Definitions + references | Slow index | Highest | Very High |
| **Tree-sitter AST** | Entity boundaries | Fast | High | Low |
| **Hybrid** | Best overall | Varies | Best | Medium-High |

### Is Cosine/Vector Search Worth It?

**YES for:**
- Semantic discovery ("find authentication code")
- Exploratory queries ("how does error handling work")
- Finding related but differently-named concepts

**NO for:**
- Exact name lookup ("find `login` function")
- Single-file/small codebase scenarios
- Real-time queries without pre-indexing

**Key insight from Sourcegraph analysis**: They use embeddings for initial retrieval but **rerank** using:
- PageRank-like scoring on SCIP reference counts
- Dependency graph proximity
- BM25 keyword overlap

---

## Indexing Large Codebases: Is It Worth It?

From Warp's Merkle tree indexing analysis:

### The Problem
- Full re-indexing is expensive
- Large repos take hours
- Need persistence across sessions

### The Solution: Merkle Tree Snapshots
```
┌─────────────────────────────────────────────┐
│  Merkle Tree Index                          │
├─────────────────────────────────────────────┤
│  Leaf nodes: Code fragments with hashes     │
│  Intermediate: Aggregated child hashes      │
│  Root: Single hash for entire repo state    │
└─────────────────────────────────────────────┘
         ↓ Incremental Update ↓
┌─────────────────────────────────────────────┐
│  File Change Detected                       │
│  → Re-fragment changed files only           │
│  → Update affected leaf hashes              │
│  → Propagate up to root                     │
│  → Sync only changed nodes                  │
└─────────────────────────────────────────────┘
```

### When Indexing IS Worth It
- Multi-session use (amortize cost)
- Large codebase (>10K files)
- Semantic/ exploratory queries
- Team shared index

### When Indexing Is NOT Worth It
- Single one-off query
- Small codebase (<1K files)
- Exact name searches only
- No persistence needed

---

## What ChromaDB Does

```
Code Entity → Embedding Model → Vector [768-1536 dims]
                                   ↓
                          ChromaDB stores + indexes
                          with metadata:
                          {
                            file_path: "src/auth.rs",
                            line_start: 45,
                            line_end: 89,
                            entity_type: "function",
                            entity_name: "login"
                          }
                                   ↓
Query → Embedding → ANN search → Top-K candidates
```

**Limitation**: Returns similar vectors, not necessarily correct entities. Needs reranking.

---

## Recommended Architecture

### Indexing Phase
```
┌──────────────────────────────────────────────────────────┐
│  1. Tree-sitter Parse                                    │
│     → Extract entities (name, type, file, line_start,    │
│       line_end, docstring, signature)                    │
│                                                          │
│  2. Dependency Graph Builder                             │
│     → Who calls who (call graph)                         │
│     → Who imports who (import graph)                     │
│     → Who implements who (type graph)                    │
│                                                          │
│  3. Embedding Generation                                 │
│     → Entity signature + docs → embedding model          │
│     → Store in ChromaDB with metadata                    │
│                                                          │
│  4. Merkle Tree Persistence                              │
│     → Hash-based snapshots                               │
│     → Incremental updates on file change                 │
└──────────────────────────────────────────────────────────┘
```

### Query Phase
```
┌──────────────────────────────────────────────────────────┐
│  1. Initial Retrieval                                    │
│     Query → Vector search (ChromaDB) → Top 20 candidates │
│                                                          │
│  2. Reranking (Multi-signal)                             │
│     - Vector similarity score                            │
│     - Dependency graph proximity (in call path?)         │
│     - Entity type matching (interface vs impl)           │
│     - BM25 keyword overlap                               │
│     - Reference count (popularity)                       │
│                                                          │
│  3. Entity Selection                                     │
│     → Return top 3 with (file, line_start, line_end)     │
│                                                          │
│  4. Context Assembly                                     │
│     → Read file at line range                            │
│     → Include related entities from graph (max 3-5)      │
│     → Return compact context for LLM                     │
└──────────────────────────────────────────────────────────┘
```

---

## What to Return to the LLM

### DO Return
```json
{
  "entity": {
    "name": "mir_body_query",
    "type": "function",
    "file_path": "crates/hir-ty/src/db.rs",
    "line_start": 30,
    "line_end": 50,
    "content": "// actual code from lines 30-50",
    "docstring": "Computes MIR body for a definition..."
  },
  "dependencies": {
    "calls": ["const_eval_discriminant_variant"],
    "called_by": ["infer_query", "monomorphization"],
    "implements": ["HirDatabase trait"]
  },
  "related_entities": [
    {"name": "MirBody", "type": "struct", "file": "...", "lines": "..."}
  ]
}
```

### DO NOT Return
- Entire file contents (waste of tokens)
- Unrelated code regions
- Full dependency graph (too large)
- Binary/non-code files

---

## Implementation Roadmap

### Phase 1: Entity Extractor (Tree-sitter)
```python
# Pseudo-code
def extract_entities(file_path, language):
    tree = tree_sitter.parse(read_file(file_path))
    entities = []
    for node in tree.root_node.children:
        if node.type in ['function_definition', 'struct_item', 'trait_item']:
            entities.append({
                'name': extract_name(node),
                'type': node.type,
                'file_path': file_path,
                'line_start': node.start_point.row,
                'line_end': node.end_point.row,
                'signature': extract_signature(node),
                'docstring': extract_docstring(node)
            })
    return entities
```

### Phase 2: Simple Vector Store
```python
import chromadb

client = chromadb.Client()
collection = client.create_collection("code_entities")

def index_entities(entities):
    for entity in entities:
        embedding = embed(entity['signature'] + ' ' + entity.get('docstring', ''))
        collection.add(
            ids=[entity['file_path'] + ':' + entity['name']],
            embeddings=[embedding],
            metadatas=[entity]
        )
```

### Phase 3: Dependency Graph
```python
# Use LSP or tree-sitter for call references
def build_call_graph(entities):
    graph = {}
    for entity in entities:
        if entity['type'] == 'function':
            calls = extract_function_calls(entity['content'])
            graph[entity['name']] = calls
    return graph
```

### Phase 4: Hybrid Reranker
```python
def rerank(candidates, query, graph):
    scores = []
    for candidate in candidates:
        score = 0.0
        score += 0.4 * cosine_similarity(query_embedding, candidate_embedding)
        score += 0.3 * graph_proximity(query_entity, candidate, graph)
        score += 0.2 * bm25_score(query, candidate['signature'])
        score += 0.1 * entity_type_match(query_intent, candidate['type'])
        scores.append((candidate, score))
    return sorted(scores, key=lambda x: x[1], reverse=True)[:3]
```

---

## Key Takeaways

1. **Entity extraction is tractable**: Tree-sitter gives you line boundaries for free
2. **Vector search alone isn't enough**: Need multi-signal reranking
3. **Dependency graphs are valuable**: Proximity in call graph = relevance
4. **Incremental indexing matters**: Merkle trees enable efficient updates
5. **Return compact context**: Entity + metadata + related, not whole files
6. **Hybrid beats pure**: Combine vector + keyword + graph signals

---

## References from This Repository

- `idiomatic-patterns/rust-analyzer/` - Entity format examples with line ranges
- `rust-analyzer-analysis/internal-crate-dependency-graph.json` - Dependency graph structure
- `rust-analyzer-analysis/crate-analysis.json` - Entity counting schema
- `warp-deconstruction-2026/11-binary-deconstruction/02-CODEBASE-INDEXING-SYSTEM.md` - Merkle tree indexing
- `parseltongue-competitive-intel-2026/01-AMP-SOURCEGRAPH.md` - SCIP indexing, reranking strategies
- `droid-deconstruction-2026/REAL-03-AI-CAPABILITIES.md` - Context management in AI agents
