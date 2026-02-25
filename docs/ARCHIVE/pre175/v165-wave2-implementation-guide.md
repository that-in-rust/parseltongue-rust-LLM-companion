# v1.6.5 Wave 2 Implementation Guide

## Completed Handlers (5/16)

### Simple Handlers (3/3) ✅
1. **code_entity_detail_view_handler.rs** - DONE
2. **dependency_edges_list_handler.rs** - DONE
3. **complexity_hotspots_ranking_handler.rs** - DONE

### Medium Handlers (2/5) ✅
4. **reverse_callers_query_graph_handler.rs** - DONE
5. **forward_callees_query_graph_handler.rs** - DONE

## Remaining Handlers (11/16)

### Medium Handlers (3/5) - Direct CodeGraph queries
6. **blast_radius_impact_handler.rs**
7. **semantic_cluster_grouping_handler.rs**
8. **smart_context_token_budget_handler.rs**

Pattern for handlers 6-8:
```rust
// 1. Add import
use crate::scope_filter_utilities_module::parse_scope_build_filter_clause;

// 2. Add to query params struct
pub scope: Option<String>,

// 3. Pass to query function
let result = query_function(&state, params, &params.scope).await;

// 4. In query function signature
async fn query_function(
    state: &SharedApplicationStateContainer,
    params: ...,
    scope_filter: &Option<String>,
) -> ... {
    // 5. Build scope clause
    let scope_clause = parse_scope_build_filter_clause(scope_filter);

    // 6. Modify CodeGraph query
    let query = format!(
        "?[...] := *CodeGraph{{..., root_subfolder_L1, root_subfolder_L2}}{}",
        scope_clause
    );
}
```

### Complex Handlers (8/8) - use build_graph_from_database_edges()
9. **circular_dependency_detection_handler.rs**
10. **strongly_connected_components_handler.rs**
11. **technical_debt_sqale_handler.rs**
12. **kcore_decomposition_layering_handler.rs**
13. **centrality_measures_entity_handler.rs**
14. **entropy_complexity_measurement_handler.rs**
15. **coupling_cohesion_metrics_handler.rs**
16. **leiden_community_detection_handler.rs**

Pattern for handlers 9-16:
```rust
// 1. Add import (same as above)
use crate::scope_filter_utilities_module::parse_scope_build_filter_clause;

// 2. Add to query params struct (same as above)
pub scope: Option<String>,

// 3. Pass to graph builder
let graph = build_graph_from_database_edges(&storage, &params.scope).await?;

// 4. Modify build_graph_from_database_edges() signature
async fn build_graph_from_database_edges(
    storage: &Arc<parseltongue_core::storage::CozoDbStorage>,
    scope_filter: &Option<String>,
) -> Result<AdjacencyListGraphRepresentation, String> {
    // 5. Build scope clause
    let scope_clause = parse_scope_build_filter_clause(scope_filter);

    // 6. Modify edge query with scope join
    let query = if scope_clause.is_empty() {
        "?[from_key, to_key, edge_type] := *DependencyEdges{from_key, to_key, edge_type}".to_string()
    } else {
        format!(
            "?[from_key, to_key, edge_type] := *DependencyEdges{{from_key, to_key, edge_type}}, *CodeGraph{{ISGL1_key: from_key, root_subfolder_L1, root_subfolder_L2}}{}",
            scope_clause
        )
    };
}
```

## Critical Notes

1. **Scope filter format**: `, root_subfolder_L1 = 'value'` or `, root_subfolder_L1 = 'val1', root_subfolder_L2 = 'val2'`
2. **Placement**: AFTER binding variables in CodeGraph atom, OUTSIDE the braces
3. **Backward compatibility**: Empty/absent scope returns all results
4. **Edge filtering**: Apply scope to `from_key` entity (the source of the dependency)
5. **Each handler** has its own `build_graph_from_database_edges()` - need to modify all 8

## Testing Checklist

After each handler:
- [ ] Cargo build --release
- [ ] Test without ?scope= (should return all)
- [ ] Test with ?scope=crates (should filter)
- [ ] Test with ?scope=crates||parseltongue-core (should filter to L2)
