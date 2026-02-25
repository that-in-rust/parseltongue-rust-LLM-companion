# CR-v173-04: oh-my-pi Competitive Deep-Dive

**Generated**: 2026-02-15
**Method**: cclsp + direct source reading (831 TypeScript files, 3 Rust crates)
**Repo**: https://github.com/can1357/oh-my-pi (fork of pi-mono by @mariozechner)
**Stack**: TypeScript (Bun) + Rust monorepo — 831 .ts files, 3 Rust crates, 13,843-line models.json
**Nature**: Full AI coding agent for terminal — direct competitor to Claude Code and Gemini CLI
**Package**: `@oh-my-pi/pi-coding-agent` (npm), version 12.4.0

---

## Feature 1: Multi-Provider LLM Client with Streaming

**ELI5**: oh-my-pi ships a unified LLM client that speaks to 10+ providers (Anthropic, OpenAI, Google, Google Vertex, Kimi, Cursor, Azure OpenAI) through a single `streamSimple()` function. It normalizes all provider-specific streaming formats (SSE, WebSocket) into a common `AssistantMessage` type with thinking blocks, tool calls, and usage tracking. A 13,843-line `models.json` defines pricing, context windows, and capabilities for every model. The client handles token cost calculation, cache retention (ephemeral/long/none), and retry logic with exponential backoff.

**Key Code Path**:
```
packages/ai/src/index.ts:1 -> exports all providers
  -> providers/anthropic.ts:1 -> Anthropic SDK with cache control
  -> providers/openai-responses.ts -> OpenAI Responses API
  -> providers/google.ts -> Gemini API
  -> providers/cursor.ts -> Cursor-specific exec handlers
  -> providers/kimi.ts -> Kimi Code (openai/anthropic format switch)
  -> models.ts:17 -> getModel(provider, modelId) from models.json registry
  -> stream.ts -> streamSimple() common streaming interface
  -> utils/event-stream.ts -> EventStream<T> async iterator
```

**Parseltongue comparison**: Parseltongue has zero LLM integration — it outputs JSON for any consumer. This is intentional: Parseltongue is a tool that LLMs call, not an LLM orchestrator. oh-my-pi IS the orchestrator.

---

## Feature 2: Agent Runtime with Steering & Follow-Up Queues

**ELI5**: The Agent class manages an event-driven agentic loop. The user sends a `prompt()`, the agent streams an LLM response, executes any tool calls, and loops until no more tools are needed. The unique feature is **steering** — the user can inject messages mid-execution via `agent.steer(message)` that interrupt after the current tool call finishes, skipping remaining tools. Follow-up messages are queued for after the agent stops naturally. The loop supports both "immediate" interrupt (check after each tool) and "wait" (defer until turn ends).

**Key Code Path**:
```
packages/agent/src/agent.ts:147 -> Agent class with #steeringQueue, #followUpQueue
  -> agent.ts:480 prompt() -> #runLoop(messages)
  -> agent-loop.ts:129 runLoop() -> outer loop (follow-ups) + inner loop (tool calls + steering)
  -> agent-loop.ts:242 streamAssistantResponse() -> convertToLlm -> streamFn
  -> agent-loop.ts:366 executeToolCalls() -> parallel with concurrency control
    -> tool.execute() with AbortSignal
    -> checkSteering() polls for interrupt messages
    -> concurrency: "exclusive" vs "shared" per tool
```

**Parseltongue comparison**: Parseltongue has no agent loop — it's a stateless HTTP API. The agent loop is a UI/orchestrator concern. However, the steering concept could inform Parseltongue's future streaming endpoints (e.g., cancel-in-progress query).

---

## Feature 3: Full MCP Client with Discovery, OAuth & Tool Caching

**ELI5**: oh-my-pi discovers MCP servers from `.mcp.json` files (project + user level), connects to them in parallel at startup with a 250ms timeout, and exposes their tools to the agent. If a server is slow to connect, cached tool definitions (from a previous session) are used as `DeferredMCPTool` proxies that resolve when the real connection lands. Supports stdio + HTTP + SSE transports. OAuth credentials are resolved from AuthStorage with rotating token support.

**Key Code Path**:
```
coding-agent/src/mcp/manager.ts:80 -> MCPManager class
  -> manager.ts:104 discoverAndConnect() -> loadAllMCPConfigs() + connectServers()
  -> manager.ts:118 connectServers() -> parallel Promise.race with 250ms timeout
  -> tool-bridge.ts -> MCPTool (live) and DeferredMCPTool (cached proxy)
  -> client.ts -> connectToServer(), listTools(), disconnectServer()
  -> transports/stdio.ts + transports/http.ts -> MCP transport implementations
  -> config.ts -> loadAllMCPConfigs() from .mcp.json
  -> oauth-flow.ts -> OAuth2 PKCE flow for MCP servers
  -> tool-cache.ts -> persist tool definitions between sessions
```

**Parseltongue comparison**: Parseltongue is an MCP **server** candidate (P0-1 in our roadmap: `pt09-mcp-protocol-bridge-server`). oh-my-pi is an MCP **client**. They're complementary — oh-my-pi would connect to Parseltongue's MCP server once pt09 is built.

---

## Feature 4: Built-in LSP Integration (diagnostics, definition, references, rename, format)

**ELI5**: oh-my-pi starts language servers for the project at startup, detects servers from file extensions (Rust: rust-analyzer, TypeScript: tsconfig -> tsc, Go: go-mod, Python: pyright), and provides a single `LspTool` that the agent calls with actions: `definition`, `references`, `hover`, `symbols`, `diagnostics`, `rename`, `reload`, `status`. Every file write goes through an "LSP writethrough" that syncs content to the LSP server, optionally formats it, gets diagnostics, and writes to disk — all in one atomic operation. Custom linter clients (Biome, SwiftLint) extend the system.

**Key Code Path**:
```
coding-agent/src/lsp/index.ts:853 -> LspTool class (name: "lsp")
  -> lsp/client.ts -> getOrCreateClient(), ensureFileOpen(), syncContent(), sendRequest()
  -> lsp/config.ts -> loadConfig(), getServersForFile()
  -> lsp/edits.ts -> applyTextEditsToString(), applyWorkspaceEdit()
  -> lsp/clients/biome-client.ts -> BiomeLinterClient (custom non-LSP linter)
  -> lsp/clients/swiftlint-client.ts -> SwiftLintClient
  -> lsp/lspmux.ts -> detectLspmux() for multiplexed LSP
  -> createLspWritethrough() -> atomic sync+format+diagnostics+write
```

**Parseltongue comparison**: Parseltongue uses tree-sitter for parsing (no LSP). LSP provides runtime type information, go-to-definition, and live diagnostics that tree-sitter can't. However, Parseltongue's CozoDB graph queries cover the same static analysis use cases (callers, callees, blast radius) without requiring a running language server. The approaches are complementary — LSP is real-time, Parseltongue is pre-computed.

---

## Feature 5: Swarm Extension — DAG-Based Multi-Agent Orchestration

**ELI5**: The swarm extension defines multi-agent workflows in YAML. Each agent has a `task`, `model`, `waits_for`, and `reports_to`. A DAG builder constructs a dependency graph, detects cycles via Kahn's algorithm, and produces execution **waves** (groups of agents that can run in parallel). A `PipelineController` executes waves sequentially — all agents in wave N run in parallel, then wave N+1 starts. Supports `pipeline` mode (repeat iterations) and `sequential` mode (chain by declaration order). State tracking persists per-agent status, iteration, and orchestrator logs.

**Key Code Path**:
```
packages/swarm-extension/src/swarm/
  -> dag.ts:17 buildDependencyGraph() from waits_for + reports_to
  -> dag.ts:63 detectCycles() via Kahn's algorithm
  -> dag.ts:106 buildExecutionWaves() via topological sort
  -> pipeline.ts:46 PipelineController class
    -> pipeline.ts:57 run() -> for each iteration -> #runIteration()
    -> pipeline.ts:124 #runIteration() -> for each wave -> Promise.all(agents)
  -> executor.ts -> executeSwarmAgent() subprocess runner
  -> state.ts -> StateTracker with agent/pipeline status
  -> schema.ts -> SwarmDefinition type (parsed from YAML)
```

**Parseltongue comparison**: Parseltongue already computes dependency graphs, SCCs, and topological ordering in CozoDB. The DAG/wave execution pattern is an orchestration concern that builds on exactly the kind of graph analysis Parseltongue provides. Future potential: Parseltongue could expose a `/topological-execution-wave-planner` endpoint that generates execution waves for any set of entities with dependencies.

---

## Feature 6: Task System with Git Worktree Isolation

**ELI5**: The Task tool delegates work to subagent processes. Each task gets a unique ID, runs in a subprocess with its own model/tools/skills, and reports progress via JSON events. The killer feature is **git worktree isolation**: when `isolated: true`, each task gets its own git worktree (a separate working copy), changes are captured as patches, and patches are applied back to the main worktree after all tasks complete. This allows truly parallel code editing without merge conflicts during execution.

**Key Code Path**:
```
coding-agent/src/task/index.ts:134 -> TaskTool class
  -> task/discovery.ts -> discoverAgents() from bundled + ~/.omp/agent/agents/*.md + .omp/agents/*.md
  -> task/executor.ts -> runSubprocess() with JSON event progress
  -> task/worktree.ts -> ensureWorktree(), captureBaseline(), captureDeltaPatch(), applyBaseline()
  -> task/parallel.ts -> mapWithConcurrencyLimit() for controlled parallelism
  -> task/template.ts -> renderTemplate() with Handlebars
  -> task/output-manager.ts -> AgentOutputManager for unique artifact IDs
```

**Parseltongue comparison**: No equivalent in Parseltongue. This is orchestrator-layer functionality. However, the blast radius analysis (`/blast-radius-impact-analysis`) could help intelligent task decomposition by identifying which files/entities can be safely modified in parallel without overlapping dependencies.

---

## Feature 7: Session Compaction with File Operation Tracking

**ELI5**: When the conversation gets too long, oh-my-pi "compacts" it by summarizing older turns into a condensed summary. It tracks which files were read and modified throughout the session, carries those file lists forward across compactions, and includes them in the summary. The compaction uses a dedicated summarization prompt and a smaller model to generate concise summaries. Branch summarization handles conversation forks.

**Key Code Path**:
```
coding-agent/src/session/compaction/
  -> compaction.ts:1 -> pure functions for compaction logic
  -> compaction.ts:42 extractFileOperations() from messages + previous compaction
  -> utils.ts -> serializeConversation(), formatFileOperations()
  -> branch-summarization.ts -> generateBranchSummary()
  -> coding-agent/src/session/session-manager.ts -> SessionManager with JSONL persistence
```

**Parseltongue comparison**: Parseltongue's smart context budget (`/smart-context-token-budget?focus=X&tokens=N`) serves a similar purpose — fitting the most relevant information into a limited token window. The session compaction pattern could inspire Parseltongue to track "analysis session state" (which entities were queried, what was found) for smarter follow-up queries.

---

## Feature 8: Extension System (Custom Tools, Commands, Skills, Slash Commands)

**ELI5**: oh-my-pi has a full extension system with 4 extension types: (1) **Extensions** subscribe to lifecycle events (tool calls, turn starts, bash/python execution) and can register tools, commands, and shortcuts; (2) **Custom Tools** are LLM-callable functions with TypeBox schemas; (3) **Skills** are markdown files with frontmatter (name, description, tools) loaded from `.omp/skills/`; (4) **Slash Commands** are file-based commands triggered by `/command` in the chat input.

**Key Code Path**:
```
coding-agent/src/extensibility/
  -> extensions/types.ts -> ExtensionContext, ExtensionActions, ExtensionUIContext
  -> extensions/ -> ExtensionRunner, ExtensionRuntime, loadExtensions
  -> custom-tools/ -> CustomToolLoader, discoverAndLoadCustomTools
  -> skills/ -> loadSkills(), loadSkillsFromDir(), SkillFrontmatter
  -> slash-commands/ -> loadSlashCommands(), FileSlashCommand
  -> hooks/ -> legacy hook system (re-exported)
```

**Parseltongue comparison**: Parseltongue has no plugin/extension system — it's a focused code analysis tool. Extensions would only make sense if Parseltongue evolved into a platform (P3 territory, intentionally deferred).

---

## Feature 9: Native Rust Bindings (grep, glob, image, shell, PTY, highlight)

**ELI5**: Performance-critical operations are written in Rust and exposed via N-API bindings. The `pi-natives` crate provides: (1) ripgrep-based regex search with context lines; (2) glob file discovery with caching; (3) photon-compatible image processing; (4) text utilities (visible width, ANSI truncation, text wrapping); (5) syntax highlighting via tree-sitter; (6) `brush-core` vendored shell for in-process shell execution; (7) PTY session management; (8) process tree management (kill, list descendants).

**Key Code Path**:
```
packages/natives/src/
  -> grep.ts -> grep(), searchContent(), fuzzyFind() backed by Rust
  -> glob.ts -> glob() with FsScanCache invalidation
  -> image.ts -> PhotonImage, SamplingFilter
  -> text.ts -> visibleWidth(), truncateToWidth() via Rust
  -> highlight.ts -> highlightCode() via tree-sitter
  -> shell.ts -> Shell class wrapping brush-core-vendored
  -> pty.ts -> PtySession for terminal emulation
  -> ps.ts -> killTree(), listDescendants()
crates/
  -> pi-natives/ -> Rust N-API bindings
  -> brush-core-vendored/ -> Vendored brush shell
  -> brush-builtins-vendored/ -> Shell builtins
```

**Parseltongue comparison**: Parseltongue is already fully Rust. Where oh-my-pi has to bridge TypeScript->Rust via N-API for performance, Parseltongue gets Rust performance natively. The vendored `brush-core` shell is interesting — Parseltongue could potentially embed shell execution for automated analysis workflows.

---

## Feature 10: Stats Dashboard (Local Observability)

**ELI5**: The `omp-stats` package provides a local observability dashboard at `http://localhost:3847`. It syncs JSONL session files into an SQLite database, computes aggregated statistics (total tokens, cost, cache rate, error rate, avg TTFT, tokens/sec) broken down by model and project folder, and serves a web dashboard. Available as CLI (`omp-stats --json`, `omp-stats --sync`, `omp-stats --port 8080`).

**Key Code Path**:
```
packages/stats/src/
  -> index.ts -> CLI entry point, syncAllSessions(), getDashboardStats()
  -> aggregator.ts -> session file -> SQLite ingestion
  -> db.ts -> SQLite database
  -> server.ts -> HTTP dashboard server
  -> types.ts -> DashboardStats, ModelStats, TimeSeriesPoint
```

**Parseltongue comparison**: No equivalent. Parseltongue computes code complexity metrics, not usage metrics. However, the pattern of "session JSONL -> SQLite -> dashboard" is similar to Parseltongue's "code -> CozoDB -> HTTP API" pipeline. Could inspire a `/analysis-session-statistics-dashboard` endpoint for tracking Parseltongue usage.

---

## Summary Table

| # | Feature | vs Parseltongue | Parseltongue Needed? | Shreyas: User Segment | Shreyas: Differentiation |
|---|---------|-----------------|----------------------|-----------------------|--------------------------|
| 1 | Multi-Provider LLM Client | PT outputs JSON, not LLM calls | No — intentionally excluded (P3-6 rationale) | Individual developers | Commoditized (every AI tool does this) |
| 2 | Agent Runtime + Steering | PT is stateless HTTP API | No — orchestrator concern | Individual developers | Neutral (good UX, not defensible) |
| 3 | MCP Client + OAuth + Caching | PT should be MCP SERVER | Yes — pt09 MCP server (P0-1, already planned) | Enterprise + Solo devs | Leverage (ecosystem unlock) |
| 4 | LSP Integration | PT uses tree-sitter (static) | Partial — tree-sitter covers 80% | Professional developers | Neutral (LSP is standard) |
| 5 | Swarm DAG Orchestration | PT HAS the graph data for this | Future — expose wave planner endpoint | Platform teams | Vitamin (nice, not essential) |
| 6 | Git Worktree Isolation | Not applicable | No — orchestrator concern | Enterprise teams | Defensible (complex to build) |
| 7 | Session Compaction | PT has `/smart-context-token-budget` | Evolve — track analysis sessions | All users | Neutral (context management) |
| 8 | Extension System | PT has no plugins | No — platform concern (P3) | Power users | Vitamin (ecosystem play) |
| 9 | Rust Native Bindings | PT is already 100% Rust | No — PT already has this natively | Performance-sensitive | Leverage for PT (native advantage) |
| 10 | Stats Dashboard | No usage tracking | Optional P2 — analysis telemetry | Enterprise | Vitamin (nice to have) |

---

## Shreyas Doshi LNO Assessment

**Leverage features** (move needle): MCP server (Feature 3 validates our P0-1 priority — every serious tool needs MCP)
**Neutral features** (expected): LSP integration, session compaction, agent runtime
**Overhead features** (avoid): Multi-LLM client, extension system, stats dashboard (for Parseltongue's positioning)

---

## Key Takeaway

oh-my-pi validates our existing roadmap rather than introducing new P0 priorities. It's an **MCP client** (consumer) while Parseltongue should be an **MCP server** (provider). The swarm DAG orchestration (Feature 5) is the most interesting intersection — oh-my-pi builds DAGs for agent execution, while Parseltongue already computes dependency DAGs for code. A future `/topological-execution-wave-planner` endpoint could make Parseltongue the graph engine that tools like oh-my-pi call for intelligent task decomposition.

The 831 .ts files and 3 Rust crates represent a massive investment (~50K+ LoC estimated) in building an orchestrator. Parseltongue's strategy of being the best **analysis backend** that orchestrators call remains correct. Building a competing orchestrator would be 6+ months of work with no defensible moat.

---

*Generated 2026-02-15. Analysis based on oh-my-pi v12.4.0 source code reading via cclsp + direct file analysis.*
