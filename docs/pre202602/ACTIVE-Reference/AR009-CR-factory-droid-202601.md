# CR-factory-droid-202601: Factory Droid Binary Decompilation Thesis

**Date**: 2026-02-18
**Binary**: `/Users/amuldotexe/.local/bin/droid` (Mach-O arm64, 95.9 MB)
**Version**: 1.0.71 (CLI), Bun 1.3.8 (embedded runtime)
**Repo**: https://github.com/Factory-AI/factory (531 stars, closed-source binary)
**Cloned to**: `CR05/factory-droid/` (gitignored)
**Method**: Binary string extraction, symbol analysis, `otool`, `nm`, `strings | grep`
**Cross-reference**: `ES-V200-attempt-01.md` (executable contract ledger)

---

## What Factory Droid Is

Factory Droid is an **enterprise AI coding agent** that runs in your terminal. It decomposes development tasks into "missions" with parallel worker sub-agents, supports multiple LLM providers (Claude, GPT, custom), and includes a built-in repository quality scorer called "ByteRank."

Key stats from decompilation:
- **96MB binary** = Rust shell (~5%) + embedded Bun 1.3.8 JS runtime (~30%) + bundled JS application (~25%) + embedded ripgrep (~5%) + WebKit JavaScriptCore (~35%)
- **Built by**: `ctate` (cargo paths) on Buildkite CI (`darwin-aarch64`)
- **Linked libs**: `libicucore.A.dylib`, `libresolv.9.dylib`, `libc++.1.dylib`, `libSystem.B.dylib`
- **Rust compiler**: `rustc 1.93.1 (2026-02-11)` — cutting-edge nightly

---

## Architecture: Rust Shell + Embedded Bun + JS Application

```
┌─────────────────────────────────────────────────────────┐
│                   droid (Mach-O arm64)                   │
│                                                         │
│  ┌─────────────────┐  ┌─────────────────────────────┐  │
│  │   RUST LAYER     │  │   BUN 1.3.8 RUNTIME         │  │
│  │                  │  │                              │  │
│  │  • ripgrep       │  │  ┌─────────────────────┐    │  │
│  │    (full embed)  │  │  │ JS APPLICATION       │    │  │
│  │  • portable-pty  │  │  │                      │    │  │
│  │  • crossbeam     │  │  │ • Agent framework    │    │  │
│  │  • serde_json    │  │  │ • Mission/Worker     │    │  │
│  │  • anyhow        │  │  │ • Tool executor      │    │  │
│  │  • clap v2       │  │  │ • TUI (React/Ink)    │    │  │
│  │  • encoding_rs   │  │  │ • LLM API clients    │    │  │
│  │                  │  │  │ • MCP client          │    │  │
│  │  Communication:  │  │  │ • OAuth/WorkOS        │    │  │
│  │  PTY bridge      │──│  │ • Sentry/Pino         │    │  │
│  │  crossbeam ch    │  │  │ • ByteRank scorer     │    │  │
│  │  JSON over pipes │  │  │ • Browser automation  │    │  │
│  │                  │  │  │ • Skill engine        │    │  │
│  └─────────────────┘  │  └─────────────────────┘    │  │
│                        │                              │  │
│                        │  WebKit JSC engine            │  │
│                        │  WASI runtime                 │  │
│                        │  LOL HTML (Cloudflare)        │  │
│                        └─────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

**Key insight**: The Rust layer is a thin native wrapper. ALL business logic — agent framework, tool execution, TUI, API clients — is in bundled JavaScript running on the embedded Bun runtime. Rust provides only: ripgrep (compiled, fast code search), PTY (interactive shell), and process orchestration.

---

## Tool System (Decompiled)

| Tool llmId | Display Name | Toolkit | Notes |
|------------|-------------|---------|-------|
| `apply_patch` | ApplyPatch | Base | Unified diff patches on files |
| `create_file` | Create File | Base | New file creation |
| `edit_file` | Edit File | Base | File editing |
| `view_file` | View File | Base | Read/view files |
| `view_folder` | View Folder | Base | Directory listing |
| `glob_tool` | Glob | Code Search | Pattern-based file finding |
| `grep_tool` | Grep | Code Search | **Powered by embedded ripgrep** (PCRE2 JIT, gitignore-aware) |
| `shell` | Shell | Terminal | Execute CLI commands via PTY |
| `web_search` | Web Search | Web Search | Internet search |
| `fetch_url` | Fetch URL | Web Search | Fetch and analyze web content |
| `slack_post_message` | Slack Post Message | Slack | Post to Slack channels |
| `store_agent_readiness_report` | ByteRank Report | Internal | Persist repository score to Firestore |
| `todo_write` | Todo Write | Base | Task management |
| `skill` | Skill | Base | Execute `.factory/skills/` |

**Architecturally identical to Claude Code's tool set** — same names (Read→view_file, Bash→shell, Glob→glob_tool, Grep→grep_tool), same permission model, same TUI framework (React/Ink). The additions are: `apply_patch` (Claude Code uses Edit), `slack_post_message`, `store_agent_readiness_report`, and `skill`.

---

## Mission / Worker / Feature Model (Flagship Feature)

This is Factory Droid's core differentiator — **hierarchical agent decomposition**:

```
User: "Refactor the auth system"
         │
         ▼
┌─────────────────────────────────┐
│    ORCHESTRATOR SESSION          │
│    (proposes mission)            │
│                                  │
│    Mission:                      │
│      title: "Auth Refactoring"   │
│      features.json: [            │
│        F1: "Extract JWT logic"   │
│        F2: "Add refresh tokens"  │
│        F3: "Write migration"     │
│      ]                           │
└──────────┬───────────────────────┘
           │
    ┌──────┼──────┐
    ▼      ▼      ▼
┌──────┐┌──────┐┌──────┐
│WORKER││WORKER││WORKER│
│  F1  ││  F2  ││  F3  │
│      ││      ││      │
│ JWT  ││Refresh││Migra-│
│logic ││tokens ││tion  │
└──┬───┘└──┬───┘└──┬───┘
   │       │       │
   ▼       ▼       ▼
┌──────┐┌──────┐┌──────┐
│HAND- ││HAND- ││HAND- │
│ OFF  ││ OFF  ││ OFF  │
│pass  ││pass  ││fail  │
└──────┘└──────┘└──────┘
           │
           ▼
  Orchestrator reviews,
  retries F3, completes
```

### Handoff Structure (from Zod schemas in binary)

```typescript
{
  featureId: string,
  resultState: "pass" | "fail",
  salientSummary: string,           // 1-3 sentences
  whatWasImplemented: string,       // min 50 chars
  whatWasLeftUndone: string,
  verification: { commands: [...], results: [...] },
  tests: { written: [...], updated: [...] },
  discoveredIssues: [...],
  skillComplianceReport?: { milestones: [...] },
  commitId?: string
}
```

**V200 relevance**: This is structurally similar to what Parseltongue's MCP tool responses could become — structured analysis results with provenance, verification commands, and confidence scores. The handoff pattern maps to V200's `get_context` ranked results (SEM-C04).

---

## ByteRank: Repository Quality Scorer (100+ Criteria)

ByteRank is Factory's **built-in repository audit system**. A specialized "Agent Readiness Droid" evaluates repos across 11 categories, 100+ criteria:

| Category | Criteria Count | Example Criteria |
|----------|---------------|------------------|
| **Style** | 4 | `lint_config`, `formatter`, `naming_consistency`, `strict_typing` |
| **Testing** | 8 | `unit_tests_exist`, `unit_tests_runnable`, `test_coverage_thresholds`, `flaky_test_detection` |
| **Security** | 7 | `secret_scanning`, `secrets_management`, `pii_handling`, `dast_scanning`, `log_scrubbing` |
| **Build** | 4 | `build`, `build_cmd_doc`, `build_performance_tracking`, `fast_ci_feedback` |
| **Docs** | 5 | `readme`, `api_schema_docs`, `documentation_freshness`, `runbooks_documented` |
| **Dev Environment** | 6 | `dev_env`, `single_command_setup`, `devcontainer`, `env_template` |
| **Deployment** | 6 | `deployment_frequency`, `progressive_rollout`, `release_automation`, `rollback_automation` |
| **Code Quality** | 8 | `code_modularization`, `cyclomatic_complexity`, `dead_code_detection`, `duplicate_code_detection` |
| **VCS/CI** | 10 | `branch_protection`, `codeowners`, `pr_templates`, `pre_commit_hooks`, `backlog_health` |
| **Observability** | 9 | `alerting_configured`, `circuit_breakers`, `distributed_tracing`, `structured_logging` |
| **AI Agent Readiness** | 11 | `agentic_development`, `agents_md`, `feature_flag_infrastructure`, `tech_debt_tracking` |

Each criterion produces: `{ numerator, denominator, rationale }`.

**V200 relevance**: This is DIRECTLY comparable to Parseltongue's analysis endpoints:
- `cyclomatic_complexity` → V200's entropy/complexity measurement
- `dead_code_detection` → V200's Ascent Datalog dead-code rule (SEM-C02)
- `code_modularization` → V200's Leiden community detection + coupling metrics
- `tech_debt_tracking` → V200's SQALE tech debt scoring

**Key difference**: ByteRank is **LLM-evaluated** (the droid reads files and uses judgment). Parseltongue V200 is **graph-computed** (algorithms on the dependency graph, no LLM needed for the measurement). ByteRank's `code_modularization` check reads the codebase and opines; Parseltongue computes actual coupling coefficients (CBO, LCOM, RFC, WMC) from the graph.

---

## Skill System

Skills are reusable agent playbooks stored at `.factory/skills/{worker-type}/SKILL.md`:

```yaml
---
name: implement-typed-ui
description: Implement typed React UI for existing endpoint
model: claude-sonnet-4-5
reasoningEffort: high
tools: [view_file, edit_file, shell, grep_tool]
version: 1.0
---

## Procedure

1. Read the endpoint contract
2. Generate TypeScript types from response schema
3. Build React components with proper type annotations
4. Write integration tests
...
```

Philosophy from embedded strings:
> "Design skills around a single responsibility... Prefer several small skills composed by a Droid over one giant 'do everything' skill."

**Directory structure**:
- `.factory/skills/` — per-project skills
- `.factory/droids/` — project-level droid definitions (system prompts, model, tools)
- `~/.factory/droids/` — personal droid definitions
- `.factory/library/` — shared state files (e.g., `perf-state.md`, `migration-status.md`)
- `.factory/commands/` — custom slash commands
- `AGENTS.md` — repository-level agent instructions (like Claude's `CLAUDE.md`)

---

## Model Support

| Provider | Evidence | Notes |
|----------|----------|-------|
| Anthropic (Claude) | `api.anthropic.com`, `claude-sonnet-4-5` | Primary provider |
| OpenAI | `api.openai.com/v1` | Full support |
| Custom/BYOK | `droid exec --model custom:deepseek-v3` | Any OpenAI-compatible endpoint |

**Reasoning effort**: Configurable per model — `low`, `medium`, `high`, `off`. Matches Claude's extended thinking.

**SpecMode**: Separate planning mode with its own model selection. Orchestrator creates a plan, user approves, workers execute. Workers are told "NEVER use the ExitSpecMode tool" — only the orchestrator can exit spec mode.

---

## External Service Integrations

| Service | URL | Purpose |
|---------|-----|---------|
| Factory API | `api.factory.ai` | Core backend |
| WorkOS | `api.workos.com` | SSO/OAuth authentication |
| E2B | `api.e2b.dev/sandboxes` | Cloud sandbox execution |
| Sentry | (DSN embedded) | Error tracking |
| Segment | `api.segment.io` | Analytics |
| Statsig | (embedded) | Feature flags |
| GitHub API | `api.github.com` | PR/issue integration |
| Braintrust MCP | `api.braintrust.dev/mcp` | AI evaluation |
| Fireflies MCP | `api.fireflies.ai/mcp` | Meeting transcripts |

**GitHub Action**: `Factory-AI/droid-action@v2` — triggers on `@droid` mentions in PR comments, issues, and reviews. Commits signed as `factory-droid[bot]`.

---

## Browser Automation

Full Chrome DevTools Protocol (CDP) integration for web interaction:

```
Agent → Browser Tool → CDP Socket → Chrome/Chromium
                                         │
                                         ▼
                                    Headless or headed
                                    Click, fill, navigate
                                    Screenshot, evaluate
                                    Cookie management
                                    HAR recording
                                    Network interception
```

Configurable via:
- `AGENT_BROWSER_EXECUTABLE_PATH` — custom browser
- `AGENT_BROWSER_HEADED` — headless or visible
- `AGENT_BROWSER_EXTENSIONS` — load extensions

**V200 relevance**: LOW. Parseltongue doesn't need browser automation. But the CDP socket pattern is similar to how V200's MCP stdio transport works — JSON-RPC over a socket.

---

## Competitive Positioning: Droid vs Claude Code vs Parseltongue

```
                CLAUDE CODE         FACTORY DROID         PARSELTONGUE V200
                -----------         -------------         -----------------

Core:           Single agent         Multi-agent           Code graph +
                + tools              orchestration         semantic analysis

Architecture:   Rust + TS            Rust + Bun            Pure Rust
                                     (embedded JS)

Code Search:    ripgrep (ext)        ripgrep (embedded)    tree-sitter + graph
                                                           (entity-level, not text)

Analysis:       None built-in        ByteRank (LLM)        SCC, PageRank, k-core,
                                     100+ criteria          Leiden, SQALE, CK,
                                     (LLM-evaluated)        taint, entropy
                                                           (graph-computed)

Decomposition:  Task tool            Mission/Worker/        N/A (tool, not agent)
                (sub-agents)         Feature + Handoff

Skills:         .claude/ hooks       .factory/skills/       N/A
                                     YAML + MD

Models:         Claude only          Claude, GPT,           Model-agnostic
                                     custom BYOK            (returns data, not runs)

MCP:            Server               Client                 Server
                (exposes tools)      (consumes tools)       (exposes analysis)

Browser:        None                 Full CDP               None

Pricing:        Pay per token        BYOK + Factory         Open source
                via Anthropic        subscription

Output:         Code changes         Code changes +         Entity graph +
                                     structured handoffs    ranked context
```

### What Droid Does That Parseltongue Doesn't

1. **Multi-agent orchestration** — Mission → Features → Workers → Handoffs. Parseltongue is a TOOL, not an AGENT. It provides data that agents consume.
2. **ByteRank scoring** — 100+ repository quality criteria. But: LLM-evaluated (expensive, non-deterministic) vs Parseltongue's graph-computed metrics (deterministic, milliseconds).
3. **Browser automation** — Web interaction for E2E testing, scraping, debugging. Outside Parseltongue's scope.
4. **Skill system** — Reusable parameterized agent playbooks. Parseltongue could provide a skill that agents call, but doesn't define agent behavior.
5. **BYOK model switching** — Use any LLM provider. Parseltongue is model-agnostic (returns structured data, not LLM responses).

### What Parseltongue Does That Droid Doesn't

1. **Graph-level code intelligence** — SCC, PageRank, k-core, Leiden, coupling/cohesion, taint analysis, blast radius. Droid's `grep_tool` is text search; Parseltongue is SEMANTIC search.
2. **12-language entity extraction** — tree-sitter parses code into a typed entity graph. Droid uses ripgrep (text patterns, no AST).
3. **Compiler truth (rust-analyzer)** — V200 enriches the graph with type-resolved semantic facts. Droid relies on LLM interpretation.
4. **Token-optimized context** — `get_context` with ranked entities and token budget. Droid's `view_file` returns raw content.
5. **Deterministic analysis** — Same input → same output → same graph → same metrics. ByteRank varies by LLM response.
6. **Embeddable** — `cargo install parseltongue`. Single binary, zero deps. Droid is 96MB with embedded Bun.

---

## Relevance to V200 Contracts

### High Relevance

| Droid Pattern | V200 Contract | Impact |
|--------------|---------------|--------|
| Mission/Worker decomposition | SEM-C04 (Context Ranking) | Droid's `features.json` is a task decomposition. V200's `get_context` provides the data that an orchestrator would use to plan features. V200 should return results structured for agent decomposition. |
| ByteRank criteria | SEM-C02 (Datalog Rule Pack) + CRT-C05 (Graph Reasoning) | 8 of ByteRank's code quality criteria overlap with V200's graph algorithms. V200 can provide BETTER versions: deterministic, instant, graph-computed vs LLM-evaluated. |
| Structured handoff format | TRN-C01 (MCP-First) + R4-GW-P7-J (XML-Tagged Response) | Droid's handoff JSON structure is a model for V200's MCP tool responses. Include `provenance`, `confidence`, `verification` fields. |
| `.factory/skills/` with YAML frontmatter | TRN-C03 (Companion Command-Generation) | V200's companion/Tauri could generate "skills" that call Parseltongue MCP tools — e.g., a skill that runs blast-radius before every PR. |
| Agent-steering tool descriptions | TRN-C01 | Droid's tools have detailed descriptions with usage instructions. V200's MCP tools need the same. |
| Embedded ripgrep | CRT-C02 (Tree Extractor) | Droid chose to embed ripgrep in Rust for fast code search. V200 embeds tree-sitter for fast code PARSING. Same design pattern: compile the performance-critical tool into the binary. |

### Medium Relevance

| Droid Pattern | V200 Contract | Impact |
|--------------|---------------|--------|
| AGENTS.md convention | TRN-C02 (HTTP Coexistence) | Droid reads `AGENTS.md` for repo instructions. V200 could read a similar file for per-project analysis configuration (e.g., which taint sources/sinks to check). |
| Autonomy levels | CRT-C00 (Interface Gateway) | Droid's `auto low|medium|high` maps to different tool permission sets. V200's gateway could have analysis depth levels (quick scan vs deep analysis). |
| E2B cloud sandboxes | QLT-C02 (Performance Envelope) | Droid can offload to cloud sandboxes. V200 could run analysis in sandboxes for untrusted codebases. Not in current contracts but worth noting. |

### Low Relevance

| Droid Pattern | Why Not Relevant |
|--------------|------------------|
| Browser automation (CDP) | Parseltongue analyzes CODE, not web pages. |
| Slack integration | Social tooling, not code analysis. |
| OAuth/WorkOS SSO | V200 is an open-source CLI tool. No user auth needed. |
| Bun runtime embedding | V200 is pure Rust. No JS runtime. |

---

## What ByteRank Gets Wrong (That Parseltongue Gets Right)

ByteRank's code quality checks are **LLM opinions masquerading as metrics**:

```
ByteRank:
  criterion: "code_modularization"
  method:    LLM reads files, judges modularity
  output:    { numerator: 1, denominator: 1, rationale: "looks modular" }
  cost:      ~5K tokens per check
  determinism: NO — different runs give different rationales

Parseltongue V200:
  metric:    CBO (Coupling Between Objects)
  method:    Count unique entities coupled to target entity in graph
  output:    { entity: "auth::login", cbo: 7, threshold: 10, status: "ok" }
  cost:      O(1) graph lookup
  determinism: YES — same graph = same number
```

```
ByteRank:
  criterion: "cyclomatic_complexity"
  method:    LLM reads function, estimates complexity
  output:    { numerator: 1, denominator: 1, rationale: "moderate complexity" }

Parseltongue V200:
  metric:    Shannon entropy + branching factor from tree-sitter
  method:    Count decision points in AST, compute entropy
  output:    { entity: "auth::login", entropy: 3.2, branches: 12 }
```

```
ByteRank:
  criterion: "dead_code_detection"
  method:    LLM reads imports, guesses what's unused
  output:    { rationale: "some imports appear unused" }

Parseltongue V200:
  metric:    Ascent Datalog unreachable-from-main rule
  method:    Graph reachability from entry points
  output:    { dead_entities: ["old_handler", "unused_struct"], count: 2 }
```

**This is V200's competitive moat against Factory Droid's ByteRank**: graph-computed metrics are faster, cheaper, deterministic, and more precise than LLM-evaluated opinions. Parseltongue can provide ByteRank-style reports as an MCP tool, but backed by actual graph algorithms instead of LLM guessing.

---

## Potential Synergy: Droid + Parseltongue

Droid as the orchestrator, Parseltongue as the analysis backend:

```
Droid Orchestrator
  │
  ├── calls parseltongue MCP: get_context("auth module", tokens=4000)
  │   └── gets: ranked entities, coupling scores, taint paths
  │
  ├── decomposes mission into features based on Parseltongue's
  │   graph clusters (Leiden communities = natural feature boundaries)
  │
  ├── assigns workers, each gets Parseltongue context:
  │   Worker 1: blast-radius for auth::login → knows what will break
  │   Worker 2: SCC membership → knows circular deps to avoid
  │   Worker 3: taint paths → knows security-critical code paths
  │
  └── validates handoffs against Parseltongue metrics:
      Did CBO decrease? Did dead code increase? Did taint coverage change?
```

This is the **"compiler truth + graph speed"** layer that Droid lacks. Droid has the agent orchestration; Parseltongue has the code intelligence.

---

## Open Questions for V200

| # | Question | Triggered By | Contract |
|---|----------|-------------|----------|
| OQ-C12 | Should V200 MCP responses include a `handoff`-style structured format with provenance, confidence, and verification fields? | Droid's handoff pattern | TRN-C01, R4 |
| OQ-C13 | Should V200 provide a "ByteRank-equivalent" MCP tool that returns a repository quality report using graph metrics instead of LLM evaluation? | ByteRank's 100+ criteria | SEM-C02, CRT-C05 |
| OQ-C14 | Should V200 read `AGENTS.md` (or `CLAUDE.md` or `.factory/droids/`) for per-project analysis configuration? | Droid's AGENTS.md convention | TRN-C02 |
| OQ-C15 | Should V200 expose "analysis depth levels" (quick/deep) analogous to Droid's autonomy levels? | Droid's `--auto low|medium|high` | CRT-C00 |

---

## Decompilation Artifacts

All extracted files in `CR05/factory-droid/`:

| File | Lines | Content |
|------|-------|---------|
| `extracted_crate_paths.txt` | 4 | Rust crate paths (ripgrep) |
| `extracted_source_paths.txt` | 162 | Source file references |
| `extracted_deps.txt` | 20 | Cargo dependency names |
| `extracted_urls.txt` | 366 | All URLs from binary |
| `extracted_rust_sources.txt` | 360 | Rust .rs file paths |
| `extracted_api_strings.txt` | 213 | API endpoints and model strings |
| `extracted_factory_apis.txt` | ~10K+ | Factory.ai specific strings (massive) |
| `extracted_agent_framework.txt` | 139 | Agent framework strings |
| `extracted_droid_core.txt` | ~50K+ | Droid core functionality (massive) |
| `rust_source_files.txt` | 13 | Rust source file structure |
| `cargo_deps_full.txt` | 20 | Full cargo dependency list |
| `exported_symbols.txt` | 1 | Exported symbols (stripped binary) |

---

## Summary

Factory Droid is a **96MB Rust+Bun hybrid** that embeds an entire JavaScript application runtime, ripgrep, and a browser automation layer into a single binary. Its architecture is remarkably similar to Claude Code (same tool names, same TUI framework, same directory conventions), with three key additions:

1. **Mission/Worker orchestration** — hierarchical multi-agent task decomposition
2. **ByteRank** — 100+ criterion repository quality scorer (LLM-evaluated)
3. **Browser automation** — full CDP-based web interaction

**For V200, Droid teaches us**:
- Structured handoff formats should be adopted for MCP tool responses
- ByteRank's criteria list is a feature roadmap for graph-computed alternatives (V200 can do 8 of 11 categories deterministically)
- The `.factory/skills/` pattern could inspire Parseltongue MCP "analysis recipes"
- Multi-agent orchestration is the CONSUMER of V200's analysis — V200 provides the data, agents like Droid orchestrate

**Risk to V200**: LOW for core analysis. MEDIUM for adoption — if Droid ships good-enough code quality analysis via LLM, developers may not seek out graph-computed alternatives. V200's counter: determinism, speed (milliseconds vs seconds), zero LLM cost, 12-language coverage.

**The fundamental difference**: Droid is an AGENT (it writes code). Parseltongue is a TOOL (it analyzes code). They're complementary — the best outcome is Droid calling Parseltongue's MCP server for graph intelligence.

---

*Generated 2026-02-18. Binary decompilation thesis for Parseltongue V200. Source: CR05/factory-droid/ (gitignored). Method: strings extraction + symbol analysis on Mach-O arm64 binary.*
