# Draw.io Diagram Guide — LLM-Enhanced Edition

*Foundation: lessons learned building the parseltongue v1.6.1 crate dependency graph via Claude Code + @drawio/mcp*
*Enhanced with: GenAI-DrawIO-Creator research (arXiv 2601.05162), drawio-ninja validation techniques, mxGraph API reference, WCAG accessibility standards, and Gestalt perceptual design principles*

---

## Table of Contents

1. [Setup](#setup)
2. [Box Sizing Rules](#box-sizing-rules)
3. [Layout Geometry](#layout-geometry)
4. [Grouping (Background Boxes)](#grouping-background-boxes)
5. [Arrows / Edges](#arrows--edges)
6. [Edge Routing — Advanced](#edge-routing--advanced)
7. [Color Palette (Semantic Coding)](#color-palette-semantic-coding)
8. [Color Theory for Diagrams](#color-theory-for-diagrams)
9. [Layout Algorithm Selection](#layout-algorithm-selection)
10. [Cognitive Load Reduction](#cognitive-load-reduction)
11. [Style Reference Sheet](#style-reference-sheet)
12. [LLM Prompting Strategy](#llm-prompting-strategy)
13. [Iterative Refinement Workflow](#iterative-refinement-workflow)
14. [Anti-patterns Catalogue](#anti-patterns-catalogue)
15. [Common Mistakes](#common-mistakes)
16. [Workflow](#workflow)
17. [File Formats](#file-formats)

---

## Setup

```bash
# Install MCP server (one-time)
claude mcp add drawio -- npx -y @drawio/mcp

# Verify
claude mcp get drawio

# Use in Claude Code
mcp__drawio__open_drawio_xml  # pass raw XML string, NOT a file path
```

**Critical**: The `open_drawio_xml` tool takes the **XML string as content**, not a file path.
Pass the XML inline. If the XML is in a file, `cat` it first and pass the result.

### Mandatory XML skeleton

Every draw.io file must contain exactly two foundation cells or it will not open. LLMs commonly omit these invisible structural cells. Always start from this skeleton:

```xml
<mxfile>
  <diagram name="Diagram" id="diagram-1">
    <mxGraphModel dx="1422" dy="762" grid="1" gridSize="10"
                  guides="1" tooltips="1" connect="1" arrows="1"
                  fold="1" page="0" pageScale="1"
                  pageWidth="1169" pageHeight="827"
                  math="0" shadow="0">
      <root>
        <!-- Foundation cell 1: REQUIRED, must be id="0" -->
        <mxCell id="0" />
        <!-- Foundation cell 2: REQUIRED, must be id="1" parent="0" -->
        <mxCell id="1" parent="0" />
        <!-- All diagram content goes here, with parent="1" -->
      </root>
    </mxGraphModel>
  </diagram>
</mxfile>
```

**Rule**: `id="0"` and `id="1"` with `parent="0"` are non-negotiable structural anchors. Never reuse these IDs.

---

## Box Sizing Rules

### Rule 1: Measure text length before setting width

At `fontSize=11`, each character is ~7px wide. Safe formula:

```
width = ceil(char_count x 7) + 20px padding
```

| Text | Chars | Min width |
|------|-------|-----------|
| `fn main()` | 9 | 83 → use **120** |
| `fn build_cli()` | 14 | 118 → use **160** |
| `fn run_http_code_query_server()` | 31 | 237 → use **268** |
| `fn run_folder_to_cozodb_streamer()` | 34 | 258 → use **280** |
| `enum HttpServerErrorTypes` | 25 | 195 → use **240** |

**Rule**: Never hardcode width without calculating `ceil(chars x 7) + 20`. Add 10% safety margin for proportional fonts.

### Rule 2: For multi-line text, use `&#xa;` and set height >= 50

```xml
<!-- Bad: text overflows -->
<mxCell value="blast_radius_impact_analysis"
  style="...fontSize=10;" ...>
  <mxGeometry width="150" height="35" />
</mxCell>

<!-- Good: explicit line break, taller box -->
<mxCell value="blast_radius&#xa;impact_analysis"
  style="...fontSize=10;whiteSpace=wrap;html=1;" ...>
  <mxGeometry width="200" height="50" />
</mxCell>
```

**Formula**: `height = ceil(line_count x fontSize x 1.5) + 16px padding`. For `fontSize=11`, two lines = `ceil(2 x 11 x 1.5) + 16 = 49` → use 50.

### Rule 3: Always set `whiteSpace=wrap;html=1;` on every box

Without this, text overflows silently — the box stays the same size but text spills outside.

### Rule 4: Use consistent widths within a row

Pick one width for all items in a logical group (e.g. all handler boxes = 200px).
Mixed widths create visual noise even when text fits.

### Rule 5: fontSize tiers and when to use each

| Context | fontSize | Notes |
|---------|----------|-------|
| Section title label | 14–16 | Bold, fontStyle=1 |
| Module/crate label | 12–13 | Bold |
| Function/entity label | 10–11 | Normal weight |
| Annotation/edge label | 9–10 | Italic, fontStyle=2 |
| Legend text | 9 | Normal |

**Rule**: Never mix more than 3 font size levels in one diagram. Visual hierarchy breaks down past 3 levels.

---

## Layout Geometry

### Plan on paper before writing XML

```
Total canvas width: 1900px
Padding left/right: 30px each side
Usable: 1840px

Sections:
  binary_bg:  x=460  y=68   w=720  h=240
  pt01_bg:    x=30   y=360  w=560  h=290
  pt08_bg:    x=620  y=360  w=1060 h=380
  core_bg:    x=30   y=790  w=1650 h=220
  rocksdb:    x=780  y=1060 w=150  h=90
```

**Vertical gap between sections**: minimum 10px. Use 20px for breathing room.

### Place children INSIDE parent bounds

Child items must be within `parent.x + padding` to `parent.x + parent.w - padding`.

```
parent: x=30, w=560  → children span x=46 to x=574
child1: x=46, w=175  → ends at 221 ✓
child2: x=235, w=175 → ends at 410 ✓
child3: x=424, w=150 → ends at 574 ✓
```

### Even distribution formula for N items across width W

```
usable = W - (2 x padding)             # e.g. 1650 - 60 = 1590
gap = 12
item_width = (usable - (N-1) x gap) / N  # e.g. (1590 - 60) / 6 = 255
x[i] = left_padding + i x (item_width + gap)
```

### Vertical rhythm

Maintain a consistent vertical rhythm grid. A 10px grid unit keeps all y-coordinates multiples of 10, making overlaps immediately obvious during review.

```
Section Y positions on a 10px grid:
  y=60   (first section top)
  y=360  (gap=60 from section bottom, divisible by 10)
  y=790  (gap=140 from section bottom)
  y=1060 (external dependency)
```

**Rule**: Use a 10px grid for all x/y values. Non-grid coordinates signal a layout mistake.

---

## Grouping (Background Boxes)

### Layer order matters: background FIRST, then children

Draw.io renders in document order. Put the background `mxCell` before any child cells.
Children do NOT need `parent=` pointing to the background — just ensure coordinates overlap.

### Use `opacity=20` on backgrounds, `strokeWidth=2` on borders

```xml
style="rounded=1;whiteSpace=wrap;html=1;
       fillColor=#dae8fc;strokeColor=#6c8ebf;
       opacity=20;strokeWidth=2;"
```

`opacity=20` = 20% opaque fill → children are readable on top.

### Title labels: separate `text` cell, positioned at top-left of group

```xml
<mxCell id="group_title" value="crate-name"
  style="text;html=1;strokeColor=none;fillColor=none;
         align=left;fontStyle=1;fontSize=13;fontColor=#006EAF;"
  vertex="1" parent="1">
  <mxGeometry x="GROUP_X+12" y="GROUP_Y+8" width="400" height="22" />
</mxCell>
```

### Nested groups: increase stroke contrast at each level

When groups are nested (e.g. a module inside a crate), reduce opacity further and lighten stroke for the inner group:

```xml
<!-- Outer group -->
<mxCell style="...fillColor=#dae8fc;strokeColor=#6c8ebf;opacity=20;strokeWidth=2;" .../>

<!-- Inner subgroup -->
<mxCell style="...fillColor=#dae8fc;strokeColor=#a0b8d0;opacity=10;strokeWidth=1;dashed=1;" .../>
```

**Rule**: Each nesting level = opacity / 2 and strokeWidth - 1. Maximum 3 nesting levels before the diagram becomes unreadable.

---

## Arrows / Edges

### Always specify both exit and entry points

```xml
style="edgeStyle=orthogonalEdgeStyle;html=1;
       exitX=0;exitY=1;exitDx=0;exitDy=0;
       entryX=0.5;entryY=0;entryDx=0;entryDy=0;
       strokeColor=#1a7a1a;strokeWidth=2;"
```

Without explicit exit/entry, draw.io auto-routes and lines often cross each other.

### Exit/entry values

| Value | Meaning |
|-------|---------|
| `exitX=0` | left side |
| `exitX=1` | right side |
| `exitX=0.5` | center |
| `exitY=0` | top |
| `exitY=1` | bottom |
| `exitY=0.5` | middle |

### Use `source=` and `target=` pointing to cell IDs, not coordinates

```xml
<mxCell id="arr1" edge="1" source="fn_run_pt01" target="pt01_bg" parent="1">
  <mxGeometry relative="1" as="geometry" />
</mxCell>
```

Draw.io auto-adjusts endpoints when you move boxes if you use IDs.

### `dashed=1` for optional/indirect relationships

```xml
style="...dashed=1;..."  <!-- reads/writes, uses, optional deps -->
```

### Edge ordering: define ALL vertices before ANY edges

This is the most important structural rule for LLM-generated diagrams. Edges with `source=` or `target=` referencing IDs that appear later in the XML will create orphaned references that silently break the diagram.

```xml
<!-- CORRECT: all vertices first -->
<mxCell id="nodeA" vertex="1" ... />
<mxCell id="nodeB" vertex="1" ... />
<mxCell id="nodeC" vertex="1" ... />

<!-- Then all edges -->
<mxCell id="edgeAB" edge="1" source="nodeA" target="nodeB" ... />
<mxCell id="edgeBC" edge="1" source="nodeB" target="nodeC" ... />

<!-- WRONG: edge before target exists -->
<mxCell id="edgeAB" edge="1" source="nodeA" target="nodeB" ... />
<mxCell id="nodeA" vertex="1" ... />  <!-- nodeB does not exist yet -->
<mxCell id="nodeB" vertex="1" ... />
```

**Rule**: Vertices first, edges last. Always.

---

## Edge Routing — Advanced

### Orthogonal routing with waypoints in XML

Waypoints are encoded as `Array` children within the edge's `mxGeometry`. Each `mxPoint` inside the `Array` element defines a bend point:

```xml
<mxCell id="edge1" edge="1" source="srcBox" target="dstBox" parent="1"
  style="edgeStyle=orthogonalEdgeStyle;html=1;
         exitX=1;exitY=0.5;exitDx=0;exitDy=0;
         entryX=0;entryY=0.5;entryDx=0;entryDy=0;
         strokeColor=#666;strokeWidth=1.5;">
  <mxGeometry relative="1" as="geometry">
    <Array as="points">
      <mxPoint x="700" y="200" />
      <mxPoint x="700" y="450" />
    </Array>
  </mxGeometry>
</mxCell>
```

**Rule**: When two edges would cross, add intermediate `mxPoint` waypoints to route them around each other. Place the waypoint at the midpoint x or y of the bypass path.

### Choosing the right edge style

| `edgeStyle` value | Best for |
|---|---|
| `orthogonalEdgeStyle` | Hierarchical diagrams; clean right-angle turns |
| `elbowEdgeStyle` | Simple two-segment bends; less configurable |
| `entityRelationEdgeStyle` | ER diagrams; crow's foot notation |
| `segmentEdgeStyle` | Manual segment control; most flexible |
| `none` (default) | Straight lines; only for sparse diagrams |

**Rule**: Use `orthogonalEdgeStyle` by default for all dependency/architecture diagrams. Switch to `segmentEdgeStyle` only when you need precise manual path control.

### Line jump style for unavoidable crossings

When two edges genuinely must cross (e.g. crossing dependency directions), use jump arcs to signal intentionality:

```xml
<!-- Set on the mxGraphModel or per-edge via style -->
style="edgeStyle=orthogonalEdgeStyle;html=1;
       jumpStyle=arc;jumpSize=10;
       strokeColor=#999;strokeWidth=1;"
```

Valid `jumpStyle` values: `none`, `arc`, `gap`, `sharp`.

### Bundling parallel edges

When multiple edges run between the same two groups, offset their entry/exit points to avoid visual stacking:

```xml
<!-- Edge 1: enters at 30% from top -->
style="...entryX=0;entryY=0.3;entryDx=0;entryDy=0;"

<!-- Edge 2: enters at 50% from top -->
style="...entryX=0;entryY=0.5;entryDx=0;entryDy=0;"

<!-- Edge 3: enters at 70% from top -->
style="...entryX=0;entryY=0.7;entryDx=0;entryDy=0;"
```

**Formula**: For N parallel edges entering the same face, `entryY = (i + 1) / (N + 1)` for `i` in `0..N-1`.

---

## Color Palette (Semantic Coding)

| Crate / Role | Fill | Stroke | Title color |
|---|---|---|---|
| Binary / entry | `#dae8fc` | `#6c8ebf` | `#006EAF` |
| Ingestion (pt01) | `#d5e8d4` | `#82b366` | `#1a7a1a` |
| HTTP server (pt08) | `#fff2cc` | `#d6b656` | `#b85450` |
| Core / shared lib | `#e1d5e7` | `#9673a6` | `#6a1aaf` |
| Error types | `#f8cecc` | `#b85450` | — |
| External clients | `#dae8fc` | `#6c8ebf` | — |
| Test fixtures | `#f5f5f5` | `#aaa` | — |
| Database | `#f5f5f5` | `#555` | — |

---

## Color Theory for Diagrams

### WCAG contrast requirements

WCAG 2.1 requires a contrast ratio of at least 4.5:1 for text against background. For large text (18pt+ or 14pt bold), the minimum is 3:1. For AAA compliance (government/enterprise), normal text needs 7:1.

**Practical rule for draw.io**: Always pair a dark stroke/font color against a light fill. The existing parseltongue palette already satisfies this — for example `#006EAF` (dark blue) on `#dae8fc` (light blue) achieves approximately 4.9:1.

### Colorblind-safe combinations

Approximately 8% of men and 0.5% of women have color vision deficiency. The most common form is red-green blindness. Blue is perceived correctly by nearly all color vision types.

| Pair | Safe? | Note |
|------|-------|------|
| Blue / Orange | Yes | Maximum distinction for red-green blind |
| Blue / Red | Acceptable | Avoid relying solely on hue; add shape/stroke differences |
| Red / Green | Avoid | Indistinguishable for deuteranopia |
| Purple / Blue | Caution | Similar luminance; add pattern or label |

**Rule**: Never use color as the ONLY differentiator. Pair color with: (a) different stroke patterns (solid vs dashed), (b) different shapes, (c) explicit text labels. This satisfies WCAG 1.4.1 (use of color).

### Colorblind-safe palette for draw.io

```
Blue group    fill=#dae8fc  stroke=#6c8ebf  title=#1f4e79
Orange group  fill=#ffe6cc  stroke=#d79b00  title=#7d4000
Teal group    fill=#d5e8d4  stroke=#82b366  title=#1f4e2a
Purple group  fill=#e1d5e7  stroke=#9673a6  title=#4a1673
Grey group    fill=#f5f5f5  stroke=#666666  title=#333333
```

All five colors are distinguishable under deuteranopia and protanopia simulation.

### Opacity guidelines

| Context | opacity value | Effect |
|---------|--------------|--------|
| Background section | 15–20 | Tinted wash, children visible |
| Highlighted selection | 40 | Noticeable emphasis |
| Disabled / inactive | 50 | Visually receded |
| Full fill (buttons, badges) | 100 | Solid opaque |

### Gradient use

Avoid gradients in architecture diagrams. Gradients add cognitive load without semantic meaning. Exception: use a subtle gradient on swimlane headers to distinguish header from body.

```xml
<!-- Acceptable gradient for swimlane header only -->
style="swimlane;fillColor=#dae8fc;gradientColor=#b8d4f0;
       strokeColor=#6c8ebf;fontStyle=1;fontSize=13;"
```

---

## Layout Algorithm Selection

### Manual vs automatic layout

Draw.io supports both manual coordinate placement (what this guide primarily covers) and automatic layout via built-in algorithms. Understanding when to use each saves hours of geometry planning.

| Approach | When to use | Tradeoffs |
|----------|-------------|-----------|
| Manual coordinates | Final-quality diagrams, known structure | Full control; brittle to changes |
| Automatic layout | Exploration, unknown structure, >50 nodes | Fast; less predictable |
| Hybrid | Large diagrams with known groupings | Use auto for internals, manual for sections |

### Draw.io built-in layout algorithms

Apply via Arrange > Layout in the GUI, or via `mxGraphModel` attributes:

| Algorithm | `layout` value | Best for |
|-----------|---------------|----------|
| Hierarchical (layered) | `hierarchical` | Dependency trees, call graphs |
| Organic (force-directed) | `organic` | Cluster/community visualization |
| Tree | `tree` | Strict parent-child hierarchies |
| Radial | `radial` | Hub-spoke architectures |
| Circle | `circle` | Peer relationship networks |
| Grid | `grid` | Uniform entity catalogs |

### Dagre vs ELK comparison

External layout libraries used by tools that wrap draw.io:

**Dagre** (JavaScript, client-side):
- Best for directed graphs and dependency trees
- Fast, minimal configuration
- Produces clean layered layouts
- Limitation: limited orthogonal routing options

**ELK** (Eclipse Layout Kernel, ported to JS):
- Full orthogonal routing with port constraints
- Handles block diagrams and circuit schematics
- Significantly more configuration options
- Slower; more complex setup
- Use when Dagre produces too many edge crossings

**Rule**: For software dependency graphs with 10–80 nodes, manual layout using the hierarchical approach described in this document produces cleaner results than either Dagre or ELK. For >100 nodes, switch to ELK layered with `org.eclipse.elk.layered` algorithm.

### Direction conventions

| Flow direction | `mxGraphModel` setting | Example |
|----------------|------------------------|---------|
| Top to bottom | `graph.getModel()` default | Dependency layers |
| Left to right | Set edge `exitX=1` / `entryX=0` patterns | Pipelines, data flow |
| Bottom to top | `exitY=0` / `entryY=1` patterns | Inheritance hierarchies |

**Rule**: Top-to-bottom for dependency/layer diagrams. Left-to-right for pipeline/sequence diagrams. Never mix directions in the same diagram.

---

## Cognitive Load Reduction

### Gestalt principles applied to architecture diagrams

Research demonstrates that applying Gestalt perceptual principles to diagrams directly reduces cognitive load and speeds comprehension.

**Proximity**: Group related entities close together. The brain perceives nearby items as belonging to the same logical unit. This is why section background boxes work — they reinforce the proximity grouping visually.

```
Rule: Items in the same module should be within 20px of each other.
      Items in different modules should be separated by at least 40px.
```

**Similarity**: Use the same fill color and shape for items of the same type. The brain immediately perceives color-matched items as a category without reading labels.

```
Rule: All crates of the same role = same fill color family.
      Never use more than 6 distinct fill colors in one diagram.
```

**Closure**: The background boxes use closure — the viewer's brain fills in the implied boundary. A semi-transparent background box reads as a container even without an explicit border.

**Continuity**: Minimize line crossings and bends. The eye naturally follows a smooth path. Each additional bend or crossing increases the cognitive cost of tracing an edge.

```
Rule: Maximum 2 bends per edge. More than 2 bends = reorganize the layout.
```

**Figure/Ground**: The background (opacity=20 fill) recedes visually while the foreground entities advance. This hierarchy is what makes background section boxes work without obscuring children.

### Chunking strategy

Miller's Law: humans process approximately 7 (+/- 2) items at a time. Apply this to diagram density.

```
Rule: Maximum 7 entities visible per section background box.
      If a section has more, split it into sub-sections or collapse detail.
```

For parseltongue-scale diagrams:
- Binary crate: 3–5 function nodes (within limit)
- pt01 crate: 4–6 nodes (within limit)
- pt08 crate: can have 8–12 handler nodes → use sub-grouping

### Visual hierarchy levels (Z-order pyramid)

```
Level 1 (highest attention):  Thick borders, bold text, bright fill
Level 2 (medium attention):   Standard borders, normal text, tinted fill
Level 3 (background context): Thin borders, small text, 15-20% opacity fill
Level 4 (annotations):        No border, italic text, no fill
```

Map to content:
```
Level 1 = Entry points, key interfaces
Level 2 = Core business logic entities
Level 3 = Section/group containers
Level 4 = Legend, notes, edge labels
```

### Information density guidelines

| Diagram size | Node count | Recommended approach |
|---|---|---|
| Micro | 1–15 | Single canvas, full label detail |
| Small | 16–40 | Single canvas, abbreviate long names |
| Medium | 41–100 | Multiple linked diagrams, or sections with drill-down |
| Large | 100+ | Overview diagram + detail diagrams per module |

**Rule**: If a printed A4 page of the diagram requires a magnifying glass to read labels, the diagram has too many nodes. Split it.

---

## Style Reference Sheet

### Complete mxCell vertex style string anatomy

```
shape;property1=value1;property2=value2;...
```

The style string is a semicolon-delimited list of key=value pairs. The first token (before any `=`) may be a named style preset.

### Fill and stroke properties

| Property | Values | Default | Notes |
|----------|--------|---------|-------|
| `fillColor` | `#RRGGBB`, `none`, `inherit` | `#ffffff` | Background fill |
| `strokeColor` | `#RRGGBB`, `none`, `inherit` | `#000000` | Border color |
| `strokeWidth` | number (px) | `1` | Border thickness |
| `dashed` | `0`, `1` | `0` | Dashed border |
| `dashPattern` | e.g. `8 4` | — | Custom dash lengths |
| `opacity` | `0`–`100` | `100` | Fill+stroke opacity |
| `fillOpacity` | `0`–`100` | `100` | Fill only opacity |
| `strokeOpacity` | `0`–`100` | `100` | Stroke only opacity |
| `gradientColor` | `#RRGGBB`, `none` | `none` | Gradient end color |
| `gradientDirection` | `north`,`south`,`east`,`west` | `south` | Gradient direction |

### Shape properties

| Property | Values | Default | Notes |
|----------|--------|---------|-------|
| `shape` | `rectangle`,`ellipse`,`rhombus`,`triangle`,`cylinder`,`cloud`,`hexagon`,`actor`,`swimlane` | `rectangle` | Shape type |
| `rounded` | `0`, `1` | `0` | Rounded rectangle corners |
| `arcSize` | number (%) | `10` | Corner rounding radius % |
| `aspect` | `fixed`, `variable` | `variable` | Maintain aspect ratio |
| `perimeter` | `ellipsePerimeter`,`rectanglePerimeter` | auto | Connection point geometry |
| `shadow` | `0`, `1` | `0` | Drop shadow |
| `sketch` | `0`, `1` | `0` | Hand-drawn look |

### Font and text properties

| Property | Values | Default | Notes |
|----------|--------|---------|-------|
| `fontSize` | number (pt) | `11` | Label font size |
| `fontColor` | `#RRGGBB` | `#000000` | Label color |
| `fontFamily` | font name string | `Helvetica` | Font family |
| `fontStyle` | bitmask: `0`=none, `1`=bold, `2`=italic, `4`=underline, `8`=strikethrough | `0` | Combine: `3`=bold+italic |
| `align` | `left`, `center`, `right` | `center` | Horizontal text alignment |
| `verticalAlign` | `top`, `middle`, `bottom` | `middle` | Vertical text alignment |
| `labelPosition` | `left`, `center`, `right` | `center` | Label outside shape |
| `verticalLabelPosition` | `top`, `middle`, `bottom` | `middle` | Label above/below shape |
| `whiteSpace` | `wrap`, `nowrap` | `nowrap` | Text wrapping |
| `html` | `0`, `1` | `0` | Enable HTML in label |
| `overflow` | `fill`, `width`, `visible`, `hidden` | `visible` | Overflow behavior |
| `spacingTop`, `spacingBottom`, `spacingLeft`, `spacingRight` | number (px) | `0` | Label padding |

### Edge-specific properties

| Property | Values | Default | Notes |
|----------|--------|---------|-------|
| `edgeStyle` | `orthogonalEdgeStyle`, `elbowEdgeStyle`, `entityRelationEdgeStyle`, `segmentEdgeStyle`, `none` | `none` | Routing algorithm |
| `exitX`, `exitY` | `0`–`1` | auto | Source connection point |
| `exitDx`, `exitDy` | number (px) | `0` | Source connection offset |
| `entryX`, `entryY` | `0`–`1` | auto | Target connection point |
| `entryDx`, `entryDy` | number (px) | `0` | Target connection offset |
| `startArrow` | `none`, `classic`, `block`, `open`, `oval`, `diamond` | `none` | Source arrowhead |
| `endArrow` | `none`, `classic`, `block`, `open`, `oval`, `diamond` | `classic` | Target arrowhead |
| `startFill` | `0`, `1` | `1` | Filled source arrowhead |
| `endFill` | `0`, `1` | `1` | Filled target arrowhead |
| `curved` | `0`, `1` | `0` | Curved edge segments |
| `jumpStyle` | `none`, `arc`, `gap`, `sharp` | `none` | Crossing jump style |
| `jumpSize` | number (px) | `6` | Jump arc/gap size |
| `jettySize` | number or `auto` | `auto` | Connector segment margin |

### Text cell quick template

```xml
<mxCell id="lbl1" value="My Label"
  style="text;html=1;strokeColor=none;fillColor=none;
         align=left;verticalAlign=top;
         fontStyle=1;fontSize=12;fontColor=#333333;"
  vertex="1" parent="1">
  <mxGeometry x="100" y="100" width="200" height="24" as="geometry" />
</mxCell>
```

### fontStyle bitmask combinations

| fontStyle value | Effect |
|----------------|--------|
| `0` | Normal |
| `1` | Bold |
| `2` | Italic |
| `3` | Bold + Italic |
| `4` | Underline |
| `5` | Bold + Underline |
| `6` | Italic + Underline |
| `7` | Bold + Italic + Underline |
| `8` | Strikethrough |

---

## LLM Prompting Strategy

### System prompt structure for draw.io generation

Research from GenAI-DrawIO-Creator (arXiv 2601.05162) and the drawio-ninja project converges on the following system prompt structure achieving 90%+ valid XML output:

```
You are a draw.io XML diagram generator. Always output valid mxGraph XML.

STRUCTURAL RULES (never violate):
1. Begin with <mxfile><diagram><mxGraphModel><root>
2. Always include id="0" and id="1" as the first two mxCell elements
3. Generate ALL vertex mxCell elements BEFORE any edge mxCell elements
4. All edge source= and target= attributes must reference IDs defined earlier
5. Use sequential non-reused IDs starting from "2"
6. Close every XML tag. No self-closing tags on mxCell (use explicit close or />)
7. Escape: & -> &amp;  < -> &lt;  > -> &gt;  " -> &quot; (inside attribute values)

LAYOUT RULES:
8. Grid: all x/y coordinates are multiples of 10
9. Background groups: opacity=20, strokeWidth=2
10. Vertices before edges; groups before their children
11. whiteSpace=wrap;html=1; on all vertex cells
12. Calculate width = ceil(char_count * 7) + 20 for each label
```

### Chain-of-thought layout planning prompt

Before asking an LLM to generate XML, ask it to plan the layout in a separate step:

```
Step 1 — Planning prompt:
"Before writing any XML, describe the layout plan:
- What are the top-level sections?
- What is the canvas width and height?
- What are the x, y, w, h of each section?
- What are the entities inside each section?
- What are the edges between entities?
Output a structured list. Do not write XML yet."

Step 2 — Generation prompt:
"Now convert the layout plan you just created into draw.io XML.
Follow the structural rules. Output only the XML, no explanation."
```

This two-step approach separates spatial reasoning (step 1) from XML syntax generation (step 2), significantly reducing structural errors.

### Iterative correction prompt

When the LLM produces invalid XML, use this correction template:

```
"The XML you generated has this problem: [SPECIFIC ISSUE].
Fix only this issue. Do not change anything else.
The correct behavior is: [EXPECTED BEHAVIOR].
Output the corrected XML."
```

Always specify the exact problem. "Fix the diagram" produces unpredictable changes. "The edge with id='e3' has source='fnA' but fnA is not defined as a vertex id — change source to 'fn_run_pt01'" produces targeted corrections.

### Prompting for specific diagram types

**Dependency graph**:
```
"Generate a draw.io XML dependency graph showing:
- Crates: [list with roles]
- Dependencies: [list of A -> B pairs]
- Layout: top (binary) -> middle (tools) -> bottom (shared core)
- Color code: blue=binary, green=tools, purple=core
Apply the vertex-before-edge rule."
```

**Data flow diagram**:
```
"Generate a draw.io XML left-to-right data flow diagram.
Flow direction: left to right (exitX=1, entryX=0 on all edges).
Nodes: [list]
Flows: [list of source -> destination with label]
Use rectangles for processes, cylinders for data stores, parallelograms for I/O."
```

**Sequence diagram alternative** (draw.io swimlane):
```
"Generate a draw.io XML swimlane diagram.
Each actor is a swimlane column.
Messages are horizontal edges between swimlanes.
Time flows top to bottom."
```

### Token budget management

For large diagrams (>30 nodes), LLMs may truncate output or lose track of IDs. Use this strategy:

1. Generate sections independently in separate LLM calls
2. Use a consistent ID prefix per section (e.g. `pt01_`, `pt08_`, `core_`)
3. Merge sections manually by concatenating their cell lists
4. Generate edges in a final separate call with the full ID map

```xml
<!-- Section 1 IDs: pt01_bg, pt01_title, pt01_fn1, pt01_fn2 -->
<!-- Section 2 IDs: pt08_bg, pt08_title, pt08_fn1 -->
<!-- Edge call: "Generate edges connecting pt01_fn1 -> pt08_bg, etc." -->
```

---

## Iterative Refinement Workflow

### The five-phase loop

```
Phase 1: PLAN
  - Enumerate all entities and their roles
  - Sketch section boundaries on paper or in text
  - Calculate canvas dimensions
  - Assign ID prefixes per section

Phase 2: GENERATE
  - Write or prompt XML section by section
  - Vertices first (backgrounds, titles, entities)
  - Edges last

Phase 3: VALIDATE (structural)
  - Check: id="0" and id="1" present
  - Check: no duplicate IDs
  - Check: all edge source/target IDs exist as vertices
  - Check: all children within parent bounds
  - Run: mcp__drawio__open_drawio_xml

Phase 4: VALIDATE (visual)
  - Check: text overflow (zoom in to each box)
  - Check: edge crossings (can you trace each edge without confusion?)
  - Check: color consistency (same role = same color family)
  - Check: spacing uniformity (gaps between rows/columns)

Phase 5: REFINE
  - Fix one category of issue at a time
  - Text overflow → width/height adjustments
  - Edge crossings → add waypoints or flip exit/entry sides
  - Visual clutter → increase gaps, reduce font size one tier
  - Re-open after each fix
```

### Structural validation checklist (pre-commit)

```
[ ] id="0" and id="1" present as first two mxCell elements
[ ] No duplicate id= values across all mxCell elements
[ ] All edge source= values match a vertex id=
[ ] All edge target= values match a vertex id=
[ ] All mxGeometry elements have as="geometry" attribute
[ ] All vertex mxCell have vertex="1" attribute
[ ] All edge mxCell have edge="1" attribute
[ ] No mxCell has both vertex="1" and edge="1"
[ ] All x/y coordinates are multiples of 10
[ ] No child box extends beyond its parent section boundary
[ ] whiteSpace=wrap;html=1; present on all content boxes
```

### Diffing iterations

When editing a diagram across sessions, track changes by comparing the cell count and edge count:

```bash
# Count vertices and edges in a .drawio file
grep -c 'vertex="1"' diagram.drawio
grep -c 'edge="1"' diagram.drawio

# List all IDs to spot duplicates
grep -o 'id="[^"]*"' diagram.drawio | sort | uniq -d
```

**Rule**: A new version of a diagram must have >= the edge count of the previous version unless nodes were intentionally removed.

### Human review checkpoints

| Checkpoint | Reviewer question | Pass criteria |
|---|---|---|
| Label readability | Can I read every label without zooming? | Yes on a 1920x1080 screen at 100% zoom |
| Edge traceability | Can I trace every edge from source to target? | No edge requires moving other edges to see |
| Color semantics | Can I tell crate roles from color alone? | Yes, without reading labels |
| Section completeness | Does each section contain all its entities? | No entity is floating outside a section |
| Legend accuracy | Does the legend match what's in the diagram? | All colors/styles in diagram are in legend |

---

## Anti-patterns Catalogue

### AP-1: Missing foundation cells

**Symptom**: draw.io refuses to open the file, or opens a blank diagram.

**Bad XML**:
```xml
<mxGraphModel>
  <root>
    <mxCell id="bg1" vertex="1" ... />
  </root>
</mxGraphModel>
```

**Good XML**:
```xml
<mxGraphModel>
  <root>
    <mxCell id="0" />
    <mxCell id="1" parent="0" />
    <mxCell id="bg1" vertex="1" parent="1" ... />
  </root>
</mxGraphModel>
```

### AP-2: Edge defined before its target vertex

**Symptom**: Edge appears disconnected; clicking it shows no source/target highlight.

**Bad XML**:
```xml
<mxCell id="edge1" edge="1" source="nodeA" target="nodeB" parent="1" ... />
<mxCell id="nodeA" vertex="1" parent="1" ... />
<mxCell id="nodeB" vertex="1" parent="1" ... />
```

**Good XML**:
```xml
<mxCell id="nodeA" vertex="1" parent="1" ... />
<mxCell id="nodeB" vertex="1" parent="1" ... />
<mxCell id="edge1" edge="1" source="nodeA" target="nodeB" parent="1" ... />
```

### AP-3: Text overflow due to undersized box

**Symptom**: Label text visually spills outside the box border; box appears to have no label in exports.

**Bad XML**:
```xml
<mxCell value="run_folder_to_cozodb_streamer"
  style="rounded=1;html=1;"
  vertex="1" parent="1">
  <mxGeometry x="50" y="100" width="120" height="30" as="geometry" />
</mxCell>
```

**Good XML**:
```xml
<mxCell value="run_folder_to&#xa;cozodb_streamer"
  style="rounded=1;html=1;whiteSpace=wrap;fontSize=10;"
  vertex="1" parent="1">
  <mxGeometry x="50" y="100" width="200" height="50" as="geometry" />
</mxCell>
```

### AP-4: Child positioned outside parent group bounds

**Symptom**: Entity floats visually disconnected from its section box; may overlap adjacent sections.

**Bad XML**:
```xml
<!-- Parent: x=30, y=360, w=560, h=290 — right edge at x=590 -->
<mxCell id="child1" vertex="1" parent="1">
  <!-- x=600 places child OUTSIDE the parent group -->
  <mxGeometry x="600" y="400" width="120" height="40" as="geometry" />
</mxCell>
```

**Good XML**:
```xml
<!-- Parent: x=30, y=360, w=560, h=290 — usable x range: 46..574 -->
<mxCell id="child1" vertex="1" parent="1">
  <mxGeometry x="424" y="400" width="150" height="40" as="geometry" />
</mxCell>
```

### AP-5: Duplicate cell IDs

**Symptom**: One of the duplicate cells is invisible; clicking empty space selects a ghost element.

**Bad XML**:
```xml
<mxCell id="fn1" value="function_a" vertex="1" ... />
<mxCell id="fn1" value="function_b" vertex="1" ... />  <!-- ID reused! -->
```

**Good XML**:
```xml
<mxCell id="fn1" value="function_a" vertex="1" ... />
<mxCell id="fn2" value="function_b" vertex="1" ... />
```

**Rule**: Use a prefix strategy: `binary_fn1`, `pt01_fn1`, `pt08_fn1`. Prefixed IDs make duplicates obvious at a glance.

### AP-6: All edges same color and weight

**Symptom**: Reader cannot distinguish dependency types; the diagram feels like a plate of spaghetti.

**Bad XML**:
```xml
<mxCell edge="1" style="strokeColor=#000000;strokeWidth=1;" ... />
<mxCell edge="1" style="strokeColor=#000000;strokeWidth=1;" ... />
<mxCell edge="1" style="strokeColor=#000000;strokeWidth=1;" ... />
```

**Good XML**:
```xml
<!-- Direct compile dependency: thick, solid, dark -->
<mxCell edge="1" style="strokeColor=#1a1a1a;strokeWidth=2.5;" ... />

<!-- Feature flag / optional: thin, dashed, grey -->
<mxCell edge="1" style="strokeColor=#888888;strokeWidth=1;dashed=1;" ... />

<!-- Writes to (data flow): medium, colored -->
<mxCell edge="1" style="strokeColor=#d79b00;strokeWidth=1.5;" ... />
```

### AP-7: No legend

**Symptom**: Colors and edge styles are arbitrary to a reader who did not build the diagram.

**Good XML** (legend section template):
```xml
<!-- LEGEND section — bottom-right corner -->
<mxCell id="legend_bg" value="" vertex="1" parent="1"
  style="rounded=1;fillColor=#fafafa;strokeColor=#cccccc;opacity=100;strokeWidth=1;">
  <mxGeometry x="1600" y="1100" width="250" height="180" as="geometry" />
</mxCell>
<mxCell id="legend_title" value="Legend" vertex="1" parent="1"
  style="text;html=1;strokeColor=none;fillColor=none;fontStyle=1;fontSize=12;align=left;">
  <mxGeometry x="1612" y="1108" width="100" height="20" as="geometry" />
</mxCell>
<!-- One row per color/style -->
<mxCell id="legend_blue" value="Binary / Entry point" vertex="1" parent="1"
  style="rounded=1;fillColor=#dae8fc;strokeColor=#6c8ebf;fontSize=10;html=1;">
  <mxGeometry x="1612" y="1135" width="220" height="24" as="geometry" />
</mxCell>
```

### AP-8: Opacity=100 on section background

**Symptom**: Background box completely covers child entities; diagram appears to have only colored rectangles.

**Bad XML**:
```xml
style="fillColor=#dae8fc;strokeColor=#6c8ebf;opacity=100;"
```

**Good XML**:
```xml
style="fillColor=#dae8fc;strokeColor=#6c8ebf;opacity=20;"
```

### AP-9: Using file path instead of XML string in MCP tool

**Symptom**: `open_drawio_xml` renders a blank diagram or throws a parse error.

```
# Bad: passing a file path
mcp__drawio__open_drawio_xml(content="/path/to/diagram.drawio")

# Good: passing the raw XML string
mcp__drawio__open_drawio_xml(content="<mxfile>...</mxfile>")
```

If the XML is stored in a file, read it first: `content=$(cat diagram.drawio)`, then pass `$content`.

### AP-10: Swimlane used as generic container

**Symptom**: Swimlane renders with an embedded header that wastes 30px of height and adds an unwanted double border.

Use `swimlane` shape only for actual swimlane diagrams (actor-based sequence-like flows). For general section containers, use `rounded=1` rectangle with `opacity=20`.

---

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Text overflows box | Increase width, add `&#xa;` line breaks, set `whiteSpace=wrap;html=1;` |
| Boxes overlap | Compute all x/y positions explicitly before writing XML |
| Arrow crosses other arrows | Use explicit `exitX/Y` and `entryX/Y`; add waypoints |
| Background hides children | Set `opacity=20` on background, keep child colors same hue but solid |
| Long function names in small boxes | Two-line label with `&#xa;`, height >= 50 |
| Uniform handler boxes look bad | Keep width uniform per group, vary only when text demands it |
| File path passed to `open_drawio_xml` | Pass the raw XML string, not the file path |
| Children outside parent bounds | Calculate `parent.x + padding` to `parent.x + w - padding` explicitly |
| Stat labels cut off | Give `text` cells enough width even if they look short (use 80-100px min) |
| id="0" or id="1" missing | Always include foundation cells as first two mxCell elements |
| Edges before vertices | Define all vertex mxCell elements before any edge mxCell elements |
| Duplicate IDs | Use section-prefixed IDs: `pt01_fn1`, `pt08_fn1`, not `fn1` for both |
| Red/green only differentiation | Add dashed vs solid, or shape differences, alongside color |
| More than 7 entities per section | Split section into sub-groups or collapse to summary node |
| No legend | Always add a legend section for color/style semantics |

---

## Workflow

```
1. Plan layout on paper (sections, widths, heights, gaps)
2. Write XML section by section: title -> groups top-to-bottom -> edges last
3. Call mcp__drawio__open_drawio_xml with raw XML string
4. Iterate: identify overflow/overlap visually -> fix geometry -> reopen
5. Save final XML to .drawio file via Write tool
```

### Section template

```xml
<!-- GROUP: my-crate  x=X y=Y w=W h=H -->
<mxCell id="my_bg" value=""
  style="rounded=1;whiteSpace=wrap;html=1;fillColor=FILL;strokeColor=STROKE;opacity=20;strokeWidth=2;"
  vertex="1" parent="1">
  <mxGeometry x="X" y="Y" width="W" height="H" as="geometry" />
</mxCell>
<mxCell id="my_title" value="my-crate-name"
  style="text;html=1;strokeColor=none;fillColor=none;align=left;fontStyle=1;fontSize=13;fontColor=COLOR;"
  vertex="1" parent="1">
  <mxGeometry x="X+12" y="Y+8" width="400" height="22" as="geometry" />
</mxCell>
<!-- children here, all within X+16 to X+W-16 -->
```

### Full diagram template (ready to fill in)

```xml
<mxfile>
  <diagram name="Architecture" id="arch-1">
    <mxGraphModel dx="1422" dy="762" grid="1" gridSize="10"
                  guides="1" tooltips="1" connect="1" arrows="1"
                  fold="1" page="0" pageScale="1" math="0" shadow="0">
      <root>
        <mxCell id="0" />
        <mxCell id="1" parent="0" />

        <!-- ========== SECTION: binary crate ========== -->
        <mxCell id="binary_bg" value="" vertex="1" parent="1"
          style="rounded=1;fillColor=#dae8fc;strokeColor=#6c8ebf;opacity=20;strokeWidth=2;">
          <mxGeometry x="460" y="60" width="720" height="240" as="geometry" />
        </mxCell>
        <mxCell id="binary_title" value="parseltongue (binary)" vertex="1" parent="1"
          style="text;html=1;strokeColor=none;fillColor=none;align=left;fontStyle=1;fontSize=13;fontColor=#006EAF;">
          <mxGeometry x="472" y="68" width="400" height="22" as="geometry" />
        </mxCell>

        <!-- Add vertex mxCell elements here -->

        <!-- ========== EDGES (always last) ========== -->

        <!-- Add edge mxCell elements here -->

      </root>
    </mxGraphModel>
  </diagram>
</mxfile>
```

---

## File Formats

`.drawio` files are raw XML — commit them to git and GitHub renders them inline.
Export to `.png` or `.svg` for README embeds or IDE display.

```bash
# Supported by: GitHub, GitLab, VS Code (Draw.io Integration ext),
#               JetBrains IDEs (Draw.io Integration plugin)
```

### Version control tips for .drawio files

Because `.drawio` files are plain XML, standard `git diff` shows structural changes. To make diffs more readable:

```bash
# .gitattributes — tell git to treat .drawio as XML
*.drawio diff=xml

# Configure git XML diff driver
git config diff.xml.textconv "xmllint --format"
```

This pretty-prints the XML before diffing, making geometry changes visible as clean line diffs rather than one-line noise.

### Export strategy

| Target | Format | Command / Method |
|--------|--------|-----------------|
| README embed | PNG @2x | Export > PNG, Scale 200% |
| Web documentation | SVG | Export > SVG, no border |
| Print / PDF | PDF | File > Print, PDF export |
| GitHub inline | .drawio | Commit raw file; GitHub auto-renders |
| Confluence | PNG or SVG | Export and attach |

---

## Quick Reference Card

```
MANDATORY STRUCTURE:
  id="0" (root) + id="1" parent="0" (layer) → always first two cells

SIZING:
  width  = ceil(chars × 7) + 20
  height = ceil(lines × fontSize × 1.5) + 16
  grid   = all x/y multiples of 10

STYLE ESSENTIALS:
  All boxes:   whiteSpace=wrap;html=1;
  Backgrounds: opacity=20;strokeWidth=2;
  Titles:      text;html=1;strokeColor=none;fillColor=none;fontStyle=1;

EDGES:
  Always: source=ID target=ID  (never coordinates)
  Always: exitX/Y + entryX/Y explicit
  Always: vertices before edges in XML
  Parallel edges: entryY = (i+1)/(N+1)

COLORS:
  Blue   = binary/entry    fill=#dae8fc stroke=#6c8ebf
  Green  = ingestion        fill=#d5e8d4 stroke=#82b366
  Yellow = HTTP server      fill=#fff2cc stroke=#d6b656
  Purple = core lib         fill=#e1d5e7 stroke=#9673a6
  Red    = error types      fill=#f8cecc stroke=#b85450

COGNITIVE LOAD:
  Max 7 entities per section
  Max 2 bends per edge
  Max 3 nesting levels
  Max 6 distinct fill colors per diagram
```

---

*Sources and research basis:*
- *GenAI-DrawIO-Creator framework: [arXiv 2601.05162](https://arxiv.org/abs/2601.05162)*
- *drawio-ninja LLM instruction project: [github.com/simonpo/drawio-ninja](https://github.com/simonpo/drawio-ninja)*
- *draw.io official connector documentation: [drawio.com/blog/use-connectors](https://www.drawio.com/blog/use-connectors)*
- *mxGraph constants reference: [jgraph.github.io/mxgraph/docs/js-api/files/util/mxConstants-js.html](https://jgraph.github.io/mxgraph/docs/js-api/files/util/mxConstants-js.html)*
- *IBM Architecture Visualization best practices: [ibm.github.io/itaa-docs/ArchVisualization.html](https://ibm.github.io/itaa-docs/ArchVisualization.html)*
- *WCAG contrast requirements: [webaim.org/articles/contrast](https://webaim.org/articles/contrast/)*
- *Gestalt principles and diagram comprehension: [dl.acm.org/doi/10.5555/2521076](https://dl.acm.org/doi/10.5555/2521076)*
- *ELK layered algorithm reference: [eclipse.dev/elk/reference/algorithms/org-eclipse-elk-layered.html](https://eclipse.dev/elk/reference/algorithms/org-eclipse-elk-layered.html)*
- *draw.io MCP npm package: [npmjs.com/package/@drawio/mcp](https://www.npmjs.com/package/@drawio/mcp)*
