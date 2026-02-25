# v1.6.5 Waves 2 & 3 Implementation Completion Plan

## Status: 6/16 Handlers Complete ✅

### Completed (6)
1. ✅ code_entity_detail_view_handler.rs
2. ✅ dependency_edges_list_handler.rs
3. ✅ complexity_hotspots_ranking_handler.rs
4. ✅ reverse_callers_query_graph_handler.rs
5. ✅ forward_callees_query_graph_handler.rs
6. ✅ blast_radius_impact_handler.rs

### Remaining Medium Handlers (2)

#### Handler #7: semantic_cluster_grouping_handler.rs
```rust
// Add import
use crate::scope_filter_utilities_module::parse_scope_build_filter_clause;

// Update query params
pub struct SemanticClusterQueryParams {
    // existing fields...
    pub scope: Option<String>,  // ADD THIS
}

// Update handler call
let clusters = compute_semantic_clusters(&state, &params.scope).await;

// Update function signature and implementation
async fn compute_semantic_clusters(
    state: &SharedApplicationStateContainer,
    scope_filter: &Option<String>,
) -> Vec<ClusterData> {
    // ... storage setup ...

    let scope_clause = parse_scope_build_filter_clause(scope_filter);

    // Update entity query
    let entity_query = if scope_clause.is_empty() {
        "?[key, file_path] := *CodeGraph{ISGL1_key: key, file_path}".to_string()
    } else {
        format!(
            "?[key, file_path] := *CodeGraph{{ISGL1_key: key, file_path, root_subfolder_L1, root_subfolder_L2}}{}",
            scope_clause
        )
    };

    // ... rest of implementation ...
}
```

#### Handler #8: smart_context_token_budget_handler.rs
```rust
// Add import
use crate::scope_filter_utilities_module::parse_scope_build_filter_clause;

// Update query params
pub struct SmartContextQueryParams {
    pub focus: String,
    pub tokens: Option<usize>,
    pub scope: Option<String>,  // ADD THIS
}

// Update handler call
let context = build_smart_context(&state, &params.focus, tokens, &params.scope).await;

// Update function signature
async fn build_smart_context(
    state: &SharedApplicationStateContainer,
    focus_entity: &str,
    token_budget: usize,
    scope_filter: &Option<String>,
) -> ContextData {
    // ... storage setup ...

    let scope_clause = parse_scope_build_filter_clause(scope_filter);
    let scope_join = if scope_clause.is_empty() {
        String::new()
    } else {
        format!(", *CodeGraph{{ISGL1_key: from_key, root_subfolder_L1, root_subfolder_L2}}{}", scope_clause)
    };

    // Update all edge queries to include scope_join
    // ... rest of implementation ...
}
```

### Remaining Complex Handlers (8) - Shared Graph Builder Pattern

All 8 handlers below use `build_graph_from_database_edges()`. The pattern is:

1. Add import and scope param to query struct
2. Pass scope to graph builder
3. Modify the `build_graph_from_database_edges()` function in EACH handler

#### Handlers 9-16:
9. circular_dependency_detection_handler.rs
10. strongly_connected_components_handler.rs
11. technical_debt_sqale_handler.rs
12. kcore_decomposition_layering_handler.rs
13. centrality_measures_entity_handler.rs
14. entropy_complexity_measurement_handler.rs
15. coupling_cohesion_metrics_handler.rs
16. leiden_community_detection_handler.rs

#### Universal Pattern for 9-16:

```rust
// STEP 1: Add import at top
use crate::scope_filter_utilities_module::parse_scope_build_filter_clause;

// STEP 2: Add to query params struct
pub struct XxxQueryParams {
    // ... existing fields ...
    pub scope: Option<String>,  // ADD THIS
}

// STEP 3: In main handler function, pass scope to graph builder
let graph = build_graph_from_database_edges(&storage, &params.scope).await?;

// STEP 4: Update build_graph_from_database_edges signature
async fn build_graph_from_database_edges(
    storage: &Arc<parseltongue_core::storage::CozoDbStorage>,
    scope_filter: &Option<String>,  // ADD THIS PARAM
) -> Result<AdjacencyListGraphRepresentation, String> {
    // Build scope clause
    let scope_clause = parse_scope_build_filter_clause(scope_filter);

    // Build query with conditional scope filtering
    let query = if scope_clause.is_empty() {
        "?[from_key, to_key, edge_type] := *DependencyEdges{from_key, to_key, edge_type}".to_string()
    } else {
        format!(
            "?[from_key, to_key, edge_type] := *DependencyEdges{{from_key, to_key, edge_type}}, *CodeGraph{{ISGL1_key: from_key, root_subfolder_L1, root_subfolder_L2}}{}",
            scope_clause
        )
    };

    let result = storage
        .raw_query(&query)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

    // ... rest of function unchanged ...
}
```

## Wave 3: Polish Tasks

### Task #78: Add ?section= to ingestion_diagnostics_coverage_handler.rs

```rust
// Update query params
pub struct DiagnosticsQueryParams {
    pub section: Option<String>,  // Values: test_entities, word_coverage, ignored_files, summary
}

// In handler function
match params.section.as_deref() {
    None => {
        // Return all sections (current behavior)
        return full_response;
    }
    Some("test_entities") => {
        // Return only test_entities section
    }
    Some("word_coverage") => {
        // Return only word_coverage section
    }
    Some("ignored_files") => {
        // Return only ignored_files section
    }
    Some("summary") => {
        // Return only aggregates (counts and averages, no file arrays)
    }
    Some(invalid) => {
        // Return error for invalid section name
    }
}
```

### Task #79: Update API docs + PRD errata

#### api_reference_documentation_handler.rs updates:
```rust
// Add to endpoint documentation:
// 1. /ingestion-diagnostics-coverage-report?section=summary|test_entities|word_coverage|ignored_files
// 2. /folder-structure-discovery-tree
// 3. Add ?scope= parameter to all query endpoints (16 total)
```

#### docs/PRD-v165.md errata section:
```markdown
## Corrections and Errata

### Function Names
- **PRD**: `build_scope_filter_clause`
- **Actual**: `parse_scope_build_filter_clause`

### Module Location
- **PRD**: scope utilities inside `http_endpoint_handler_modules/`
- **Actual**: `scope_filter_utilities_module.rs` at `pt08/src/` level

### CozoDB Datalog Syntax
- **PRD**: Scope constraints can be inside atom braces
- **Actual**: Constraints with `= 'value'` MUST be OUTSIDE atom braces, after binding variables

### CodeGraph Schema
- **PRD**: 17 columns
- **Actual**: 19 columns (already includes root_subfolder_L1 and root_subfolder_L2)
```

## Implementation Checklist

- [x] Handlers 1-6 complete
- [ ] Handler 7: semantic_cluster_grouping_handler.rs
- [ ] Handler 8: smart_context_token_budget_handler.rs
- [ ] Handlers 9-16: Update all complex handlers (8 total)
- [ ] Task #78: Add ?section= to diagnostics
- [ ] Task #79: Update API docs + PRD errata
- [ ] cargo build --release
- [ ] cargo test --all
- [ ] Manual testing of scope parameter
- [ ] Update docs/v165-executive-implementation-specs.md

## Testing Commands

```bash
# Test without scope (backward compatible)
curl "http://localhost:7777/code-entities-list-all" | jq '.data.total_count'

# Test with L1 scope
curl "http://localhost:7777/code-entities-list-all?scope=crates" | jq '.data.total_count'

# Test with L1+L2 scope
curl "http://localhost:7777/code-entities-list-all?scope=crates||parseltongue-core" | jq '.data.total_count'

# Test diagnostics sections
curl "http://localhost:7777/ingestion-diagnostics-coverage-report?section=summary" | jq '.'

# Verify all 18 endpoints support scope
for endpoint in code-entities-list-all code-entity-detail-view dependency-edges-list-all complexity-hotspots-ranking-view reverse-callers-query-graph forward-callees-query-graph blast-radius-impact-analysis semantic-cluster-grouping-list smart-context-token-budget circular-dependency-detection-scan strongly-connected-components-list technical-debt-sqale-analysis kcore-decomposition-layering-view centrality-measures-entity-ranking entropy-complexity-measurement-report coupling-cohesion-metrics-analysis leiden-community-detection-clusters; do
  echo "Testing $endpoint with scope..."
  curl -s "http://localhost:7777/$endpoint?scope=crates" | jq '.success' || echo "FAILED: $endpoint"
done
```
