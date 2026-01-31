# Post-Ingestion Analytics & Terminal Visualization

**Document Version:** 1.0
**Date:** 2025-11-05
**Purpose:** Quick visual feedback after codebase ingestion using Unicode & emoji-based terminal graphics

---

## CRITICAL: FOUR-WORD NAMING CONVENTION

**ALL function names: EXACTLY 4 words** (underscores: `_`)
**ALL crate/folder/command names: EXACTLY 4 words** (hyphens: `-`)

Pattern: `verb_constraint_target_qualifier()`

Examples:
```rust
✅ render_entity_count_bar_chart()
✅ filter_implementation_entities_only()
✅ save_visualization_output_to_file()

❌ render_chart()           // Too short (2)
❌ render_entity_chart()    // Too short (3)
❌ render_entity_count_bar_chart_with_colors()  // Too long (6)
```

**Why**: 4 words = optimal LLM tokenization and semantic density.

Make this a ritual before every commit.

---

## Executive Summary

After Parseltongue completes codebase ingestion into CozoDB, we can immediately generate beautiful, informative visualizations using Unicode box-drawing characters and emojis. These provide instant insights without requiring expensive computation, as all data is already in the database.

**Key Insight:** Since ingestion is complete, CozoDB already has all entities and dependencies. We can run fast aggregation queries (<100ms) and render visual summaries that give developers immediate understanding of their codebase structure, complexity, and health.

**Inspiration:** Claude Code's context usage display shows how effective simple Unicode graphics can be:
```
⛁ ⛁ ⛁ ⛁ ⛁ ⛁ ⛁ ⛁ ⛀ ⛁   64k/200k tokens (32%)
⛀ ⛶ ⛶ ⛶ ⛶ ⛶ ⛶ ⛶ ⛶ ⛶
```

---

## Table of Contents

- [Part 1: Quick Analytics Categories](#part-1-quick-analytics-categories)
- [Part 2: Unicode Visualization Toolkit](#part-2-unicode-visualization-toolkit)
- [Part 3: Specific Visualizations](#part-3-specific-visualizations)
- [Part 4: CozoDB Query Specifications](#part-4-cozodb-query-specifications)
- [Part 5: Implementation Guide](#part-5-implementation-guide)
- [Part 6: Complete Examples](#part-6-complete-examples)

---

# Part 1: Quick Analytics Categories

## Overview

After ingestion, we have complete data about:
- Entities (functions, structs, traits, etc.)
- Dependencies (calls, uses, implements)
- Metadata (complexity, signatures, classifications)
- File/package structure

**All queries below run in <100ms for typical codebases.**

## Category 1: Ingestion Summary

**What:** High-level statistics about what was just ingested.

**Metrics:**
- Total entities by type
- Total dependencies
- Languages detected
- Files processed
- Lines of code
- Ingestion duration

**Value:** Confirms ingestion success, gives codebase overview at a glance.

**Visualization Style:** Bar charts, pie charts, summary cards.

---

## Category 2: Dependency Graph Health

**What:** Quick structural analysis of the dependency graph.

**Metrics:**
- Graph density (how connected)
- Strongly connected components (circular dependencies)
- Average fan-in/fan-out
- Top 10 most connected entities
- Isolated entities (no dependencies)

**Value:** Immediate architectural health check.

**Visualization Style:** Network metrics, top-N lists, health indicators.

---

## Category 3: Complexity Distribution

**What:** Where complexity lives in the codebase.

**Metrics:**
- Complexity histogram (distribution)
- Top 10 most complex functions
- Average complexity per package
- Complexity hotspots (files/modules)

**Value:** Identify technical debt immediately.

**Visualization Style:** Histograms, heatmaps, sparklines.

---

## Category 4: Test Coverage Quick View

**What:** Implementation vs test balance.

**Metrics:**
- Implementation entities vs Test entities
- TDD classification breakdown
- Packages without tests
- Test-to-code ratio

**Value:** Spot testing gaps instantly.

**Visualization Style:** Ratios, percentage bars, gauges.

---

## Category 5: Package/Module Structure

**What:** Codebase organization overview.

**Metrics:**
- Package hierarchy tree
- Inter-package dependencies
- Package sizes (entity count)
- Module cohesion scores

**Value:** Understand codebase architecture.

**Visualization Style:** Tree diagrams, dependency matrices.

---

# Part 2: Unicode Visualization Toolkit

## Character Sets for Terminal Graphics

### Box Drawing Characters

**Full Box Set:**
```
┌─┬─┐  ╔═╦═╗  ╭─┬─╮
│ │ │  ║ ║ ║  │ │ │
├─┼─┤  ╠═╬═╣  ├─┼─┤
│ │ │  ║ ║ ║  │ │ │
└─┴─┘  ╚═╩═╝  ╰─┴─╯
```

**Usage:**
- `┌─┐│└┘` - Standard box drawing
- `╔═╗║╚╝` - Double-line boxes (emphasis)
- `╭─╮│╰╯` - Rounded corners (modern look)

### Block Elements

**Vertical Bars (Histograms):**
```
█ ▇ ▆ ▅ ▄ ▃ ▂ ▁
```

**Horizontal Bars (Progress):**
```
▰▰▰▰▰▰▰▱▱▱  70%
```

**Shaded Blocks:**
```
█ ▓ ▒ ░
Full → Empty
```

**Partial Blocks:**
```
▀ Upper half
▄ Lower half
▌ Left half
▐ Right half
```

### Sparklines (Mini Charts)

**Using Eighth-Blocks:**
```
▁▂▃▄▅▆▇█
```

**Example Trend:**
```
CPU: ▃▅▇█▆▄▂▁
MEM: ▁▂▄█████
```

### Geometric Shapes

**Bullets & Indicators:**
```
• ◦ ○ ● ◉ ◎  - Dots
■ □ ▪ ▫ ◾ ◽  - Squares
▲ △ ▼ ▽ ◀ ▶  - Triangles/Arrows
★ ☆ ✓ ✗ ✱ ✦  - Symbols
```

**Status Indicators:**
```
✓ Success
✗ Failure
⚠ Warning
ℹ Info
```

### Emojis for Visual Impact

**Categories:**
```
📊 Charts/Graphs
📈 Trending Up
📉 Trending Down
🔥 Hot/Important
⚡ Fast/Performance
🎯 Target/Goal
🏗️ Structure/Architecture
🧪 Testing
⚠️ Warning
✅ Success
❌ Error
🔍 Analysis
📦 Package/Module
🔗 Dependencies
⚙️ Configuration
💡 Insight
```

### Color Codes (ANSI)

**Basic Colors:**
```rust
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";
```

**Background Colors:**
```rust
const BG_RED: &str = "\x1b[41m";
const BG_GREEN: &str = "\x1b[42m";
```

**Styles:**
```rust
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const UNDERLINE: &str = "\x1b[4m";
```

**Usage Example:**
```rust
println!("{}✓{} Success", GREEN, RESET);
println!("{}⚠{} Warning", YELLOW, RESET);
println!("{}✗{} Error", RED, RESET);
```

---

# Part 3: Specific Visualizations

## Visualization 1: Ingestion Summary Card

**Purpose:** Show what was just ingested in a beautiful summary.

**Design:**
```
╔══════════════════════════════════════════════════════════╗
║           📦 PARSELTONGUE INGESTION COMPLETE             ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Repository: that-in-rust/parseltongue                  ║
║  Duration:   2.3 seconds                                 ║
║  Database:   parseltongue.db (4.2 MB)                   ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  📊 ENTITIES DISCOVERED                                  ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Functions        ████████████████████ 1,247  (42%)     ║
║  Structs          ████████████░░░░░░░░   687  (23%)     ║
║  Traits           ████████░░░░░░░░░░░░   423  (14%)     ║
║  Enums            ██████░░░░░░░░░░░░░░   312  (11%)     ║
║  Modules          ████░░░░░░░░░░░░░░░░   189   (6%)     ║
║  Tests            ██░░░░░░░░░░░░░░░░░░   123   (4%)     ║
║                                                          ║
║  Total:           2,981 entities                        ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  🔗 DEPENDENCIES                                         ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Calls            8,234                                  ║
║  Uses             3,456                                  ║
║  Implements       1,234                                  ║
║  Derives            567                                  ║
║                                                          ║
║  Total:           13,491 edges                          ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  📈 CODEBASE METRICS                                     ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Files:           234 Rust files                        ║
║  Packages:        12 top-level modules                  ║
║  Lines of Code:   45,678 LOC                            ║
║  Avg Complexity:  4.2 (per function)                    ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝
```

**CozoDB Query:**
```datalog
# Count entities by type
entity_count[entity_type, count] :=
    *CodeGraph{entity_class: entity_type},
    count = count(entity_type)

# Count dependencies by type
dependency_count[edge_type, count] :=
    *DependencyEdges{edge_type},
    count = count(edge_type)
```

---

## Visualization 2: Dependency Graph Health Dashboard

**Purpose:** Quick structural health check of the dependency graph.

**Design:**
```
╔══════════════════════════════════════════════════════════╗
║       🔍 DEPENDENCY GRAPH HEALTH ANALYSIS                ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Graph Density:    ▰▰▰▰▰▰▰▱▱▱  0.072  (7.2%)           ║
║  Connectedness:    ████████░░  High                     ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  ⚠️  CIRCULAR DEPENDENCIES                               ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Cycles Detected:  3                                     ║
║                                                          ║
║    1. auth ⟲ session ⟲ user (3 entities)              ║
║    2. parser ⟲ ast ⟲ tokenizer (3 entities)           ║
║    3. cache ⟲ storage (2 entities)                     ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  🔥 TOP 10 MOST CONNECTED ENTITIES                       ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  1. CozoDbStorage::run_query()         ████████ 47 refs ║
║  2. ISGKey::from_str()                 ███████░ 39 refs ║
║  3. DependencyExtractor::extract()     ██████░░ 34 refs ║
║  4. CodeGraph::insert()                ██████░░ 32 refs ║
║  5. Result<T>::unwrap()                █████░░░ 28 refs ║
║  6. parse_rust_file()                  █████░░░ 26 refs ║
║  7. EntityClassification::classify()   ████░░░░ 23 refs ║
║  8. traverse_ast()                     ████░░░░ 21 refs ║
║  9. log::info()                        ████░░░░ 20 refs ║
║  10. to_string()                       ███░░░░░ 18 refs ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  📊 CONNECTIVITY DISTRIBUTION                            ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Fan-In  (avg):   3.4 incoming dependencies             ║
║  Fan-Out (avg):   4.2 outgoing dependencies             ║
║                                                          ║
║  Distribution:                                           ║
║   0-2 deps:  ████████████████████ 1,234 entities (62%)  ║
║   3-5 deps:  ██████████░░░░░░░░░░   456 entities (23%)  ║
║   6-10 deps: ████░░░░░░░░░░░░░░░░   189 entities (10%)  ║
║   11+ deps:  ██░░░░░░░░░░░░░░░░░░   102 entities  (5%)  ║
║                                                          ║
║  Isolated:   23 entities with no dependencies           ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝
```

**Key Insight:** Shows immediate red flags (cycles) and important entities.

---

## Visualization 3: Complexity Heatmap

**Purpose:** Show where complexity concentrates in the codebase.

**Design:**
```
╔══════════════════════════════════════════════════════════╗
║          🌡️  COMPLEXITY DISTRIBUTION                     ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Average Complexity:  4.2  (target: <10)                ║
║  Median Complexity:   3.0                                ║
║  Max Complexity:      67   (CRITICAL!)                   ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  📊 HISTOGRAM                                            ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║   1-5   ████████████████████  945  (75.8%)              ║
║   6-10  ████░░░░░░░░░░░░░░░░  187  (15.0%)              ║
║  11-20  ██░░░░░░░░░░░░░░░░░░   78   (6.3%)              ║
║  21-50  ░░░░░░░░░░░░░░░░░░░░   34   (2.7%)              ║
║  51+    ░░░░░░░░░░░░░░░░░░░░    3   (0.2%)  🔥          ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  🔥 TOP 10 COMPLEXITY HOTSPOTS                           ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  1. parser::parse()              ████████████ 67  🔴     ║
║  2. validator::validate_all()    ██████████░░ 58  🔴     ║
║  3. analyzer::analyze_complex()  ████████░░░░ 45  🟡     ║
║  4. extract_dependencies()       ███████░░░░░ 42  🟡     ║
║  5. build_isg_key()              ███████░░░░░ 39  🟡     ║
║  6. process_ast_node()           ██████░░░░░░ 34  🟡     ║
║  7. handle_error_cases()         █████░░░░░░░ 28  🟢     ║
║  8. format_output()              █████░░░░░░░ 26  🟢     ║
║  9. merge_results()              ████░░░░░░░░ 23  🟢     ║
║  10. compute_metrics()           ████░░░░░░░░ 21  🟢     ║
║                                                          ║
║  Legend: 🔴 Critical (>50)  🟡 High (20-50)  🟢 OK (<20) ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  📦 COMPLEXITY BY PACKAGE                                ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  pt-parser      ▓▓▓▓▓▓▓▓▓░░  avg: 8.9  (needs review)  ║
║  pt-extractor   ▓▓▓▓▓▓▓░░░░  avg: 6.2                   ║
║  pt-analyzer    ▓▓▓▓▓░░░░░░  avg: 5.1                   ║
║  pt-storage     ▓▓▓▓░░░░░░░  avg: 4.3                   ║
║  pt-cli         ▓▓▓░░░░░░░░  avg: 3.8                   ║
║  pt-utils       ▓▓░░░░░░░░░  avg: 2.9                   ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝
```

**CozoDB Query:**
```datalog
# Complexity distribution
complexity_hist[bucket, count] :=
    *CodeGraph{cyclomatic_complexity: c},
    bucket = case
        when c <= 5 then '1-5'
        when c <= 10 then '6-10'
        when c <= 20 then '11-20'
        when c <= 50 then '21-50'
        else '51+'
    end,
    count = count(c)

# Top complex functions
?[entity, complexity] :=
    *CodeGraph{ISGL1_key: entity, cyclomatic_complexity: complexity},
    :order -complexity,
    :limit 10
```

---

## Visualization 4: Test Coverage Gauge

**Purpose:** Immediate view of test coverage.

**Design:**
```
╔══════════════════════════════════════════════════════════╗
║            🧪 TEST COVERAGE OVERVIEW                     ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Implementation:  1,247 entities                         ║
║  Tests:             123 entities                         ║
║                                                          ║
║  Test Ratio:      ▰▱▱▱▱▱▱▱▱▱  9.9%  (target: 20%+)      ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  📊 TDD CLASSIFICATION                                   ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Implementation      █████████████████░░  1,247  (84%)  ║
║  TestImplementation  ██░░░░░░░░░░░░░░░░    123   (8%)   ║
║  TestHelper          █░░░░░░░░░░░░░░░░░     67   (5%)   ║
║  TestFixture         ░░░░░░░░░░░░░░░░░░     34   (2%)   ║
║  MockImplementation  ░░░░░░░░░░░░░░░░░░     12   (1%)   ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  ⚠️  PACKAGES WITHOUT TESTS                              ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  • pt-parser      (87 entities, 0 tests)  🔴            ║
║  • pt-extractor   (45 entities, 0 tests)  🔴            ║
║  • pt-utils       (23 entities, 2 tests)  🟡            ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  💡 TESTING RECOMMENDATIONS                              ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Priority 1: Add tests to pt-parser (high complexity)   ║
║  Priority 2: Add tests to pt-extractor (critical path)  ║
║  Priority 3: Increase pt-utils test coverage            ║
║                                                          ║
║  Estimated effort: 16 hours                              ║
║  Expected defect reduction: 45%                          ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝
```

---

## Visualization 5: Package Structure Tree

**Purpose:** Show codebase organization at a glance.

**Design:**
```
╔══════════════════════════════════════════════════════════╗
║         📦 PACKAGE STRUCTURE                             ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  parseltongue/                                           ║
║  ├─ 📁 crates/                                           ║
║  │  ├─ pt-cli/              234 entities  ▓▓▓▓▓░░░░░    ║
║  │  ├─ pt-parser/           487 entities  ▓▓▓▓▓▓▓▓▓▓    ║
║  │  ├─ pt-extractor/        312 entities  ▓▓▓▓▓▓▓░░░    ║
║  │  ├─ pt-storage/          189 entities  ▓▓▓▓░░░░░░    ║
║  │  ├─ pt-analyzer/         145 entities  ▓▓▓░░░░░░░    ║
║  │  └─ pt-utils/             67 entities  ▓░░░░░░░░░    ║
║  ├─ 📁 tests/                123 entities  ▓▓░░░░░░░░    ║
║  └─ 📄 lib.rs                 12 entities  ░░░░░░░░░░    ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  🔗 INTER-PACKAGE DEPENDENCIES                           ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║        cli  parser  extract  storage  analyze  utils    ║
║  cli    ■     ═══     ═══      ═══      ──      ──      ║
║  parser ─     ■       ═══      ──       ──      ═══     ║
║  extract─     ═══     ■        ═══      ═══     ──      ║
║  storage─     ─       ──       ■        ──      ──      ║
║  analyze─     ═══     ═══      ═══      ■       ──      ║
║  utils  ─     ─       ─        ─        ─       ■       ║
║                                                          ║
║  Legend: ═══ Strong (10+ deps)  ── Weak (1-9 deps)     ║
║          ■ Self (module internals)                      ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝
```

---

## Visualization 6: Quick Insights Panel

**Purpose:** Actionable insights discovered during ingestion.

**Design:**
```
╔══════════════════════════════════════════════════════════╗
║           💡 QUICK INSIGHTS                              ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  ✓ Well-structured codebase with clear modules          ║
║  ✓ Consistent naming conventions detected               ║
║  ✓ Low average complexity (4.2)                         ║
║                                                          ║
║  ⚠️ 3 circular dependencies need resolution              ║
║  ⚠️ Test coverage below 10% (target: 20%+)               ║
║  ⚠️ 2 functions with critical complexity (>50)           ║
║                                                          ║
║  🔥 PRIORITY ACTIONS                                     ║
║                                                          ║
║  1. Break circular dependency: auth ⟲ session ⟲ user   ║
║     Impact: High | Effort: Medium | Priority: 🔴        ║
║                                                          ║
║  2. Refactor parser::parse() (complexity: 67)           ║
║     Impact: High | Effort: High | Priority: 🔴          ║
║                                                          ║
║  3. Add tests to pt-parser package                      ║
║     Impact: Medium | Effort: High | Priority: 🟡        ║
║                                                          ║
╠══════════════════════════════════════════════════════════╣
║  📈 CODEBASE HEALTH SCORE                                ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Overall:         ▰▰▰▰▰▰▰▰▱▱  78/100  (Good)            ║
║                                                          ║
║  Structure:       ▰▰▰▰▰▰▰▰▰▱  85/100  ✓                 ║
║  Complexity:      ▰▰▰▰▰▰▰▰▱▱  82/100  ✓                 ║
║  Dependencies:    ▰▰▰▰▰▰▱▱▱▱  65/100  ⚠️                 ║
║  Test Coverage:   ▰▰▱▱▱▱▱▱▱▱  45/100  🔴                ║
║  Documentation:   ▰▰▰▰▰▰▰▰▰▰  92/100  ✓                 ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝
```

---

## Visualization 7: Sparkline Trends (for re-ingestion)

**Purpose:** Show how metrics change over time (if ingesting multiple times).

**Design:**
```
╔══════════════════════════════════════════════════════════╗
║           📈 CODEBASE EVOLUTION                          ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Last 10 ingestions:                                     ║
║                                                          ║
║  Entity Count:     ▁▂▂▃▄▅▆▆▇█  2,981 (+247)  ↑         ║
║  Dependencies:     ▂▂▃▄▅▆▇▇██  13,491 (+892)  ↑         ║
║  Avg Complexity:   ▇▆▅▅▄▃▃▂▂▁  4.2 (-1.8)  ↓ 🎉         ║
║  Test Ratio:       ▁▁▂▂▃▃▄▄▅▅  9.9% (+3.2%)  ↑ 🎉       ║
║  Circular Deps:    ▅▅▅▄▄▃▃▃▂▂  3 (-2)  ↓ 🎉             ║
║                                                          ║
║  Trend: Improving! ✓                                     ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝
```

---

# Part 4: CozoDB Query Specifications

## Query Category: Entity Statistics

### Query 1: Count Entities by Type

```datalog
?[entity_class, count] :=
    *CodeGraph{entity_class},
    count = count(entity_class)

:order -count
```

**Output Example:**
```
entity_class           | count
-----------------------+-------
Implementation         | 1247
Struct                 | 687
Trait                  | 423
Enum                   | 312
Module                 | 189
TestImplementation     | 123
```

**Rendering:**
```rust
let total: u32 = results.iter().map(|r| r.count).sum();
for result in results {
    let percentage = (result.count as f64 / total as f64) * 100.0;
    let bar_length = (percentage / 5.0) as usize; // Scale to 20 chars max
    let bar = "█".repeat(bar_length) + &"░".repeat(20 - bar_length);
    println!("  {} {} {:>6}  ({:.1}%)",
        result.entity_class,
        bar,
        result.count,
        percentage
    );
}
```

---

### Query 2: Dependency Count by Type

```datalog
?[edge_type, count] :=
    *DependencyEdges{edge_type},
    count = count(edge_type)

:order -count
```

---

### Query 3: File and Package Counts

```datalog
# Count unique files
?[file_count] :=
    *CodeGraph{file_path},
    file_count = count_unique(file_path)

# Count unique packages (top-level folders)
?[package_count] :=
    *CodeGraph{file_path},
    package = extract_package(file_path),
    package_count = count_unique(package)
```

---

## Query Category: Dependency Graph Health

### Query 4: Graph Density

```datalog
# Graph density = |E| / (|V| * (|V| - 1))
?[density] :=
    node_count = count(*CodeGraph{}),
    edge_count = count(*DependencyEdges{}),
    max_edges = node_count * (node_count - 1),
    density = edge_count / max_edges
```

---

### Query 5: Strongly Connected Components (Cycles)

```datalog
# Find cycles using CozoDB's built-in algorithms
# Note: This uses recursive queries

# Reachable via transitive closure
reachable[from, to] :=
    *DependencyEdges{from_key: from, to_key: to}

reachable[from, to] :=
    reachable[from, mid],
    *DependencyEdges{from_key: mid, to_key: to}

# Entity is in a cycle if it can reach itself
?[entity] :=
    reachable[entity, entity]
```

**Better Approach (Use CozoDB's SCC algorithm):**
```datalog
# CozoDB has built-in strongly_connected_components
# Check documentation for exact syntax
```

---

### Query 6: Top N Most Connected Entities

```datalog
# Calculate in-degree + out-degree for each entity
?[entity, total_connections] :=
    *CodeGraph{ISGL1_key: entity},
    in_degree = count(*DependencyEdges{to_key: entity}),
    out_degree = count(*DependencyEdges{from_key: entity}),
    total_connections = in_degree + out_degree

:order -total_connections
:limit 10
```

---

### Query 7: Fan-In and Fan-Out Distribution

```datalog
# Average fan-in (incoming dependencies)
?[avg_fan_in] :=
    fan_in[entity, count] := *DependencyEdges{to_key: entity},
                             count = count(entity),
    avg_fan_in = mean(count)

# Average fan-out (outgoing dependencies)
?[avg_fan_out] :=
    fan_out[entity, count] := *DependencyEdges{from_key: entity},
                              count = count(entity),
    avg_fan_out = mean(count)

# Distribution buckets
?[bucket, entity_count] :=
    total_deps[entity, total] :=
        in_count = count(*DependencyEdges{to_key: entity}),
        out_count = count(*DependencyEdges{from_key: entity}),
        total = in_count + out_count,
    bucket = case
        when total <= 2 then '0-2'
        when total <= 5 then '3-5'
        when total <= 10 then '6-10'
        else '11+'
    end,
    entity_count = count(entity)
```

---

## Query Category: Complexity Analysis

### Query 8: Complexity Distribution Histogram

```datalog
?[bucket, count] :=
    *CodeGraph{cyclomatic_complexity: c},
    bucket = case
        when c <= 5 then '1-5'
        when c <= 10 then '6-10'
        when c <= 20 then '11-20'
        when c <= 50 then '21-50'
        else '51+'
    end,
    count = count(c)

:order bucket
```

---

### Query 9: Top 10 Most Complex Functions

```datalog
?[entity, complexity, file_path] :=
    *CodeGraph{
        ISGL1_key: entity,
        cyclomatic_complexity: complexity,
        file_path
    },

:order -complexity
:limit 10
```

---

### Query 10: Average Complexity by Package

```datalog
?[package, avg_complexity, entity_count] :=
    *CodeGraph{
        ISGL1_key: entity,
        file_path,
        cyclomatic_complexity: c
    },
    package = extract_package(file_path),
    avg_complexity = mean(c),
    entity_count = count(entity)

:order -avg_complexity
```

**Note:** `extract_package` is a custom function you'd implement in Rust.

---

## Query Category: Test Coverage

### Query 11: Implementation vs Test Counts

```datalog
# Count by TDD classification
?[classification, count] :=
    *CodeGraph{tdd_classification: classification},
    count = count(classification)
```

---

### Query 12: Packages Without Tests

```datalog
# Packages with no test entities
?[package] :=
    *CodeGraph{file_path, entity_class: 'Implementation'},
    package = extract_package(file_path),
    not exists(*CodeGraph{
        file_path: test_path,
        entity_class: 'TestImplementation'
    } where extract_package(test_path) == package)
```

---

## Query Category: Package Structure

### Query 13: Package Sizes

```datalog
?[package, entity_count] :=
    *CodeGraph{ISGL1_key: entity, file_path},
    package = extract_package(file_path),
    entity_count = count(entity)

:order -entity_count
```

---

### Query 14: Inter-Package Dependencies

```datalog
?[from_package, to_package, edge_count] :=
    *DependencyEdges{from_key, to_key},
    from_package = extract_package(from_key),
    to_package = extract_package(to_key),
    from_package != to_package,
    edge_count = count(from_key)

:order -edge_count
```

---

## Query Category: Quick Insights

### Query 15: Isolated Entities

```datalog
# Entities with no incoming or outgoing dependencies
?[entity] :=
    *CodeGraph{ISGL1_key: entity},
    not exists(*DependencyEdges{from_key: entity}),
    not exists(*DependencyEdges{to_key: entity})
```

---

### Query 16: Codebase Health Metrics

```datalog
# Composite query for overall health score
# This would combine multiple metrics

# 1. Structure score (based on modularity)
# 2. Complexity score (based on average complexity)
# 3. Dependency score (based on cycles and coupling)
# 4. Test score (based on test ratio)

# Each component would be a separate query, then combined in Rust
```

---

# Part 5: Implementation Guide

## Architecture

```
┌─────────────────────────────────────────────────────┐
│         Post-Ingestion Analytics Engine             │
└─────────────────────────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│   Query     │ │ Calculation │ │ Rendering   │
│   Engine    │ │   Engine    │ │   Engine    │
└─────────────┘ └─────────────┘ └─────────────┘
        │               │               │
        └───────────────┼───────────────┘
                        ▼
        ┌───────────────────────────────┐
        │       Terminal Output         │
        └───────────────────────────────┘
```

## Module Structure

```rust
// crates/pt-analytics/src/post_ingestion/

pub mod query;          // CozoDB query execution
pub mod calculate;      // Metric calculations
pub mod render;         // Terminal rendering
pub mod visualize;      // Unicode/emoji art
pub mod dashboard;      // Full dashboard assembly

// Main entry point
pub struct PostIngestionAnalytics {
    db: Arc<CozoDbStorage>,
    config: AnalyticsConfig,
}

impl PostIngestionAnalytics {
    pub async fn run(&self) -> Result<()> {
        // 1. Execute all queries in parallel
        let metrics = self.gather_metrics().await?;

        // 2. Calculate derived metrics
        let insights = self.calculate_insights(&metrics)?;

        // 3. Render visualizations
        let output = self.render_dashboard(&metrics, &insights)?;

        // 4. Print to terminal
        println!("{}", output);

        Ok(())
    }
}
```

## Query Module

```rust
// crates/pt-analytics/src/post_ingestion/query.rs

use crate::storage::CozoDbStorage;

pub struct QueryEngine {
    db: Arc<CozoDbStorage>,
}

impl QueryEngine {
    /// Execute all queries in parallel
    pub async fn gather_all_metrics(&self) -> Result<RawMetrics> {
        let (
            entity_counts,
            dependency_counts,
            complexity_dist,
            top_connected,
            cycles,
            test_stats,
            package_stats,
        ) = tokio::join!(
            self.query_entity_counts(),
            self.query_dependency_counts(),
            self.query_complexity_distribution(),
            self.query_top_connected(),
            self.query_cycles(),
            self.query_test_stats(),
            self.query_package_stats(),
        );

        Ok(RawMetrics {
            entity_counts: entity_counts?,
            dependency_counts: dependency_counts?,
            complexity_dist: complexity_dist?,
            top_connected: top_connected?,
            cycles: cycles?,
            test_stats: test_stats?,
            package_stats: package_stats?,
        })
    }

    async fn query_entity_counts(&self) -> Result<Vec<EntityCount>> {
        let query = r#"
            ?[entity_class, count] :=
                *CodeGraph{entity_class},
                count = count(entity_class)
            :order -count
        "#;

        let results = self.db.run_query(query).await?;
        // Parse results into EntityCount structs
        Ok(parse_entity_counts(results))
    }

    // ... other query methods
}

pub struct RawMetrics {
    pub entity_counts: Vec<EntityCount>,
    pub dependency_counts: Vec<DependencyCount>,
    pub complexity_dist: Vec<ComplexityBucket>,
    pub top_connected: Vec<ConnectedEntity>,
    pub cycles: Vec<Cycle>,
    pub test_stats: TestStats,
    pub package_stats: Vec<PackageStats>,
}
```

## Calculation Module

```rust
// crates/pt-analytics/src/post_ingestion/calculate.rs

pub struct CalculationEngine;

impl CalculationEngine {
    pub fn calculate_insights(metrics: &RawMetrics) -> Insights {
        Insights {
            health_score: Self::calculate_health_score(metrics),
            priority_actions: Self::identify_priority_actions(metrics),
            trends: Self::calculate_trends(metrics),
        }
    }

    fn calculate_health_score(metrics: &RawMetrics) -> HealthScore {
        let structure_score = Self::score_structure(metrics);
        let complexity_score = Self::score_complexity(metrics);
        let dependency_score = Self::score_dependencies(metrics);
        let test_score = Self::score_tests(metrics);

        HealthScore {
            overall: (structure_score + complexity_score +
                     dependency_score + test_score) / 4.0,
            structure: structure_score,
            complexity: complexity_score,
            dependencies: dependency_score,
            tests: test_score,
        }
    }

    fn score_structure(metrics: &RawMetrics) -> f64 {
        // Logic to score structure
        // - Package organization
        // - Module cohesion
        // - Naming consistency
        85.0 // Placeholder
    }

    fn score_complexity(metrics: &RawMetrics) -> f64 {
        let avg_complexity = metrics.complexity_dist
            .iter()
            .map(|b| b.avg_complexity)
            .sum::<f64>() / metrics.complexity_dist.len() as f64;

        // Score: 100 if avg <= 5, scale down to 0 at avg >= 20
        if avg_complexity <= 5.0 {
            100.0
        } else if avg_complexity >= 20.0 {
            0.0
        } else {
            100.0 - ((avg_complexity - 5.0) / 15.0) * 100.0
        }
    }

    fn score_dependencies(metrics: &RawMetrics) -> f64 {
        let cycle_penalty = metrics.cycles.len() as f64 * 10.0;
        (100.0 - cycle_penalty).max(0.0)
    }

    fn score_tests(metrics: &RawMetrics) -> f64 {
        let test_ratio = metrics.test_stats.test_ratio;
        // Score: 0 at 0%, 100 at 20%+
        (test_ratio * 5.0).min(100.0)
    }

    fn identify_priority_actions(metrics: &RawMetrics) -> Vec<Action> {
        let mut actions = Vec::new();

        // Check for cycles
        if !metrics.cycles.is_empty() {
            actions.push(Action {
                title: format!("Break {} circular dependencies", metrics.cycles.len()),
                impact: Impact::High,
                effort: Effort::Medium,
                priority: Priority::Critical,
            });
        }

        // Check for high complexity
        let high_complexity: Vec<_> = metrics.complexity_dist
            .iter()
            .filter(|b| b.max_complexity > 50)
            .collect();

        if !high_complexity.is_empty() {
            actions.push(Action {
                title: "Refactor high-complexity functions".to_string(),
                impact: Impact::High,
                effort: Effort::High,
                priority: Priority::Critical,
            });
        }

        // Check for low test coverage
        if metrics.test_stats.test_ratio < 0.1 {
            actions.push(Action {
                title: "Increase test coverage".to_string(),
                impact: Impact::Medium,
                effort: Effort::High,
                priority: Priority::High,
            });
        }

        actions
    }
}
```

## Rendering Module

```rust
// crates/pt-analytics/src/post_ingestion/render.rs

use super::visualize::{BarChart, Box, Gauge, ProgressBar, Sparkline};

pub struct RenderEngine {
    width: usize,  // Terminal width
}

impl RenderEngine {
    pub fn render_dashboard(
        &self,
        metrics: &RawMetrics,
        insights: &Insights,
    ) -> String {
        let mut output = String::new();

        // Render each section
        output.push_str(&self.render_ingestion_summary(metrics));
        output.push('\n');
        output.push_str(&self.render_dependency_health(metrics));
        output.push('\n');
        output.push_str(&self.render_complexity_overview(metrics));
        output.push('\n');
        output.push_str(&self.render_test_coverage(metrics));
        output.push('\n');
        output.push_str(&self.render_insights(insights));

        output
    }

    fn render_ingestion_summary(&self, metrics: &RawMetrics) -> String {
        let mut output = String::new();

        // Top box
        output.push_str(&Box::double_line()
            .title("📦 PARSELTONGUE INGESTION COMPLETE")
            .render());

        // Entity counts with bars
        output.push_str(&self.render_entity_counts(&metrics.entity_counts));

        // Dependency counts
        output.push_str(&self.render_dependency_counts(&metrics.dependency_counts));

        output
    }

    fn render_entity_counts(&self, counts: &[EntityCount]) -> String {
        let total: u32 = counts.iter().map(|c| c.count).sum();

        let mut output = String::new();
        output.push_str("║  📊 ENTITIES DISCOVERED\n");
        output.push_str("╠══════════════════════════════════════════════════════════╣\n");
        output.push_str("║\n");

        for entity in counts {
            let percentage = (entity.count as f64 / total as f64) * 100.0;
            let bar = ProgressBar::new(20)
                .value(percentage / 5.0)  // Scale to 0-20
                .render();

            output.push_str(&format!(
                "║  {:18} {} {:>5}  ({:.0}%)\n",
                entity.entity_class,
                bar,
                entity.count,
                percentage
            ));
        }

        output.push_str("║\n");
        output.push_str(&format!("║  Total:           {} entities\n", total));
        output.push_str("║\n");

        output
    }

    fn render_complexity_overview(&self, metrics: &RawMetrics) -> String {
        let mut output = String::new();

        output.push_str(&Box::double_line()
            .title("🌡️  COMPLEXITY DISTRIBUTION")
            .render());

        // Histogram
        output.push_str(&BarChart::horizontal()
            .data(&metrics.complexity_dist)
            .render());

        // Top 10 complex
        output.push_str("║  🔥 TOP 10 COMPLEXITY HOTSPOTS\n");
        for (i, entity) in metrics.top_complex.iter().enumerate().take(10) {
            let indicator = match entity.complexity {
                c if c > 50 => "🔴",
                c if c > 20 => "🟡",
                _ => "🟢",
            };

            let bar_len = (entity.complexity as f64 / 70.0 * 12.0) as usize;
            let bar = "█".repeat(bar_len) + &"░".repeat(12 - bar_len);

            output.push_str(&format!(
                "║  {:2}. {:30} {} {:>3}  {}\n",
                i + 1,
                truncate(&entity.name, 30),
                bar,
                entity.complexity,
                indicator
            ));
        }

        output
    }

    fn render_insights(&self, insights: &Insights) -> String {
        let mut output = String::new();

        output.push_str(&Box::double_line()
            .title("💡 QUICK INSIGHTS")
            .render());

        // Health score
        output.push_str("║  📈 CODEBASE HEALTH SCORE\n");
        output.push_str("║\n");

        let gauge = Gauge::new()
            .value(insights.health_score.overall)
            .max(100.0)
            .width(20)
            .render();

        let status = match insights.health_score.overall {
            s if s >= 80.0 => "Good",
            s if s >= 60.0 => "Fair",
            _ => "Needs Improvement",
        };

        output.push_str(&format!(
            "║  Overall:         {}  {:.0}/100  ({})\n",
            gauge,
            insights.health_score.overall,
            status
        ));

        // Priority actions
        output.push_str("║\n");
        output.push_str("║  🔥 PRIORITY ACTIONS\n");
        output.push_str("║\n");

        for (i, action) in insights.priority_actions.iter().enumerate().take(3) {
            let priority_icon = match action.priority {
                Priority::Critical => "🔴",
                Priority::High => "🟡",
                Priority::Medium => "🟢",
                Priority::Low => "⚪",
            };

            output.push_str(&format!(
                "║  {}. {} {}\n",
                i + 1,
                action.title,
                priority_icon
            ));
            output.push_str(&format!(
                "║     Impact: {:?} | Effort: {:?}\n",
                action.impact,
                action.effort
            ));
            output.push_str("║\n");
        }

        output
    }
}
```

## Visualization Module

```rust
// crates/pt-analytics/src/post_ingestion/visualize.rs

pub struct ProgressBar {
    width: usize,
    value: f64,  // 0.0 to 1.0
    filled_char: char,
    empty_char: char,
}

impl ProgressBar {
    pub fn new(width: usize) -> Self {
        Self {
            width,
            value: 0.0,
            filled_char: '▰',
            empty_char: '▱',
        }
    }

    pub fn value(mut self, value: f64) -> Self {
        self.value = value.clamp(0.0, 1.0);
        self
    }

    pub fn render(&self) -> String {
        let filled = (self.value * self.width as f64) as usize;
        let empty = self.width - filled;

        format!(
            "{}{}",
            self.filled_char.to_string().repeat(filled),
            self.empty_char.to_string().repeat(empty)
        )
    }
}

pub struct Gauge {
    value: f64,
    max: f64,
    width: usize,
}

impl Gauge {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            max: 100.0,
            width: 10,
        }
    }

    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn render(&self) -> String {
        let ratio = (self.value / self.max).clamp(0.0, 1.0);
        let filled = (ratio * self.width as f64) as usize;
        let empty = self.width - filled;

        "▰".repeat(filled) + &"▱".repeat(empty)
    }
}

pub struct Sparkline {
    values: Vec<f64>,
}

impl Sparkline {
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }

    pub fn render(&self) -> String {
        if self.values.is_empty() {
            return String::new();
        }

        let min = self.values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = self.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        if range == 0.0 {
            return "▄".repeat(self.values.len());
        }

        let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        self.values
            .iter()
            .map(|&v| {
                let normalized = (v - min) / range;
                let index = (normalized * (chars.len() - 1) as f64) as usize;
                chars[index]
            })
            .collect()
    }
}

pub struct Box {
    title: String,
    style: BoxStyle,
}

pub enum BoxStyle {
    Single,
    Double,
    Rounded,
}

impl Box {
    pub fn double_line() -> Self {
        Self {
            title: String::new(),
            style: BoxStyle::Double,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn render(&self) -> String {
        let (top_left, top_right, bottom_left, bottom_right, horizontal, vertical) =
            match self.style {
                BoxStyle::Single => ('┌', '┐', '└', '┘', '─', '│'),
                BoxStyle::Double => ('╔', '╗', '╚', '╝', '═', '║'),
                BoxStyle::Rounded => ('╭', '╮', '╰', '╯', '─', '│'),
            };

        let width = 60;
        let title_padded = if self.title.is_empty() {
            horizontal.to_string().repeat(width - 2)
        } else {
            let padding = (width - 2 - self.title.len()) / 2;
            format!(
                "{}{}{}",
                horizontal.to_string().repeat(padding),
                self.title,
                horizontal.to_string().repeat(width - 2 - padding - self.title.len())
            )
        };

        format!(
            "{}{}{}\n",
            top_left,
            title_padded,
            top_right
        )
    }
}
```

## Hook into Ingestion

```rust
// crates/pt-cli/src/commands/ingest.rs

use pt_analytics::post_ingestion::PostIngestionAnalytics;

pub async fn run_ingest(args: IngestArgs) -> Result<()> {
    // ... existing ingestion logic ...

    println!("Ingestion complete!");

    // NEW: Run post-ingestion analytics
    if !args.skip_analytics {
        println!("\nGenerating analytics...\n");

        let analytics = PostIngestionAnalytics::new(
            storage.clone(),
            AnalyticsConfig::default()
        );

        analytics.run().await?;
    }

    Ok(())
}
```

---

# Part 6: Complete Examples

## Example 1: Minimal Output (Fast)

**For quick feedback:**

```
✓ Ingestion complete: 2,981 entities | 13,491 dependencies | 2.3s

📊 Quick Stats:
  Entities:   Functions: 1,247 | Structs: 687 | Traits: 423
  Health:     ▰▰▰▰▰▰▰▰▱▱ 78/100
  Complexity: avg 4.2 | max 67 🔴
  Tests:      ▰▱▱▱▱▱▱▱▱▱ 9.9%

⚠️  3 issues: 3 cycles | 2 high complexity | low test coverage

Run 'pt-analytics dashboard' for detailed analysis.
```

**Implementation:**
```rust
pub fn render_minimal(metrics: &RawMetrics) -> String {
    format!(
        "✓ Ingestion complete: {} entities | {} dependencies | {:.1}s\n\n\
         📊 Quick Stats:\n\
         ...",
        metrics.total_entities(),
        metrics.total_dependencies(),
        metrics.duration
    )
}
```

---

## Example 2: Standard Output (Recommended)

**Full dashboard as shown in visualizations above.**

---

## Example 3: Verbose Output (Detailed)

**Includes everything plus:**
- Per-package detailed breakdown
- Full cycle chains
- All isolated entities
- Recommendations with effort estimates

---

## Example 4: JSON Output (for tools)

```json
{
  "timestamp": "2025-11-05T10:30:00Z",
  "duration_ms": 2300,
  "entities": {
    "total": 2981,
    "by_type": {
      "Implementation": 1247,
      "Struct": 687,
      "Trait": 423
    }
  },
  "dependencies": {
    "total": 13491,
    "by_type": {
      "Calls": 8234,
      "Uses": 3456
    }
  },
  "health_score": {
    "overall": 78,
    "structure": 85,
    "complexity": 82,
    "dependencies": 65,
    "tests": 45
  },
  "issues": [
    {
      "type": "cycle",
      "severity": "high",
      "message": "Circular dependency: auth -> session -> user"
    }
  ]
}
```

---

## Performance Targets

All analytics should complete in:
- **Minimal**: <100ms
- **Standard**: <500ms
- **Verbose**: <1s
- **JSON**: <200ms

For 10,000 entities and 50,000 dependencies.

**Optimization strategies:**
1. Parallel query execution (tokio::join!)
2. Indexed CozoDB queries
3. Incremental computation (cache results)
4. Lazy rendering (only compute what's displayed)

---

## Configuration

```rust
pub struct AnalyticsConfig {
    pub output_style: OutputStyle,
    pub color_enabled: bool,
    pub max_top_n: usize,
    pub show_packages: bool,
    pub show_cycles: bool,
    pub show_complexity: bool,
    pub show_tests: bool,
    pub show_insights: bool,
}

pub enum OutputStyle {
    Minimal,
    Standard,
    Verbose,
    Json,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            output_style: OutputStyle::Standard,
            color_enabled: true,
            max_top_n: 10,
            show_packages: true,
            show_cycles: true,
            show_complexity: true,
            show_tests: true,
            show_insights: true,
        }
    }
}
```

---

## CLI Integration

```bash
# Automatic after ingestion
parseltongue ingest /path/to/code

# Manual run
parseltongue analytics dashboard
parseltongue analytics summary --minimal
parseltongue analytics health
parseltongue analytics complexity --top 20
parseltongue analytics cycles
parseltongue analytics tests

# Export
parseltongue analytics export --format json > metrics.json
```

---
