# CR-codex-eli5-ascii-summary.md
# Phase 5: ELI5 ASCII Summary — OpenAI Codex + Factory Droid + V200
# Date: 2026-02-19

---

## The 30-Second Version

```
┌──────────────────────────────────────────────────────────────────┐
│                                                                  │
│   CODEX = A skilled programmer with a bulletproof office         │
│           (kernel sandbox, MCP both ways, zero cycles)           │
│                                                                  │
│   DROID = A project manager + team with filing cabinets          │
│           (mission plans, handoff receipts, ByteRank scores)     │
│                                                                  │
│   PARSELTONGUE = The X-ray machine in the hospital               │
│           (graph analysis, deterministic metrics, token ranking)  │
│                                                                  │
│   All three are needed. None replaces the others.                │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## Key Line 1: Codex is a 40-Crate Rust Monster

```
                    codex-rs/
                    ├── core/         ← The brain (CBO=1124, Grade F)
                    ├── tui/          ← The face (50 files, Ratatui)
                    ├── cli/          ← The mouth
                    ├── mcp-server/   ← Talks TO other agents
                    ├── rmcp-client/  ← Talks FROM other agents
                    ├── linux-sandbox/← Seatbelt + Landlock + bwrap
                    ├── exec/         ← Runs your commands
                    ├── execpolicy/   ← Decides what's allowed
                    ├── protocol/     ← Shared language
                    ├── lmstudio/     ← Local models (air-gapped)
                    ├── ollama/       ← Local models (air-gapped)
                    └── ... (30+ more)

    15,901 entities | 136,130 edges | 5 languages | 0 circular deps
```

**ELI5**: Codex is like a big company with 44 departments (crates). Each department has its own job. No department depends on itself in a loop (zero circular deps). But there's one department (codex.rs) that talks to EVERY other department — it's become a bottleneck.

---

## Key Line 2: Triple-Platform Sandbox = Codex's Superpower

```
    macOS                  Linux                    Windows
    ┌──────────┐          ┌──────────────────┐     ┌──────────┐
    │ Seatbelt  │          │ Landlock LSM      │     │ ACL/DACL  │
    │ (deny-all │          │ + Seccomp BPF     │     │ (file     │
    │  + allow  │          │ + PID namespace    │     │  path     │
    │  list)    │          │ + Mount namespace  │     │  restrict)│
    │           │          │ + User namespace   │     │           │
    │           │          │ + Network ns       │     │           │
    │           │          │ + bubblewrap (C)   │     │           │
    └──────────┘          └──────────────────┘     └──────────┘

    CBO = 1 for all sandbox entry points (Grade A!)
    Best-designed part of the entire codebase.
```

**ELI5**: When Codex runs your code, it locks it in a jail cell first. On Linux, that jail has 6 layers of locks. On macOS, 2. On Windows, 1. The jail system is the best-written code in the whole project — perfectly clean, minimal dependencies.

---

## Key Line 3: Droid = Orchestrator + Workers + Receipts

```
    User: "Refactor auth"
         │
         ▼
    ORCHESTRATOR (never writes code)
    ├── Creates: mission.md
    ├── Creates: features.json [F1, F2, F3...]
    ├── Creates: services.yaml
    └── For each feature:
         │
         ▼
    WORKER (writes code)
    ├── Reads: mission.md + SKILL.md
    ├── Executes: code changes
    ├── Verifies: runs tests
    └── Returns: handoff.json
         │
         ▼
    ORCHESTRATOR reviews handoff
    ├── pass → next feature
    └── fail → retry with notes
```

**ELI5**: Droid is like hiring a project manager (orchestrator) who breaks your request into tasks, then assigns programmers (workers) to each task. Each programmer reports back with a receipt (handoff) showing what they did, what tests passed, and what problems they found. The project manager can't code — only plan and review.

---

## Key Line 4: Same Tools, Different Orchestration

```
    IDENTICAL TOOLS (9 of 14):
    ┌──────────────────────────────────────────────────┐
    │  Read file    Edit file    Create file            │
    │  Run shell    Find files   Search text            │
    │  Web search   Fetch URL    Todo list              │
    └──────────────────────────────────────────────────┘

    CODEX ONLY:          DROID ONLY:
    • Sub-agents          • ProposeMission
    • Notebooks           • RunNextWorker
    • Plan mode           • Skills system
    • MCP server          • ByteRank scoring
    • Structured Q&A      • Browser automation
                          • Slack integration
```

**ELI5**: Codex and Droid are like two carpenters using the same 9 tools from the same toolbox. The difference isn't in the tools — it's in how they organize the work. Codex lets YOU direct each step. Droid makes a plan first, then executes it feature-by-feature with progress reports.

---

## Key Line 5: Neither Has What Parseltongue Has

```
    WHAT THEY USE FOR CODE UNDERSTANDING:

    Codex:  grep (text search) ─────────────────┐
    Droid:  grep (text search) + ByteRank (LLM) ─┤
                                                   │
                          TEXT-LEVEL ONLY           │
                          No graph. No AST.         │
                          No dependencies.          │
                          No architecture.          │
                                                   │
    ┌──────────────────────────────────────────────┘
    │
    │  PARSELTONGUE V200:
    │
    │  ┌─────────────────────────────────────────┐
    │  │  12-language AST parsing (tree-sitter)   │
    │  │  136K+ dependency edges                  │
    │  │  7 graph algorithms:                     │
    │  │    SCC    PageRank   k-core   Leiden     │
    │  │    SQALE  CK-metrics Entropy              │
    │  │  18 Datalog rules (embeddable CodeQL)    │
    │  │  Token-budgeted context ranking           │
    │  │  Cross-language edge detection            │
    │  │  Deterministic (same input = same output) │
    │  └─────────────────────────────────────────┘
    │
    │  THIS IS THE GAP PARSELTONGUE FILLS.
```

**ELI5**: Codex and Droid look at code like reading a book — word by word, page by page. Parseltongue looks at code like an X-ray — it sees the skeleton (structure), the blood vessels (dependencies), and the organs (modules). No amount of reading replaces an X-ray.

---

## Key Line 6: The Scoreboard

```
    AXIS                  CODEX    DROID    PARSELTONGUE
    ====================  =======  =======  ============
    Sandboxing            ★★★★★    ★★       N/A (tool)
    MCP integration       ★★★★★    ★★       ★★★★★ (server)
    Multi-agent           ★★★      ★★★★★    N/A (tool)
    Multi-model           ★★★★     ★★★      N/A (tool)
    Tool system           ★★★★     ★★★★     N/A (tool)
    Graph analyzability   ★★★★★    ★        ★★★★★ (IS the graph)
    State persistence     ★★       ★★★★★    ★★★★ (DB)
    Approval/safety       ★★★★★    ★★★      N/A (read-only)
    Task decomposition    ★★       ★★★★★    N/A (tool)
    Code quality metrics  —        ★★★      ★★★★★ (deterministic)
    Skill/recipe system   ★★       ★★★★     N/A (tool)
    Binary size           ★★★★★    ★★       ★★★★ (~15MB)
    ====================  =======  =======  ============
    CODEX: 7 wins         DROID: 4 wins     PT: Fills the gap

    THEY ARE AGENTS.  PARSELTONGUE IS THE INTELLIGENCE LAYER.
    COMPLEMENTARY, NOT COMPETITIVE.
```

---

## What V200 Should Do (5 Bullets)

1. **Ship get_context as MCP tool.** Neither competitor has ranked, token-budgeted context. This is V200's moat.

2. **Replace ByteRank with graph algorithms.** 8 of 11 categories covered. Faster, cheaper, deterministic. Say this out loud.

3. **Add provenance/confidence to MCP responses.** Copy Droid's handoff pattern: every result includes where the data came from and how confident the analysis is.

4. **Monitor own CBO to avoid the God Object trap.** Codex's codex.rs grew to CBO=1124 because nobody watched. V200 watches from day one.

5. **Stay a TOOL, not an AGENT.** Codex and Droid handle orchestration. V200 provides the intelligence they both lack. Don't try to be all three.

---

*Generated 2026-02-19. Phase 5: ELI5 ASCII Summary.*
*Minto Pyramid: Governing thought → 6 key lines → supporting ASCII → 5 action bullets.*
