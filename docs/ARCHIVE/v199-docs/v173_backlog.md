# v1.7.3 Backlog — Deferred Ideas & Future Enhancements

**Date**: 2026-02-15
**Status**: Backlog (not in current build order, but valuable for future versions)
**Source**: CR-v173-03 competitive research, bidirectional LLM research (2026-02-01), session discussions

---

## 1. Structural Pattern Search (Named Graph Query Aliases)

### Why It's Here (Not in PRD)

These patterns already exist as compositions of existing endpoints. A dedicated endpoint would be a convenience wrapper, not new analysis. The real value is in workflow documentation (see PRD section: "Workflow Patterns for --help"), not another endpoint.

### Pattern Library

| Pattern Name | What It Finds | Existing Endpoints That Already Do This |
|---|---|---|
| `god_class` | Entities with CBO > 50 + low cohesion | `/coupling-cohesion-metrics-suite?entity=X` → filter high CBO + low LCOM |
| `hub_function` | Functions called by >10 callers AND calling >10 callees | `/centrality-measures-entity-ranking?method=betweenness` → top results |
| `dead_code` | Entities with 0 reverse callers (nobody calls them) | `/reverse-callers-query-graph?entity=X` → check each entity for 0 callers |
| `fragile_bridge` | Single entity connecting two otherwise-separate clusters | `/centrality-measures-entity-ranking?method=betweenness` → highest between Leiden communities |
| `cycle_participant` | Entities that are part of circular dependencies | `/strongly-connected-components-analysis` → SCC with size > 1 |
| `complexity_hotspot` | Top-N most complex entities | `/complexity-hotspots-ranking-view?top=10` → already exactly this |
| `test_orphan` | Test entities that don't test any production code | `/dependency-edges-list-all` → filter TestCode with 0 edges to CoreCode |

### If We Ever Build It

One endpoint: `/structural-pattern-search-query?pattern=god_class&threshold=50`

The only genuinely new capability would be **ast-grep integration** — shelling out to the `sg` CLI for source-level structural patterns ("find all functions with >5 parameters"). Our graph queries answer relationship questions; ast-grep answers syntax questions. The combination is unique, but it's a separate feature from named graph aliases.

### CozoDB Datalog Behind Each Pattern

```
# god_class: CBO > threshold AND LCOM < 0.3
?[entity, cbo] := *CouplingMetrics{entity, cbo}, cbo > 50

# dead_code: 0 incoming edges, not an entry point
?[entity] := *CodeGraph{ISGL1_key: entity}, not *DependencyEdges{to_key: entity}

# hub_function: high in-degree AND high out-degree
?[entity, in_deg, out_deg] := in_deg = count(from), *DependencyEdges{to_key: entity, from_key: from},
                               out_deg = count(to), *DependencyEdges{from_key: entity, to_key: to},
                               in_deg > 10, out_deg > 10

# cycle_participant: member of SCC with size > 1
?[entity] := *SCC{component_id: c, members: ms}, len(ms) > 1, entity in ms
```

---

## 2. Bidirectional LLM-CPU Enhancement

**Origin**: `docs/PRD-research-20260131v1/PARSELTONGUE_V2_BIDIRECTIONAL_LLM_ENHANCEMENT.md`
**Key Insight**: LLM -> CPU (semantic guidance) + CPU -> LLM (structured data) = Higher accuracy than either alone

### Core Concept

```
Traditional:    CPU computes -> LLM consumes (one-way)
Bidirectional:  LLM provides semantic context -> CPU computes with guidance -> LLM refines -> CPU recomputes (feedback loop)
```

This is the architectural thesis for how Parseltongue's graph algorithms become exponentially more valuable when combined with LLM semantic understanding.

---

### Feature B1: Semantic-Guided Module Boundary Detection

**The Problem**: Leiden algorithm groups by edge density only. It sees high coupling between `authenticate` and `hash_password`, groups them together. But semantically, authentication and cryptography are different domains.

**The Fix**: LLM reads docstrings/comments/function names, extracts domain concepts ("Authentication", "Logging", "Cryptography"), maps them to keywords, and passes them as soft constraints to Leiden. The algorithm uses keyword seeds as penalties: high penalty for splitting entities that share seed keywords.

**Endpoint**: `/semantic-module-boundary-detection`

**Request**:
```json
{
  "target_directory": "auth/",
  "semantic_hints": {
    "domain_concepts": [
      {
        "name": "authentication",
        "keywords": ["auth", "verify", "validate", "login", "session"],
        "priority": "high"
      },
      {
        "name": "logging",
        "keywords": ["log", "event", "audit", "track", "record"],
        "priority": "medium"
      },
      {
        "name": "cryptography",
        "keywords": ["hash", "encrypt", "crypto", "secure", "sign"],
        "priority": "high"
      }
    ],
    "boundary_preferences": {
      "prefer_semantic_coherence": true,
      "min_modularity_score": 0.6,
      "max_modules": 15
    }
  }
}
```

**Response includes**: Modules with semantic labels, matched keywords, boundary rationale, cohesion scores.

**Accuracy improvement**: 91% semantic coherence vs 67% without hints.

---

### Feature B2: Business-Context Technical Debt Scoring

**The Problem**: Pure SQALE treats all debt equally. `payment_processor.rs` (47 debt minutes, critical business path) gets lower priority than `legacy_report_generator.rs` (52 debt minutes, rarely used). Business reality: fix the payment processor first.

**The Fix**:
1. CPU: Run SQALE algorithm → raw debt scores
2. LLM: Read code comments, git history, documentation → classify business criticality
3. CPU: Recompute with business weights → `Enhanced Score = SQALE x Business_Weight x Churn x Centrality`

**Endpoint**: `/business-aware-technical-debt-scoring`

**Result**: payment_processor.rs scores 329 weighted points (P0), legacy_report_generator.rs scores 78 (P3).

**Accuracy improvement**: 89% correct prioritization vs 67% pure SQALE.

---

### Feature B3: Semantic Cycle Classification

**The Problem**: Tarjan's SCC reports all circular dependencies equally. But some cycles are intentional (Observer pattern, Visitor pattern) and some are bugs (God object dependency cycle).

**The Fix**:
1. CPU: Run Tarjan's SCC → find all cycles
2. LLM: Analyze each cycle's code structure and naming against known design patterns
3. Classify: INTENTIONAL_PATTERN (Observer, Visitor, DI) vs ARCHITECTURAL_VIOLATION (God object, circular imports)

**Endpoint**: `/semantic-cycle-classification`

**Result**: "5 cycles found: 2 intentional (design patterns, ignore) + 3 bugs (architectural violations, fix these)"

**Accuracy**: 100% actionable (vs raw SCC which reports all 5 as "cycles" with no classification).

---

### Feature B4: Context-Aware Complexity Scoring

**The Problem**: Cyclomatic complexity treats all branches equally. A function with 15 branches for input validation (essential complexity — each branch validates a business rule) scores the same as a God function with 15 branches doing unrelated things (accidental complexity).

**The Fix**:
1. CPU: Calculate McCabe complexity
2. LLM: Read source code, determine if function has single responsibility
3. Classify: ESSENTIAL_COMPLEXITY (accept) vs ACCIDENTAL_COMPLEXITY (refactor)

**Endpoint**: `/semantic-complexity-classification`

**Result**: `validate_user_input` (15 branches) → ACCEPT. `process_request` (15 branches) → REFACTOR into 4 functions.

---

### Feature B5: Intelligent Refactoring Suggestions

**The Problem**: CPU metrics detect "high coupling (CBO: 23)" but don't suggest HOW to fix it.

**The Fix**:
1. CPU: Detect problem (high coupling, low cohesion, high complexity)
2. LLM: Analyze code structure, identify refactoring pattern
3. LLM: Generate pseudocode for fix, estimate expected improvement

**Endpoint**: `/intelligent-refactoring-suggestions`

**Patterns identified**: Split God Object, Extract Interface, Dependency Inversion.

**Response includes**: Pattern name, confidence, impact score, pseudocode, expected metric improvements ("coupling 23 -> 8 per struct").

---

### Bidirectional Architecture

```
LLM Layer (Semantic Understanding)
  Read Code → Extract Domain Concepts → Classify Intent → Generate Suggestions
       ↓
Translation Layer (Semantic → Algorithmic)
  Domain Keywords → Clustering Seeds
  Business Context → Scoring Weights
  Design Patterns → Algorithm Constraints
       ↓
CPU Layer (Fast Graph Algorithms)
  Leiden Clustering | SQALE Debt | Tarjan Cycles | McCabe Complexity
       ↓
Enhancement Layer (Results → Semantics)
  Add Semantic Labels → Generate Rationale → Propose Actions
       ↓
  (Feedback loop back to LLM Layer for iterative refinement)
```

### Performance Characteristics

| Operation | CPU Only | LLM Only | Bidirectional | Notes |
|-----------|---------|----------|---------------|-------|
| Module Detection (1K entities) | 0.3s | 45s | 2.1s | LLM adds 1.8s |
| Debt Scoring (100 files) | 0.8s | 120s | 4.2s | LLM adds 3.4s |
| Cycle Classification (10 cycles) | 0.1s | 15s | 1.3s | LLM adds 1.2s |
| Complexity Analysis (50 functions) | 0.2s | 30s | 2.8s | LLM adds 2.6s |

**Key insight**: Bidirectional is 7-20x slower than pure CPU, but 15-40x faster than pure LLM, with significantly better accuracy.

### Accuracy Summary

| Feature | CPU Only | LLM Only | Bidirectional | Improvement |
|---------|---------|----------|---------------|-------------|
| Module Boundaries | 67% | 85% (slow) | 91% (fast) | +24% vs CPU |
| Tech Debt Priority | 64% | 81% | 89% | +25% vs CPU |
| Cycle Classification | 0% | 88% | 95% | N/A vs CPU |
| Complexity Analysis | 0% | 86% | 93% | N/A vs CPU |
| Refactoring Suggestions | 0% | 79% | 91% | N/A vs CPU |

**Average improvement**: +21% accuracy vs CPU-only, +7% vs LLM-only.

### When to Use Bidirectional

**Use it when**: Accuracy matters more than speed, semantic context is critical, results need human interpretation, one-time deep analysis (architecture review, codebase audit).

**Use pure CPU when**: Speed is critical (CI/CD), semantic context not needed (raw metrics), high-frequency queries (IDE integration, file watchers).

### Future: Self-Improving Feedback Loop

```
Current:  LLM provides hints → CPU computes → Results
Future:   LLM provides hints → CPU computes → Results → Human validates → Feedback → LLM learns better hints → repeat
```

Over time, the LLM learns which semantic hints produce the most accurate CPU results.

---

## 3. Competitive Features (CR-v173-03 Deferred Items)

### P1 — Plan Next

| Feature | What It Does | Source |
|---|---|---|
| Graph-Native Taint Analysis | Track dirty data (user input) flowing through DependencyEdges to dangerous operations (SQL, shell). New CozoDB relations: TaintSources, TaintSinks, TaintFlows. | CR-v173-03 Feature 1.2 |
| Datalog Policy Engine | Policies as CozoDB Datalog queries. "No entity shall have CBO > 50" is one line. Enterprise compliance (SOC2, HIPAA) maps to graph constraints. | CR-v173-03 Feature 1.5 |

### P2 — Explore

| Feature | What It Does | Source |
|---|---|---|
| Lightweight Telemetry | `tracing` crate for request latency, error rates, ingestion throughput. NOT full OpenTelemetry. | CR-v173-03 Feature 3.3 |
| Session Lifecycle Tracking | Timestamp-based session detection. "Welcome back, 3 files changed." | CR-v173-03 Feature 2.4 |
| Model-Aware Token Budgets | `?model=claude-sonnet` parameter on smart-context endpoint. | CR-v173-03 Feature 3.4 |
| Lua Language Support | `tree-sitter-lua` for 13th language. Analyze Neovim plugins. | CR-v173-03 Feature 4.1 |
| Semgrep Annotation Overlay | Store semgrep findings as annotations on graph entities. "CWE-89 findings within blast radius of user-facing APIs." | CR-v173-03 Feature 5.3 |
| SARIF Export | Serialize Parseltongue analysis results (tech debt, cycles, coupling) as SARIF JSON for GitHub Code Scanning. | CR-v173-03 Feature 5.3 |
| Surgical Source Extraction | Extend smart-context to return actual source code within real token budget. Use graph traversal to include dependencies. | CR-v173-03 Feature 1.4 |

### P3 — Explicitly NOT Building

| Feature | Why Not |
|---|---|
| Z3 Symbolic Execution | code-scalpel has 5K+ LoC, 3 normalizers, unassailable lead. 12-16 weeks effort. |
| Event-Driven Tool Scheduler | Client-side orchestration. PT receives tool calls, doesn't orchestrate them. |
| MCP Client Manager | PT is an MCP server (producer), not client (consumer). |
| ACP Adapter Pattern | Client-side LLM routing. PT doesn't talk to LLMs. |
| Tool Orchestrator | Same as scheduler. PT should be the best tool orchestrators call. |
| Multi-Adapter LLM | Model-agnostic JSON output is the correct strategy. 99% token reduction works for all models. |
| Obsidian-Aware Search | Wrong domain. PT analyzes code, not Markdown. |

---

## 4. Workflow Patterns (for --help and system prompts)

These multi-step recipes teach LLMs and humans how to compose existing endpoints. This is documentation, not code — but it's high-value documentation that should appear in `--help`, `/api-reference-documentation-help`, and LLM system prompts.

### Workflow 1: Understand a Function

```bash
# What is it?
curl /{slug}/{mode}/code-entity-detail-view?key=X

# Who calls it?
curl /{slug}/{mode}/reverse-callers-query-graph?entity=X

# What does it call?
curl /{slug}/{mode}/forward-callees-query-graph?entity=X

# What breaks if I change it?
curl /{slug}/{mode}/blast-radius-impact-analysis?entity=X&hops=2
```

### Workflow 2: Find Architectural Problems

```bash
# Circular dependencies
curl /{slug}/{mode}/circular-dependency-detection-scan

# Worst coupling hotspots
curl /{slug}/{mode}/complexity-hotspots-ranking-view?top=10

# Most central entities (fragile bridges)
curl /{slug}/{mode}/centrality-measures-entity-ranking?method=betweenness

# Tightly coupled clusters
curl /{slug}/{mode}/strongly-connected-components-analysis
```

### Workflow 3: Plan a Refactoring

```bash
# What's the blast radius?
curl /{slug}/{mode}/blast-radius-impact-analysis?entity=X&hops=3

# How coupled is it?
curl /{slug}/{mode}/coupling-cohesion-metrics-suite?entity=X

# What's the tech debt score?
curl /{slug}/{mode}/technical-debt-sqale-scoring?entity=X

# Get LLM-optimized context
curl /{slug}/{mode}/smart-context-token-budget?focus=X&tokens=8000
```

### Workflow 4: Codebase Health Check

```bash
# High-level stats
curl /{slug}/{mode}/codebase-statistics-overview-summary

# Community structure
curl /{slug}/{mode}/leiden-community-detection-clusters

# Core vs periphery
curl /{slug}/{mode}/kcore-decomposition-layering-analysis

# Ingestion coverage
curl /{slug}/{mode}/ingestion-coverage-folder-report?depth=2
```

### Workflow 5: Entropy and Complexity Deep Dive

```bash
# Shannon entropy (information density)
curl /{slug}/{mode}/entropy-complexity-measurement-scores?entity=X

# CK metrics suite (CBO/LCOM/RFC/WMC)
curl /{slug}/{mode}/coupling-cohesion-metrics-suite?entity=X

# PageRank (structural importance)
curl /{slug}/{mode}/centrality-measures-entity-ranking?method=pagerank
```

---

**Last Updated**: 2026-02-15
**Key Principle**: The PRD contains what we're building. This backlog contains what we know is valuable but isn't in the current build order.
