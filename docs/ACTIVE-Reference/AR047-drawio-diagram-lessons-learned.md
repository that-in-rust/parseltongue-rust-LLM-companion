# Draw.io Diagram Lessons Learned

*From building the parseltongue v1.6.1 crate dependency graph via Claude Code + @drawio/mcp*

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

---

## Box Sizing Rules

### Rule 1: Measure text length before setting width

At `fontSize=11`, each character is ~7px wide. Safe formula:

```
width = ceil(char_count × 7) + 20px padding
```

| Text | Chars | Min width |
|------|-------|-----------|
| `fn main()` | 9 | 83 → use **120** |
| `fn build_cli()` | 14 | 118 → use **160** |
| `fn run_http_code_query_server()` | 31 | 237 → use **268** |
| `fn run_folder_to_cozodb_streamer()` | 34 | 258 → use **280** |
| `enum HttpServerErrorTypes` | 25 | 195 → use **240** |

### Rule 2: For multi-line text, use `&#xa;` and set height ≥ 50

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

### Rule 3: Always set `whiteSpace=wrap;html=1;` on every box

Without this, text overflows silently — the box stays the same size but text spills outside.

### Rule 4: Use consistent widths within a row

Pick one width for all items in a logical group (e.g. all handler boxes = 200px).
Mixed widths create visual noise even when text fits.

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
usable = W - (2 × padding)             # e.g. 1650 - 60 = 1590
gap = 12
item_width = (usable - (N-1) × gap) / N  # e.g. (1590 - 60) / 6 = 255
x[i] = left_padding + i × (item_width + gap)
```

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

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Text overflows box | Increase width, add `&#xa;` line breaks, set `whiteSpace=wrap;html=1;` |
| Boxes overlap | Compute all x/y positions explicitly before writing XML |
| Arrow crosses other arrows | Use explicit `exitX/Y` and `entryX/Y` |
| Background hides children | Set `opacity=20` on background, keep child colors same hue but solid |
| Long function names in small boxes | Two-line label with `&#xa;`, height ≥ 50 |
| Uniform handler boxes look bad | Keep width uniform per group, vary only when text demands it |
| File path passed to `open_drawio_xml` | Pass the raw XML string, not the file path |
| Children outside parent bounds | Calculate `parent.x + padding` to `parent.x + w - padding` explicitly |
| Stat labels cut off | Give `text` cells enough width even if they look short (use 80-100px min) |

---

## Workflow

```
1. Plan layout on paper (sections, widths, heights, gaps)
2. Write XML section by section: title → groups top-to-bottom → edges last
3. Call mcp__drawio__open_drawio_xml with raw XML string
4. Iterate: identify overflow/overlap visually → fix geometry → reopen
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

---

## File Formats

`.drawio` files are raw XML — commit them to git and GitHub renders them inline.
Export to `.png` or `.svg` for README embeds or IDE display.

```bash
# Supported by: GitHub, GitLab, VS Code (Draw.io Integration ext),
#               JetBrains IDEs (Draw.io Integration plugin)
```
