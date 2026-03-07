# CR-delegate-research-summary.md
# CR08: Delegate Competitive Research — Full Summary
# Date: 2026-02-21
# Repo: https://github.com/nikhilgarg28/delegate.git
# Parseltongue Analysis: Port 7781 | 1,959 entities | 14,352 edges | 22K LoC

---

## Executive Summary

**Delegate** is an "engineering manager for AI agents" — a Python-based orchestration layer that runs persistent teams of Claude Code agents. Unlike Codex and Droid (which are single agents), Delegate manages multi-agent teams with role-based permissions, async task execution, and automated code review workflows.

**Key Differentiator**: Delegate doesn't just write code — it manages the *entire software development lifecycle* including task decomposition, agent assignment, peer review, merge automation, and team coordination.

---

## 1. The Numbers

```
+==============================================================================+
|                    DELEGATE vs CODEX vs DROID vs PARSELTONGUE                |
+==============================================================================+
|                                                                              |
|  METRIC                     DELEGATE      CODEX         DROID      PARSELTONGUE |
|  ========================  ==========   ==========   ==========  ============ |
|  Language                   Python        Rust          Rust+Bun    Rust        |
|  Total LoC                   22,236       ~500K         ~100K       ~50K        |
|  Source files                150          1,401         N/A        ~400        |
|  Parseltongue entities       1,959        15,901       0 (opaque)  N/A        |
|  Dependency edges           14,352       136,130       0 (opaque)  22K+       |
|  Languages detected          3            5             N/A        12         |
|  Circular dependencies       TBD          0             N/A        0          |
|  Open source                 YES          YES           NO         YES        |
|  Binary size                 pip pkg     ~30 MB        95.9 MB    ~15 MB     |
|  Architecture pattern         Orchestrator Single agent   Orch/Worker Graph engine |
|                                                                              |
+==============================================================================+
```

---

## 2. Architecture Overview

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                              DELEGATE ARCHITECTURE                            │
└──────────────────────────────────────────────────────────────────────────────┘

    USER (Browser)                DAEMON                      AGENTS
    ==========                  ======                     ======
         │                          │                           │
         │  SSE/HTTP                │  Async                    │  Persistent
         │                          │  Background               │  Claude Code
         ▼                          │  Tasks                     │  Subprocesses
    ┌─────────┐                    │                              │
    │  Web UI  │                    │                              │
    │ (React)  │                    │  ┌──────────────────────┐     │
    └────┬────┘                    │  │ Message Router       │     │
         │                         │  │ Turn Dispatcher      │     │
         │                         │  │ Merge Worker         │     │
         │                         │  └───────┬──────────────┘     │
         │                         │          │                     │
         │                         │          ▼                     │
         │                         │  ┌─────────────────────────┐│
         │                         │  │ Telephone Exchange      ││
         │                         │  │ (Persistent Claude      ││
         │                         │  │  Code Subprocesses)     ││
         │                         │  └───────┬─────────────────┘│
         │                         │          │                     │
         │                         │    ┌─────┴─────┐              │
         │                         │    │             │              │
         ▼                         ▼    ▼             ▼              ▼
    ┌───────────────────────────────────────────────────────────────┐
    │                     SQLITE DATABASE                           │
    │  ~/.delegate/teams/{team}/db.sqlite                            │
    │  ┌────────────────────────────────────────────────────────────┐ │
    │  │ tasks    messages   sessions   agents   projects        │ │
    │  └────────────────────────────────────────────────────────────┘ │
    └───────────────────────────────────────────────────────────────┘
         │                         │                     │
         │                         │                     │
         ▼                         ▼                     ▼
    ┌─────────┐              ┌─────────┐            ┌───────────┐
    │  Git    │              │Protected│            │ Worktrees │
    │ Repos  │              │ Files   │            │ (isolated)│
    └─────────┘              └─────────┘            └───────────┘
```

---

## 3. Package Structure (41 Python Files)

```
delegate/                          # Main package
├── agent.py                      # 852 lines - Agent helpers, prompts, worktrees
├── task.py                       # 1,292 lines - SQLite task management
├── runtime.py                    # 1,170 lines - Agent turn execution loop
├── web.py                        # 3,464 lines - FastAPI web interface
├── eval.py                       # 1,581 lines - Testing framework
├── env.py                        # 1,064 lines - Environment setup
├── merge.py                      # 1,134 lines - Git merge automation
├── cli.py                        # 1,078 lines - Command-line interface
├── telephone.py                  # 840 lines - Claude Code subprocess wrapper
├── chat.py                       # 503 lines - Event timeline & chat
├── mcp_tools.py                  # 509 lines - Agent data access via MCP
├── workflow.py                   # 662 lines - Task lifecycle DSL
├── mailbox.py                    # 508 lines - Inter-agent messaging
├── bootstrap.py                  # 599 lines - Team setup
├── db.py                         # 739 lines - Database operations
├── activity.py                   # 372 lines - Real-time SSE broadcasts
├── repo.py                       # 470 lines - Repository management
├── config.py                     # 362 lines - Configuration management
├── prompt.py                     # 525 lines - Prompt builders
├── review.py                     # 259 lines - Code review automation
├── uploads.py                    # 306 lines - File uploads
├── notify.py                     # 301 lines - Notifications
├── sim_boss.py                   # 340 lines - Simulation testing
├── paths.py                      # 344 lines - Path utilities
├── runtime.py                    # 1,170 lines - Agent turn execution loop
├── daemon.py                     # 281 lines - Background process management
├── router.py                     # 83 lines - Message routing logic
├── db_ids.py                     # 255 lines - ID management
├── doctor.py                     # 165 lines - Health checks
├── fmt.py                        # 88 lines - Formatting utilities
├── logging_setup.py              # 104 lines - Logging configuration
├── run.py                        # 100 lines - CLI entry point
├── names.py                      # 190 lines - Name utilities
└── network.py                    # 260 lines - Network security

workflows/                         # Custom workflow definitions
├── default.py                     # Standard workflow
├── core.py                        # Core workflow stages
└── git.py                         # Git-related workflows

charter/                           # Team charter templates
├── values.md
├── communication.md
├── task-management.md
├── code-review.md
└── roles/                         # Role-specific charters
    ├── manager.md
    ├── engineer.md
    ├── designer.md
    └── qa.md

frontend/                          # React-based web UI
├── src/
│   ├── api.js
│   ├── audio.js
│   ├── chat.js
│   ├── commands.js
│   ├── state.js
│   └── ...
```

---

## 4. Key Architectural Subsystems

### 4.1 Agent Coordination System

**Components**:
- **TelephoneExchange**: Registry of persistent Claude Code subprocesses
- **AgentLogger**: Structured per-agent logging with context
- **Telephone**: Claude Code subprocess wrapper with MCP integration

**Security Model**:
- OS sandboxing via Claude Code's native sandbox
- Write-path isolation (managers: full team dir, engineers: own dir + shared)
- Tool deny-list for dangerous git operations
- Network allowlist (domain-level egress filtering)

### 4.2 Task Lifecycle Management

**Task States**:
```
todo → in_progress → in_review → in_approval → merging → done
  ↓         ↓           ↓            ↓           ↓
cancelled  cancelled   cancelled    cancelled   cancelled
                                                        ↓
                                            rejected → in_progress
```

**Database Schema**:
- `tasks`: 25+ fields including status, assignee, DRI, dependencies
- `sessions`: token/cost tracking
- `messages`: inter-agent communication
- `projects`: team management

### 4.3 Workflow Engine

**Declarative DSL**:
```python
class Todo(Stage):
    label = "To Do"

class InProgress(Stage):
    label = "In Progress"
    def enter(self, ctx):
        ctx.require(ctx.task.base_sha, "Base commit required")

@workflow(name="default", version=1)
def default():
    return [Todo, InProgress, InReview, Done]
```

**Stage Hooks**:
- `enter(ctx)`: Precondition checks and setup
- `exit(ctx)`: Cleanup
- `assign(ctx)`: Assignment logic
- `action(ctx)`: Automated transitions (auto stages)

### 4.4 MCP Tool Integration

**13 MCP Tools** (running in daemon process, outside sandbox):

| Category | Tools |
|----------|-------|
| Task Management | task_create, task_list, task_show, task_assign, task_status, task_comment, task_cancel, task_attach, task_detach |
| Communication | mailbox_send, mailbox_inbox |
| Repository | repo_list |
| Git | rebase_to_main |

### 4.5 Git Worktree Management

**Isolation Model**:
```
~/.delegate/teams/{team_uuid}/
├── worktrees/
│   ├── {repo_name}/T0001/          # Agent worktrees (persistent)
│   └── _merge/{uuid}/T0001/         # Merge worktrees (temporary)
├── agents/
│   ├── delegate/                   # Manager agent
│   ├── alice/                       # Engineer agents
│   └── bob/
├── repos/                          # Symlinks to real repos
├── shared/                         # Team shared files
└── db.sqlite                       # Team-specific database
```

---

## 5. Security Model (6-Layer Defense)

| Layer | Mechanism | What It Protects |
|-------|-----------|------------------|
| **1. OS Sandbox** | Claude Code native sandbox | Process isolation, filesystem restrictions |
| **2. Tool Deny List** | Dangerous git operations blocked | Branch topology protection |
| **3. Bash Pattern Denials** | Command substring blocking | SQL injection, destructive commands |
| **4. MCP Tool Boundary** | In-process tools run outside sandbox | Protected data access |
| **5. Network Allowlist** | Domain-level egress filtering | Data exfiltration prevention |
| **6. Daemon Management** | Git operations only by daemon | Repository integrity |

**Write-Path Isolation**:
- Managers: Full team directory access
- Engineers: Own agent dir + shared folder + task worktrees

---

## 6. Comparison: Delegate vs Codex vs Droid

```
+==============================================================================+
|                       DELEGATE vs CODEX vs DROID                               |
+==============================================================================+
|                                                                              |
|  DIMENSION              DELEGATE           CODEX              DROID         |
|  =====================  ===============   ===============   ============ |
|                                                                              |
|  Architecture            Orchestrator +     Single agent        Orchestrator  |
|                          Teams of agents    (God Object)       + Workers     |
|                                                                              |
|  Language                Python             Rust                Rust+Bun      |
|                                                                              |
|  LoC                      ~22K               ~500K               ~100K         |
|                                                                              |
|  Agent Coordination      Multi-agent (N)     Single              Multi-agent  |
|                          with roles          (collab)            (orch/workr) |
|                                                                              |
|  Task Decomposition      Built-in           None (ad-hoc)       Structured    |
|                                                                              |
|  Code Review             Automated          None                Structured    |
|                          (peer review)                            (handoffs)   |
|                                                                              |
|  Merge Automation       Yes (full)          None                None         |
|                                                                              |
|  Git Worktrees           Isolated per task  N/A                 N/A          |
|                                                                              |
|  Persistence            SQLite + Files     Chat history        JSON on disk |
|                                                                              |
|  Async                   Native             No                  No           |
|                                                                              |
|  Web UI                  Yes (React PWA)    Yes (TUI)          No           |
|                                                                              |
|  MCP                     Server (13 tools)  Server+Client       Client only  |
|                                                                              |
|  Sandbox                6 layers           5 kernel layers    3 layers     |
|                                                                              |
|  Network Allowlist       Yes                No                  No           |
|                                                                              |
|  Open Source             Yes                Yes                 No           |
|                                                                              |
+==============================================================================+
```

---

## 7. What Delegate Teaches Us

### 7.1 Orchestrator Patterns Worth Studying

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  DELEGATE'S ORCHESTRATOR PATTERN                                           │
└──────────────────────────────────────────────────────────────────────────────┘

    USER
     │ "Add /health endpoint"
     │
     ▼
┌─────────────────┐
│ DELEGATE AGENT   │  (Manager - Opus model)
│ (manager role)   │
│                 │  1. Parse request
│                 │  2. Break down into tasks
│                 │  3. Create task records
│                 │  4. Assign to engineers
└────────┬────────┘
         │
    ┌────┼────┬────┐
    ▼    ▼    ▼    ▼
┌────────┐┌────────┐┌────────┐
│ALICE   ││BOB     ││CHARLIE │  (Engineers - Sonnet model)
│(eng)   ││(eng)   ││(eng)   │
│        ││        ││        │
│T0001   ││T0002   ││T0003   │
│/health ││/tests  ││/docs   │
└───┬────┘└───┬────┘└───┬────┘
    │         │         │
    ▼         ▼         ▼
┌─────────────────────────┐
│ PEER REVIEW             │  (Another engineer)
│                         │  • Reviews diff
│ Approves/Requests      │  • Runs tests
│ changes                 │  • Comments
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ USER APPROVAL            │
│ (or auto-merge)         │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ MERGE WORKER            │  (Daemon)
│                         │  • Rebase onto main
│ Atomic fast-forward     │  • Pre-merge tests
│ merge                   │  • Update branch
└─────────────────────────┘
```

**Key Insight**: Delegate's orchestrator is NOT a God Object. It delegates work to specialized sub-agents (engineers) who work in isolation. The coordinator only manages state — it doesn't do the work itself.

### 7.2 Workflow DSL — Customizable Lifecycle

Delegate's workflow system is **extensible**:
```python
@workflow(name="with-deploy", version=1)
def my_workflow():
    return [Todo, InProgress, InReview, Deploy, Done]

class Deploy(Stage):
    label = "Deploying"
    def enter(self, ctx):
        ctx.run_script("./deploy.sh")
```

**For V200**: This pattern could apply to *analysis workflows* — custom pipelines for different types of code investigation (security audit, refactor planning, tech debt assessment).

### 7.3 MCP Tools for Safe Data Access

Delegate runs 13 MCP tools **inside the daemon process** (outside the OS sandbox):
- Agents access database through tools, not direct shell
- Tool closures capture agent identity (no impersonation possible)
- Authorization checks enforced at tool layer

**For V200**: If V200 ever adds execution capabilities, use this pattern — tools for data access, not direct database connections.

### 7.4 Git Worktree Isolation

Each task gets an isolated git worktree:
- No locking required (agent worktrees never touched during merge)
- Environment isolation (separate venv, node_modules, etc.)
- Automatic cleanup on task completion

**For V200**: When analyzing codebases, worktree isolation prevents contamination between analysis contexts.

---

## 8. What V200 Can Integrate Easily

### 8.1 Task Context Ranking for Delegate Agents

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  V200 MCP TOOL FOR DELEGATE                                                  │
└──────────────────────────────────────────────────────────────────────────────┘

  get_context_for_task(
    task_id: "T0001",
    focus_entity: "add_health_endpoint",
    token_budget: 4096,
    include_tests: true
  )

  Returns:
  • Ranked entities for the task's feature branch
  • Recent commits affecting the same files
  • Test files related to the feature
  • Dependent services that might break
```

**Integration Point**: Delegate's agents run via Claude Code. V200 MCP server could be added to provide intelligent context ranking for each agent turn.

### 8.2 Code Quality Gates in Workflows

```python
class QualityGate(Stage):
    def enter(self, ctx):
        # Call V200 MCP for quality check
        result = ctx.mcp("parseltongue", "sqale_check", {
            "branch": ctx.task.branch,
            "threshold": "C"
        })
        if result["grade"] in ["D", "F"]:
            ctx.fail(f"Code quality too low: {result['grade']}")
```

**Integration Point**: V200's SQALE scoring becomes a workflow gate that prevents low-quality code from progressing.

### 8.3 Cross-Agent Knowledge Sharing

Delegate has a `shared/` folder per team. V200 could provide:
- Team-wide code intelligence summaries
- Architectural hotspots tracking
- Dependency impact analysis
- Dead code detection for cleanup

---

## 9. One-Liner Takeaways

1. **Delegate is orchestrator-as-a-service** — Python-based, uses Claude Code as subprocesses
2. **22K LoC vs Codex's 500K** — Much smaller, more focused scope
3. **13 MCP tools** — All running in daemon process, outside OS sandbox
4. **6-layer security model** — Defense-in-depth with network allowlist
5. **Workflow DSL is extensible** — Custom task lifecycles in Python
6. **Git worktree isolation** — Each task gets isolated environment
7. **Automated code review** — Peer review between engineer agents
8. **Merge automation** — Atomic fast-forward merges with pre-merge tests
9. **SQLite-based persistence** — All state in local files
10. **React PWA web UI** — Installable as desktop app

---

## 10. Files Created

- docs/CR-delegate-research-summary.md (this file)
- docs/CR-delegate-agent-coordination.md
- docs/CR-delegate-task-lifecycle.md
- docs/CR-delegate-mcp-analysis.md
- docs/CR-delegate-security-model.md
- docs/CR-delegate-architecture.md
- docs/CR08-delegate-research-progress-tracker.md

---

*Generated: 2026-02-21*
*Parseltongue v1.7.2 | Port 7781 | Database: rocksdb:parseltongue20260221204410/analysis.db*
