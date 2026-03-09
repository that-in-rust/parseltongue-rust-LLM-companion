# Parseltongue v2.0+: LLM Agent Interaction Patterns

**Date**: 2026-02-01
**Purpose**: Demonstrate how LLM agents iteratively build semantic context using CPU-powered graph features
**Key Insight**: CPU graph algorithms + LLM semantic reasoning = Exponentially more powerful than either alone

---

## Core Concept: The Iterative Intelligence Loop

```mermaid
graph LR
    LLM[LLM Agent<br/>Semantic Reasoning] -->|Query| CPU[CPU Graph Features<br/>Fast Algorithms]
    CPU -->|Structured Data| LLM
    LLM -->|Refined Query| CPU
    CPU -->|More Context| LLM
    LLM -->|Final Insight| User[Human Developer]

    style LLM fill:#e1f5ff
    style CPU fill:#fff9c4
    style User fill:#c8e6c9
```

**The Magic**: Each query result gives the LLM semantic context to ask smarter next questions.

---

## Interaction Pattern 1: Progressive Root Cause Analysis

### Use Case
Developer: "Our app is slow. Find the root cause."

### Interaction Flow

```mermaid
sequenceDiagram
    participant Dev as Developer
    participant LLM as LLM Agent
    participant P as Parseltongue v2.0

    Dev->>LLM: App is slow, find root cause

    Note over LLM: Step 1: Get overview
    LLM->>P: GET /codebase-statistics-overview-summary
    P->>LLM: 230 entities, 3867 edges, 12 modules

    Note over LLM: Semantic reasoning: Check most important entities first
    LLM->>P: GET /centrality-measures-entity-importance
    P->>LLM: Top 5: main, request_handler, db_pool, cache_manager, auth_middleware

    Note over LLM: Semantic insight: request_handler likely bottleneck
    LLM->>P: GET /blast-radius-impact-analysis?entity=request_handler&hops=2
    P->>LLM: Impacts 47 entities, high coupling to db_pool and cache_manager

    Note over LLM: Semantic reasoning: Check coupling metrics
    LLM->>P: GET /coupling-cohesion-metrics?entity=request_handler
    P->>LLM: Efferent coupling: 23, Afferent coupling: 31, Cohesion: 0.42 (LOW)

    Note over LLM: Semantic conclusion: request_handler is God object
    LLM->>P: GET /hierarchical-module-boundary-detection?focus=request_handler
    P->>LLM: request_handler spans 3 natural modules, modularity violation

    LLM->>Dev: Root cause: request_handler is God object<br/>spreading across 3 modules with low cohesion.<br/>Recommendation: Split into auth, caching, data layers.
```

**Key Insight**: LLM uses semantic reasoning to navigate from broad overview → high-level suspects → deep analysis → actionable fix.

**CPU Features Used**: Centrality (PageRank), Blast Radius (Random Walk), Coupling (CK Metrics), Module Detection (Leiden)

**Time Saved**: 2 days of manual debugging → 30 seconds of intelligent queries

---

## Interaction Pattern 2: Iterative Refactoring Planning

### Use Case
Tech Lead: "We need to extract microservices. Where should we start?"

### Interaction Flow

```mermaid
sequenceDiagram
    participant Lead as Tech Lead
    participant LLM as LLM Agent
    participant P as Parseltongue v2.0

    Lead->>LLM: Need to extract microservices

    Note over LLM: Step 1: Find natural boundaries
    LLM->>P: GET /hierarchical-module-boundary-detection
    P->>LLM: 12 modules detected, modularity score: 0.73

    Note over LLM: Semantic reasoning: Which modules are most isolated?
    LLM->>P: GET /coupling-cohesion-metrics (for all modules)
    P->>LLM: Module 3 (payment): cohesion 0.89, external coupling: 7 edges<br/>Module 7 (notification): cohesion 0.91, external coupling: 4 edges

    Note over LLM: Semantic insight: notification is most isolated
    LLM->>P: GET /dependency-edges-list-all?source_module=notification
    P->>LLM: 4 external deps: event_bus, logger, config, metrics

    Note over LLM: Semantic reasoning: Check if deps form cycles
    LLM->>P: GET /circular-dependency-detection-scan?module=notification
    P->>LLM: No cycles detected, safe to extract

    Note over LLM: Validate extraction safety
    LLM->>P: GET /blast-radius-impact-analysis?module=notification&hops=1
    P->>LLM: Removing notification affects 12 entities (low impact)

    LLM->>Lead: Start with notification module:<br/>- High cohesion (0.91)<br/>- Low coupling (4 deps)<br/>- No circular dependencies<br/>- Low blast radius (12 entities)<br/><br/>Next candidates: payment, then analytics
```

**Key Insight**: LLM chains multiple graph features to build confidence score for extraction safety.

**CPU Features Used**: Module Detection (Leiden), Coupling (CK Metrics), Cycle Detection (Tarjan SCC), Blast Radius (Random Walk)

**Semantic Value-Add**: LLM interprets metrics to rank candidates and provide actionable plan.

---

## Interaction Pattern 3: Context-Aware Code Review

### Use Case
PR Review Bot: "Analyze PR #427 changing auth/session.rs"

### Interaction Flow

```mermaid
flowchart TD
    Start[PR changes auth/session.rs] --> LLM1{LLM: What entity is this?}
    LLM1 -->|Query| P1[GET /code-entity-detail-view/auth-session.rs]
    P1 --> C1[Entity: rust:mod:session<br/>47 functions, 12 structs]

    C1 --> LLM2{LLM: How important?}
    LLM2 -->|Query| P2[GET /centrality-measures-entity-importance]
    P2 --> C2[PageRank: 0.087 - top 3% of codebase]

    C2 --> LLM3{LLM: Semantic insight -<br/>Critical component!<br/>What breaks if changed?}
    LLM3 -->|Query| P3[GET /blast-radius-impact-analysis?entity=session&hops=2]
    P3 --> C3[Impacts 67 entities across 8 modules]

    C3 --> LLM4{LLM: Are there cycles?}
    LLM4 -->|Query| P4[GET /circular-dependency-detection-scan?entity=session]
    P4 --> C4[3 circular deps found!<br/>session ↔ user ↔ auth ↔ session]

    C4 --> LLM5{LLM: Check architectural compliance}
    LLM5 -->|Query| P5[GET /layered-architecture-compliance?entity=session]
    P5 --> C5[VIOLATION: session in business layer<br/>depends on presentation layer]

    C5 --> Report[LLM generates report]
    Report --> Output[PR Review Comment:<br/><br/>⚠️ HIGH RISK CHANGE<br/>- Top 3% importance entity<br/>- Impacts 67 entities in 8 modules<br/>- Has 3 circular dependencies<br/>- Violates layer architecture<br/><br/>Recommendations:<br/>1. Break circular deps first<br/>2. Fix layer violation<br/>3. Expand test coverage to 67 impacted entities]

    style LLM1 fill:#e1f5ff
    style LLM2 fill:#e1f5ff
    style LLM3 fill:#e1f5ff
    style LLM4 fill:#e1f5ff
    style LLM5 fill:#e1f5ff
    style Output fill:#c8e6c9
```

**Key Insight**: Each query adds semantic context that informs the next query. LLM builds a risk profile iteratively.

**CPU Features Used**: Entity Detail, Centrality (PageRank), Blast Radius, Cycle Detection (SCC), Layer Compliance

**Human Impact**: Automated code review that catches architectural issues before merge.

---

## Interaction Pattern 4: Semantic-Guided Tech Debt Prioritization

### Use Case
Engineering Manager: "We have 2 weeks for refactoring. What should we fix?"

### Full Interaction Flow

```mermaid
sequenceDiagram
    participant EM as Engineering Manager
    participant LLM as LLM Agent
    participant P as Parseltongue v2.0
    participant Report as Final Report

    EM->>LLM: 2 weeks for refactoring, prioritize

    Note over LLM: Step 1: Get debt landscape
    LLM->>P: GET /technical-debt-quantification-scoring
    P->>LLM: 47 files with debt, top 10:<br/>utils.rs: 873 points<br/>helpers.rs: 712 points<br/>legacy_auth.rs: 654 points<br/>... (87 total minutes to fix top 10)

    Note over LLM: Semantic reasoning: High debt ≠ high priority<br/>Check importance and impact
    LLM->>P: GET /centrality-measures-entity-importance
    P->>LLM: PageRank scores for all entities

    Note over LLM: Cross-reference: debt × importance
    LLM->>P: GET /git-churn-hotspot-correlation
    P->>LLM: utils.rs: 47 commits last month<br/>helpers.rs: 3 commits last month<br/>legacy_auth.rs: 23 commits last month

    Note over LLM: Semantic insight: utils.rs is both high-debt AND high-churn
    LLM->>P: GET /blast-radius-impact-analysis?entity=utils.rs&hops=2
    P->>LLM: utils.rs impacts 89 entities - highest in codebase

    Note over LLM: Check if utils is God module
    LLM->>P: GET /hierarchical-module-boundary-detection?focus=utils
    P->>LLM: utils.rs spans 4 natural modules, severe boundary violation

    Note over LLM: Validate refactoring is safe
    LLM->>P: GET /circular-dependency-detection-scan?entity=utils
    P->>LLM: utils.rs participates in 7 circular dependencies

    Note over LLM: Final validation: coupling metrics
    LLM->>P: GET /coupling-cohesion-metrics?entity=utils
    P->>LLM: Efferent: 34, Afferent: 67, Cohesion: 0.23 (VERY LOW)

    LLM->>Report: Calculate ROI scores:<br/>utils.rs: High debt × High churn × High impact × Low cohesion = CRITICAL<br/>legacy_auth.rs: High debt × Med churn × Med impact = IMPORTANT<br/>helpers.rs: High debt × Low churn × Low impact = LOW PRIORITY

    Report->>EM: 2-Week Refactoring Plan:<br/><br/>Week 1: utils.rs (CRITICAL)<br/>- Debt: 873 points (87 min to fix)<br/>- Churn: 47 commits/month (developer pain)<br/>- Impact: 89 entities (highest blast radius)<br/>- Spans 4 modules (needs splitting)<br/>- 7 circular deps (architectural issue)<br/>- Cohesion: 0.23 (God module)<br/><br/>Week 2: legacy_auth.rs (IMPORTANT)<br/>- Debt: 654 points<br/>- Churn: 23 commits/month<br/>- Medium impact, no cycles<br/><br/>Skip: helpers.rs (only 3 commits/month, low impact)
```

**Key Insight**: LLM synthesizes 7 different CPU features to calculate multi-dimensional ROI score that pure metrics cannot provide.

**CPU Features Used**: Tech Debt (SQALE), Centrality (PageRank), Git Churn, Blast Radius, Module Boundaries (Leiden), Cycle Detection (SCC), Coupling (CK)

**Semantic Magic**: LLM combines metrics using domain knowledge (debt × churn × impact × cohesion) to prioritize work.

**Business Value**: Focus 2 weeks on highest-ROI refactoring instead of arbitrary decisions.

---

## Meta-Pattern: Context Accumulation Over Queries

```mermaid
graph TD
    Q1[Query 1: Overview] --> C1[Context: 230 entities, 12 modules]
    C1 --> Q2[Query 2: Centrality<br/>LLM uses module count to interpret centrality]
    Q2 --> C2[Context: Top 5 important entities]
    C2 --> Q3[Query 3: Blast Radius<br/>LLM focuses on high-centrality entities]
    Q3 --> C3[Context: request_handler impacts 47 entities]
    C3 --> Q4[Query 4: Coupling<br/>LLM investigates high-impact entity]
    Q4 --> C4[Context: Low cohesion 0.42]
    C4 --> Q5[Query 5: Module Boundaries<br/>LLM hypothesizes God object]
    Q5 --> C5[Final Context: Spans 3 modules - CONFIRMED]
    C5 --> Insight[LLM Synthesizes:<br/>request_handler is God object<br/>low cohesion, high coupling,<br/>spans 3 modules, needs splitting]

    style Q1 fill:#e3f2fd
    style Q2 fill:#e3f2fd
    style Q3 fill:#e3f2fd
    style Q4 fill:#e3f2fd
    style Q5 fill:#e3f2fd
    style Insight fill:#c8e6c9
```

**Key Principle**: Each query result becomes input to LLM's semantic reasoning engine, informing the next query.

---

## Query Strategy Decision Tree

```mermaid
graph TD
    Start{User Question Type?} --> Perf[Performance Issue]
    Start --> Refactor[Refactoring Planning]
    Start --> Review[Code Review]
    Start --> Debt[Tech Debt]

    Perf --> Perf1[1. Centrality: Find hot paths]
    Perf1 --> Perf2[2. Blast Radius: Check impact]
    Perf2 --> Perf3[3. Coupling: Identify God objects]
    Perf3 --> PerfEnd[LLM: Propose optimization targets]

    Refactor --> Ref1[1. Module Boundaries: Natural splits]
    Ref1 --> Ref2[2. Coupling: Validate isolation]
    Ref2 --> Ref3[3. Cycles: Check safety]
    Ref3 --> Ref4[4. Blast Radius: Estimate impact]
    Ref4 --> RefEnd[LLM: Rank extraction candidates]

    Review --> Rev1[1. Entity Detail: What changed?]
    Rev1 --> Rev2[2. Centrality: How important?]
    Rev2 --> Rev3[3. Blast Radius: What breaks?]
    Rev3 --> Rev4[4. Layer Compliance: Violates rules?]
    Rev4 --> RevEnd[LLM: Generate risk assessment]

    Debt --> Debt1[1. Tech Debt Scores: Find debt]
    Debt1 --> Debt2[2. Git Churn: Find pain]
    Debt2 --> Debt3[3. Centrality: Find importance]
    Debt3 --> Debt4[4. Blast Radius: Find impact]
    Debt4 --> DebtEnd[LLM: Calculate ROI priority]

    style Start fill:#fff9c4
    style PerfEnd fill:#c8e6c9
    style RefEnd fill:#c8e6c9
    style RevEnd fill:#c8e6c9
    style DebtEnd fill:#c8e6c9
```

**Insight**: Different user questions require different query sequences. LLM chooses strategy based on semantic understanding.

---

## Semantic Reasoning Examples

### Example 1: Inference Between Queries

```
Query 1 Result: "request_handler has PageRank 0.15 (top 1%)"
LLM Reasoning: "High centrality suggests critical component. Need to check coupling."

Query 2: GET /coupling-cohesion-metrics?entity=request_handler
Query 2 Result: "Efferent coupling: 23"
LLM Reasoning: "High centrality + high coupling = God object candidate. Check module boundaries."

Query 3: GET /hierarchical-module-boundary-detection?focus=request_handler
```

**Key**: LLM doesn't just fetch data - it interprets results to choose next query.

### Example 2: Cross-Feature Synthesis

```
Feature 1 (Tech Debt): utils.rs has 873 debt points
Feature 2 (Git Churn): utils.rs has 47 commits last month
Feature 3 (Blast Radius): utils.rs impacts 89 entities
Feature 4 (Cohesion): utils.rs has cohesion 0.23

LLM Synthesis:
IF debt > 800 AND churn > 40 AND blast_radius > 80 AND cohesion < 0.3:
    priority = CRITICAL
ELSE IF debt > 600 AND (churn > 20 OR blast_radius > 50):
    priority = HIGH
ELSE:
    priority = MEDIUM
```

**Key**: LLM creates multi-dimensional priority function that no single CPU feature provides.

### Example 3: Semantic Hypothesis Testing

```
Hypothesis: "This file is a God object"

Test 1: High centrality? → YES (PageRank 0.12)
Test 2: Low cohesion? → YES (0.31)
Test 3: High coupling? → YES (Efferent: 28)
Test 4: Spans multiple modules? → YES (3 modules)
Test 5: Participates in cycles? → YES (4 cycles)

LLM Conclusion: "CONFIRMED - God object with 5/5 indicators"
```

**Key**: LLM uses multiple CPU features as evidence to test architectural hypotheses.

---

## The Exponential Power Equation

```mermaid
graph LR
    subgraph "CPU Features Alone"
        CF1[Fast graph algorithms<br/>O-E-log-V complexity]
        CF2[Structured data output]
        CF3[No interpretation]
        Value1[Linear Value]
    end

    subgraph "LLM Reasoning Alone"
        LLM1[Semantic understanding]
        LLM2[Pattern recognition]
        LLM3[No structured data access]
        Value2[Limited Value]
    end

    subgraph "CPU + LLM Together"
        Both1[Fast structured data]
        Both2[Semantic interpretation]
        Both3[Iterative refinement]
        Both4[Context accumulation]
        Both5[Multi-feature synthesis]
        ValueExp[EXPONENTIAL VALUE]
    end

    CF1 & CF2 & CF3 --> Value1
    LLM1 & LLM2 & LLM3 --> Value2
    Both1 & Both2 & Both3 & Both4 & Both5 --> ValueExp

    Value1 -.->|Limited by lack of reasoning| ValueExp
    Value2 -.->|Limited by lack of data| ValueExp

    style ValueExp fill:#c8e6c9,stroke:#4caf50,stroke-width:3px
    style Value1 fill:#ffccbc
    style Value2 fill:#ffccbc
```

**Formula**: `Value = CPU_Speed × LLM_Reasoning × Iteration_Depth`

- **CPU alone**: Fast but no interpretation → Linear value
- **LLM alone**: Smart but no data access → Limited value
- **CPU + LLM**: Fast data + Smart reasoning + Iterative refinement → Exponential value

---

## Real-World Agent Workflow: Full Example

### Scenario
Developer asks: "Why is the login feature slow?"

### Complete Interaction Chain

```mermaid
sequenceDiagram
    autonumber
    participant Dev as Developer
    participant LLM as LLM Agent
    participant P as Parseltongue v2.0

    Dev->>LLM: Why is login feature slow?

    Note over LLM: Semantic: "login" → search for entities
    LLM->>P: GET /code-entities-search-fuzzy?q=login
    P->>LLM: Found: authenticate_user, login_handler, session_create

    Note over LLM: Semantic: login_handler likely entry point
    LLM->>P: GET /centrality-measures?entity=login_handler
    P->>LLM: PageRank: 0.089 (high), Betweenness: 0.12 (very high)

    Note over LLM: High betweenness = bottleneck! Check what it calls
    LLM->>P: GET /forward-callees-query-graph?entity=login_handler
    P->>LLM: Calls: validate_credentials, check_session, log_event, send_analytics, update_stats

    Note over LLM: Too many responsibilities! Check complexity
    LLM->>P: GET /cyclomatic-complexity?entity=login_handler
    P->>LLM: Complexity: 47 (VERY HIGH - should be under 10)

    Note over LLM: Semantic: High complexity suggests many branches. Check cohesion
    LLM->>P: GET /coupling-cohesion-metrics?entity=login_handler
    P->>LLM: Cohesion: 0.34 (LOW), Efferent: 18, Afferent: 12

    Note over LLM: Low cohesion confirms multiple responsibilities. Check module boundaries
    LLM->>P: GET /hierarchical-module-boundary-detection?focus=login_handler
    P->>LLM: login_handler spans 3 modules: auth, analytics, logging

    Note over LLM: Semantic: Performance issue is architectural. Check blast radius
    LLM->>P: GET /blast-radius-impact-analysis?entity=login_handler&hops=1
    P->>LLM: Impacts 34 entities if changed

    Note over LLM: Final semantic synthesis...
    LLM->>Dev: Root Cause Analysis:<br/><br/>login_handler is slow because:<br/>1. High betweenness (0.12) → bottleneck in call graph<br/>2. Cyclomatic complexity (47) → too many branches<br/>3. Low cohesion (0.34) → doing 3 jobs at once<br/>4. Spans 3 modules → violates single responsibility<br/>5. Calls 5 unrelated functions → sequential overhead<br/><br/>Performance Fix:<br/>Split login_handler into:<br/>- AuthHandler (validate_credentials, check_session)<br/>- AnalyticsHandler (send_analytics, update_stats)<br/>- AuditHandler (log_event)<br/><br/>Run analytics/audit asynchronously → 60% speedup<br/><br/>Impact: Safe to refactor (34 entities, medium risk)
```

**CPU Features Used**: 7 features in sequence
**Time**: 8 seconds total
**Human Alternative**: 4 hours of manual code reading and guessing

---

## Comparison: Agent vs. Human Developer

| Task | Human Developer | LLM Agent + Parseltongue |
|------|----------------|--------------------------|
| **Find root cause** | 2-4 hours of debugging | 30 seconds (5-7 queries) |
| **Prioritize refactoring** | Gut feeling, politics | Multi-dimensional ROI score in 15 seconds |
| **Code review risk** | Spot-check, manual testing | Comprehensive risk profile in 10 seconds |
| **Architecture analysis** | 2 days of whiteboarding | 1 minute of iterative queries |
| **Semantic reasoning** | High (but slow) | High (and fast) |
| **Data access** | Manual (grep, IDE search) | Instant (CPU algorithms) |
| **Iteration speed** | Minutes per hypothesis | Seconds per hypothesis |

**Winner**: LLM Agent + Parseltongue (100× faster with equal or better reasoning)

---

## Key Takeaways

### 1. Iteration is the Superpower
- Single query: Useful
- 3-4 chained queries: Powerful
- 7+ queries with context accumulation: Game-changing

### 2. Semantic Reasoning Bridges Features
LLM interprets results from Feature A to ask smarter questions to Feature B.

Example:
```
High centrality (Feature 11) + Low cohesion (Feature 8)
→ LLM infers: God object candidate
→ Queries module boundaries (Feature 1) to confirm
```

### 3. Multi-Dimensional Synthesis
No single CPU feature calculates "refactoring priority", but LLM synthesizes:
```
Priority = f(debt, churn, centrality, blast_radius, cohesion, cycles)
```

### 4. Hypothesis Testing Workflow
LLM uses CPU features as evidence:
```
Hypothesis: "This is a performance bottleneck"
Evidence:
  ✓ High betweenness centrality
  ✓ High cyclomatic complexity
  ✓ Low cohesion
  ✓ Spans multiple modules
Conclusion: CONFIRMED
```

### 5. Context Accumulation
Each query result enriches LLM's semantic model:
```
Query 1: What exists?
Query 2: What's important? (uses Query 1 context)
Query 3: What's wrong? (uses Query 1+2 context)
Query 4: What breaks? (uses Query 1+2+3 context)
Query 5: How to fix? (uses all prior context)
```

---

## Implementation Patterns for Agents

### Pattern 1: Progressive Zoom

```python
def analyze_performance_issue(issue_description):
    # Start broad
    overview = query("/codebase-statistics-overview-summary")

    # Zoom to suspects
    suspects = identify_suspects(overview, issue_description)
    centrality = query(f"/centrality-measures?entities={suspects}")

    # Zoom to root cause
    top_suspect = max(centrality, key=lambda x: x.pagerank)
    blast_radius = query(f"/blast-radius?entity={top_suspect}")
    coupling = query(f"/coupling-cohesion?entity={top_suspect}")

    # Final diagnosis
    return synthesize_diagnosis(centrality, blast_radius, coupling)
```

### Pattern 2: Hypothesis Testing

```python
def test_god_object_hypothesis(entity):
    evidence = {}

    # Test 1: High centrality?
    centrality = query(f"/centrality-measures?entity={entity}")
    evidence['high_centrality'] = centrality.pagerank > 0.05

    # Test 2: Low cohesion?
    coupling = query(f"/coupling-cohesion?entity={entity}")
    evidence['low_cohesion'] = coupling.cohesion < 0.4

    # Test 3: Spans modules?
    boundaries = query(f"/module-boundaries?focus={entity}")
    evidence['spans_modules'] = boundaries.module_count > 1

    # Conclusion
    confidence = sum(evidence.values()) / len(evidence)
    return confidence > 0.7  # 70% threshold
```

### Pattern 3: Multi-Dimensional Ranking

```python
def prioritize_refactoring_candidates(files):
    priorities = []

    for file in files:
        # Dimension 1: Technical debt
        debt = query(f"/tech-debt-score?entity={file}")

        # Dimension 2: Churn
        churn = query(f"/git-churn?entity={file}")

        # Dimension 3: Impact
        blast = query(f"/blast-radius?entity={file}")

        # Dimension 4: Coupling
        coupling = query(f"/coupling?entity={file}")

        # Synthesize priority score
        score = (debt.score * 0.3 +
                 churn.commits * 0.25 +
                 blast.impact * 0.25 +
                 coupling.efferent * 0.2)

        priorities.append((file, score))

    return sorted(priorities, key=lambda x: x[1], reverse=True)
```

---

## Future: Multi-Agent Collaboration

```mermaid
graph TD
    User[User Question] --> Orchestrator[Orchestrator Agent]

    Orchestrator -->|Delegate| A1[Analysis Agent<br/>Runs diagnostic queries]
    Orchestrator -->|Delegate| A2[Planning Agent<br/>Ranks refactoring options]
    Orchestrator -->|Delegate| A3[Validation Agent<br/>Checks safety constraints]

    A1 -->|Findings| CPU1[Centrality, Blast Radius, Coupling]
    A2 -->|Rankings| CPU2[Module Boundaries, Tech Debt, Churn]
    A3 -->|Safety| CPU3[Cycles, Layer Compliance]

    CPU1 --> Synthesizer[Synthesis Agent]
    CPU2 --> Synthesizer
    CPU3 --> Synthesizer

    Synthesizer --> Report[Comprehensive Report<br/>with evidence and recommendations]
    Report --> User

    style Orchestrator fill:#fff9c4
    style Synthesizer fill:#c8e6c9
```

**Vision**: Multiple specialized LLM agents query Parseltongue in parallel, then synthesize findings.

---

## Conclusion

**The Magic Formula**:

```
Fast CPU Algorithms (O-E-log-V)
+
Semantic LLM Reasoning
+
Iterative Context Building
=
10× faster insights than humans
100× faster than LLM without structured data
1000× faster than CPU without reasoning
```

Parseltongue v2.0 features are designed for **iterative LLM consumption**, not one-shot queries. The real power emerges when LLM agents chain 5-10 queries with semantic reasoning between each step.

---

**Last Updated**: 2026-02-01
**Source**: Analysis of LLM agent interaction patterns with graph databases
**Key Insight**: Context accumulation over iterations = Exponential intelligence gain
