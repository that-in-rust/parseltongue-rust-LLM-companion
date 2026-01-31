# Parseltongue Visualization Research Thesis

> **Research Branch**: `research/visualization-improvements-20260110-1914`  
> **Date**: 2026-01-10  
> **Author**: AI Research Assistant

---

## Executive Summary

Parseltongue already provides a rich HTTP API with 15 endpoints exposing code entities, dependency graphs, impact analysis, and complexity metrics. This document explores **how to transform this data into compelling visual experiences** that help developers understand, navigate, and analyze codebases.

---

## Part 1: Current State Analysis

### What Parseltongue Already Provides

| Endpoint | Data Type | Visualization Potential |
|----------|-----------|------------------------|
| `/code-entities-list-all` | Nodes (215+) | Node catalog, filterable list |
| `/dependency-edges-list-all` | Edges (2880+) | Graph edges, connection lines |
| `/reverse-callers-query-graph` | Inbound connections | Radial dependency view |
| `/forward-callees-query-graph` | Outbound connections | Call tree visualization |
| `/blast-radius-impact-analysis` | Transitive impact | Ripple/explosion animation |
| `/complexity-hotspots-ranking-view` | Ranked metrics | Heatmap, size-scaled nodes |
| `/semantic-cluster-grouping-list` | Grouped entities | Cluster bubbles, force layout |
| `/circular-dependency-detection-scan` | Cycles | Highlighted loop paths |
| `/smart-context-token-budget` | Prioritized context | Relevance-weighted view |

### Gap Analysis

| Missing Capability | Impact | Effort |
|-------------------|--------|--------|
| No web UI | Requires curl/code | High |
| No snapshot comparison | Can't see evolution | Medium |
| No visual diff | Text-only changes | Medium |
| No real-time updates | Static snapshots | Low |

---

## Part 2: Visual User Journeys

### Journey 1: "I Just Joined This Team"

**Goal**: Understand codebase architecture in < 30 minutes

```
┌──────────────────────────────────────────────────────────────────┐
│  CODEBASE EXPLORER - Interactive Overview                        │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│   [Language Filter: Rust ▾]  [Entity Type: All ▾]  [🔍 Search]  │
│                                                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                                                         │   │
│   │      ┌────────┐    ┌────────┐    ┌────────┐            │   │
│   │      │ Core   │───▶│Storage │───▶│ CozoDB │            │   │
│   │      │  68    │    │   12   │    │   5    │            │   │
│   │      └────────┘    └────────┘    └────────┘            │   │
│   │           │                                             │   │
│   │           ▼                                             │   │
│   │      ┌────────┐    ┌────────┐                          │   │
│   │      │ HTTP   │───▶│Handlers│                          │   │
│   │      │  45    │    │   89   │                          │   │
│   │      └────────┘    └────────┘                          │   │
│   │                                                         │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│   Click any cluster to drill down │ Size = entity count         │
└──────────────────────────────────────────────────────────────────┘
```

**API Source**: `/semantic-cluster-grouping-list` + `/codebase-statistics-overview-summary`

---

### Journey 2: "What Happens If I Change This?"

**Goal**: Understand impact before making changes

```
┌──────────────────────────────────────────────────────────────────┐
│  BLAST RADIUS ANALYZER                                           │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Focus Entity: [CozoDbStorage::new                    ▾] [Analyze]│
│                                                                  │
│  Hops: ○1  ●2  ○3  ○5       Affected: 278 entities              │
│                                                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                         ●                               │   │
│   │                   ╱───────────╲                         │   │
│   │                  ●             ●                        │   │
│   │                 ╱│╲           ╱│╲                       │   │
│   │                ● ● ●         ● ● ●    ◄── Hop 1 (red)   │   │
│   │               ╱│╲ │ ╲       ╱│╲ │ ╲                     │   │
│   │              ○○○ ○ ○○      ○○○ ○ ○○   ◄── Hop 2 (orange)│   │
│   │                                                         │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│   ⚠️ HIGH RISK: 213 direct callers, 65 at depth 2               │
│   📋 Export affected list   🔗 View all callers                 │
└──────────────────────────────────────────────────────────────────┘
```

**API Source**: `/blast-radius-impact-analysis?entity=X&hops=N`

---

### Journey 3: "Show Me How This Codebase Evolved"

**Goal**: Compare two snapshots to see what changed

```
┌──────────────────────────────────────────────────────────────────┐
│  SNAPSHOT DIFF VIEWER                                            │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Snapshot A: [parseltongue_20260101  ▾]                          │
│  Snapshot B: [parseltongue_20260110  ▾]    [Compare]             │
│                                                                  │
│  ┌─────────────────────┬─────────────────────────────────────┐  │
│  │   BEFORE (Jan 1)    │      AFTER (Jan 10)                 │  │
│  ├─────────────────────┼─────────────────────────────────────┤  │
│  │                     │                                     │  │
│  │   [  ] storage      │   [■■] storage (+5 entities)        │  │
│  │   [  ] handlers     │   [■ ] handlers (+2 entities)       │  │
│  │   [■■] parsing      │   [■■] parsing (unchanged)          │  │
│  │   [■ ] core         │   [  ] core (-3 entities)           │  │
│  │                     │                                     │  │
│  └─────────────────────┴─────────────────────────────────────┘  │
│                                                                  │
│  Summary: +7 entities, -3 entities, +45 edges, -12 edges         │
│                                                                  │
│  🟢 Added: new_handler, validate_input, parse_swift              │
│  🔴 Removed: legacy_parser, old_cache, deprecated_fn             │
│  🟡 Modified: CozoDbStorage (lines 23-45 → 23-67)                │
└──────────────────────────────────────────────────────────────────┘
```

**Requires**: New endpoint `/snapshot-comparison-diff-report?db1=X&db2=Y`

---

### Journey 4: "Where Are The Problem Areas?"

**Goal**: Find complexity hotspots and technical debt

```
┌──────────────────────────────────────────────────────────────────┐
│  COMPLEXITY HEATMAP                                              │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  View: ○ Treemap  ●Heatmap  ○ Bar Chart    Top: [20 ▾]          │
│                                                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │ ████████████████████████████████  new()         213     │   │
│   │ ██████████████████████████        unwrap()      158     │   │
│   │ ████████████████████              to_string()   124     │   │
│   │ █████████████                     Ok()           83     │   │
│   │ ██████████                        Some()         61     │   │
│   │ ████████                          clone()        48     │   │
│   │ ██████                            handle_*       35     │   │
│   │ █████                             get_entity     28     │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│   🔥 Hottest: stdlib calls dominate (expected)                   │
│   ⚠️ Code hotspot: CozoDbStorage (68 inbound, 24 outbound)       │
└──────────────────────────────────────────────────────────────────┘
```

**API Source**: `/complexity-hotspots-ranking-view?top=N`

---

## Part 3: Technology Recommendations

### Option A: Lightweight Static Dashboard (Recommended for MVP)

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Framework | **Vanilla HTML/JS** | Zero build step, embed anywhere |
| Graph Library | **Cytoscape.js** | Purpose-built for graphs, MIT license |
| Layout | **Cola.js** (Cytoscape plugin) | Constraint-based layouts |
| Styling | **CSS Variables** | Dark/light theme support |
| Deployment | **Single HTML file** | Self-contained, works offline |

**Effort**: ~3 days to MVP

---

### Option B: Full Interactive Web App

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Framework | **Svelte** or **SolidJS** | Lightweight, fast reactivity |
| Graph Library | **D3.js** | Maximum customization |
| 3D Option | **Three.js** | CodeCity-style 3D view |
| State | **Zustand** | Simple, no boilerplate |
| API | Connect to existing parseltongue HTTP |

**Effort**: ~2 weeks to v1.0

---

### Option C: VS Code Extension

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Webview | Built-in VS Code | Direct IDE integration |
| Graph | Cytoscape.js in webview | Familiar patterns |
| Commands | Extension API | "Show impact of this function" |
| Integration | Language Server Protocol | Jump to definition |

**Effort**: ~1 week for basic, ~3 weeks for full

---

## Part 4: Creative Ideas 💡

### Idea 1: "Code Galaxy" - 3D Space Visualization

Represent codebase as a galaxy where:
- **Stars** = Entities (brightness = complexity)
- **Constellations** = Semantic clusters
- **Orbits** = Dependency relationships
- **Black holes** = Circular dependencies
- **Nebulae** = Untested code regions

Users can "fly through" the codebase, zooming into clusters.

---

### Idea 2: "Time Machine" - Animated Evolution

Using timestamped databases that already exist:
1. Load all `parseltongue_YYYYMMDD*` databases
2. Create keyframes of entity/edge counts
3. Animate graph morphing between states
4. Show "births" (green pulse) and "deaths" (red fade)
5. Soundtrack: complexity score as audio waveform

---

### Idea 3: "Impact Ripple" - Animated Blast Radius

When querying blast radius:
1. Center entity pulses
2. Wave expands outward (hop 1)
3. Second wave (hop 2)
4. Each wave colored by risk level
5. Affected entities "shake" briefly

---

### Idea 4: "Conversation Mode" - LLM-Guided Exploration

Integrate with Claude/GPT to enable:
```
User: "Show me what depends on the storage layer"
LLM: *calls /code-entities-search-fuzzy?q=storage*
     *calls /reverse-callers-query-graph for each*
     *renders combined graph*
     "I found 12 storage entities with 89 dependents. 
      The main entry point is CozoDbStorage::new()."
```

Natural language → API calls → Visualization

---

### Idea 5: "Code Weather" - Dashboard Metrics

Daily summary like a weather report:
- **Temperature**: Overall complexity trend (↑ hotter = more complex)
- **Pressure**: Dependency density
- **Storms**: Circular dependency alerts
- **Forecast**: Predicted impact areas based on recent changes

---

### Idea 6: "Diff Theatre" - Side-by-Side Visual Comparison

Two-panel view:
| Left Panel (Before) | Right Panel (After) |
|---------------------|---------------------|
| Graph state at T1 | Graph state at T2 |
| Synchronized zoom/pan | |
| Entities fade in/out based on diff | |
| Edges animate their changes | |

---

### Idea 7: "Function Lineage" - Call Stack Visualization

For any function, show its complete story:
```
main() 
  └─▶ run_server()
        └─▶ handle_request()
              └─▶ query_database()  ◀── YOU ARE HERE
                    └─▶ CozoDbStorage::get()
                          └─▶ cozo::run_query()
```

Forward and backward tracing with depth control.

---

## Part 5: Implementation Roadmap

### Phase 1: Foundation (Week 1)
- [ ] Create `/static` directory for web assets
- [ ] Build minimal HTML viewer fetching from API
- [ ] Implement basic Cytoscape.js graph rendering
- [ ] Add entity click → detail panel

### Phase 2: Core Visualizations (Week 2-3)
- [ ] Cluster overview (semantic grouping)
- [ ] Blast radius explorer
- [ ] Complexity heatmap
- [ ] Search + filter panel

### Phase 3: Temporal Features (Week 4)
- [ ] Implement snapshot comparison endpoint
- [ ] Build diff visualization
- [ ] Add timeline slider

### Phase 4: Polish (Week 5)
- [ ] Dark/light themes
- [ ] Export to PNG/SVG
- [ ] Shareable URLs
- [ ] Performance optimization

---

## Part 6: API Enhancements Needed

| New Endpoint | Purpose | Priority |
|--------------|---------|----------|
| `GET /snapshot-list` | List available databases | High |
| `GET /snapshot-diff?a=X&b=Y` | Compare two snapshots | High |
| `GET /graph-export-cytoscape` | Pre-formatted for Cytoscape.js | Medium |
| `GET /graph-export-d3` | Pre-formatted for D3.js | Medium |
| `GET /entity-timeline?key=X` | History of single entity | Low |
| `WS /live-updates` | WebSocket for real-time | Low |

---

## Appendix A: Competitive Analysis

| Tool | Strengths | Weaknesses | Parseltongue Advantage |
|------|-----------|------------|------------------------|
| **CodeScene** | Behavioral analysis | Expensive, hosted | Open-source, local |
| **Sourcetrail** | Beautiful UI | Discontinued | Active development |
| **CodeSee** | Auto-mapping | SaaS only | Self-hosted, private |
| **Understand** | Deep analysis | Complex, expensive | Simple HTTP API |

---

## Appendix B: Visualization Library Comparison

| Library | Graph Support | 3D | Animation | Learning Curve | License |
|---------|--------------|-----|-----------|----------------|---------|
| **D3.js** | Excellent | Via plugins | Excellent | Steep | BSD |
| **Cytoscape.js** | Excellent | No | Good | Medium | MIT |
| **Vis.js** | Good | Yes | Good | Easy | Apache 2 |
| **Three.js** | Manual | Yes | Excellent | Steep | MIT |
| **Sigma.js** | Excellent | No | Limited | Easy | MIT |

**Recommendation**: Cytoscape.js for 2D graphs, Three.js for optional 3D mode.

---

## Conclusion

Parseltongue has **all the data needed** for rich visualization. The HTTP API already exposes:
- Complete entity graphs
- Dependency relationships
- Impact analysis algorithms
- Complexity metrics
- Semantic clustering

The missing piece is **a web frontend** that consumes this data. Starting with a simple Cytoscape.js-based viewer would provide immediate value, with room to grow into more ambitious features like snapshot comparison and LLM-guided exploration.

**Recommended First Step**: Build a single-page HTML viewer that:
1. Connects to running parseltongue server
2. Fetches cluster overview
3. Renders interactive graph
4. Links nodes to entity details

This can be done in **~3 days** and would immediately demonstrate the value of visualization.

---

*End of Research Document*
