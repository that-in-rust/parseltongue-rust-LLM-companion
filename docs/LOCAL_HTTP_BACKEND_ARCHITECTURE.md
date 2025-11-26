# Parseltongue: Local HTTP Backend Architecture

**Decision**: Replace CLI + file exports with a local HTTP server.

## The Shift

```
BEFORE: pt01 index → pt02 export → agent reads files
AFTER:  parseltongue serve . → agent queries HTTP
```

## API Design

```bash
parseltongue serve ./project --port 3333
```

```
GET /stats                    → {"entities": 156, "edges": 870}
GET /entities?name=export     → matching entities
GET /edges                    → all edges (Level 0)
GET /callers/:entity          → who calls this
GET /callees/:entity          → what this calls
GET /blast/:entity?hops=3     → impact analysis
GET /cycles                   → circular dependencies
GET /hotspots?top=10          → complexity hotspots
GET /search?q=pattern         → fuzzy search
```

## HTTP vs MCP Comparison

| Aspect | HTTP Server | MCP |
|--------|-------------|-----|
| **Client support** | ANY (curl, agents, scripts, browsers) | Claude Desktop, compatible clients only |
| **Protocol** | HTTP/REST (universal) | MCP protocol (proprietary) |
| **Setup** | Zero config | Tool registration required |
| **Debugging** | curl, Postman, browser | Specialized tools |
| **Integration** | Works with GPT, Claude, Cursor, any agent | Claude ecosystem only |
| **Implementation** | ~550 LOC (Rust + axum) | ~1300 LOC (Python + MCP SDK) |

## Why HTTP Wins for Agent Access

### Claude Code / Terminal Agents

Claude Code and similar terminal-based agents can:
- Execute `curl` commands natively
- Parse JSON responses directly
- No special protocol needed

```bash
# Agent just runs:
curl -s http://localhost:3333/callers/export | jq .
```

### MCP Limitations

- Requires MCP-compatible client
- Claude Code doesn't natively speak MCP (it's for Claude Desktop)
- Additional protocol layer adds complexity

### HTTP is Universal

Every programming language, every agent, every tool understands HTTP:
- Python: `requests.get()`
- JavaScript: `fetch()`
- Rust: `reqwest`
- Shell: `curl`
- Browser: Just open the URL

## Agent Workflow Example

```
User: "What functions call export?"

Agent (Claude Code):
1. Bash: curl http://localhost:3333/callers/export
2. Response: {"callers": ["main", "run_export"], "count": 2}
3. Agent formats answer

No file paths. No database paths. No MCP protocol.
```

## Implementation Estimate

```
HTTP server (axum):           ~150 LOC
Endpoint handlers:            ~300 LOC
Startup indexing:             ~100 LOC
─────────────────────────────────────
Total:                        ~550 LOC
```

## Process Management Options

```bash
# Foreground (see output)
parseltongue serve .

# Background daemon
parseltongue serve . --daemon

# Auto-shutdown after idle
parseltongue serve . --timeout 30m

# Specific port
parseltongue serve . --port 8080
```

## Response Format (LLM-Friendly)

```json
{
  "success": true,
  "query": "callers/export",
  "count": 2,
  "data": [
    {"from": "rust:fn:main:src_main_rs:45", "edge_type": "Calls"},
    {"from": "rust:fn:run_export:src_cli_rs:120", "edge_type": "Calls"}
  ],
  "tokens_estimate": 150
}
```

## The Mental Model

Parseltongue = **Local code intelligence backend**

Like:
- PostgreSQL: Start server → query with psql/any client
- Redis: Start server → query with redis-cli/any client
- **Parseltongue**: Start server → query with HTTP/curl/any agent

## Key Benefits

1. **One command** to start (not 3+)
2. **No JSON files** to manage
3. **No path copying** - server owns state
4. **Universal access** - any HTTP client
5. **Real-time queries** - instant answers
6. **Agent-native** - curl is everywhere

## Conclusion

HTTP is wider in use than MCP. Every agent can speak HTTP. Not every agent can speak MCP.

For maximum compatibility with Claude Code, GPT agents, Cursor, and any future LLM tool: **HTTP wins**.
