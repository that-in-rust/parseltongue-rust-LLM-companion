# Parseltongue Visual Roadmap: v1.4.2 → v1.9

**Created**: 2026-01-31
**Status**: Strategic Planning Document

---

## The Evolution Timeline

```
┌──────────────────────────────────────────────────────────────────────────┐
│                   PARSELTONGUE AGENT MEMORY ROADMAP                      │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  v1.4.2 (NOW)          v1.6 (Q2)           v1.7 (Q2)          v1.8 (Q3) │
│  ─────────             ────────            ────────           ──────── │
│  ┌─────────┐          ┌─────────┐        ┌─────────┐       ┌─────────┐ │
│  │ HTTP    │          │ MCP     │        │ Preview │       │ Subgraph│ │
│  │ Only    │  ───►    │ Protocol│  ───► │ Pointers│ ───►  │ Export  │ │
│  │         │          │         │        │         │       │         │ │
│  │ 14 REST │          │ Unix    │        │ Token   │       │ Hot Path│ │
│  │ Endpoints│          │ Piping  │        │ Budget  │       │ Cache   │ │
│  │         │          │         │        │         │       │         │ │
│  │ No Agent│          │ SSE     │        │ Stateful│       │ Pipeline│ │
│  │ Protocol│          │ Stream  │        │ Cursors │       │ Compose │ │
│  └─────────┘          └─────────┘        └─────────┘       └─────────┘ │
│      │                    │                   │                 │       │
│      │                    │                   │                 │       │
│      └────────────────────┴───────────────────┴─────────────────┘       │
│                                  │                                      │
│                                  ▼                                      │
│                           ┌─────────────┐                               │
│                           │   v1.9      │                               │
│                           │   (Q4)      │                               │
│                           ├─────────────┤                               │
│                           │ Budget-Aware│                               │
│                           │ Query       │                               │
│                           │ Planner     │                               │
│                           │             │                               │
│                           │ 200K → ∞    │                               │
│                           └─────────────┘                               │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Feature Comparison Matrix

### Agent Memory Capabilities: v1.4.2 vs. v1.9

| Capability | v1.4.2 (Now) | v1.6 | v1.7 | v1.8 | v1.9 |
|------------|--------------|------|------|------|------|
| **Protocol** | HTTP only | ✅ MCP + HTTP | ✅ MCP + HTTP | ✅ MCP + HTTP | ✅ MCP + HTTP |
| **Response Format** | JSON only | ✅ JSON/NDJSON/TSV | ✅ + Tiered (preview/pointer/full) | ✅ + Export format | ✅ + Budget-optimized |
| **Streaming** | Blocking | ✅ SSE streaming | ✅ SSE + Pagination cursors | ✅ + Chunked export | ✅ + Auto-optimized streams |
| **Composability** | None | ✅ Unix pipes | ✅ Unix pipes | ✅ + Query pipelines | ✅ + Pipeline optimization |
| **Multi-Workspace** | Single DB | ✅ Multi-DB query | ✅ Multi-DB query | ✅ + Subgraph merge | ✅ + Cross-workspace planning |
| **Token Management** | None | None | ✅ Estimation + Warnings | ✅ + Dry-run mode | ✅ Auto-optimization |
| **Caching** | None | None | ✅ Stateful cursors | ✅ Hot path cache | ✅ + Budget-aware cache |
| **Local Execution** | Remote only | Remote only | Remote only | ✅ Subgraph export | ✅ + Smart export scopes |
| **Self-Correction** | None | None | ✅ Token warnings | ✅ + Suggestions | ✅ Auto-rewriting |
| **Agent UX** | Manual HTTP | ✅ MCP discovery | ✅ + Preview mode | ✅ + Cached queries | ✅ Invisible optimization |

**Legend**:
- ❌ = Not available
- ✅ = Available

---

## Token Efficiency Evolution

### How Query Token Costs Change Across Versions

```
┌────────────────────────────────────────────────────────────────┐
│        TOKEN COST: "List all functions in auth module"        │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  v1.4.2: Full details, no optimization                        │
│  ┌──────────────────────────────────────────────────────┐    │
│  │                                                        │    │
│  │  500 entities × 30 tokens each = 15,000 tokens       │    │
│  │                                                        │    │
│  │  (100% baseline)                                      │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                                │
│  v1.7: Preview mode (signatures only)                         │
│  ┌─────────────────────┐                                      │
│  │                     │                                      │
│  │  500 × 2 = 1,000   │                                      │
│  │                     │                                      │
│  │  (6.7% of baseline) │                                      │
│  └─────────────────────┘                                      │
│                                                                │
│  v1.9: Budget-aware planner (20K token budget)                │
│  ┌─────────────────────────────┐                              │
│  │                             │                              │
│  │  Auto-optimized:            │                              │
│  │  - Preview mode             │                              │
│  │  - Limit to 200 entities    │                              │
│  │  - Ranked by relevance      │                              │
│  │                             │                              │
│  │  Result: 18,500 tokens      │                              │
│  │  (Fits budget + relevant)   │                              │
│  │                             │                              │
│  └─────────────────────────────┘                              │
│                                                                │
└────────────────────────────────────────────────────────────────┘

TOKEN SAVINGS: v1.4.2 → v1.7 = 93.3% reduction
SMART FITTING: v1.9 auto-fits 20K budget with relevance ranking
```

---

## Arize Pattern Application

### How Each v1.7-v1.9 Feature Maps to Arize Principles

```
┌─────────────────────────────────────────────────────────────────────┐
│                   ARIZE PATTERN → PARSELTONGUE PRD                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. PREVIEW/POINTER (Alyx truncated tables)                        │
│     ┌────────────────┐                                             │
│     │ File System    │  Show filename + first 5 lines + file ID    │
│     └────────────────┘                                             │
│     ┌────────────────┐                                             │
│     │ Parseltongue   │  Show signature + SigHash → full on demand  │
│     │ ISG            │  ✅ PRD #1: Entity Preview Signature Pointers│
│     └────────────────┘                                             │
│                                                                     │
│  2. SELF-CORRECTION (Claude detecting context overflow)            │
│     ┌────────────────┐                                             │
│     │ Generic Agent  │  "Results too large, backtrack"             │
│     └────────────────┘                                             │
│     ┌────────────────┐                                             │
│     │ Parseltongue   │  "Query = 105K tokens, suggest filter"      │
│     │ ISG            │  ✅ PRD #2: Query Token Budget Estimator     │
│     └────────────────┘                                             │
│                                                                     │
│  3. PAGING RESULTS (Cursor stepping incrementally)                 │
│     ┌────────────────┐                                             │
│     │ File System    │  Page 1/10 of file list                     │
│     └────────────────┘                                             │
│     ┌────────────────┐                                             │
│     │ Parseltongue   │  Entity set 1/N with graph continuity       │
│     │ ISG            │  ✅ PRD #3: Stateful Query Pagination        │
│     └────────────────┘                                             │
│                                                                     │
│  4. SQL vs FILE SYSTEM (Remote query vs. local processing)         │
│     ┌────────────────┐                                             │
│     │ File System    │  Remote DB query OR download files          │
│     └────────────────┘                                             │
│     ┌────────────────┐                                             │
│     │ Parseltongue   │  In-DB graph query OR export subgraph JSON  │
│     │ ISG            │  ✅ PRD #4: Subgraph Export Local Execution  │
│     └────────────────┘                                             │
│                                                                     │
│  5. DYNAMIC INDEXING (Unix `find` creating indexes at runtime)     │
│     ┌────────────────┐                                             │
│     │ File System    │  `find` creates temp file list              │
│     └────────────────┘                                             │
│     ┌────────────────┐                                             │
│     │ Parseltongue   │  Session-scoped hot path cache              │
│     │ ISG            │  ✅ PRD #5: Session Hot Path Cache           │
│     └────────────────┘                                             │
│                                                                     │
│  6. COMPOSABLE CHAINS (grep | sort | uniq)                         │
│     ┌────────────────┐                                             │
│     │ File System    │  Pipe text between Unix commands            │
│     └────────────────┘                                             │
│     ┌────────────────┐                                             │
│     │ Parseltongue   │  Pipe graph operations server-side          │
│     │ ISG            │  ✅ PRD #6: ISG Query Composition Pipeline   │
│     └────────────────┘                                             │
│                                                                     │
│  7. BUDGET MANAGEMENT (200K context → infinite memory feel)        │
│     ┌────────────────┐                                             │
│     │ File System    │  Estimate file sizes, manual filtering      │
│     └────────────────┘                                             │
│     ┌────────────────┐                                             │
│     │ Parseltongue   │  Auto-optimize queries to fit budget        │
│     │ ISG            │  ✅ PRD #7: Budget Aware Query Planner       │
│     └────────────────┘                                             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Version-by-Version Effort Breakdown

### Time Investment and Deliverables

```
┌──────────────────────────────────────────────────────────────────────┐
│                        IMPLEMENTATION TIMELINE                       │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Q2 2026: v1.6 Foundation (6.5-7.5 weeks)                           │
│  ┌────────────────────────────────────────────────────────┐         │
│  │ Week 1-3:   MCP Server Core (P0)                       │         │
│  │ Week 4:     Unix Piping Output (P1)                    │         │
│  │ Week 5-6:   SSE Streaming API (P1)                     │         │
│  │ Week 7-8:   Multi-Workspace Multiplexing (P1)          │         │
│  └────────────────────────────────────────────────────────┘         │
│  Deliverable: Agent-native protocol, composability, streaming       │
│                                                                      │
│  ─────────────────────────────────────────────────────────          │
│                                                                      │
│  Q2 2026: v1.7 Memory Foundation (4.5 weeks)                        │
│  ┌────────────────────────────────────────────────────────┐         │
│  │ Week 1-1.5: Preview Signature Pointers (P1)            │         │
│  │ Week 2-3:   Token Budget Estimator (P1)                │         │
│  │ Week 3.5-5.5: Pagination Cursors (P1)                  │         │
│  └────────────────────────────────────────────────────────┘         │
│  Deliverable: 90% token reduction, self-correction, stateful paging │
│                                                                      │
│  ─────────────────────────────────────────────────────────          │
│                                                                      │
│  Q3 2026: v1.8 Advanced Patterns (5.5 weeks)                        │
│  ┌────────────────────────────────────────────────────────┐         │
│  │ Week 6-8:   Subgraph Export (P2)                       │         │
│  │ Week 8.5-10: Hot Path Cache (P2)                       │         │
│  │ Week 10.5-12.5: Query Composition Pipeline (P2)        │         │
│  └────────────────────────────────────────────────────────┘         │
│  Deliverable: Local execution, 10-50× speedups, composability       │
│                                                                      │
│  ─────────────────────────────────────────────────────────          │
│                                                                      │
│  Q4 2026: v1.9 Intelligence (2 weeks)                               │
│  ┌────────────────────────────────────────────────────────┐         │
│  │ Week 13-15: Budget-Aware Planner (P1)                  │         │
│  └────────────────────────────────────────────────────────┘         │
│  Deliverable: Auto-optimization, 200K → infinite memory feel        │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘

TOTAL EFFORT: v1.6 → v1.9 = 18.5-20.5 weeks (Q2-Q4 2026)
```

---

## Agent Workflow Transformation

### Before/After Comparison: Real Use Case

**Scenario**: Agent refactoring authentication module in 100K LOC codebase

#### v1.4.2 (Current State)
```
Agent Task: "What functions call authenticate()? Show me the top 10 by coupling."

Step 1: Manual HTTP query
  curl http://localhost:7777/reverse-callers-query-graph?entity=rust:fn:authenticate
  ⏱️ 350ms
  📊 Returns: 127 callers, full details
  💾 Token cost: 3,800 tokens

Step 2: Agent processes in-memory
  - Filters out test functions
  - Calculates coupling scores
  - Sorts by score
  - Takes top 10
  💭 Agent context consumed: 3,800 tokens

Step 3: Agent asks for blast radius
  curl http://localhost:7777/blast-radius-impact-analysis?entity=rust:fn:authenticate&hops=2
  ⏱️ 420ms
  📊 Returns: 500 affected entities, full details
  💾 Token cost: 15,000 tokens

Step 4: Agent realizes context overflow
  - Total consumed: 18,800 tokens
  - Remaining budget: Low
  - Agent retries with manual filtering
  - Wastes 2 seconds of compute

Total Time: ~2 seconds
Total Tokens: 18,800 (wasteful)
Agent UX: Poor (manual optimization required)
```

#### v1.9 (Future State)
```
Agent Task: "What functions call authenticate()? Show me the top 10 by coupling."

MCP Tool Invocation (Auto-Selected):
  Tool: parseltongue.reverse_callers_query_graph
  Params: {
    entity: "rust:fn:authenticate",
    detail: "preview",           # Auto-selected based on agent context budget
    token_budget: 5000,           # Agent passes remaining budget
    rank_by: "coupling_score",    # Composable pipeline
    limit: 10                     # Agent's constraint
  }

Parseltongue v1.9 Processing:
  1. Budget-Aware Planner activates
     - Estimates: 127 entities × 30 tokens = 3,810 (exceeds reasonable size)
     - Optimization: Switch to preview mode (127 × 2 = 254 tokens)
     - Optimization: Apply ranking + limit server-side (10 × 2 = 20 tokens)

  2. Session Hot Path Cache check
     - authenticate() already queried 30s ago
     - Cache hit: 8ms response time (vs. 350ms)

  3. Query Pipeline Composition
     - reverse_callers | rank_by_coupling | limit_10
     - Single efficient Datalog query

  4. Preview Mode Response
     - Returns: Top 10 callers, signatures only
     - Token cost: 20 tokens (vs. 3,800)

  ⏱️ 8ms (44× faster)
  💾 20 tokens (190× more efficient)

Agent receives result:
  - Fits comfortably in context
  - Response includes: "Optimization applied: preview mode + ranking"
  - Suggestion: "For full implementation, use entity_full_body_retrieve/{sighash}"

Agent selects 2 interesting entities for deeper analysis:
  Tool: parseltongue.entity_full_body_retrieve
  Params: {sighash: "blake3_abc123"}

  ⏱️ 3ms (cache hit)
  💾 45 tokens (1 full entity)

Total Time: ~20ms (100× faster)
Total Tokens: 110 (171× more efficient)
Agent UX: Excellent (invisible optimization, instant feedback)
```

---

## Strategic Moat Analysis

### Why Parseltongue's ISG Beats File Systems

```
┌─────────────────────────────────────────────────────────────────┐
│             COMPETITIVE DIFFERENTIATION: ISG vs FILES           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Cursor/Claude/Alyx                Parseltongue ISG             │
│  (Generic File System)             (Structured Graph)           │
│  ───────────────────               ──────────────────           │
│                                                                 │
│  FILE: auth.rs                     ENTITY: rust:fn:authenticate │
│  ├─ Line 1-500                     ├─ Signature: ✅ Known       │
│  ├─ Contains "function"?           ├─ Type: Function            │
│  ├─ First 5 lines = preview        ├─ Parameters: Request       │
│  ├─ File size ≈ tokens             ├─ Returns: Response         │
│  └─ Dependencies: Unknown          ├─ Callers: ✅ Indexed       │
│                                    ├─ Callees: ✅ Indexed       │
│                                    ├─ Coupling: ✅ Computed     │
│                                    ├─ Token cost: ✅ Exact      │
│                                    └─ SigHash: ✅ Pointer       │
│                                                                 │
│  PREVIEW: "Show first 5 lines"     PREVIEW: "Show signature"   │
│  Problem: Incomplete semantic      Solution: Complete interface│
│           context                            definition         │
│                                                                 │
│  TOKEN ESTIMATION:                 TOKEN ESTIMATION:           │
│  Heuristic: 4 chars ≈ 1 token      Exact: Pre-computed from    │
│  Accuracy: ±30%                            entity metadata      │
│                                    Accuracy: ±5%                │
│                                                                 │
│  CACHING:                          CACHING:                    │
│  File content cache                Entity + relationship cache │
│  Cache key: File path              Cache key: ISGL1 key +      │
│  Invalidation: File modified               signature hash       │
│                                    Invalidation: Entity changed │
│                                                                 │
│  COMPOSITION:                      COMPOSITION:                │
│  grep | awk | sort                 search | filter | traverse  │
│  Type: Text pipes                          | rank | limit      │
│  Safety: None                      Type: Graph-aware pipelines │
│                                    Safety: Type-checked         │
│                                                                 │
│  BUDGET OPTIMIZATION:              BUDGET OPTIMIZATION:        │
│  "File too large, truncate"        "Query exceeds budget:      │
│  Strategy: Size-based                   - Preview mode         │
│                                         - Limit results         │
│                                         - Rank by relevance"    │
│                                    Strategy: Semantic-aware     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

KEY INSIGHT: Files are bags of text. ISG entities are typed, connected,
             semantic units. Every Arize pattern works better on graphs.
```

---

## The Meta-Narrative

### From v1.4.2 to v1.9: Transformation Journey

```
┌────────────────────────────────────────────────────────────────┐
│                 PARSELTONGUE EVOLUTION STORY                   │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  Act 1: The Foundation (v1.4.2)                               │
│  ─────────────────────────────                                │
│  "We have a deterministic code graph that reduces            │
│   tokens by 99% and queries 31× faster than grep."           │
│                                                                │
│  Problem: Powerful backend, no agent-native interface         │
│                                                                │
│  ──────────────────────────────────────────────────────       │
│                                                                │
│  Act 2: The Protocol (v1.6)                                   │
│  ───────────────────────                                      │
│  "We expose our capabilities through MCP, Unix pipes,         │
│   and streaming. Agents can discover and invoke us."          │
│                                                                │
│  Achievement: Agent accessibility unlocked                     │
│  Gap: Still verbose, no token awareness, no optimization      │
│                                                                │
│  ──────────────────────────────────────────────────────       │
│                                                                │
│  Act 3: The Intelligence (v1.7-v1.9)                          │
│  ────────────────────────────────                             │
│  "We apply Arize memory patterns to structured graphs,        │
│   not files. Preview/pointer, budget management,              │
│   self-correction, compositional queries."                    │
│                                                                │
│  Result:                                                       │
│  - 90% token reduction (preview mode)                         │
│  - Auto-optimization (budget planner)                         │
│  - 10-50× speedups (hot path cache)                           │
│  - Composable queries (pipeline system)                       │
│  - 200K context → infinite memory feel                        │
│                                                                │
│  ──────────────────────────────────────────────────────       │
│                                                                │
│  The Outcome: L2 Cache for Agent Code Memory                  │
│  ──────────────────────────────────────────                   │
│  "Agents read 1% of codebase, understand 100% of              │
│   architecture. Deterministic facts, zero hallucinations,     │
│   budget-aware delivery, compositional reasoning."            │
│                                                                │
│  Competitive Moat:                                             │
│  - File systems: Unstructured (Cursor/Claude/Alyx)            │
│  - Parseltongue ISG: Structured graph (typed, indexed)        │
│                                                                │
│  Winner: Graph-aware patterns compound advantages              │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

---

## Decision Framework

### When to Implement Which Features

```
┌─────────────────────────────────────────────────────────────┐
│               FEATURE PRIORITIZATION MATRIX                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                    HIGH IMPACT                              │
│                         │                                   │
│                         │                                   │
│         ┌───────────────┼───────────────┐                  │
│         │               │               │                  │
│         │  v1.6: MCP    │  v1.7:        │                  │
│         │  Protocol     │  Preview Mode │                  │
│         │               │  Budget Est   │                  │
│    HIGH │  P0           │  P1           │ LOW              │
│    EFFORT                                  EFFORT           │
│         │               │               │                  │
│         │  v1.8:        │  v1.9:        │                  │
│         │  Subgraph     │  Budget       │                  │
│         │  Export       │  Planner      │                  │
│         │  Pipeline     │               │                  │
│         │  P2           │  P1           │                  │
│         └───────────────┼───────────────┘                  │
│                         │                                   │
│                         │                                   │
│                    LOW IMPACT                               │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  RECOMMENDED SEQUENCE:                                      │
│                                                             │
│  1. v1.6 (P0): MCP Protocol                                │
│     Why: Foundational, unlocks entire ecosystem            │
│                                                             │
│  2. v1.7 (P1): Preview + Budget                            │
│     Why: High impact, low effort, immediate value          │
│                                                             │
│  3. v1.9 (P1): Budget Planner                              │
│     Why: Synthesizes all prior features, capstone          │
│                                                             │
│  4. v1.8 (P2): Advanced Patterns                           │
│     Why: Power user features, incremental improvements     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Success Metrics

### How to Measure v1.7-v1.9 Impact

```
┌──────────────────────────────────────────────────────────────┐
│                    KPI TRACKING BY VERSION                   │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  v1.7 METRICS (Memory Foundation)                           │
│  ────────────────────────────────                           │
│  ✓ Token Reduction:     90% (preview vs. full)              │
│  ✓ Estimation Accuracy: ±15% of actual                      │
│  ✓ Cursor Performance:  <100ms to create/fetch              │
│  ✓ Agent Context Saved: 10-15K tokens per query             │
│  ✓ Zero Context Overflow: <1% of queries                    │
│                                                              │
│  v1.8 METRICS (Advanced Patterns)                           │
│  ─────────────────────────────                              │
│  ✓ Export Performance:  <5s for 1K entity subgraph          │
│  ✓ Cache Hit Rate:      >70% on repeated queries            │
│  ✓ Cache Speedup:       10-50× vs. cold query               │
│  ✓ Pipeline Efficiency: 1 query vs. N HTTP round-trips      │
│  ✓ Composition Adoption: >50% of queries use pipelines      │
│                                                              │
│  v1.9 METRICS (Intelligence)                                │
│  ────────────────────────                                   │
│  ✓ Auto-Optimization:   >80% of queries optimized           │
│  ✓ Budget Compliance:   ±10% of target budget               │
│  ✓ Relevance Accuracy:  Top 10 results match manual ranking │
│  ✓ Agent Satisfaction:  No manual optimization needed       │
│  ✓ Context Efficiency:  Agent uses <50% context on avg      │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## Document Status

**Created**: 2026-01-31
**Version**: 1.0
**Purpose**: Visual roadmap for v1.4.2 → v1.9 evolution

**Next Steps**:
1. Review roadmap with team
2. Commit to v1.6 scope (P0+P1)
3. Begin v1.7 planning after v1.6 delivery
4. Track metrics throughout implementation

---

**This document is part of PRD research series**:
- `01_ARCHITECTURE_OVERVIEW.md` - Current state analysis
- `02_V16_PRD_IDEAS_EXTRACTED.md` - v1.6 feature specs
- `03_V17_V19_ARIZE_PATTERNS_EXTRACTED.md` - v1.7-v1.9 feature specs
- `04_VISUAL_ROADMAP_V14_TO_V19.md` - This document
