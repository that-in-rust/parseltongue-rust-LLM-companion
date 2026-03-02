# Code-Understanding Domain Thesis - Design Document

**Date:** 2026-03-02
**Status:** Approved
**Purpose:** Internal strategic guide for Parseltongue v2.0.0 roadmap and architecture decisions

---

## Executive Summary

This document defines the design for a comprehensive research thesis on the code-understanding domain. The thesis will inform key architectural decisions for Parseltongue v2.0.0, a Rust LLM companion tool for OSS contributors working on large repositories.

---

## Research Scope

### Core Insight

> Graphs are their own universe. Code is just one application. The alpha lies in discovering what graph theory can unlock for code understanding that nobody has done yet.

### Research Questions

1. **Graph Theory Landscape:** What algorithm categories exist in graph theory that could apply to code?
2. **Code-as-Graph Representations:** What are ALL the ways code can be represented as graphs?
3. **Intersection Reality:** What has been tried? What works? What failed?
4. **White Space:** What's missing that Parseltongue can pioneer?

---

## Research Phases

### Phase 0a: arXiv - Pure Graph Theory
**Goal:** Survey graph algorithm taxonomy independent of code

**Search Queries:**
- Graph algorithm taxonomy survey
- Graph theory applications survey
- Centrality algorithms survey
- Community detection algorithms
- Graph neural networks survey
- Graph mining techniques
- Knowledge graph algorithms
- Temporal graph analysis
- Multi-layer graph analysis

**Deliverables:**
- Comprehensive taxonomy of graph algorithm categories
- Cross-domain applications (bio, social, knowledge, fraud, etc.)
- Novel approaches not yet applied to code

---

### Phase 0b: arXiv - Code-as-Graph Representations
**Goal:** Map ALL ways code can be represented as graphs

**Search Queries:**
- Code property graph
- Program dependence graph
- Control flow graph analysis
- Data flow graph analysis
- Call graph construction
- AST graph representation
- Code embedding graph
- Semantic code graph
- Multi-layer code graph
- Temporal code analysis

**Levels of Aggregation:**
- Statement level
- Function level
- Module level
- Crate level
- Ecosystem level

**Dimensions:**
- Syntax (AST)
- Control (CFG)
- Data (data flow)
- Types (type graph)
- Semantics (similarity)
- Time (evolution)

---

### Phase 0c: arXiv - Intersection Research
**Goal:** Understand where graphs meet code

**Search Queries:**
- Graph neural networks for code
- Graph-based vulnerability detection
- Code similarity via graph matching
- Refactoring via graph analysis
- Bug detection via graph patterns
- Architecture analysis via graphs
- Code clone detection graphs

**Deliverables:**
- What's been tried
- What works
- What failed
- Why things failed

---

### Phase 1: GitHub - Reality Check
**Goal:** Map what's actually implemented in production

**Method:** ghcli research using keywords discovered in Phase 0

**Research Targets:**
- Repos implementing graph algorithms for code
- Production scale characteristics
- Library ecosystem (petgraph, etc.)
- Real-world performance data
- Active vs abandoned projects

---

### Phase 2: Synthesis
**Goal:** Produce actionable strategic recommendations

**Outputs:**
- White space identification
- Parseltongue opportunity mapping
- Build vs buy vs defer decisions
- Risk assessment for each recommendation
- Priority-ranked action items

---

## Thesis Document Structure

```
0. EXECUTIVE SUMMARY
   - Key findings per research track
   - Direct implications for Parseltongue v2.0.0
   - Recommended next actions

1. INTEGRATION PATTERN LANDSCAPE
   - Who's using which of the 5 patterns
   - Migration trends
   - Stability vs capability trade-offs
   - Parseltongue decision: recommended pattern(s)

2. UX/WORKFLOW PATTERNS IN THE WILD
   - How competitors handle the 7 moments
   - Disambiguation patterns
   - Option card presentation patterns
   - Parseltongue decision: UX differentiators

3. GRAPH ALGORITHMS FOR CODE-GRAPHS (EXPANDED)
   3.1 Taxonomy (from arXiv research)
       - Graph types used in code analysis
       - Algorithm categories
       - Emerging approaches
   3.2 Production Landscape (from GitHub research)
       - Repos implementing each category
       - Scale characteristics
       - Library ecosystem
   3.3 Novel Alpha (cross-pollination)
       - What's in papers but not in GitHub
       - What's in other domains
       - What Parseltongue could pioneer
   3.4 Parseltongue Decision Matrix
       - Build vs buy vs defer for each capability

4. ENTITY/CONTEXT MODELS
   - How tools represent code entities
   - Read pointer / chunk key patterns
   - Freshness and versioning strategies
   - Parseltongue decision: entity schema finalization

5. LLM INTEGRATION PATTERNS
   - How tools connect LLMs to compiler data
   - Context packaging strategies
   - Token budgeting approaches
   - Parseltongue decision: LLM integration architecture

6. STRATEGIC RECOMMENDATIONS
   - Priority-ranked action items
   - Risk assessment
   - Dependencies between decisions
```

---

## Constraints

- **DO NOT** read from local repo
- Use **ONLY** ghcli for GitHub research
- Use **arXiv** for academic research
- Progress tracked in `docs/plans/2026-03-02-research-journal.md`

---

## Success Criteria

1. All 5 research tracks completed with actionable findings
2. Graph algorithms section provides "game changer" alpha
3. Clear build/buy/defer recommendations for each decision point
4. Identified white space opportunities for Parseltongue differentiation

---

## Context from User Notes

### 5 Integration Patterns for Rust Compiler Tools
1. Direct rustc_private (30+ tools)
2. rustc_plugin Framework (Flowistry, Aquascope, Paralegal, Argus)
3. Charon/LLBC Decoupling (Aeneas, Hax)
4. Stable MIR / rustc_public
5. No Compiler Dependency

### 7-Moment UX Blueprint
0. Intent Classification
1. Intent Confirmation
2. Option Cards Presented
3. Card Details Viewed
4. Context Preview
5. Deep Dive
6. Action Bar
7. Final Answer

### Key Graph Algorithms (Initial List)
- Circular dependency detection
- Complexity hotspots
- SCC (Tarjan)
- k-Core decomposition
- Centrality (PageRank, Betweenness)
- Leiden community detection
- Coupling/Cohesion metrics
- Blast radius analysis

---

*Design approved: 2026-03-02*
