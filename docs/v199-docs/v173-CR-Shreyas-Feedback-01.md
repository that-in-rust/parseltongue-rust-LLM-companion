# Shreyas Doshi Evaluation: FEATURE-MATRIX-COMPARISON.md

> Date: 2026-02-14
> Framework: LNO (Leverage / Neutral / Overhead)
> Source: `competitor_research/FEATURE-MATRIX-COMPARISON.md`

**LNO Rating: NEUTRAL — useful reference artifact, but the effort-to-insight ratio is poor.**

---

## What It Does

336 lines. 15 sections. 60+ features. 16 tools compared across 13 categories. Culminates in a feature count scorecard.

## What's Genuinely Leverage

1. **Section 4 (Graph Analysis)** — This is the money chart. Parseltongue owns this entire category. 8 out of 11 features are PT-only. No competitor touches Tarjan SCC, PageRank, k-core, Leiden, SQALE, or blast radius. If you show one table to an investor or a blog reader, it's this one. This section justifies Parseltongue's existence in 11 rows.

2. **Section 14 (Competitive Positioning Summary)** — The "Direct Competitors" table with AiDex and Code Scalpel is the actual strategic output. Three rows that answer "who should we worry about and why." This is the section someone will actually use to make a product decision.

3. **The discovery that Code Scalpel has 48 features vs PT's 24** — This is uncomfortable information. Code Scalpel has security, refactoring, test generation, Z3 symbolic execution — entire capability categories where PT scores zero. That's the kind of honest finding that changes roadmap priorities.

## What's Overhead

1. **Sections 7-8-9 (Modification, Testing, Session)** — Parseltongue scores 0/0/0 in all three. These sections exist to make Code Scalpel look good and Parseltongue look bad. They don't inform any decision because PT isn't building refactoring tools or Z3 symbolic execution. Including them inflates the scorecard gap (48 vs 24) in a misleading way — it's comparing a graph analysis engine to a full IDE.

2. **8 grep/ripgrep tools analyzed individually** — grep-mcp (3 features), grep_app_mcp (3 features), mcp-ripgrep (4 features), CodeSeeker-MCP (5 features), kp-ripgrep-mcp (5 features). Five tools that all do the same thing (wrap ripgrep/grep for MCP) analyzed as if they're separate competitors. They're not. They're 5 implementations of 1 idea. One row — "ripgrep MCP wrappers (5 variants)" — covers all of them.

3. **greptimedb-mcp-server** — The doc itself admits "this is a time-series database tool, not a code analysis tool. Included for completeness." Completeness is overhead. If it's not a competitor, it shouldn't consume a column across 13 tables.

4. **ACP (Agent Client Protocol)** — A protocol spec, not a tool. Gets a column across all 13 tables, scores N/A on most. Should be a footnote, not a full column.

5. **Section 11 (UX & Resilience)** — "Vim mode", "Screenshot capture", "Slash commands." These are Gemini CLI and CodeCompanion features. PT is a server, not an editor. This section is noise for PT's competitive analysis.

6. **Section 13 (Performance Claims)** — 10 rows, 7 are "--" for most tools. Only PT, AiDex, and Code Scalpel have data. Should be 3 rows in the positioning summary, not a dedicated section.

## The Uncomfortable Math

```
16 tools compared × 60+ features × 13 categories = ~1,000 cells in tables

Cells that matter for PT's product decisions:
  - Graph analysis column (8 cells)
  - AiDex direct comparison (1 row)
  - Code Scalpel direct comparison (1 row)
  - MCP protocol row showing PT = "--" (1 cell — the gap we're fixing)
  = ~20 cells

Useful cell ratio: ~2%
```

## What The Doc Should Be

**A 50-line competitive brief with 3 sections:**

1. **PT's moat** — Graph analysis. Nobody else does it. Table with 8 features, 3 columns (PT, nearest competitor, gap).

2. **PT's gaps** — MCP (fixing in v1.7.3), security (not on roadmap), code modification (not on roadmap). Three sentences each: what the gap is, who fills it, whether it matters.

3. **The two real competitors** — AiDex (closest: tree-sitter + persistent index + token reduction) and Code Scalpel (broadest: security + refactoring + graph). One table each: PT advantage vs competitor advantage.

Everything else — the 5 ripgrep variants, greptimedb, ACP, vim mode, screenshot capture, the 1,000-cell matrix — is research theater.

## The Scorecard Problem

The Feature Count Scorecard (Section 15) is the most dangerous part. It makes Code Scalpel look 2x better than Parseltongue (48 vs 24). But 24 of Code Scalpel's 48 are in Security/Modification/Testing — categories PT deliberately doesn't compete in. It's like comparing a scalpel to a Swiss Army knife by counting blades. The scalpel wins at surgery.

A weighted scorecard limited to **categories PT competes in** (Parse, Search, Intelligence, Graph, Metrics, Protocol) tells a different story:

| Tool | Parse | Search | Intel | Graph | Metrics | Protocol | **Relevant Total** |
|------|:-----:|:------:|:-----:|:-----:|:-------:|:--------:|:------------------:|
| **Parseltongue** | 4 | 2 | 4 | 8 | 4 | 2 | **24** |
| **Code Scalpel** | 5 | 2 | 8 | 3 | 3 | 7 | **28** |
| **AiDex** | 7 | 4 | 3 | 0 | 3 | 4 | **21** |

PT is competitive. The gap is MCP (Protocol column), not capability.

## Shreyas Would Say

"You built a comprehensive matrix when you needed a competitive brief. The matrix took hours. The brief would take 20 minutes. The matrix tells you 1,000 things about 16 tools. The brief tells you the 3 things about 2 tools that actually change what you build next. Ship the brief. Archive the matrix."

---

## Part 2: Deep Competitor Research — What Can We Learn?

> Research conducted by general-purpose agent. Goal: go beyond feature checkboxes.
> For each competitor (including categories PT doesn't target), extract ideas that could enrich the Parseltongue experience.

*Research completed 2026-02-14 via general-purpose agent across 9 competitors.*
---

### Tier 1: Direct Competitors (Deep Dives)

---

### AiDex (CSCSoftware) — The "Index Once, Query Forever" Persistent Code Index

**GitHub**: [CSCSoftware/AiDex](https://github.com/CSCSoftware/AiDex) | TypeScript | MCP Server | 22 tools | MIT

**Architecture**:
AiDex is a TypeScript MCP server that pre-indexes codebases using Tree-sitter and stores results in SQLite (WAL mode). The data flow is: `project files -> Tree-sitter parse -> identifier extraction -> SQLite (.aidex/index.db) -> MCP stdio transport -> AI assistant`. It deliberately indexes *identifiers only*, not raw text. Searching for `log` returns the function `log()`, not occurrences in `catalog` or `logarithm`. The `.aidex/` directory contains `index.db` (the SQLite database) and optionally `summary.md`. Source code is organized as `src/commands/` (tool implementations), `src/db/` (SQLite wrapper), `src/parser/` (Tree-sitter integration), `src/server/` (MCP protocol handler).

Supports 11 languages: C#, TypeScript, JavaScript, Rust, Python, C, C++, Java, Go, PHP, Ruby. Almost identical to PT's 12.

**Clever Ideas**:

1. **The ~50 token response envelope**: AiDex's core value proposition is brutally simple. A grep for `PlayerHealth` returns 200 hits across 40 files (~2000+ tokens). AiDex returns `Engine.cs:45, Player.cs:23, UI.cs:156` (~50 tokens). This is not a technical achievement so much as a *framing achievement* — they defined "context waste" as the core metric and optimized for it. PT claims 99% token reduction (2-5K tokens vs 500K raw dumps), but AiDex's "50 tokens per query" is a punchier number because it measures per-query cost, not total codebase cost.

2. **Session Notes**: `aidex_note({ path: ".", note: "Test the glob fix after restart" })` — persisted in SQLite, survives session restarts. This is clever because it recognizes that AI coding sessions are *ephemeral* but human intent is not. The AI can write a note before ending a session, and the next session picks it up. PT has no equivalent — once a query is done, the context is gone.

3. **Task Backlog**: A full task management system embedded in the index database. Create, update, tag, prioritize, log history. Priorities (high/medium/low), statuses (`backlog -> active -> done | cancelled`), tags for categorization. The AI can say "found a bug in the parser, add it to the backlog" during analysis. This turns the code index into a lightweight project management tool — no Jira, no Trello, no context-switching.

4. **Time-based Filtering**: `aidex_query({ term: "render", modified_since: "2h" })` — supports relative time (`30m`, `2h`, `1d`, `1w`) and ISO dates. Answers questions like "What did I change in the last hour?" without touching git. This is genuinely useful for AI assistants that need to understand *recent* work, not historical work.

5. **Cross-Project Links**: `aidex_link` connects multiple indexed projects. Query across related repos in a single call. Useful for monorepo-adjacent architectures.

6. **Browser Viewer**: `aidex_viewer` opens an interactive project tree in the browser. This is a lightweight visualization that lets humans audit what the index contains without going through the AI. PT has no equivalent visual inspection tool.

7. **Auto-Setup Detection**: `aidex setup` auto-detects installed AI clients (Claude Code, Claude Desktop, Cursor, Windsurf, Gemini CLI, VS Code Copilot) and registers itself. Zero-friction onboarding.

8. **Screenshot Capture**: Cross-platform screenshot tool (`aidex_screenshot`) for fullscreen, window, or region. Lets the AI "see" what the user sees. Unusual for a code indexer — they are expanding beyond pure code analysis into visual context.

**What PT Can Steal**:

- **Session persistence layer**: PT should offer a `/session-notes-persist-store` endpoint. When an LLM is done analyzing a codebase, it can store findings, TODOs, and observations *in the database alongside the graph*. Next query session picks up where the last left off. This is low effort and high leverage — it turns PT from a stateless query tool into a stateful analysis companion.

- **Time-based filtering on entities**: PT already stores entities with file metadata. Adding `?modified_since=2h` to entity search endpoints would be trivial to implement but incredibly useful for "what changed recently?" workflows.

- **Per-query token reporting**: Every PT response should include a `token_estimate` field showing how many tokens the response would consume. AiDex's "50 tokens" metric is powerful marketing and useful for LLM context window management.

- **Auto-registration CLI**: A `parseltongue setup` command that auto-detects MCP clients and registers the server would reduce friction enormously for MCP adoption.

**What PT Should NOT Copy**:

- **Task management**: PT is a graph analysis server, not a project management tool. Adding task CRUD would dilute the focus. Tasks belong in Jira/Linear/GitHub Issues.
- **Screenshot capture**: Completely orthogonal to code graph analysis. The fact that AiDex has it suggests scope creep.
- **Browser viewer**: Tempting, but PT's HTTP endpoints already serve structured JSON. A frontend is a separate product, not a feature.

**UX Patterns**:

- Responses are extremely compressed: entity name, file path, line number. No surrounding code context unless explicitly requested via a separate tool (`aidex_signature`). This two-tier approach (overview first, detail on demand) reduces context consumption. PT could adopt this: a "summary mode" for all endpoints that returns entity keys and locations only, reserving full detail for `/code-entity-detail-view/{key}`.

---

### Code Scalpel (3D Tech Solutions) — The "Swiss Army Knife" MCP Server for AI Agents

**GitHub**: [3d-tech-solutions/code-scalpel](https://mcpservers.org/servers/3d-tech-solutions/code-scalpel) | Python | MCP Server | 22 tools (20 dev + 3 system) | MIT Community Edition

**Architecture**:
Code Scalpel is a Python MCP server with an "MCP-First Architecture" — designed from the ground up as an MCP server, not a wrapper around an existing library. It treats code as a **Graph (AST + PDG — Program Dependence Graph)**, giving agents deterministic tools for extraction, analysis, modification, and verification. The server exposes 22 tools across 6 categories: Surgical Extraction (6), Taint-Based Security (6), Safe Modification (4), Verification & Testing (4), Advanced Analysis (1), System Infrastructure (3). All 22 tools are available at every tier (Community/Pro/Enterprise) — tiers only limit analysis scope, not tool availability.

v3.3.0 is in beta with 4,700+ tests. v1.0 public release expected Q1 2026.

**Clever Ideas**:

1. **Taint Analysis with Source-Sink Tracking**: The `security_scan` and `cross_file_security_scan` tools implement genuine taint propagation — tracking data flow from user input sources (request parameters, file reads, network input) through variable assignments and function calls to dangerous sinks (SQL execution, command execution, path operations). Cross-file tracking follows taint across module boundaries. Claims <10% false positive rate (vs. Semgrep's 22.4% and Bandit's 31.7%, measured across 2,000+ repos). Covers 12+ CWE categories: SQL injection, XSS, command injection, path traversal, NoSQL injection, LDAP injection. This is impressive because most MCP code analysis tools stop at pattern matching; Code Scalpel tracks actual data flow.

2. **Z3 Symbolic Execution**: The `symbolic_execute` tool uses the Z3 theorem prover to explore all possible code paths deterministically — not just paths that are actually executed in tests. This is fundamentally different from test coverage: it mathematically proves what paths exist and what inputs trigger them. The `generate_unit_tests` tool then auto-creates tests from discovered execution paths. This is the most technically ambitious feature in any MCP server I found. Whether it works well in practice at scale is another question, but the ambition is real.

3. **Refactoring Simulation (Dry Run)**: `simulate_refactor` verifies that proposed changes preserve behavior *before* application. This is the "preview mode" for code modifications — the AI can ask "what would happen if I renamed this function?" and get a deterministic answer without touching any files. `rename_symbol` then does the actual project-wide rename with reference updates. The fact that they separated simulation from execution is a good architectural decision.

4. **k-Hop Graph Neighborhood**: `get_graph_neighborhood` extracts a k-hop neighborhood around a given entity. This is conceptually identical to PT's blast-radius analysis, but framed as a generic graph query primitive rather than a domain-specific feature. The naming is better — "graph neighborhood" is more intuitive than "blast radius" for developers who are not familiar with dependency analysis terminology.

5. **Unified Sink Detection (Polyglot)**: `unified_sink_detect` detects dangerous functions (sinks) across multiple languages with confidence thresholds. The "unified" part means a single tool handles Python's `os.system()`, JavaScript's `eval()`, PHP's `mysql_query()`, etc. This polyglot approach to security pattern detection is something PT could adapt for its entity classification.

6. **Cryptographic Policy Verification**: `verify_policy_integrity` and `code_policy_check` evaluate code against compliance standards with cryptographic integrity verification of policy files. This prevents policy tampering — a concern when AI agents are modifying code.

7. **Type Evaporation Scanning**: `type_evaporation_scan` detects TypeScript type system vulnerabilities — places where types are lost through `any` casts, unsafe assertions, or improper type guards. This is a niche but genuinely useful security check for TypeScript codebases.

**What PT Can Steal**:

- **Graph neighborhood as a first-class primitive**: PT already has `/blast-radius-impact-analysis?entity=X&hops=N`. Rename/alias this to also be queryable as `/graph-neighborhood-extraction-query?entity=X&hops=N`. Same implementation, better discoverability. Developers searching for "graph neighborhood" in MCP directories will find PT; they will not search for "blast radius."

- **Confidence scores on edges**: Code Scalpel's `unified_sink_detect` uses confidence thresholds. PT's dependency edges are currently binary (exists/doesn't exist). Adding a `confidence` field to edges (e.g., 0.9 for a direct function call, 0.5 for a type reference, 0.3 for a string-based import) would make graph analysis more nuanced and enable better filtering.

- **"What would break?" simulation endpoint**: A `/refactoring-impact-simulation-preview?rename=old_name&to=new_name` endpoint that uses PT's existing graph to show all entities that would be affected by a rename — without actually modifying anything. This leverages PT's graph infrastructure to provide simulation-like value without implementing actual code modification.

- **The deterministic tool framing**: Code Scalpel's philosophy is "deterministic tools, not AI guessing." `extract_function("process_payment")` instead of reading the file. PT already does this (query by entity key), but should emphasize this in marketing: "Parseltongue gives your AI deterministic graph queries, not string matching."

**What PT Should NOT Copy**:

- **Z3 symbolic execution**: Technically impressive but wildly out of scope for a graph analysis server. The implementation complexity is enormous and it serves a fundamentally different use case (test generation, not dependency analysis).
- **Code modification tools**: `update_symbol`, `rename_symbol` — Code Scalpel is building an IDE backend. PT is a read-only analysis tool. Adding write operations would change PT's entire security and reliability model.
- **Taint analysis engine**: Building a full taint propagation engine is a multi-month effort that competes with Semgrep and Bandit. PT's value is graph structure, not security scanning. Better to recommend users pair PT with Semgrep.
- **The "22 tools" approach**: Code Scalpel's 22 tools in MCP means 22 tool descriptions consuming context tokens just for discovery. PT should keep its tool count low and its endpoints focused.

**UX Patterns**:

- Every tool is framed as a verb + precise target: `extract_function("process_payment")`, `get_cross_file_dependencies("Order")`. The specificity is the UX — the AI never needs to guess file paths or line numbers. PT's endpoints are already well-named but could adopt this exact-entity-reference pattern more explicitly in MCP tool descriptions.
- Code Scalpel claims 99% lower token costs — essentially the same claim as PT. The difference is framing: Code Scalpel frames it as "your AI won't hallucinate because it gets deterministic data" (correctness-first), while PT frames it as "99% token reduction" (efficiency-first). Both are selling the same underlying value.

---

### Tier 2: Interesting Ideas

---

### Greptile MCP — AI-Powered Semantic Search as a Service

**GitHub**: [sosacrazy126/greptile-mcp](https://github.com/sosacrazy126/greptile-mcp) | TypeScript | MCP Server | 4 tools | MIT

**Architecture**:
Greptile MCP is a thin TypeScript MCP wrapper around the Greptile cloud API. It is NOT a local tool — it sends your code to Greptile's servers for indexing and semantic search. The server provides 4 tools: `index_repository` (trigger indexing), `query` (natural language code search with code references), `search` (return relevant files without full answers), and `get_repository_info` (check index status). Supports both stdio and HTTP transport (JSON-RPC 2.0). Requires a Greptile API key + GitHub/GitLab PAT.

**Clever Ideas**:

1. **Natural Language Code Queries**: You ask "How does authentication work?" and get an answer with specific code references, not just file matches. This is fundamentally different from keyword search — it understands semantic intent. However, this requires cloud processing and is not local.

2. **Session IDs for Conversational State**: Follow-up queries use session IDs to maintain context. "How does authentication work?" followed by "What about the token refresh?" — the second query understands it is still about authentication. This is conversation checkpointing at the API level.

3. **Multi-Repo Synthesis**: Query across multiple repositories simultaneously. "How do the frontend and backend handle user sessions?" can reference code from both repos in a single response.

4. **Two-Tier Response Model**: `query` returns a full answer with code references (expensive, high context). `search` returns just file paths (cheap, low context). The caller chooses the depth. This is the same "overview first, detail on demand" pattern AiDex uses.

**What PT Can Steal**:

- **Session IDs on PT endpoints**: Add an optional `?session_id=X` parameter to PT analysis endpoints. The server stores previous query results for that session, enabling follow-up queries that build on prior context. For example, `/blast-radius-impact-analysis?entity=X&hops=2&session_id=abc` followed by `/blast-radius-impact-analysis?entity=Y&hops=2&session_id=abc` could return a combined/comparative view.

- **Natural language query interface**: PT could add a `/natural-language-graph-query?q=who calls the payment handler?` endpoint that translates natural language into CozoDB queries. This is a high-effort feature but would dramatically improve accessibility for non-graph-experts. Could start with a simple keyword-to-entity-type mapping.

**What PT Should NOT Copy**:

- **Cloud-dependent architecture**: Greptile requires sending code to external servers. PT's local-first, zero-network-dependency model is a fundamental advantage. Never compromise this.
- **API key requirements**: PT should always work without external accounts or API keys.

**UX Patterns**:

- Zero-install via `npx greptile-mcp-server --api-key=X`. Instant start. PT's setup requires building from source and running two separate commands (pt01 then pt08). A single `parseltongue serve .` that handles both ingestion and serving would be a significant UX improvement.

---

### kp-ripgrep-mcp — The Obsidian-Aware Knowledge Graph Inspector

**GitHub**: [kpetrovsky/kp-ripgrep-mcp](https://github.com/kpetrovsky/kp-ripgrep-mcp) | Python | MCP Server | 5 tools | Obsidian-focused

**Architecture**:
A Python MCP server that wraps ripgrep with Obsidian-specific intelligence. It understands wiki links (`[[page]]`), markdown links, YAML frontmatter, and tags. The "backlink detection" and "orphaned node detection" features are not generic graph algorithms — they are Obsidian-specific link parsing.

**Clever Ideas**:

1. **Backlink Detection**: Given a note, find all other notes that link TO it. In code terms: reverse dependency lookup. In Obsidian terms: "who references this note?" This is conceptually identical to PT's `/reverse-callers-query-graph`, but for markdown documents.

2. **Orphaned Node Detection**: Find notes with zero incoming AND zero outgoing links. These are isolated nodes in the knowledge graph — forgotten content that needs integration. PT's graph already has this data; exposing it as a first-class endpoint would be valuable.

3. **Frontmatter Intelligence**: Parse YAML frontmatter and query by metadata properties. The equivalent for code would be querying by annotations, decorators, or attributes.

4. **Date Validation**: Validates date formats in frontmatter. Simple but prevents data quality issues.

**What PT Can Steal**:

- **Orphaned entity detection endpoint**: `/orphaned-entities-isolation-detection` — entities with zero incoming and zero outgoing edges. These are dead code candidates, unused imports, or disconnected modules. PT already has the graph data; this is a trivial query to expose. High value for code cleanup workflows.

- **"Graph health" diagnostics**: Inspired by the knowledge graph maintenance features — PT could offer a `/graph-health-diagnostic-report` endpoint that reports: orphaned nodes, strongly connected components (already have), node degree distribution, potential dead code, entities with unusually high fan-in/fan-out. A single endpoint for "tell me what is wrong with this codebase's structure."

**What PT Should NOT Copy**:

- Obsidian-specific features (wiki link parsing, frontmatter handling). These are domain-specific and irrelevant to code analysis.

**UX Patterns**:

- Token-efficient structured results designed for LLM consumption. Responses are compact and machine-parseable. PT already does this well with JSON responses.

---

### Semgrep MCP (Official) — The Security Scanner as an MCP Primitive

**GitHub**: [semgrep/mcp](https://github.com/semgrep/mcp) (deprecated; now `semgrep mcp` subcommand) | Python | MCP Server | 7 tools

**Architecture**:
Semgrep's official MCP server exposes 7 tools: `security_check` (quick vulnerability scan), `semgrep_scan` (scan with config string), `semgrep_scan_with_custom_rule` (scan with inline YAML rule), `get_abstract_syntax_tree` (dump AST), `semgrep_findings` (fetch from Semgrep AppSec Platform API), `supported_languages`, `semgrep_rule_schema`. Supports both stdio and SSE/streamable HTTP transport. The standalone MCP repo was deprecated — Semgrep now ships MCP as a built-in subcommand (`semgrep mcp`), which is a strong architectural signal: MCP is becoming a standard distribution mechanism for developer tools.

**Clever Ideas**:

1. **OpenTelemetry Tracing**: Semgrep supports `--trace` to emit OpenTelemetry traces from scans. The `--trace-endpoint=VAL` flag sends traces to any OTel collector. Currently used internally for `semgrep lsp`, but the principle is powerful: instrumenting static analysis with distributed tracing so you can debug performance bottlenecks, track scan durations, and integrate into observability stacks. Best practice guidance recommends emitting traces with `traceparent` header and redacting sensitive values.

2. **Metavariable Capture in Custom Rules**: Custom rules use `$X` metavariables (e.g., `$Z` for z-index values, `$X` for import paths) with configurable messages, languages, and severity levels. This is essentially parameterized pattern matching — the rule author defines the shape, and Semgrep captures the variable parts. This is similar to ast-grep's metavariables but integrated into a security-first workflow.

3. **AST Dump as a Tool**: `get_abstract_syntax_tree` exposes the AST directly to the AI agent. This is an underrated idea — instead of the tool making all the decisions about what to extract, it lets the AI inspect the raw structure and decide for itself. PT could offer a raw CozoDB query endpoint for advanced users.

4. **MCP as Built-in Subcommand**: Semgrep deprecated the separate MCP repo and folded it into the main binary (`semgrep mcp`). This is the right architecture — MCP should be a distribution mechanism, not a separate product. PT should consider `parseltongue mcp` as a subcommand.

5. **Path Security Validation**: The MCP server validates file paths to prevent path traversal attacks. When an AI agent asks to scan a file, the server ensures the path is within allowed boundaries. Critical for any tool that accepts file paths from AI agents.

**What PT Can Steal**:

- **OpenTelemetry integration**: Add `--trace` and `--trace-endpoint` flags to `pt08-http-code-query-server`. Every endpoint gets automatic span tracing: ingestion duration, query latency, CozoDB query time, response serialization time. This is valuable for both debugging and for demonstrating performance to users. Low effort (Rust has excellent OTel crates: `tracing` + `opentelemetry` + `tracing-opentelemetry`), high signal.

- **Raw CozoDB query endpoint**: `/raw-cozo-query-execution-direct?q=...` — Let advanced users and AI agents execute CozoDB Datalog queries directly. Dangerous but powerful. Gate behind a `--enable-raw-queries` flag. This is the equivalent of Semgrep's "AST dump" — give the AI the raw substrate and let it reason.

- **MCP as subcommand**: `parseltongue mcp` that wraps pt08's HTTP endpoints as MCP tools. Single binary, single command. No separate MCP adapter needed.

- **Path validation on all inputs**: Every PT endpoint that accepts file paths or entity keys should validate inputs to prevent path traversal or injection. Security hygiene.

**What PT Should NOT Copy**:

- **The security scanning domain**: Semgrep does security scanning. PT does graph analysis. These are complementary, not overlapping. PT should recommend Semgrep integration, not replicate it.
- **Semgrep AppSec Platform API integration**: Cloud platform dependency. Keep PT local-first.

**UX Patterns**:

- The `write_custom_semgrep_rule` reusable prompt is interesting — it is an MCP Prompt (not a tool) that helps the AI write Semgrep rules. PT could offer MCP Prompts for common analysis patterns: "How to find the most critical entity," "How to identify circular dependencies," "How to trace a call chain."

---

### Gemini CLI (Google) — The Terminal AI Agent with Enterprise-Grade Safety

**GitHub**: [google-gemini/gemini-cli](https://github.com/google-gemini/gemini-cli) | TypeScript | CLI Agent | 94K+ stars

**Architecture**:
Gemini CLI is Google's open-source terminal AI agent. Unlike the other tools in this analysis, it is not a code analysis tool — it is a full AI coding agent that happens to have excellent safety and extensibility infrastructure. The architecture relevant to PT includes: sandbox execution, git-based checkpointing, plan mode, sub-agent delegation, and a policy engine.

**Clever Ideas**:

1. **Git Shadow Repository for Checkpointing**: When the AI modifies files, Gemini CLI creates a full Git snapshot in a shadow repository (`~/.gemini/history/<project_hash>`). This does NOT interfere with the user's own Git history. The `/restore` command reverts files using this snapshot. Implementation lives in `packages/core/src/services/gitService.ts` — it sets `GIT_DIR` to the shadow repo and `GIT_WORK_TREE` to the project directory, then stages and commits all files. Three components per checkpoint: git snapshot, conversation history (JSON), and the tool call that was about to execute.

    This is genuinely brilliant for AI coding workflows. The AI can experiment freely, and the user can always roll back. Known issues: silently fails in non-git projects (should create its own repo regardless), and can cause massive disk usage if `.git` directory is included in snapshots (fixed by adding `.git` to shadow `.gitignore`). Uses `GIT_CONFIG_GLOBAL` to isolate shadow repo git configuration from user's real git config.

2. **Plan Mode**: A strict read-only mode for research and design. When enabled, the AI can read files and analyze code but cannot modify anything. The policy engine applies different rule sets based on mode (`yolo`, `autoEdit`, `plan`). Plan files are isolated per session. This is the "look but don't touch" workflow — perfect for initial codebase exploration before making changes.

3. **Sub-Agent Delegation**: Custom agents are defined as Markdown files with YAML frontmatter, placed in `.gemini/agents/*.md` (project-level, shared) or `~/.gemini/agents/*.md` (user-level, personal). Each sub-agent has its own system prompt, persona, and restricted tool set. Sub-agents appear to the main agent as callable tools. When invoked, the main agent delegates the task, the sub-agent executes in its own context, and reports back. Supports remote sub-agents via Agent-to-Agent (A2A) protocol.

    The implementation is clever: sub-agents run as independent `gemini-cli` processes in `--yolo` mode, coordinated by task files on disk. Each agent is a stateless worker — gets instructions, performs the job, exits. No complex process manager needed.

4. **Multi-Tier Sandbox**: Three sandbox options: macOS `sandbox-exec` (lightweight, restricts writes outside project directory), Docker containers (cross-platform, complete process isolation), and Docker Desktop v4.58+ kernel-isolated sandboxes (dedicated kernel per agent). Custom sandboxes via `.gemini/sandbox.Dockerfile`. The Docker sandbox mounts projects at the same absolute path as the host, so file paths are consistent inside and outside the sandbox.

5. **Policy Engine**: Rules-based system that controls what tools the AI can use and when. Rules can be scoped to specific modes, specific tools, and specific approval requirements. Custom deny messages explain why an action was blocked. This is enterprise-grade safety infrastructure.

6. **Agent Skills Standard**: A new extensibility model — skills are self-contained directories packaging instructions and resources. Three-tier progressive disclosure: Discovery (only metadata loaded), Activation (full instructions loaded on request), Execution (supporting assets available). Resolved at project, user, and extension levels.

**What PT Can Steal**:

- **Checkpoint-like state for analysis sessions**: PT could offer a `/checkpoint-analysis-session-state` endpoint that snapshots the current analysis state (which entities have been queried, what patterns were found, what the blast radius looked like). The AI can explore, then "restore" to review earlier findings. Implementation: store session state as CozoDB relations, keyed by session ID.

- **Read-only "plan mode" for PT queries**: Add a `?mode=plan` parameter that returns analysis results with explicit "this is read-only analysis, no modifications were made" framing. Useful for compliance-sensitive environments where AI interactions must be auditable.

- **Agent skill definitions**: PT could publish `.gemini/agents/parseltongue-analyst.md` files that teach Gemini CLI (and other agents) how to use PT's endpoints effectively. This is free marketing in the Gemini CLI ecosystem.

- **Progressive disclosure in API responses**: PT endpoints could adopt a three-tier response model: `?detail=summary` (entity counts and names only), `?detail=standard` (current default), `?detail=full` (include raw CozoDB data, edge weights, all metadata). This mirrors the progressive disclosure pattern from Agent Skills.

**What PT Should NOT Copy**:

- **Sandbox infrastructure**: PT is a read-only HTTP server. It does not modify files. Sandboxing is irrelevant.
- **Sub-agent orchestration**: PT is a tool that agents call, not an agent itself. Sub-agent delegation is for agent frameworks, not analysis servers.
- **The kitchen-sink feature scope**: 94K stars and Google resources allow Gemini CLI to build everything. PT should stay focused.

**UX Patterns**:

- The "slash command" pattern (`/restore`, `/plan`, `/subagents`) is an effective CLI UX for mode-switching. If PT ever builds a CLI REPL (beyond HTTP), slash commands would be natural for switching between analysis modes.
- Markdown-with-YAML-frontmatter as a configuration format for agent definitions is elegant and human-readable. PT could use this format for analysis presets or saved query templates.

---

### Tier 3: Quick Scans

---

### ast-grep MCP — Structural Pattern Matching Done Right

**GitHub**: [ast-grep/ast-grep](https://github.com/ast-grep/ast-grep) | Rust + Python MCP | 4 MCP tools

ast-grep is a Rust CLI for code structural search, lint, and rewriting at large scale. It uses Tree-sitter (same parser PT uses) to build ASTs, then matches patterns against the AST structure rather than text.

**The Key Insight**: Patterns are written as ordinary code with `$METAVAR` wildcards. `console.log($GREETING)` matches any `console.log()` call. `$A == $A` finds self-comparisons. `$$$ARGS` matches zero or more arguments. This is wildly more intuitive than regex-based pattern matching.

YAML rules compose atomic rules (`pattern`, `kind`, `regex`), relational rules (`inside`, `has`, `follows`, `precedes`), and composite rules (`any`, `all`, `not`, `matches`). Constraints filter metavariables: `constraints: { ARG: { kind: number } }` only matches numeric arguments.

Code rewriting uses the `fix` field: matched metavariables are substituted into templates. Example: rewriting `$PROP && $PROP()` to `$PROP?.()` converts old defensive code to optional chaining. Supports recursive rewriting via `rewriters`.

**What PT Can Steal**: Expose entity search with structural patterns. Instead of fuzzy string search, let users search by entity structure: "find all functions that call `db.query` and return a `Result`." PT has the AST data in CozoDB — the missing piece is a pattern language for querying it. Even a simplified version (e.g., `/structural-pattern-entity-search?pattern=fn:*:calls:db.query`) would be a differentiator.

**What to Skip**: Code rewriting. PT is read-only analysis.

---

### CodeCompanion.nvim — Multi-Adapter Architecture for LLM Backends

**GitHub**: [olimorris/codecompanion.nvim](https://github.com/olimorris/codecompanion.nvim) | Lua | Neovim Plugin

CodeCompanion is a Neovim plugin that supports every major LLM provider through a unified adapter system. The architecture has two adapter types: HTTP adapters (for stateless LLM APIs: Anthropic, OpenAI, Copilot, Ollama, Gemini, etc.) and ACP adapters (for stateful AI agents: Claude Code, Codex, Gemini CLI via Agent Client Protocol).

**The Key Insight**: Different interactions (Chat, Inline, Cmd, Background) can each use a different adapter. Use a cheap Ollama model for background title generation, Anthropic for chat, OpenAI for inline code completion. The adapter is the abstraction boundary — the rest of the plugin doesn't care which LLM is behind it.

Data flow: `User Input -> Interaction -> Adapter -> LLM -> Response Parser (handlers) -> Tools (optional) -> Display (Chat Buffer / Inline Buffer / Diff)`.

**What PT Can Steal**: PT's HTTP server could offer response format adapters. Same analysis, different output formats: `?format=json` (current), `?format=markdown` (human-readable), `?format=mcp` (MCP-native tool response), `?format=csv` (for spreadsheet analysis), `?format=dot` (for Graphviz visualization). The adapter pattern — same analysis engine, multiple output interfaces — is a good architectural principle.

**What to Skip**: Editor integration, chat buffers, inline completion. PT is a backend server.

---

### ACP (Agent Client Protocol) — The Other Side of the Stack

**GitHub**: [agentclientprotocol/agent-client-protocol](https://github.com/agentclientprotocol/agent-client-protocol) | Spec

There are actually **two different "ACPs"** causing confusion:

1. **Agent Client Protocol** (by Zed Industries): Standardizes communication between code editors and coding agents. Like LSP but for AI agents. Supported by Zed, JetBrains, Neovim, and Marimo. Agents: Claude Code, Codex CLI, Gemini CLI, StackPack, Goose. The registry is live — developers can discover and connect agents in their IDE.

2. **Agent Communication Protocol** (by IBM/BeeAI): Standardizes agent-to-agent communication. Has been merged into A2A under the Linux Foundation. Essentially defunct as a standalone spec.

**The Key Insight**: MCP standardizes the bottom of the stack (agent-to-tools), ACP standardizes the top (editor-to-agent). They are complementary. MCP answers "what data and tools can agents access?" ACP answers "where does the agent live in your workflow?" PT currently speaks HTTP. It should speak MCP (v1.7.3 roadmap). It does NOT need to speak ACP — that's for agents, not tools. But knowing that ACP exists means PT's MCP tools will be discovered by ACP-connected editors (Zed, JetBrains, Neovim) automatically, through the agent as intermediary.

**What PT Can Steal**: Nothing directly. But understanding the protocol stack (Editor <-> ACP <-> Agent <-> MCP <-> Tool) clarifies PT's position. PT is the rightmost node. It speaks MCP. It should be the best MCP tool for graph analysis, and let ACP-connected agents handle the editor integration.

---

## Part 3: Actionable Takeaways for Parseltongue

> Prioritized list of concrete improvements distilled from all competitor research.
> Rated by LNO framework. Sorted by leverage-to-effort ratio.

---

### LEVERAGE (Build These — They Multiply PT's Value)

**L1. Orphaned Entity Detection Endpoint** `[Effort: 1 day]` `[Inspired by: kp-ripgrep-mcp]`

Add `/orphaned-entities-isolation-detection` — entities with zero incoming AND zero outgoing edges. This is a single CozoDB query against the existing graph. Zero new infrastructure. Answers the question "what dead code exists?" which is one of the most common code analysis questions. Every competitor that has graph data exposes this; PT does not.

```
?[entity] := *edges{from: entity}, not *edges{to: entity}, not *edges{from: entity}
```

**L2. Per-Query Token Estimation in Responses** `[Effort: 2 hours]` `[Inspired by: AiDex]`

Add a `token_estimate` field to every JSON response. Calculate it as `response_bytes / 4` (rough token approximation). AiDex's "50 tokens per query" is their most effective marketing claim. PT should own "2-5K tokens per codebase analysis" with per-response proof. Every response becomes its own marketing material.

```json
{
  "data": [...],
  "meta": {
    "token_estimate": 847,
    "raw_codebase_tokens": 500000,
    "reduction_ratio": "99.8%"
  }
}
```

**L3. Time-Based Entity Filtering** `[Effort: 2 days]` `[Inspired by: AiDex]`

Add `?modified_since=2h` and `?modified_before=2026-01-15` parameters to entity list/search endpoints. PT already stores file paths with entities. Add file modification timestamps during ingestion (a single `fs::metadata().modified()` call per file). Support relative time formats (`30m`, `2h`, `1d`, `1w`) and ISO dates. Enables "what changed recently?" workflows without touching git.

**L4. OpenTelemetry Tracing** `[Effort: 3 days]` `[Inspired by: Semgrep MCP]`

Add `--trace` and `--trace-endpoint=URL` flags to `pt08-http-code-query-server`. Instrument every endpoint with OTel spans: request parsing, CozoDB query execution, response serialization. Use Rust crates `tracing` + `opentelemetry` + `tracing-opentelemetry`. This provides: (a) production debugging, (b) performance benchmarking evidence, (c) enterprise credibility. Semgrep built this; PT should too.

**L5. Session Persistence Layer** `[Effort: 3 days]` `[Inspired by: AiDex, Greptile]`

Add `/session-notes-persist-store` endpoint. Store key-value notes in CozoDB alongside the graph. AI agents write observations during analysis; next session retrieves them. Example: the AI discovers a circular dependency, writes `"circular dep between auth and user modules — needs investigation"`, and the next session starts with that context. Turns PT from a stateless query tool into a stateful analysis companion.

**L6. Graph Neighborhood Alias** `[Effort: 30 minutes]` `[Inspired by: Code Scalpel]`

Add `/graph-neighborhood-extraction-query?entity=X&hops=N` as an alias for `/blast-radius-impact-analysis`. Same implementation, different name. Developers searching MCP directories for "graph neighborhood" will find PT. "Blast radius" is domain jargon that assumes the user already knows what they want; "graph neighborhood" is self-explanatory.

**L7. MCP as Subcommand** `[Effort: 1 week]` `[Inspired by: Semgrep MCP]`

Implement `parseltongue mcp` that wraps pt08's HTTP endpoints as MCP tools via stdio transport. Single binary, single command. No separate MCP adapter project. This is the v1.7.3 roadmap item, but the architectural decision matters: build it INTO the main binary, not as a separate crate. Follow Semgrep's lead — they deprecated their separate MCP repo and folded it into `semgrep mcp`.

**L8. Progressive Disclosure Response Tiers** `[Effort: 2 days]` `[Inspired by: Gemini CLI, AiDex, Greptile]`

Add `?detail=summary|standard|full` parameter to all endpoints. Summary: entity keys and counts only (~50 tokens). Standard: current default. Full: include raw edge data, all metadata, CozoDB relation dumps. Three competitors independently converged on this pattern (AiDex's two-tier query/signature, Greptile's query/search split, Gemini's progressive disclosure). It is clearly the right approach for LLM-optimized APIs.

---

### NEUTRAL (Consider These — Good Ideas with Moderate ROI)

**N1. Multi-Format Response Adapters** `[Effort: 3 days]` `[Inspired by: CodeCompanion.nvim]`

Add `?format=json|markdown|csv|dot` to all endpoints. JSON is default. Markdown is human-readable. CSV enables spreadsheet analysis. DOT enables Graphviz rendering. The adapter pattern — same analysis, multiple outputs — is a clean architectural principle. But JSON is sufficient for most MCP/API use cases, so this is not urgent.

**N2. Auto-Setup CLI** `[Effort: 4 days]` `[Inspired by: AiDex]`

A `parseltongue setup` command that auto-detects installed MCP clients and registers PT. AiDex's `aidex setup` detects Claude Code, Claude Desktop, Cursor, Windsurf, Gemini CLI, VS Code Copilot. This reduces onboarding friction but only matters once MCP support is shipped (v1.7.3+).

**N3. Confidence Scores on Graph Edges** `[Effort: 1 week]` `[Inspired by: Code Scalpel]`

Add a `confidence` float to each edge in the graph. Direct function calls: 0.95. Type references: 0.7. String-based dynamic imports: 0.3. Decorator/attribute references: 0.5. This makes graph analysis more nuanced but requires changes to the core ingestion pipeline and all downstream query endpoints.

**N4. Graph Health Diagnostic Endpoint** `[Effort: 3 days]` `[Inspired by: kp-ripgrep-mcp]`

A single `/graph-health-diagnostic-report` endpoint that returns: orphaned nodes (L1), strongly connected components (existing), node degree distribution, entities with abnormally high fan-in/fan-out, potential dead code. Combines multiple existing analyses into one "codebase health check." Useful but not essential if individual endpoints exist.

**N5. Structural Pattern Search** `[Effort: 2 weeks]` `[Inspired by: ast-grep]`

A `/structural-pattern-entity-search?pattern=fn:*:calls:db.query` endpoint that lets users search the graph by entity structure rather than name. "Find all functions that call db.query and return a Result." PT has the graph data; the missing piece is a pattern language. High effort but would be a genuine differentiator.

**N6. Agent Skill File Publication** `[Effort: 1 day]` `[Inspired by: Gemini CLI]`

Publish `.gemini/agents/parseltongue-analyst.md` and similar agent definition files that teach AI agents how to use PT effectively. Free marketing in the Gemini CLI, Claude Code, and Cursor ecosystems. Minimal effort — it is just a well-written Markdown file with YAML frontmatter.

---

### OVERHEAD (Skip These — They Look Cool But Dilute PT's Focus)

**O1. Task Management / Project Management**: AiDex has it. PT should not. Graph analysis server, not Jira.

**O2. Code Modification / Refactoring Tools**: Code Scalpel has `update_symbol`, `rename_symbol`, `simulate_refactor`. PT is read-only. Adding write operations changes the entire security model.

**O3. Z3 Symbolic Execution**: Code Scalpel's crown jewel. Technically impressive but requires a theorem prover integration, fundamentally different use case (test generation), multi-month effort. Not PT's fight.

**O4. Taint Analysis Engine**: Code Scalpel and Semgrep own this space. PT should recommend pairing with Semgrep, not competing with it.

**O5. Screenshot Capture**: AiDex has it. It is scope creep even for AiDex.

**O6. Browser Viewer / Frontend**: Tempting. But PT serves JSON over HTTP — any frontend can consume it. Building a bespoke viewer is a maintenance burden. Let the community build viewers.

**O7. Cloud-Dependent Semantic Search**: Greptile requires cloud processing. PT's local-first model is a feature, not a limitation.

**O8. Sandbox / Process Isolation**: Gemini CLI needs it because it modifies files. PT is read-only. Sandboxing a read-only HTTP server is unnecessary complexity.

**O9. Sub-Agent Orchestration**: Gemini CLI delegates to sub-agents. PT is a tool that agents call, not an agent that delegates. Wrong layer of the stack.

---

### The Big Picture

The competitor landscape reveals three distinct product archetypes:

1. **Indexers** (AiDex, ripgrep wrappers): Parse codebases, store structured data, serve low-token responses. PT competes here on *depth* — they index identifiers, PT indexes relationships.

2. **Analyzers** (Code Scalpel, Semgrep MCP): Perform active analysis — security scanning, taint tracking, symbolic execution. PT competes here on *graph algorithms* — nobody else has Tarjan SCC, PageRank, k-core, Leiden, SQALE, or blast radius.

3. **Agents** (Gemini CLI, CodeCompanion): Orchestrate AI coding workflows with safety infrastructure. PT does NOT compete here — it is a tool that agents consume, not an agent itself.

PT's strategic position is clear: **the graph analysis engine that indexers wish they had and analyzers have not built.** The 8 graph algorithms are the moat. The improvements that matter most are the ones that make those algorithms easier to discover (L6, L8), easier to consume (L2, L7), and more useful in iterative workflows (L3, L5).

The single highest-leverage action from this entire research: **Ship MCP support (L7) with progressive disclosure (L8) and per-query token estimates (L2).** That combination makes PT's graph analysis discoverable by every AI agent, consumable at any context budget, and self-marketing with every response.
