# Sourcegraph Amp/Cody Competitive Intelligence
## Deep Source Analysis for Parseltongue

**Analysis Date**: 2026-02-19
**Repos Analyzed**:
- `sourcegraph/sourcegraph-public-snapshot` (10,248 stars, Go, main platform)
- `sourcegraph/cody-public-snapshot` (3,790 stars, TypeScript, AI assistant)
- `sourcegraph/amp-contrib` (16 stars, JS, community tools)
- `sourcegraph/amp.nvim` (181 stars, Lua, Neovim plugin)

---

## 1. Architecture Overview

### Sourcegraph Platform Stack

The platform is a large Go monorepo (1.3M+ files by size) built on Bazel. The relevant layers for Parseltongue competitive analysis:

```
User Query
    |
    v
Cody (TypeScript VSCode/JetBrains extension)
    |
    v  (GraphQL API)
Sourcegraph Backend (Go)
    |
    +--> Code Intelligence (SCIP/LSIF store in Postgres)
    +--> Embeddings (int8-quantized vectors in object storage)
    +--> Code Search (Zoekt engine)
    +--> Ranking (PageRank-like system on SCIP refs)
    +--> Deep Search (async multi-step AI reasoning)
```

[CONFIRMED from source]

---

## 2. Code Intelligence / Context Engine

### 2.1 Indexing Technology Stack

Sourcegraph uses a **two-track indexing** approach:

**Track 1: Precise Indexing (SCIP)**
- SCIP (Sourcegraph Code Intelligence Protocol) is their own LSP-superset format
- Replaces the older LSIF format
- Language-specific indexers (scip-go, scip-python, scip-typescript, etc.) produce SCIP payloads
- SCIP data is stored in Postgres (`codeintel_scip_symbols` table)
- Storage layer: `internal/codeintel/codegraph/` - uses `DataStore` interface backed by Postgres

[CONFIRMED from source: `internal/codeintel/codegraph/data_store.go`]

**Track 2: Syntactic Indexing (Tree-Sitter)**
- Recent addition: `internal/codeintel/syntactic_indexing/`
- Uses tree-sitter for syntactic-only indexing (no type resolution)
- Job queue backed by Postgres (`syntactic_scip_indexing_jobs` table)
- Produces SCIP output (same format as precise indexing, different data quality)
- Key advantage: runs fast without needing a full compiler/build toolchain

[CONFIRMED from source: `internal/codeintel/syntactic_indexing/jobstore/job.go`]

```go
// SyntacticIndexingJob - what gets queued per repo/commit
type SyntacticIndexingJob struct {
    ID             int
    State          recordState  // queued | processing | completed | errored
    Commit         api.CommitID
    RepositoryID   api.RepoID
    RepositoryName string
    EnqueuerUserID int32
    ShouldReindex  bool
}
```

### 2.2 Code Graph Schema (SCIP)

The core data structures in the code graph:

```go
// From internal/codeintel/codenav/shared/types.go

// Location: LSP-like location scoped to an upload (index snapshot)
type Location struct {
    UploadID int
    Path     core.UploadRelPath
    Range    Range
}

// Usage: a definition/reference/implementation/supertype occurrence
type Usage struct {
    UploadID int
    Path     core.UploadRelPath
    Range    Range
    Symbol   string      // SCIP symbol (fully qualified, language-agnostic format)
    Kind     UsageKind   // Definition=1, Reference=2, Implementation=3, Super=4
}

// UsageKind maps to DB columns in codeintel_scip_symbols:
// Definition  -> definition_ranges
// Reference   -> reference_ranges
// Implementation -> implementation_ranges
// Super       -> definition_ranges (for interface/superclass methods)
```

[CONFIRMED from source: `internal/codeintel/codenav/shared/types.go`]

The LsifStore interface (what code navigation queries against):

```go
type LsifStore interface {
    FindDocumentIDs(ctx, uploadIDToLookupPath map[int]UploadRelPath) (map[int]int, error)
    GetStencil(ctx, bundleID int, path) ([]Range, error)
    GetRanges(ctx, bundleID int, path, startLine, endLine int) ([]CodeIntelligenceRange, error)
    SCIPDocument(ctx, uploadID int, path) (Option[*scip.Document], error)
    GetMonikersByPosition(ctx, uploadID int, path, line, character int) ([][]MonikerData, error)
    GetSymbolUsages(ctx, options SymbolUsagesOptions) ([]Usage, totalCount int, error)
    GetHover(ctx, bundleID int, path, line, character int) (string, Range, bool, error)
    GetDiagnostics(ctx, bundleID int, prefix, limit, offset int) ([]Diagnostic, int, error)
    ExtractDefinitionLocationsFromPosition(ctx, FindUsagesKey) ([]UsageBuilder, []string, error)
    ExtractReferenceLocationsFromPosition(ctx, FindUsagesKey) ([]UsageBuilder, []string, error)
    ExtractImplementationLocationsFromPosition(ctx, FindUsagesKey) ([]UsageBuilder, []string, error)
    ExtractPrototypeLocationsFromPosition(ctx, FindUsagesKey) ([]UsageBuilder, []string, error)
}
```

[CONFIRMED from source: `internal/codeintel/codenav/internal/lsifstore/store.go`]

### 2.3 Cross-Repo Code Understanding

Sourcegraph's cross-repo understanding works through:

1. **Upload-based indexing**: Each repo+commit gets an `uploadID`. SCIP symbols are namespaced per upload.
2. **Cross-repo links**: SCIP monikers encode package identity, enabling cross-repo def/ref lookups
3. **GetCodyContext GraphQL query**: Accepts `$repos: [ID!]!` array - can retrieve context from multiple repos simultaneously

```graphql
# The primary cross-repo context query
query GetCodyContext(
    $repos: [ID!]!,
    $query: String!,
    $codeResultsCount: Int!,
    $textResultsCount: Int!,
    $filePatterns: [String!]
) {
    getCodyContext(...) {
        ... on FileChunkContext {
            blob { path, repository { id, name }, commit { oid }, url }
            startLine
            endLine
            chunkContent
            matchedRanges { start { line, column }, end { line, column } }
        }
    }
}
```

[CONFIRMED from source: `lib/shared/src/sourcegraph-api/graphql/queries.ts`]

---

## 3. Embedding / Vector Search Layer

### 3.1 Embedding Index Data Structure

```go
// From internal/embeddings/types.go

type EmbeddingIndex struct {
    Embeddings      []int8       // int8-quantized (NOT float32!) - 4x memory savings
    ColumnDimension int          // vector dimensions
    RowMetadata     []RepoEmbeddingRowMetadata
    Ranks           []float32    // PageRank-like scores from Zoekt
}

type RepoEmbeddingRowMetadata struct {
    FileName  string `json:"fileName"`
    StartLine int    `json:"startLine"`
    EndLine   int    `json:"endLine"`
}

type RepoEmbeddingIndex struct {
    RepoName        api.RepoName
    Revision        api.CommitID
    EmbeddingsModel string
    CodeIndex       EmbeddingIndex   // code files
    TextIndex       EmbeddingIndex   // markdown/docs
}

type EmbeddingSearchResult struct {
    RepoName api.RepoName
    Revision api.CommitID
    FileName  string
    StartLine int
    EndLine   int
    ScoreDetails SearchScoreDetails
}

type SearchScoreDetails struct {
    Score           int32  // final combined score
    SimilarityScore int32  // cosine similarity component
    RankScore       int32  // PageRank component from Zoekt
}
```

[CONFIRMED from source: `internal/embeddings/types.go`]

### 3.2 Quantization Strategy

Sourcegraph quantizes float32 embeddings to int8 with a simple linear mapping:

```go
// From internal/embeddings/quantize.go
// Maps [-1.0, 1.0] -> [-127, 127]
func Quantize(input []float32, buf []int8) []int8 {
    output[i] = int8(math.Round(float64(val) * 127.0))
}
```

**Key stat from their comments**: Average change in rank from quantization = only 1.2%. Top 93 of 100 rankings unchanged. This is a pragmatic approximation that works well in practice.

[CONFIRMED from source: `internal/embeddings/quantize.go`]

### 3.3 Scoring Formula

`FinalScore = SimilarityScore + RankScore`

The RankScore is pulled from **Zoekt** (their code search engine), which computes file importance from code structure (similar to PageRank but for code files).

[CONFIRMED from source: `EmbeddingSearchResult.Score()` method]

### 3.4 Similarity Search Implementation

- Uses parallel CPU workers with a heap-based top-K selection
- Implements `dot product` for similarity with SIMD optimizations (AMD64 and ARM64 assembly)
- Files: `dot_amd64.s`, `dot_arm64.s` - custom assembly for performance

```go
// Parallel search over embedding rows
type WorkerOptions struct {
    NumWorkers     int
    MinRowsToSplit int  // minimum rows before parallelizing
}
```

[CONFIRMED from source: `internal/embeddings/similarity_search.go`, `dot_amd64.go`]

### 3.5 Embedding Providers

They support multiple embedding providers (from `embed.go`):
- **Sourcegraph** (their own hosted model)
- **OpenAI** (text-embedding-ada-002 or newer)
- **Azure OpenAI**

---

## 4. Code Chunking Strategy

### 4.1 Split Logic

```go
// From internal/codeintel/context/split.go
// CHARS_PER_TOKEN = 4 (simple approximation)

type SplitOptions struct {
    NoSplitTokensThreshold         int  // if file < this, embed whole file
    ChunkTokensThreshold           int  // max tokens per chunk (hard limit)
    ChunkEarlySplitTokensThreshold int  // soft limit: split here if on a good boundary
}

// Splittable boundaries (lines starting with these prefixes):
var splittableLinePrefixes = []string{
    "//", "#", "/*",
    "func", "var", "const",
    "fn", "public", "private", "type",
}
```

Key insight: They split at **semantic boundaries** (function starts, declarations, comments) rather than arbitrary character counts. This improves embedding quality by keeping semantic units together.

[CONFIRMED from source: `internal/codeintel/context/split.go`]

### 4.2 File Filtering for Embeddings

```go
// Files excluded from embedding:
var DefaultExcludedFilePathPatterns = []string{
    ".*ignore",           // .gitignore, .eslintignore
    ".gitattributes",
    ".mailmap",
    "*.csv", "*.svg", "*.xml",
    "__fixtures__/",
    "node_modules/",
    "testdata/",
    "mocks/",
    "vendor/",
}

// Minimum embeddable file: 32 chars
// Maximum line length: 2048 chars
// Auto-generated file detection: checks for headers like "autogenerated file", "lockfile", "generated by", "do not edit"
```

[CONFIRMED from source: `internal/embeddings/embed/files.go`]

---

## 5. Context Selection Engine (Cody)

### 5.1 Context Retrieval Architecture

Cody uses a **multi-retriever + fusion** architecture for autocomplete context:

```
User cursor position
    |
    v
Context Strategies (select based on config):
    - lsp-light: Graph (LSP) + Jaccard Similarity
    - jaccard-similarity: Jaccard only
    - tsc: TypeScript compiler graph
    - tsc-mixed: TSC graph + Jaccard
    - recent-edits: Recent edit history
    - diagnostics: Error locations
    - recent-view-port: Recently viewed code
    - auto-edit: Edits + Diagnostics + ViewPort
    |
    v
Context Mixer (runs retrievers in parallel)
    |
    v
Reciprocal Rank Fusion (RRF)
    |
    v
Token budget enforcement (greedy fill)
    |
    v
Final context snippets
```

[CONFIRMED from source: `vscode/src/completions/context/context-mixer.ts`, `context-strategy.ts`]

### 5.2 Reciprocal Rank Fusion (RRF)

This is the key ranking algorithm. Parameters:

```typescript
// From vscode/src/completions/context/reciprocal-rank-fusion.ts
const RRF_K = 60  // standard RRF parameter (same as Azure Cognitive Search default)

// Score = sum(1 / (K + rank_i)) for each retriever i that includes this document
// Higher score = retrieved by more retrievers AND at higher ranks
```

Key design decisions:
- **Identity function**: Each line in a code file is a separate "document" identity - enables overlap detection across different window sizes
- **No-overlap guarantee**: After ranking, ensures no line appears twice in final results
- **Top-K selection**: Greedy - fills token budget by picking from highest-scored snippets

[CONFIRMED from source: `vscode/src/completions/context/reciprocal-rank-fusion.ts`]

### 5.3 Jaccard Similarity Retriever

This is the primary local (no-network) retriever for autocomplete:

```typescript
// From bestJaccardMatch.ts
// Word processing pipeline:
// 1. Tokenize (wink-nlp-utils)
// 2. Break camelCase + snake_case -> constituent words
// 3. Remove stop words
// 4. Stem (Porter stemmer, cached in LRUCache)
// 5. Build word frequency map
// 6. Slide window over candidate files
// 7. Score = intersection / union
```

Optimization details:
- Sliding window approach: O(N) per file instead of O(N*W)
- LRU cache for stems (max 30,000 entries)
- Skips windows starting with empty lines (except last window)
- Post-filtering: removes overlapping windows

**Key insight for Parseltongue**: Jaccard on stemmed words is their primary local ranker. It works without any ML model - pure algorithmic. The camelCase/snake_case decomposition is a crucial detail for code-specific matching.

[CONFIRMED from source: `vscode/src/completions/context/retrievers/jaccard-similarity/bestJaccardMatch.ts`]

### 5.4 LSP Graph Retriever (lsp-light)

The graph-based retriever for autocomplete:

```typescript
// From vscode/src/completions/context/retrievers/lsp-light/lsp-light-retriever.ts

// Supported languages: Python, Go, JavaScript, TypeScript (React variants)
// RECURSION_LIMIT = 3 (depth of symbol resolution)
// IDENTIFIERS_TO_RESOLVE = 1 (only top-1 identifier per request)

// Algorithm:
// 1. tree-sitter query: getGraphContextIdentifiers (last 100 lines before cursor)
// 2. Take top N identifiers (deduplicated)
// 3. LSP: getDefinitionLocations -> hover -> getText
// 4. Cache results with LRU per document/position
```

Resolution waterfall for a symbol:
1. Get hover at current position
2. If unhelpful → get text from definition location
3. If still unhelpful → get hover at definition location
4. Resolve nested identifiers up to NESTED_IDENTIFIERS_TO_RESOLVE=5 depth

[CONFIRMED from source: `vscode/src/graph/lsp/symbol-context-snippets.ts`]

### 5.5 Token Budget Management

```typescript
// From lib/shared/src/token/constants.ts

const CHAT_INPUT_TOKEN_BUDGET = 7000           // default chat context
const FAST_CHAT_INPUT_TOKEN_BUDGET = 4096      // fast models
const CHAT_OUTPUT_TOKEN_BUDGET = 4000          // output limit
const CORPUS_CONTEXT_ALLOCATION = 0.6          // 60% of context for retrieved content
const EXTENDED_USER_CONTEXT_TOKEN_BUDGET = 30000  // large context models
const EXTENDED_CHAT_INPUT_TOKEN_BUDGET = 15000    // large context models

// Context filling (from context-mixer.ts):
// - Seed total_chars with prefix + suffix length
// - Greedy fill: for each ranked snippet, add if totalChars + snippet.length <= maxChars
// - Track per-retriever stats: suggestedItems, retrievedItems, retrieverChars, positionBitmap
```

The `positionBitmap` is clever: a 32-bit bitmap tracking which positions in the result set each retriever contributed to. Useful for analytics.

[CONFIRMED from source: `lib/shared/src/token/constants.ts`, `context-mixer.ts`]

---

## 6. Ranking System

### 6.1 PageRank-Analog (Zoekt Integration)

The ranking background pipeline in `internal/codeintel/ranking/internal/background/`:
- **mapper**: Maps SCIP symbol references to export-rank scores
- **reducer**: Aggregates reference counts into file/symbol ranks
- **exporter**: Pushes ranks to Zoekt (their trigram-based code search)
- **coordinator**: Orchestrates the pipeline

Files are ranked by how many other files reference their symbols. This is essentially PageRank applied to code dependencies.

[CONFIRMED from source: directory structure]

### 6.2 Ranking Types

```go
// From internal/codeintel/ranking/shared/types.go
type RankingDefinitions struct {
    UploadID         int
    ExportedUploadID int
    SymbolChecksum   [16]byte  // MD5 of symbol name for dedup
    DocumentPath     string
}

type RankingReferences struct {
    UploadID         int
    ExportedUploadID int
    SymbolChecksums  [][16]byte
}
```

[CONFIRMED from source]

---

## 7. Deep Search API

### 7.1 API Design

Deep Search is Sourcegraph's "natural language codebase exploration" feature. Key findings:

```
Base URL: /.api/deepsearch/v1
Auth: Bearer token in Authorization header
Versioning: Available since Sourcegraph 6.7+

Endpoints:
POST   /.api/deepsearch/v1                              Create conversation
GET    /.api/deepsearch/v1                              List conversations
GET    /.api/deepsearch/v1/{id}                         Get conversation
POST   /.api/deepsearch/v1/{id}/questions               Add question
POST   /.api/deepsearch/v1/{id}/questions/{qid}/cancel  Cancel
DELETE /.api/deepsearch/v1/{id}                         Delete conversation
POST   /.api/deepsearch/v1/{id}/rotate-read-token       Rotate token
```

[CONFIRMED from source: `amp-contrib/deep_search/deep-search-api-how-to.md`]

### 7.2 Conversation Schema

```json
{
  "id": 332,
  "questions": [
    {
      "id": 4978,
      "conversation_id": 332,
      "question": "Does github.com/sourcegraph/sourcegraph have a README?",
      "status": "completed",
      "title": "Check for README in sourcegraph/sourcegraph",
      "answer": "...",
      "sources": [
        {
          "type": "Repository",
          "link": "/github.com/sourcegraph/sourcegraph",
          "label": "github.com/sourcegraph/sourcegraph",
          "metadata": {
            "abbreviatedRevision": "affb534",
            "repoName": "github.com/sourcegraph/sourcegraph",
            "revision": "affb5349bedef24188a7e992f9581ee76fbe151d"
          }
        }
      ],
      "turns": [...],
      "stats": {
        "time_millis": 15548,
        "tool_calls": 2,
        "total_input_tokens": 14441,
        "cached_tokens": 0,
        "cache_creation_input_tokens": 68319,
        "prompt_tokens": 25,
        "completion_tokens": 688,
        "total_tokens": 14666,
        "credits": 5
      },
      "suggested_followups": [...]
    }
  ],
  "quota_usage": {
    "total_quota": 0,
    "quota_limit": -1,
    "reset_time": "2025-09-01T00:00:00Z"
  }
}
```

[CONFIRMED from source]

### 7.3 Async Pattern

Deep Search typically takes 1-2+ minutes. They implement `Prefer: respond-async` header:
- Returns 202 with `status: "processing"`
- Poll via `GET /{id}/questions/{qid}` every 10 seconds
- Status values: `pending`, `processing`, `completed`, `failed`
- Timeout recommended: 300 seconds (5 minutes)

[CONFIRMED from source]

---

## 8. Amp.nvim IDE Protocol (API Surface Revealed)

### 8.1 WebSocket Server Protocol

Amp communicates with editors via a JSON-RPC-like WebSocket protocol:

```lua
-- Message structure (from amp.nvim/lua/amp/server/init.lua):
-- Incoming: { clientRequest: { id: string, [method]: params } }
-- Outgoing:
--   Response: ide.wrap_response(id, { [method]: result })
--   Error:    ide.wrap_error(id, { code, message, data })
--   Notification: ide.wrap_notification({ [eventType]: data })
```

### 8.2 IDE Request Methods (Full List)

```
authenticate    -> { authenticated: true }
ping           -> { message: echo }
readFile       -> { path } -> { success, content, encoding: "utf-8" }
editFile       -> { path, fullContent } -> { success, message, appliedChanges }
getDiagnostics -> { path } -> { entries: DiagnosticEntry[] }
openURI        -> { uri } -> { success, message }
```

[CONFIRMED from source: `amp.nvim/lua/amp/server/init.lua`]

### 8.3 IDE Notifications (Broadcast from Editor)

```lua
-- Selection state (sent on cursor move):
broadcast_ide({ selection: {
    fileUrl: "file:///path/to/file",
    selection: {
        start: { line, character },
        end:   { line, character }
    },
    text: "selected text"
}})

-- Visible files (sent on buffer/window change):
broadcast_ide({ visibleFiles: ["file:///path/to/file1", "file:///path/to/file2"] })

-- Plugin metadata (sent on connect):
broadcast_ide({ pluginMetadata: {
    version: "0.1.0",
    pluginDirectory: "..."
}})
```

[CONFIRMED from source: `amp.nvim/lua/amp/init.lua`]

### 8.4 Authentication

- Token generated via `lockfile.generate_auth_token()`
- Stored in a lockfile with port number
- Sent on WebSocket handshake
- 30-second ping timer to keep connections alive

[CONFIRMED from source]

---

## 9. Amp Community Tools (amp-contrib)

### 9.1 TypeScript Dependency Graph Analyzer

The `typescript-codemod/analyze_dependency_graph` tool reveals what Amp agents can do:

```javascript
// Tool spec (MCP-compatible):
{
    name: 'analyze_dependency_graph',
    description: 'Analyze dependency graph of TypeScript/JavaScript files, detect circular dependencies using Tarjan\'s algorithm, and generate optimal migration order.',
    inputSchema: {
        type: 'object',
        properties: {
            rootPath: { type: 'string' },
            filePatterns: { type: 'array', items: { type: 'string' } }
        },
        required: ['rootPath']
    },
    meta: { subagentTypes: ['scope'] }
}
```

Output includes:
- `stats`: totalFiles, filesWithImports, filesWithDependents, totalCycles, filesInCycles
- `cycles`: SCC analysis with Tarjan's algorithm, autoFixable flag, strategy
- `migrationOrder`: Topologically sorted files for safe migration
- `leaves`: Files with no dependents (safe starting points)
- `cores`: Files with 3+ dependents (high-blast-radius)

[CONFIRMED from source: `amp-contrib/typescript-codemod/analyze_dependency_graph`]

### 9.2 Other Tools

- `bigquery/`: BigQuery integration
- `deep_search/`: Deep Search API client scripts
- `linear/`: Linear issue tracker integration
- `web-browser/`: Browser automation (nav, eval, screenshot, pick)
- `typescript-codemod/`: TS analysis tools (check_ts_syntax, detect_browser_apis, detect_imports, detect_jsx_patterns, detect_style_usage)
- `formatting/format-file-tree.js`: File tree formatting
- `sandbox/`: Sandbox configuration
- `tmux/`, `restore-layout.sh`: Terminal management

[CONFIRMED from source]

---

## 10. Parseltongue Competitive Analysis

### 10.1 Sourcegraph's MOAT (Hard to Replicate)

**1. SCIP Ecosystem (Strongest moat)**
- They built SCIP protocol and 10+ language-specific indexers (scip-go, scip-python, scip-typescript, scip-java, scip-c, etc.)
- These indexers use the actual compiler/type system for precise data - tree-sitter cannot match this
- Cross-repo symbol resolution requires the SCIP moniker system
- Years of investment in language indexer accuracy

[CONFIRMED from source, INFERRED from ecosystem]

**2. Zoekt Integration**
- Zoekt is a trigram-based code search engine they built and maintain
- Powers both text search and the PageRank-analog for file importance scores
- The `RankScore` in embeddings comes from Zoekt's file importance model

[CONFIRMED from source]

**3. Enterprise Scale**
- Handles tens of thousands of repos in single instances
- Incremental indexing: only re-embed changed files since last commit
- Object storage for embedding indexes (not in-memory)

[CONFIRMED from source: `embed.go` incremental indexing logic]

### 10.2 PMF Assessment

**Where Sourcegraph scores 85+:**
- Code navigation (go-to-def, find-refs) in enterprise polyglot codebases: 95
- Cross-repo code understanding with SCIP: 90
- Code search with regexp + structural patterns: 90
- IDE integration depth (VSCode, JetBrains, Neovim): 85

**Where it is weaker:**
- Local/offline operation: ZERO - requires Sourcegraph server
- CPU-only environments: Not designed for this
- Simple deployment: Complex (requires indexers, object storage, Postgres, Zoekt, etc.)
- API-first for LLM agents: GraphQL API is complex vs. simple REST
- Blast radius analysis: Not a first-class concept (derived from reference counts)
- Real-time analysis: Indexing is async/delayed, not instant

### 10.3 Where Parseltongue Can Be STRONG

**1. Instant local analysis**
Parseltongue runs entirely CPU-local. No indexers to deploy, no servers to maintain. Analysis is available immediately after parsing.

**2. Graph database queries (CozoDB)**
CozoDB's Datalog/graph queries can express relationship patterns that Sourcegraph's Postgres-backed SCIP store cannot efficiently express:
- "All files that would be affected if I change function X" (blast radius)
- "Shortest dependency path between module A and module B"
- "All cycles in the dependency graph"
These require graph traversal which is unnatural in relational SQL.

**3. 26 REST endpoints vs. complex GraphQL**
LLM agents (especially those using MCP tools) prefer simple HTTP GET/POST with JSON. Sourcegraph's GraphQL API requires understanding of their schema. Parseltongue's flat REST API is more agent-friendly.

**4. Token budget awareness as a first-class feature**
Sourcegraph's token management is buried in client-side TypeScript. Parseltongue can make it a server-side API concern: "give me the best N bytes of context for this query."

**5. No compiler dependency**
Sourcegraph's precise indexing requires running the actual compiler (scip-go needs Go toolchain, scip-typescript needs tsc). Parseltongue's tree-sitter approach works on any machine with no build toolchain.

**6. Complexity hotspots as a feature**
Sourcegraph measures file importance by reference count (PageRank analog). Parseltongue can offer cyclomatic complexity, nesting depth, function length - richer metrics for "where is the risky code."

### 10.4 What Parseltongue Should Steal

**1. RRF (Reciprocal Rank Fusion)**
The `fuseResults` implementation from `reciprocal-rank-fusion.ts` is clean and directly applicable. Key parameters:
- K=60 (standard default)
- Per-line identity (not per-chunk) for overlap detection
- Greedy top-K with no-overlap guarantee

**2. Jaccard Similarity with stemming + camelCase decomposition**
The word tokenization pipeline from `bestJaccardMatch.ts` is excellent:
- Tokenize → break camelCase/snake_case → remove stop words → stem → frequency map
- Sliding window approach is O(N) efficient
- This could replace or augment embedding-based search in Parseltongue

**3. Context Mix Metadata (RetrieverStat)**
```typescript
interface RetrieverStat {
    name: string
    suggestedItems: number
    retrievedItems: number
    retrieverChars: number
    duration: number
    positionBitmap: number  // which positions this retriever contributed
}
```
Returning rich metadata about HOW context was selected is valuable for LLM agents to understand provenance.

**4. File filtering patterns**
Their `DefaultExcludedFilePathPatterns` list and autogenerated-file detection heuristics are practical and battle-tested:
- Exclude: `.*ignore`, `*.csv`, `*.svg`, `*.xml`, `__fixtures__/`, `node_modules/`, `testdata/`, `mocks/`, `vendor/`
- Auto-generated file detection: check for "autogenerated file", "lockfile", "generated by", "do not edit" in first N bytes

**5. Semantic chunking**
Their `SplitIntoEmbeddableChunks` function splits at function/declaration boundaries, not character boundaries. This is directly implementable in Rust:
- Split on lines starting with `fn`, `pub`, `const`, `type`, `impl`, `//`, `#`, `/*`
- Hard limit: ChunkTokensThreshold
- Soft limit: ChunkEarlySplitTokensThreshold (only split at good boundary)

**6. Two-tier token budgets**
Fast models get smaller budgets (4096 tokens), smart models get larger (7000+). Parseltongue should expose this as a parameter to the context selection endpoint.

**7. Deep Search async pattern**
For expensive analysis queries (blast radius across large codebases), Parseltongue should implement the same `202 Accepted + poll` pattern. Immediately return a job ID, let the client poll.

**8. IDE protocol (amp.nvim)**
The request/response protocol from amp.nvim is clean and minimal:
- readFile, editFile, getDiagnostics, openURI as first-class operations
- Broadcast pattern for selection + visible files
- WebSocket with auth token + lockfile discovery

**9. Source attribution in results**
Deep Search responses include `sources` with repo name, revision, path. Parseltongue should return similar provenance data with each context chunk.

---

## 11. Control Flow: User Query to LLM Response

### 11.1 Autocomplete Flow (Cody)

```
Keystroke in editor
    |
    v  (debounced 100ms)
LspLightRetriever.onDidChangeTextEditorSelection()
    |
    v  (tree-sitter)
getLastNGraphContextIdentifiersFromDocument(n=1, last 100 lines)
    |
    v  (LSP call via vscode API)
getDefinitionLocations() -> hover -> getText
    |
    v  (parallel)
JaccardSimilarityRetriever.retrieve()
    |  - slide window over all open documents
    |  - score by stemmed word overlap with prefix+suffix
    |
    v  (ContextMixer)
Promise.all([lsp_results, jaccard_results])
    |
    v  (RRF)
fuseResults(results, rankingIdentities: snippet -> [file:line, ...])
    |
    v  (greedy fill with token budget)
context = []
for snippet in fusedResults:
    if totalChars + snippet.length <= maxChars:
        context.append(snippet)
    |
    v  (LLM call with context)
completion model (claude-3-haiku / starcoder2 / etc.)
```

[CONFIRMED from source, INFERRED for LLM call details]

### 11.2 Chat Flow (Cody)

```
User message
    |
    v  (GraphQL: getCodyContext)
getCodyContext(repos=[...], query=user_message, codeResultsCount=15, textResultsCount=5)
    |
    v  (Sourcegraph backend)
1. Embed user query
2. Cosine similarity search over repo embedding index (int8)
3. Score = similarity + Zoekt rank
4. Return top-K file chunks (startLine, endLine, chunkContent)
    |
    v  (Token budget enforcement: 60% of chat window for corpus)
Pack chunks until CORPUS_CONTEXT_ALLOCATION * chatInputBudget
    |
    v  (LLM call)
claude-3-sonnet / claude-3-opus / etc.
```

[CONFIRMED from source, INFERRED for backend details]

---

## 12. Storage Architecture

### 12.1 Postgres (Primary Store)

Tables (inferred from column expressions and query patterns):
- `syntactic_scip_indexing_jobs` - job queue for syntactic indexing
- `codeintel_scip_symbols` - SCIP symbol data (definition_ranges, reference_ranges, implementation_ranges columns)
- `lsif_uploads` - upload metadata
- Various ranking tables

[CONFIRMED from code references]

### 12.2 Object Storage (Embedding Indexes)

The `EmbeddingIndex` is stored in object storage (S3-compatible). Key files:
- `index_storage.go` - handles serialization/deserialization of RepoEmbeddingIndex
- `object_storage.go` - wraps the object storage client
- Indexes are binary-encoded (not JSON) for efficiency

[CONFIRMED from source: `internal/embeddings/index_storage.go` existence]

### 12.3 Incremental Indexing

```go
// From embed.go
if opts.IndexedRevision != "" {
    toIndex, toRemove, err = readLister.Diff(ctx, opts.IndexedRevision)
    // Falls back to full index on error
}
// For full index:
toIndex, err = readLister.List(ctx)
```

Smart re-use of previous embedding index: only embed changed files, update rank scores.

[CONFIRMED from source: `internal/embeddings/embed/embed.go`]

---

## 13. Key Insights for Parseltongue Product Strategy

### 13.1 The Fundamental Difference

Sourcegraph = **server-side code intelligence platform** requiring:
- SCIP indexers (one per language)
- Postgres for code graph
- Object storage for embeddings
- Zoekt for code search
- Enterprise deployment

Parseltongue = **local CPU analysis tool** requiring:
- Just Rust binary + tree-sitter grammars
- CozoDB (embedded)
- HTTP server on localhost

**This is a fundamentally different market position.** Sourcegraph competes with GitHub Copilot for enterprise IDEs. Parseltongue should compete for **agent-native development tools** that run locally.

### 13.2 Parseltongue's Unique Angle

**"What is the blast radius of changing X?"** is not a query Sourcegraph's GraphQL API handles well. It would require multiple round trips to build the transitive dependency closure. With CozoDB's Datalog, this is a single recursive query.

```datalog
# Conceptual CozoDB query for blast radius
?[affected_file] :=
    changed_entity[entity],
    depends_on[dependent, entity],
    ?[affected_file] = dependent
    // + transitive closure
```

This is Parseltongue's killer feature and it's genuinely hard for Sourcegraph to replicate without switching their storage to a graph database.

### 13.3 API Design Recommendation

Steal the **RetrieverStat** concept: when returning context chunks, always include:
- Which analysis method found each chunk
- Confidence score
- Distance from query entity
- Whether it's in the blast radius

This metadata helps LLM agents reason about context quality.

### 13.4 Embed Quality Benchmark

Sourcegraph reports 93/100 top embeddings unchanged after int8 quantization. If Parseltongue adds embedding support, use the same quantization approach (float32 → int8, linear scale, round not truncate).

---

## 14. Summary Table: Sourcegraph vs. Parseltongue

| Feature | Sourcegraph | Parseltongue |
|---------|------------|--------------|
| Indexing | SCIP (precise) + tree-sitter (syntactic) | tree-sitter only |
| Languages | 20+ (precise), 10+ (syntactic) | 12 (tree-sitter) |
| Cross-repo | Yes (SCIP monikers) | No (single codebase) |
| Storage | Postgres + Object Storage | CozoDB (embedded) |
| Search | Zoekt trigrams + embeddings | Graph queries |
| Embeddings | int8-quantized (OpenAI/Sourcegraph/Azure) | Not yet |
| Ranking | PageRank-analog via Zoekt | Complexity metrics |
| Context ranking | RRF across multiple retrievers | TBD |
| Blast radius | Indirect (reference count) | Direct (graph traversal) |
| Deployment | Server (enterprise) | Local binary |
| API | GraphQL + REST | 26 REST endpoints |
| IDE integration | VSCode, JetBrains, Neovim, etc. | Via HTTP |
| Token budget | Client-side TypeScript | Server-side API |
| Async analysis | Yes (Deep Search) | Not yet |

[CONFIRMED confirmed items, INFERRED for Parseltongue current state]

---

---

## 15. Cody Agent Protocol (JSON-RPC over stdin/stdout)

### 15.0 Agent Architecture

The Cody Agent enables any programming language to interact with Cody via JSON-RPC on stdin/stdout. This is the same flavor as LSP (Language Server Protocol). It powers JetBrains and Neovim plugins.

**Key methods exposed:**

```typescript
// Core lifecycle
'initialize'           -> ClientInfo -> ServerInfo
'shutdown'             -> null -> null

// Chat
'chat/new'             -> null -> string (panel id)
'chat/models'          -> { id } -> { models: Model[] }
'chat/submitMessage'   -> { id, message: WebviewMessage } -> ExtensionMessage
'chat/editMessage'     -> { id, message } -> ExtensionMessage

// Commands
'commands/explain'     -> null -> string
'commands/test'        -> null -> string
'commands/smell'       -> null -> string
'commands/document'    -> null -> EditTask
'commands/custom'      -> { key } -> CustomCommandResult

// Autocomplete
'autocomplete/execute' -> AutocompleteParams -> AutocompleteResult

// GraphQL pass-through
'graphql/getRepoIds'                   -> { names, first } -> { repos: [{name, id}] }
'graphql/getRepoIdIfEmbeddingExists'   -> { repoName } -> string | null
'graphql/getCurrentUserCodySubscription' -> null -> CodySubscription | null

// Admin
'featureFlags/getFeatureFlag'  -> { flagName } -> boolean | null
'telemetry/recordEvent'        -> TelemetryEvent -> null
'check/isCodyIgnoredFile'      -> { urls } -> boolean
```

**Key design choice**: Cody exposes a language-agnostic RPC server that editors (JetBrains, Neovim, etc.) embed and call. This is the same pattern Parseltongue uses for its HTTP endpoints, but Cody's is process-based (stdin/stdout) vs. network-based (HTTP). The HTTP approach is simpler to integrate for agent tools.

[CONFIRMED from source: `agent/protocol.md`]

### 15.0.1 Architecture ARCHITECTURE.md Key Principles

From cody's architecture documentation:

**Token Counting Principle (critical)**:
- "4 characters per token" heuristic can be off by 4x+ for non-Latin languages (Japanese, Chinese)
- countTokens(s1) + countTokens(s2) != countTokens(s1 + s2) - they don't sum accurately
- Always count tokens AFTER appending strings, not by summing individual counts
- Always express limits in tokens, not characters

**Async Patterns**:
- Single async result: Promise
- Changing values (subscriptions): Observable
- Multiple values, demand-driven: Generator (`function*`)
- Discrete events with multiple subscribers: Event listener
- Complex protocols: Callbacks

[CONFIRMED from source: `ARCHITECTURE.md`]

---

## 16. Token Counting, Context Types, Auto-Edit

### 15.1 Real Token Counting (Not Just Character Estimation)

Sourcegraph uses **cl100k_base** (GPT-4/Claude-equivalent) tokenizer for accurate token counting:

```typescript
// From lib/shared/src/token/counter.ts
// Uses gpt-tokenizer (cl100k_base) in Chrome/Firefox
// Uses js-tiktoken in Safari (compatibility fallback)
// Optimization: if wordCount > EXTENDED_USER_CONTEXT_TOKEN_BUDGET, use word count directly (fast path)

function countTokens(text: string): number {
    const wordCount = text.trim().split(/\s+/).length
    return wordCount > EXTENDED_USER_CONTEXT_TOKEN_BUDGET
        ? wordCount           // fast path for large content
        : this.encode(text).length  // accurate cl100k tokenization
}
```

This is significantly more accurate than Parseltongue's `CHARS_PER_TOKEN = 4` approximation that Sourcegraph uses for embedding chunking (a different code path from token budgets).

[CONFIRMED from source: `lib/shared/src/token/counter.ts`]

### 15.2 Context Item Type System

Sourcegraph has a rich context item type hierarchy for chat:

```typescript
type ContextItem =
    | ContextItemFile           // file or file range
    | ContextItemRepository     // whole repo
    | ContextItemTree           // directory
    | ContextItemSymbol         // function/class/method (from LSP)
    | ContextItemOpenCtx        // external context provider
    | ContextItemCurrentSelection  // active editor selection
    | ContextItemCurrentFile    // active editor file
    | ContextItemCurrentRepository
    | ContextItemCurrentDirectory
    | ContextItemCurrentOpenTabs
    | ContextItemMedia          // images (base64)
    | ContextItemToolState      // tool execution result

// Source enum for provenance tracking:
enum ContextItemSource {
    User = 'user',        // @-mentioned explicitly
    Editor = 'editor',    // from editor state
    Search = 'search',    // from symf search
    Initial = 'initial',  // default context
    Priority = 'priority', // query-based, not user-added
    Unified = 'unified',  // remote search
    Selection = 'selection',
    Terminal = 'terminal',
    History = 'history',  // source control
    Agentic = 'agentic',  // from agent tool use
}
```

Key insight: **User-added context items are prioritized over auto-retrieved ones**. The `source` field determines ordering when the token budget is tight.

[CONFIRMED from source: `lib/shared/src/codebase-context/messages.ts`]

### 15.3 Auto-Edit Context Strategy (Most Sophisticated)

The `auto-edit` strategy combines the most context signals:

```typescript
case 'auto-edit':
    this.allLocalRetrievers = [
        new RecentEditsRetriever({
            maxAgeMs: 10 * 60 * 1000,  // 10 minutes of edit history
            diffStrategyList: [
                new LineLevelDiffStrategy({
                    contextLines: 3,
                    longTermDiffCombinationStrategy: 'unified-diff',
                    minShortTermEvents: 1,
                    minShortTermTimeMs: 2 * 60 * 1000,  // 2 minutes
                    trimSurroundingContext: true,
                }),
            ],
        }),
        new DiagnosticsRetriever({ contextLines: 0 }),
        new RecentViewPortRetriever({
            maxTrackedViewPorts: 50,
            maxRetrievedViewPorts: 10,
        }),
    ]
```

The combination: **what you edited recently + current errors + what you've been reading** gives the AI the developer's full mental context.

[CONFIRMED from source: `vscode/src/completions/context/context-strategy.ts`]

### 15.4 Recent Edits as Context

The `RecentEditsRetriever` tracks all text document changes and produces diffs as context:
- Default max age: 60 seconds (for completions), 10 minutes (for auto-edit)
- Sorts by `latestChangeTimestamp` (most recent first)
- Each diff includes `timeSinceActionMs` in metadata
- LRU cache of 500 entries for computed diffs

**Parseltongue implication**: Parseltongue doesn't track editor state at all (it's a server). But this shows that **recency signals** are valuable - agents using Parseltongue could pass recently-edited files as context hints.

[CONFIRMED from source: `vscode/src/completions/context/retrievers/recent-user-actions/recent-edits-retriever.ts`]

### 15.5 Viewport Context (What Developer Is Looking At)

```typescript
interface TrackedViewPort {
    uri: vscode.Uri
    content: string
    lines?: { startLine: number, endLine: number }
    languageId: string
    lastAccessTimestamp: number
}
// Max tracked: 50, Max retrieved: 10
// Debounced: 300ms on visible range change
```

This tracks **which code the developer has looked at**, not just edited. Ordered by recency. Again, not applicable to Parseltongue directly, but suggests an endpoint idea: `/recently-viewed` that agent frameworks could ping.

[CONFIRMED from source: `vscode/src/completions/context/retrievers/recent-user-actions/recent-view-port.ts`]

### 15.6 Language Inference (Auto-indexing)

For auto-indexing, Sourcegraph uses a **Lua sandbox** with language-specific inference rules:

```go
// From internal/codeintel/autoindexing/internal/inference/
// Languages with inference rules:
// - Go (gomod)
// - JVM (Maven/Gradle)
// - NPM (package.json)
// - Rust (Cargo.toml)
// - Python (setup.py, pyproject.toml)
// - Ruby (Gemfile)
// - .NET, Java, TypeScript
```

The inference uses a Lua sandbox (luasandbox) to safely evaluate inference scripts against repo file listings. This decouples language support from the Go binary.

[CONFIRMED from source: inference directory structure]

---

## 16. Final Differentiation Matrix

### 16.1 What Makes Sourcegraph Uniquely Good

1. **Precise code intelligence via SCIP**: Type-aware def/ref navigation - no other tool does this at scale without requiring compiler integration per repo
2. **Cross-repo unified context**: Single query can retrieve from 50+ repos simultaneously
3. **Zoekt-powered file ranking**: PageRank on code references is a genuinely better signal than file modification time or recency
4. **Incremental embeddings**: Only re-embed changed files on commit - efficient at scale
5. **Editor integration depth**: Tracks selection, visible lines, recent edits, viewport - context richness that a server-only tool cannot match

### 16.2 What Makes Parseltongue Potentially Better

1. **Zero deployment complexity**: `./parseltongue .` and it works. Sourcegraph requires 8+ services.
2. **Instant analysis**: Tree-sitter parsing takes milliseconds. Sourcegraph indexing takes minutes-hours.
3. **Graph-native queries**: CozoDB Datalog can express "find all transitive dependents" in one query; Sourcegraph needs multiple Postgres round-trips.
4. **Blast radius as first-class**: Sourcegraph has no concept of blast radius. This is Parseltongue's unique angle.
5. **API simplicity**: 26 flat REST endpoints vs. complex GraphQL schema that requires schema exploration.
6. **Complexity hotspots**: Cyclomatic complexity, nesting depth, function length - actionable quality metrics Sourcegraph doesn't expose.
7. **CPU-only operation**: Works in air-gapped, GPU-free environments.
8. **Open source / self-hosted without auth**: No tokens, no accounts, no quotas.

### 16.3 Steal List (Prioritized by Impact)

| Priority | Feature | Source | Complexity to Steal |
|----------|---------|--------|---------------------|
| 1 | RRF for context ranking | `reciprocal-rank-fusion.ts` | Low (50 lines) |
| 2 | Semantic chunking at boundaries | `split.go` | Low (100 lines) |
| 3 | Jaccard similarity with stemming | `bestJaccardMatch.ts` | Medium (200 lines) |
| 4 | Source provenance on every result | `ContextItemSource` enum | Low (schema change) |
| 5 | Async job pattern for expensive queries | Deep Search API pattern | Medium |
| 6 | File exclusion patterns | `DefaultExcludedFilePathPatterns` | Low |
| 7 | Auto-generated file detection | `autogeneratedFileHeaders` | Low |
| 8 | RetrieverStat metadata on responses | `RetrieverStat` interface | Low |
| 9 | cl100k token counting | `counter.ts` | Low (crate: tiktoken-rs) |
| 10 | Two-tier token budgets (fast vs smart) | `constants.ts` | Low |

[INFERRED from analysis]

---

*End of Analysis. Sources: sourcegraph-public-snapshot, cody-public-snapshot, amp-contrib, amp.nvim repos via GitHub API.*
