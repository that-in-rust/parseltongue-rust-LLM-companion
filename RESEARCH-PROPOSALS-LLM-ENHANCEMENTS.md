# Research Document: 5 Innovative Features for Parseltongue to Revolutionize LLM Code Generation

**Author**: AI Research Team (1000 IQ Analysis)
**Date**: 2026-01-30
**Context**: Parseltongue v1.4.2 Enhancement Proposals
**Objective**: Make LLMs write significantly better code through advanced analysis

---

## Executive Summary

This document proposes five research-level features for Parseltongue that leverage advanced computer science theory to make LLMs write better code. Each feature is grounded in formal methods, graph theory, information theory, or machine learning, and addresses specific challenges in LLM-assisted code generation.

**Current Foundation**: Parseltongue already provides:
- Graph-based code representation (CozoDB with Datalog queries)
- Entity/dependency extraction across 12 languages
- Blast radius analysis (BFS traversal)
- Semantic clustering (label propagation)
- Smart context optimization (greedy knapsack)
- Always-on file watching with incremental reindexing

**Innovation Gap**: While Parseltongue excels at structural analysis, LLMs still struggle with:
1. Understanding temporal evolution patterns (how code changes over time)
2. Detecting architectural violations and anti-patterns
3. Generating test cases that maximize coverage
4. Predicting bug locations based on graph patterns
5. Inferring semantic meaning beyond syntax

---

## Feature 1: Temporal Pattern Inference Engine

### Feature Name
**temporal-pattern-inference-engine**

### Problem Statement
LLMs currently view code as static snapshots. When generating modifications, they lack understanding of:
- How entities evolve over time (creation → growth → stabilization → decay)
- Temporal coupling (entities that change together despite no structural dependency)
- Code churn patterns (high-change entities often harbor bugs)
- Evolutionary hotspots (areas of rapid change that indicate design instability)

**Concrete LLM Challenge**: When asked to refactor a module, LLMs cannot distinguish between stable APIs (rarely changed, high blast radius) vs experimental code (frequently changed, lower risk).

### Technical Approach

#### 1. Data Structure: Temporal Event Log
```rust
pub struct TemporalEvent {
    pub timestamp: DateTime<Utc>,
    pub entity_key: String,
    pub event_type: EventType,  // Create, Modify, Delete, Rename
    pub commit_hash: String,
    pub author: String,
    pub change_magnitude: f64,  // Levenshtein distance / original size
}

pub enum EventType {
    Create,
    Modify { lines_added: usize, lines_deleted: usize },
    Delete,
    Rename { old_key: String, new_key: String },
}
```

**Storage**: New CozoDB relation `TemporalEvents { timestamp, entity_key, event_type, ... }`

#### 2. Algorithm: Change Frequency Analysis (Exponential Decay)
Compute change frequency with recency weighting:

```
f(entity, t) = Σ exp(-λ * (t - t_i)) * magnitude(change_i)
```

Where:
- `t_i` = timestamp of change i
- `λ` = decay parameter (e.g., 0.01 for ~100 day half-life)
- `magnitude(change)` = normalized change size (0-1)

**Complexity**: O(E) where E = number of events per entity

#### 3. Algorithm: Temporal Coupling Detection (Frequent Itemset Mining)
Use FP-Growth algorithm to find entity pairs that frequently change together:

```
Input: Transactions = {T1, T2, ..., Tn} where Ti = {entities changed in commit i}
Output: {(entity_a, entity_b, confidence)} where confidence = P(b changed | a changed)

1. Build FP-tree from transaction database
2. Mine frequent patterns with min_support threshold (e.g., 0.05)
3. Calculate lift ratio: P(a ∩ b) / (P(a) * P(b))
4. Filter pairs with lift > 1.5 (indicating correlation beyond random chance)
```

**Complexity**: O(N * L) where N = commits, L = average entities per commit

#### 4. Algorithm: Change Pattern Classification (Hidden Markov Model)
Model entity lifecycle as HMM with states:

```
States: {Experimental, Growing, Stable, Legacy, Deprecated}
Observations: {High churn, Medium churn, Low churn, No change}

Transition Matrix P[i][j] = P(state_j | state_i):
                Exp    Grow   Stable  Legacy  Deprec
Experimental   [0.6    0.3    0.05    0.0     0.05  ]
Growing        [0.05   0.7    0.2     0.0     0.05  ]
Stable         [0.01   0.05   0.9     0.03    0.01  ]
Legacy         [0.0    0.0    0.1     0.85    0.05  ]
Deprecated     [0.0    0.0    0.0     0.05    0.95  ]

Use Viterbi algorithm to infer most likely state sequence.
```

**Complexity**: O(T * S²) where T = time periods, S = states (constant 5)

### Implementation Sketch

**New HTTP Endpoints** (following 4-word naming):
1. `GET /temporal-change-frequency-analysis?entity=X&window=90d`
   - Returns change frequency score, decay curve, churn ranking

2. `GET /temporal-coupling-detection-pairs?min_confidence=0.6`
   - Returns entity pairs with temporal coupling metrics

3. `GET /temporal-lifecycle-state-inference?entity=X`
   - Returns HMM state classification (Experimental/Growing/Stable/Legacy/Deprecated)

4. `GET /temporal-hotspot-evolution-view?hops=3&days=180`
   - Combines blast radius with temporal metrics for risk scoring

**Database Schema Extensions**:
```sql
:create TemporalEvents {
    timestamp: String,
    entity_key: String,
    event_type: String,
    commit_hash: String,
    author: String,
    change_magnitude: Float
    =>
    lines_added: Int?,
    lines_deleted: Int?,
    old_entity_key: String?
}

:create TemporalCouplings {
    entity_a: String,
    entity_b: String
    =>
    confidence: Float,
    lift_ratio: Float,
    support: Float,
    last_updated: String
}
```

**Integration Point**: `crates/parseltongue-core/src/temporal.rs`

### LLM Benefit

**Concrete Use Cases**:

1. **Refactoring Risk Assessment**
```bash
# LLM asks: "Should I refactor authenticate()?"
curl "http://localhost:7777/temporal-lifecycle-state-inference?entity=rust:fn:authenticate:src_auth_rs:10-50"

# Response: {"state": "Stable", "confidence": 0.92, "last_changed": "145 days ago"}
# LLM decision: HIGH RISK - stable API with long unchanged period. Suggest incremental approach.
```

2. **Change Prediction**
```bash
# LLM asks: "If I modify UserService, what else might need updating?"
curl "http://localhost:7777/temporal-coupling-detection-pairs?entity=rust:struct:UserService"

# Response: [
#   {"coupled_entity": "rust:struct:AuthService", "confidence": 0.87, "explanation": "Changed together in 23/27 commits"},
#   {"coupled_entity": "rust:fn:validate_token", "confidence": 0.76}
# ]
# LLM: Proactively suggests checking AuthService for consistency
```

3. **Bug Prediction Hotspot Ranking**
```bash
curl "http://localhost:7777/temporal-hotspot-evolution-view?days=90"

# Combines high churn + high coupling + high blast radius
# Returns ranked list of entities statistically likely to harbor bugs
```

### Novel Insight

**Why This Is Creative**: Existing code analysis tools treat time as a snapshot. This feature applies **financial time series analysis** (exponential decay, volatility indices) to code evolution, treating commits as transactions and entities as securities. The HMM lifecycle model borrows from **natural language processing** (POS tagging) to classify code maturity automatically.

**Mathematical Foundation**:
- Exponential decay: Standard in signal processing for recency weighting
- FP-Growth: Data mining classic (Han et al., 2000) for association rules
- HMM: Probabilistic model from speech recognition, adapted for code states

### Feasibility: **Medium**

**Implementation Complexity**:
- **LOW**: Event storage in CozoDB (straightforward schema extension)
- **MEDIUM**: FP-Growth implementation (well-documented algorithm, ~500 LOC)
- **MEDIUM**: HMM Viterbi algorithm (standard dynamic programming, ~300 LOC)
- **HIGH**: Git integration for automatic event extraction (requires robust parsing)

**Estimated Effort**: 2-3 weeks for MVP (using existing Rust crates like `rust-hmmm`, `apriori`)

**Dependencies**:
- Git history access (via `libgit2` or `git log` parsing)
- Time-series data structure (ring buffer for performance)
- Incremental computation (recompute only changed entities)

---

## Feature 2: Constraint-Based Type Refinement Synthesizer

### Feature Name
**constraint-type-refinement-synthesizer**

### Problem Statement

LLMs generate syntactically correct code but often violate semantic invariants:
- Functions assume inputs are non-null but don't validate
- Array indices may be out-of-bounds
- Division by zero not checked
- Overflow/underflow in arithmetic

**Concrete Challenge**: When asked to generate a function like `get_user_by_id(id: i32)`, LLM produces code that doesn't verify `id > 0` or handle missing users.

**Solution**: Use **refinement types** to encode preconditions/postconditions in the type system, then synthesize runtime checks.

### Technical Approach

#### 1. Refinement Type Specification (Liquid Types)
Extend type annotations with predicates:

```rust
// Standard type
fn divide(a: i32, b: i32) -> i32 { a / b }

// Refined type (conceptual syntax)
fn divide(a: i32, b: {v: i32 | v != 0}) -> i32 { a / b }
                        ^^^^^^^^^^^^^^
                        Refinement predicate
```

**Implementation**: Parse docstring annotations with predicate logic:

```rust
/// # Preconditions
/// - `numerator` must be finite
/// - `denominator` must satisfy `denom != 0`
///
/// # Postconditions
/// - Returns `result` where `result * denominator ≈ numerator`
pub fn divide(numerator: f64, denominator: f64) -> f64
```

#### 2. Algorithm: Predicate Extraction (Abstract Syntax Tree + SMT)
Parse predicates into symbolic constraints:

```
Input: Function signature + docstring predicates
Output: Symbolic constraint formula

1. Parse docstring with regex/grammar (e.g., "x > 0 && x < 100")
2. Convert to SMT-LIB2 formula:
   (assert (and (> x 0) (< x 100)))
3. Build constraint graph: parameter → predicates
```

Use **Z3 SMT solver** to check constraint satisfiability.

#### 3. Algorithm: Runtime Check Synthesis
Generate guard code automatically:

```rust
// Original code (LLM generated)
fn calculate_discount(price: f64, percentage: f64) -> f64 {
    price * (percentage / 100.0)
}

// Synthesized code with refinement checks
fn calculate_discount(price: f64, percentage: f64) -> Result<f64, ValidationError> {
    // Synthesized precondition checks
    if !(price >= 0.0) {
        return Err(ValidationError::InvalidArgument {
            param: "price",
            constraint: "price >= 0.0",
            actual: price,
        });
    }
    if !(percentage >= 0.0 && percentage <= 100.0) {
        return Err(ValidationError::InvalidArgument {
            param: "percentage",
            constraint: "percentage in [0.0, 100.0]",
            actual: percentage,
        });
    }

    // Original logic
    let result = price * (percentage / 100.0);

    // Synthesized postcondition check
    debug_assert!(result >= 0.0 && result <= price);

    Ok(result)
}
```

#### 4. Algorithm: Weakest Precondition Calculus
Use Hoare logic to infer necessary preconditions from postconditions:

```
Given: Postcondition Q and statement S
Compute: Weakest precondition P such that {P} S {Q}

Example:
Statement: y = x / 2
Postcondition: y > 0
Weakest Precondition: x > 0  (inferred automatically)
```

**Formal Rules**:
```
WP(x := E, Q) = Q[E/x]  (substitution)
WP(S1; S2, Q) = WP(S1, WP(S2, Q))  (composition)
WP(if B then S1 else S2, Q) = (B ⇒ WP(S1, Q)) ∧ (¬B ⇒ WP(S2, Q))
```

### Implementation Sketch

**New HTTP Endpoints**:
1. `POST /constraint-type-predicate-extraction`
   - Body: Function source code + docstring
   - Returns: Extracted predicates in SMT-LIB2 format

2. `POST /constraint-guard-code-synthesis`
   - Body: Function + refinement types
   - Returns: Function with runtime checks inserted

3. `POST /constraint-weakest-precondition-inference`
   - Body: Function + desired postcondition
   - Returns: Inferred preconditions

**Core Implementation**:
```rust
// crates/parseltongue-core/src/refinement_types.rs

use z3::{Config, Context, Solver};

pub struct RefinementPredicate {
    pub parameter: String,
    pub constraint: String,  // e.g., "x > 0 && x < 100"
    pub smt_formula: String, // SMT-LIB2 format
}

pub struct RefinementTypeSynthesizer {
    z3_ctx: Context,
    solver: Solver,
}

impl RefinementTypeSynthesizer {
    pub fn extract_predicates_from_docstring(
        &self,
        docstring: &str
    ) -> Result<Vec<RefinementPredicate>> {
        // Parse "# Preconditions" section
        // Convert to SMT formulas
        // Check satisfiability
    }

    pub fn synthesize_guard_code(
        &self,
        function: &FunctionAST,
        predicates: &[RefinementPredicate]
    ) -> Result<String> {
        // Insert if-checks at function start
        // Convert SMT formulas to Rust conditionals
        // Wrap return type in Result<T, ValidationError>
    }

    pub fn infer_weakest_precondition(
        &self,
        postcondition: &str,
        function_body: &[Statement]
    ) -> Result<String> {
        // Backward symbolic execution
        // Apply WP calculus rules
        // Return inferred precondition
    }
}
```

### LLM Benefit

**Concrete Use Cases**:

1. **Automatic Contract Generation**
```python
# LLM generates:
def withdraw(account_balance: float, amount: float) -> float:
    return account_balance - amount

# Tool analyzes and suggests:
curl -X POST http://localhost:7777/constraint-weakest-precondition-inference \
  -d '{"postcondition": "result >= 0", "code": "..."}'

# Returns:
{
  "inferred_precondition": "amount <= account_balance",
  "synthesized_guard": "if amount > account_balance: raise InsufficientFundsError",
  "confidence": 0.95
}

# LLM rewrites with precondition check
```

2. **Invariant-Aware Code Review**
```bash
# Agent uploads proposed function
curl -X POST http://localhost:7777/constraint-type-predicate-extraction \
  -d '{"code": "fn get_element(arr: &[i32], idx: usize) -> i32 { arr[idx] }"}'

# Returns:
{
  "missing_constraints": [
    {"parameter": "idx", "required": "idx < arr.len()", "reason": "prevent out-of-bounds"}
  ],
  "suggested_fix": "fn get_element(arr: &[i32], idx: usize) -> Option<i32> { arr.get(idx).copied() }"
}
```

3. **Test Case Generation from Constraints**
```bash
# Extract constraints from function
curl /constraint-type-predicate-extraction?entity=rust:fn:calculate_discount

# Returns: [price >= 0, percentage in [0, 100]]

# LLM generates boundary value tests:
# - price = 0, percentage = 0 (lower boundary)
# - price = 100, percentage = 100 (upper boundary)
# - price = -1 (invalid, should error)
# - percentage = 101 (invalid, should error)
```

### Novel Insight

**Why This Is Creative**: This feature brings **formal methods from aerospace/medical software** (DO-178C, IEC 62304) into everyday programming. Instead of requiring programmers to learn theorem provers, it **automates** refinement type inference from natural language docstrings.

**Mathematical Foundation**:
- Liquid Types: Practical refinement types (Rondon et al., 2008)
- Hoare Logic: Foundation of program verification (1969)
- Z3 SMT Solver: State-of-the-art constraint solver from Microsoft Research

### Feasibility: **High**

**Implementation Complexity**:
- **LOW**: Docstring parsing (regex + grammar)
- **MEDIUM**: Z3 integration (mature Rust bindings exist: `z3-sys`)
- **HIGH**: WP calculus implementation (requires symbolic execution engine)

**Estimated Effort**: 3-4 weeks for MVP (predicate extraction + simple guard synthesis)

**Dependencies**:
- `z3-sys` (Z3 SMT solver bindings)
- Tree-sitter for AST parsing (already in Parseltongue)
- Docstring parser (custom grammar)

---

## Feature 3: Graph Neural Network Code Embeddings

### Feature Name
**graph-neural-network-code-embeddings**

### Problem Statement

Current code similarity is lexical (fuzzy string matching on entity names). This fails for:
- Semantically equivalent code with different naming (refactored functions)
- Identifying code clones (copy-paste with variable renaming)
- Finding analogous patterns across languages
- Understanding structural similarity (same graph topology, different syntax)

**Concrete Challenge**: LLM asked to "find similar functions" can't detect that two functions with different names implement the same algorithm (e.g., both are binary search).

### Technical Approach

#### 1. Data Structure: Code Graph Representation
Convert AST + dependency graph to attributed graph:

```rust
pub struct CodeGraphNode {
    pub node_id: String,
    pub node_type: NodeType,  // Function, Variable, Operator, ControlFlow
    pub attributes: HashMap<String, f64>,  // Features extracted from code
}

pub enum NodeType {
    Function, Method, Struct, Variable,
    Operator(OpKind),  // Add, Multiply, Compare, etc.
    ControlFlow(CFKind),  // If, Loop, Return, etc.
}

pub struct CodeGraphEdge {
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,  // DataFlow, ControlFlow, Calls, Uses
}
```

#### 2. Algorithm: Graph Neural Network (GNN) Architecture
Use **Graph Isomorphism Network (GIN)** for node embeddings:

```
Input: Graph G = (V, E) with node features X
Output: Node embeddings Z ∈ ℝ^{|V| × d}

Layer l update (GIN):
h_v^(l) = MLP^(l) ((1 + ε^(l)) · h_v^(l-1) + Σ_{u ∈ N(v)} h_u^(l-1))
                                               ^^^^^^^^^^^^^^^^^^^
                                               Aggregate neighbor features

Final embedding:
z_v = READOUT({h_v^(0), h_v^(1), ..., h_v^(L)})
    = Σ_{l=0}^L σ(h_v^(l))  (sum pooling with activation)
```

**Complexity**: O(L * E * d²) where L = layers, E = edges, d = embedding dimension

#### 3. Algorithm: Code Clone Detection (Cosine Similarity)
```
1. Compute embeddings for all entities: z_1, z_2, ..., z_n
2. For query entity z_q, compute similarity:
   sim(z_q, z_i) = (z_q · z_i) / (||z_q|| * ||z_i||)  (cosine similarity)
3. Return top-k entities with highest similarity (threshold > 0.85)
```

#### 4. Training: Contrastive Learning (SimCLR)
Learn embeddings without labels using self-supervision:

```
1. Create positive pairs:
   - (original_function, same_function_with_renamed_variables)
   - (original_function, semantically_equivalent_refactored_version)

2. Create negative pairs:
   - (function_A, random_function_B) where A and B are unrelated

3. Loss function (NT-Xent):
   L = -log( exp(sim(z_i, z_j) / τ) / Σ_{k≠i} exp(sim(z_i, z_k) / τ) )

   Where τ = temperature parameter (e.g., 0.5)

4. Optimize with SGD to maximize similarity for positive pairs,
   minimize for negative pairs
```

#### 5. Feature Engineering: Node Attributes
Extract node-level features:

```rust
pub fn extract_node_features(node: &ASTNode) -> Vec<f64> {
    vec![
        // Structural features
        node.depth as f64,
        node.num_children as f64,
        node.degree_centrality(),

        // Syntactic features
        if matches!(node.node_type, NodeType::Operator(_)) { 1.0 } else { 0.0 },
        if matches!(node.node_type, NodeType::ControlFlow(_)) { 1.0 } else { 0.0 },

        // Semantic features (from static analysis)
        node.cyclomatic_complexity() as f64,
        node.nesting_depth as f64,

        // One-hot encoding for node type (sparse)
        // ... encode as multi-hot vector
    ]
}
```

### Implementation Sketch

**New HTTP Endpoints**:
1. `GET /graph-embedding-vector-compute?entity=X`
   - Returns: 128-dimensional embedding vector for entity

2. `GET /graph-similarity-search-nearest?entity=X&top_k=10`
   - Returns: Most similar entities ranked by cosine similarity

3. `GET /graph-clone-detection-cluster?threshold=0.90`
   - Returns: Clusters of likely code clones

4. `POST /graph-embedding-model-training`
   - Trigger incremental training on new data

**Core Implementation**:
```rust
// crates/parseltongue-core/src/graph_embeddings.rs

use tch::{nn, Tensor, Device};  // PyTorch bindings for Rust

pub struct GINLayer {
    mlp: nn::Sequential,
    epsilon: Tensor,
}

impl GINLayer {
    pub fn forward(&self, node_features: &Tensor, adj_matrix: &Tensor) -> Tensor {
        // Aggregate neighbor features
        let aggregated = adj_matrix.matmul(node_features);

        // Combine with self features
        let combined = (1.0 + &self.epsilon) * node_features + aggregated;

        // Apply MLP
        self.mlp.forward(&combined)
    }
}

pub struct GraphEmbeddingModel {
    layers: Vec<GINLayer>,
    readout: nn::Linear,
}

impl GraphEmbeddingModel {
    pub fn compute_embedding(&self, graph: &CodeGraph) -> Tensor {
        // Convert graph to tensors
        let node_features = graph.node_features_to_tensor();
        let adj_matrix = graph.adjacency_matrix_to_tensor();

        // Forward pass through GIN layers
        let mut h = node_features;
        for layer in &self.layers {
            h = layer.forward(&h, &adj_matrix);
        }

        // Global pooling (sum all node embeddings)
        h.sum_dim_intlist(&[0], false, tch::Kind::Float)
    }
}
```

**Database Schema**:
```sql
:create CodeEmbeddings {
    entity_key: String
    =>
    embedding_vector: String,  -- JSON array of floats
    model_version: String,
    last_updated: String
}
```

### LLM Benefit

**Concrete Use Cases**:

1. **Semantic Code Search**
```bash
# LLM needs to find "functions similar to binary search"
curl "http://localhost:7777/graph-similarity-search-nearest?entity=rust:fn:binary_search:src_search_rs:10-30&top_k=5"

# Returns:
[
  {"entity": "rust:fn:find_element_sorted", "similarity": 0.94, "reason": "Similar control flow + divide-and-conquer pattern"},
  {"entity": "rust:fn:locate_index", "similarity": 0.89},
  {"entity": "python:def:bisect_search", "similarity": 0.87, "cross_language": true},
  ...
]
```

2. **Code Clone Refactoring**
```bash
curl "http://localhost:7777/graph-clone-detection-cluster?threshold=0.92"

# Returns:
{
  "clusters": [
    {
      "cluster_id": 1,
      "entities": [
        "rust:fn:parse_json_config:src_config_rs:50-80",
        "rust:fn:load_yaml_settings:src_settings_rs:100-130"
      ],
      "similarity": 0.95,
      "suggestion": "Extract common parsing logic into shared function"
    }
  ]
}
```

3. **Cross-Language Pattern Detection**
```bash
# Find equivalent implementations across languages
curl "http://localhost:7777/graph-similarity-search-nearest?entity=rust:fn:authenticate&cross_language=true"

# Returns:
[
  {"entity": "python:def:verify_credentials", "similarity": 0.88, "language": "python"},
  {"entity": "java:method:checkAuth", "similarity": 0.85, "language": "java"}
]

# LLM learns from existing Python implementation to improve Rust version
```

4. **Anomaly Detection**
```bash
# Find outlier functions (dissimilar to everything else)
curl "http://localhost:7777/graph-embedding-anomaly-detection?threshold=0.3"

# Returns entities with low average similarity to codebase
# Often indicates dead code, technical debt, or overly complex logic
```

### Novel Insight

**Why This Is Creative**: Applying **graph neural networks from drug discovery** (molecular graph classification) to code analysis. Just as GNNs identify similar molecular structures, they can identify similar code structures **independent of variable names**. This is fundamentally different from token-based embeddings (like CodeBERT) because it preserves graph topology.

**Mathematical Foundation**:
- Graph Isomorphism Network (Xu et al., 2019): Provably maximally expressive GNN
- SimCLR contrastive learning (Chen et al., 2020): Self-supervised training
- Weisfeiler-Lehman graph kernel: Theoretical foundation for GNN expressiveness

### Feasibility: **Medium**

**Implementation Complexity**:
- **MEDIUM**: Graph construction from AST (already have tree-sitter)
- **HIGH**: GNN implementation (use `tch-rs` for PyTorch bindings)
- **HIGH**: Training pipeline (requires dataset of labeled code clones)
- **MEDIUM**: Embedding storage/retrieval (CozoDB or vector DB like Qdrant)

**Estimated Effort**: 4-6 weeks for MVP (using pretrained model like CodeBERT, then fine-tuning)

**Dependencies**:
- `tch-rs` (Rust PyTorch bindings)
- Pretrained model (e.g., CodeBERT from Hugging Face)
- Vector database (Qdrant) for efficient k-NN search
- GPU for training (optional, can use CPU for inference)

---

## Feature 4: Causal Impact Analysis for Code Changes

### Feature Name
**causal-impact-analysis-code-changes**

### Problem Statement

**Correlation ≠ Causation**: Current blast radius analysis shows structural dependencies, but not causal relationships:
- If function A calls function B, does changing A **cause** B to break?
- Temporal coupling shows entities changing together, but is it causal or coincidental?
- Which code changes **actually cause** bugs vs just correlated?

**Concrete Challenge**: LLM modifies authentication logic, and tests start failing in seemingly unrelated modules. Is there a causal path, or is this a flaky test?

### Technical Approach

#### 1. Causal Graph Construction (Structural Causal Model)
Build directed acyclic graph (DAG) of causal relationships:

```rust
pub struct CausalNode {
    pub entity_key: String,
    pub observed_variables: Vec<ObservableMetric>,
}

pub enum ObservableMetric {
    TestPassRate(f64),          // % tests passing
    CyclomaticComplexity(u32),  // McCabe complexity
    ChangeFrequency(f64),       // Changes per week
    BugCount(u32),              // Bugs reported
}

pub struct CausalEdge {
    pub source: String,
    pub target: String,
    pub causal_strength: f64,  // Estimated effect size
    pub confounders: Vec<String>,  // Confounding variables
}
```

#### 2. Algorithm: Causal Discovery (PC Algorithm)
Discover causal relationships from observational data using conditional independence tests:

```
Input: Dataset D with variables {X1, X2, ..., Xn}
Output: Causal DAG G

Phase 1: Skeleton Discovery
1. Start with fully connected undirected graph
2. For each pair (Xi, Xj):
   - Test independence Xi ⊥ Xj | ∅ (unconditional)
   - If independent, remove edge
3. For each triple (Xi, Xk, Xj):
   - Test Xi ⊥ Xj | Xk (conditional independence)
   - If independent, remove edge Xi—Xj, record Xk as separator

Phase 2: Edge Orientation
1. For each collider pattern Xi — Xk — Xj where Xi and Xj not adjacent:
   - Orient as Xi → Xk ← Xj (Xk is collider)
2. Apply orientation rules to avoid cycles:
   - If Xi → Xk and Xk — Xj, orient as Xk → Xj
   - If Xi → Xk and Xi → Xj, avoid Xi → Xk ← Xj (would create cycle)
```

**Complexity**: O(n² * k) where n = variables, k = max conditioning set size

#### 3. Algorithm: Causal Effect Estimation (Do-Calculus)
Estimate effect of interventions using Pearl's do-calculus:

```
Query: What is P(Y | do(X = x))?  (effect of setting X to x on Y)

Adjustment Formula (back-door criterion):
P(Y | do(X = x)) = Σ_z P(Y | X = x, Z = z) * P(Z = z)

Where Z is a set of variables that blocks all back-door paths
```

**Example**:
```
Question: "What's the impact of increasing cyclomatic_complexity on bug_count?"

Causal DAG:
  CyclomaticComplexity → BugCount
  ChangeFrequency → CyclomaticComplexity
  ChangeFrequency → BugCount (confounding path!)

Adjustment Set: {ChangeFrequency}
Effect: E[BugCount | do(CC = high)] - E[BugCount | do(CC = low)]
      = 0.42 bugs per unit complexity (after adjusting for change frequency)
```

#### 4. Algorithm: Counterfactual Reasoning
Answer "what if" questions about past changes:

```
Question: "If we had NOT refactored module X last month, would test coverage be higher?"

Abduction-Action-Prediction (3-step process):
1. Abduction: Infer latent variables from observed data
   - Observed: TestCoverage = 0.75, Refactored = true
   - Infer: Complexity_hidden = high (latent variable)

2. Action: Intervene on causal graph
   - do(Refactored = false)

3. Prediction: Compute outcome under intervention
   - E[TestCoverage | do(Refactored = false), Complexity_hidden = high]
   - Result: TestCoverage = 0.82 (counterfactual estimate)
```

### Implementation Sketch

**New HTTP Endpoints**:
1. `GET /causal-graph-structure-discovery`
   - Returns: Causal DAG of entities with directed edges

2. `GET /causal-effect-intervention-estimate?cause=X&effect=Y`
   - Returns: Estimated causal effect size with confidence interval

3. `POST /causal-counterfactual-query-what-if`
   - Body: {"intervention": "do(refactor_module)", "outcome": "test_pass_rate"}
   - Returns: Counterfactual prediction

4. `GET /causal-root-cause-attribution?event=test_failure`
   - Returns: Most likely causal path leading to event (backwards causal tracing)

**Core Implementation**:
```rust
// crates/parseltongue-core/src/causal_analysis.rs

use statrs::distribution::ChiSquared;

pub struct CausalDiscoveryEngine {
    pub skeleton: Graph,
    pub significance_level: f64,  // α for independence tests
}

impl CausalDiscoveryEngine {
    /// PC algorithm for causal discovery
    pub fn discover_causal_dag(&self, data: &Dataset) -> CausalDAG {
        // Phase 1: Build skeleton
        let mut skeleton = self.build_skeleton(data);

        // Phase 2: Orient edges
        self.orient_edges(&mut skeleton, data);

        CausalDAG::from_skeleton(skeleton)
    }

    fn test_conditional_independence(
        &self,
        x: &Variable,
        y: &Variable,
        conditioning_set: &[Variable],
        data: &Dataset,
    ) -> bool {
        // Partial correlation test
        let r = self.partial_correlation(x, y, conditioning_set, data);
        let n = data.num_samples();
        let z = 0.5 * ((1.0 + r) / (1.0 - r)).ln();  // Fisher Z-transform
        let se = 1.0 / ((n - conditioning_set.len() - 3) as f64).sqrt();
        let z_score = z / se;

        // Two-tailed test
        let chi_sq = ChiSquared::new(1.0).unwrap();
        let p_value = 1.0 - chi_sq.cdf(z_score * z_score);

        p_value > self.significance_level  // Accept independence
    }
}

pub struct CausalEffectEstimator {
    pub dag: CausalDAG,
}

impl CausalEffectEstimator {
    /// Estimate average treatment effect (ATE)
    pub fn estimate_ate(
        &self,
        treatment: &str,
        outcome: &str,
        data: &Dataset,
    ) -> Result<f64> {
        // Find adjustment set (back-door criterion)
        let adjustment_set = self.find_adjustment_set(treatment, outcome)?;

        // Stratify by adjustment variables and average
        let mut effect = 0.0;
        for stratum in data.stratify_by(&adjustment_set) {
            let p_stratum = stratum.probability();
            let e_outcome_treated = stratum.mean(outcome, treatment, true);
            let e_outcome_control = stratum.mean(outcome, treatment, false);
            effect += p_stratum * (e_outcome_treated - e_outcome_control);
        }

        Ok(effect)
    }
}
```

**Database Schema**:
```sql
:create CausalNodes {
    entity_key: String
    =>
    complexity: Float,
    change_freq: Float,
    test_pass_rate: Float,
    bug_count: Int,
    last_updated: String
}

:create CausalEdges {
    source_entity: String,
    target_entity: String
    =>
    causal_strength: Float,
    confidence_interval: String,
    confounders: String
}
```

### LLM Benefit

**Concrete Use Cases**:

1. **Root Cause Analysis**
```bash
# Test suite suddenly failing after commit
curl "http://localhost:7777/causal-root-cause-attribution?event=test_failure&commit=abc123"

# Returns:
{
  "causal_chain": [
    {"entity": "rust:fn:parse_config", "contribution": 0.72, "reason": "Modified error handling"},
    {"entity": "rust:struct:Config", "contribution": 0.18, "reason": "Changed field types"},
    {"entity": "rust:fn:validate", "contribution": 0.10, "reason": "Indirect dependency"}
  ],
  "confidence": 0.89
}

# LLM: Focus debugging on parse_config first (highest causal contribution)
```

2. **What-If Refactoring Simulation**
```bash
# Before refactoring, predict impact
curl -X POST http://localhost:7777/causal-counterfactual-query-what-if \
  -d '{
    "intervention": "do(reduce_complexity, module=UserService, from=25, to=10)",
    "outcome": "bug_count"
  }'

# Returns:
{
  "current_state": {"bug_count": 12},
  "counterfactual": {"bug_count": 7},
  "estimated_benefit": -5,  // 5 fewer bugs
  "confidence_interval": [-8, -2]
}

# LLM: Strong evidence that refactoring reduces bugs. Recommend proceeding.
```

3. **Causal Feature Selection for Testing**
```bash
# Which metrics causally affect test pass rate?
curl "http://localhost:7777/causal-graph-structure-discovery?outcome=test_pass_rate"

# Returns:
{
  "causal_parents": [
    {"variable": "code_coverage", "strength": 0.65},
    {"variable": "cyclomatic_complexity", "strength": -0.42},  // Negative effect
    {"variable": "team_size", "strength": 0.15}
  ],
  "non_causal_correlates": [
    {"variable": "file_size", "correlation": 0.50, "reason": "Mediated by complexity"}
  ]
}

# LLM: Focus testing efforts on improving coverage and reducing complexity (causal factors)
#      Ignore file size (spurious correlation)
```

### Novel Insight

**Why This Is Creative**: Applies **causal inference from epidemiology and econometrics** (Judea Pearl's causality framework) to software engineering. Instead of asking "what correlates with bugs?" (traditional statistical analysis), it asks "what **causes** bugs?" This enables **counterfactual reasoning** ("what would have happened if..."), impossible with correlation-based approaches.

**Mathematical Foundation**:
- Structural Causal Models (Pearl, 2000): Formal framework for causality
- PC Algorithm (Spirtes et al., 2000): Causal discovery from observational data
- Do-Calculus (Pearl, 1995): Algebra for causal interventions

### Feasibility: **Medium**

**Implementation Complexity**:
- **MEDIUM**: Dataset collection (augment temporal events with metrics)
- **HIGH**: PC algorithm implementation (~800 LOC with independence tests)
- **MEDIUM**: Do-calculus effect estimation (well-defined formulas)
- **HIGH**: Counterfactual reasoning (requires structural equations)

**Estimated Effort**: 4-5 weeks for MVP (causal discovery + basic effect estimation)

**Dependencies**:
- `statrs` (statistical distributions for independence tests)
- Historical metrics database (complexity, test results, bug counts)
- Graphical models library (DAG manipulation)

---

## Feature 5: Specification Mining from Code Examples

### Feature Name
**specification-mining-code-examples**

### Problem Statement

**LLMs lack formal specifications**. When generating code, they guess at:
- API usage patterns (method call ordering, required initializations)
- Invariants (data structure properties that must hold)
- Temporal properties (state machine protocols)
- Error handling requirements

**Concrete Challenge**: LLM generates code using a library API incorrectly (e.g., calls `send()` before `connect()` for a socket). No formal spec exists, only scattered examples.

### Technical Approach

#### 1. Algorithm: Frequent API Usage Pattern Mining (Sequence Mining)
Extract common API call sequences from codebase:

```
Input: Code examples using library L
Output: Frequent temporal patterns

1. Extract call sequences for each function:
   Example 1: [Socket::new(), connect(), send(), close()]
   Example 2: [Socket::new(), connect(), send(), send(), close()]
   Example 3: [Socket::new(), bind(), listen(), accept()]

2. Apply PrefixSpan algorithm (sequential pattern mining):
   - Find patterns with support ≥ min_support (e.g., 80%)
   - Prune infrequent subsequences

3. Output: Frequent patterns
   - Pattern A: [new(), connect(), send()] (support = 0.90)
   - Pattern B: [new(), connect(), close()] (support = 0.85)
   - Pattern C: [new(), bind(), listen()] (support = 0.30)  // Server-side pattern
```

**Complexity**: O(n * m²) where n = sequences, m = avg sequence length

#### 2. Algorithm: Finite State Automaton Inference (RPNI Algorithm)
Infer state machine from examples using Regular Positive and Negative Inference:

```
Input:
  - Positive examples: [P1, P2, ..., Pk] (valid sequences)
  - Negative examples: [N1, N2, ..., Nm] (invalid sequences, from bug reports)

Output: Minimal DFA accepting all positive, rejecting all negative

Algorithm (RPNI):
1. Build prefix tree acceptor from positive examples
2. Iteratively merge states while maintaining consistency:
   - Try merging states qi and qj
   - Check if resulting automaton still accepts all positive examples
   - Check if it rejects all negative examples
   - If yes, commit merge; otherwise, undo
3. Return minimal DFA
```

**Complexity**: O(k * n³) where k = iterations, n = states

#### 3. Algorithm: Daikon-style Invariant Detection
Detect likely invariants through dynamic analysis:

```
Input: Execution traces {(input, output, intermediate_states)}
Output: Candidate invariants

1. Instrument code to log variable values at program points
2. For each program point, collect observations:
   - x = [5, 10, 15, 20, 25]
   - y = [1, 2, 3, 4, 5]
3. Test candidate invariants:
   - Linear: y = ax + b → Test: y = 0.2x - 0.0 ✓
   - Comparison: x > 0 → Test: ∀ observations, x > 0 ✓
   - Bounds: x ∈ [5, 25] → Test: ✓
4. Return invariants that hold across all traces
```

**Supported Invariant Templates**:
```
- Unary: x > 0, x != null, x ∈ [min, max]
- Binary: x == y, x < y, x = y + c
- Array: arr.length == size, ∀i. arr[i] > 0
- Collections: set.contains(elem) ⇒ map.contains_key(elem)
```

#### 4. Algorithm: Precondition/Postcondition Inference (Houdini)
Iteratively refine candidate contracts:

```
Input:
  - Function f
  - Candidate preconditions {P1, P2, ..., Pk}
  - Candidate postconditions {Q1, Q2, ..., Qm}

Output: Validated contracts

Algorithm:
1. Assume all candidates true (optimistic)
2. For each test case:
   - Check if precondition holds at entry
   - Execute function
   - Check if postcondition holds at exit
   - If violated, remove that candidate
3. Repeat until convergence (no more candidates removed)
4. Return surviving candidates as likely contracts
```

**Example**:
```rust
// Inferred preconditions for divide():
// ✓ divisor != 0        (survived all tests)
// ✗ divisor > 0         (violated by test with divisor = -5)
// ✓ numerator is finite (survived all tests)
```

### Implementation Sketch

**New HTTP Endpoints**:
1. `POST /specification-pattern-mining-api-usage`
   - Body: List of file paths using target API
   - Returns: Frequent call sequences with support metrics

2. `POST /specification-fsm-inference-protocol`
   - Body: Positive/negative example traces
   - Returns: State machine diagram in DOT format

3. `POST /specification-invariant-detection-dynamic`
   - Body: Execution traces (JSON logs)
   - Returns: Candidate invariants with confidence scores

4. `GET /specification-contract-suggestions-houdini?entity=X`
   - Returns: Inferred preconditions/postconditions for function

**Core Implementation**:
```rust
// crates/parseltongue-core/src/spec_mining.rs

pub struct APIUsagePattern {
    pub sequence: Vec<String>,  // ["new", "connect", "send"]
    pub support: f64,           // Fraction of examples matching
    pub confidence: f64,        // P(sequence valid | usage)
}

pub struct SpecificationMiner {
    pub min_support: f64,
}

impl SpecificationMiner {
    /// PrefixSpan sequential pattern mining
    pub fn mine_api_patterns(&self, examples: &[CallSequence]) -> Vec<APIUsagePattern> {
        let mut patterns = Vec::new();
        let db_size = examples.len() as f64;

        // Find frequent length-1 patterns
        let mut freq_items = self.find_frequent_items(examples);

        // Recursively grow patterns
        while !freq_items.is_empty() {
            let mut extended = Vec::new();

            for pattern in &freq_items {
                // Project database to suffix starting after pattern
                let projected = self.project_database(examples, pattern);

                // Find frequent extensions
                for extension in self.find_frequent_items(&projected) {
                    let mut new_pattern = pattern.clone();
                    new_pattern.sequence.push(extension);

                    let support = self.count_support(&new_pattern, examples) / db_size;
                    if support >= self.min_support {
                        extended.push(new_pattern);
                    }
                }
            }

            patterns.extend(freq_items);
            freq_items = extended;
        }

        patterns
    }

    /// RPNI algorithm for DFA inference
    pub fn infer_fsm(
        &self,
        positive_examples: &[Vec<String>],
        negative_examples: &[Vec<String>],
    ) -> FiniteStateAutomaton {
        // Build prefix tree acceptor
        let mut pta = self.build_prefix_tree(positive_examples);

        // Merge states while maintaining consistency
        loop {
            let merge_candidate = self.find_merge_candidate(&pta);
            match merge_candidate {
                Some((q1, q2)) => {
                    let merged = self.try_merge(&pta, q1, q2);
                    if self.is_consistent(&merged, positive_examples, negative_examples) {
                        pta = merged;
                    } else {
                        break;  // No more valid merges
                    }
                }
                None => break,
            }
        }

        pta
    }

    /// Daikon-style invariant detection
    pub fn detect_invariants(&self, traces: &[ExecutionTrace]) -> Vec<Invariant> {
        let mut candidates = self.generate_candidate_invariants(traces);

        // Filter to those that hold on all traces
        candidates.retain(|inv| {
            traces.iter().all(|trace| inv.check(trace))
        });

        candidates
    }
}
```

**Database Schema**:
```sql
:create APIPatterns {
    pattern_id: String,
    sequence: String,  -- JSON array of method names
    support: Float,
    last_mined: String
}

:create InferredInvariants {
    program_point: String,  -- Entity key + line number
    invariant_formula: String,
    confidence: Float,
    counterexamples: Int  -- How many violations observed
}
```

### LLM Benefit

**Concrete Use Cases**:

1. **API Usage Guidance**
```bash
# LLM needs to use tokio::net::TcpStream API
curl -X POST http://localhost:7777/specification-pattern-mining-api-usage \
  -d '{"api": "tokio::net::TcpStream", "examples_from": "codebase"}'

# Returns:
{
  "frequent_patterns": [
    {
      "sequence": ["TcpStream::connect", "write_all", "read", "shutdown"],
      "support": 0.87,
      "template": "async fn use_stream() -> Result<()> { ... }"
    },
    {
      "sequence": ["TcpStream::connect", "split", "write_half.write", "read_half.read"],
      "support": 0.65
    }
  ]
}

# LLM uses pattern 1 (highest support) as template for generated code
```

2. **State Machine Protocol Enforcement**
```bash
# Mine state machine for Socket lifecycle
curl -X POST http://localhost:7777/specification-fsm-inference-protocol \
  -d '{
    "positive": [
      ["new", "connect", "send", "close"],
      ["new", "connect", "recv", "close"]
    ],
    "negative": [
      ["new", "send", "connect"],  // send before connect (bug!)
      ["new", "close", "send"]      // use after close (bug!)
    ]
  }'

# Returns:
{
  "fsm": {
    "states": ["Init", "Connected", "Closed"],
    "transitions": [
      {"from": "Init", "via": "connect", "to": "Connected"},
      {"from": "Connected", "via": "send|recv", "to": "Connected"},
      {"from": "Connected", "via": "close", "to": "Closed"}
    ],
    "errors": [
      {"state": "Init", "illegal_action": "send", "reason": "Must connect first"},
      {"state": "Closed", "illegal_action": "*", "reason": "Socket closed"}
    ]
  }
}

# LLM generates code that follows FSM protocol
```

3. **Invariant-Guided Test Generation**
```bash
# Detect invariants for binary search function
curl "http://localhost:7777/specification-invariant-detection-dynamic?entity=rust:fn:binary_search"

# Returns:
{
  "invariants": [
    {
      "location": "binary_search:entry",
      "formula": "arr is sorted",
      "confidence": 1.0,
      "type": "precondition"
    },
    {
      "location": "binary_search:loop",
      "formula": "low <= high",
      "confidence": 0.98,
      "type": "loop_invariant"
    },
    {
      "location": "binary_search:exit",
      "formula": "result == None OR arr[result] == target",
      "confidence": 1.0,
      "type": "postcondition"
    }
  ]
}

# LLM generates tests that:
# 1. Ensure arr is sorted (precondition)
# 2. Check loop invariant holds (mutation testing)
# 3. Verify result correctness (postcondition)
```

4. **Contract Suggestion for Undocumented APIs**
```bash
curl "http://localhost:7777/specification-contract-suggestions-houdini?entity=rust:fn:process_payment"

# Returns:
{
  "inferred_preconditions": [
    "amount > 0.0",
    "!account_id.is_empty()",
    "payment_method != PaymentMethod::Unknown"
  ],
  "inferred_postconditions": [
    "result.is_ok() ==> transaction_id is set",
    "result.is_err() ==> balance unchanged"
  ],
  "confidence": 0.89,
  "tested_on": "127 existing call sites"
}

# LLM adds these as docstring contracts
```

### Novel Insight

**Why This Is Creative**: Combines **data mining** (PrefixSpan), **automata theory** (RPNI), and **dynamic analysis** (Daikon) to automatically extract specifications that developers forgot to write. This is like **reverse engineering documentation** from code rather than code from docs.

**Mathematical Foundation**:
- PrefixSpan (Pei et al., 2001): Sequential pattern mining
- RPNI Algorithm (Oncina & García, 1992): Grammatical inference
- Daikon Invariant Detector (Ernst et al., 2001): Dynamic invariant inference
- Houdini (Flanagan & Leino, 2001): Annotation assistant

### Feasibility: **Medium-High**

**Implementation Complexity**:
- **MEDIUM**: PrefixSpan pattern mining (well-documented algorithm, ~600 LOC)
- **HIGH**: RPNI FSM inference (complex state merging logic, ~800 LOC)
- **HIGH**: Daikon-style invariant detection (requires extensive template library)
- **MEDIUM**: Houdini contract refinement (iterative elimination logic)

**Estimated Effort**: 5-6 weeks for MVP (focus on pattern mining + basic invariants)

**Dependencies**:
- Execution trace collection (instrumentation via tree-sitter or LLVM)
- Pattern mining library (custom implementation or adapt `sequoia` crate)
- SMT solver for invariant checking (Z3, already used in Feature 2)

---

## Comparative Summary

| Feature | Problem Solved | Key Algorithm | LLM Benefit | Feasibility | Estimated Effort |
|---------|---------------|---------------|-------------|-------------|------------------|
| **1. Temporal Pattern Inference** | Code evolution blindness | Exponential decay + HMM | Risk-aware refactoring, change prediction | Medium | 2-3 weeks |
| **2. Constraint Type Refinement** | Missing contracts/preconditions | Liquid types + Z3 SMT | Auto-generate input validation, boundary tests | High | 3-4 weeks |
| **3. GNN Code Embeddings** | Lexical-only similarity | Graph Isomorphism Network | Semantic code search, clone detection | Medium | 4-6 weeks |
| **4. Causal Impact Analysis** | Correlation vs causation confusion | Pearl's causal inference + PC algorithm | Root cause diagnosis, counterfactual "what-if" | Medium | 4-5 weeks |
| **5. Specification Mining** | Undocumented API usage patterns | PrefixSpan + RPNI + Daikon | Correct API usage, protocol compliance | Medium-High | 5-6 weeks |

## Cross-Feature Synergies

### Synergy 1: Temporal + Causal Analysis
- **Use Case**: "Which recent changes **caused** the test regression?"
- **Method**: Combine temporal change events (Feature 1) with causal root-cause tracing (Feature 4)
- **Query**: `/causal-root-cause-attribution?event=test_failure&temporal_window=30d`

### Synergy 2: GNN Embeddings + Specification Mining
- **Use Case**: "Find similar API usage patterns across the codebase"
- **Method**: Use GNN embeddings (Feature 3) to cluster structurally similar code, then mine specifications (Feature 5) from each cluster
- **Benefit**: Discover domain-specific patterns (e.g., all authentication flows follow similar protocol)

### Synergy 3: Constraint Refinement + Specification Mining
- **Use Case**: "Auto-generate comprehensive test suite"
- **Method**:
  1. Mine invariants from execution traces (Feature 5: Daikon)
  2. Convert to refinement type predicates (Feature 2: Liquid types)
  3. Synthesize boundary-value tests from constraints
- **Example**: Invariant `x ∈ [0, 100]` → Generate tests for `x = -1, 0, 1, 50, 99, 100, 101`

### Synergy 4: Temporal + GNN for Clone Evolution
- **Use Case**: "Track how code clones diverge over time"
- **Method**:
  1. Detect clones with GNN similarity (Feature 3)
  2. Track temporal evolution of each clone (Feature 1)
  3. Alert when clones diverge (one changes, others don't)
- **Benefit**: Prevent inconsistent bug fixes across copy-paste code

---

## Critical Files for Implementation

Based on the Parseltongue architecture, here are the most critical files for implementing these features:

### 1. Core Type Definitions
- **File**: `crates/parseltongue-core/src/entities.rs`
- **Reason**: Extend `CodeEntity` struct with new fields (temporal metrics, embeddings, causal variables). Currently has 47KB of entity types - need to add `temporal_metrics: Option<TemporalMetrics>`, `embedding_vector: Option<Vec<f64>>`, `causal_attributes: Option<CausalAttributes>`.

### 2. Database Storage Layer
- **File**: `crates/parseltongue-core/src/storage/cozo_client.rs`
- **Reason**: Add new CozoDB schema relations (TemporalEvents, CausalEdges, CodeEmbeddings, APIPatterns, InferredInvariants). Currently 1668 lines with methods like `create_schema()` - extend with `create_temporal_events_schema()`, `create_causal_edges_schema()`, etc.

### 3. HTTP Route Definitions
- **File**: `crates/pt08-http-code-query-server/src/route_definition_builder_module.rs`
- **Reason**: Register new endpoint handlers. Pattern follows existing routes like `/smart-context-token-budget` - add routes for temporal analysis, causal queries, embedding search.

### 4. Existing Smart Context Handler
- **File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/smart_context_token_budget_handler.rs`
- **Reason**: Template for implementing new handlers. Shows greedy selection algorithm (lines 128-248), token estimation (line 256), graph traversal pattern. Use as reference for implementing temporal/causal/embedding handlers.

### 5. New Feature Implementation Modules
- **Location**: Create new files in `crates/parseltongue-core/src/`
  - `temporal_analysis.rs` - Feature 1 implementation
  - `refinement_types.rs` - Feature 2 implementation
  - `graph_embeddings.rs` - Feature 3 implementation
  - `causal_inference.rs` - Feature 4 implementation
  - `spec_mining.rs` - Feature 5 implementation
- **Reason**: Follow existing module pattern (e.g., `temporal.rs` already exists with 23KB of temporal versioning code - extend this for Feature 1).

---

## Conclusion

These five features represent **10-year horizon innovations** that bring cutting-edge research from formal methods, machine learning, and causal inference into practical LLM-assisted code generation. Each feature:

1. **Solves a real LLM limitation** (not just incremental improvements)
2. **Has strong theoretical foundations** (published algorithms, mathematical proofs)
3. **Is implementable in Rust** (with existing crates and reasonable effort)
4. **Integrates naturally with Parseltongue** (extends CozoDB schema, adds HTTP endpoints)
5. **Provides concrete LLM benefits** (better code generation, fewer bugs, smarter refactoring)

**Recommended Implementation Order**:
1. **Feature 2** (Constraint Refinement) - Highest feasibility, immediate LLM benefit
2. **Feature 1** (Temporal Analysis) - Builds on existing temporal infrastructure
3. **Feature 5** (Specification Mining) - High practical value for undocumented APIs
4. **Feature 3** (GNN Embeddings) - Requires ML infrastructure but huge semantic search improvement
5. **Feature 4** (Causal Analysis) - Most research-level, but revolutionary for debugging

**Total Estimated Effort**: 18-24 weeks for all five features (MVP versions)

---

**Document Status**: Research Proposal
**Next Steps**: Prioritize features, create implementation plan, allocate resources
**Contact**: Open for discussion at https://github.com/that-in-rust/parseltongue-dependency-graph-generator/issues
