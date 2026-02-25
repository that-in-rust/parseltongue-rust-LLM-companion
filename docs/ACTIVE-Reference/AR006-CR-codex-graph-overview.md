# CR-codex-graph-overview.md
# Phase 1: Codex CLI Graph Overview
# Date: 2026-02-19
# Parseltongue server: port 7780
# Database: rocksdb:parseltongue20260219195022/analysis.db

---

## 1. Codebase Statistics

| Metric | Value |
|---|---|
| Total code entities | 15,901 |
| Dependency edges | 136,130 |
| Languages detected | Rust, TypeScript, JavaScript, Python, C |
| Database path | rocksdb:parseltongue20260219195022/analysis.db |
| Circular dependencies | 0 (confirmed clean) |
| SCC count | 34,933 (all size-1, meaning no real cycles in resolved code) |
| Leiden clusters | 460 (largest cluster has 16,748 members) |
| Ingestion coverage | 60.96% (854 of 1,401 eligible files parsed) |
| Parse errors | 554 files failed ingestion |

### Entity Breakdown by Language

| Language | Entity Count | Notes |
|---|---|---|
| Rust | 14,324 | Core + TUI + CLI + 40+ crates |
| TypeScript | 627 | codex-cli frontend + sdk/typescript |
| C | 386 | vendor/bubblewrap Linux sandbox |
| Python | 326 | windows-sandbox-rs smoke tests + SDK |
| JavaScript | 238 | codex-cli transpiled artifacts |
| **Total** | **15,901** | |

---

## 2. Repo Structure Map

```
codex/
├── codex-cli/           TypeScript CLI frontend (legacy)
├── codex-rs/            Rust core - 40+ crates
│   ├── core/            Main agent loop and turn processing
│   ├── tui/             Terminal UI (Ratatui-based)
│   ├── cli/             CLI binary dispatcher
│   ├── app-server/      HTTP JSON-RPC app server
│   ├── app-server-protocol/  Shared protocol types
│   ├── exec/            Shell command execution engine
│   ├── exec-server/     Exec server with sandbox escalation
│   ├── execpolicy/      Command allowlist/policy (new)
│   ├── execpolicy-legacy/ Policy engine (old, TOML-based)
│   ├── mcp-server/      MCP server implementation
│   ├── rmcp-client/     Remote MCP client with OAuth
│   ├── protocol/        Shared protocol types
│   ├── network-proxy/   Network egress proxy
│   ├── linux-sandbox/   Linux seatbelt/bubblewrap
│   ├── windows-sandbox-rs/ Windows AppContainer sandbox
│   ├── codex-api/       REST API server
│   ├── codex-client/    HTTP client for Codex backend
│   ├── hooks/           Pre/post-turn hook system
│   ├── otel/            OpenTelemetry observability
│   ├── config/          Config loading/merging
│   ├── state/           Conversation state management
│   ├── chatgpt/         ChatGPT-compatible mode
│   ├── apply-patch/     Patch application engine
│   ├── file-search/     File search tool
│   ├── cloud-tasks/     Cloud task execution
│   ├── lmstudio/        LM Studio local model integration
│   ├── ollama/          Ollama local model integration
│   ├── responses-api-proxy/ Responses API proxy
│   ├── backend-client/  Backend HTTP client
│   ├── debug-client/    Debug CLI client
│   └── vendor/bubblewrap  Linux namespace sandbox (C)
├── sdk/                 Python + TypeScript SDKs
└── shell-tool-mcp/      Shell tool as MCP server
```

---

## 3. Top Complexity Hotspots (Coupling Ranking)

### High-Outbound Files (Fan-Out, Design Complexity Hubs)

| Rank | Entity | Outbound Edges | Health Grade | Notes |
|---|---|---|---|---|
| 1 | `codex-rs/core/src/codex.rs` | 1,124 | F | Main agent loop - monolithic hub |
| 2 | `codex-rs/tui/src/chatwidget.rs` | 1,075 | F | Full chat UI state machine |
| 3 | `codex-rs/app-server/src/codex_message_processor.rs` | 929 | F | HTTP server message handler |
| 4 | `codex-rs/tui/src/app.rs` | 585 | F | TUI application root |
| 5 | `codex-rs/tui/src/bottom_pane/chat_composer.rs` | 544 | F | Chat input composer |

### High-Inbound Functions (Fan-In, Dependency Magnets)

| Rank | Function | Inbound Count | Notes |
|---|---|---|---|
| 1 | `new` (unresolved) | 3,425 | Constructor pattern - used everywhere |
| 2 | `Some` (unresolved) | 2,710 | Option construction |
| 3 | `to_string` (unresolved) | 2,549 | String conversion |
| 4 | `Ok` (unresolved) | 2,323 | Result construction |
| 5 | `clone` (unresolved) | 2,088 | Clone trait - Arc/data copying |
| 6 | `expect` (unresolved) | 1,400 | Panic-on-None - signals tech debt |
| 7 | `map` (unresolved) | 1,192 | Iterator transformation |
| 8 | `path` (unresolved) | 1,030 | Path handling - file I/O heavy |
| 9 | `join` (unresolved) | 903 | Path join - file system ops |
| 10 | `iter` (unresolved) | 895 | Iteration pattern |

**Key insight**: The top inbound entities are all unresolved Rust stdlib methods, confirming Codex RS is idiomatically Rust. The `expect` count (1,400) suggests areas where panic-on-failure is still used rather than proper error propagation - a refactoring target.

---

## 4. PageRank - Central Entities

Top resolved (non-stdlib) entities by PageRank score:

| Entity | Score | Significance |
|---|---|---|
| `rust:fn:new:unresolved-reference:0-0` | 0.01139 | Constructor gravity |
| `rust:fn:Some:unresolved-reference:0-0` | 0.00887 | Option pattern |
| `rust:fn:to_string:unresolved-reference:0-0` | 0.00788 | String conversion |
| `typescript:module:Array:0-0` | 0.00139 | TS array type - central to codex-cli |

**Note**: The unresolved references dominating PageRank are expected for Rust (trait method dispatch). The key insight is that `typescript:module:Array:0-0` appearing at rank 20 confirms the TypeScript codex-cli frontend treats array manipulation as a core primitive.

---

## 5. K-Core Decomposition (Architecture Layering)

**Max coreness**: 33

The k-core analysis reveals the innermost architectural kernel (coreness=33):

| Core Files | Layer | Significance |
|---|---|---|
| `codex-rs/codex-api/src/sse_responses.rs` | CORE (33) | SSE streaming protocol |
| `codex-rs/core/src/shell_snapshot.rs` | CORE (33) | Shell state snapshots |
| `codex-rs/core/src/models_manager/manager.rs` | CORE (33) | Model registry hub |
| `codex-rs/core/tests/suite/compact.rs` | CORE (33) | Context compaction tests |
| `codex-rs/core/tests/suite/view_image.rs` | CORE (33) | Multimodal image tests |
| `codex-rs/mcp-server/tests/suite/codex_tool.rs` | CORE (33) | MCP tool integration tests |
| `codex-rs/protocol/src/protocol.rs` | CORE (33) | Core protocol definitions |
| `codex-rs/app-server-protocol/src/export.rs` | CORE (33) | App server protocol exports |
| `codex-rs/tui/src/` (many files) | CORE (33) | TUI kernel |

**Insight**: The k-core inner ring reveals that SSE streaming, model management, and MCP tooling are the true architectural kernel - not just the codex.rs agent loop. The shell snapshot system being in the core suggests conversation persistence/replay is a first-class feature.

---

## 6. SCC Analysis

- **Total SCCs**: 34,933
- **Cycles detected**: 0
- **All SCCs**: Size 1 (no actual circular dependencies in resolved code)
- **Risk level**: NONE across all components

This confirms Codex RS has excellent architectural discipline - zero circular dependencies across its 40+ crate system. This is notable for a codebase of this scale.

---

## 7. Leiden Community Clusters

- **Total entities in clustering**: 34,933 (including unresolved references)
- **Cluster count**: 460
- **Largest cluster**: 16,748 members (the main Rust ecosystem cluster)

The largest cluster encompasses the majority of Codex RS functionality, indicating a tightly integrated (not microservice-style) architecture. This is intentional for an agent system where all components must collaborate closely on turn processing.

---

## 8. Key Architectural Subsystems (from Entity Search)

### 8.1 Sandbox Architecture (Multi-Platform)

The graph reveals three distinct sandbox implementations:

| Platform | Implementation | Key Entities |
|---|---|---|
| Linux | bubblewrap (C vendored) | `c:fn:acquire_privs`, `c:fn:do_init`, `c:fn:drop_all_caps` |
| Linux | seatbelt (Rust) | `codex-rs/linux-sandbox/src/` (67% coverage) |
| Windows | AppContainer (Rust+Python) | `codex-rs/windows-sandbox-rs/src/` (67% coverage) |
| macOS | seatbelt profile | Referenced in core config |

Key sandbox enums:
- `rust:enum:SandboxCommand` (cli/src/main.rs) - CLI entry for sandbox debugging
- `rust:enum:SandboxType` (cli/src/debug_sandbox.rs) - sandbox type selection
- `rust:enum:SetupMode` (windows-sandbox-rs/src/setup_main_win.rs) - Windows setup modes

### 8.2 MCP (Model Context Protocol) System

The graph shows a full MCP stack:

| Component | File | Role |
|---|---|---|
| MCP server | `codex-rs/mcp-server/src/` | Codex as MCP server |
| MCP client | `codex-rs/rmcp-client/src/` | Remote MCP client with OAuth |
| MCP auth | `codex-rs/core/src/mcp/auth.rs` | OAuth login support |
| MCP exec approval | `codex-rs/mcp-server/src/exec_approval.rs` | Exec approval flow |
| MCP patch approval | `codex-rs/mcp-server/src/patch_approval.rs` | Patch approval flow |
| MCP connection mgr | `codex-rs/core/src/mcp_connection_manager.rs` | Multi-server management |

Key enum: `rust:enum:McpOAuthLoginSupport` signals OAuth-gated MCP server connections.

### 8.3 Tool System Architecture

| Tool Type | File | Notes |
|---|---|---|
| Shell execution | `core/src/tools/runtimes/shell.rs` | Shell command runner |
| Patch application | `core/src/tools/runtimes/apply_patch.rs` | File edit tool |
| File search | `codex-rs/file-search/src/` | File search tool |
| Function tools | `core/src/function_tool.rs` | Dynamic function dispatch |
| Tool registry | `core/src/tools/registry.rs` | Tool kind registry |
| Tool events | `core/src/tools/events.rs` | ToolEmitter, ToolEventStage |
| Dynamic tools | `protocol/src/dynamic_tools.rs` | Runtime tool registration |

### 8.4 Approval/Safety Policy System

Multi-layer approval architecture:

| Layer | Component | Notes |
|---|---|---|
| CLI | `utils/cli/src/approval_mode_cli_arg.rs` | `ApprovalModeCliArg` enum |
| Protocol | `protocol/src/approvals.rs` | `ElicitationAction`, `NetworkApprovalProtocol`, `ExecPolicyAmendment` |
| Exec policy (new) | `execpolicy/src/` | `Decision` enum, rule-based |
| Exec policy (legacy) | `execpolicy-legacy/src/` | TOML-based allowlist |
| MCP server | `mcp-server/src/exec_approval.rs` | MCP approval handler |
| TUI | `tui/src/bottom_pane/bottom_pane_view.rs` | UI approval prompt |

### 8.5 Multi-Model Support

| Model Provider | Crate | Notes |
|---|---|---|
| OpenAI (default) | `codex-rs/core/` | Primary backend |
| LM Studio | `codex-rs/lmstudio/` | Local model via LM Studio API |
| Ollama | `codex-rs/ollama/` | Local model via Ollama API |
| ChatGPT | `codex-rs/chatgpt/` | ChatGPT-compatible mode |
| Cloud | `codex-rs/cloud-tasks/` | Cloud task execution |

---

## 9. Technical Debt Indicators

### From SQALE Scoring (core/src/codex.rs)

```
Total debt: 14.0 hours
Violations:
  HIGH_COUPLING: CBO=1124 (threshold: 10)  -> 4 hours remediation
  LOW_COHESION:  LCOM=1.0 (threshold: 0.8) -> 8 hours remediation
  HIGH_COMPLEXITY: WMC=1124 (threshold: 15) -> 2 hours remediation
Health grade: F
```

`codex.rs` is a ~6000 line God Object that should be decomposed. The file uses every symbol in the codebase (1,124 outbound deps).

### From SQALE Scoring (app-server/src/codex_message_processor.rs)

```
CBO=929, LCOM=1.0, RFC=929, WMC=929
Health grade: F
```

Similar pattern - message processor is also a monolith.

### Ingestion Coverage Gaps (files that failed parsing)

Key unparsed files (significant architecture impact):
- `codex-rs/network-proxy/src/policy.rs` - Network egress policy
- `codex-rs/network-proxy/src/runtime.rs` - Network runtime
- `codex-rs/rmcp-client/src/oauth.rs` - OAuth implementation
- `codex-rs/rmcp-client/src/auth_status.rs` - Auth state
- Entire `codex-rs/core/tests/suite/` (85 files) - Integration tests

Coverage by crate (notable high-coverage areas):
- `codex-rs/app-server-protocol/schema/` - 99.5% (OpenAPI schema files)
- `codex-rs/execpolicy-legacy/src/` - 92.9%
- `codex-rs/execpolicy/src/` - 88.9%
- `codex-rs/core/src/` - 37.6% (partially due to large unparsed test suite)

---

## 10. Forward Callees of codex.rs (sample - 1,124 total)

The `codex-rs/core/src/codex.rs` file uses these key architectural modules:

```
AbsolutePathBuf           -> codex-utils-absolute-path crate
AgentControl              -> agent control flow
AgentStatus               -> agent state machine
AnalyticsEventsClient     -> telemetry
AppInfo                   -> app-server-protocol
ActiveTurn                -> turn state management
AgentMessageContentDeltaEvent -> streaming events
AgentReasoningSectionBreakEvent -> reasoning section tracking
```

This confirms `codex.rs` is the central nervous system - it imports from virtually every subsystem.

---

## 11. Phase 1 Summary: Key Findings

1. **Architecture Pattern**: Hub-and-spoke with `codex.rs` as the central God Object. Clean crate boundaries elsewhere.

2. **Multi-Platform by Design**: Linux (bubblewrap C + seatbelt Rust), Windows (AppContainer), macOS (seatbelt) - sandbox is a first-class architectural concern.

3. **MCP First-Class**: Codex is both an MCP server AND an MCP client. The `rmcp-client` handles OAuth-authenticated remote MCP connections.

4. **Multi-Model Ready**: Local model support (LM Studio, Ollama) alongside OpenAI backend. Model manager in k-core inner ring.

5. **Dual UI Architecture**: TUI (Ratatui Rust) + legacy TypeScript CLI. TUI is the current primary UI (50 source files, 40.98% coverage).

6. **Safety Policy Layered**: Two exec policy engines (legacy TOML + new rule-based), plus network proxy, plus per-action approval flow.

7. **Zero Circular Dependencies**: Clean DAG across 40+ crates. Impressive for codebase this scale.

8. **Technical Debt Concentration**: `codex.rs` (1,124 deps) and `codex_message_processor.rs` (929 deps) are clear refactoring targets.

9. **Streaming as Core**: SSE streaming (`codex-api/src/sse_responses.rs`) in the k-core innermost ring - real-time streaming is architectural bedrock.

10. **Collab/Hierarchical Agents**: `CollabAgentStatus`, `CollabTool`, `CollabToolCallStatus` enums suggest multi-agent collaboration is implemented (hierarchical agents test suite exists).

---

## Next Steps (Phase 2)

- Trace control flow: `cli/src/main.rs` -> `app-server` -> `core/src/codex.rs`
- Blast radius of `protocol/src/protocol.rs` (k-core 33)
- Forward callees of `exec/src/lib.rs` (shell execution entry)
- Reverse callers of `execpolicy/src/decision.rs`
- Map the full approval flow: user input -> exec policy -> sandbox -> result
- Analyze `network-proxy` architecture (28.6% coverage - gaps in policy.rs)
- Trace MCP tool dispatch: `mcp-server/src/` -> `exec_approval` -> `sandbox`

---

*Generated by Parseltongue v1.7.2 graph analysis | Port 7780 | 2026-02-19*
