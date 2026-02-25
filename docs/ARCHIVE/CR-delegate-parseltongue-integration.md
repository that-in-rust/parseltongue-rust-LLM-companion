# CR-delegate-parseltongue-integration.md
# CR08: Delegate → Parseltongue V200 Integration Opportunities
# Date: 2026-02-21
# Focus: What's easy to integrate and what V200 can learn

---

## Executive Summary

Delegate and Parseltongue V200 are **highly complementary**. Delegate orchestrates AI agents; Parseltongue provides code intelligence. The integration is **low-hanging fruit** compared to Codex/Droid because:

1. Delegate is **Python-based** (no Rust rewrites needed)
2. Delegate **already uses MCP** (drop-in V200 server)
3. Delegate has **explicit extension points** (workflow DSL)
4. Delegate's security model **validates V200's design choices**

---

## 1. Drop-In Integration: V200 MCP Server

Delegate already runs 13 MCP tools in-process. Adding V200 as another MCP server is trivial:

```python
# In delegate/runtime.py
mcp_servers = {
    "delegate": build_agent_mcp_server(hc_home, team, agent),
    "parseltongue": connect_to_parseltongue("http://localhost:7777"),  # NEW
}
```

**Agent Usage**:
```python
# Agent can now call V200 tools
response = await ctx.mcp("parseltongue", "get_context", {
    "entity": "python:function:add_health_endpoint",
    "token_budget": 4096
})
```

**Integration Cost**: ~20 lines of code to add V200 to the MCP server registry.

---

## 2. Workflow Stage: Quality Gate

Delegate's workflow DSL allows custom stages. V200 becomes a quality gate:

```python
from delegate.workflow import Stage, workflow

class QualityGate(Stage):
    label = "Quality Check"
    terminal = False

    async def enter(self, ctx):
        # Call V200 for SQALE check
        result = await ctx.mcp("parseltongue", "technical_debt_sqale_scoring", {
            "entity": ctx.task.get("main_file"),
        })

        grade = result.get("grade", "A")
        cbo = result.get("coupling_outbound", 0)

        if grade in ["D", "F"] or cbo > 20:
            ctx.fail(f"Code quality gate failed: {grade}, CBO={cbo}")

        # Store metrics in task metadata
        ctx.set_metadata("sqale_grade", grade)
        ctx.set_metadata("sqale_cbo", cbo)

@workflow(name="with-quality-gate", version=1)
def workflow_with_quality():
    return [Todo, InProgress, QualityGate, InReview, Done]
```

**Integration Cost**: ~30 lines for the stage + workflow definition.

---

## 3. MCP Tool: Task Context Provider

V200 could provide a dedicated MCP tool for Delegate's task context:

```python
@tool(
    "get_task_context",
    "Get ranked code context for a Delegate task",
    {
        "task_id": "string",
        "token_budget": "integer",
        "include_tests": "boolean"
    }
)
async def get_task_context(args: dict) -> dict:
    task = get_task(hc_home, team, args["task_id"])
    branch = task.get("branch")

    # Call V200's get_context for the branch
    result = parseltongue_get_context(
        focus_entity=task.get("main_file"),
        token_budget=args["token_budget"],
        include_tests=args.get("include_tests", True)
    )

    return _json_result(result)
```

**Integration Cost**: V200 provides this tool; Delegate consumes it via MCP.

---

## 4. Comparison: Integration Difficulty

```
+==============================================================================+
|            INTEGRATION WITH PARSELTONGUE V200                               |
+==============================================================================+
|                                                                              |
|  ASPECT                   CODEX             DROID              DELEGATE       |
|  =======================  ================  ================  ============= |
|                                                                              |
|  Language                  Rust (recompile)   Binary (opaque)   Python (pip)   |
|  Integration Difficulty  HIGH               VERY HIGH          LOW           |
|                                                                              |
|  MCP Compatibility          Yes (server+client) Client only       Yes (13 tools) |
|  V200 as MCP Server         Can relay         Can't relay       Can add to     |
|                            to other agents                      existing MCP   |
|                                                                              |
|  Workflow Integration     No workflow DSL   Handoff format    Extensible     |
|  Customization                                        DSL (Python)    |
|                                                                              |
|  Quality Gates             None              ByteRank (LLM)     Build custom    |
|  V200 Integration          Medium            High               LOW           |
|                                                                              |
|  Data Access               Open source       Closed source      Open source   |
|  for V200 Study            Easy              Impossible         Easy          |
|                                                                              |
|  Learning Opportunity      Sandboxing        Orchestration      Orchestrator   |
|  for V200                  (5 layers)        pattern           + workflows   |
|                                                                              |
+==============================================================================+

VERDICT: DELEGATE IS THE EASIEST INTEGRATION TARGET.
```

---

## 5. Specific Integrations (Ranked by Effort)

### P0 — TRIVIAL (< 50 lines)

**1. Add V200 to Delegate's MCP registry**
```python
# File: delegate/runtime.py (around line 622)
mcp_servers = {
    "delegate": create_agent_mcp_server(hc_home, team, agent),
}
# Add after V200 server is known to be running:
if parseltongue_url := os.getenv("PARSELTONGUE_URL"):
    mcp_servers["parseltongue"] = await connect_to_parseltongue(parseltongue_url)
```

**2. Environment variable for V200 URL**
```bash
# Delegate's .env or config
PARSELTONGUE_URL=http://localhost:7777
```

### P1 — EASY (< 200 lines)

**3. Quality Gate workflow stage** (see Section 2 above)

**4. Task context MCP tool** (see Section 3 above)

### P2 — MODERATE (< 500 lines)

**5. Blast radius analysis for task changes**
```python
class ImpactAnalysis(Stage):
    async def enter(self, ctx):
        task = ctx.task
        files = task.get("changed_files", [])

        for file_path in files:
            result = await ctx.mcp("parseltongue", "blast_radius_impact_analysis", {
                "entity": f"python:file:{file_path}",
                "hops": 2
            })

            affected = result.get("affected_entities", [])
            if len(affected) > 50:
                ctx.notify(ctx.manager, f"Large impact: {file_path} affects {len(affected)} entities")
```

**6. Dead code cleanup task template**
```python
@task_template("cleanup_dead_code")
async def cleanup_dead_code(ctx):
    # Get dead code from V200
    dead = await ctx.mcp("parseltongue", "datalog_query", {
        "rule": "dead_code",
        "scope": ctx.task.get("scope")
    })

    for entity in dead["entities"]:
        ctx.log(f"Removing dead code: {entity}")
        ctx.tool("Edit", ...)
```

### P3 — WORTHWHILE BUT OPTIONAL

**7. Team-wide code intelligence dashboard**
- Query V200 for team's most critical files
- Display in Delegate's web UI
- Track technical debt over time

**8. Automated refactor planning**
- Use V200's Leiden communities to suggest feature boundaries
- Use V200's coupling metrics to prioritize refactors
- Generate task dependencies automatically

---

## 6. What V200 Should Learn from Delegate

### 6.1 Orchestrator Pattern (Not God Object)

Delegate's manager agent **does NOT write code**. It only:
- Parses user requests
- Breaks down work into tasks
- Assigns to engineers
- Coordinates peer review

**For V200**: If V200 ever adds orchestration, use this pattern. The orchestrator should call V200 tools, not implement them directly.

### 6.2 Workflow DSL for Customization

Delegate's `@workflow` decorator makes it easy to customize task lifecycles:
```python
@workflow(name="custom", version=1)
def custom():
    return [Todo, InProgress, CustomStage, Done]
```

**For V200**: Consider a similar DSL for analysis workflows (e.g., `@analysis_pipeline`).

### 6.3 MCP Tools for Safe Data Access

Delegate's MCP tools run in the daemon process (outside OS sandbox). This is the right pattern for:
- Database access
- Configuration management
- Cross-agent communication

**For V200**: If V200 adds execution, use MCP tools for privileged operations, not direct agent access.

---

## 7. Integration Roadmap

```
+==============================================================================+
|                    DELEGATE + PARSELTONGUE INTEGRATION ROADMAP                    |
+==============================================================================+
|                                                                              |
|  PHASE 1 (WEEK 1)                                                           |
|  ────────────────                                                            |
|  • Add V200 to Delegate's MCP server registry                                 |
|  • Test get_context from Delegate agents                                     |
|  • Validate quality gate workflow stage                                       |
|                                                                              |
|  PHASE 2 (WEEK 2-3)                                                         |
|  ──────────────────                                                          |
|  • Implement task context provider MCP tool                                 |
|  • Add blast radius impact analysis to workflows                              |
|  • Create dead code cleanup task template                                     |
|                                                                              |
|  PHASE 3 (WEEK 4+)                                                          |
|  ────────────────                                                           |
|  • Team-wide code intelligence dashboard                                      |
|  • Automated refactor planning                                                |
|  • Cross-agent knowledge sharing via V200                                      |
|                                                                              |
+==============================================================================+
```

---

## 8. Quick Win: 15-Minute Integration

The fastest way to see Delegate + V200 working together:

```bash
# 1. Start V200 server
cd /path/to/parseltongue
./parseltongue pt08-http-code-query-server \
  --db "rocksdb:analysis.db" \
  --port 7777

# 2. Set environment variable for Delegate
export PARSELTONGUE_URL=http://localhost:7777

# 3. Start Delegate
cd /path/to/delegate
delegate start

# 4. In Delegate's web UI, send agent message:
# "Use V200 to get context for add_health_endpoint function"
```

The agent will call V200's MCP server and return ranked context.

---

## 9. Final Verdict

**Delegate is the MOST V200-compatible competitor studied so far.**

| Reasoning | Evidence |
|----------|----------|
| Same language | Both Python, no rewrites needed |
| MCP-first design | 13 MCP tools already, drop-in V200 |
| Extensible workflows | Python DSL for custom stages |
| Open source | Full code access for integration |
| Complementary scope | Delegate orchestrates, V200 analyzes |
| Security model alignment | Both emphasize defense-in-depth |

**Codex**: Needs Rust rewrites → HARD integration
**Droid**: Closed binary → IMPOSSIBLE integration
**Delegate**: pip install + MCP → TRIVIAL integration

---

*Generated: 2026-02-21*
*Delegate: https://github.com/nikhilgarg28/delegate*
*Parseltongue: Graph-based code analysis toolkit*
