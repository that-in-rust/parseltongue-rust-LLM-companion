# UX Blueprint → Compiler Data Mapping

**Document Purpose:** Map every moment in the user experience to exact data points extractable from the Rust compiler.

**Target User:** OSS Rust contributors working on large repos

---

## Part 1: The 7-Moment User Journey

```
┌─────────────────────────────────────────────────────────────────────────┐
│  MOMENT 0: Intent Classification                                        │
│  User: "How does authentication work?"                                  │
│  System: Classify intent → explain-architecture (84%)                  │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────┐
│  MOMENT 1: Intent Confirmation                                          │
│  System: "I interpreted your intent as: Explain architecture. Change?"  │
│  User: Accepts or edits                                                 │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────┐
│  MOMENT 2: Option Cards Presented                                       │
│  System: 3-5 cards, each with breadcrumb, anchor, neighbors, confidence │
│  User: Scans cards                                                      │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────┐
│  MOMENT 3: Card Details Viewed                                          │
│  System: Each card shows: path, entity, top neighbors, risk, tokens     │
│  User: Clicks one card                                                   │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────┐
│  MOMENT 4: Context Preview                                              │
│  System: "Preview context (900 tokens): signature map + call slice"    │
│  User: Clicks "Deep dive" or "Compare"                                  │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────┐
│  MOMENT 5: Deep Dive                                                    │
│  System: Execution path, data movement, related tests, invariants       │
│  User: Asks follow-up                                                   │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────┐
│  MOMENT 6: Action Bar                                                   │
│  System: "Generate patch plan / Run blast radius / Review checklist"   │
│  User: Picks next action                                                │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────┐
│  MOMENT 7: Final Answer                                                 │
│  System: Exact files/lines + confidence + unknowns                      │
│  User: Proceeds to edit/review                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Part 2: Compiler Data Sources by Layer

### Layer 1: ra_ap_* Crates (Stable-ish, 5-10% break rate)

| Crate | What It Gives | UX Moment Used |
|-------|---------------|----------------|
| `ra_ap_hir` | Module definitions, function signatures, struct fields | 2, 3, 4, 5 |
| `ra_ap_hir_def` | Item trees, module hierarchy | 2, 3 |
| `ra_ap_hir_ty` | Type inference, trait resolution | 4, 5, 6 |
| `ra_ap_base` | File IDs, source database | All |
| `ra_ap_syntax` | AST nodes, tokens | 0, 1 |

**Exact Types:**
```rust
// From ra_ap_hir
ModuleDefId    // Canonical entity identity
Function       // Function with signature
Struct         // Struct with fields
Trait          // Trait with associated items
Impl           // Trait implementation
Type           // Inferred type

// From ra_ap_hir_def
ModuleId       // Module identity
ItemId         // Generic item identity
DefWithBodyId // Item that has a body (fn, const, static)

// From ra_ap_hir_ty
Ty             // Type representation
InferenceResult // Type inference results
TraitEnvironment // Trait bounds in scope
```

### Layer 2: rustc_middle (Nightly, MIR access)

| Crate | What It Gives | UX Moment Used |
|-------|---------------|----------------|
| `rustc_middle::mir` | Control flow graph, data flow | 5, 6 |
| `rustc_middle::ty` | Type system, trait solver | 5, 6 |
| `rustc_middle::hir` | High-level IR | 2, 3, 4 |
| `rustc_span` | Source locations | All |

**Exact Types:**
```rust
// From rustc_middle::mir
Body<'tcx>          // MIR body (CFG + locals)
BasicBlock          // CFG node
TerminatorKind      // Branch/return/call
Place<'tcx>         // Memory location
Operand<'tcx>       // Value
Rvalue<'tcx>        // Expression

// From rustc_middle::ty
TyCtxt<'tcx>        // Global context (query everything)
Ty<'tcx>            // Type representation
Instance<'tcx>      // Monomorphized function
DefId               // Unique definition ID

// From rustc_span
Span                // Source location (file:line:col)
```

### Layer 3: Polonius (Borrow checker facts)

| Data | What It Gives | UX Moment Used |
|------|---------------|----------------|
| Borrowck facts | Originates at, flows into | 5 (ownership) |
| Loan paths | Reference lifetimes | 5 |
| Move data | When values move | 5 |

---

## Part 3: Moment-by-Moment Data Requirements

### MOMENT 0: Intent Classification

**Data Needed:**
- Keywords in query
- Repo structure (top-level crates)
- Recent git context (current branch, recent files)

**Source:** Git + `ra_ap_syntax` for tokenization

**Implementation:**
```rust
// Tokenize query
let tokens = query.split_whitespace().collect::<Vec<_>>();

// Get crate names from Cargo.toml
let crates = parse_cargo_toml(workspace_root);

// Classify intent
let intent = match tokens {
    ["how", "does", x, "work"] => Intent::ExplainArchitecture(x),
    ["what", "calls", x] => Intent::FindCallers(x),
    ["where", "is", x, "defined"] => Intent::FindDefinition(x),
    ["what", "implements", x] => Intent::FindImpls(x),
    _ => Intent::Unknown,
};
```

**No compiler data needed at this stage.**

---

### MOMENT 1: Intent Confirmation

**Data Needed:**
- Confidence score from classification
- Alternative interpretations

**Source:** NLP classifier (not compiler)

**No compiler data needed.**

---

### MOMENT 2: Option Cards Presented

**Data Needed:**
- Candidate entities matching query
- Breadcrumb paths (module hierarchy)
- Confidence scores
- Token estimates

**Source:** `ra_ap_hir` + `ra_ap_hir_def`

**Implementation:**
```rust
// 1. Find matching entities
fn find_candidates(db: &RootDatabase, query: &str) -> Vec<ModuleDefId> {
    let mut candidates = vec![];
    
    // Iterate all modules
    for module in all_modules(db) {
        for def in module.defs(db) {
            let name = def.name(db).to_string();
            if fuzzy_match(&name, query) {
                candidates.push(def);
            }
        }
    }
    
    candidates
}

// 2. Build breadcrumb path
fn breadcrumb(db: &RootDatabase, def: ModuleDefId) -> Vec<String> {
    let module = def.module(db);
    let mut path = vec![def.name(db).to_string()];
    
    let mut current = module;
    while let Some(parent) = current.parent(db) {
        path.insert(0, current.name(db)?.to_string());
        current = parent;
    }
    
    path
}

// 3. Estimate tokens
fn estimate_tokens(db: &RootDatabase, def: ModuleDefId) -> usize {
    match def {
        ModuleDefId::FunctionId(f) => {
            let sig = f.signature(db);
            // ~50 tokens for signature + docs preview
            50 + sig.params.len() * 10
        }
        ModuleDefId::StructId(s) => {
            let fields = s.fields(db).len();
            30 + fields * 5
        }
        _ => 50,
    }
}
```

**Option Card Structure:**
```rust
struct OptionCard {
    title: String,           // "auth::service::authenticate_user"
    breadcrumb: Vec<String>, // ["crates", "auth", "src", "service.rs"]
    confidence: f32,         // 0.85
    token_estimate: usize,   // 700
    freshness: Freshness,    // UpToDate | Stale
    why_matched: Vec<MatchReason>,
}

enum MatchReason {
    LexicalMatch,
    SemanticSimilarity(f32),
    GraphProximity,
    RecentGitContext,
}
```

---

### MOMENT 3: Card Details Viewed

**Data Needed:**
- Top 3-4 connected entities (callers/callees/impls)
- Risk badge (change impact)
- Module path

**Source:** `ra_ap_hir` + `ra_ap_hir_ty`

**Implementation:**
```rust
// Get related entities
fn get_neighbors(db: &RootDatabase, def: ModuleDefId) -> Vec<NeighborInfo> {
    let mut neighbors = vec![];
    
    match def {
        ModuleDefId::FunctionId(f) => {
            // Find callers (up the call graph)
            let callers = find_callers(db, f);
            neighbors.extend(callers.map(NeighborInfo::Caller));
            
            // Find callees (down the call graph)
            let body = f.body(db);
            for expr in body.exprs() {
                if let Expr::Call { callee, .. } = expr {
                    neighbors.push(NeighborInfo::Callee(*callee));
                }
            }
        }
        ModuleDefId::TraitId(t) => {
            // Find implementations
            let impls = t.impls_in_crate(db);
            neighbors.extend(impls.map(NeighborInfo::Impl));
        }
        _ => {}
    }
    
    neighbors.into_iter().take(4).collect()
}

// Calculate risk badge
fn calculate_risk(db: &RootDatabase, def: ModuleDefId) -> RiskLevel {
    let fan_in = count_callers(db, def);
    let fan_out = count_callees(db, def);
    
    match (fan_in, fan_out) {
        (0..=3, 0..=5) => RiskLevel::Low,
        (4..=10, 6..=15) => RiskLevel::Medium,
        _ => RiskLevel::High,
    }
}
```

**Card Detail Structure:**
```rust
struct CardDetail {
    anchor: ModuleDefId,
    neighbors: Vec<NeighborInfo>,
    risk: RiskLevel,
    signature: String,
    doc_preview: Option<String>,
    tests: Vec<Function>,  // Related test functions
}

struct NeighborInfo {
    entity: ModuleDefId,
    relationship: Relationship,
    distance: usize,  // Graph distance
}

enum Relationship {
    Caller,      // Calls this entity
    Callee,      // Called by this entity
    Impl,        // Implements this trait
    TypeUser,    // Uses this type
    SameModule,  // In same module
}
```

# Part 4: The 7-Moment UX Blueprint

## Part 4: Option Card Requirements

Each Option card must to communicate:

### Why this card exists
| Reason | Source |
|------|--------|
| Lexical match | Text similarity in codebase |
| Semantic match | Type/function name embedding |
| Graph proximity | Call graph neighbors |
| Git context | Recently modified files |

### Data per card

```json
{
  "title": "auth::service::authenticate_user",
  "breadcrumb": "crates/auth/src/service.rs",
  "entity_type": "public fn",
  "why_matched": ["lexical", "semantic", "graph"],
  "top_neighbors": [
    {"name": "login", "relationship": "callee"},
    {"name": "validate_token", "relationship": "callee"},
    {"name": "User", "relationship": "type_user"}
  ],
  "risk": "medium",
  "confidence": 0.89,
  "token_estimate": 1200,
  "freshness": "up-to-date"
}
```

---

## Part 5: Data Flow Analysis (What We CAN Extract)

### Level 1: Type Flow (ra_ap_hir)

**Source:** `raApHir::Type`
```rust
pub fn type_flow_example(db: &RootDatabase, fn: FunctionId) -> Vec<(String, Ty)> {
    let infer = db.infer(fn);
    let sig = fn.signature(fn);
    
    // Get return type
    let ret_ty = infer[fn].return_type();
    println!("Function {} returns {:?}", ret_ty);
    
    // get parameters
    for param in &fn.params {
        let param_ty = infer[param].ty;
        println!("  Parameter {}: Type: {:?}", param_ty);
    }
}
```

**This gives us:**
- "This function takes a User and returns a Result<Token, AuthError>"
- "Parameter types tell us what flows through"
- "Return type indicates what can go wrong"

### Level 2: Def-Use Chains (rustc MIR)

**Source:** `rustc_middle::mir::Body`
```rust
pub fn def_use_chains(body: &Body<'tcx>) -> Vec<(Local, Vec<Local>)> {
    // For each local, find all places it's used
    for place in body.all_places(tcx, def_id) {
        // Build name map from debug info
        let name_map = body.debug_info_name_map();
        for (local, name, places) {
            println!("Local {}: is at {:?}", name);
        }
    }
}
```

**This tells us:**
- "This local is written to at line 47"
- "This local is read from at line 52"
- "This local is used in the conditional at line 89"

### Level 3: Taint/Predicate Flow (Flowistry territory)

**Source:** `flowistry::infoflow::compute_flow`
```rust
// See flowistry/crates/flowistry/src/infoflow.rs for full implementation
```

**Note:** Level 3 requires specialized analysis. Defer unless you need taint tracking.

---

## Part 6: Control Flow Analysis


# Part 7: Deep Dive Content

## Part 7: Deep Dive Content
## Part 7: Deep Dive content
## Part 8: Deep Dive Content

### What to Include in Deep Dive

| Category | Data Sources | UX Moment |
|----------|--------------|---------|
| **Signature** | ra_ap_hir | Signature + doc comment | 3, 4 |
| **Type map** | ra_ap_hir_ty | Type flow analysis | 5 |
| **Call slice** | MIR call graph (1-2 hops) | 5, 6 |
| **Related types** | ra_ap_hir_ty | Related types | 6 |
| **Trait impls** | ra_ap_hir_ty | Trait implementations | 7, 8 |
| **Tests** | MIR-based detection + integration tests | 9, 10 |
| **Invariants** | Flowistry + manual analysis | 9 |
| **Error paths** | MIR terminators + control dependencies | 10 |
| **Doc strings** | ra_ap_hir | Doc comments | 11 |

| **Example code** | Specific spans from source files | 12 |

### Implementation Priority

| Priority | Data | Value | Effort |
|----------|------|-------|-----|
| 1 | Signatures | ra_ap_hir | Low | SHIP NOW |
| 2 | Type flow | ra_ap_hir_ty | Medium | Implement next |
| 3 | Call graph | MIR call graph + rustc_utils | Medium | Good foundation |
| 4 | Related types | ra_ap_hir_ty | Low | Good foundation |
| 5 | Trait impls | ra_ap_hir_ty | Low | Good foundation |
| 6 | Tests | MIR + rustc_utils | Medium | Requires MIR access |
| 7 | Error paths | MIR + rustc_utils | Medium | Good foundation |
| 8 | Doc strings | ra_ap_hir | Low | Nice to have |
| 9 | Invariants | Flowistry/Analysis + manual | | | Low priority |
| 10 | Control dependencies | MIR + rustc_utils | Medium | Low priority |
| 11 | Data flow | Flowistry | rustc_utils | Low | Destructure |

            let neighbors.push(neighbor);
        }
    }
}
```
part3

echo "Part 3 written"

# Part 9: Graph Algorithms → UX Enhancement
## Part 9: Graph Algorithms → UX Enhancement

### How Graph Algorithms Amplify the UX

| Algorithm | Compiler Data Source | UX Enhancement |
|-----------|-----------------|------------------|
| Circular Dependencies | `use` edge, module dependencies | "This function is create a cycle." |
| Complexity Hotspots | Call graph degrees + type complexity | "Which code is complex?" |
| SCC | Call graph cycles | MCFG errors | "Show cycles" | Kick complexity" |
| Semantic Clusters | Call density + trait impls | "Find related functionality" |
| Leiden | Call graph community structure | "Natural groupings" |
| **k-Core** | `rustc_utils::control_dependencies` + dominators + "Is this function important?" |
| Centrality | Call graph degrees + trait impl rank | "Which functions are core?" |
| Technical debt | MIR complexity + type complexity | "how much debt does this have?" |
| Coupling/Cohesion | Type usage patterns | "How tightly coupled is this?" |
| Blast Radius | Call graph + trait impls | "What breaks if I change this?" |

---

## Part 10: Implementation Roadmap

### Phase 1: Core (Week 1-2)

| Priority | Task | Effort | Status |
|----------|---------------------------------|----------|--------|
| 1 | Entity extraction | ra_ap_hir | LOW | ✅ |
| 2 | Graph construction | ra_ap_hir + ra_ap_hir_ty | Medium | 🔶 |
| 3 | Intent classification | ra_ap_syntax | Low | ✅ |
| 4 | Risk scoring | ra_ap_hir + rustc_utils | Medium | 🔶 |
| 5 | Deep dive | ra_ap_hir + charon/Miri | Medium | 🔶 |
| 6 | Blast radius | Call graph + trait impls | Medium | 🔶 |

**Phase 2: Enhanced Capabilities** (MEDIUM effort)
| 6 | Graph community detection (Low effort) |
| 7 | Proof-carrying context (Medium effort) |
| 8 | Counterfactual mode (low effort) |
| 9 | Verification hooks (medium effort) |
| 10 | Performance analysis (defer) |

---

## Appendix A: Key rustc Types Quick Reference

### From rustc_middle

```rust
// TyCtxt - global type context
pub struct TyCtxt<'tcx> {
    // Query to def_id by type
    pub fn type_of(def_id: DefId) -> Option<Ty<'tcx> {
        Some(tcx.types
            .map(|def_id| ty) {
                match ty.kind {
                    TyKind::Adt(..) if let TyKind::Adt(def_id) = |adt| def| Some(tcx.adt_def(def_id)),
                        ...
                    }
                }
            }
        }
    }
}

    /// Get the calls this function
    pub fn callees(def_id: impl Iterator<DefId> {
        for callee in tcx.call_visibility {
            if callee.is_empty {
                continue;
            }
        }
        let locals: Vec<Local> = body.local_decls.iter().for (decl in &decl) {
            let name = body.var_debug_info.iter()
                .filter_map(|info| value {
                    if value.place == Place::RETURN_PLACE {
                        let name_map = body.debug_info_name_map()
        }

        // Return map of name -> Place
        let name_map = body.debug_info_name_map();
    }

    // All places a function touches
    fn all_places(&self, tcx: def_id) -> HashSet<Place<'tcx>> {
        self.all_places(self, tcx, def_id).into_iter()
    }

}
```

**Example usage:**
```rust
// For complexity hotspot
let hotspots: Vec<HotspotInfo> = Vec::new();

// Sort by fan-in
hotspots.sort_by(|h| fan_in| fan_out, Reverse);

// Take top N
for let top_n as 5..=10 {
    let hotspots = hotspots.into_iter().take(5);
}

// Report
println!("Top 5 hotspots:");
for h in hotspots {
    println!("  {}: {} @ {:?}", h.entity);
    println!("  fan_in: {}", h.fan_in);
    println!("  fan_out: {}", h.fan_out);
    println!("  functions:");
    for f in hotspots.functions {
        println!("    - {}", f.name);
    }
}

// Complexity score
let complexity: hotspots.len()
    .sum(|complexity| hotspots.iter().sum(|c| {
        complexity += 1;
    }
}

println!("\nComplexity ranking complete.");
```

**Example output:**
```
Top 3 Complexity Hotspots:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  #1: process_request() - 47 callers, High fan-out (23)
  #2: validate() - 12 callers, Medium complexity
  #3: handle_oauth() - 8 callers, Low complexity
  #4: database_query() - 15 callers, High complexity
  #5: router() - 31 callers, High complexity

```

---

## Part 11: Segment-Specific Workflows

### Segment: New OSS Contributor (First contribution to a repo)

**Key insight:** They need less context, not a power user. They make the workflow.

| Phase | Data Shown | Success Moment |
|-------|--------------------|----------------------|
| 0. Entry | Git context | Entity extraction | <60s | Opens first PR |
| 1. Search | Query expansion | Entity extraction | <2m | Gets "read order" |
| 2. Deep Dive | ra_ap_hir | Struct signatures | Context on codebase |
    | Type relationships |
    | Graph algorithms | Hotspots, centrality, communities |
| 3. Disambiguation | Intent classification + entity anchors + <30s | First PR |

| Phase | Data Shown | Success moment |
|-------|--------------------|----------------------|
| 0. Entry | Repo overview | `crates`, `cargo read` | < 60s | Finds where to start |
| 1. Target Selection | Semantic search + disambiguation cards | < 2m | Narrowed to 1 function |
    | 2. Patch Plan | Uses blast radius analysis from Step 3 | | | | < 60s |

---

## Part 12: Technical Implementation Notes

### Performance Consider

| Component | Priority | Notes |
|-----------|----------|-------|
| Entity extraction | HMR only | Low | Easy, ship now |
| Graph construction | In-memory | Medium | Incremental, parallel edges |
| Intent classification | NLP + git | Fast, no compiler |
| Context preview | HIR signatures | Partial code | 120s | Small |
| Deep Dive | MIR body + call graph | Full code + invariants | MIR + rustc_utils |
| Blast radius | Call graph + trait impls | Medium |
| Verification | FUTURE | stable MIR |

| Performance | Parallel edges not needed (token budget) | Incremental not needed | Accuracy-critical |
| Freshness | Git + hash verification | Required |

---

## Appendix B: Future Enhancements

### Enhanced Graph Algorithms (Future)

| Algorithm | New Capability | Use Case |
|-----------|----------------|----------|
| **k-truss** | Bridge detection between modules | Architecture visualization |
| **Leiden with temporal communities** | Temporal code evolution analysis |
| **Walktrap** | Security vulnerability detection | Continuous monitoring |
| **E-Graph** | Incremental graph updates for entity changes |

---

### Integration with Existing Tools

| Tool | What to Use |
|------|------------------------|
| Miri | UB detection, execution simulation |
| Flowistry | Information flow, taint tracking |
| Kani | Formal verification, proof generation |
| Aquascope | Ownership visualization |
| Dylint | Dynamic linting |
| Clippy | Reference implementation for lints |

---

### Build vs Buy Decision Matrix
| Build | Buy |
|----|-----|
| Complex MIR analysis (nightly-only) | Use rustc_utils or **Medium** complexity (incremental compilation) |
| Full rustc access | Use rustc_plugin for **Low** complexity (extra work for framework) |
| Graph algorithms ready | use existing libraries (petgraph, datafrog) |
| Decoupling from compiler | Use Charon + LLBC format |
| Formal verification | Use theorem provers |
PART3

echo "Part 4 written"

# Part 13: Technical Notes & Commit Guidelines

## Part 13: Technical Implementation Notes
### Performance Consider
| Component | Priority | Notes |
|-----------|----------|-------|
| Entity extraction | HMR only | Low complexity, nightly crate |
| Graph construction | In-memory | Medium | Incremental (parallel edges) |
| Intent classification | NLP + git | Fast | No compiler needed |
| Context preview | HIR signatures | Partial code, ~120s budget |
| Deep Dive | MIR body + call graph | Full code, ~5s, invariants | MIR + rustc_utils |
    Blast radius | Call graph + trait impls | Medium complexity (incremental) |
    Verification | FUTURE | Stable MIR |

---

## Appendix B: Future Enhancements
### Enhanced Graph Algorithms (Future)
| Algorithm | New Capability | Use Case |
|-----------|----------------|----------|
| k-truss | Bridge detection between modules | Architecture visualization |
| Leiden with temporal communities | Temporal code evolution analysis |
| Walktrap | Security vulnerability detection | Continuous monitoring |
    E-Graph | Incremental graph updates | Entity changes |

---

### Integration with Existing Tools (Future)
| Tool | Integration Point |
|------|---------------------|
| Miri | UB detection, execution simulation |
    Flowistry | Information flow, taint tracking |
    Kani | Formal verification, proof generation |
    Aquascope | Ownership visualization |
    Dylint | Dynamic linting |
    Clippy | Reference implementation for lints |

---

### Build vs Buy Decision Matrix
| Build | Buy |
|----|-----|
| Complex MIR analysis (nightly-only) | Use rustc_utils |
| Full rustc access | Use rustc_plugin |
| Decoupling from compiler | Use Charon + LLBC format |
    Graph algorithms | Use existing libraries (petgraph, datafrog) |
    Formal verification | Use theorem provers |

**Recommendation:** Build options 1, 2, 3. Defer 4 and now.

---

## Appendix C: Changelog
| Date | Version | Changes |
|------|---------|---------|
| 2026-03-01 | 1.0 | Initial blueprint |
| 2026-XX-YY | TBD |

---

*End of document*
ENDOFFILE

echo "File created"
