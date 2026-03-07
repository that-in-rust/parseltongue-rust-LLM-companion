# GitNexus Deep Analysis for Parseltongue v2.0.0

**Date:** 2026-03-03
**Purpose:** Extract learnings from GitNexus codebase to inform Parseltongue architecture
**Source:** https://github.com/abhigyanpatwari/GitNexus

---

## 1. Executive Summary

GitNexus is a multi-language code knowledge graph with MCP integration. It uses tree-sitter for parsing, KuzuDB for graph storage, and implements Leiden community detection + process tracing. Key insights:

| Aspect | GitNexus Approach | Parseltongue Opportunity |
|--------|-------------------|--------------------------|
| **Parsing** | Tree-sitter (syntactic) | rust-analyzer ra_ap_* (semantic) |
| **Graph Storage** | KuzuDB | Custom Rust graph (potentially) |
| **Search** | BM25 + semantic embeddings + RRF | Same + compiler-aware ranking |
| **Communities** | Leiden via graphology | Same + semantic clustering |
| **Processes** | Entry-point BFS tracing | Same + MIR control flow |
| **Impact** | Depth-grouped blast radius | Same + type-based taint |
| **MCP Tools** | 7 tools | Similar pattern |

---

## 2. Architecture Overview

### 2.1 Technology Stack

```
┌─────────────────────────────────────────────────────────────────┐
│                        MCP Server Layer                         │
│  (Claude Code, Cursor, Windsurf, OpenCode integration)         │
├─────────────────────────────────────────────────────────────────┤
│                     Local Backend (multi-repo)                  │
│  - Lazy KuzuDB connections                                      │
│  - Global repo registry (~/.gitnexus/repos.json)               │
├──────────────────────┬──────────────────────────────────────────┤
│    Core Ingestion    │           Query Layer                    │
│  - Tree-sitter parse │  - BM25 FTS (KuzuDB native)             │
│  - Process detection │  - Semantic (ONNX embeddings)           │
│  - Leiden community  │  - RRF fusion                           │
├──────────────────────┴──────────────────────────────────────────┤
│                      Graph Storage                              │
│  - KuzuDB (Cypher queries)                                     │
│  - Node types: 20+ (Function, Class, Struct, Trait, etc.)     │
│  - Edge types: 11 (CALLS, IMPORTS, EXTENDS, etc.)             │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Node and Relationship Types

**Node Labels (20+):**
```typescript
type NodeLabel =
  | 'Project' | 'Package' | 'Module' | 'Folder' | 'File'
  | 'Class' | 'Function' | 'Method' | 'Variable' | 'Interface'
  | 'Enum' | 'Decorator' | 'Import' | 'Type' | 'CodeElement'
  | 'Community' | 'Process'
  // Multi-language additions
  | 'Struct' | 'Macro' | 'Typedef' | 'Union' | 'Namespace'
  | 'Trait' | 'Impl' | 'TypeAlias' | 'Const' | 'Static'
  | 'Property' | 'Record' | 'Delegate' | 'Annotation'
  | 'Constructor' | 'Template';
```

**Relationship Types (11):**
```typescript
type RelationshipType = 
  | 'CONTAINS'   // File → Function
  | 'CALLS'      // Function → Function
  | 'INHERITS'   // Class → Class
  | 'OVERRIDES'  // Method → Method
  | 'IMPORTS'    // File → Module
  | 'USES'       // General dependency
  | 'DEFINES'    // Module → Symbol
  | 'DECORATES'  // Decorator → Function
  | 'IMPLEMENTS' // Class → Interface
  | 'EXTENDS'    // Class → Class
  | 'MEMBER_OF'  // Symbol → Community
  | 'STEP_IN_PROCESS';  // Symbol → Process (with step number)
```

### 2.3 Key Data Structures

```typescript
interface GraphNode {
  id: string;
  label: NodeLabel;
  properties: {
    name: string;
    filePath: string;
    startLine?: number;
    endLine?: number;
    language?: string;
    isExported?: boolean;
    // Framework hints from AST
    astFrameworkMultiplier?: number;
    astFrameworkReason?: string;
    // Community/Process specific
    heuristicLabel?: string;
    cohesion?: number;
    entryPointScore?: number;
    entryPointReason?: string;
  };
}

interface GraphRelationship {
  id: string;
  sourceId: string;
  targetId: string;
  type: RelationshipType;
  confidence: number;  // 0-1, critical for filtering
  reason: string;      // 'import-resolved', 'same-file', 'fuzzy-global'
  step?: number;       // For STEP_IN_PROCESS edges
}
```

---

## 3. Core Algorithms

### 3.1 Process Detection (Entry Point → Trace → Dedupe)

**Algorithm:**
1. **Find Entry Points** - Score functions by:
   - Call ratio: `callees / (callers + 1)` (higher = better entry point)
   - Export status: 2x multiplier for public functions
   - Name patterns: 1.5x for `handle*`, `on*`, `*Controller`, etc.
   - Framework detection: Path-based (e.g., `pages/api/` for Next.js)
   - Utility penalty: 0.3x for getters/setters, helpers

2. **BFS Trace** - From each entry point:
   - Follow CALLS edges with `confidence >= 0.5`
   - Limit: maxDepth=10, maxBranching=4 per node
   - Avoid cycles

3. **Deduplication** - Two phases:
   - Subset removal: Remove traces that are prefixes of longer traces
   - Endpoint deduplication: Keep only longest trace per (entry, terminal) pair

4. **Process Creation** - Generate heuristic labels:
   - Format: `{EntryName} → {TerminalName}`
   - Track: cross_community vs intra_community

```typescript
// From process-processor.ts
const DEFAULT_CONFIG: ProcessDetectionConfig = {
  maxTraceDepth: 10,
  maxBranching: 4,
  maxProcesses: 75,
  minSteps: 3,  // 3+ steps = genuine flow, 2-step is just "A calls B"
};
```

### 3.2 Community Detection (Leiden Algorithm)

**Algorithm:**
1. Build graphology graph with symbol nodes + CALLS/EXTENDS/IMPLEMENTS edges
2. For large graphs (>10K symbols):
   - Filter low-confidence edges (<0.5)
   - Remove degree-1 nodes (noise reduction)
   - Use higher resolution (2.0 vs 1.0)
   - Cap iterations at 3 (diminishing returns)
3. Generate heuristic labels from folder patterns:
   - Most common folder name in community
   - Skip generic names (src, lib, utils, common)
   - Fallback: common prefix of function names

```typescript
// Cohesion calculation (sampling for large communities)
const calculateCohesion = (memberIds: string[], graph: any): number => {
  const sample = memberIds.length <= 50 ? memberIds : memberIds.slice(0, 50);
  // Cohesion = fraction of edges that stay internal
  return internalEdges / totalEdges;
};
```

### 3.3 Hybrid Search (BM25 + Semantic + RRF)

**Reciprocal Rank Fusion:**
```typescript
// RRF score = 1 / (k + rank), k=60 is standard
const rrfScore = 1 / (60 + rank);

// Merge BM25 and semantic results
for (const [rank, result] of bm25Results) {
  scoreMap.set(key, { score: 1 / (60 + rank), data: result });
}
for (const [rank, result] of semanticResults) {
  const existing = scoreMap.get(key);
  existing.score += 1 / (60 + rank);  // Additive fusion
}
```

**Process-Grouped Query Response:**
```json
{
  "processes": [
    {
      "id": "proc_0_handleLogin",
      "summary": "HandleLogin → CreateSession",
      "priority": 0.042,
      "symbol_count": 5,
      "process_type": "cross_community",
      "step_count": 7
    }
  ],
  "process_symbols": [
    {
      "id": "Function:src/auth.rs:handleLogin",
      "name": "handleLogin",
      "type": "Function",
      "filePath": "src/auth.rs",
      "process_id": "proc_0_handleLogin",
      "step_index": 1
    }
  ],
  "definitions": [...]
}
```

### 3.4 Impact Analysis (Blast Radius)

**Algorithm:**
1. Find target symbol by name
2. BFS traverse upstream (callers) or downstream (callees)
3. Group impacted nodes by depth (1 = direct, 2 = transitive, etc.)
4. Enrich with:
   - Affected processes: Which execution flows are broken
   - Affected modules: Which communities are hit
   - Risk scoring: LOW/MEDIUM/HIGH/CRITICAL

```typescript
// Risk scoring heuristics
let risk = 'LOW';
if (directCount >= 30 || processCount >= 5 || moduleCount >= 5 || totalImpacted >= 200) {
  risk = 'CRITICAL';
} else if (directCount >= 15 || processCount >= 3 || moduleCount >= 3 || totalImpacted >= 100) {
  risk = 'HIGH';
} else if (directCount >= 5 || totalImpacted >= 30) {
  risk = 'MEDIUM';
}
```

---

## 4. MCP Tool Design Patterns

### 4.1 Tool Description Format

GitNexus uses a structured format for tool descriptions:

```typescript
{
  name: 'query',
  description: `
Semantic code search across the indexed codebase.
Returns symbols grouped by process (execution flow).

WHEN TO USE:
- Finding relevant code for a task
- Understanding feature implementation
- Exploring unfamiliar codebase

WHEN THIS TOOL IS USEFUL:
- "How is authentication implemented?"
- "Where is user data validated?"
- "Find code related to payment processing"

AFTER THIS TOOL:
- Use 'context' for detailed symbol info
- Use 'impact' to understand change effects
`.trim()
}
```

**Learnings for Parseltongue:**
- Structured descriptions help LLMs choose correctly
- `WHEN TO USE` / `AFTER THIS` guidance reduces tool confusion
- Examples embedded in description improve selection accuracy

### 4.2 Tool Implementations

| Tool | Purpose | Key Features |
|------|---------|--------------|
| `list_repos` | List indexed repos | Auto-refresh registry |
| `query` | Semantic search | Process-grouped, RRF fusion |
| `cypher` | Raw Cypher queries | Read-only, markdown tables |
| `context` | Symbol detail | Disambiguation, categorized refs |
| `impact` | Blast radius | Depth grouping, risk scoring |
| `detect_changes` | Git diff analysis | Affected processes |
| `rename` | Multi-file rename | Graph + text search, dry-run |

---

## 5. What Parseltongue Should ADOPT

### 5.1 Multi-Repo Registry Pattern

```typescript
// Global registry at ~/.gitnexus/repos.json
interface RegistryEntry {
  name: string;
  path: string;
  storagePath: string;
  indexedAt: string;
  lastCommit: string;
  stats: {
    files: number;
    nodes: number;
    communities: number;
    processes: number;
  };
}
```

**Parseltongue Adaptation:**
- Store registry at `~/.parseltongue/repos.json`
- Include rust-analyzer specific metadata (crate count, feature flags)
- Lazy loading of graph connections

### 5.2 Process-Grouped Search Results

Instead of flat symbol lists, group by execution flow:

```json
{
  "processes": [
    {
      "summary": "HandleRequest → ProcessBody → ValidateAuth → FetchData → ReturnResponse",
      "symbol_count": 12,
      "priority": 0.85
    }
  ],
  "symbols": [...]
}
```

**Parseltongue Enhancement:**
- Use MIR control flow for more accurate traces
- Include type information at each step
- Show data flow (what data is passed between steps)

### 5.3 Impact Analysis with Depth Grouping

```json
{
  "target": { "name": "UserRepository", "type": "Struct" },
  "impactedCount": 47,
  "risk": "HIGH",
  "summary": {
    "direct": 12,
    "processes_affected": 3,
    "modules_affected": 2
  },
  "byDepth": {
    "1": [ /* direct callers/callees */ ],
    "2": [ /* transitive */ ],
    "3": [ /* deep transitive */ ]
  }
}
```

### 5.4 Tool Description Format

Adopt the structured description pattern:

```typescript
{
  name: 'trace',
  description: `
Trace data flow through the codebase starting from a symbol.

WHEN TO USE:
- Understanding how data moves through the system
- Finding where a value originates or is consumed
- Debugging data transformation pipelines

EXAMPLES:
- "Trace where user input is validated"
- "How does the config value reach this function?"

AFTER THIS TOOL:
- Use 'context' for symbol details
- Use 'impact' to understand downstream effects
`
}
```

### 5.5 Confidence Scoring on Edges

```typescript
interface GraphRelationship {
  confidence: number;  // 0-1
  reason: string;      // 'import-resolved', 'same-file', 'fuzzy-global'
}
```

**Parseltongue Enhancement:**
- rust-analyzer provides deterministic resolution → confidence always 1.0 for resolved edges
- Use confidence for edges that cross crate boundaries (external deps)
- Track resolution method in `reason` field

### 5.6 Entry Point Scoring Heuristics

GitNexus's scoring system is well-designed:

```typescript
// Call ratio base
baseScore = calleeCount / (callerCount + 1);

// Export bonus
exportMultiplier = isExported ? 2.0 : 1.0;

// Pattern bonus/penalty
nameMultiplier = matchesEntryPointPattern ? 1.5 : 
                 matchesUtilityPattern ? 0.3 : 1.0;

// Framework bonus
frameworkMultiplier = detectFramework(filePath);
```

**Parseltongue Rust-Specific Patterns:**
```rust
// Rust entry point patterns
let rust_entry_patterns = [
    "main",           // Binary entry
    "run",            // Common runner
    "handle_",        // Handler convention
    "execute",        // Command pattern
    "process_",       // Processor pattern
    "from_request",   // Actix/axum extractors
    "IntoResponse",   // Tower services
];

// Rust utility patterns (penalize)
let rust_utility_patterns = [
    "new",            // Constructor
    "default",        // Default trait
    "clone",          // Clone trait
    "as_",            // Conversion
    "into_",          // Into trait
    "try_from",       // TryFrom trait
];
```

---

## 6. What Parseltongue Should INNOVATE On

### 6.1 Semantic Depth (rust-analyzer vs tree-sitter)

| Capability | Tree-sitter (GitNexus) | rust-analyzer (Parseltongue) |
|------------|------------------------|------------------------------|
| Function calls | Name matching | IDE-grade resolution |
| Type inference | None | Full inference |
| Generics | String-based | Monomorphization |
| Traits | Syntax only | Resolution to impls |
| Macros | Text expansion | Hygienic expansion |
| Cross-crate | Fuzzy | Precise (Cargo.lock) |
| Data flow | None | MIR analysis |

**Example: Resolving a method call**

Tree-sitter (GitNexus):
```
CALLS: user.validate() → ??? (can't resolve without type info)
```

rust-analyzer (Parseltongue):
```
CALLS: user.validate() → User::validate (impl Validate for User)
RESOLVED_TYPE: user: User
TRAIT: Validate
```

### 6.2 MIR-Based Control/Data Flow

GitNexus traces via CALLS edges only. Parseltongue can use MIR:

```rust
// From rust-analyzer's MIR
pub enum Terminator {
    Goto { target: BasicBlockId },
    SwitchInt { discr: Operand, targets: SwitchTargets },
    Return,
    Unreachable,
    Drop { place: Place, target: BasicBlockId },
    Call { func: Operand, args: Vec<Operand>, destination: Place },
    Assert { cond: Operand, target: BasicBlockId },
}

// Data flow via Place
pub struct Place {
    local: LocalId,
    projection: Vec<ProjectionElem>,  // Field access, indexing, etc.
}
```

**Parseltongue Process Tracing Enhancement:**
```json
{
  "process_id": "proc_auth_flow",
  "steps": [
    {
      "symbol": "handle_login",
      "step": 1,
      "control_flow": "Entry",
      "data_in": ["credentials: Credentials"],
      "data_out": ["user: Option<User>"]
    },
    {
      "symbol": "validate_credentials",
      "step": 2,
      "control_flow": "Branch { if success → step 3, else → step 5 }",
      "data_in": ["credentials"],
      "data_out": ["is_valid: bool", "user: Option<User>"]
    }
  ]
}
```

### 6.3 Type-Based Taint Analysis

```rust
// Track how sensitive types flow through the system
struct TaintAnalysis {
    source_types: Vec<Ty>,  // User input, file read, network
    sink_types: Vec<Ty>,    // Database write, network send, file write
    sanitizers: Vec<FnDef>, // Validators, encoders
}

// Result
struct TaintPath {
    source: Symbol,
    path: Vec<(Symbol, Ty)>,
    sink: Symbol,
    is_sanitized: bool,
}
```

### 6.4 Crate-Aware Scoping

GitNexus treats all files equally. Parseltongue can use Cargo structure:

```json
{
  "crate_graph": {
    "my-crate": {
      "dependencies": ["dep-a", "dep-b"],
      "features": ["feature-1"],
      "targets": ["lib", "bin"],
      "public_api": ["MyStruct", "public_function"],
      "internal_modules": ["private_module"]
    }
  }
}
```

**UX Enhancement:**
- "Show me the public API of this crate" → Filter to exported items
- "Where is this internal function called from outside?" → Cross-crate boundary detection
- "What features affect this code path?" → Feature-gated path tracing

### 6.5 Option Cards with Disambiguation

GitNexus returns ambiguous candidates. Parseltongue can provide rich disambiguation:

```json
{
  "status": "ambiguous",
  "message": "Found 3 symbols matching 'process'",
  "options": [
    {
      "id": "Function:src/parser.rs:process",
      "kind": "Function",
      "signature": "fn process(input: &str) -> Result<Token>",
      "module": "parser",
      "visibility": "pub(crate)",
      "callers": 12,
      "when_to_choose": "Use for token processing in the parser pipeline"
    },
    {
      "id": "Function:src/queue.rs:process",
      "kind": "Function",
      "signature": "async fn process(&mut self, job: Job) -> Outcome",
      "module": "queue::worker",
      "visibility": "pub",
      "callers": 3,
      "when_to_choose": "Use for job queue processing"
    },
    {
      "id": "Struct:src/process.rs:Process",
      "kind": "Struct",
      "signature": "pub struct Process { pid: u32, ... }",
      "module": "process",
      "visibility": "pub",
      "callers": 8,
      "when_to_choose": "Use for OS process management"
    }
  ]
}
```

### 6.6 Incremental Indexing

GitNexus re-parses on changes. Parseltongue can leverage rust-analyzer's incremental compilation:

```rust
struct IncrementalUpdate {
    changed_files: Vec<PathBuf>,
    affected_symbols: Vec<Symbol>,  // Symbols that need re-resolution
    affected_edges: Vec<Edge>,      // Call graph edges to update
    cached_analysis: HashMap<FileId, Analysis>,  // Unchanged files
}
```

---

## 7. Gaps in GitNexus That Parseltongue Can Fill

### 7.1 No Type Information

GitNexus stores function names but not signatures:
```json
// GitNexus
{ "name": "process", "type": "Function" }

// Parseltongue should include
{
  "name": "process",
  "signature": "fn process<T: Into<String>>(input: T) -> Result<Vec<Token>>",
  "generics": ["T: Into<String>"],
  "return_type": "Result<Vec<Token>>",
  "visibility": "pub(crate)"
}
```

### 7.2 No Data Flow

GitNexus only traces calls, not data:
```
// GitNexus can show: A calls B
// Parseltongue should show: A calls B with data X, B transforms X to Y
```

### 7.3 No Macro Expansion

Tree-sitter sees macro invocations as text:
```rust
// Tree-sitter sees: "some_macro!(...)"
// rust-analyzer sees: expanded AST with resolved types
```

### 7.4 No Cross-Crate Precision

GitNexus uses fuzzy matching for external dependencies:
```
// GitNexus: "probably calls external::some_function"
// Parseltongue: "definitely calls dep_crate::Module::function (v1.2.3)"
```

### 7.5 No Ownership/Borrowing Information

Critical for Rust:
```rust
// Parseltongue should track
struct OwnershipInfo {
    ownership: Ownership,  // Owned, Borrowed, MutBorrowed
    lifetime: Option<Lifetime>,
    moved_at: Option<Location>,
    dropped_at: Option<Location>,
}
```

---

## 8. Recommended Parseltongue Architecture

Based on GitNexus learnings + rust-analyzer capabilities:

```
┌─────────────────────────────────────────────────────────────────┐
│                     MCP Tool Layer (7 tools)                    │
│  trace, context, impact, search, disambiguate, explain, rename │
├─────────────────────────────────────────────────────────────────┤
│                   Semantic Context Compressor                   │
│  - Token budgeting (10k hard limit)                            │
│  - Signature extraction (ra_ap_hir)                            │
│  - Process grouping (CALLS + MIR flow)                        │
│  - Option card generation                                      │
├──────────────────────┬──────────────────────────────────────────┤
│   Graph Operations   │         Query Layer                      │
│  - Custom Rust graph │  - BM25 FTS (Tantivy?)                  │
│  - Leiden community  │  - Semantic (FastEmbed?)                │
│  - Process detection │  - RRF fusion                           │
│  - Impact analysis   │  - Compiler-aware ranking               │
├──────────────────────┴──────────────────────────────────────────┤
│                   rust-analyzer Integration                     │
│  - ra_ap_ide (IDE features)                                    │
│  - ra_ap_hir (HIR types)                                       │
│  - ra_ap_hir_def (definitions)                                 │
│  - ra_ap_mir (data flow)                                       │
├─────────────────────────────────────────────────────────────────┤
│                      Storage Layer                              │
│  - Multi-repo registry (~/.parseltongue/repos.json)           │
│  - Graph storage (sled? custom?)                               │
│  - Embedding cache                                             │
└─────────────────────────────────────────────────────────────────┘
```

---

## 9. Implementation Priority

### Phase 1: Core (Weeks 1-4)
1. ra_ap_* integration for entity extraction
2. Basic graph storage with confidence edges
3. MCP tools: `context`, `search`, `disambiguate`

### Phase 2: Intelligence (Weeks 5-8)
4. Process detection (entry point scoring + BFS)
5. Leiden community detection
6. Impact analysis with depth grouping
7. MCP tools: `trace`, `impact`

### Phase 3: Rust-Specific (Weeks 9-12)
8. MIR data flow integration
9. Type-based taint analysis
10. Ownership tracking
11. MCP tools: `explain`, advanced `trace`

### Phase 4: UX Polish (Weeks 13-16)
12. Token-aware context compression
13. Option card generation
14. Incremental indexing
15. Performance optimization

---

## 10. Key Code References

### Entry Point Scoring
- File: `gitnexus/src/core/ingestion/entry-point-scoring.ts`
- Key function: `calculateEntryPointScore()`
- Patterns: `ENTRY_POINT_PATTERNS`, `UTILITY_PATTERNS`

### Process Detection
- File: `gitnexus/src/core/ingestion/process-processor.ts`
- Key function: `processProcesses()`
- Config: `maxTraceDepth: 10, maxBranching: 4, minSteps: 3`

### Community Detection
- File: `gitnexus/src/core/ingestion/community-processor.ts`
- Algorithm: Leiden via graphology
- Optimization: Large graph filtering (>10K symbols)

### Hybrid Search
- File: `gitnexus/src/mcp/local/local-backend.ts`
- Methods: `bm25Search()`, `semanticSearch()`, RRF in `query()`

### Impact Analysis
- File: `gitnexus/src/mcp/local/local-backend.ts`
- Method: `impact()`
- Features: Depth grouping, risk scoring, process/module enrichment

### Tool Descriptions
- File: `gitnexus/src/mcp/tools.ts`
- Format: `WHEN TO USE`, `WHEN THIS TOOL IS USEFUL`, `AFTER THIS TOOL`

---

## 11. Conclusion

GitNexus provides an excellent reference architecture for:
- Multi-repo MCP integration
- Process-grouped search results
- Impact analysis UX
- Tool description patterns

Parseltongue's key differentiator is **semantic depth via rust-analyzer**:
- Deterministic symbol resolution (no fuzzy matching)
- Type inference for precise call graphs
- MIR for data flow analysis
- Ownership tracking for Rust-specific insights

The combination of GitNexus's UX patterns + rust-analyzer's semantic power = a uniquely capable Rust code companion.
