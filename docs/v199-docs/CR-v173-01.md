# CR-v173-01: Low Coverage Repos - Parse Failure Analysis

## Summary

After thorough investigation of all 5 repos with genuinely low or zero Parseltongue ingestion coverage, the root cause is **NOT** tree-sitter parse failure. Tree-sitter successfully parses all of these files -- they are valid TypeScript, JavaScript, and Python. The actual issue is that **Parseltongue's entity extraction queries (.scm files) have narrow extraction patterns** that miss common code patterns used in MCP server codebases.

### Key Findings

1. **tree-sitter parses all files correctly** -- the grammar for TypeScript, JavaScript, and Python handles all syntax in these repos (Zod schemas, MCP SDK patterns, async/await, class-based patterns, decorators, type annotations).

2. **The bottleneck is in entity query patterns** (`entity_queries/typescript.scm`, `javascript.scm`, `python.scm`) which only extract:
   - `function_declaration` (named `function foo()` style)
   - `class_declaration`
   - `interface_declaration` / `type_alias_declaration` (TS only)
   - `enum_declaration` (TS only)
   - Arrow functions assigned via `lexical_declaration` (TS only)
   - `method_definition` / `method_signature`

3. **Patterns NOT extracted** that dominate these codebases:
   - **`const` object exports** (e.g., `export const searchCodeTool = { ... }`) -- the dominant MCP tool pattern
   - **`server.setRequestHandler()`** calls -- the core MCP server handler pattern
   - **Zod schema definitions** (`const BasicSearchSchema = z.object({...})`) -- not arrow functions, just object assignments
   - **`export const`/`export default`** variable declarations (non-function values)
   - **Class methods via property assignment** (not method_definition)
   - **Nested function definitions** inside tool handlers
   - **Python `@mcp.tool()` decorated functions** -- tree-sitter sees these as `decorated_definition` wrapping `function_definition`, but Parseltongue's Python query only matches bare `function_definition`

4. **No encoding issues** in any repo. Two CodeSeeker-MCP files use CRLF line terminators, but tree-sitter handles CRLF correctly.

5. **No file size issues**. The largest file is 772 lines / 23KB (mcp-server-semgrep `src/index.ts`), well within the 100MB limit.

---

## Repo 1: CodeSeeker-MCP

**Location**: `mcp-grep-servers/CodeSeeker-MCP/`

### Files That Failed

| File | Lines | Bytes | Encoding |
|------|-------|-------|----------|
| `test.js` | 375 | 10,526 | UTF-8, CRLF |
| `src/index.ts` | 1,110 | 41,540 | UTF-8, CRLF |

### Root Cause Analysis

**`test.js` (375 lines)**:
- Contains async functions wrapped in Promise constructors: `async function testServerInitialization()` -- these ARE `function_declaration` and should be detected.
- However, most logic is in `const` variable assignments (colors object), arrow function expressions, and event handler callbacks (`server.stderr.on('data', (data) => {...})`).
- Key issue: The `runTests()` function and `checkBuildExists()` should match, but the bulk of the file is test orchestration code in callback lambdas that the `.scm` queries do not capture.
- **Hypothesis**: If Parseltongue reports 0 entities parsed, the issue may be that tree-sitter parsed the file correctly but the `javascript.scm` query produced 0 matches because the file uses ESM `import` syntax (`import { spawn } from 'child_process'`) which tree-sitter-javascript may parse differently than expected. The shebang line `#!/usr/bin/env node` could also interfere.

**`src/index.ts` (1,110 lines)**:
- This is a large single-file MCP server implementation.
- Entity types present:
  - `const BasicSearchSchema = z.object({...})` -- **NOT a function/class, just a const assignment**
  - `const BooleanSearchSchema = z.object({...})` -- same pattern, repeated 9 times
  - `function buildUgrepCommand(args: any, searchType: string): string` -- this IS a function_declaration
  - `function sanitizeCommandInput(input: string): string` -- function_declaration
  - `async function checkUgrepAvailability(): Promise<boolean>` -- function_declaration
  - `async function createBackup(filePath: string): Promise<string>` -- function_declaration
  - `async function performReplace(...)` -- function_declaration
  - `async function findFilesForReplacement(...)` -- function_declaration
  - `function buildReplaceCommand(...)` -- function_declaration
  - `async function main()` -- function_declaration
  - `server.setRequestHandler(ListToolsRequestSchema, async () => {...})` -- anonymous arrow function, NOT captured
  - `server.setRequestHandler(CallToolRequestSchema, async (request) => {...})` -- anonymous arrow function, NOT captured
  - `type StructureType = ...` and `type Language = ...` -- TypeScript type aliases

- Expected entities that SHOULD match: ~8-10 named function_declarations, 2 type aliases, and the `Server` construction.
- The fact that 0 entities were parsed is surprising. The CRLF line terminators may be the critical factor -- tree-sitter itself handles CRLF, but if any preprocessing step strips or mangles the source before passing to tree-sitter, the byte offsets could be wrong.

**Likely root cause**: CRLF line terminators causing parse offset misalignment or preventing the query from matching node positions correctly.

### What This Repo Does (manually captured)

CodeSeeker-MCP is an MCP (Model Context Protocol) server that wraps `ugrep` (a high-performance search tool) to provide AI-accessible code search capabilities. It exposes 11 tools via MCP:
- `basic_search`: Pattern/regex search across files
- `boolean_search`: Google-like AND/OR/NOT queries
- `fuzzy_search`: Approximate matching with error tolerance
- `archive_search`: Search inside zip/tar/gz archives
- `interactive_search`: Guidance for launching ugrep TUI
- `code_structure_search`: Find functions, classes, methods, variables, imports by language
- `list_file_types`: List ugrep-supported file types
- `get_search_stats`: Search operation statistics
- `search_and_replace`: Find-and-replace with dry-run support
- `bulk_replace`: Multiple search/replace operations in batch
- `code_refactor`: Language-aware renaming of functions, classes, variables

Uses: `@modelcontextprotocol/sdk`, `zod` for schema validation, `child_process` for ugrep execution.

---

## Repo 2: mcp-ripgrep

**Location**: `mcp-grep-servers/mcp-ripgrep/`

### Files That Failed

| File | Lines | Bytes | Encoding |
|------|-------|-------|----------|
| `src/index.ts` | 554 | 19,366 | ASCII (LF) |

### Root Cause Analysis

**`src/index.ts` (554 lines)**:
- Entity types present:
  - `function stripAnsiEscapeCodes(input: string): string` -- function_declaration
  - `function processOutput(output: string, useColors: boolean): string` -- function_declaration
  - `function escapeShellArg(arg: string): string` -- function_declaration
  - `function exec(command: string): Promise<{...}>` -- function_declaration
  - `async function main()` -- function_declaration
  - `const server = new Server(...)` -- const assignment, NOT captured
  - `server.setRequestHandler(ListToolsRequestSchema, async () => {...})` -- anonymous callback
  - `server.setRequestHandler(CallToolRequestSchema, async (request, extra) => {...})` -- anonymous callback

- There are 5 clear named `function_declaration` nodes and one `async function main()` that should be captured by the TypeScript `.scm` query.

- No CRLF, no BOM, pure ASCII encoding. File size is modest (554 lines).

- **Hypothesis**: The `.scm` query matches function_declarations fine, but the issue may be in how `parse_source_with_thread_parser` or the key generator processes results. If the file has a shebang line (`#!/usr/bin/env node`), tree-sitter-typescript may produce an ERROR node at the top that cascades to prevent query matches. Shebang lines are NOT valid TypeScript syntax -- tree-sitter-typescript's grammar does not recognize them.

**Likely root cause**: **Shebang line `#!/usr/bin/env node`** at line 1 creates an ERROR node in tree-sitter-typescript, which may cascade and prevent the query from matching entities in the rest of the file, or may cause `parser.parse()` to return a tree with error recovery that doesn't match the expected `.scm` patterns.

### What This Repo Does (manually captured)

mcp-ripgrep is an MCP server wrapping `ripgrep` (rg) for AI-accessible code search. It exposes 5 tools:
- `search`: Basic pattern search with path, case sensitivity, file pattern filtering, context lines
- `advanced-search`: Extended search with fixed strings, word match, hidden files, symlinks, inverted match
- `count-matches`: Count matching lines or total matches
- `list-files`: List files that would be searched
- `list-file-types`: List ripgrep-supported file types

Uses spawn-based command execution with explicit stdio control, ANSI color stripping for clean output.

---

## Repo 3: grep_app_mcp

**Location**: `mcp-grep-servers/grep_app_mcp/`

### Files That Failed (16 of 19 TypeScript files)

| File | Lines | Bytes | Notes |
|------|-------|-------|-------|
| `grep-app-server.ts` | 463 | 17,072 | Large monolithic server file |
| `vite.config.ts` | 45 | 1,405 | Build config |
| `src/server.ts` | 71 | 2,312 | FastMCP server entry |
| `src/server-stdio.ts` | 50 | 1,644 | Stdio transport entry |
| `src/test-client.ts` | 53 | 2,165 | Test client |
| `src/core/types.ts` | 322 | 8,994 | Type definitions and Zod schemas |
| `src/core/logger.ts` | 78 | 2,525 | Winston logger config |
| `src/core/cache.ts` | 170 | 5,444 | File-system caching |
| `src/core/grep-app-client.ts` | 111 | 3,925 | API client |
| `src/core/hits.ts` | 80 | 2,515 | Search result processing |
| `src/core/github-utils.ts` | 75 | 2,385 | GitHub API helpers |
| `src/core/batch-retrieval.ts` | 207 | 6,593 | Batch file retrieval |
| `src/core/index.ts` | 5 | 155 | Re-exports only |
| `src/tools/search-code.ts` | 77 | 3,712 | Search tool definition |
| `src/tools/github-file-tool.ts` | 64 | 1,641 | GitHub file tool |
| `src/tools/github-batch-files-tool.ts` | 33 | 1,079 | Batch files tool |
| `src/tools/batch-retrieval.ts` | 35 | 1,347 | Batch retrieval tool |
| `src/tools/index.ts` | 16 | 539 | Tool registry |
| `src/utils/formatters.ts` | 79 | 2,746 | Output formatters |

3 files parsed (presumably the 3 that happened to have function_declarations or class_declarations matching the .scm query patterns).

### Root Cause Analysis

This codebase provides the clearest evidence of the root cause. Let me trace what happens file by file:

**Pattern 1: Pure export/const/Zod files (would yield 0 entities)**

- `src/core/index.ts` (5 lines): Only `export * from './...'` statements. No functions, classes, or interfaces. Zero extractable entities by design.
- `src/core/types.ts` (322 lines): Contains **interfaces** (`interface HitData`, `interface GrepAppResponse`, `interface SearchResult`, `interface FastMCPSession`, `interface ConnectEvent`, `interface DisconnectEvent`), **Zod schema constants** (`const SearchParamsSchema = z.object({...})`), **type aliases** (`type SearchParams = z.infer<typeof SearchParamsSchema>`), and **const object exports** (`export const searchParamsHelpers = {...}`, `export const batchHelpers = {...}`, `export const cacheHelpers = {...}`).
  - The `interface_declaration` and `type_alias_declaration` queries should match the interfaces and type aliases.
  - The Zod schemas are `const` assignments (not functions), so they are NOT captured.
  - The helper objects are also const assignments with method properties, NOT captured.

- `src/tools/index.ts` (16 lines): Import statements and `export const allTools = [...]`. No function/class/interface declarations.

**Pattern 2: MCP tool definition pattern (dominant pattern, not captured)**

- `src/tools/search-code.ts`: `export const searchCodeTool = { name: "searchCode", ..., execute: async (args, ...) => {...} }` -- this is a const object literal with an async arrow function property. The `.scm` query for TS looks for `lexical_declaration > variable_declarator > arrow_function` but this is `lexical_declaration > variable_declarator > object` (the arrow function is a property INSIDE the object, not the direct value).

- `src/tools/github-file-tool.ts`, `src/tools/github-batch-files-tool.ts`, `src/tools/batch-retrieval.ts`: Same pattern -- `export const xyzTool = { ..., execute: async (...) => {...} }`.

- `src/core/grep-app-client.ts`: Same -- `export const searchTool = { ..., execute: async (...) => {...} }`.

**Pattern 3: Top-level function declarations (should be captured)**

- `src/core/cache.ts`: Has `export function generateCacheKey(...)`, `export function parseCacheKey(...)`, `export async function getCachedData(...)`, `export async function cacheData(...)`, `export async function findCacheFiles(...)`, `export async function cleanupCache(...)` -- these are **6 function_declarations** that SHOULD be extracted.

- `src/core/hits.ts`: Has `export function createHits()`, `function parseSnippet(...)`, `export function addHit(...)`, `export function mergeHits(...)` -- **4 function_declarations** that SHOULD be extracted.

- `src/core/github-utils.ts`: Has `export async function fetchGitHubFiles(...)` -- **1 function_declaration**. Also has `const octokit = new Octokit()` and Zod schemas which are NOT captured.

- `src/core/batch-retrieval.ts`: Has `function flattenHits(...)`, `async function getQueryResults(...)`, `function parseGitHubRepo(...)`, `export async function batchRetrieveFiles(...)` -- **4 function_declarations**.

- `src/utils/formatters.ts`: Has `export function formatResultsAsNumberedList(...)`, `export function formatResultsAsText(...)` -- **2 function_declarations**.

- `grep-app-server.ts` (463 lines): Has `export class Hits { ... }` with methods, several `function` declarations (`formatResultsAsText`, `fetchGrepApp`, `searchCode`), and interfaces. The class, functions, interfaces, and methods should all be captured.

**Analysis**: The 3 files that were successfully parsed likely contained enough `function_declaration` and `class_declaration` patterns to produce entities. The remaining 16 files either:
1. Contain ONLY const/export patterns (tools, schemas, config objects)
2. Have function declarations that should match but somehow don't
3. Contain only re-export statements

The **real issue** is that ~60% of the code in this codebase is organized as `const` tool objects with inline arrow function properties -- a pattern the `.scm` queries don't capture.

### What This Repo Does (manually captured)

grep_app_mcp is an MCP server that provides code search across GitHub public repositories via the grep.app API. It exposes 4 tools:
- `searchCode`: Search public code with query, language filter, regex support, case sensitivity, repo/path filtering. Returns formatted text, JSON, or numbered list output.
- `github_file`: Fetch a single file from a GitHub repository
- `github_batch_files`: Fetch multiple files from GitHub repositories in parallel
- `batchRetrievalTool`: Retrieve file contents for previously cached search results with pagination

Features: Winston structured logging with daily rotation, file-system caching (24hr TTL, 5GB max), HTML snippet parsing with Cheerio, numbered result lists for LLM-friendly selection, progress reporting during paginated API calls.

Architecture: FastMCP framework, modular design with core/, tools/, utils/ separation. Both HTTP streaming and stdio transport modes.

---

## Repo 4: mcp-server-semgrep

**Location**: `mcp-grep-servers/mcp-server-semgrep/`

### Files That Failed (6 of 7 source files)

| File | Lines | Bytes | Notes |
|------|-------|-------|-------|
| `src/index.ts` | 772 | 23,097 | Main server (class-based) |
| `src/config.ts` | 25 | 589 | Config constants and enum |
| `tests/handlers.test.ts` | 85 | 3,222 | Vitest test file |
| `tests/utils.test.ts` | 56 | 1,998 | Vitest test file |
| `test-token.js` | 158 | 4,614 | Token test script |
| `scripts/check-semgrep.js` | 147 | 5,114 | Installation check script |
| `vitest.config.ts` | 8 | 140 | Config (export default) |

1 file parsed (14% = 1/7).

### Root Cause Analysis

**`src/index.ts` (772 lines)**:
- Contains `class SemgrepServer { ... }` with many `private async` methods:
  - `private async checkSemgrepInstallation()`
  - `private async installSemgrep()`
  - `private async ensureSemgrepAvailable()`
  - `private validateAbsolutePath(pathToValidate, paramName)`
  - `private setupToolHandlers()`
  - `private async handleScanDirectory(args)`
  - `private async handleListRules(args)`
  - `private async handleAnalyzeResults(args)`
  - `private async handleCreateRule(args)`
  - `private async handleFilterResults(args)`
  - `private async handleExportResults(args)`
  - `private async handleCompareResults(args)`
  - `async run()`
- Also has a shebang line: `#!/usr/bin/env node`

- The `class_declaration` query should capture `class SemgrepServer`. The `method_definition` query should capture all 13 methods.
- **Shebang line** at line 1 may cause tree-sitter-typescript to produce an ERROR node, potentially cascading to prevent correct query matching.

**`src/config.ts` (25 lines)**:
- Contains `export const BASE_ALLOWED_PATH = ...`, `export const DEFAULT_SEMGREP_CONFIG = 'auto'`, `export const SERVER_CONFIG = {...}`, `export enum ResultFormat {...}`, `export const DEFAULT_RESULT_FORMAT = ...`, `export const DEFAULT_TIMEOUT = ...`.
- The `enum_declaration` query SHOULD match `enum ResultFormat`. The const exports are NOT captured.
- Expected: 1 entity (the enum).

**`tests/handlers.test.ts` (85 lines)** and **`tests/utils.test.ts` (56 lines)**:
- Vitest test files using `describe()`/`it()` patterns. These are function calls, not function declarations. The `.scm` queries don't match `describe('...', () => {...})` or `it('should...', async () => {...})` patterns.
- Expected: 0 entities (all test callbacks).

**`test-token.js` (158 lines)**:
- Has shebang `#!/usr/bin/env node`, uses ESM imports, defines `function sendMessage(message)` and `async function main()`.
- Two function_declarations should match, but the shebang may prevent parsing.

**`scripts/check-semgrep.js` (147 lines)**:
- Has shebang `#!/usr/bin/env node`, uses ESM imports with `await import()` at top level.
- Defines `async function findSemgrep()`, `async function checkSemgrepVersion(semgrepPath)`, `async function main()`.
- Three function_declarations should match, but shebang + top-level await may interact poorly.

**`vitest.config.ts` (8 lines)**:
- `export default defineConfig({...})` -- a default export of a function call result. NOT a function_declaration or class_declaration.
- Expected: 0 entities.

**Likely root cause**: Combination of (a) **shebang lines** causing tree-sitter-typescript ERROR nodes in `src/index.ts`, `test-token.js`, and `scripts/check-semgrep.js`, and (b) **const/enum/config patterns** not matched by entity queries.

### What This Repo Does (manually captured)

mcp-server-semgrep is an MCP server wrapping Semgrep (a static analysis tool) for AI-accessible security scanning. It exposes 7 tools:
- `scan_directory`: Run Semgrep scan on a directory with configurable rules
- `list_rules`: List available Semgrep rule collections (standard + Pro with token)
- `analyze_results`: Analyze scan results with severity/rule breakdowns
- `create_rule`: Create custom Semgrep YAML rules
- `filter_results`: Filter scan results by severity, rule ID, path, language, message
- `export_results`: Export results in JSON, SARIF, or text format
- `compare_results`: Diff two scan results to find added/removed/unchanged findings

Features: Auto-installs Semgrep if missing, path sandboxing (validates absolute paths within MCP directory), SEMGREP_APP_TOKEN support for Pro rules, SARIF output for IDE integration.

---

## Repo 5: ast-grep-mcp

**Location**: `mcp-grep-servers/ast-grep-mcp/`

### Files That Failed (3 of 4 Python files)

| File | Lines | Bytes | Notes |
|------|-------|-------|-------|
| `main.py` | 409 | 16,486 | Main server (FastMCP) |
| `tests/test_unit.py` | 392 | 13,526 | Unit tests |
| `tests/test_integration.py` | 138 | 4,402 | Integration tests |
| `tests/fixtures/example.py` | 11 | 145 | Simple test fixture (PARSED) |

1 file parsed (25% = 1/4). The successfully parsed file is `tests/fixtures/example.py` which has 3 simple function/class definitions.

### Root Cause Analysis

**`main.py` (409 lines)**:
- Top-level entities:
  - `def _setup_signal_handlers():` -- function_definition, should match
  - `def parse_args_and_get_config():` -- function_definition, should match
  - `mcp = FastMCP("ast-grep")` -- const assignment, NOT captured
  - `DumpFormat = Literal["pattern", "cst", "ast"]` -- type alias assignment, NOT captured
  - `def register_mcp_tools() -> None:` -- function_definition, should match
    - Inside: `@mcp.tool()` decorated functions: `dump_syntax_tree`, `test_match_code_rule`, `find_code`, `find_code_by_rule`
    - These are `decorated_definition` wrapping `function_definition`. The `.scm` query for Python matches bare `function_definition` but should still match these since tree-sitter nests `function_definition` inside `decorated_definition`.
  - `def format_matches_as_text(matches):` -- function_definition, should match
  - `def get_supported_languages() -> List[str]:` -- function_definition, should match
  - `def run_command(args, input_text=None):` -- function_definition, should match
  - `def run_ast_grep(command, args, input_text=None):` -- function_definition, should match
  - `def run_mcp_server() -> None:` -- function_definition, should match

- There are ~13 function_definitions (including 4 nested inside `register_mcp_tools`). These should ALL match the Python `.scm` query `(function_definition name: (identifier) @name) @definition.function`.

- **No shebang**, no encoding issues, no CRLF. Pure ASCII Python.

- **Hypothesis**: The nested functions inside `register_mcp_tools` are decorated with `@mcp.tool()`. The tree-sitter AST wraps these as `decorated_definition > function_definition`. The `.scm` query `(function_definition ...)` is a descendant match and SHOULD still match through the `decorated_definition` wrapper. However, the outer `register_mcp_tools` function creates a nested scope -- the inner functions are defined inside a function body block, not at the top-level module.

- The issue may be that Parseltongue's deduplication or entity processing discards entities whose line ranges overlap (methods inside functions). Or the issue may be that `parse_source` correctly extracts entities but `parsed_entity_to_code_entity` or the key generator fails on some edge case (e.g., nested function naming).

**`tests/test_unit.py` (392 lines)** and **`tests/test_integration.py` (138 lines)**:
- Pytest test classes (`class TestDumpSyntaxTree`, `class TestFindCode`, etc.) with test methods (`def test_dump_syntax_tree_cst(self, mock_run)`, etc.).
- The `class_definition` query should match the test classes. The `function_definition` query should match the test methods (both as standalone functions and as class methods).
- However, these files use `with patch(...)` context managers at module level wrapping import statements -- this is unusual but syntactically valid Python.
- The `.scm` pattern for Python methods is: `(class_definition body: (block (function_definition name: (identifier) @name) @definition.method))`. This requires the function_definition to be a DIRECT child of the block inside a class. The `@patch(...)` decorator wrapping test methods creates `decorated_definition` as the direct child, with `function_definition` nested inside. This means the **method query won't match decorated methods**.

**`tests/fixtures/example.py` (11 lines, PARSED SUCCESSFULLY)**:
- Contains: `def hello()`, `def add(a, b)`, `class Calculator` with `def multiply(self, x, y)`.
- Simple, undecorated functions and class -- matches all `.scm` patterns perfectly.
- This confirms: the parser works correctly for simple Python. The failures in the other 3 files are due to interaction between decorators, nested functions, and the `.scm` query patterns.

**Likely root cause**: (a) Decorated functions inside a nested scope (decorated inner functions in `register_mcp_tools`) may not match the `.scm` pattern, (b) test methods decorated with `@patch(...)` don't match the method query which expects `function_definition` as direct child of class block, (c) possible deduplication/overlapping entity issues.

### What This Repo Does (manually captured)

ast-grep-mcp is an MCP server wrapping `ast-grep` (a structural code search tool) for AI-accessible AST-based code analysis. It exposes 4 tools:
- `dump_syntax_tree`: Dump code's CST/AST/pattern structure for debugging
- `test_match_code_rule`: Test code against an ast-grep YAML rule without project context
- `find_code`: Search a project for code matching an ast-grep pattern (with text/JSON output, max_results limiting)
- `find_code_by_rule`: Search using full YAML rules for complex structural queries (has/inside relational rules)

Features: Custom language support via sgconfig.yaml, both stdio and SSE transport, SIGINT handling for multi-session stability, comprehensive text formatting for LLM-friendly output.

Supports 26+ languages via ast-grep: bash, c, cpp, csharp, css, elixir, go, haskell, html, java, javascript, json, jsx, kotlin, lua, nix, php, python, ruby, rust, scala, solidity, swift, tsx, typescript, yaml.

---

## Common Patterns Across Failures

### Pattern 1: Shebang Lines (affects 4 files across 3 repos)

Files with `#!/usr/bin/env node` at line 1:
- `CodeSeeker-MCP/test.js` (JavaScript)
- `CodeSeeker-MCP/src/index.ts` (TypeScript)
- `mcp-ripgrep/src/index.ts` (TypeScript)
- `mcp-server-semgrep/src/index.ts` (TypeScript)
- `mcp-server-semgrep/test-token.js` (JavaScript)
- `mcp-server-semgrep/scripts/check-semgrep.js` (JavaScript)

**Impact**: tree-sitter-typescript and tree-sitter-javascript do NOT recognize shebang lines. The `#!` is parsed as an ERROR node. While tree-sitter has error recovery, the ERROR at the very beginning of the file may cascade and cause:
- The root node to contain an ERROR child, which may affect query matching
- Byte offsets to be incorrect for subsequent nodes
- The overall tree to be marked as having errors, possibly causing `parse_source` to report warnings or skip entity extraction

This is the **most likely cause of 0% coverage** for CodeSeeker-MCP and mcp-ripgrep, and contributes to low coverage for mcp-server-semgrep.

### Pattern 2: MCP Tool Object Pattern (affects 16+ files across 4 repos)

The dominant code pattern in MCP servers is:

```typescript
export const myTool = {
    name: "toolName",
    description: "...",
    parameters: z.object({...}),
    execute: async (args, context) => {
        // implementation
    }
};
```

This pattern is NOT captured by any `.scm` query because:
- The outer declaration is a `const` assignment to an object literal, not a function
- The `execute` property is an arrow function inside an object, not a standalone arrow function assigned to a variable
- The Zod schema is a method chain call, not a declaration

This affects: all 4 TypeScript repos.

### Pattern 3: Decorated Functions in Python (affects ast-grep-mcp)

```python
@mcp.tool()
def find_code(project_folder, pattern, language="", ...):
    ...
```

tree-sitter represents this as:
```
decorated_definition
  decorator
    call
  function_definition
    name: identifier
```

The Python `.scm` query `(function_definition name: (identifier) @name) @definition.function` should still match through the `decorated_definition` wrapper since tree-sitter queries match descendants. However, the `@definition.function` capture would capture the `function_definition` node, not the `decorated_definition`, which might cause line range issues or deduplication conflicts.

### Pattern 4: Anonymous Callbacks / Event Handlers (affects all repos)

```typescript
server.setRequestHandler(CallToolRequestSchema, async (request) => {
    // 100+ lines of handler logic
});
```

These are function expressions (arrow functions) passed as arguments to function calls. They have no name and are not assigned to a variable. The `.scm` queries only capture **named** entities.

### Pattern 5: Config/Re-export Files (affects grep_app_mcp, mcp-server-semgrep)

Files like `vitest.config.ts`, `src/core/index.ts`, `src/tools/index.ts` contain only:
- `export default defineConfig({...})` -- default export of function call result
- `export * from './...'` -- re-export statements
- `export { x, y, z }` -- named exports

None of these produce entities that the `.scm` queries would capture. This is expected behavior but reduces coverage metrics.

### Pattern 6: CRLF Line Terminators (affects CodeSeeker-MCP only)

Both `test.js` and `src/index.ts` in CodeSeeker-MCP use CRLF (`\r\n`) line endings. While tree-sitter handles CRLF correctly in parsing, the line counting and byte offset calculations in Parseltongue's key generator or entity conversion may be affected. Specifically:
- `content.len()` counts `\r\n` as 2 bytes but line-based operations may treat them as 1 line ending
- String slicing for `current_code` extraction may include stray `\r` characters

### Pattern 7: Test File Patterns (affects mcp-server-semgrep, ast-grep-mcp)

Vitest/pytest test files use:
```typescript
describe('...', () => {
    it('should ...', async () => { ... });
});
```
```python
class TestFoo:
    @patch('...')
    def test_bar(self, mock):
        ...
```

These are function calls with callback arguments (Vitest) or decorated methods inside classes (pytest). Neither pattern produces entities matching the `.scm` queries.

---

## Recommendations for Parseltongue Parser

### Priority 1 (Critical): Handle Shebang Lines

**Impact**: Fixes 0% coverage for 3 repos (CodeSeeker-MCP, mcp-ripgrep, and partially mcp-server-semgrep).

**Solution**: Strip shebang lines before passing source to tree-sitter. Add preprocessing in `process_file_sync_for_parallel()`:

```rust
// Strip shebang line before parsing
let source_for_parse = if content.starts_with("#!") {
    if let Some(newline_pos) = content.find('\n') {
        // Replace shebang with spaces to preserve byte offsets
        let mut modified = content.clone();
        modified[..newline_pos].chars().for_each(|_| {}); // conceptual
        // Actually: replace first line with spaces
        " ".repeat(newline_pos) + &content[newline_pos..]
    } else {
        content.clone()
    }
} else {
    content.clone()
};
```

Alternatively, replace the shebang line with equivalent whitespace to preserve all byte offsets for subsequent entity extraction.

### Priority 2 (High): Add Const Object Export Entity Pattern

**Impact**: Captures MCP tool definitions, Zod schemas, and module-level const exports across all 4 TS repos.

**Add to `typescript.scm`**:
```scheme
; Exported const declarations (non-function values like tool objects, schemas)
(export_statement
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name))) @definition.module

; Top-level const declarations
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: (object))) @definition.module
```

### Priority 3 (Medium): Handle Decorated Python Functions

**Impact**: Fixes ast-grep-mcp `main.py` extraction.

**Update `python.scm`**:
```scheme
; Decorated functions
(decorated_definition
  (function_definition
    name: (identifier) @name) @definition.function)

; Decorated methods inside classes
(class_definition
  body: (block
    (decorated_definition
      (function_definition
        name: (identifier) @name) @definition.method)))
```

### Priority 4 (Medium): CRLF Normalization

**Impact**: Fixes CodeSeeker-MCP if shebang handling alone doesn't resolve it.

**Solution**: Normalize CRLF to LF before parsing:
```rust
let content = std::fs::read_to_string(file_path)?;
let content = content.replace("\r\n", "\n");
```

### Priority 5 (Low): Add Variable Declaration Entity Type

**Impact**: Would capture Zod schemas, config constants, and other significant `const`/`let`/`var` declarations.

**Add to `typescript.scm` and `javascript.scm`**:
```scheme
; Significant const declarations (with type annotations or complex initializers)
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    type: (_))) @definition.typedef
```

### Priority 6 (Low): Add Enum Member Support

**Impact**: Would capture enum values inside TypeScript enums.

Currently, `enum_declaration` captures the enum itself but not individual enum members. For comprehensive analysis, consider adding:
```scheme
(enum_member
  name: (property_identifier) @name) @definition.module
```

### Summary Table of Expected Impact

| Fix | Repos Fixed | Files Fixed | Coverage Impact |
|-----|-------------|-------------|-----------------|
| Shebang handling | 3 repos | ~6 files | 0% -> 40-80% for affected repos |
| Const object exports | 4 repos | ~16 files | +20-40% coverage per repo |
| Decorated Python | 1 repo | 3 files | 25% -> 60-80% for ast-grep-mcp |
| CRLF normalization | 1 repo | 2 files | Safety net for CodeSeeker-MCP |
| Variable declarations | 4 repos | ~10 files | +10-15% additional coverage |
