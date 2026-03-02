# Research Agenda for Parseltongue

**Purpose:** 6-12 month forward-looking research plan
**Target:** Maintain competitive edge through continued innovation

---

## Executive Summary

```
┌─────────────────────────────────────────────────────────────────────┐
│                    RESEARCH AGENDA OVERVIEW                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  MONTHS 1-3: Foundation Research                                    │
│  ├── Algorithm optimization                                         │
│  ├── Code graph construction improvements                           │
│  └── Performance benchmarking                                       │
│                                                                     │
│  MONTHS 4-6: Advanced Capabilities                                  │
│  ├── LLM + Graph integration                                        │
│  ├── Temporal analysis                                              │
│  └── Multi-layer graphs                                             │
│                                                                     │
│  MONTHS 7-12: Cutting Edge                                          │
│  ├── Neuro-symbolic approaches                                      │
│  ├── Self-supervised learning                                       │
│  └── Causal code analysis                                           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 1. LLM + GRAPH INTEGRATION

### Problem Statement
How do we combine LLMs with graph algorithms for superior code understanding?

### Research Questions
1. Can graph structure improve LLM context selection?
2. Can LLMs improve graph construction/analysis?
3. What's the optimal graph-aware prompting strategy?

### Experiments

#### Exp 1.1: Graph-Guided Context Selection
```
Hypothesis: Graph centrality improves context selection over naive retrieval

Method:
1. Build code graph
2. Query for "authentication"
3. Compare:
   - Baseline: Top-K embedding similarity
   - Graph: Centrality-weighted + community sampling
   - Hybrid: Embedding + graph reranking

Metrics:
- Context relevance (human eval)
- Token efficiency
- Answer quality

Timeline: 2 weeks
Resources: 1 researcher + human eval
```

#### Exp 1.2: LLM-Guided Graph Construction
```
Hypothesis: LLM can infer missing edges (semantic relationships)

Method:
1. Build base graph (calls, imports)
2. Use LLM to infer semantic relationships
3. Compare graph quality

Metrics:
- Precision/recall of inferred edges
- Downstream task improvement

Timeline: 3 weeks
```

#### Exp 1.3: Graph RAG for Code
```
Hypothesis: Graph-based retrieval outperforms vector-only

Method:
1. Implement Graph RAG:
   - Entity extraction
   - Graph traversal
   - Multi-hop retrieval
2. Compare with standard RAG

Metrics:
- Retrieval accuracy
- Answer correctness
- Latency

Timeline: 4 weeks
```

### Expected Outcomes
- Graph-aware prompting templates
- Graph RAG implementation
- 20%+ improvement in context selection

### Priority: 🔴 HIGH

---

## 2. TEMPORAL CODE ANALYSIS

### Problem Statement
How do we track and analyze code evolution over time?

### Research Questions
1. How do we efficiently track graph changes?
2. What temporal patterns predict bugs?
3. How do we identify stable vs volatile code?

### Experiments

#### Exp 2.1: Incremental Graph Updates
```
Hypothesis: Incremental updates are 10x faster than full rebuild

Method:
1. Parse git history
2. Build graph for each commit
3. Compute diff graphs
4. Measure update time vs full rebuild

Metrics:
- Time per update
- Memory efficiency
- Accuracy (match to full rebuild)

Timeline: 2 weeks
```

#### Exp 2.2: Code Stability Prediction
```
Hypothesis: Temporal graph features predict future changes

Method:
1. Extract features from code history:
   - Change frequency
   - Centrality trends
   - Community stability
2. Train classifier for "will change in next month"
3. Evaluate on held-out repos

Metrics:
- AUC-ROC
- Precision at k

Timeline: 4 weeks
```

#### Exp 2.3: Bug Introduction Detection
```
Hypothesis: Temporal analysis identifies when bugs were introduced

Method:
1. Identify bugs from issue tracker + commits
2. Trace back through graph history
3. Identify "bug-introducing commits"
4. Analyze graph patterns at introduction

Metrics:
- Detection accuracy
- False positive rate

Timeline: 6 weeks
```

### Expected Outcomes
- Incremental graph update algorithm
- Stability prediction model
- Bug archaeology tool

### Priority: 🟡 MEDIUM

---

## 3. MULTI-LAYER GRAPH ANALYSIS

### Problem Statement
How do we combine multiple graph types (AST, CFG, call, type)?

### Research Questions
1. What's the optimal multi-layer representation?
2. How do we analyze cross-layer dependencies?
3. Which layer combinations are most useful?

### Experiments

#### Exp 3.1: Layer Combination Strategies
```
Hypothesis: Layered approach outperforms single-graph

Method:
1. Build multiple layers:
   - L1: Syntax (AST)
   - L2: Control flow (CFG)
   - L3: Data flow (DFG)
   - L4: Types
2. Compare:
   - Flatten to single graph
   - Multi-layer with inter-layer edges
   - Tensor representation

Metrics:
- Analysis accuracy
- Computational cost
- Memory usage

Timeline: 3 weeks
```

#### Exp 3.2: Cross-Layer Pattern Detection
```
Hypothesis: Patterns span layers (e.g., CFG pattern + type pattern)

Method:
1. Define cross-layer patterns
2. Implement detection
3. Evaluate on known patterns

Metrics:
- Detection recall
- False positives

Timeline: 4 weeks
```

### Expected Outcomes
- Multi-layer graph data structure
- Cross-layer analysis algorithms
- Pattern library

### Priority: 🟡 MEDIUM

---

## 4. NEURO-SYMBOLIC CODE ANALYSIS

### Problem Statement
Can we combine symbolic (graph algorithms) with neural (GNNs)?

### Research Questions
1. When do GNNs outperform symbolic algorithms?
2. Can we get best of both worlds?
3. What's the right abstraction?

### Experiments

#### Exp 4.1: GNN vs Symbolic Benchmark
```
Hypothesis: GNNs better for fuzzy matching, symbolic for precision

Method:
1. Create benchmark tasks:
   - Clone detection (fuzzy)
   - Impact analysis (precise)
   - Bug detection (mixed)
2. Compare:
   - Pure symbolic
   - Pure GNN
   - Hybrid

Metrics:
- Accuracy per task type
- Computational cost
- Interpretability

Timeline: 4 weeks
```

#### Exp 4.2: Neuro-Symbolic Hybrid
```
Hypothesis: Symbolic pre-processing + GNN > either alone

Method:
1. Use symbolic algorithms to:
   - Build graph
   - Compute initial features (centrality, community)
2. Use GNN to:
   - Learn task-specific representations
   - Make predictions

Metrics:
- Task accuracy
- Training efficiency
- Inference speed

Timeline: 6 weeks
```

### Expected Outcomes
- Benchmark suite
- Hybrid architecture design
- Best practices for when to use which

### Priority: 🟡 MEDIUM-HIGH

---

## 5. SELF-SUPERVISED LEARNING ON CODE GRAPHS

### Problem Statement
Can we learn useful representations without labels?

### Research Questions
1. What pretext tasks work for code graphs?
2. How much unlabeled data helps?
3. Do representations transfer?

### Experiments

#### Exp 5.1: Pretext Task Design
```
Hypothesis: Code-specific pretext tasks outperform generic

Method:
1. Design pretext tasks:
   - Masked node prediction
   - Edge prediction
   - Subgraph isomorphism
   - Community membership
2. Pretrain GNN
3. Evaluate on downstream tasks

Metrics:
- Downstream task accuracy
- Label efficiency
- Transfer performance

Timeline: 6 weeks
```

#### Exp 5.2: Contrastive Learning on Graphs
```
Hypothesis: Contrastive learning creates useful representations

Method:
1. Define positive/negative pairs:
   - Same function, different versions
   - Similar functions
   - Same community
2. Train contrastive model
3. Evaluate representations

Metrics:
- Representation quality
- Downstream task improvement

Timeline: 4 weeks
```

### Expected Outcomes
- Pretrained code graph encoder
- Pretext task library
- Label-efficient learning

### Priority: 🟢 LOW (research-stage)

---

## 6. CAUSAL CODE ANALYSIS

### Problem Statement
Can we infer causal relationships in code?

### Research Questions
1. Does X cause Y (not just correlate)?
2. What's the causal graph of code dependencies?
3. Can we do counterfactual reasoning?

### Experiments

#### Exp 6.1: Causal Dependency Graph
```
Hypothesis: Causal analysis reveals hidden dependencies

Method:
1. Build causal graph using:
   - Control flow
   - Data flow
   - Side effects
2. Distinguish correlation from causation

Metrics:
- Causal discovery accuracy
- Intervention prediction

Timeline: 8 weeks
```

#### Exp 6.2: Counterfactual Code Reasoning
```
Hypothesis: "What if" analysis helps understand code

Method:
1. Given code change, predict:
   - What would break?
   - What tests would fail?
   - What performance impact?
2. Validate on actual changes

Metrics:
- Prediction accuracy
- Coverage

Timeline: 8 weeks
```

### Expected Outcomes
- Causal analysis framework
- Counterfactual reasoning tool
- Better change prediction

### Priority: 🟢 LOW (experimental)

---

## 7. RESEARCH TIMELINE

```
┌─────────────────────────────────────────────────────────────────────┐
│ MONTH │ 1  │ 2  │ 3  │ 4  │ 5  │ 6  │ 7  │ 8  │ 9  │ 10 │ 11 │ 12 │
├─────────────────────────────────────────────────────────────────────┤
│ LLM+Graph                                                              │
│ ████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
│ Temporal                                                               │
│ ░░░░░░████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
│ Multi-layer                                                            │
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░████████████████████░░░░░░░░░░░░░░░░░░░░ │
│ Neuro-symbolic                                                         │
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████████████████████████ │
│ Self-supervised                                                        │
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████████ │
│ Causal                                                                 │
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
└─────────────────────────────────────────────────────────────────────┘

█ Active research
░ Background/planning
```

---

## 8. RESOURCE REQUIREMENTS

### Personnel
```
┌─────────────────────────────────────────────────────────────────────┐
│ Role                    │ FTE  │ Focus                               │
├─────────────────────────────────────────────────────────────────────┤
│ Research Engineer       │ 1.0  │ Algorithm implementation            │
│ ML Engineer            │ 0.5  │ GNN experiments                     │
│ Rust Developer         │ 1.0  │ Core development                    │
│ Intern/Research Asst   │ 1.0  │ Experiments, data collection        │
└─────────────────────────────────────────────────────────────────────┘
```

### Compute
```
Development:
- 1x GPU instance (A100 or equiv) for ML experiments
- 4x CPU cores for graph processing

Production:
- Scale based on user demand
```

### Data
```
Training Data:
- 10,000+ open-source Rust repos
- 100+ repos with full git history
- Bug database (RustSec, GitHub issues)

Evaluation:
- Human-labeled test sets
- Benchmark suites
```

---

## 9. SUCCESS METRICS

### Quarter 1
- [ ] PageRank implementation complete
- [ ] Betweenness implementation complete
- [ ] Leiden implementation complete
- [ ] Benchmark suite established

### Quarter 2
- [ ] LLM+Graph integration working
- [ ] Temporal analysis MVP
- [ ] 20%+ improvement in context selection
- [ ] First research paper draft

### Quarter 3
- [ ] Multi-layer graph support
- [ ] Neuro-symbolic prototype
- [ ] Self-supervised pretraining working
- [ ] Published research

### Quarter 4
- [ ] All experiments complete
- [ ] Production-ready implementations
- [ ] 2+ research papers published
- [ ] Clear competitive advantage

---

## 10. OPEN PROBLEMS

### Problems We're Uniquely Positioned to Solve

1. **Optimal graph construction for code**
   - No consensus on best representation
   - Parseltongue can establish the standard

2. **LLM context selection**
   - Current: heuristic-based
   - Opportunity: graph-informed selection

3. **Code understanding at scale**
   - Current: slow or inaccurate
   - Opportunity: petgraph + algorithms

4. **Cross-repo analysis**
   - Current: per-repo only
   - Opportunity: ecosystem graphs

### Risks

| Risk | Probability | Mitigation |
|------|-------------|------------|
| Research doesn't pan out | Medium | Parallel approaches |
| Competitor leapfrogs | Low | Trade secret protection |
| Technical debt | Medium | Clean architecture |

---

## 11. ACADEMIC COLLABORATIONS

### Potential Partners
1. **Brown PLSE Lab** (Will Crichton)
   - Flowistry, rustc_plugin
   - UX research

2. **ETH Zurich** (Prusti team)
   - Formal verification
   - Rust semantics

3. **CMU** (Software Engineering)
   - Large-scale analysis
   - Mining repositories

4. **Georgia Tech** (Rudra team)
   - Vulnerability detection
   - Static analysis

### Collaboration Models
- Joint research papers
- Intern hosting
- Grant applications
- Open-source contributions

---

## APPENDIX: PAPERS TO READ

### LLM + Graphs
1. "Graph-Toolformer" (2023)
2. "GraphLLM" (2023)
3. "LLMxCPG" (2025)

### Temporal Analysis
1. "TGN: Temporal Graph Networks" (2020)
2. "Continuous-Time Dynamic Graphs" (2020)

### Neuro-Symbolic
1. "Neuro-Symbolic AI" survey (2023)
2. "Neural-Symbolic Machine" (2017)

### Self-Supervised
1. "Self-Supervised Learning on Graphs" (2020)
2. "Graph Contrastive Learning" (2020)

### Causal
1. "Causal Inference in Graphs" (2018)
2. "Counterfactual Reasoning" (2021)
