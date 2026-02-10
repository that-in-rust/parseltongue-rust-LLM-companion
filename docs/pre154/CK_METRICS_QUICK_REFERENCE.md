# CK Metrics Suite - Quick Reference Card

## Usage Example

```rust
use parseltongue_core::graph_analysis::{
    AdjacencyListGraphRepresentation,
    compute_ck_metrics_suite,
    grade_ck_metrics_health,
    HealthGrade,
};

// Build graph from edges
let edges = vec![
    ("ClassA".to_string(), "ClassB".to_string(), "Calls".to_string()),
    ("ClassA".to_string(), "ClassC".to_string(), "Calls".to_string()),
    // ... more edges
];
let graph = AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges);

// Compute all 4 CK metrics for a node
let metrics = compute_ck_metrics_suite(&graph, "ClassA");

// Grade the metrics (A-F)
let grade = grade_ck_metrics_health(&metrics);

println!("ClassA Metrics:");
println!("  CBO (Coupling): {}", metrics.cbo);
println!("  LCOM (Cohesion): {:.2}", metrics.lcom);
println!("  RFC (Response): {}", metrics.rfc);
println!("  WMC (Complexity): {}", metrics.wmc);
println!("  Grade: {:?}", grade);
```

## Individual Metric Functions

```rust
use parseltongue_core::graph_analysis::{
    calculate_coupling_between_objects,
    calculate_lack_cohesion_methods,
    calculate_response_for_class,
    calculate_weighted_methods_class,
};

let cbo = calculate_coupling_between_objects(&graph, "MyClass");
let lcom = calculate_lack_cohesion_methods(&graph, "MyClass");
let rfc = calculate_response_for_class(&graph, "MyClass");
let wmc = calculate_weighted_methods_class(&graph, "MyClass");
```

## Metrics Explained

### 1. CBO (Coupling Between Objects)
**What it measures**: Number of unique entities this class is coupled with
**How it's computed**: `|forward neighbors| + |reverse neighbors|` (deduplicated)
**Threshold**: FAIL if > 10
**Interpretation**:
- **Low (0-5)**: Good - minimal coupling
- **Medium (6-10)**: OK - moderate coupling
- **High (>10)**: FAIL - excessive coupling, refactor needed

**Example**:
```
Class A calls B, C
Class D, E, F call A
→ CBO(A) = |{B,C}| + |{D,E,F}| = 5
```

### 2. LCOM (Lack of Cohesion of Methods)
**What it measures**: How well the "methods" of a class work together
**How it's computed**: Compare pairs of child nodes by shared targets
- P = pairs sharing NO common targets
- Q = pairs sharing ≥1 target
- LCOM = P / (P + Q)

**Threshold**: FAIL if > 0.8
**Interpretation**:
- **0.0**: Perfect cohesion - all methods share targets
- **0.0-0.5**: Good cohesion
- **0.5-0.8**: Moderate cohesion
- **>0.8**: FAIL - low cohesion, consider splitting class

**Example**:
```
Class A calls methods B, C
B calls D, E
C calls D, F
→ B and C share target D
→ 1 pair with shared target (Q=1), 0 without (P=0)
→ LCOM = 0 / 1 = 0.0 (good cohesion)
```

### 3. RFC (Response For a Class)
**What it measures**: Total number of methods that can be executed in response
**How it's computed**: Direct calls + their transitive calls (1 hop, deduplicated)
**Threshold**: WARNING if > 50
**Interpretation**:
- **Low (1-20)**: Simple response set
- **Medium (21-50)**: OK - moderate complexity
- **High (>50)**: WARNING - complex response set, hard to test

**Example**:
```
Class A calls B, C
B calls D, E
C calls F
→ RFC(A) = |{B, C, D, E, F}| = 5 unique methods
```

### 4. WMC (Weighted Methods per Class)
**What it measures**: Sum of method complexities
**How it's computed**: Out-degree (proxy for cyclomatic complexity)
**Threshold**: WARNING if > 50
**Interpretation**:
- **Low (1-10)**: Simple class
- **Medium (11-50)**: OK - moderate complexity
- **High (>50)**: WARNING - very complex, consider refactoring

**Example**:
```
Class A calls methods B, C, D, E
→ WMC(A) = out-degree = 4
```

## Grading System

| Grade | Meaning | Criteria |
|-------|---------|----------|
| **A** | Excellent | All metrics OK |
| **B** | Good | 1 WARNING |
| **C** | Fair | 2 WARNING |
| **D** | Poor | 1 FAIL or 3+ WARNING |
| **F** | Critical | 2+ FAIL |

## Threshold Summary

| Metric | Threshold | Type | Meaning |
|--------|-----------|------|---------|
| CBO | > 10 | FAIL | Too many coupling dependencies |
| LCOM | > 0.8 | FAIL | Class lacks cohesion |
| RFC | > 50 | WARNING | Response set too large |
| WMC | > 50 | WARNING | Class too complex |

## Common Patterns

### Pattern 1: Highly Coupled Class
```rust
// CBO = 15 (calls 8, called by 7)
// Grade: D (1 FAIL)
// Action: Reduce dependencies, use facades
```

### Pattern 2: Low Cohesion Class
```rust
// LCOM = 0.9 (methods don't share targets)
// Grade: F (1 FAIL)
// Action: Split into multiple focused classes
```

### Pattern 3: Complex Response
```rust
// RFC = 65 (calls many methods transitively)
// Grade: B (1 WARNING)
// Action: Consider simplifying call chains
```

### Pattern 4: God Class
```rust
// CBO = 12, LCOM = 0.85, RFC = 55, WMC = 60
// Grade: F (2 FAIL, 2 WARNING)
// Action: Major refactoring needed - split into smaller classes
```

## Best Practices

1. **Measure Early**: Run CK metrics on new classes during development
2. **Set Team Standards**: Define acceptable thresholds for your codebase
3. **Track Trends**: Monitor metric changes over time
4. **Refactor Grade D/F**: Classes with poor grades are technical debt
5. **Combine with Tests**: Low LCOM often means hard-to-test classes

## References

- Chidamber, S.R. & Kemerer, C.F. (1994). "A Metrics Suite for Object-Oriented Design"
- Parseltongue v1.6.0 implementation uses graph-based approximations for language-agnostic analysis
