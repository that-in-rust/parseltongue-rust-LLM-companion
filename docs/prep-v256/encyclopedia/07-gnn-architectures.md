# Graph Neural Network Architectures for Code Graphs

**Category:** Deep Learning on Graphs
**Target:** 30+ architectures documented
**Purpose:** Learn code representations, detect patterns, enable ML-based analysis

---

## Overview

GNNs answer: **"How do we learn from graph-structured code?"**

Applications:
- Vulnerability detection
- Code similarity
- Bug prediction
- Type inference
- Code summarization

---

## 1. FOUNDATIONAL ARCHITECTURES

### 1.1 Graph Convolutional Network (GCN)

```
Paper: Kipf & Welling (2017)
Idea: Spectral convolution approximation
```

**Message Passing:**
```
h_v^(l+1) = σ( Σ (1/√(d_u × d_v)) × W^(l) × h_u^(l) )
           u∈N(v)
```

**Implementation:**
```rust
// Pseudocode for GCN layer
struct GCNLayer {
    weight: Matrix,  // Learnable weights
}

impl GCNLayer {
    fn forward(&self, features: &Matrix, adj: &Matrix) -> Matrix {
        // Normalized adjacency
        let norm_adj = normalize(adj);

        // Aggregate neighbors
        let aggregated = norm_adj * features;

        // Linear transformation + activation
        (aggregated * &self.weight).relu()
    }
}
```

**Code Applications:**
- Function embedding
- Module classification
- Vulnerability detection (Devign uses GCN)

**Pros:** Simple, fast
**Cons:** Over-smoothing with depth, treats all neighbors equally

---

### 1.2 Graph Attention Network (GAT)

```
Paper: Veličković et al. (2018)
Idea: Attention-weighted neighbor aggregation
```

**Message Passing:**
```
α_ij = softmax(LeakyReLU(a^T [Wh_i || Wh_j]))
h_v^(l+1) = σ( Σ α_ij × W × h_j^(l) )
           j∈N(v)
```

**Implementation:**
```rust
struct GATLayer {
    weight: Matrix,
    attention: Vector,
}

impl GATLayer {
    fn forward(&self, features: &Matrix, adj: &SparseMatrix) -> Matrix {
        let transformed = features * &self.weight;

        // Compute attention scores
        let attention = self.compute_attention(&transformed, adj);

        // Weighted aggregation
        attention * transformed
    }
}
```

**Code Applications:**
- **Weighted dependencies**: Some calls more important than others
- **Bug localization**: Attention highlights suspicious code
- **Code review**: Which neighbors matter?

**Pros:** Learns importance weights
**Cons:** More expensive than GCN

---

### 1.3 GraphSAGE (SAmple and aggreGatE)

```
Paper: Hamilton et al. (2017)
Idea: Sample neighbors + aggregate inductively
```

**Message Passing:**
```
h_v^(l+1) = σ(W × [AGGREGATE({h_u^(l), u∈S(v)}) || h_v^(l)])
```

**Aggregators:**
- Mean: Average neighbor embeddings
- LSTM: Sequential aggregation (random order)
- Pooling: Max-pool neighbor features

**Implementation:**
```rust
struct GraphSAGELayer {
    weight: Matrix,
    aggregator: Aggregator,
    sample_size: usize,
}

impl GraphSAGELayer {
    fn forward(&self, features: &Matrix, neighbors: &Neighbors) -> Matrix {
        let mut output = Vec::new();

        for v in features.rows() {
            // Sample neighbors
            let sampled = neighbors[v].sample(self.sample_size);

            // Aggregate
            let agg = self.aggregator.aggregate(&features[&sampled]);

            // Concatenate with self
            let combined = concatenate(&[agg, features[v].clone()]);

            // Transform
            output.push((&combined * &self.weight).relu());
        }

        Matrix::from_rows(output)
    }
}
```

**Code Applications:**
- **Inductive learning**: Works on new code not in training
- **Scalable**: Sample-based for large codebases
- **New repos**: No retraining needed

**Pros:** Inductive, scalable
**Cons:** Sampling loses information

---

### 1.4 Graph Isomorphism Network (GIN)

```
Paper: Xu et al. (2019)
Idea: Maximum expressive GNN
```

**Message Passing:**
```
h_v^(l+1) = MLP((1 + ε^(l)) × h_v^(l) + Σ h_u^(l))
                                   u∈N(v)
```

**Key Insight:** GIN is as powerful as WL graph kernel

**Code Applications:**
- **Graph classification**: Is this code vulnerable?
- **Clone detection**: Code similarity at graph level

---

## 2. ADVANCED ARCHITECTURES

### 2.1 Gated Graph Neural Network (GG-NN)

```
Paper: Li et al. (2016)
Idea: Recurrent message passing with GRU
```

**Message Passing:**
```
m_v = Σ W_edge × h_u
     u∈N(v)

h_v = GRU(h_v, m_v)
```

**Code Applications:**
- **Program reasoning**: Variable dependency analysis
- **Bug detection**: ICLR paper used GG-NN

---

### 2.2 Message Passing Neural Network (MPNN)

```
Paper: Gilmer et al. (2017)
Idea: General framework for GNNs
```

**Framework:**
```
m_v^(l+1) = Σ M_l(h_v^(l), h_u^(l), e_vu)
           u∈N(v)

h_v^(l+1) = U_l(h_v^(l), m_v^(l+1))
```

**Code Applications:**
- **Custom message functions**: Encode code semantics
- **Edge-aware**: Use edge types (call, import, type)

---

### 2.3 Relational GCN (R-GCN)

```
Paper: Schlichtkrull et al. (2018)
Idea: Different weights for different edge types
```

**Message Passing:**
```
h_v^(l+1) = σ( Σ W_r^(l) × h_u^(l) + W_0^(l) × h_v^(l) )
           r,u where r = edge type
```

**Code Applications:**
- **Multi-edge code graphs**: Calls, imports, types, tests
- **Semantic relationships**: Different semantics per edge

---

### 2.4 Graph Transformer / Graphormer

```
Paper: Ying et al. (2021), Shi et al. (2021)
Idea: Transformer attention on graphs
```

**Key Components:**
- Spatial encoding (shortest path distance)
- Edge encoding in attention
- Centrality encoding (node importance)

**Code Applications:**
- **Long-range dependencies**: Transformer handles better than GNN
- **Code understanding**: Full context attention

---

## 3. TASK-SPECIFIC ARCHITECTURES

### 3.1 Devign (Vulnerability Detection)

```
Paper: Zhou et al. (2019)
Idea: GCN + GRU for vulnerability detection
```

**Architecture:**
1. Build code property graph
2. GCN layers for local patterns
3. GRU for sequential dependencies
4. Conv1D for final classification

**Results:** 89% accuracy on synthetic vulnerabilities

---

### 3.2 ReGVD (Vulnerability Detection)

```
Paper: Nguyen et al. (2022)
Idea: Graph neural network with vulnerability-specific pooling
```

**Architecture:**
- GNN backbone
- Vulnerability-specific readout
- Multi-head attention pooling

---

### 3.3 LineVul (Line-level Vulnerability)

```
Paper: Fu et al. (2022)
Idea: Predict vulnerable lines, not just files
```

**Architecture:**
- CodeBERT for tokens
- GNN for graph structure
- Line-level attention

---

### 3.4 CodeBERT + GNN

```
Idea: Combine pretrained LM with graph structure
```

**Approach:**
1. CodeBERT for node features
2. GNN for graph propagation
3. Joint fine-tuning

**Code Applications:**
- Code search
- Similarity detection
- Documentation generation

---

## 4. HETEROGENEOUS GNN ARCHITECTURES

### 4.1 Heterogeneous Graph Transformer (HGT)

```
Paper: Hu et al. (2020)
Idea: Transformer for heterogeneous graphs
```

**Code Applications:**
- **Multi-entity graphs**: Functions, classes, modules
- **Type-aware attention**: Different attention per type

---

### 4.2 HetGNN

```
Paper: Zhang et al. (2019)
Idea: Heterogeneous graph neural network
```

**Code Applications:**
- Code with multiple entity types
- Cross-type relationships

---

## 5. TEMPORAL GNN ARCHITECTURES

### 5.1 Temporal Graph Network (TGN)

```
Paper: Rossi et al. (2020)
Idea: Handle dynamic/evolving graphs
```

**Components:**
- Message store (memory)
- Message aggregator
- Memory updater (GRU)
- Embedding module

**Code Applications:**
- **Code evolution**: Track changes over time
- **Bug introduction**: When was bug introduced?
- **Emerging patterns**: New code smell detection

---

### 5.2 DySAT

```
Paper: Sankar et al. (2020)
Idea: Dynamic self-attention on graphs
```

**Architecture:**
- Structural attention (spatial)
- Temporal attention (time)
- Combined embeddings

---

### 5.3 EvolveGCN

```
Paper: Pareja et al. (2020)
Idea: GCN with evolving weights
```

**Code Applications:**
- Code drift detection
- Evolving patterns

---

## 6. POOLING METHODS

### 6.1 Hierarchical Pooling

**DiffPool:**
```
Learn soft cluster assignments
Pool to coarsened graph
Repeat hierarchically
```

**SAGPool:**
```
Self-attention based pooling
Keep top-k important nodes
```

### 6.2 Global Pooling

**Set2Set:**
```
Attention-based pooling
Order-invariant
```

**Global Attention:**
```
Weighted sum with learned attention
```

---

## 7. COMPARATIVE ANALYSIS

| Architecture | Expressiveness | Scalability | Inductive | Code Best For |
|--------------|----------------|-------------|-----------|---------------|
| GCN | Low | High | ❌ | Simple embeddings |
| GAT | Medium | Medium | ❌ | Weighted dependencies |
| GraphSAGE | Medium | High | ✅ | New repos |
| GIN | High | Medium | ❌ | Graph classification |
| GG-NN | Medium | Low | ❌ | Sequential reasoning |
| R-GCN | Medium | Medium | ❌ | Multi-edge graphs |
| Graphormer | High | Low | ✅ | Long-range deps |
| TGN | Medium | Medium | ✅ | Code evolution |

---

## 8. IMPLEMENTATION LANDSCAPE

### Python Libraries
- **PyTorch Geometric**: Most popular, 30+ GNN layers
- **DGL**: Deep Graph Library, production-ready
- **GraphNets**: TensorFlow graph networks

### Rust Options
- ❌ No mature Rust GNN library
- ✅ burn-rs (rust ML) has graph potential
- ✅ Custom implementation from scratch

### Recommended Path for Parseltongue
1. **Don't build GNN in Rust initially**
2. **Use Python for experimentation** (PyTorch Geometric)
3. **Export models to ONNX** for inference
4. **Later**: Rust inference with tract or candle

---

## 9. CODE-SPECIFIC GNN DESIGN

### Node Features for Code
```rust
struct CodeNodeFeatures {
    // Syntactic
    node_type: NodeType,  // Function, Class, Module
    name_embedding: Vec<f64>,  // Name via FastText

    // Semantic
    doc_embedding: Vec<f64>,  // Doc comment via CodeBERT
    signature_features: Vec<f64>,

    // Structural
    line_count: f64,
    complexity: f64,
    fan_in: f64,
    fan_out: f64,
}
```

### Edge Features for Code
```rust
struct CodeEdgeFeatures {
    edge_type: EdgeType,  // Call, Import, Type, Test
    weight: f64,  // Call frequency
    distance: f64,  // Line distance
}
```

### GNN Config for Code
```rust
struct CodeGNNConfig {
    hidden_dim: 64,
    num_layers: 3,
    dropout: 0.1,
    aggregator: "mean",  // or "attention"
    pooling: "set2set",
}
```

---

## 10. IMPLEMENTATION PRIORITY FOR PARSERLTONGUE

### Phase 3: Research/Experimentation
1. GCN baseline (simplest)
2. GAT for attention visualization
3. GraphSAGE for inductive learning

### Phase 4: Production (if experiments succeed)
4. Pretrained model via ONNX
5. Rust inference pipeline

### Skip for Now
- Temporal GNNs (need code history)
- Graph Transformers (too expensive)
- Heterogeneous GNNs (complexity)

---

## References

1. Kipf & Welling (2017). "Semi-Supervised Classification with GCNs"
2. Veličković et al. (2018). "Graph Attention Networks"
3. Hamilton et al. (2017). "Inductive Representation Learning on Large Graphs"
4. Xu et al. (2019). "How Powerful are GNNs?"
5. Zhou et al. (2019). "Devign: Effective Vulnerability Identification"
