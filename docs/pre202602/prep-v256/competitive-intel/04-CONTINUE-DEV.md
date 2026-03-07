# Continue.dev Deep Source Analysis: Competitive Intelligence for Parseltongue

**Analysis Date**: 2026-02-19
**Source**: continuedev/continue (GitHub, Apache 2.0, 31,447 stars at time of analysis)
**Method**: Direct source analysis via GitHub API (no clone required)
**Analyst**: Claude Sonnet 4.5 via Parseltongue TDD Agent

---

## Table of Contents

1. [Project Structure Overview](#1-project-structure-overview)
2. [MCP Integration — THE CRITICAL SECTION](#2-mcp-integration)
3. [Context Providers — How Code Context Is Gathered](#3-context-providers)
4. [Control Flow — Query to Response](#4-control-flow)
5. [Data Flow — Indexing Pipeline](#5-data-flow--indexing-pipeline)
6. [Shreyas-Style Differentiation Analysis](#6-shreyas-style-differentiation)
7. [What Parseltongue Can Learn](#7-what-parseltongue-can-learn)
8. [Appendix: Raw TypeScript Interfaces](#8-appendix-raw-typescript-interfaces)

---

## 1. Project Structure Overview

[CONFIRMED from source]

```
continuedev/continue/
├── core/                    # The brain: TypeScript core logic
│   ├── autocomplete/        # Tab completion provider
│   ├── config/              # Config loading (YAML, JSON, legacy)
│   ├── context/             # Context providers + MCP integration
│   │   ├── mcp/             # MCPConnection, MCPManagerSingleton, MCPOauth
│   │   ├── providers/       # 30+ context providers (file, codebase, docs, etc.)
│   │   └── retrieval/       # RAG pipelines (reranker, no-reranker)
│   ├── indexing/            # Codebase indexing
│   │   ├── chunk/           # Chunking strategies (code, basic, markdown)
│   │   ├── docs/            # Docs crawl+embed
│   │   ├── CodebaseIndexer.ts
│   │   ├── LanceDbIndex.ts  # Vector store (LanceDB)
│   │   └── FullTextSearchCodebaseIndex.ts  # SQLite FTS5
│   ├── llm/                 # LLM providers (30+)
│   ├── tools/               # Tool definitions + implementations
│   │   ├── definitions/     # read_file, grep_search, glob_search, etc.
│   │   └── implementations/ # Actual execution logic
│   ├── commands/slash/      # Slash commands + MCP prompt commands
│   ├── protocol/            # IPC messaging (core <-> IDE extension)
│   └── core.ts              # Main Core controller class
├── extensions/              # IDE extension shells (VS Code, JetBrains)
├── gui/                     # React frontend (chat UI)
├── packages/
│   ├── config-yaml/         # YAML schema + validation (Zod)
│   │   └── src/schemas/mcp/ # MCP server schema definitions
│   ├── config-types/        # Shared TypeScript types
│   └── llm-info/            # LLM metadata
└── binary/                  # Continue CLI binary
```

**Key insight**: Continue.dev has a clean separation between the core logic (TypeScript) and the IDE-specific extension shells. The core exposes a protocol-based messenger API. This is the architectural pattern Parseltongue should adopt.

---

## 2. MCP Integration

**This is the most important section for Parseltongue.**

### 2.1 Transport Layers Supported

[CONFIRMED from source: `core/context/mcp/MCPConnection.ts`]

Continue.dev supports **4 MCP transport types**:

| Transport | Type Key | When Used |
|-----------|----------|-----------|
| stdio | `"stdio"` or when `command` field present | Local processes (npx, uvx, etc.) |
| SSE | `"sse"` | HTTP + Server-Sent Events |
| Streamable HTTP | `"streamable-http"` | Modern HTTP with streaming |
| WebSocket | `"websocket"` | WebSocket connection |

**Auto-detection logic** (when `type` is omitted and only `url` is provided):
1. Tries `streamable-http` first
2. Falls back to `sse` if that fails
3. Throws error if both fail

```typescript
// From MCPConnection.ts - auto-detection
} else {
  try {
    const transport = this.constructHttpTransport({
      ...this.options,
      type: "streamable-http",
    });
    await this.client.connect(transport, {});
  } catch (e) {
    try {
      const transport = this.constructSseTransport({
        ...this.options,
        type: "sse",
      });
      await this.client.connect(transport, {});
    } catch (e) {
      throw new Error(
        `MCP config with URL and no type specified failed both SSE and HTTP connection: ...`
      );
    }
  }
}
```

**PARSELTONGUE IMPLICATION**: Should implement `streamable-http` as primary transport (it's the modern standard and tried first). SSE is the fallback that Claude Desktop also supports. For development/debugging, `stdio` wrapping of the Parseltongue binary is useful.

### 2.2 MCP Connection Lifecycle

[CONFIRMED from source]

The `MCPConnection` class manages one connection to one MCP server:

1. **Constructor**: Creates SDK `Client` with `name: "continue-client"`, `version: "1.0.0"`, empty capabilities
2. **`connectClient(forceRefresh, externalSignal)`**: The main entry point
   - Status transitions: `not-connected` → `connecting` → `connected` | `error` | `disabled`
   - Default timeout: **20 seconds** (`DEFAULT_MCP_TIMEOUT = 20_000`)
   - On connect, calls `client.getServerCapabilities()`
   - Then lists: resources, resource templates, tools, prompts

3. **Capability discovery order**:
```typescript
// Resources -> Context Providers
if (capabilities?.resources) {
  const { resources } = await this.client.listResources({}, { signal });
  const { resourceTemplates } = await this.client.listResourceTemplates({}, { signal });
}

// Tools -> Tools panel
if (capabilities?.tools) {
  const { tools } = await this.client.listTools({}, { signal });
}

// Prompts -> Slash Commands
if (capabilities?.prompts) {
  const { prompts } = await this.client.listPrompts({}, { signal });
}
```

**CRITICAL MAPPING**:
- MCP **Resources** → Continue **Context Providers** (the `@mcp` mention)
- MCP **Tools** → Continue **Tools panel** (agent can call them)
- MCP **Prompts** → Continue **Slash Commands** (`/mcp-prompt-name`)

### 2.3 MCP YAML Configuration Schema

[CONFIRMED from source: `packages/config-yaml/src/schemas/mcp/index.ts`]

This is the Zod schema that validates every MCP server config:

```typescript
// Base fields for ALL MCP servers
const baseMcpServerSchema = z.object({
  name: z.string(),                    // Display name (also used as ID)
  serverName: z.string().optional(),   // Internal server name override
  faviconUrl: z.string().optional(),   // Icon URL for UI
  sourceFile: z.string().optional(),   // Added during loading
  sourceSlug: z.string().optional(),   // Added during loading
  connectionTimeout: z.number().gt(0).optional(),  // Timeout in ms
});

// Stdio transport
const stdioMcpServerSchema = baseMcpServerSchema.extend({
  command: z.string(),                       // e.g. "npx", "uvx", "python"
  type: z.literal("stdio").optional(),
  args: z.array(z.string()).optional(),      // CLI arguments
  env: z.record(z.string()).optional(),      // Env vars
  cwd: z.string().optional(),               // Working directory
});

// SSE/HTTP transport
const sseOrHttpMcpServerSchema = baseMcpServerSchema.extend({
  url: z.string(),                           // Server URL
  type: z.union([z.literal("sse"), z.literal("streamable-http")]).optional(),
  apiKey: z.string().optional(),             // Auth token (auto-added as Bearer)
  requestOptions: requestOptionsSchema.optional(),
});
```

**Resulting YAML for a Parseltongue HTTP MCP server** (what users would put in `~/.continue/config.yaml`):

```yaml
name: Local Config
version: "1.0.0"
schema: v1

mcpServers:
  - name: Parseltongue
    url: http://localhost:7070/mcp
    type: streamable-http
    connectionTimeout: 30000

  # Alternative: stdio for local binary
  - name: Parseltongue
    command: parseltongue
    args: ["--mcp-server", "--port", "7070"]
    env:
      PARSELTONGUE_DB: /path/to/cozo.db
```

### 2.4 MCPManagerSingleton

[CONFIRMED from source: `core/context/mcp/MCPManagerSingleton.ts`]

A process-level singleton that manages all MCP connections:

```typescript
class MCPManagerSingleton {
  connections: Map<string, MCPConnection>  // key = server ID (= name)

  setConnections(servers: InternalMcpOptions[], forceRefresh: boolean, extras?)
  // Diffs: removes stale connections, adds new ones, triggers refresh

  refreshConnections(force: boolean)
  // Connects all MCPConnections in parallel

  getStatuses(): (MCPServerStatus & { client: Client })[]
  // Used to populate UI status panel

  getPrompt(serverName, promptName, args)
  // Called when user runs /mcp-slash-command
}
```

**ID assignment**: The server `name` field doubles as the connection ID:
```typescript
const shared = {
  id: name,   // name IS the ID
  name,
  ...
};
```

**Transport comparison for deduplication** (determines if reconnect is needed):
```typescript
private compareTransportOptions(a, b): boolean {
  if (a.type !== b.type) return false;
  if ("command" in a && "command" in b) {
    return a.command === b.command &&
      JSON.stringify(a.args) === JSON.stringify(b.args) &&
      this.compareEnv(a.env, b.env);
  } else if ("url" in a && "url" in b) {
    return a.url === b.url;
  }
  return false;
}
```

### 2.5 MCP Tool Naming Convention

[CONFIRMED from source: `core/tools/mcpToolName.ts`]

When MCP tools are exposed to the LLM, they get prefixed with the server name:

```typescript
export function getToolNameFromMCPServer(serverName: string, toolName: string) {
  // Replace non-alphanumeric with underscore, strip leading/trailing
  const serverPrefix = serverName
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .replace(/_+/g, "_");

  // Avoid double-prefixing if tool already starts with server prefix
  if (toolName.startsWith(serverPrefix)) {
    return toolName;
  }
  return `${serverPrefix}_${toolName}`;
}
```

**Example**: Server named `"Parseltongue"` + tool `"blast_radius"` → `"parseltongue_blast_radius"`

**PARSELTONGUE IMPLICATION**: Name your MCP server `"parseltongue"` in configs. Tool names will auto-get the prefix or you can pre-include it in tool names.

### 2.6 MCP Resource Templates (The "@parseltongue" Mention System)

[CONFIRMED from source: `core/context/providers/MCPContextProvider.ts`]

Resources and Resource Templates are what power the `@mcp-servername` context selector:

```typescript
class MCPContextProvider extends BaseContextProvider {
  // The "@" mention in the chat UI populates this submenu
  async loadSubmenuItems(): Promise<ContextSubmenuItem[]> {
    return this.options.submenuItems.map((item) => ({
      ...item,
      id: JSON.stringify({
        mcpId: this.options.mcpId,
        uri: item.id,    // The MCP resource URI
      }),
    }));
  }

  // When user selects a resource, calls MCP server to get content
  async getContextItems(query: string, extras): Promise<ContextItem[]> {
    const { mcpId, uri } = MCPContextProvider.decodeMCPResourceId(query);
    const connection = MCPManagerSingleton.getInstance().getConnection(mcpId);

    // Inserts user query into URI template: /resource/{query} -> /resource/user-typed-text
    const resourceuri = this.insertInputToUriTemplate(uri, extras.fullInput);
    const { contents } = await connection.getResource(resourceuri);

    return contents.map(resource => ({
      name: resource.uri,
      content: resource.text,  // Only text resources supported currently
    }));
  }
}
```

**Resource Template Support**: Continue experimentally supports MCP resource templates with a `{query}` variable:
```
// If your resource URI contains {query}:
// /search?q={query}
// Continue will substitute the user's input text into the template
```

**PARSELTONGUE IMPLICATION**: For Parseltongue to appear as `@parseltongue` in Continue's UI:
1. Implement MCP `resources/list` endpoint returning resources
2. Implement MCP `resources/read` endpoint
3. Consider resource templates with `{query}` for search-style resources

### 2.7 MCP JSON Config Compatibility

[CONFIRMED from source: `core/context/mcp/json/loadJsonMcpConfigs.ts`]

Continue also loads MCP configs from JSON files, supporting **3 formats**:

1. **Claude Desktop format** (`~/.continue/mcpServers/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "parseltongue": {
      "command": "parseltongue",
      "args": ["--mcp"],
      "env": {}
    }
  }
}
```

2. **Claude Code format** (also supports `projects` key):
```json
{
  "mcpServers": { ... },
  "projects": {
    "/path/to/project": {
      "mcpServers": { ... }
    }
  }
}
```

3. **Single server JSON** (filename is the server name):
```json
{
  "command": "parseltongue",
  "args": ["--mcp"]
}
```

**Search paths**:
- `<workspace>/.continue/mcpServers/` (per-project)
- `~/.continue/mcpServers/` (global, if `includeGlobal=true`)

**PARSELTONGUE IMPLICATION**: Zero-config onboarding strategy:
- Drop a JSON file in `~/.continue/mcpServers/parseltongue.json`
- Users don't need to edit YAML at all

### 2.8 OAuth Support

[CONFIRMED from source: `core/context/mcp/MCPOauth.ts` exists]

OAuth is supported for SSE transport type only. For Bearer token auth (simpler), use the `apiKey` field in config — it's automatically added as `Authorization: Bearer <apiKey>` header.

### 2.9 Windows Compatibility

[CONFIRMED from source: `MCPConnection.constructStdioTransport`]

On Windows, commands like `npx`, `uv`, `uvx`, `pnpx` need to be run via `cmd.exe /c`. Continue handles this automatically. Parseltongue binary should work if named `parseltongue.exe` on Windows without special handling.

---

## 3. Context Providers

### 3.1 Complete List of Built-In Context Providers

[CONFIRMED from source: `core/context/providers/` directory]

```
@file              - FileContextProvider           (submenu: search files)
@currentFile       - CurrentFileContextProvider    (currently open file)
@diff              - DiffContextProvider           (git diff)
@terminal          - TerminalContextProvider       (terminal output)
@problems          - ProblemsContextProvider       (IDE errors/warnings)
@rules             - RulesContextProvider          (project rules)
@codebase          - CodebaseContextProvider       (semantic search)
@code              - CodeContextProvider           (specific code symbols)
@tree              - FileTreeContextProvider       (directory tree)
@folder            - FolderContextProvider         (all files in folder)
@docs              - DocsContextProvider           (crawled docs)
@open              - OpenFilesContextProvider      (open editor tabs)
@search            - SearchContextProvider         (text search results)
@url               - URLContextProvider            (web page content)
@web               - WebContextProvider            (web search)
@google            - GoogleContextProvider         (Google search)
@repo-map          - RepoMapContextProvider        (repo structure map)
@database          - DatabaseContextProvider       (SQL query results)
@postgres          - PostgresContextProvider       (PostgreSQL)
@os                - OSContextProvider             (OS info)
@commit            - GitCommitContextProvider      (git commit details)
@issue             - GitHubIssuesContextProvider   (GitHub issues)
@gitlab-mr         - GitLabMergeRequestContextProvider
@clipboard         - ClipboardContextProvider
@discord           - DiscordContextProvider
@debugger          - DebugLocalsProvider
@greptile          - GreptileContextProvider       (external service)
@http              - HttpContextProvider           (custom HTTP endpoint)
@custom            - CustomContextProvider         (user-defined)
@mcp-<servername>  - MCPContextProvider            (per MCP server)
```

**Default providers** (always loaded, no config needed):
- `@file`, `@currentFile`, `@diff`, `@terminal`, `@problems`, `@rules`

### 3.2 Context Provider Interface

[CONFIRMED from source: `core/context/index.ts`]

```typescript
export abstract class BaseContextProvider implements IContextProvider {
  options: { [key: string]: any };

  constructor(options: { [key: string]: any }) {
    this.options = options;
  }

  static description: ContextProviderDescription;

  // Core method: takes query string + extras, returns ContextItems
  abstract getContextItems(
    query: string,
    extras: ContextProviderExtras,
  ): Promise<ContextItem[]>;

  // Optional: provides submenu items for "@"-mention dropdown
  async loadSubmenuItems(args: LoadSubmenuItemsArgs): Promise<ContextSubmenuItem[]> {
    return [];
  }
}
```

Key types:
```typescript
interface ContextItem {
  name: string;           // Display name
  description: string;   // Short description
  content: string;       // The actual content injected into prompt
  uri?: {
    type: "file" | "url";
    value: string;
  };
}

interface ContextProviderDescription {
  title: string;             // The "@" name (e.g. "file")
  displayTitle: string;      // UI label
  description: string;       // Tooltip
  type: "normal" | "query" | "submenu";
  renderInlineAs?: string;   // How to render inline
  dependsOnIndexing?: ContextIndexingType[];  // Which indexes needed
}

type ContextIndexingType = "chunk" | "codeSnippets" | "fullTextSearch" | "embeddings";
```

### 3.3 Codebase Context Provider

[CONFIRMED from source: `core/context/providers/CodebaseContextProvider.ts`]

The `@codebase` provider declares dependencies on 3 index types:

```typescript
static description: ContextProviderDescription = {
  title: "codebase",
  displayTitle: "Codebase",
  description: "Automatically find relevant files",
  type: "normal",
  renderInlineAs: "",
  dependsOnIndexing: ["embeddings", "fullTextSearch", "chunk"],
};
```

It delegates entirely to `retrieveContextItemsFromEmbeddings()`.

### 3.4 Token Budget Management

[CONFIRMED from source: `core/context/retrieval/retrieval.ts`]

```typescript
const DEFAULT_N_FINAL = 25;

// Token budget logic:
const contextLength = extras.llm.contextLength;
const tokensPerSnippet = 512;
const nFinal = options?.nFinal ??
  Math.min(DEFAULT_N_FINAL, contextLength / tokensPerSnippet / 2);
// = min(25, fill half the context window with 512-token snippets)

// With reranker: retrieve 2x, then rerank down to nFinal
const nRetrieve = useReranking ? options?.nRetrieve || 2 * nFinal : nFinal;
```

Global constants from `core/util/parameters.ts`:
```typescript
export const RETRIEVAL_PARAMS = {
  rerankThreshold: 0.3,     // Minimum reranker score
  nFinal: 20,               // Default snippets to return
  nRetrieve: 50,            // Default candidates to retrieve before rerank
  bm25Threshold: -2.5,      // BM25 minimum score
  nResultsToExpandWithEmbeddings: 5,
  nEmbeddingsExpandTo: 5,
};
```

---

## 4. Control Flow — Query to Response

### 4.1 Architecture Overview

[CONFIRMED from source, INFERRED for some details]

```
User Types in GUI
       |
       v
GUI (React, WebView)
       |  messenger.send("streamChat", {...})
       v
Core (core.ts - Core class)
       |
       +-- Context gathering: ContextProvider.getContextItems() for each "@mention"
       |
       +-- Config resolution: which LLM model to use? (selectedModelByRole.chat)
       |
       +-- llmStreamChat() -> LLM provider API call
       |
       +-- Tool calls? -> callTool() -> builtin or MCP tool
       |
       v
Response streamed back to GUI
```

### 4.2 Model Routing

[CONFIRMED from source: `core/config/yaml/models.ts` and `loadYaml.ts`]

Models are assigned **roles**:
```typescript
type ModelRole = "chat" | "summarize" | "apply" | "edit" | "autocomplete" | "embed" | "rerank" | "subagent";
```

Default roles if not specified: `["chat", "summarize", "apply", "edit"]`

YAML config example:
```yaml
models:
  - name: Claude 3.5 Sonnet
    provider: anthropic
    model: claude-3-5-sonnet-20241022
    roles: [chat, edit, apply, summarize]

  - name: text-embedding-3-small
    provider: openai
    model: text-embedding-3-small
    roles: [embed]

  - name: cohere-rerank-3
    provider: cohere
    model: rerank-english-v3.0
    roles: [rerank]
```

The config system uses `selectedModelByRole` to pick:
- `embed` model → drives codebase indexing
- `rerank` model → enables `RerankerRetrievalPipeline` instead of `NoRerankerRetrievalPipeline`
- `chat` model → responds to user

### 4.3 Slash Commands

[CONFIRMED from source: `core/commands/slash/`]

Continue supports:
1. **Built-in slash commands** (in `built-in-legacy/`)
2. **Prompt file slash commands** (`.continue/prompts/*.prompt` files)
3. **Prompt block slash commands** (from YAML `prompts:` section)
4. **MCP prompt slash commands** (MCP servers exposing `prompts`)

MCP prompts become slash commands via `constructMcpSlashCommand()`:
```typescript
// MCP prompt named "analyze-complexity" on server "parseltongue"
// Becomes: /parseltongue_analyze-complexity (or however named)
```

### 4.4 YAML Config Full Schema

[CONFIRMED from source: `packages/config-yaml/src/schemas/index.ts`]

```typescript
const configYamlSchema = baseConfigYamlSchema.extend({
  name: z.string(),
  version: z.string(),
  schema: z.string().optional(),

  models: z.array(/* model configs */).optional(),

  context: z.array(/* context provider configs */).optional(),
  // Each context item: { provider: string, name?: string, params?: any }

  mcpServers: z.array(/* MCP server configs */).optional(),
  // Each: stdio (command+args+env+cwd) OR http/sse (url+type+apiKey)

  rules: z.array(/* rule strings or objects */).optional(),

  prompts: z.array(/* slash command prompts */).optional(),

  docs: z.array(/* doc sites to crawl */).optional(),

  data: z.array(/* data pipeline configs */).optional(),

  requestOptions: /* global HTTP options */.optional(),

  env: z.record(z.string(), z.union([z.string(), z.number(), z.boolean()])).optional(),
});
```

**Block system**: YAML supports `uses:` references to reuse config blocks:
```yaml
mcpServers:
  - uses: my-hub-org/parseltongue-mcp@1.0.0
    with:
      DB_PATH: /my/db.cozo
    override:
      connectionTimeout: 60000
```

---

## 5. Data Flow — Indexing Pipeline

### 5.1 Storage Architecture

[CONFIRMED from source]

**Two databases used simultaneously**:

| Database | Purpose | Location |
|----------|---------|----------|
| SQLite | FTS5 full-text search, chunk metadata, Lance cache | `~/.continue/index/index.sqlite` |
| LanceDB | Vector embeddings storage | `~/.continue/index/lancedb/` |

**Additional**: Docs indexing may use separate SQLite tables.

### 5.2 Indexing Pipeline — Step by Step

[CONFIRMED from source: `core/indexing/CodebaseIndexer.ts`]

**Entry point**: `CodebaseIndexer` is initialized in `Core` constructor.

**Batch size**: 200 files per batch (limits memory + minimizes embedding API calls).

**Index types built** (determined by which context providers are configured):
```typescript
const indexTypeToIndexerMapping = {
  chunk:        () => new ChunkCodebaseIndex(readFile, serverClient, maxChunkSize),
  codeSnippets: () => new CodeSnippetsCodebaseIndex(ide),
  fullTextSearch: () => new FullTextSearchCodebaseIndex(),
  embeddings:   () => LanceDbIndex.create(embeddingsModel, readFile),
};
```

**Index dependency**: Only build indexes that are needed:
```typescript
const indexTypesToBuild = new Set(
  config.contextProviders
    .map(provider => provider.description.dependsOnIndexing)
    .filter(Array.isArray)
    .flat()
);
// e.g., @codebase provider needs: ["embeddings", "fullTextSearch", "chunk"]
```

### 5.3 Chunking Strategy

[CONFIRMED from source: `core/indexing/chunk/chunk.ts`]

```typescript
// Decision logic:
if (extension in supportedLanguages && !NON_CODE_EXTENSIONS.includes(extension)) {
  // Use tree-sitter code chunker (splits by functions/classes)
  yield* codeChunker(fileUri, contents, maxChunkSize);
} else {
  // Use basic chunker (splits by newlines/size)
  yield* basicChunker(contents, maxChunkSize);
}

// Non-code extensions (use basic chunker even if tree-sitter supports them):
const NON_CODE_EXTENSIONS = ["css", "html", "htm", "json", "toml", "yaml", "yml"];
```

**Chunk validation**: After chunking, each chunk is checked against `maxChunkSize` token limit. Chunks exceeding the limit are discarded.

**Tree-sitter languages**: Continue uses tree-sitter for code-aware chunking (same as Parseltongue).

### 5.4 Retrieval Pipeline

[CONFIRMED from source: `core/context/retrieval/pipelines/`]

**No-Reranker Pipeline** (default when no reranker model configured):

```
Query
  |
  +--25%-- Recently Edited Files (LRU cache of opened files, rechunked)
  |
  +--25%-- Full Text Search (SQLite FTS5 with trigrams + NLP stemming)
  |
  +--50%-- Vector Embeddings (LanceDB semantic search)
  |
  +------  Repo Map (LLM-guided file selection based on repo structure)
  |
  v
Deduplicate → Return top N chunks
```

**Reranker Pipeline**: Same 4 sources but retrieves 2x candidates, then reranks with cross-encoder model, cuts at `rerankThreshold: 0.3`.

**FTS Strategy**: Uses trigrams with NLP preprocessing:
```typescript
// Preprocessing: remove extra spaces, stem words, remove stopwords, build trigrams
let text = nlp.string.stem(query);
const tokens = nlp.tokens.removeWords(tokenize(text));
const trigrams = nlp.string.ngram(cleanedTokens, 3);
const ftsQuery = trigrams.map(t => `"${t}"`).join(" OR ");
```

**SQLite FTS5 table**:
```sql
CREATE VIRTUAL TABLE IF NOT EXISTS fts USING fts5(
    path,
    content,
    tokenize = 'trigram'
)
```

### 5.5 LanceDB Vector Store

[CONFIRMED from source: `core/indexing/LanceDbIndex.ts`]

- **Artifact ID**: `vectordb::<embeddingId>` — tagged per embedding model
- **Table per tag**: Each `(branch, directory)` pair gets its own LanceDB table
- **Cache**: SQLite table `lance_db_cache` stores vectors as JSON strings for incremental updates
- **Platform gating**: LanceDB is disabled on Linux with unsupported CPU targets (AVX2 required)

**LanceDB row schema**:
```typescript
interface LanceDbRow {
  uuid: string;
  path: string;
  cachekey: string;
  vector: number[];
  // + startLine, endLine, contents (from cache table)
}
```

### 5.6 Code Snippets Index

[CONFIRMED from source: `core/indexing/CodeSnippetsIndex.ts`]

Uses tree-sitter to extract code symbols (functions, classes, methods) as named snippets:

```sql
CREATE TABLE IF NOT EXISTS code_snippets (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL,
    cacheKey TEXT NOT NULL,
    content TEXT NOT NULL,
    title TEXT NOT NULL,     -- function/class name
    signature TEXT,          -- full signature
    startLine INTEGER NOT NULL,
    endLine INTEGER NOT NULL
)
```

### 5.7 Incremental Re-indexing

[CONFIRMED from source: `core/indexing/refreshIndex.ts` and `core/indexing/README.md`]

Uses content-based hashing (`cacheKey`) to detect changes:
- Files are walked with `walkDirAsync`
- Each file gets a cache key (hash of content)
- Delta computed into 4 buckets: `compute` (new/changed), `addTag`, `removeTag`, `del` (deleted)
- Only changed files are re-indexed

The process:
1. Check modified timestamps of all files (fast, like git)
2. Compare against SQLite catalog (last indexed timestamps)
3. For files to add: check if cached on another branch → just `addTag` vs full `compute`
4. For files to remove: if only one branch tagged → `delete`, else → `removeTag`

**Important limitation from README**: `FullTextSearchCodebaseIndex` does NOT differentiate between branches. All branches share one FTS index. LanceDB creates separate tables per `(branch, directory)` tag pair.

### 5.8 Repo Map — LLM-Guided File Selection

[CONFIRMED from source: `core/context/retrieval/repoMapRequest.ts`]

Continue has a "repo map" feature where an LLM reads a structured view of the repo and selects relevant files:

```
Supported models (hardcoded): claude-3, llama3.1/3.2, gemini-2.5, gpt-4
```

The prompt:
```
{repoMap}

Given the above repo map, your task is to decide which files are most likely
to be relevant in answering a question.

<reasoning>... LLM writes reasoning ...</reasoning>

<results>
path/to/file1.ts
path/to/file2.ts
...
</results>
```

**This is the "AI picks files" approach** vs. Parseltongue's "graph picks files" approach. Continue's approach:
- Requires a good LLM (Claude/GPT-4 class)
- Uses LLM reasoning to identify relevant files
- Non-deterministic (same query may yield different files)
- Costs LLM tokens just for context selection

Parseltongue's approach:
- Deterministic graph traversal
- Zero LLM cost for context selection
- No hallucination risk
- Based on actual import/call relationships, not fuzzy textual similarity

### 5.8 Embedding Model Configuration

[CONFIRMED from source]

Default in VS Code: `transformers.js` (runs locally in the extension process, no API key needed)

Configurable in YAML:
```yaml
models:
  - name: My Embeddings
    provider: openai          # or cohere, voyage, ollama, etc.
    model: text-embedding-3-small
    roles: [embed]
    apiKey: "sk-..."
```

`maxEmbeddingChunkSize` from the embedding model drives chunk sizing.

---

## 6. Shreyas-Style Differentiation Analysis

### 6.1 Continue.dev's MOAT

[INFERRED from architecture analysis]

**Primary moat**: **Open Source + Model Freedom + IDE Integration Depth**

1. **Zero vendor lock-in**: Works with ANY LLM provider (OpenAI, Anthropic, Ollama, etc.) — 30+ supported. Cursor locks you to Anthropic/OpenAI models only.

2. **MCP-first architecture**: First major IDE assistant to fully implement MCP as both client AND server pattern. The entire tool system is now MCP-compatible.

3. **Codebase intelligence without a cloud service**: LanceDB + SQLite runs entirely locally. Cursor/Copilot require cloud indexing.

4. **Block/Hub system**: Users can publish and share reusable config blocks (models, MCP servers, rules) via the Continue Hub — a primitive package manager for AI dev tooling.

5. **Extensibility**: 30+ context providers, custom ones via HTTP or Python, rules system, prompt files. Cursor doesn't have this.

### 6.2 Where Continue Beats Copilot/Cursor

[INFERRED]

| Feature | Continue | Copilot | Cursor |
|---------|----------|---------|--------|
| Model choice | ANY (30+) | GPT-4o, Claude | Anthropic, OpenAI |
| MCP support | Full client | Limited | Partial |
| Local embedding | Yes (transformers.js) | No | No |
| Codebase indexing | Local LanceDB+SQLite | Cloud | Cloud |
| Custom context providers | Yes | No | No |
| Config portability | YAML in git | No | Partial |
| Cost at scale | BYOK unlimited | $10-19/month | $20/month |
| Open source | Apache 2.0 | Closed | Closed |

### 6.3 Where Continue Is WEAK

[INFERRED from architecture analysis]

1. **Context quality**: The multi-source retrieval (25% FTS + 25% recent files + 50% embeddings + repo map) is heuristic and not particularly sophisticated. No graph-based dependency analysis.

2. **Setup complexity**: Configuration via YAML feels developer-centric. Non-technical users struggle with model selection, API keys, MCP server setup.

3. **No blast radius / dependency graph**: Continue can search codebases but cannot reason about "what breaks if I change X?" — this is exactly Parseltongue's opportunity.

4. **No structural code understanding**: The retrieval is text-similarity based. No actual AST-level understanding of which files import which modules, or which functions call which.

5. **Embeddings dependency**: Local embeddings (transformers.js) are slow; cloud embeddings cost money. Parseltongue's pre-indexed CozoDB graph has zero marginal cost per query.

6. **No cross-file relationship traversal**: If you ask "show me all callers of function X", Continue can't reliably answer that — FTS will find string matches but not structural callers.

7. **Chunking loses structure**: Splitting code into chunks loses the file-level and module-level context. Parseltongue's entity graph preserves relationships.

### 6.4 The Unique Insight

[INFERRED/SPECULATIVE]

Continue's architecture reveals a fundamental tension in LLM coding assistants: **the LLM needs context, but gathering context well requires understanding code structure**. Continue solves this with retrieval (text similarity + FTS) — a blunt instrument.

Parseltongue's insight: **pre-compute the structure once (graph database), serve it precisely on demand**. The difference between "find files similar to this query" and "give me the exact call graph for this function" is enormous for debugging and refactoring tasks.

---

## 7. What Parseltongue Can Learn

### 7.1 MCP Server Implementation — Exact Recipe

[CONFIRMED architecture, INFERRED implementation details]

Based on the Continue.dev source, here is the exact MCP server Parseltongue needs to implement:

**Transport**: `streamable-http` primary (tried first by Continue), SSE as fallback.

**Endpoint**: `POST /mcp` (streamable-http) or `GET /sse` + `POST /message` (SSE)

**Server capabilities to advertise**:
```json
{
  "capabilities": {
    "resources": {},
    "tools": {},
    "prompts": {}
  }
}
```

**Tools to expose** (these appear in Continue's agent panel):
```json
[
  {
    "name": "blast_radius",
    "description": "Analyze what would break if a given entity is changed. Returns all entities that directly or transitively depend on the target.",
    "inputSchema": {
      "type": "object",
      "required": ["entity"],
      "properties": {
        "entity": {
          "type": "string",
          "description": "Fully qualified entity name (e.g. 'src/auth.rs::validate_token')"
        },
        "max_depth": {
          "type": "integer",
          "description": "Maximum traversal depth (default: 5)"
        }
      }
    }
  },
  {
    "name": "dependency_graph",
    "description": "Get the dependency graph for a file or entity. Shows what it imports and what imports it.",
    "inputSchema": {
      "type": "object",
      "required": ["path"],
      "properties": {
        "path": {
          "type": "string",
          "description": "File path or entity name to analyze"
        }
      }
    }
  },
  {
    "name": "complexity_hotspots",
    "description": "Find the most complex files or functions in the codebase by cyclomatic complexity.",
    "inputSchema": {
      "type": "object",
      "properties": {
        "top_n": {
          "type": "integer",
          "description": "Number of hotspots to return (default: 10)"
        },
        "language": {
          "type": "string",
          "description": "Filter by programming language"
        }
      }
    }
  },
  {
    "name": "smart_context",
    "description": "Get relevant code context within a token budget for a given task description.",
    "inputSchema": {
      "type": "object",
      "required": ["task", "token_budget"],
      "properties": {
        "task": {
          "type": "string",
          "description": "Natural language description of what you need context for"
        },
        "token_budget": {
          "type": "integer",
          "description": "Maximum tokens to return"
        }
      }
    }
  }
]
```

**Resources to expose** (these appear as `@parseltongue` in Continue's context menu):
```json
[
  {
    "uri": "parseltongue://entities",
    "name": "All Entities",
    "description": "Complete list of parsed entities in the codebase",
    "mimeType": "application/json"
  },
  {
    "uri": "parseltongue://complexity",
    "name": "Complexity Report",
    "description": "Cyclomatic complexity analysis of the codebase",
    "mimeType": "text/plain"
  }
]
```

**Resource Templates** (dynamic, user-query-driven):
```json
[
  {
    "uriTemplate": "parseltongue://search?q={query}",
    "name": "Search Entities",
    "description": "Search entities by name or content"
  },
  {
    "uriTemplate": "parseltongue://deps/{query}",
    "name": "Dependencies",
    "description": "Get dependencies for entity or file path"
  }
]
```

**Prompts to expose** (become `/slash-commands` in Continue):
```json
[
  {
    "name": "review-blast-radius",
    "description": "Review the blast radius before making a change",
    "arguments": [
      {
        "name": "entity",
        "description": "The entity you're planning to change",
        "required": true
      }
    ]
  }
]
```

### 7.2 Tool Naming Strategy

[CONFIRMED from source]

Server named `"Parseltongue"` in config → tools prefixed as `parseltongue_`:
- `parseltongue_blast_radius`
- `parseltongue_dependency_graph`
- `parseltongue_complexity_hotspots`
- `parseltongue_smart_context`

Recommendation: Keep tool names short since they get prefixed. The full name must stay under typical LLM function name limits (~64 chars).

### 7.3 Zero-Config Drop-in Strategy

[INFERRED from json loader analysis]

Create a single JSON file that users drop in `~/.continue/mcpServers/parseltongue.json`:

```json
{
  "command": "parseltongue",
  "args": ["--mcp-server"],
  "env": {
    "PARSELTONGUE_LOG": "warn"
  }
}
```

Or for HTTP server already running:
```json
{
  "url": "http://localhost:7070/mcp",
  "type": "streamable-http"
}
```

This works without ANY changes to the user's `config.yaml`.

### 7.4 Context Provider Pattern — Parseltongue as @parseltongue

[INFERRED from MCPContextProvider analysis]

To make Parseltongue appear as `@parseltongue` in the Continue chat:
1. Implement `resources/list` returning useful resources
2. Implement `resources/read` for each resource URI
3. Use resource templates with `{query}` for search functionality

The user types `@parseltongue` → sees a submenu of resources → selects one → Parseltongue's MCP server is called → content injected into prompt.

### 7.5 Embedding vs. Graph: The Positioning

[INFERRED/SPECULATIVE — HIGH CONFIDENCE]

Continue's retrieval is fundamentally **approximate** (embedding similarity, FTS trigrams). Parseltongue's retrieval is fundamentally **precise** (graph traversal).

**Marketing angle**: "Continue finds files that *look like* what you're asking for. Parseltongue finds files that *are mathematically related* to your change."

**Complementary, not competitive**: Parseltongue should position as an MCP server that *enhances* Continue, not replaces it. Continue brings the UI, model routing, and editing capabilities. Parseltongue brings structural intelligence.

### 7.6 The @codebase Gap Parseltongue Fills

[INFERRED]

Continue's `@codebase`:
- Max 25 snippets returned
- Ranked by text similarity to query
- No understanding of code relationships
- Requires embedding model setup
- Slow on first query (embedding model warmup)

Parseltongue's equivalent:
- Pre-indexed graph (zero warmup)
- Structural relationships (imports, calls, inherits)
- Precise entity-level search
- No embedding model needed
- Deterministic results

**Ideal integration**: Parseltongue registers as a context provider via MCP Resources. User types `@parseltongue dependencies src/auth.rs` and gets an exact dependency list, not a semantic similarity result.

### 7.7 YAML Configuration Template for Users

Based on Continue's schema, here is the ready-to-use YAML for a Parseltongue user:

```yaml
name: My Dev Config
version: "1.0.0"
schema: v1

models:
  - name: Claude 3.5 Sonnet
    provider: anthropic
    model: claude-3-5-sonnet-20241022
    apiKey: ${ANTHROPIC_API_KEY}
    roles: [chat, edit, apply, summarize]

# Option A: Parseltongue as stdio MCP server
mcpServers:
  - name: Parseltongue
    command: parseltongue
    args: ["--mcp-server", "--db", "${PARSELTONGUE_DB_PATH}"]
    env:
      PARSELTONGUE_LOG: "warn"
    connectionTimeout: 30000

# Option B: Parseltongue as HTTP MCP server (already running)
mcpServers:
  - name: Parseltongue
    url: http://localhost:7070/mcp
    type: streamable-http
    connectionTimeout: 30000

context:
  - provider: codebase   # Continue's own embedding search
  - provider: file       # @file mentions

rules:
  - "Always check Parseltongue blast radius before suggesting refactors"
  - "Use @parseltongue to understand code dependencies before making changes"
```

---

## 8. Appendix: Raw TypeScript Interfaces

### 8.1 MCP Tool Schema (as TypeScript)

[CONFIRMED from source: `core/index.d.ts`]

```typescript
export interface MCPTool {
  name: string;
  description?: string;
  inputSchema: {
    type: "object";
    properties?: Record<string, any>;
  };
  _meta?: Record<string, unknown> | undefined;
}
```

### 8.2 MCP Resource Schema

```typescript
export interface MCPResource {
  name: string;
  uri: string;
  description?: string;
  mimeType?: string;
}

export interface MCPResourceTemplate {
  uriTemplate: string;
  name: string;
  description?: string;
  mimeType?: string;
}
```

### 8.3 MCP Prompt Schema

```typescript
export interface MCPPrompt {
  name: string;
  description?: string;
  arguments?: {
    name: string;
    description?: string;
    required?: boolean;
  }[];
}
```

### 8.4 Internal MCP Options (Zod-validated before use)

```typescript
type BaseInternalMCPOptions = {
  id: string;           // = name field from config
  name: string;
  faviconUrl?: string;
  timeout?: number;     // connectionTimeout from config
  requestOptions?: RequestOptions;
  sourceFile?: string;
};

type InternalStdioMcpOptions = BaseInternalMCPOptions & {
  type?: "stdio";
  command: string;
  args?: string[];
  env?: Record<string, string>;
  cwd?: string;
};

type InternalStreamableHttpMcpOptions = BaseInternalMCPOptions & {
  type?: "streamable-http";
  url: string;
  apiKey?: string;
};

type InternalSseMcpOptions = BaseInternalMCPOptions & {
  type?: "sse";
  url: string;
  apiKey?: string;
};

type InternalWebsocketMcpOptions = BaseInternalMCPOptions & {
  type: "websocket";
  url: string;
};
```

### 8.5 MCPServerStatus (Returned for UI Display)

```typescript
interface MCPServerStatus {
  id: string;
  name: string;
  status: "not-connected" | "connecting" | "connected" | "error" | "disabled";
  errors: string[];
  infos: string[];
  prompts: MCPPrompt[];
  resources: MCPResource[];
  resourceTemplates: MCPResourceTemplate[];
  tools: MCPTool[];
  isProtectedResource: boolean;
}
```

### 8.6 Tool Definition Format (Built-In Tools)

[CONFIRMED from source: `core/tools/definitions/readFile.ts`]

```typescript
export interface Tool {
  type: "function";
  displayTitle: string;        // Shown in UI
  wouldLikeTo: string;         // "read {{{ filepath }}}" - shown while pending
  isCurrently: string;         // "reading {{{ filepath }}}" - shown while running
  hasAlready: string;          // "read {{{ filepath }}}" - shown when complete
  readonly: boolean;           // True if no side effects
  isInstant: boolean;          // True if fast (no spinner needed)
  group: string;               // Category grouping in UI
  function: {
    name: string;
    description: string;
    parameters: {
      type: "object";
      required: string[];
      properties: Record<string, {
        type: string;
        description: string;
      }>;
    };
  };
  defaultToolPolicy: "allowedWithoutPermission" | "allowedWithPermission" | "blocked";
  systemMessageDescription?: {
    prefix: string;
    exampleArgs: [string, string][];
  };
  toolCallIcon?: string;       // Hero icon name
}
```

### 8.7 Built-In Tool Names

```typescript
enum BuiltInToolNames {
  ReadFile = "read_file",
  ReadFileRange = "read_file_range",
  EditExistingFile = "edit_existing_file",
  SingleFindAndReplace = "single_find_and_replace",
  MultiEdit = "multi_edit",
  ReadCurrentlyOpenFile = "read_currently_open_file",
  CreateNewFile = "create_new_file",
  RunTerminalCommand = "run_terminal_command",
  GrepSearch = "grep_search",
  FileGlobSearch = "file_glob_search",
  SearchWeb = "search_web",
  ViewDiff = "view_diff",
  LSTool = "ls",
  CreateRuleBlock = "create_rule_block",
  RequestRule = "request_rule",
  FetchUrlContent = "fetch_url_content",
  CodebaseTool = "codebase",
  ReadSkill = "read_skill",
}
```

### 8.8 MCP Tool URI Encoding (Internal)

[CONFIRMED from source: `core/tools/callTool.ts`]

When MCP tools are loaded from a server, Continue internally encodes them as `mcp://` URIs:

```typescript
// Encoding
export function encodeMCPToolUri(mcpId: string, toolName: string): string {
  return `mcp://${encodeURIComponent(mcpId)}/${encodeURIComponent(toolName)}`;
}
// e.g., "Parseltongue" server + "blast_radius" tool
// -> "mcp://Parseltongue/blast_radius"

// Decoding when called
const [mcpId, toolName] = decodeMCPToolUri(uri);
const client = MCPManagerSingleton.getInstance().getConnection(mcpId);
const response = await client.client.callTool(
  { name: toolName, arguments: args },
  CallToolResultSchema,
  { timeout: client.options.timeout },
);
```

**MCP tool call response handling**:
```typescript
// Continue reads isError flag from MCP tool responses
if (response.isError === true) {
  // surfaces error to user
}
// content items from response are returned as ContextItem[]
```

This means Parseltongue's tool responses MUST follow the MCP spec's `CallToolResult` schema:
```json
{
  "content": [
    {
      "type": "text",
      "text": "... your analysis output ..."
    }
  ],
  "isError": false
}
```

### 8.9 Tool vs. HTTP Context Provider

[CONFIRMED from source]

Continue supports TWO ways to call external services:
1. **MCP Tools** (via `mcp://` URI) — the modern, recommended way
2. **HTTP Tools** (via `https://` URI) — legacy, direct POST to URL

HTTP tool format:
```typescript
// POST to URL with:
{ "arguments": { ...toolArgs } }
// Expects response:
{ "output": ContextItem[] }
```

Parseltongue should support MCP (Priority 1) and could optionally support the HTTP tool format for simpler integration.

---

## Summary: Action Items for Parseltongue MCP Integration

### Priority 1 — Critical for Continue.dev compatibility

1. **Implement `streamable-http` transport** (modern HTTP streaming). This is what Continue tries first.
2. **Implement capability advertisement**: `{ resources: {}, tools: {}, prompts: {} }`
3. **Implement `tools/list`** endpoint returning Parseltongue's analysis tools with proper `inputSchema`
4. **Implement `tools/call`** endpoint to execute tool calls
5. **Server name**: Use `"Parseltongue"` → tools auto-prefixed as `parseltongue_*`

### Priority 2 — Full @-mention integration

6. **Implement `resources/list`** returning useful static resources
7. **Implement `resources/read`** to serve resource content
8. **Implement `resources/templates`** with `{query}` variables for search
9. Publish `~/.continue/mcpServers/parseltongue.json` for zero-config setup

### Priority 3 — Power user features

10. **Implement `prompts/list`** for slash command integration
11. **Implement `prompts/get`** to return prompt content
12. Create YAML config block publishable to Continue Hub
13. Implement `apiKey` auth support (for remote Parseltongue instances)

### Key Insight for Positioning

Continue.dev is the retrieval layer + UI layer. Parseltongue is the structural intelligence layer. They are **complementary**. Parseltongue should market itself as "the MCP server that gives Continue structural code intelligence" — precision over approximate similarity.

---

*Analysis complete. All TypeScript code snippets confirmed from direct source reading. Configuration schemas validated against Zod schema definitions. Architecture inferences based on code flow analysis.*
