# Embedded Code Graph Database Thesis

**Version:** 1.0.0
**Date:** 2026-03-09
**Author:** Research synthesis for Parseltongue v300
**Mindset:** Shreyas Doshi - Impact over Activity

---

# Executive Summary

## The Question

> Is the market needing a new embedded graphical database for code analysis?

## The Short Answer

**No. But the market needs embedded graph ALGORITHMS validated against trustworthy oracles.**

The database is solved. The algorithms are the gap.

---

# Part 1: What Exists Today

## 1.1 Embedded Graph Databases in Rust (2025)

| Database | Language | Query Lang | Code-Ready? | Status |
|----------|----------|------------|-------------|--------|
| **GraphLite** | Rust | ISO GQL | ⚠️ Generic | New 2025, "SQLite of graphs" |
| **sqlitegraph** | Rust | SQL-like | ⚠️ Generic | HNSW vector, algorithms built-in |
| **CozoDB** | Rust | Datalog | ⚠️ Generic | Current Parseltongue choice |
| **Kuzu** | C++ (Rust bindings) | Cypher | ⚠️ Generic | Production-ready, MIT |
| **HelixDB** | Rust | Custom | ⚠️ Generic | Vector + graph hybrid |

### Key Observation

**All are generic graph databases.** None are specialized for CODE. They store nodes/edges, not functions/modules/types with language-specific semantics.

## 1.2 Code-Specific Graph Systems

| System | Language | Database | Code-Specific? | Production? |
|--------|----------|----------|----------------|-------------|
| **Joern/CPG** | Scala | OverflowDB | ✅ Yes | Production |
| **Code-Graph-RAG** | Python | Memgraph | ✅ Yes | Open source |
| **codegraph crate** | Rust | Custom | ✅ Yes | v0.2.0 (new) |
| **Code Pathfinder** | Go | Custom | ✅ Yes | Open source |
| **Graphlr** | Java | Neo4j | ⚠️ Java-only | Production |

### Key Observation

**Code-specific systems exist but:**
- Joern is JVM-based, not embeddable in Rust
- Code-Graph-RAG is Python + Memgraph (server, not embedded)
- codegraph crate is new (v0.2.0, Nov 2025) - unproven
- Code Pathfinder is Go-based

## 1.3 Code Property Graph (CPG) Ecosystem

### Joern / ShiftLeft / Qwiet AI

**What it is:**
- Open source code analysis platform
- Code Property Graph = AST + CFG + DFG in one graph
- OverflowDB = specialized graph DB for CPGs

**Maturity:**
- Production since 2014
- Used by security researchers globally
- Supports: C/C++, Java, JavaScript, Python, Go, and more

**Problem for Parseltongue:**
- Scala-based, not Rust
- Requires JVM
- Not embeddable
- OverflowDB is Java, not Rust

---

# Part 2: Validation State

## 2.1 Algorithm Validation Landscape

| Algorithm Category | Who Validates? | Against What? |
|--------------------|---------------|---------------|
| BFS/DFS | petgraph | Unit tests |
| Shortest path | petgraph | NetworkX |
| SCC (Tarjan) | petgraph | NetworkX |
| PageRank | petgraph, rustworkx | NetworkX, igraph |
| k-core | rustworkx | NetworkX |
| Leiden/Louvain | graphrs | NetworkX |
| Betweenness centrality | rustworkx | NetworkX |
| Closeness centrality | rustworkx | NetworkX |

### The Validation Chain

```
NetworkX (Python) → Academic papers → Production truth
        ↑
        └── rustworkx (Rust) → Compares to NetworkX
        └── petgraph (Rust) → No external oracle, internal tests only
```

### Key Observation

**NetworkX IS the oracle.** The Python ecosystem has been the validation ground truth since 2004. Any Rust graph algorithm should test against NetworkX.

## 2.2 Current Parseltongue Validation Gap

From `docs/v300/algorithm_endpoint_journey_matrix_202603091430.md`:

| Algorithm | Implemented In | Validated? |
|-----------|---------------|------------|
| Blast radius (BFS) | Rust + CozoDB | ❌ No oracle |
| Centrality (PageRank) | Rust | ❌ No oracle |
| k-core decomposition | Rust | ❌ No oracle |
| Leiden community | Rust | ❌ No oracle |
| Tarjan SCC | Rust | ❌ No oracle |

**The Problem:** We wrote algorithms. We didn't validate them.

## 2.3 Trustworthy Algorithm Libraries

| Library | Stars | Ecosystem | Trust Level |
|---------|-------|-----------|-------------|
| **petgraph** | 3,773 | Rust standard | HIGH |
| **rustworkx-core** | N/A | Qiskit (IBM) | HIGH |
| **NetworkX** | 16,000+ | Python standard | ORACLE |
| **graphrs** | 100+ | Rust specialized | MEDIUM |
| **igraph** | 1,500+ | C/R/Python | HIGH |

### Recommendation Matrix

| Need | Use | Validate Against |
|------|-----|------------------|
| Graph structure | petgraph | - |
| BFS/DFS/Shortest path | petgraph | NetworkX |
| SCC | petgraph | NetworkX |
| PageRank | petgraph | NetworkX |
| Betweenness | rustworkx-core | NetworkX |
| k-core | rustworkx-core | NetworkX |
| Leiden/Louvain | graphrs | NetworkX |

---

# Part 3: Market Gap Analysis

## 3.1 The Shreyas Doshi Framework

### Jobs-to-be-Done

**What job is the user hiring a "code graph database" for?**

1. **Understanding:** "What does this code do?"
2. **Navigation:** "Where is X implemented?"
3. **Impact:** "What breaks if I change Y?"
4. **Security:** "Where are the vulnerabilities?"

### Current Solutions

| Job | Current Solution | Friction |
|-----|-----------------|----------|
| Understanding | IDE + grep + LLM dump | Too many tokens |
| Navigation | grep, IDE search | No graph context |
| Impact | Manual tracing, grep | Error-prone |
| Security | SAST tools | High false positives |

### The Parseltongue Job

**"Give me compiler-verified context for LLM decisions."**

This is NOT "store code as graph."
This is "extract TRUTH from compiler, surface it efficiently."

## 3.2 Market Gap: Not the Database

### What the Market HAS

- ✅ Embedded graph databases (GraphLite, CozoDB, Kuzu, sqlitegraph)
- ✅ Graph algorithm libraries (petgraph, rustworkx, NetworkX)
- ✅ Code parsing (tree-sitter, rustc_private)
- ✅ Code Property Graph concept (Joern/CPG)

### What the Market LACKS

- ❌ **Embedded Rust code graph with compiler truth**
- ❌ **Validated algorithms specific to code analysis**
- ❌ **Graph algorithms tested against NetworkX oracle**
- ❌ **Span → public interface anchoring**
- ❌ **Candidate → cluster mapping for LLM decisions**

## 3.3 The Real Gap

```
                    PARSING          STORAGE          ALGORITHMS          SURFACE
                    
tree-sitter    ────► ???      ────► petgraph     ────► ???
rustc_private  ────► SQLite      ────► rustworkx    ────► 7-event journey
                    
     ✅ Solved         ⚠️ Solved       ✅ Solved           ❌ UNSOLVED
```

**The unsolved part is NOT the database. It's the SURFACE - how code graph meets LLM decisions.**

---

# Part 4: The Shreyas Doshi Thesis

## 4.1 Impact Over Activity

### Activity Trap

> "Build a new embedded graph database for code!"

This is activity. It feels productive. It's technically interesting. But is it impact?

### Impact Question

> "What outcomes matter for our users?"

**Outcomes:**
1. LLM makes correct decisions with fewer tokens
2. User understands code structure in <500ms
3. User trusts the output (compiler-verified)

**Do these outcomes require a new database?**

**No.** They require:
- Efficient storage (SQLite/libSQL ✅ exists)
- Validated algorithms (petgraph/rustworkx ✅ exists)
- Compiler truth extraction (rustc_private ✅ exists)
- The 7-event surface (❌ we build this)

## 4.2 The "Why Now" for Code Graph

### Market Trends 2025

1. **LLMs eating code comprehension** - Copilot, Cursor, Windsurf
2. **Context windows growing** - But token costs still matter
3. **RAG becoming standard** - But naive RAG misses structure
4. **Graph RAG emerging** - Memgraph, Neo4j pushing this

### The Window

**LLMs need structure.** They're drowning in tokens. The first tool that delivers:
- Structure (graph)
- Trust (compiler)
- Efficiency (<500ms)

...wins the LLM developer tooling market.

## 4.3 Positioning

### NOT a Graph Database Company

We are NOT competing with Neo4j, Memgraph, Kuzu, GraphLite.

### A Code Understanding Company

We ARE competing with:
- Sourcegraph (code search)
- Cursor (AI IDE)
- Aider (AI pair programmer)

**Our differentiation:**
- Sourcegraph: Search, no graph
- Cursor: AI, no compiler truth
- Aider: AI, no structured context

**Parseltongue: Compiler-verified graph context for AI decisions.**

---

# Part 5: Technical Recommendations

## 5.1 Do NOT Build a New Database

### Why Not

1. **SQLite/libSQL solves storage** - ACID, WAL, FTS5, one file
2. **GraphLite exists** - If we NEED embedded graph DB, use it
3. **Kuzu exists** - Cypher support, Rust bindings
4. **CozoDB works** - We already use it

### The Switch Decision

| Option | Effort | Risk | Reward |
|--------|--------|------|--------|
| Keep CozoDB | 0 | Low | Same |
| Switch to libSQL | Medium | Medium | Simpler architecture |
| Switch to GraphLite | Medium | Medium | ISO GQL standard |
| Build new DB | HIGH | HIGH | ??? |

**Recommendation:** Follow Decision 03 - Single libSQL/SQLite store. The database is not the moat.

## 5.2 DO Build Validated Algorithms

### What to Build

1. **Test fixtures** - Python scripts calling NetworkX
2. **Validation suite** - Compare our outputs to NetworkX
3. **Integration tests** - petgraph + rustworkx + NetworkX agreement

### The Validation Pipeline

```bash
# For each algorithm
cargo test --package parseltongue-core test_pagerank_validation
python3 scripts/validate_networkx.py --algorithm pagerank
# Compare outputs, fail if >0.01% difference
```

### Algorithm Library Choice

| Algorithm | Library | Reason |
|-----------|---------|--------|
| BFS/DFS | petgraph | Standard, fast |
| Shortest path | petgraph | Dijkstra built-in |
| SCC | petgraph | Tarjan built-in |
| PageRank | petgraph | Built-in, validated |
| Betweenness | rustworkx-core | Better implementation |
| k-core | rustworkx-core | Validated |
| Leiden | graphrs | Specialized |

## 5.3 DO Build the 7-Event Surface

### What to Build

This IS the moat. The 7-event journey:

```
Event 1: QUERY     →  LLM sends ~7 words
Event 2: SEARCH     →  RRF fusion finds 4 candidates (<10ms)
Event 3: ANCHOR     →  BFS to public interface (<50ms)
Event 4: CLUSTER    →  Ego network 1-hop (<100ms)
Event 5: ASK        →  Present 4 options (~200 tokens)
Event 6: CHOICE     →  LLM picks [1-4] or none
Event 7: DEEP DIVE  →  Full context up to 20k tokens (<500ms)
```

### Why This Is the Moat

- No competitor has this surface
- Requires: compiler truth + graph algorithms + LLM optimization
- Delivers: understanding, not just search results

---

# Part 6: Competitive Analysis

## 6.1 Embedded Graph DBs

### GraphLite (2025)

**Pros:**
- ISO GQL standard (future-proof)
- Pure Rust
- "SQLite of graphs" positioning
- ACID transactions

**Cons:**
- New (2025), unproven at scale
- Generic, not code-specific
- No code analysis examples

**Verdict:** Watch, don't adopt yet.

### sqlitegraph

**Pros:**
- HNSW vector search built-in
- SQLite backend (familiar)
- Algorithms included

**Cons:**
- Small community
- Generic, not code-specific

**Verdict:** Consider for hybrid graph+vector needs.

### Kuzu

**Pros:**
- Cypher support (industry standard)
- Production-ready
- Rust bindings exist
- Columnar, fast for analytics

**Cons:**
- C++ core (not pure Rust)
- Requires schema upfront

**Verdict:** Best option if we NEED a dedicated graph DB.

### CozoDB (Current)

**Pros:**
- Already integrated
- Datalog powerful for recursive queries
- Pure Rust

**Cons:**
- Datalog learning curve
- Not standardized (like Cypher/GQL)

**Verdict:** Works, but simpler architecture wins.

## 6.2 Code Graph Systems

### Joern/CPG

**Pros:**
- Most mature code graph system
- Security-focused (vulnerability detection)
- Multi-language
- Academic backing

**Cons:**
- JVM required
- Not embeddable in Rust
- OverflowDB is Java

**Verdict:** Learn from, don't integrate.

### codegraph crate (Rust)

**Pros:**
- Pure Rust
- Code-specific
- Multi-language parsers

**Cons:**
- v0.2.0 (very new)
- Unproven
- Small community

**Verdict:** Watch, may mature into useful option.

---

# Part 7: Final Recommendations

## 7.1 The Shreyas Doshi Decision

### What to Do

1. **Keep using SQLite/libSQL for storage** - Decision 03 is correct
2. **Adopt petgraph + rustworkx-core for algorithms** - Decision 04 is correct
3. **Validate all algorithms against NetworkX** - The missing piece
4. **Build the 7-event surface** - The real moat

### What NOT to Do

1. **Do NOT build a new graph database** - The market doesn't need it
2. **Do NOT write homegrown graph algorithms** - Use trusted libraries
3. **Do NOT try to compete with Neo4j/Memgraph** - Wrong market

## 7.2 The Thesis Statement

> **The market does not need another embedded graph database. The market needs embedded graph algorithms validated against trustworthy oracles, applied to code understanding.**
>
> **Parseltongue's moat is not the database. It's the compiler-to-LLM surface - the 7-event journey that turns code structure into LLM decisions.**

## 7.3 Implementation Priority

### Phase 1: Validation (Weeks 1-2)

- [ ] Create NetworkX validation fixtures
- [ ] Integrate petgraph for BFS/DFS/SCC/PageRank
- [ ] Integrate rustworkx-core for betweenness/k-core
- [ ] Run validation tests, ensure <0.01% deviation

### Phase 2: Architecture (Weeks 3-4)

- [ ] Implement libSQL schema from Decision 03
- [ ] Migrate entity/edge storage from CozoDB
- [ ] Add FTS5 for search
- [ ] Test Event 2 (search) performance

### Phase 3: Surface (Weeks 5-8)

- [ ] Implement Event 3 (anchoring)
- [ ] Implement Event 4 (clustering)
- [ ] Implement Event 5-7 (ask/choice/deep-dive)
- [ ] End-to-end latency test (<500ms)

### Phase 4: Compiler Integration (Weeks 9-12)

- [ ] rustc_private integration
- [ ] Type extraction
- [ ] Call edge extraction
- [ ] MIR/CFG for deep dive

---

# Appendix A: Search Results Summary

## Embedded Graph Databases Found

1. **GraphLite** - ISO GQL embedded graph DB in Rust (2025)
2. **sqlitegraph** - Embedded graph DB for Rust with HNSW
3. **Kuzu** - Embedded property graph DB with Cypher, Rust bindings
4. **CozoDB** - Embedded Datalog graph DB in Rust
5. **HelixDB** - Hybrid vector-graph DB in Rust

## Code Graph Systems Found

1. **Joern/CPG** - Code Property Graph analysis, OverflowDB
2. **Code-Graph-RAG** - Python + Memgraph for code
3. **codegraph crate** - Rust crate for code graph (v0.2.0, 2025)
4. **Code Pathfinder** - Go-based, AI-native static analysis
5. **Graphlr** - Java AST to Neo4j

## Algorithm Libraries Found

1. **petgraph** - 3,773 stars, Rust standard
2. **rustworkx-core** - From Qiskit, validated against NetworkX
3. **NetworkX** - Python standard, 16,000+ stars, THE ORACLE
4. **graphrs** - Rust graph algorithms
5. **igraph** - C/R/Python, highly optimized

---

# Appendix B: Algorithm Validation Matrix

| Algorithm | Parseltongue Current | Recommended Library | NetworkX Function |
|-----------|---------------------|---------------------|-------------------|
| BFS traversal | Custom | petgraph | `bfs_edges()` |
| DFS traversal | Custom | petgraph | `dfs_edges()` |
| Shortest path | Custom | petgraph | `shortest_path()` |
| SCC (Tarjan) | Custom | petgraph | `strongly_connected_components()` |
| PageRank | Custom | petgraph | `pagerank()` |
| Betweenness | Custom | rustworkx-core | `betweenness_centrality()` |
| Closeness | Custom | rustworkx-core | `closeness_centrality()` |
| k-core | Custom | rustworkx-core | `core_number()` |
| Leiden | Custom | graphrs | No direct equivalent |
| Louvain | Not implemented | graphrs | `louvain_communities()` |
| Eigenvector centrality | Not implemented | rustworkx-core | `eigenvector_centrality()` |
| Clustering coefficient | Not implemented | rustworkx-core | `clustering()` |

---

# Appendix C: References

## Papers & Articles

- "A comparative evaluation of social network analysis tools" (Springer, 2025)
- "CODEXGRAPH: Bridging LLMs and Graph Databases" (NAACL 2025)
- "rustworkx: A High-Performance Graph Library for Python" (arXiv, 2022)
- "BYO: A Unified Framework for Benchmarking Large-Scale Graph Containers" (arXiv, 2024)

## Repositories

- https://github.com/petgraph/petgraph
- https://github.com/Qiskit/rustworkx
- https://github.com/networkx/networkx
- https://github.com/kuzudb/kuzu
- https://github.com/cozodb/cozo
- https://github.com/joernio/joern

## Documentation

- https://docs.kuzudb.com/
- https://www.cozodb.org/
- https://networkx.org/documentation/stable/
- https://www.rustworkx.org/

---

**Document Status:** Complete
**Next Action:** Implement validation suite against NetworkX
**Owner:** Parseltongue v300 Architecture
