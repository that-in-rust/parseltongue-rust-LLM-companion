# Interface Signature Graph Visualization: A Design Thesis

**Date**: 2025-01-13
**Status**: Design Research
**Context**: Parseltongue Dependency Graph Generator v1.2.0

---

## Abstract

This thesis explores how developers should visualize **interface signature graphs** and **dependency graphs** of Rust codebases. Through deep analysis of user mental models, jobs to be done, and the unique characteristics of Rust's trait system, I propose five distinct visualization metaphors—each optimized for specific questions developers ask when engaging with code architecture.

The core insight: **No single visualization serves all needs**. Developers cycle through different mental modes (orientation, investigation, refactoring, audit), and each mode benefits from a different visual metaphor.

---

## Part 1: The User Journey Analysis

### The Mental Models of Code Comprehension

When a developer approaches a codebase visualization, they are rarely in a neutral state. Their cognitive frame is shaped by an immediate need:

| Mental State | Core Question | Emotional State | Time Pressure |
|--------------|---------------|-----------------|---------------|
| **Disorientation** | "Where am I?" | Confusion, anxiety | High |
| **Investigation** | "Why is this happening?" | Curiosity, frustration | Medium |
| **Refactoring** | "What breaks if I touch this?" | Caution, dread | Low (need thoroughness) |
| **Audit** | "Is this architecture sound?" | Judgment, scrutiny | Low (need depth) |
| **Teaching** | "How do I explain this?" | Clarity-seeking | Variable |

A visualization that fails to acknowledge these states fails the user. A disoriented developer needs **landmarks**, not exhaustive detail. An auditing developer needs **provable completeness**, not artistic abstraction.

### The Jobs To Be Done

When developers open a dependency visualization, they are trying to accomplish specific jobs:

#### Job 1: Orient Thyself
*"I just joined this team. I need to know what exists here without reading 10,000 lines of code."*

**Success criteria**: Within 60 seconds, I can name the main modules, identify the complexity hotspots, and spot the architecture pattern (layered? hexagonal? microkernel?).

#### Job 2: Trace the Blast Radius
*"Product wants to change behavior X. I need to know what breaks."*

**Success criteria**: I can see the complete transitive closure of dependencies. I know which tests will fail. I can communicate risk in concrete terms.

#### Job 3: Find the Smoking Gun
*"We have a bug in production. The error trace points here. Why is this happening?"*

**Success criteria**: I can trace the execution path backward. I can identify hidden coupling (temporal dependencies). I can form testable hypotheses.

#### Job 4: Assess Technical Debt
*"We need to decide between refactoring and rewriting. What's the structural health?"*

**Success criteria**: I can quantify coupling. I can identify architectural violations (cycles, layer violations). I can prioritize hotspots.

#### Job 5: Understand Rust's Trait Web
*"What implements this trait? What traits does this type implement? Why is the compiler yelling at me?"*

**Success criteria**: I can see the trait hierarchy. I can identify blanket implementations. I can understand type inference paths.

### The Rust Context: Why Interface Signatures Matter

Rust's trait system creates unique visualization challenges:

1. **Traits are interfaces, not classes**—multiple types can implement the same trait
2. **Blanket implementations**—traits can be implemented for all types satisfying a constraint
3. **Trait bounds as constraints**—functions are only callable for types satisfying certain traits
4. **Associated types**—traits can expose types as part of their signature
5. **Coherence rules**—the compiler prohibits overlapping implementations

A Rust codebase's dependency graph is fundamentally a **type-level constraint graph**, not just a call graph. Visualizing it requires showing:
- Which types implement which traits
- Which functions require which trait bounds
- Where trait bounds propagate through the call chain
- Which constraints are inferred vs explicit

---

## Part 2: Five Visualization Metaphors

### Metaphor 1: Circular CodeCity (Current Implementation)

**The Core Concept**: Buildings arranged on a circular periphery, with curved arcs passing through the center showing dependencies.

```
         [Building] ===arc=== [Building]
              /                     \
        [Building]               [Building]
               \                   /
                 =====arc=====
```

**What It Offers**:

| Strength | Weakness |
|----------|----------|
| Excellent for seeing "who talks to whom" at a glance | Poor at showing hierarchy/layering |
| Arcs through center create visual focus on connections | Becomes illegible with >500 edges (hairball) |
| Module grouping on circle is intuitive | Circular arrangement doesn't reflect architecture |
| Neon aesthetic against dark background is striking | Color coding only shows entity type, not relationship semantics |

**Jobs It Serves Best**:
- **Job 1 (Orientation)**: Quick scan of entity types and their distribution
- **Job 2 (Blast Radius)**: Seeing how many connections converge on a building

**Parseltongue Integration**:
- Uses `/code-entities-list-all` for building placement
- Uses `/dependency-edges-list-all` for arc rendering
- Could use `/semantic-cluster-grouping-list` for intelligent grouping

**When to Use**:
- Small-to-medium codebases (<500 entities)
- Quick "health check" visualizations
- Demonstrations and stakeholder communication

**Verdict**: **Keep as default mode** for first-time visitors, but offer alternatives.

---

### Metaphor 2: Force-Directed Galaxy

**The Core Concept**: Nodes float in space based on physics simulation—entities that connect heavily are pulled closer. Traits form gravitational centers that attract their implementors.

```
        [Trait Node]
       /      |      \
  [Struct]  [Struct]  [Struct]
      |        |         |
  [Method] [Method]   [Method]
```

**What It Offers**:

| Strength | Weakness |
|----------|----------|
| Self-organizing—clusters emerge naturally from topology | Unpredictable layout—different each time |
| Dense clusters visually indicate high coupling | Can be unstable (jittery) during simulation |
- Traits naturally become cluster centers | Requires careful tuning of physics parameters |
| Excellent for "neighborhood exploration"—click and expand | Hard to create stable screenshots |

**Jobs It Serves Best**:
- **Job 1 (Orientation)**: Understanding module structure organically
- **Job 4 (Technical Debt)**: Spotting coupling hotspots (dense clumps)
- **Job 5 (Trait Web)**: Seeing trait implementation clusters

**Parseltongue Integration**:
- Uses `/semantic-cluster-grouping-list` for initial placement hints
- Uses `/forward-callees-query-graph` and `/reverse-callers-query-graph` for edge weights
- Uses `/complexity-hotspots-ranking-view` to color-code hotspots

**Implementation Considerations**:
- Use `d3-force-3d` for physics simulation
- Implement "pinned nodes" for stability (traits as anchors)
- Add "cluster collapse" for progressive disclosure
- Animate changes when filtering

**Verdict**: **High priority** for Rust trait visualization. Makes trait hierarchies immediately intuitive.

---

### Metaphor 3: Hierarchical Terrain Map

**The Core Concept**: A 2.5D topographic map where elevation represents architectural layer. Dependencies flow "downhill." Plateaus represent modules, rivers represent dependency paths.

```
    [HTTP Layer] ←── elevation: high
         │
         ↓ (dependency river)
    [Service Layer]
         │
         ↓
    [Repository Layer]
         │
         ↓
    [Storage/DB]
```

**What It Offers**:

| Strength | Weakness |
|----------|----------|
| Layer violations are visually obvious (uphill edges) | Requires accurate layer detection (not always available) |
| Natural metaphor—water flows downhill | Can obscure horizontal dependencies within layers |
| Elevation as metric—height = abstraction level | 3D terrain can be occlusion-heavy |
| Excellent for architecture audits | Requires good color coding for depth perception |

**Jobs It Serves Best**:
- **Job 4 (Technical Debt)**: Architecture audits, detecting layer violations
- **Job 2 (Refactoring)**: Understanding cross-layer impact
- **Job 1 (Orientation)**: Understanding high-level architecture

**Parseltongue Integration**:
- Uses `/semantic-cluster-grouping-list` to infer layers
- Uses `/circular-dependency-detection-scan` to highlight cycles (they become "waterfalls")
- Uses `/dependency-edges-list-all` with edge direction for flow

**Implementation Considerations**:
- Infer layer from call patterns: entities with many incoming edges, few outgoing = lower layer
- Color edges by direction: blue (downhill, normal), red (uphill, suspicious)
- Implement "cross-section view" to see layered slices

**Verdict**: **Medium priority**—valuable for architecture reviews but requires layer inference heuristics.

---

### Metaphor 4: Trait Constellation

**The Core Concept**: A trait-centric view where traits are "constellations" and implementing types are "stars" connected to them. Function calls form "nebulae" between constellations.

```
        [Trait: Iterator] ◇────────◇ [Trait: IntoIterator]
              ●  ●  ●                    ●
              ●  ●  ●                    ●
        [Struct: Vec]              [Struct: Range]
```

**What It Offers**:

| Strength | Weakness |
|----------|----------|
| Optimized for Rust's trait system | Less useful for languages without traits |
| Blanket implementations become visual patterns | Requires trait-specific parsing |
| Trait bounds visible as constraint chains | Can be complex for deeply nested bounds |
| Associated types shown as satellite nodes | Multiple views needed (trait view vs call view) |

**Jobs It Serves Best**:
- **Job 5 (Trait Web)**: Understanding trait implementations
- **Job 3 (Debugging)**: "Why doesn't my type implement this trait?"
- **Job 1 (Orientation)**: Understanding the type system landscape

**Parseltongue Integration**:
- Requires new trait-specific endpoint or filtering `/code-entities-list-all` for `entity_type=trait`
- Uses `/forward-callees-query-graph` to show trait method calls
- Uses `/reverse-callers-query-graph` to show where trait bounds are required

**Implementation Considerations**:
- Traits as diamond-shaped nodes (distinct from structs)
- Implementations as dashed lines with labels (blanket impls differentiated)
- Trait bounds shown as constraint annotations on functions
- "Trait explorer" mode: click trait to see all implementors

**Verdict**: **High priority for Rust**—unique value proposition, leverages Parseltongue's parsing capabilities.

---

### Metaphor 5: Signature Gallery

**The Core Concept**: A museum-like layout where entities are "paintings" on walls, grouped by module. Dependencies are "threads" running between frames. Zooming in reveals signatures, source code, and documentation.

```
┌─────────────────────────────────────────────────────────────┐
│  [auth.rs] Module                                           │
│  ┌─────┐  ╱╲╱╲  ┌─────┐                                    │
│  │Auth │ ╱    ╲ │Login│                                    │
│  └─────┘╱      ╲└─────┘                                    │
│           ╲    ╱                                             │
│            ╲  ╱                                              │
│  ┌──────────────────┐                                       │
│  │  [user.rs]       │                                       │
│  │  ┌─────┐ ┌─────┐ │                                       │
│  │  │User │ │Role │ │                                       │
│  │  └─────┘ └─────┘ │                                       │
│  └──────────────────┘                                       │
└─────────────────────────────────────────────────────────────┘
```

**What It Offers**:

| Strength | Weakness |
|----------|----------|
| Familiar 2D layout, no camera acrobatics | Limited spatial relationships |
| Rich information density on zoom | Doesn't scale to 1000+ entities well |
| Natural reading order (left-to-right) | Less "wow factor" for demos |
| Module boundaries as actual walls | Harder to show cross-module dependencies cleanly |

**Jobs It Serves Best**:
- **Job 1 (Orientation)**: Traditional codebase overview
- **Job 3 (Investigation)**: Detailed code reading
- **Job 5 (Teaching)**: Walking through code with a team

**Parseltongue Integration**:
- Uses `/semantic-cluster-grouping-list` for wall/group assignment
- Uses `/code-entity-detail-view` for zoomed-in content
- Uses `/api-reference-documentation-help` for doc displays

**Implementation Considerations**:
- Virtual canvas with pan/zoom (infinite canvas pattern)
- Progressive LOD: boxes → signatures → source code
- Thread curvature for dependencies between modules
- Search highlights entities like a gallery guide

**Verdict**: **Medium priority**—complementary to 3D views, better for focused reading sessions.

---

## Part 3: Decision Framework

### Which Visualization for Which Job?

| Job | Best Metaphor | Rationale |
|-----|---------------|-----------|
| **Orientation (New to codebase)** | Force-Directed Galaxy OR Signature Gallery | Galaxy for organic understanding; Gallery for structured overview |
| **Blast Radius (What breaks?)** | Circular CodeCity OR Terrain Map | Circular for visual convergence; Terrain for cross-layer impact |
| **Investigation (Bug hunting)** | Trait Constellation | For type-related bugs; otherwise Force-Directed |
| **Refactoring (Safe changes)** | Terrain Map + CodeCity combo | Terrain for architecture; CodeCity for direct connections |
| **Audit (Tech debt)** | Terrain Map | Layer violations and cycle detection |
| **Trait Understanding** | Trait Constellation | Purpose-built for this |
| **Teaching/Onboarding** | Signature Gallery + Force-Directed | Gallery for structure; Galaxy for relationships |

### Multi-Mode Exploration Strategy

Instead of choosing one visualization, implement **seamless mode transitions**:

1. **Entry**: Circular CodeCity (striking, understandable)
2. **Group view**: Click to expand Force-Directed Galaxy of cluster
3. **Detail view**: Click entity for Signature Gallery zoom-in
4. **Trait view**: Toggle Trait Constellation overlay
5. **Architecture view**: Switch to Terrain Map for audit

**Key Insight**: The user's mental state changes during exploration. Allow fluid transitions between metaphors without losing context.

### Progressive Disclosure Principles

Regardless of metaphor, follow these rules:

1. **Level 0 (30,000 ft)**: Show only modules and trait clusters. Hide individual entities.
2. **Level 1 (10,000 ft)**: Show structs, impl blocks, trait definitions. Hide methods.
3. **Level 2 (1,000 ft)**: Show public methods. Hide private methods.
4. **Level 3 (Ground)**: Show all entities with signatures.

**Parseltongue endpoints for each level**:
- Level 0: `/semantic-cluster-grouping-list`
- Level 1: `/code-entities-list-all?entity_type=struct,trait,impl,enum`
- Level 2: `/code-entities-list-all` filtered for public access
- Level 3: `/code-entity-detail-view` for each entity

---

## Part 4: Implementation Roadmap

### Phase 1: Foundation (Current State)
- ✅ Circular CodeCity with neon arcs
- ✅ Basic entity type coloring
- ✅ Click-to-select details panel
- ✅ Parseltongue API integration

### Phase 2: Mode Switching (Immediate Next Step)
- Add view mode selector: [Circular | Force | Terrain | Constellation | Gallery]
- Implement Force-Directed Galaxy using `3d-force-graph`
- Implement smooth camera transitions between modes

### Phase 3: Rust-Specific Features
- Trait Constellation view with trait-implementor edges
- Blanket implementation visualization
- Trait bound annotation display

### Phase 4: Terrain Map
- Layer inference algorithm
- Elevation calculation based on abstraction level
- Upside-down dependency highlighting

### Phase 5: Signature Gallery
- 2D infinite canvas with zoom
- LOD-based content (box → signature → source)
- Module wall boundaries

### Phase 6: Advanced Interactions
- Multi-select for comparison
- Time-travel (git history overlay)
- Collaboration (shared views, annotations)

---

## Part 5: Design Principles

### Principle 1: The 60-Second Rule

A new user must understand the visualization within 60 seconds. If they need a tutorial, the design has failed.

**Test**: Show screenshot to a developer. If they can explain what they're looking at in under a minute, pass.

### Principle 2: Respect Cognitive Load

Each visual element should serve a purpose. If removing an element doesn't break understanding, remove it.

**Heuristics**:
- Max 7 colors (human working memory limit)
- Max 3 concurrent animation types
- Max 5 panel/sidebar sections

### Principle 3: Make the Invisible Visible

Parseltongue provides data that grep cannot. Visualizations should highlight:
- Temporal coupling (files that change together without code edges)
- Hidden dependencies via trait bounds
- Transitive impact beyond direct calls

**Action**: Use `/temporal-coupling-hidden-deps` to render "ghost edges" in a different color.

### Principle 4: Support Both Exploration and Explanation

The same tool serves both solo exploration and team explanation. Provide:
- Presentation mode (larger fonts, fewer details, clear labels)
- Investigation mode (dense information, filtering, search)
- Export mode (screenshots, video recording, shareable URLs)

---

## Part 6: Measuring Success

### Metrics for Each Visualization

| Metaphor | Primary Metric | Secondary Metric |
|----------|----------------|------------------|
| Circular CodeCity | Time to first click (engagement) | Recall of entity distribution |
| Force-Directed Galaxy | Cluster identification accuracy | Time to find hotspots |
| Terrain Map | Layer violation detection rate | Architecture audit confidence |
| Trait Constellation | Trait relationship accuracy | Compiler error resolution time |
| Signature Gallery | Code reading session duration | Return visit frequency |

### A/B Testing Framework

For each feature, run a controlled experiment:
- Group A: Current visualization
- Group B: New visualization
- Measure: Task completion time, error rate, satisfaction survey

---

## Conclusion

Interface signature graph visualization is not a solved problem. The "right" visualization depends on:
1. The user's mental state (disoriented vs investigative vs auditing)
2. The specific job (orientation vs blast radius vs trait understanding)
3. The codebase characteristics (Rust traits vs Java classes vs Python functions)

**Thesis Statement**: The optimal interface signature graph visualization for Rust codebases is not a single view, but a **multi-modal exploration environment** that allows seamless transitions between:
- **Circular CodeCity** for quick orientation and blast radius assessment
- **Force-Directed Galaxy** for understanding module clustering and trait relationships
- **Hierarchical Terrain Map** for architecture audits and layer violation detection
- **Trait Constellation** for Rust-specific type system comprehension
- **Signature Gallery** for detailed code reading and team walkthroughs

Parseltongue's 15 HTTP endpoints provide all the data needed for these views. The challenge is not data availability—it's **designing the right visual metaphor for each mental mode**.

The path forward:
1. Build **Force-Directed Galaxy** next (highest ROI, leverages existing data)
2. Add **Trait Constellation** for Rust-specific value (unique differentiation)
3. Implement **mode switching** for fluid exploration
4. Add **Terrain Map** for architecture audits (specialized use case)
5. Round out with **Signature Gallery** for focused reading (complementary)

The goal is not prettier pictures—it's **faster, more accurate understanding of code architecture**. Every design decision should serve that goal.

---

## References

- Shreyas Doshi's product philosophy: Mental models, jobs to be done, reducing cognitive friction
- Munir et al., "CodeCity: A 3D Visualization of Software Evolution" (original CodeCity thesis)
- d3-force-3d library for physics-based graph layout
- 3d-force-graph for production-ready force-directed 3D graphs
- Parseltongue README and API Reference (15 endpoints, token-efficient queries)

---

**Generated**: 2025-01-13
**Author**: Claude Opus 4.5
**Context**: Parseltongue Dependency Graph Generator v1.2.0
**Branch**: `research/visualization-improvements-20260110-1914`
