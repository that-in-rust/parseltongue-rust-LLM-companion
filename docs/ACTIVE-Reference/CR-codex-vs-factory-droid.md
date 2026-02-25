# CR-codex-vs-factory-droid.md
# Phase 3: OpenAI Codex vs Factory Droid — Side-by-Side Architecture Comparison
# Date: 2026-02-19
# Sources: Parseltongue v1.7.2 graph analysis (port 7780) + binary decompilation (CR05)

---

## 0. Governing Insight (Minto Pyramid)

> **Codex and Droid are two sides of the same coin — one went deep on infrastructure
> (kernel-level sandboxing, compiled Rust, MCP server/client), the other went deep
> on workflow (mission decomposition, worker handoffs, skill system). Neither has
> what Parseltongue provides: deterministic graph intelligence.**

Key lines:
1. Codex = 44 Rust crates, kernel-level sandbox, zero circular deps, MCP both ways
2. Droid = Rust+Bun hybrid, orchestrator/worker hierarchy, disk artifacts, ByteRank
3. Same tool DNA (9/14 identical tools), divergent architecture philosophy
4. Codex has 15,901 graph entities; Droid is a 96MB opaque binary (no graph possible)
5. Both LACK graph-computed code intelligence — Parseltongue's lane is wide open

---

## 1. The Numbers

```
+==============================================================================+
|                    CODEX vs FACTORY DROID — RAW METRICS                       |
+==============================================================================+
|                                                                              |
|  METRIC                     CODEX (graph-analyzed)    DROID (decompiled)     |
|  ========================   =====================     ====================   |
|  Binary size                ~30 MB (Rust native)      95.9 MB (Rust+Bun)    |
|  Language (implementation)  Rust (90%) + TS (4%)      JS (60%) + Rust (5%)   |
|                             + C (2%) + Py (2%)        + Bun runtime (35%)    |
|  Crate/module count         44 Rust crates            1 monolith binary      |
|  Code entities (PT graph)   15,901                    N/A (closed binary)    |
|  Dependency edges           136,130                   N/A                    |
|  Circular dependencies      0                         Unknown                |
|  Languages supported        5 (Rust,TS,JS,Py,C)      N/A (runtime, not      |
|                                                        analyzable)           |
|  Ingestion coverage         60.96% (854/1,401)        0% (opaque binary)    |
|  God files (CBO > 500)     3 files                   Unknown                |
|  Max CBO                    1,124 (codex.rs)          Unknown                |
|  Max k-core                 33                        N/A                    |
|  Leiden clusters            460                       N/A                    |
|  SCC count                  34,933 (all size-1)       N/A                    |
|  Open source                YES (GitHub)              NO (closed binary)     |
|  Compiler                   rustc stable              rustc 1.93.1 nightly   |
|  MCP role                   Server AND Client         Client only            |
|  Sandbox depth              Kernel-level (3 OS)       Cloud sandbox (E2B)    |
|  Multi-agent                Yes (Collab* enums)       Yes (Orch/Worker)      |
|  Multi-model                Yes (4 providers)         Yes (3+ providers)     |
|                                                                              |
+==============================================================================+
```

---

## 2. Architecture Philosophy

```
+==============================================================================+
|                                                                              |
|            CODEX                              DROID                           |
|            =====                              =====                          |
|                                                                              |
|   "Build the INFRASTRUCTURE              "Build the WORKFLOW                 |
|    right, and features follow"            right, and users follow"           |
|                                                                              |
|   44 crates, clean DAG,                  1 binary, opaque internals,         |
|   zero circular deps,                    orchestrator/worker hierarchy,      |
|   kernel-level sandboxing,               disk artifact system,               |
|   compile-time safety,                   runtime flexibility,                |
|   MCP both directions                    skill system + ByteRank             |
|                                                                              |
|   ┌────────────────────┐                 ┌────────────────────┐              |
|   │ Systems engineering │                 │ Product engineering │              |
|   │ mindset             │                 │ mindset             │              |
|   └────────────────────┘                 └────────────────────┘              |
|                                                                              |
|   Optimizes for:                         Optimizes for:                      |
|   • Correctness (type safety)            • Cognitive load reduction          |
|   • Security (sandbox depth)             • Task decomposition speed          |
|   • Performance (compiled)               • Progress visibility               |
|   • Extensibility (MCP)                  • Skill reuse                       |
|                                                                              |
+==============================================================================+
```

---

## 3. Axis 1: Sandboxing Depth

This is Codex's single biggest architectural advantage over Droid.

```
+==============================================================================+
|                       SANDBOXING COMPARISON                                  |
+==============================================================================+
|                                                                              |
|  CODEX (kernel-level, per-platform)        DROID (cloud + env-based)         |
|  ===================================      ===========================        |
|                                                                              |
|  ┌─────────────────────────────────┐      ┌──────────────────────────┐      |
|  │         cli_main()              │      │      droid exec          │      |
|  │              │                  │      │           │              │      |
|  │    ┌─────────┼──────────┐      │      │     ┌─────┼─────┐       │      |
|  │    │         │          │      │      │     │           │       │      |
|  │    ▼         ▼          ▼      │      │     ▼           ▼       │      |
|  │ [macOS]   [Linux]   [Windows]  │      │  [Local]     [E2B]     │      |
|  │ Seatbelt  Landlock  ACL+DACL   │      │  (no        (cloud     │      |
|  │ +sandbox  +bwrap    +UAC       │      │  sandbox)   sandbox)   │      |
|  │  -exec    +seccomp  elevation  │      │                        │      |
|  │           +PID ns              │      │  Autonomy levels:      │      |
|  │           +mount ns            │      │  off|low|med|high      │      |
|  │           +user ns             │      │  (permission gates,    │      |
|  │           +net ns              │      │   not kernel isolation)│      |
|  └─────────────────────────────────┘      └──────────────────────────┘      |
|                                                                              |
|  SECURITY LAYERS:                          SECURITY LAYERS:                  |
|                                                                              |
|  L1: OS sandbox (kernel-enforced)          L1: Autonomy levels (SW)          |
|      Seatbelt profiles (macOS)                 off → all approval            |
|      Landlock LSM (Linux)                      low → read + edit only        |
|      Windows DACL/ACE (Windows)                med → reversible cmds         |
|                                                high → all commands            |
|  L2: Namespace isolation (Linux)                                             |
|      PID namespace                         L2: E2B cloud sandbox             |
|      Mount namespace (bind mounts)             (optional, $$ per use)        |
|      Network namespace (loopback)                                            |
|      User namespace (rootless)                                               |
|      Seccomp BPF filters                  L3: Risk classification            |
|                                                per shell command              |
|  L3: Bubblewrap (vendored C)                   low|med|high risk             |
|      build_bwrap_argv()                                                      |
|      386 C entities in graph                                                 |
|                                                                              |
|  L4: ExecPolicy engine                                                       |
|      248 entities (new + legacy)                                             |
|      Decision enum (Allow|Deny)                                              |
|      TOML allowlist (legacy)                                                 |
|      Rule-based (new)                                                        |
|                                                                              |
|  L5: Network proxy                                                           |
|      207 entities                                                            |
|      Egress filtering                                                        |
|                                                                              |
|  PARSELTONGUE GRAPH DATA:                  PARSELTONGUE GRAPH DATA:          |
|  • sandbox entities: 415                   • N/A (opaque binary)             |
|  • sandbox entry CBO: 1 (Grade A)                                           |
|  • landlock entities: 16                                                     |
|  • seatbelt entities: 23                                                     |
|  • execpolicy entities: 248                                                  |
|  • network entities: 207                                                     |
|                                                                              |
+==============================================================================+

VERDICT: Codex has 5 security layers with kernel enforcement.
         Droid has 3 layers, all software-enforced.
         Codex's sandbox entry points are the ONLY Grade-A (CBO=1) entities
         in the entire codebase — textbook Single Responsibility Principle.
```

---

## 4. Axis 2: MCP Integration

```
+==============================================================================+
|                       MCP INTEGRATION COMPARISON                             |
+==============================================================================+
|                                                                              |
|  CODEX: Both Server AND Client            DROID: Client Only                 |
|  ==============================           ==================                 |
|                                                                              |
|  ┌─────────────────────────────────┐      ┌──────────────────────────┐      |
|  │        AS MCP SERVER             │      │      AS MCP CLIENT       │      |
|  │  (other agents call Codex)       │      │  (Droid calls servers)   │      |
|  │                                  │      │                          │      |
|  │  mcp-server/ crate              │      │  Braintrust MCP          │      |
|  │  ├── message_processor.rs        │      │  Fireflies MCP           │      |
|  │  ├── codex_tool_runner.rs        │      │  Custom MCP servers      │      |
|  │  ├── exec_approval.rs            │      │                          │      |
|  │  └── patch_approval.rs           │      │  No server mode.         │      |
|  │                                  │      │  Droid cannot BE a       │      |
|  │  Approval protocol:              │      │  tool for other agents.  │      |
|  │  sampling/createMessage for      │      │                          │      |
|  │  human-in-the-loop approval      │      └──────────────────────────┘      |
|  │                                  │                                        |
|  │  550 MCP entities in graph       │                                        |
|  │  223 RMCP entities               │                                        |
|  └─────────────────────────────────┘                                        |
|                                                                              |
|  ┌─────────────────────────────────┐                                        |
|  │        AS MCP CLIENT             │                                        |
|  │  (Codex calls other servers)     │                                        |
|  │                                  │                                        |
|  │  rmcp-client/ crate              │                                        |
|  │  ├── rmcp_client.rs              │                                        |
|  │  ├── perform_oauth_login.rs      │                                        |
|  │  └── OAuth2 device flow          │                                        |
|  │                                  │                                        |
|  │  core/src/mcp/                   │                                        |
|  │  ├── auth.rs (McpOAuthLogin)     │                                        |
|  │  └── mcp_connection_manager.rs   │                                        |
|  │                                  │                                        |
|  │  Transport modes:                │                                        |
|  │  • stdio (pipes)                 │                                        |
|  │  • streamable HTTP (SSE)         │                                        |
|  │  • OAuth2 device flow            │                                        |
|  └─────────────────────────────────┘                                        |
|                                                                              |
|  ESCALATION POLICY:                                                          |
|  exec-server/posix/mcp_escalation_policy.rs                                 |
|  ExecPolicyOutcome: Allow | Deny | Escalate                                 |
|  ┌─────────────┐    ┌─────────────┐    ┌──────────────┐                     |
|  │ MCP client   │───▷│ eval policy  │───▷│ Allow: run   │                     |
|  │ requests     │    │ on command   │    │ Deny: error  │                     |
|  │ tool call    │    │              │    │ Escalate:    │                     |
|  │              │    │              │    │  ask client  │                     |
|  └─────────────┘    └─────────────┘    └──────────────┘                     |
|                                                                              |
+==============================================================================+

VERDICT: Codex can be CONSUMED as a tool by other agents (MCP server mode).
         Droid can only CONSUME tools. This is a fundamental architectural
         asymmetry — Codex participates in agent ecosystems; Droid is terminal.

         For Parseltongue: Both Codex and Droid would benefit from calling
         Parseltongue's MCP server for graph intelligence. But only Codex
         could also serve as an intermediary (Codex calls Parseltongue,
         then exposes results to other agents via its own MCP server).
```

---

## 5. Axis 3: Multi-Agent Architecture

```
+==============================================================================+
|                    MULTI-AGENT COMPARISON                                     |
+==============================================================================+
|                                                                              |
|  CODEX: Hierarchical Collab Agents        DROID: Orchestrator/Worker          |
|  =================================       ==========================          |
|                                                                              |
|  ┌─────────────────────────────────┐      ┌──────────────────────────┐      |
|  │        MAIN AGENT               │      │      ORCHESTRATOR         │      |
|  │  (codex.rs — God Object)        │      │  (architect only)         │      |
|  │  CBO=1124                       │      │  NEVER writes code        │      |
|  │                                  │      │                          │      |
|  │  Can spawn sub-agents:           │      │  Tools:                  │      |
|  │  multi_agents::handle()          │      │  • ProposeMission        │      |
|  │  ├── spawn_agent() [explorer]    │      │  • RunNextWorker         │      |
|  │  ├── resume_agent() [restore]    │      │  • SelectFeature         │      |
|  │  ├── send_input() [push data]    │      │  • Read tools only       │      |
|  │  ├── wait() [with timeout]       │      │  • NO code tools         │      |
|  │  └── close_agent() [cleanup]     │      │                          │      |
|  │                                  │      └─────────┬────────────────┘      |
|  │  Collab enums (143 entities):    │                │ RunNextWorker          |
|  │  • CollabAgentStatus             │      ┌─────────┼──────────┐            |
|  │  • CollabTool                    │      ▼         ▼          ▼            |
|  │  • CollabToolCallStatus          │   Worker 1  Worker 2  Worker N         |
|  │  • CollabAgentSpawnBegin         │   (codes)   (codes)   (codes)          |
|  │  • CollabAgentSpawnEnd           │                                        |
|  │  • CollabAgentInteractionBegin   │   Each worker:                         |
|  │  • CollabAgentInteractionEnd     │   • Gets fresh context                 |
|  │  • CollabCloseBegin              │   • Reads mission.md                   |
|  │  • CollabCloseEnd                │   • Reads SKILL.md                     |
|  │  • CollabResumeBegin             │   • Executes feature                   |
|  │  • CollabResumeEnd               │   • Returns handoff JSON               |
|  │                                  │   • Handoff persisted to disk          |
|  │  spawn_agent entities: 6         │                                        |
|  │                                  │   Handoff structure:                    |
|  │  Depth limiting:                 │   ├── salientSummary (20-400 chars)    |
|  │  Prevents infinite recursion     │   ├── whatWasImplemented               |
|  │  of sub-agent spawning           │   ├── verification.commandsRun         |
|  │                                  │   ├── tests.added                      |
|  │  Context: SHARED                 │   ├── discoveredIssues                 |
|  │  (sub-agent sees parent's        │   └── skillFeedback                    |
|  │   conversation)                  │                                        |
|  └─────────────────────────────────┘   Context: ISOLATED                     |
|                                        (worker gets clean slate              |
|                                         + mission files only)                |
|                                                                              |
+==============================================================================+

KEY DIFFERENCES:
+=======================================================================+
| Dimension          | Codex                  | Droid                   |
+====================+========================+=========================+
| Agent separation   | Same agent type        | Architect vs Coder      |
|                    | (all can code)         | (strict separation)     |
+--------------------+------------------------+-------------------------+
| Context model      | Shared (sub-agents     | Isolated (workers get   |
|                    | see parent context)    | fresh context + files)  |
+--------------------+------------------------+-------------------------+
| State persistence  | In-memory only         | Disk artifacts          |
|                    |                        | (mission.md, handoffs/) |
+--------------------+------------------------+-------------------------+
| Error recovery     | No structured retry    | Orch retries failed     |
|                    | (user re-prompts)      | workers automatically   |
+--------------------+------------------------+-------------------------+
| Progress format    | Free-form text         | Structured handoff JSON |
|                    |                        | with verification proof |
+--------------------+------------------------+-------------------------+
| Decomposition      | Ad-hoc (agent/user     | Systematic              |
|                    | decides what to        | (features.json with     |
|                    | delegate)              | deps + verification)    |
+--------------------+------------------------+-------------------------+
| Handoff entities   | 0 in graph             | Full Zod schema         |
| in codebase        | (no structured format) | (structured contract)   |
+=======================================================================+

VERDICT: Droid's orchestrator/worker pattern with structured handoffs is
         more mature for complex multi-feature projects. Codex's collab
         agents are more flexible but less structured. The key gap in
         Codex: no equivalent of features.json or disk-persisted handoffs.
```

---

## 6. Axis 4: Multi-Model Support

```
+==============================================================================+
|                    MULTI-MODEL COMPARISON                                     |
+==============================================================================+
|                                                                              |
|  CODEX (4 providers, compiled)            DROID (3+ providers, runtime)      |
|  ==============================           ============================       |
|                                                                              |
|  codex-rs/core/ (primary OpenAI)          Anthropic (Claude) — primary       |
|  codex-rs/lmstudio/ (local models)        OpenAI (GPT) — full support        |
|  codex-rs/ollama/ (local models)          Custom BYOK — any OpenAI-compat    |
|  codex-rs/chatgpt/ (ChatGPT compat)                                         |
|  codex-rs/cloud-tasks/ (cloud exec)       SpecMode: separate model for       |
|                                           planning vs execution              |
|  Model manager in k-core ring (33):                                          |
|  core/src/models_manager/manager.rs       Reasoning effort:                  |
|  → Architectural kernel entity            low | medium | high | off          |
|                                           (per-model configurable)           |
|  models_manager entities in graph:                                           |
|  deep integration with protocol           Model selection at runtime:        |
|  layer and SSE streaming                  droid exec --model custom:X        |
|                                                                              |
|  LOCAL MODEL ADVANTAGE:                   CLOUD-ONLY:                        |
|  ┌────────────────────────────┐           ┌──────────────────────┐           |
|  │ LM Studio: local inference │           │ All models require    │           |
|  │ Ollama: local inference    │           │ API access (no local  │           |
|  │ ChatGPT: compat mode      │           │ inference support)     │           |
|  │ → Air-gapped operation    │           │                        │           |
|  │   possible                 │           │ BYOK for custom        │           |
|  │ → No API costs for local  │           │ endpoints only         │           |
|  └────────────────────────────┘           └──────────────────────┘           |
|                                                                              |
+==============================================================================+

VERDICT: Codex wins on local model support (LM Studio, Ollama = air-gapped).
         Droid wins on model/planning separation (SpecMode uses different model).
         Both support BYOK/custom endpoints.
```

---

## 7. Axis 5: Tool System Design

```
+==============================================================================+
|                    TOOL SYSTEM COMPARISON                                     |
+==============================================================================+
|                                                                              |
|  CODEX (compiled, type-safe dispatch)     DROID (JS runtime, dynamic)        |
|  ====================================    ============================        |
|                                                                              |
|  core/src/tools/registry.rs               JS tool registry (runtime)         |
|  ├── ToolKind enum (compiled)             ├── apply_patch (unified diffs)    |
|  ├── dispatch() (static routing)          ├── create_file                    |
|  └── dispatch_after_tool_use_hook()       ├── edit_file                      |
|                                           ├── view_file / view_folder        |
|  core/src/tools/orchestrator.rs           ├── glob_tool                      |
|  ├── ToolOrchestrator (struct)            ├── grep_tool (embedded ripgrep)   |
|  ├── run() (approval + sandbox + exec)    ├── shell (PTY-based)             |
|  └── run_attempt() (single exec)          ├── web_search / fetch_url        |
|                                           ├── todo_write                     |
|  core/src/tools/events.rs                 ├── skill (factory skills)         |
|  ├── ToolEmitter                          ├── slack_post_message             |
|  ├── ToolEventStage                       ├── store_agent_readiness_report   |
|  └── ToolEventFailure                     └── (dynamic MCP tools)            |
|                                                                              |
|  core/src/tools/handlers/                                                    |
|  ├── shell.rs          → Shell execution                                     |
|  ├── apply_patch.rs    → Patch application                                   |
|  ├── unified_exec.rs   → Unified process mgmt                               |
|  └── multi_agents.rs   → Sub-agent spawning                                 |
|                                                                              |
|  Tool entity count: 555                   Tool entity count: ~14 (from       |
|  Protocol entities: 1,106                 decompiled strings only)            |
|                                                                              |
+==============================================================================+

IDENTICAL TOOLS (9/14):
+====================================+====================================+
| Claude Code / Codex                | Factory Droid                      |
+====================================+====================================+
| Read / view_file                   | view_file                          |
| Write / create_file                | create_file                        |
| Edit / edit_file                   | edit_file                          |
| Bash / shell                       | shell                              |
| Glob / glob_tool                   | glob_tool                          |
| Grep / grep_tool                   | grep_tool                          |
| WebSearch / web_search             | web_search                         |
| WebFetch / fetch_url               | fetch_url                          |
| TodoWrite / todo_write             | todo_write                         |
+====================================+====================================+

CODEX-ONLY TOOLS:
+====================================+========================================+
| Tool                               | What It Does                           |
+====================================+========================================+
| Task (sub-agents)                  | Spawn specialized sub-agents           |
| NotebookEdit                       | Edit Jupyter notebooks                 |
| EnterPlanMode                      | Switch to planning mode                |
| AskUserQuestion                    | Structured multi-choice questions      |
| mcp__* (MCP server tools)          | Expose tools to other agents           |
| Dynamic tools (runtime)            | Register tools at runtime via protocol |
+====================================+========================================+

DROID-ONLY TOOLS:
+====================================+========================================+
| Tool                               | What It Does                           |
+====================================+========================================+
| ProposeMission                     | Orchestrator proposes mission plan     |
| RunNextWorker                      | Dispatch worker for feature            |
| SelectFeature                      | Choose feature for worker              |
| EndFeatureRun                      | Worker submits handoff                 |
| Skill                              | Execute .factory/skills/               |
| GenerateDroid                      | Create custom droid config             |
| store_agent_readiness_report       | ByteRank report persistence            |
| slack_post_message                 | Post to Slack channels                 |
| apply_patch (unified diff)         | Apply unified diff patches             |
| Browser tools (CDP)                | Chrome DevTools Protocol               |
+====================================+========================================+

ARCHITECTURAL DIFFERENCE:
┌──────────────────────────────────────────────────────────────────────┐
│ Codex: Tool dispatch is COMPILED (Rust enum → match → handler)      │
│        Every tool call goes through:                                 │
│        registry::dispatch() → orchestrator::run() → handler          │
│        with approval checking, sandbox mode selection, and           │
│        network approval at each step.                                │
│                                                                      │
│ Droid: Tool dispatch is RUNTIME (JS object → function call)          │
│        Tools checked against autonomy level (off/low/med/high)       │
│        and per-command risk assessment (low/med/high).                │
│        No kernel-level sandbox — just permission gates.              │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 8. Axis 6: Graph Structure & Code Health

```
+==============================================================================+
|                    GRAPH STRUCTURE COMPARISON                                 |
+==============================================================================+
|                                                                              |
|  CODEX (fully analyzable)                 DROID (opaque binary)              |
|  ========================                 ====================               |
|                                                                              |
|  15,901 entities                          N/A                                |
|  136,130 edges                            (96MB Mach-O, minified JS)         |
|  5 languages detected                     (strings extraction only)          |
|                                                                              |
|  HEALTH PROFILE:                          HEALTH PROFILE:                    |
|  ┌──────────────────────────────┐         ┌──────────────────────────┐      |
|  │ God files:                   │         │ Unknown — binary is       │      |
|  │ codex.rs        CBO=1124  F  │         │ opaque. Decompiled JS     │      |
|  │ chatwidget.rs   CBO=1075  F  │         │ is minified, no AST      │      |
|  │ msg_processor   CBO=929   F  │         │ analysis possible.        │      |
|  │                              │         │                           │      |
|  │ Clean modules:               │         │ From string analysis:     │      |
|  │ sandbox entries  CBO=1    A  │         │ ~14 tools, ~11 ByteRank   │      |
|  │                              │         │ categories, ~100 criteria │      |
|  │ SCC: 0 cycles (perfect)      │         │ No cycle data.            │      |
|  │ K-core: max 33 (deep core)   │         │ No k-core data.           │      |
|  │ Leiden: 460 clusters          │         │ No clustering data.       │      |
|  │ PageRank: dominated by       │         │ No centrality data.       │      |
|  │ stdlib (expected for Rust)    │         │                           │      |
|  └──────────────────────────────┘         └──────────────────────────┘      |
|                                                                              |
|  TECHNICAL DEBT:                                                             |
|  ┌──────────────────────────────┐         ┌──────────────────────────┐      |
|  │ codex.rs: 14h SQALE debt     │         │ Unknown                   │      |
|  │ expect() count: 1,400        │         │ (but embedded Bun +       │      |
|  │ (panic-on-failure patterns)   │         │  bundled JS suggests      │      |
|  │                              │         │  high runtime overhead)   │      |
|  │ STRENGTH: zero circular deps │         │                           │      |
|  │ across 44 crates. Clean DAG. │         │ STRING EVIDENCE:          │      |
|  │ This is RARE at this scale.  │         │ Zod schemas embedded      │      |
|  │                              │         │ → runtime validation      │      |
|  │ WEAKNESS: 3 God files with   │         │ (vs Codex compile-time)   │      |
|  │ combined CBO of 3,128.       │         │                           │      |
|  └──────────────────────────────┘         └──────────────────────────┘      |
|                                                                              |
+==============================================================================+

VERDICT: Codex's open-source Rust codebase is fully graph-analyzable.
         Droid's closed binary is a black box. This is a fundamental
         TRANSPARENCY asymmetry. Any organization evaluating code agents
         can graph-analyze Codex but not Droid.

         Parseltongue implication: Codex is a SHOWCASE for Parseltongue's
         analysis capabilities. Droid is a showcase for WHY you need tools
         like Parseltongue (to analyze what's inside opaque binaries).
```

---

## 9. Axis 7: State Persistence & Session Model

```
+==============================================================================+
|                    STATE PERSISTENCE COMPARISON                               |
+==============================================================================+
|                                                                              |
|  CODEX (in-memory + shell snapshots)      DROID (disk artifacts)             |
|  ====================================    ========================            |
|                                                                              |
|  Session state:                           Session state:                     |
|  ┌──────────────────────────────┐         ┌──────────────────────────┐      |
|  │ Chat history (memory)        │         │ Chat history (memory)     │      |
|  │ Shell snapshots (k-core 33)  │         │ + disk artifacts:         │      |
|  │ Context compaction           │         │                           │      |
|  │ Session fork/resume          │         │ mission/                  │      |
|  │                              │         │ ├── mission.md            │      |
|  │ state/ crate handles:       │         │ ├── features.json         │      |
|  │ conversation persistence     │         │ ├── AGENTS.md             │      |
|  │                              │         │ └── handoffs/             │      |
|  │ Streaming: SSE (k-core 33)   │         │     ├── F1.json           │      |
|  │ codex-api/sse_responses.rs   │         │     ├── F2.json           │      |
|  │                              │         │     └── F3.json           │      |
|  │ Model manager (k-core 33):   │         │                           │      |
|  │ models_manager/manager.rs    │         │ .factory/                 │      |
|  │                              │         │ ├── services.yaml         │      |
|  │ Config:                      │         │ ├── skills/               │      |
|  │ config/ crate (CBO=398)      │         │ │   ├── backend-worker/   │      |
|  │                              │         │ │   │   └── SKILL.md      │      |
|  │ Hooks:                       │         │ │   └── frontend-worker/  │      |
|  │ hooks/ crate (pre/post turn) │         │ │       └── SKILL.md      │      |
|  └──────────────────────────────┘         │ ├── library/              │      |
|                                           │ │   ├── perf-state.md     │      |
|  DIRECTORY FOOTPRINT:                     │ │   └── migration-status  │      |
|  .claude/                                 │ ├── commands/             │      |
|  ├── settings.json                        │ ├── droids/               │      |
|  └── plans/                               │ └── init.sh              │      |
|  CLAUDE.md                                │                           │      |
|                                           │ AGENTS.md                 │      |
|  MINIMAL footprint.                       └──────────────────────────┘      |
|  State is ephemeral.                                                         |
|                                           HEAVY footprint.                   |
|                                           State survives crashes,            |
|                                           context limits, restarts.          |
|                                                                              |
+==============================================================================+

TRADEOFF:
┌──────────────────────────────────────────────────────────────────────┐
│ Codex: Fast to start, minimal ceremony, state vanishes on crash.    │
│        Good for: quick edits, debugging, exploration, single tasks. │
│                                                                      │
│ Droid: Slow to start (mission proposal → approval → artifact        │
│        creation), heavy ceremony, state survives everything.         │
│        Good for: multi-feature projects, team handoffs, auditing.   │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 10. Axis 8: Approval & Safety Architecture

```
+==============================================================================+
|                    APPROVAL SYSTEM COMPARISON                                 |
+==============================================================================+
|                                                                              |
|  CODEX (multi-layer, protocol-level)      DROID (tiered autonomy)            |
|  ====================================    ========================            |
|                                                                              |
|  approval entities: 178                   Autonomy levels:                   |
|                                           off|low|med|high +                 |
|  LAYER 1: CLI Approval Mode               skip-permissions-unsafe            |
|  ApprovalModeCliArg enum                                                     |
|  (utils/cli/src/)                         LAYER 1: Autonomy Level            |
|                                           ┌──────────────────────┐           |
|  LAYER 2: Protocol Approval               │ off  → all approval  │           |
|  ElicitationAction enum                   │ low  → read+edit     │           |
|  NetworkApprovalProtocol enum             │ med  → reversible    │           |
|  ExecPolicyAmendment                      │ high → all commands  │           |
|  (protocol/src/approvals.rs)              └──────────────────────┘           |
|                                                                              |
|  LAYER 3: ExecPolicy (new)                LAYER 2: Per-Command Risk          |
|  Decision enum (Allow|Deny)               ┌──────────────────────┐           |
|  Rule-based evaluation                    │ Each shell command:   │           |
|  (execpolicy/src/)                        │ riskLevel: {          │           |
|                                           │   value: low|med|high│           |
|  LAYER 4: ExecPolicy (legacy)             │   reason: string     │           |
|  TOML-based allowlist                     │ }                    │           |
|  (execpolicy-legacy/src/)                 └──────────────────────┘           |
|                                                                              |
|  LAYER 5: MCP Approval Gateway            LAYER 3: services.yaml            |
|  handle_exec_approval_request()           ┌──────────────────────┐           |
|  handle_patch_approval_request()          │ Known commands are    │           |
|  sampling/createMessage protocol          │ lower risk.           │           |
|  (mcp-server/src/)                        │ Port kills on         │           |
|                                           │ undeclared ports =    │           |
|  LAYER 6: Network Proxy                   │ high risk.            │           |
|  Network egress filtering                 └──────────────────────┘           |
|  (network-proxy/ - 207 entities)                                             |
|                                                                              |
|  LAYER 7: TUI Approval Prompt                                                |
|  bottom_pane_view.rs                                                         |
|  (tui/src/bottom_pane/)                                                      |
|                                                                              |
|                                                                              |
|  APPROVAL FLOW (Codex):                   APPROVAL FLOW (Droid):             |
|                                                                              |
|  User request                             User request                       |
|       │                                        │                             |
|       ▼                                        ▼                             |
|  ExecPolicy check                         Autonomy level check               |
|       │                                        │                             |
|       ├── Allow → sandbox                      ├── off → prompt user         |
|       ├── Deny → reject                        ├── low → allow if read/edit  |
|       └── Escalate ──┐                         ├── med → allow if reversible |
|                      │                         └── high → allow all          |
|                      ▼                              │                        |
|              MCP approval request                   ▼                        |
|              (sampling/createMessage)          Risk assessment               |
|                      │                         (per command)                  |
|                      ▼                              │                        |
|              Client approves/denies                 ▼                        |
|                      │                         Execute or prompt             |
|                      ▼                                                       |
|              Sandbox execution                                               |
|              (kernel-level)                                                   |
|                                                                              |
+==============================================================================+

VERDICT: Codex has 7 approval layers with kernel enforcement at the end.
         Droid has 3 layers, all software-enforced.
         Codex's approval system is protocol-level (works across MCP).
         Droid's is session-level (works within the droid process only).
```

---

## 11. Head-to-Head Summary Matrix

```
+==============================================================================+
|                   CODEX vs DROID — SUMMARY MATRIX                            |
+==============================================================================+
|                                                                              |
|  AXIS                  CODEX           DROID           WINNER                |
|  ====================  ==============  ==============  ===========           |
|  Sandboxing depth      5 layers,       3 layers,       CODEX                 |
|                        kernel-level    software-only    (not close)          |
|  -----------------------------------------------------------------          |
|  MCP integration       Server+Client   Client only     CODEX                 |
|                        (bidirectional)  (consume only)  (structural)         |
|  -----------------------------------------------------------------          |
|  Multi-agent           Collab agents   Orch/Worker     DROID                 |
|                        (flexible)      (structured)    (maturity)            |
|  -----------------------------------------------------------------          |
|  Multi-model           4 providers     3+ providers    CODEX                 |
|                        + local models  cloud-only      (local wins)          |
|  -----------------------------------------------------------------          |
|  Tool system           555 entities    ~14 tools       TIE                   |
|                        compiled        runtime         (same DNA)            |
|  -----------------------------------------------------------------          |
|  Graph analyzability   15,901 entities N/A (opaque)    CODEX                 |
|                        open source     closed binary   (transparency)        |
|  -----------------------------------------------------------------          |
|  State persistence     Ephemeral       Disk artifacts  DROID                 |
|                        (memory only)   (survives all)  (durability)          |
|  -----------------------------------------------------------------          |
|  Approval/safety       7 layers        3 layers        CODEX                 |
|                        protocol-level  session-level   (depth)               |
|  -----------------------------------------------------------------          |
|  Task decomposition    Ad-hoc          Systematic      DROID                 |
|                        (user-driven)   (automatic)     (UX)                  |
|  -----------------------------------------------------------------          |
|  Code quality metrics  None built-in   ByteRank        DROID                 |
|                                        (LLM-based)     (exists>none)         |
|  -----------------------------------------------------------------          |
|  Skill/recipe system   Hooks only      .factory/skills DROID                 |
|                                        YAML+MD         (richer)              |
|  -----------------------------------------------------------------          |
|  Binary size           ~30 MB          95.9 MB         CODEX                 |
|                        (pure Rust)     (Rust+Bun+JSC)  (lean)               |
|  =================================================================          |
|                                                                              |
|  SCORE:  CODEX 7  |  DROID 4  |  TIE 1                                     |
|                                                                              |
+==============================================================================+
```

---

## 12. What Each Can Learn From The Other

```
+==============================================================================+
|               CODEX SHOULD STEAL FROM DROID                                  |
+==============================================================================+
|                                                                              |
| 1. STRUCTURED HANDOFFS                                                       |
|    Droid's handoff JSON (salientSummary, verification, tests, issues)        |
|    is a contract between agents. Codex's collab agents return free-form      |
|    text. Adding structured returns would make Codex's multi-agent            |
|    actually usable for complex projects.                                     |
|                                                                              |
| 2. DISK-PERSISTED MISSION STATE                                              |
|    features.json + handoffs/ on disk means progress survives crashes.        |
|    Codex has shell snapshots but no equivalent of mission-level state.       |
|                                                                              |
| 3. ARCHITECT/CODER SEPARATION                                                |
|    Droid's orchestrator CANNOT code (tools disabled). This prevents the      |
|    planner from losing the big picture. Codex's sub-agents have full         |
|    tool access — no role separation.                                         |
|                                                                              |
| 4. SKILL SYSTEM                                                              |
|    .factory/skills/ with YAML frontmatter = reusable procedures.             |
|    Codex has hooks (shell commands on events) but nothing equivalent         |
|    to parameterized, per-project agent playbooks.                            |
|                                                                              |
| 5. CODE QUALITY METRICS                                                      |
|    ByteRank exists. Codex has nothing built-in. Even LLM-evaluated          |
|    metrics are better than no metrics.                                        |
|                                                                              |
+==============================================================================+

+==============================================================================+
|               DROID SHOULD STEAL FROM CODEX                                  |
+==============================================================================+
|                                                                              |
| 1. KERNEL-LEVEL SANDBOXING                                                   |
|    Seatbelt + Landlock + Windows ACL = actual OS-enforced isolation.          |
|    Droid's autonomy levels are just permission gates in JS. A malicious      |
|    command that bypasses the JS layer has no kernel barrier.                  |
|                                                                              |
| 2. MCP SERVER MODE                                                           |
|    Codex can BE consumed as a tool by other agents. Droid is terminal —      |
|    it can't participate in agent-to-agent ecosystems.                         |
|                                                                              |
| 3. LOCAL MODEL SUPPORT                                                       |
|    LM Studio + Ollama = air-gapped operation, zero API costs for             |
|    local inference. Droid requires cloud API access for all models.           |
|                                                                              |
| 4. OPEN-SOURCE TRANSPARENCY                                                  |
|    Codex's 44-crate Rust codebase can be graph-analyzed, audited,            |
|    and contributed to. Droid is a 96MB opaque binary.                         |
|                                                                              |
| 5. COMPILED TYPE SAFETY                                                      |
|    Codex's tool dispatch is a Rust enum match — exhaustive, checked at       |
|    compile time. Droid's is JS runtime dispatch with Zod validation.         |
|    Type errors in Codex are caught at build; in Droid, at runtime.           |
|                                                                              |
| 6. ZERO CIRCULAR DEPENDENCIES                                                |
|    Codex's 44-crate DAG has zero cycles. This is hard to achieve and         |
|    easy to lose. Droid's monolith binary structure hides its dep graph.      |
|                                                                              |
+==============================================================================+
```

---

## 13. Where Parseltongue Fits

```
+==============================================================================+
|                                                                              |
|    ┌─────────────────────────────────────────────────────────────────────┐   |
|    │                    THE INTELLIGENCE GAP                              │   |
|    │                                                                     │   |
|    │   CODEX has:  Infrastructure  (sandbox, MCP, type safety)          │   |
|    │   DROID has:  Workflow        (decomposition, handoffs, skills)    │   |
|    │   NEITHER:    Code Intelligence  (graph analysis, semantic search) │   |
|    │                                                                     │   |
|    │                    PARSELTONGUE FILLS THIS GAP                      │   |
|    │                                                                     │   |
|    └─────────────────────────────────────────────────────────────────────┘   |
|                                                                              |
|    CODEX + PARSELTONGUE:                                                     |
|    ┌─────────────────────────────────────────────────────────────────────┐   |
|    │  Codex calls Parseltongue MCP server for:                           │   |
|    │  • blast-radius before editing (which entities will break?)        │   |
|    │  • coupling metrics before refactoring (what's the current CBO?)   │   |
|    │  • smart context (which entities to include in LLM context?)       │   |
|    │  • SCC check before committing (did I introduce a cycle?)          │   |
|    │  • dead code detection (what can I safely remove?)                 │   |
|    │                                                                     │   |
|    │  Codex then serves results to other agents via its MCP server.     │   |
|    │  (Codex becomes a RELAY for Parseltongue intelligence)             │   |
|    └─────────────────────────────────────────────────────────────────────┘   |
|                                                                              |
|    DROID + PARSELTONGUE:                                                     |
|    ┌─────────────────────────────────────────────────────────────────────┐   |
|    │  Droid orchestrator calls Parseltongue MCP for:                     │   |
|    │  • Leiden clusters → natural feature boundaries for decomposition  │   |
|    │  • coupling metrics → risk-aware feature ordering                  │   |
|    │  • blast radius → worker scope estimation                          │   |
|    │                                                                     │   |
|    │  Droid workers call Parseltongue MCP for:                           │   |
|    │  • forward/reverse callees → understand call chain before editing  │   |
|    │  • smart context → get relevant entities within token budget       │   |
|    │  • taint paths → security-critical code awareness                  │   |
|    │                                                                     │   |
|    │  Droid validates handoffs against Parseltongue metrics:             │   |
|    │  • Did CBO decrease after refactoring?                             │   |
|    │  • Did dead code increase after deletion?                          │   |
|    │  • Are there new circular dependencies?                            │   |
|    └─────────────────────────────────────────────────────────────────────┘   |
|                                                                              |
|    PARSELTONGUE vs BYTERANK:                                                 |
|    ┌─────────────────────────────────────────────────────────────────────┐   |
|    │                                                                     │   |
|    │  ByteRank (Droid)              Parseltongue                         │   |
|    │  ===============               ==============                       │   |
|    │  100+ criteria                 22 HTTP endpoints                    │   |
|    │  LLM-evaluated                 Graph-computed                       │   |
|    │  ~5K tokens/check              O(1) graph lookup                    │   |
|    │  Non-deterministic             Deterministic                        │   |
|    │  Seconds per criterion         Milliseconds per query               │   |
|    │  Qualitative ("looks ok")      Quantitative (CBO=7)                │   |
|    │  $$$ per evaluation            Free (local computation)             │   |
|    │                                                                     │   |
|    │  8 of 11 ByteRank categories can be REPLACED by Parseltongue:      │   |
|    │  • code_modularization  → Leiden + CBO/LCOM/RFC/WMC               │   |
|    │  • cyclomatic_complexity → Shannon entropy + branch count          │   |
|    │  • dead_code_detection  → Datalog reachability from entry points   │   |
|    │  • duplicate_detection  → Entity similarity in graph               │   |
|    │  • naming_consistency   → Entity naming pattern analysis           │   |
|    │  • code quality overall → SQALE tech debt scoring                  │   |
|    │  • testing coverage     → Test entity → code entity mapping        │   |
|    │  • dependency health    → SCC + PageRank + k-core                  │   |
|    │                                                                     │   |
|    └─────────────────────────────────────────────────────────────────────┘   |
|                                                                              |
+==============================================================================+
```

---

## 14. V200 Contract Implications

| Finding | V200 Action | Priority |
|---------|------------|----------|
| Codex MCP server mode = relay pattern | V200 MCP responses should be structured for relay (Codex calls PT, serves to others) | P0 |
| Droid's handoff format is a structured contract | V200 tool responses should include `provenance`, `confidence`, `verification` fields | P0 |
| Droid's features.json = decomposition from clusters | V200's Leiden endpoint should return results structured for agent decomposition | P1 |
| ByteRank's 8 categories replaceable by PT | Ship deterministic alternatives to LLM-evaluated code quality checks | P1 |
| Codex's ExecPolicy Decision enum | V200 could provide analysis-based exec policy recommendations | P2 |
| Droid's services.yaml = operational manifest | V200 could read per-project config for analysis customization | P2 |
| Codex's shell snapshots in k-core | V200 could track analysis state across sessions (snapshot/restore) | P3 |
| Droid's skill system = reusable recipes | V200 could expose "analysis presets" as named configurations | P3 |

---

## 15. Final ASCII: The Three-Body Architecture

```
+==============================================================================+
|                                                                              |
|                        THE CODE AGENT ECOSYSTEM                              |
|                                                                              |
|   ┌───────────────────────────────────────────────────────────────────────┐  |
|   │                                                                       │  |
|   │              ┌─────────────┐                                          │  |
|   │              │ PARSELTONGUE│                                          │  |
|   │              │ (graph DB)  │                                          │  |
|   │              │             │                                          │  |
|   │              │ 12 langs    │                                          │  |
|   │              │ 22 endpoints│                                          │  |
|   │              │ 7 algorithms│                                          │  |
|   │              │             │                                          │  |
|   │              │ PROVIDES:   │                                          │  |
|   │              │ • SCC       │                                          │  |
|   │              │ • PageRank  │                                          │  |
|   │              │ • k-core    │                                          │  |
|   │              │ • Leiden    │                                          │  |
|   │              │ • SQALE     │                                          │  |
|   │              │ • CK metrics│                                          │  |
|   │              │ • Entropy   │                                          │  |
|   │              │ • Blast rad │                                          │  |
|   │              │ • Taint     │                                          │  |
|   │              └──────┬──────┘                                          │  |
|   │                     │ MCP                                             │  |
|   │           ┌─────────┼─────────┐                                       │  |
|   │           │                   │                                       │  |
|   │           ▼                   ▼                                       │  |
|   │   ┌──────────────┐   ┌──────────────┐                                │  |
|   │   │    CODEX      │   │    DROID      │                                │  |
|   │   │  (OpenAI)     │   │  (Factory AI) │                                │  |
|   │   │               │   │               │                                │  |
|   │   │  44 crates    │   │  Rust + Bun   │                                │  |
|   │   │  Rust native  │   │  96 MB binary │                                │  |
|   │   │               │   │               │                                │  |
|   │   │  STRONG:      │   │  STRONG:      │                                │  |
|   │   │  • Sandbox    │   │  • Decompose  │                                │  |
|   │   │  • MCP both   │   │  • Handoffs   │                                │  |
|   │   │  • Local model│   │  • Skills     │                                │  |
|   │   │  • Type safe  │   │  • ByteRank   │                                │  |
|   │   │  • Open source│   │  • Browser    │                                │  |
|   │   │               │   │               │                                │  |
|   │   │  WEAK:        │   │  WEAK:        │                                │  |
|   │   │  • Decompose  │   │  • Sandbox    │                                │  |
|   │   │  • Handoffs   │   │  • MCP server │                                │  |
|   │   │  • Skills     │   │  • Local model│                                │  |
|   │   │  • Code intel │   │  • Code intel │                                │  |
|   │   │               │   │  • Open source│                                │  |
|   │   │  CAN RELAY PT │   │               │                                │  |
|   │   │  via MCP srv  │   │  CANNOT RELAY │                                │  |
|   │   └──────────────┘   └──────────────┘                                │  |
|   │                                                                       │  |
|   │   BOTH NEED: deterministic code intelligence (Parseltongue)          │  |
|   │   BOTH HAVE: same core tools (9/14 identical)                        │  |
|   │   BOTH LACK: graph-computed quality metrics                          │  |
|   │                                                                       │  |
|   └───────────────────────────────────────────────────────────────────────┘  |
|                                                                              |
+==============================================================================+
```

---

## 16. One-Liner Takeaways

1. **Codex = deep infrastructure, shallow workflow. Droid = shallow infrastructure, deep workflow.**
2. **9 of 14 tools are identical.** The differentiation is in orchestration, not in tools.
3. **Codex's sandbox is 5 kernel-level layers. Droid's is 3 software layers.** Not comparable.
4. **Codex is bidirectional MCP (server+client). Droid is unidirectional (client only).**
5. **Droid's structured handoffs are what Codex's collab agents should have been.**
6. **ByteRank (LLM-evaluated, non-deterministic) is Parseltongue's easiest target.**
7. **Neither has graph-computed code intelligence.** The lane is wide open.
8. **Codex is fully graph-analyzable (15,901 entities). Droid is opaque (0 entities).**
9. **Codex's zero circular dependencies across 44 crates is genuinely impressive.**
10. **For V200: Parseltongue is complementary to BOTH. The integration patterns differ.**

---

*Generated 2026-02-19. Phase 3: Codex vs Factory Droid side-by-side comparison.*
*Sources: Parseltongue v1.7.2 graph analysis (port 7780, 15,901 entities) + binary decompilation (CR05, 96MB Mach-O).*
*Graph data: CR07/CR-codex-graph-overview.md + CR07/CR-codex-architecture.md.*
*Droid data: CR05/factory-droid/ + docs/CR-droid-factory-20260219/.*
