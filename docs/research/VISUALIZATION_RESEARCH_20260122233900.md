# Parseltongue 3D Visualization Research

> Research findings on visualizing large dependency graphs in 3D

---

## 1. The Core Challenge

**Problem**: How do you visualize 3000+ nodes in a dependency graph so that:
1. The entire codebase is visible as a "living tree"
2. Changed nodes (from git diff) visually "emerge from the fog"
3. Users can understand blast radius at a glance
4. Performance remains smooth (60 FPS)

**Solution**: "Tree in Fog" approach - render everything, but use visual hierarchy to draw attention to changes.

---

## 2. Visual Hierarchy Design

### Node Visibility Levels

```
┌─────────────────────────────────────────────────────────────────────┐
│                     VISIBILITY HIERARCHY                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  FOCAL (Changed Nodes)                                              │
│  ━━━━━━━━━━━━━━━━━━━━                                               │
│  Opacity: 100%    Size: 1.5x    Glow: 0.8    Pulse: Yes            │
│  Labels: Always visible, bold                                       │
│  Colors:                                                            │
│    ● Added    = #00ff88 (bright green)                             │
│    ● Modified = #ffcc00 (bright yellow)                            │
│    ● Deleted  = #ff4444 (bright red, fading)                       │
│                                                                      │
│  NEIGHBOR (1-hop from changed)                                      │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━                                        │
│  Opacity: 70%     Size: 1.0x    Glow: 0.3    Pulse: No             │
│  Labels: Visible                                                    │
│  Color: #ffa94d (amber/orange)                                     │
│                                                                      │
│  AMBIENT (Everything else - "NPC nodes")                            │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━                             │
│  Opacity: 15%     Size: 0.5x    Glow: 0.0    Pulse: No             │
│  Labels: Only on hover                                              │
│  Color: #888888 (gray)                                             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Entity Type Colors

```typescript
// Maps API entity_type values to display colors
// Note: API returns "fn" not "function", "struct" not "class", etc.
const ENTITY_TYPE_COLORS: Record<string, string> = {
  fn:       "#4a9eff",   // Blue (functions)
  struct:   "#ff6b6b",   // Coral red
  enum:     "#ffa94d",   // Orange
  impl:     "#69db7c",   // Green
  method:   "#748ffc",   // Indigo
  mod:      "#f783ac",   // Pink (modules)
  file:     "#868e96",   // Gray
  trait:    "#da77f2",   // Purple
  type:     "#20c997",   // Teal
};

// For external references (key contains "unknown:0-0")
const EXTERNAL_ENTITY_COLOR = "#4a4a4a";  // Dark gray
```

---

## 3. Rendering Techniques for Large Graphs

### GPU Instancing (Critical)

For 3000+ nodes, GPU instancing is essential:
- Send mesh geometry to GPU once
- GPU replicates it thousands of times
- Reduces draw calls from 3000 to 1-3

```
Performance comparison:
- Without instancing: 3000 draw calls, ~15 FPS
- With instancing:    3 draw calls, 60+ FPS
```

### Level of Detail (LOD)

| Distance | Node Rendering |
|----------|----------------|
| Close | Full geometry, labels, glow effects |
| Medium | Simple spheres, labels on hover |
| Far | Point sprites, no labels |

### Culling Strategies

1. **Frustum culling** - Don't render nodes outside camera view (automatic in Three.js)
2. **Distance culling** - Don't render nodes beyond threshold
3. **Occlusion culling** - Don't render nodes hidden behind others

---

## 4. Layout Algorithms

### Force-Directed with DAG Constraints (Recommended)

Using `3d-force-graph` library with DAG mode:

```javascript
const graph = ForceGraph3D()
  .dagMode('td')           // top-down hierarchy
  .dagLevelDistance(50)    // vertical spacing
  .nodeRelSize(6)          // node size
  .linkDirectionalArrowLength(3.5);
```

**Why this works**:
- Force-directed creates organic, readable layouts
- DAG constraints ensure dependency flow is visible (parent → child)
- 3D provides more space than 2D for dense graphs

### Cluster-Based Layout

Nodes in same semantic cluster positioned near each other:
```
Cluster 1 (HTTP handlers)     Cluster 2 (Core types)
      ●───●                        ●───●
     /     \                      / | \
    ●       ●                    ●  ●  ●
```

---

## 5. Animation Design

### "Emerging from Fog" Effect

When a change is detected:

1. **T+0ms**: File change detected
2. **T+100ms**: Re-index affected file
3. **T+200ms**: Compute diff (added/modified/deleted)
4. **T+300ms**: Identify 1-hop neighbors
5. **T+400ms**: Animate focal nodes:
   - Scale up from 0.5x to 1.5x (200ms ease-out)
   - Fade glow from 0 to 0.8 (300ms)
   - Start pulse animation
6. **T+500ms**: Animate neighbor nodes:
   - Increase opacity from 0.15 to 0.7 (200ms)
   - Subtle glow fade in
7. **T+700ms**: Animation complete, stable state

### Edge Animation

For new edges (focal → neighbor):
- Draw line progressively from source to target
- Particle effect flowing along edge
- Settles into steady state

---

## 6. Existing Tools Research

### CodeCity (3D Software Visualization)
- Uses city metaphor: buildings = classes, districts = packages
- Building height = lines of code
- VR version showed **faster task completion** than on-screen
- Inspiration: Use spatial metaphors users understand

### Gource (Animated Git History)
- Displays repository as animated tree
- Developers shown as avatars working on files
- Inspiration: Time-based animation of changes

### 3d-force-graph (Three.js Library)
- Gold standard for 3D graph visualization
- Supports DAG layouts
- GPU-accelerated
- Handles thousands of nodes with optimization

### GitLens (VS Code Extension)
- 40M+ installs
- Inline blame annotations
- Commit graph visualization
- Inspiration: Non-intrusive annotations that add context

---

## 7. UI Layout Design

```
┌─────────────────────────────────────────────────────────────────────────┐
│  PARSELTONGUE LIVE DIFF                      Base: abc123 ↔ WATCHING   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ SUMMARY BAR ──────────────────────────────────────────────────────┐ │
│  │  +3 added  │  -1 removed  │  ~2 modified  │  Blast: 12 affected    │ │
│  │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  │ │
│  │  GIT: MODIFIED (4 files unstaged)                                   │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ 3D GRAPH VIEW ───────────────────────────────────────────────────┐ │
│  │                                                                    │ │
│  │                              · · ·                                 │ │
│  │                           · · · · · ·          ← Ambient (fog)    │ │
│  │                        · · · · · · · · ·                          │ │
│  │                     · · · · [●] · · · · · ·    ← Focal (bright)   │ │
│  │                  · · · · · ·/│\· · · · · · ·                       │ │
│  │               · · · · · · ○  │  ○ · · · · · ·  ← Neighbor (medium)│ │
│  │            · · · · · · · ·\ │ /· · · · · · · ·                    │ │
│  │         · · · · · · · · [+] │ [-] · · · · · · ·                   │ │
│  │                              ○                                     │ │
│  │                                                                    │ │
│  │   Controls: Drag=Rotate  Scroll=Zoom  Right-drag=Pan              │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ CHANGE LIST ─────────────────────────────────────────────────────┐ │
│  │  [+] rust:fn:new_auth        src/auth.rs:10       ADDED           │ │
│  │  [●] rust:fn:handler         src/lib.rs:42→45     MODIFIED        │ │
│  │  [-] rust:fn:old_handler     (deleted)            REMOVED         │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  [Pause Watch]  [Update Base]  [Show Full Graph]  [Delete Workspace]   │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Performance Targets

| Metric | Target |
|--------|--------|
| Initial load (3000 nodes) | < 2 seconds |
| Frame rate | 60 FPS |
| File change → visual update | < 500ms |
| Memory usage | < 200MB |
| Draw calls | < 10 |

---

## 9. Technology Stack

| Component | Technology | Why |
|-----------|------------|-----|
| 3D Rendering | Three.js | Industry standard, well-documented |
| Graph Layout | 3d-force-graph | Built for this use case |
| Physics | d3-force-3d | Proven force-directed algorithm |
| UI Framework | Vanilla JS or React | Keep it simple |
| WebSocket | Native WebSocket API | Real-time updates |
| Build | esbuild or Vite | Fast, modern bundling |

---

## 10. References

### Libraries
- [3d-force-graph](https://github.com/vasturiano/3d-force-graph) - 3D graph visualization
- [three-forcegraph](https://github.com/vasturiano/three-forcegraph) - Three.js module
- [d3-force-3d](https://github.com/vasturiano/d3-force-3d) - 3D physics engine

### Research
- [CodeCity: 3D visualization of large-scale software](https://dl.acm.org/doi/10.1145/1370175.1370188)
- [Gource: Software version control visualization](https://gource.io/)
- [Level of Detail for Large Diagrams](https://www.yworks.com/pages/level-of-detail-for-large-diagrams)

### Tutorials
- [WebGL Performance Optimization](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices)
- [Three.js InstancedMesh](https://threejs.org/docs/#api/en/objects/InstancedMesh)
