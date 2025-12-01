# FAQs to Parseltongue v1.1.0

> A comprehensive guide to questions you can ask Parseltongue's HTTP API about your codebase.

**Server Endpoint**: `http://localhost:7777`
**Start Server**: `./parseltongue serve-http-code-backend --db "rocksdb:your.db" --port 7777`

---

## Table of Contents

1. [Quick Start Questions](#1-quick-start-questions)
2. [Codebase Discovery](#2-codebase-discovery)
3. [Architecture Analysis](#3-architecture-analysis)
4. [Impact Analysis (Blast Radius)](#4-impact-analysis-blast-radius)
5. [Code Quality & Hotspots](#5-code-quality--hotspots)
6. [LLM Context Optimization](#6-llm-context-optimization)
7. [Dependency Graph Exploration](#7-dependency-graph-exploration)
8. [Developer Scenarios](#8-developer-scenarios)
9. [API Quick Reference](#9-api-quick-reference)

---

## 1. Quick Start Questions

### "Is the server running?"
```bash
curl http://localhost:7777/server-health-check-status
```
**Answer**: Returns server status, uptime, and database connection state.

### "What's in this codebase?"
```bash
curl http://localhost:7777/codebase-statistics-overview-summary
```
**Answer**: Total entities (CODE/TEST), edges, languages detected, database path.

### "What APIs are available?"
```bash
curl http://localhost:7777/api-reference-documentation-help
```
**Answer**: Complete list of 13 endpoints with descriptions and parameters.

---

## 2. Codebase Discovery

### "What are all the functions, structs, and methods in this codebase?"
```bash
curl http://localhost:7777/code-entities-list-all
```
**Sample Questions Answered**:
- How many functions exist?
- What structs are defined?
- What methods are implemented?
- What entity types exist (function, struct, method, impl, trait)?

### "Show me details about a specific function"
```bash
curl "http://localhost:7777/code-entity-detail-view?key=rust:fn:main:crates/parseltongue/src/main.rs:8-45"
```
**Sample Questions Answered**:
- What file is this function in?
- What line numbers does it span?
- What is the function signature?
- What's the code implementation?

### "Find all entities matching a pattern"
```bash
curl "http://localhost:7777/code-entities-search-fuzzy?q=query"
curl "http://localhost:7777/code-entities-search-fuzzy?q=handler"
curl "http://localhost:7777/code-entities-search-fuzzy?q=storage"
```
**Sample Questions Answered**:
- Where are all the query-related functions?
- Find all handler implementations
- What storage-related code exists?
- Where is "parse" functionality implemented?

---

## 3. Architecture Analysis

### "What dependencies exist in this codebase?"
```bash
curl http://localhost:7777/dependency-edges-list-all
```
**Sample Questions Answered**:
- What files depend on what other files?
- How many dependency relationships exist?
- What's the dependency structure?

### "What does this function call (forward dependencies)?"
```bash
curl "http://localhost:7777/forward-callees-query-graph?entity=rust:fn:main:crates/parseltongue/src/main.rs:8-45"
```
**Sample Questions Answered**:
- What functions does `main()` call?
- What are the dependencies of this module?
- What external libraries are used?
- What stdlib functions are called?

### "What calls this function (reverse dependencies)?"
```bash
curl "http://localhost:7777/reverse-callers-query-graph?entity=rust:method:new:crates/parseltongue-core/src/parsing/entities.rs:38-54"
```
**Sample Questions Answered**:
- Who uses this struct?
- What functions call this method?
- Where is this API consumed?
- How widely used is this function?

### "Are there circular dependencies?"
```bash
curl http://localhost:7777/circular-dependency-detection-scan
```
**Sample Questions Answered**:
- Does the codebase have circular imports?
- Are there dependency cycles that could cause issues?
- Is the architecture clean from circular refs?

---

## 4. Impact Analysis (Blast Radius)

### "If I change this function, what breaks?"
```bash
curl "http://localhost:7777/blast-radius-impact-analysis?entity=rust:method:new:path:38-54&hops=3"
```
**Sample Questions Answered**:
- What's the blast radius of changing `new()`?
- How many entities depend on this transitively?
- What's the risk of modifying this code?
- Which parts of the system would be affected?

### "What's the transitive impact at different depths?"
```bash
# 1-hop: Direct callers only
curl "http://localhost:7777/blast-radius-impact-analysis?entity=KEY&hops=1"

# 3-hop: Three levels of transitive dependencies
curl "http://localhost:7777/blast-radius-impact-analysis?entity=KEY&hops=3"

# 5-hop: Deep transitive analysis
curl "http://localhost:7777/blast-radius-impact-analysis?entity=KEY&hops=5"
```
**Sample Questions Answered**:
- What directly depends on this?
- What's the full transitive closure?
- How does impact grow with depth?

---

## 5. Code Quality & Hotspots

### "What are the most complex/connected parts of the codebase?"
```bash
curl http://localhost:7777/complexity-hotspots-ranking-view
curl "http://localhost:7777/complexity-hotspots-ranking-view?top=20"
```
**Sample Questions Answered**:
- What functions have the most dependencies?
- Where are the complexity hotspots?
- What code might need refactoring?
- What's the most interconnected code?

### "How is the code organized into clusters?"
```bash
curl http://localhost:7777/semantic-cluster-grouping-list
```
**Sample Questions Answered**:
- What logical groupings exist?
- How is functionality organized?
- What are the semantic boundaries?

---

## 6. LLM Context Optimization

### "Give me focused context for working on X within token budget"
```bash
# 4K token budget for working on storage
curl "http://localhost:7777/smart-context-token-budget?focus=rust:fn:storage:path:1-50&tokens=4000"

# 8K token budget for working on parsing
curl "http://localhost:7777/smart-context-token-budget?focus=rust:struct:Parser:path:1-100&tokens=8000"
```
**Sample Questions Answered**:
- What context do I need to understand this function?
- What's the minimal set of code to include for this task?
- How do I stay under token limits while getting full context?

### "What's the most relevant code for understanding X?"
```bash
curl "http://localhost:7777/smart-context-token-budget?focus=ENTITY_KEY&tokens=2000"
```
**The Killer Feature**: Returns prioritized context:
1. The focus entity itself
2. Direct dependencies (callees)
3. Direct callers
4. Transitive dependencies (if budget allows)

---

## 7. Dependency Graph Exploration

### "What does the `unknown:0-0` pattern mean?"
External/stdlib function calls appear as:
- `rust:fn:new:unknown:0-0` → `HashMap::new()`, `Vec::new()`
- `rust:fn:unwrap:unknown:0-0` → `.unwrap()` calls
- `rust:fn:expect:unknown:0-0` → `.expect()` calls
- `rust:fn:clone:unknown:0-0` → `.clone()` calls

These represent calls to standard library or external crate functions that aren't defined in your codebase.

### "What's the ISGL1 key format?"
```
language:type:name:path:start_line-end_line
```
Examples:
- `rust:fn:main:crates/cli/src/main.rs:8-45` (function)
- `rust:struct:Parser:crates/core/src/parser.rs:10-25` (struct)
- `rust:method:new:crates/core/src/storage.rs:38-54` (method)
- `rust:impl:Storage:crates/core/src/storage.rs:56-200` (impl block)

---

## 8. Developer Scenarios

### Scenario 1: "I'm new to this codebase. Where do I start?"
```bash
# 1. Get overview
curl http://localhost:7777/codebase-statistics-overview-summary

# 2. Find main entry points
curl "http://localhost:7777/code-entities-search-fuzzy?q=main"

# 3. See what main calls
curl "http://localhost:7777/forward-callees-query-graph?entity=rust:fn:main:..."

# 4. Find complexity hotspots
curl "http://localhost:7777/complexity-hotspots-ranking-view?top=10"
```

### Scenario 2: "I need to modify the storage layer. What's the impact?"
```bash
# 1. Find storage entities
curl "http://localhost:7777/code-entities-search-fuzzy?q=storage"

# 2. Get blast radius for the main storage struct
curl "http://localhost:7777/blast-radius-impact-analysis?entity=rust:struct:CozoDbStorage:...&hops=3"

# 3. Get smart context for editing
curl "http://localhost:7777/smart-context-token-budget?focus=rust:struct:CozoDbStorage:...&tokens=8000"
```

### Scenario 3: "Is it safe to refactor this function?"
```bash
# 1. Check who calls it (reverse deps)
curl "http://localhost:7777/reverse-callers-query-graph?entity=FUNCTION_KEY"

# 2. Check blast radius
curl "http://localhost:7777/blast-radius-impact-analysis?entity=FUNCTION_KEY&hops=2"

# 3. If few callers and small blast radius → safe to refactor
```

### Scenario 4: "I found a bug in X. What tests cover it?"
```bash
# 1. Get codebase stats to see test count
curl http://localhost:7777/codebase-statistics-overview-summary

# 2. Search for test entities
curl "http://localhost:7777/code-entities-search-fuzzy?q=test"

# 3. Check reverse callers to find test coverage
curl "http://localhost:7777/reverse-callers-query-graph?entity=BUGGY_FUNCTION"
```

### Scenario 5: "Prepare context for LLM code review"
```bash
# 1. Get focused context within token budget
curl "http://localhost:7777/smart-context-token-budget?focus=FILE_ENTITY&tokens=4000"

# 2. Include the response in your LLM prompt
# Result: 99% token reduction vs dumping raw files
```

---

## 9. API Quick Reference

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/server-health-check-status` | GET | Health check |
| `/codebase-statistics-overview-summary` | GET | Stats overview |
| `/api-reference-documentation-help` | GET | API docs |
| `/code-entities-list-all` | GET | List all entities |
| `/code-entity-detail-view?key=X` | GET | Entity details |
| `/code-entities-search-fuzzy?q=X` | GET | Fuzzy search |
| `/dependency-edges-list-all` | GET | List all edges |
| `/reverse-callers-query-graph?entity=X` | GET | Who calls X? |
| `/forward-callees-query-graph?entity=X` | GET | What does X call? |
| `/blast-radius-impact-analysis?entity=X&hops=N` | GET | Impact analysis |
| `/circular-dependency-detection-scan` | GET | Find cycles |
| `/complexity-hotspots-ranking-view?top=N` | GET | Hotspots |
| `/semantic-cluster-grouping-list` | GET | Code clusters |
| `/smart-context-token-budget?focus=X&tokens=N` | GET | LLM context |

---

## Common Question Patterns

| Question Type | Endpoint | Example |
|--------------|----------|---------|
| "What is X?" | `/code-entity-detail-view` | Get function signature and code |
| "Where is X?" | `/code-entities-search-fuzzy` | Find by name pattern |
| "What uses X?" | `/reverse-callers-query-graph` | Find callers |
| "What does X use?" | `/forward-callees-query-graph` | Find callees |
| "What breaks if X changes?" | `/blast-radius-impact-analysis` | Impact analysis |
| "What's most complex?" | `/complexity-hotspots-ranking-view` | Find hotspots |
| "Give me context for X" | `/smart-context-token-budget` | LLM-ready context |

---

## Data Granularity Notes

### Entities (Fine-Grained)
- Functions, methods, structs, traits, impls
- Line-level precision: `start_line-end_line`
- Full code content available

### Edges (File-to-Symbol)
- Track which files call which symbols
- External calls use `unknown:0-0` pattern
- Both forward and reverse traversal supported

---

*Generated from Parseltongue v1.1.0 HTTP API*
