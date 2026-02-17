# Prep-V200: MCP Protocol Integration Research

**Date**: 2026-02-16
**Context**: Deep research into the Model Context Protocol (MCP) for Parseltongue v2.0.0. The v2.0.0 binary (`rust-llm`) will be MCP-FIRST: the primary consumer is an LLM (Claude, GPT, Gemini, etc.), not a human. This document covers the protocol specification, Rust SDK (`rmcp`), client integration patterns, capability mapping, reference implementations, and the MCP-vs-HTTP coexistence strategy.

---

## Table of Contents

1. [MCP Protocol Specification](#1-mcp-protocol-specification)
2. [The rmcp Crate (Official Rust SDK)](#2-the-rmcp-crate-official-rust-sdk)
3. [How MCP Clients Consume Servers](#3-how-mcp-clients-consume-servers)
4. [Mapping Parseltongue Capabilities to MCP Primitives](#4-mapping-parseltongue-capabilities-to-mcp-primitives)
5. [Reference MCP Servers to Study](#5-reference-mcp-servers-to-study)
6. [MCP vs HTTP: Coexistence Strategy](#6-mcp-vs-http-coexistence-strategy)
7. [Implementation Roadmap for rust-llm MCP Server](#7-implementation-roadmap-for-rust-llm-mcp-server)

---

## 1. MCP Protocol Specification

### 1.1 What Is MCP?

The Model Context Protocol (MCP) is an open standard introduced by Anthropic in November 2024, now governed by the Linux Foundation, that standardizes how AI systems (LLMs) integrate with external tools, data sources, and services. MCP re-uses the message-flow ideas of the Language Server Protocol (LSP) and is transported over JSON-RPC 2.0.

**The analogy**: Just as LSP standardized how editors talk to language servers (solving the N editors x M languages problem), MCP standardizes how LLMs talk to tools (solving the N agents x M tools problem). MCP is "USB-C for AI" -- one universal connector replacing dozens of custom integrations.

**Governance**: Following adoption by OpenAI, Google DeepMind, and others, MCP is now hosted by the Linux Foundation. The specification is open source and community-driven.

**Specification versions**:

```
VERSION         DATE          KEY ADDITIONS
-------         ----          -------------
2024-11-05      Nov 2024      Initial release
2025-03-26      Mar 2025      Streamable HTTP transport, SSE deprecated
2025-06-18      Jun 2025      Output schemas, JSON Schema 2020-12 default
2025-11-25      Nov 2025      Tasks primitive, OAuth 2.1, extensions framework (CURRENT)
```

Sources:
- https://modelcontextprotocol.io/specification/2025-11-25
- https://github.com/modelcontextprotocol

---

### 1.2 Wire Protocol: JSON-RPC 2.0

MCP uses JSON-RPC 2.0 as its wire format. All messages MUST be UTF-8 encoded. There are three fundamental message types:

#### Request (Client to Server or Server to Client)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_codebase",
    "arguments": { "path": "/home/user/my-project" }
  }
}
```

Fields: `jsonrpc` (always "2.0"), `id` (unique string or number), `method` (operation name), `params` (optional arguments).

#### Response (Server to Client)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Analysis complete: 3,105 entities, 15,547 edges..."
      }
    ]
  }
}
```

Success responses contain `result`. Error responses contain `error` with `code`, `message`, and optional `data`. The `result` and `error` fields are mutually exclusive.

#### Notification (one-way, no response expected)

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/resources/updated",
  "params": { "uri": "codebase://entities" }
}
```

Notifications have no `id` field. They are fire-and-forget messages used for progress updates, state change alerts, and lifecycle events.

---

### 1.3 Connection Lifecycle: The Initialize Handshake

Every MCP session begins with a three-step handshake:

**Step 1**: Client sends `initialize` request with its protocol version and capabilities.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-11-25",
    "capabilities": {
      "roots": { "listChanged": true },
      "sampling": {}
    },
    "clientInfo": {
      "name": "ClaudeDesktop",
      "version": "3.2.0"
    }
  }
}
```

**Step 2**: Server responds with its chosen protocol version and advertised capabilities.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2025-11-25",
    "capabilities": {
      "tools": {},
      "resources": { "subscribe": true, "listChanged": true },
      "prompts": { "listChanged": true }
    },
    "serverInfo": {
      "name": "rust-llm",
      "version": "2.0.0"
    },
    "instructions": "Code intelligence server for LLMs. Analyzes codebases and provides architectural context, dependency graphs, safety audits, and token-budgeted context windows."
  }
}
```

**Step 3**: Client sends `notifications/initialized` to confirm readiness.

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized"
}
```

Only `ping` requests and server logging are permitted before initialization completes. All other requests are forbidden until the handshake finishes.

---

### 1.4 Transport Options

MCP defines transport as a separate concern from the protocol itself. The specification currently defines two standard transports:

#### stdio (Standard Input/Output)

```
Client (parent process)                    Server (child process)
    |                                          |
    |-------- JSON-RPC via stdin ------------->|
    |<------- JSON-RPC via stdout ------------|
    |         (stderr used for logging)        |
```

- Client launches the MCP server as a subprocess
- Server reads JSON-RPC from stdin, writes to stdout
- Messages are newline-delimited, MUST NOT contain embedded newlines
- Eliminates network stack overhead: microsecond-level response times
- Best for: Local CLI tools, Claude Desktop integration, IDE plugins

**This is our primary transport.** Claude Desktop, Cursor, and VS Code all launch MCP servers as child processes via stdio.

#### Streamable HTTP (the modern remote transport)

```
Client                                     Server
    |                                          |
    |--- POST /mcp (JSON-RPC request) ------->|
    |<-- 200 application/json (simple) -------|
    |    OR                                    |
    |<-- 200 text/event-stream (SSE) ---------|
    |    (for streaming / long-running ops)    |
    |                                          |
    |--- GET /mcp (open SSE stream) --------->|
    |<-- text/event-stream (notifications) ----|
```

- Single HTTP endpoint (e.g., `https://example.com/mcp`) supporting POST and GET
- Simple tool calls return JSON; long-running operations stream via SSE
- Built-in session management via `Mcp-Session-Id` header
- Resumable streams via `Last-Event-ID` header
- Best for: Remote deployments, multi-client scenarios, cloud-hosted servers

#### SSE (Server-Sent Events) -- DEPRECATED

SSE is the legacy transport from MCP 2024-11-05. It required two separate endpoints (POST for requests, GET for SSE stream). Deprecated in favor of Streamable HTTP. Skip entirely.

**Our strategy**: Implement stdio as the primary transport (Claude Desktop, Cursor, VS Code). Add Streamable HTTP as a secondary transport (remote/multi-client). Skip SSE.

Sources:
- https://modelcontextprotocol.io/specification/2025-03-26/basic/transports
- https://mcpcat.io/guides/comparing-stdio-sse-streamablehttp/

---

### 1.5 The Three Core Primitives

MCP servers expose capabilities through three primitives. Each serves a distinct purpose and is controlled by a different party:

```
PRIMITIVE     CONTROLLED BY     ANALOGY              PURPOSE
---------     -------------     -------              -------
Tools         Model (LLM)       POST endpoints       Actions the LLM can invoke
Resources     Application       GET endpoints        Data the application can read
Prompts       User              Templates            Pre-built interaction patterns
```

#### 1.5.1 Tools (Model-Controlled)

Tools are functions that the LLM decides when to call. They accept structured input (JSON Schema) and return results. This is the primary mechanism for agents to interact with external systems.

**Tool definition structure**:

```json
{
  "name": "blast_radius",
  "description": "Analyze the impact of changing a code entity. Returns all entities within N hops that would be affected, ranked by coupling strength.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "entity": {
        "type": "string",
        "description": "The entity key (e.g., 'rust:fn:main' or 'ts:class:UserService')"
      },
      "hops": {
        "type": "integer",
        "description": "Number of dependency hops to traverse (default: 2)",
        "default": 2
      }
    },
    "required": ["entity"]
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "affected_entities": { "type": "array" },
      "total_blast_radius": { "type": "integer" }
    }
  }
}
```

**Tool invocation flow**:
1. Client sends `tools/list` -- server returns all available tools with schemas
2. LLM examines tool descriptions and decides which to call
3. Client sends `tools/call` with tool name and arguments
4. Server executes the tool and returns a `CallToolResult`

**Error handling**: Tool errors SHOULD be returned inside the result object with `isError: true`, NOT as MCP protocol-level errors. This way the LLM can see the error and self-correct.

**Annotations** (metadata for safety): Tools can include annotations like `readOnlyHint`, `destructiveHint`, `idempotentHint`, and `openWorldHint` to help clients make trust/approval decisions.

**Best practices**:
- Keep schemas flat (deep nesting increases token count for the LLM)
- One tool = one well-scoped task
- Descriptions should explain WHEN to use the tool, not just WHAT it does
- Include `required` fields and descriptions on every parameter

#### 1.5.2 Resources (Application-Controlled)

Resources are read-only data that the application (not the LLM) decides when to fetch. Each resource is identified by a URI. Resources provide context to the LLM without giving it the power to trigger side effects.

**Resource discovery**: Client sends `resources/list`, server returns URIs with names, descriptions, and MIME types.

**Resource reading**: Client sends `resources/read` with a URI, server returns content (text or base64-encoded binary).

**URI schemes**: `file://`, `https://`, or custom schemes (e.g., `codebase://entities`, `analysis://metrics`).

**Resource templates** -- parameterized URIs for dynamic content:

```json
{
  "uriTemplate": "codebase://entity/{key}",
  "name": "Code Entity Detail",
  "description": "Detailed information about a specific code entity",
  "mimeType": "application/json"
}
```

**Subscriptions**: Clients can subscribe to resource changes:
1. Client sends `resources/subscribe` with a URI
2. Server sends `notifications/resources/updated` when the resource changes
3. Client fetches updated content via `resources/read`

**List change notifications**: Server sends `notifications/resources/list_changed` when available resources change (e.g., after re-analysis).

#### 1.5.3 Prompts (User-Controlled)

Prompts are pre-defined message templates that users select to initiate structured interactions. Unlike tools (LLM-initiated) and resources (app-initiated), prompts are always user-initiated. An MCP client will NEVER automatically invoke a prompt.

**Prompt definition**:

```json
{
  "name": "analyze_architecture",
  "title": "Analyze Codebase Architecture",
  "description": "Comprehensive architectural analysis: hotspots, cycles, communities, tech debt",
  "arguments": [
    {
      "name": "path",
      "description": "Path to the codebase to analyze",
      "required": true
    },
    {
      "name": "focus",
      "description": "Specific area to focus on (e.g., 'security', 'coupling', 'complexity')",
      "required": false
    }
  ]
}
```

**Prompt retrieval**: Client sends `prompts/get` with prompt name and arguments. Server returns a list of messages (system, user, assistant roles) that the client feeds to the LLM.

**Content types in prompt messages**: Text, images (base64), audio (base64), embedded resources (inline server-managed content).

Sources:
- https://modelcontextprotocol.io/specification/2025-06-18/server/resources
- https://www.merge.dev/blog/mcp-tool-schema
- https://modelcontextprotocol.io/specification/2025-06-18/server/prompts
- https://workos.com/blog/mcp-features-guide

---

### 1.6 November 2025 Specification Updates (2025-11-25)

The latest specification version, released on MCP's one-year anniversary, adds several enterprise-grade features:

#### Tasks Primitive (Experimental)

The most transformative addition. Any request can now return a task handle for long-running operations. Tasks move through states: `working` -> `input_required` -> `completed` / `failed` / `cancelled`.

```
Client: tools/call { name: "analyze_codebase", arguments: { path: "/large/repo" } }
                   + task hint

Server: { task_id: "abc123", status: "working" }

Client: tasks/get { task_id: "abc123" }
Server: { status: "working", progress: { current: 50, total: 100 } }

Client: tasks/result { task_id: "abc123" }
Server: { status: "completed", result: { ... } }
```

**Relevance to us**: Codebase analysis (especially large repos) is inherently long-running. The Tasks primitive lets us return immediately with a task handle and stream progress updates, rather than blocking the LLM.

#### OAuth 2.1 Authorization

- Protected Resource Metadata discovery (RFC 9728)
- Client ID Metadata Documents (CIMD) for decentralized registration
- PKCE mandatory, incremental scope consent
- Machine-to-machine (M2M) OAuth via `client_credentials` flow
- Cross App Access (XAA) for enterprise IdP integration

**Relevance to us**: Minimal for v2.0.0 (local tool, no auth needed for stdio). Important for future cloud-hosted or multi-tenant deployments.

#### Extensions Framework

Optional capabilities that can be negotiated during initialization. Extensions have explicit naming, discovery, and configuration. Popular extensions can graduate into core spec later.

#### Other Changes

- Standardized tool-name format (canonical casing/namespace)
- Icons metadata for tools, resources, templates, prompts
- JSON Schema 2020-12 as default dialect
- SDK tiering system with maintenance commitments

Sources:
- https://modelcontextprotocol.io/specification/2025-11-25/changelog
- https://workos.com/blog/mcp-2025-11-25-spec-update
- http://blog.modelcontextprotocol.io/posts/2025-11-25-first-mcp-anniversary/

---

## 2. The rmcp Crate (Official Rust SDK)

### 2.1 Overview

`rmcp` is the **official** Rust SDK for the Model Context Protocol, maintained at the `modelcontextprotocol/rust-sdk` GitHub repository. It is the most widely adopted choice for building MCP servers and clients in Rust.

**Stats** (as of 2026-02): 3,000+ GitHub stars, 462 forks, 136 contributors, 54 releases. Latest version: 0.15.0.

**Key strengths**:
- Async/await with Tokio runtime
- Proc-macro based tool registration (`#[tool]`, `#[tool_router]`, `#[tool_handler]`)
- Multiple transport backends via feature flags
- Full MCP specification compliance (targets 2025-11-25)
- Type-safe JSON Schema generation via `schemars`

**Companion crates in the ecosystem**:
- `rmcp-server-builder` -- Builder pattern for composing servers from capability providers
- `rmcp-openapi` / `rmcp-openapi-server` -- Bridge OpenAPI specs to MCP tools automatically
- `rmcp-actix-web` -- Actix-web-based transport alternative to built-in Axum

Sources:
- https://crates.io/crates/rmcp
- https://docs.rs/rmcp/latest/rmcp/
- https://github.com/modelcontextprotocol/rust-sdk

---

### 2.2 Installation and Feature Flags

```toml
[dependencies]
rmcp = { version = "0.15", features = ["server", "macros", "transport-io"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1"
```

#### Available Transport Features

```
FEATURE FLAG                             TRANSPORT                USE CASE
------------                             ---------                --------
transport-io                             stdio                    Local tools, Claude Desktop
transport-streamable-http-server         Streamable HTTP server   Remote multi-client
transport-streamable-http-client         Streamable HTTP client   Connecting to HTTP servers
transport-streamable-http-client-reqwest HTTP client (reqwest)    Default HTTP client impl
transport-child-process                  Child process            Client launching subprocess
transport-sse-server                     SSE (legacy)             Legacy web server
transport-sse                            SSE client               Legacy SSE connection
```

**For our MCP server, we need**: `server`, `macros`, `transport-io` (primary), `transport-streamable-http-server` (secondary).

---

### 2.3 Building an MCP Server: Core Pattern

The standard pattern uses three components: a server struct, `#[tool_router]` for tool registration, and `ServerHandler` for protocol handling.

#### Step 1: Define the Server Struct

```rust
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::tool::ToolRouter,
    model::*, tool, tool_handler, tool_router,
    transport::stdio, ErrorData as McpError,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct CodeIntelligenceServer {
    // Shared state: the analyzed codebase
    store: Arc<Mutex<TypedAnalysisStore>>,
    // Tool routing infrastructure (auto-generated by macro)
    tool_router: ToolRouter<Self>,
}
```

#### Step 2: Register Tools with `#[tool_router]`

```rust
#[tool_router]
impl CodeIntelligenceServer {
    fn new(store: Arc<Mutex<TypedAnalysisStore>>) -> Self {
        Self {
            store,
            tool_router: Self::tool_router(),  // auto-generated by macro
        }
    }

    #[tool(description = "Analyze a codebase at the given path. Extracts entities, \
        dependency edges, and architectural metrics. Returns a summary of findings.")]
    async fn analyze_codebase(
        &self,
        #[tool(param, description = "Absolute path to the codebase root directory")]
        path: String,
    ) -> Result<CallToolResult, McpError> {
        // ... analysis logic ...
        Ok(CallToolResult::success(vec![
            Content::text(serde_json::to_string_pretty(&summary)?)
        ]))
    }

    #[tool(description = "Get LLM-optimized context for a code entity within a token budget. \
        Returns the most architecturally relevant code snippets ranked by coupling, \
        PageRank, and community membership.")]
    async fn get_context(
        &self,
        #[tool(param, description = "Entity key (e.g., 'rust:fn:handle_request')")]
        entity: String,
        #[tool(param, description = "Maximum token budget (default: 4096)")]
        token_budget: Option<u32>,
    ) -> Result<CallToolResult, McpError> {
        let budget = token_budget.unwrap_or(4096);
        // ... context extraction logic ...
        Ok(CallToolResult::success(vec![Content::text(context_output)]))
    }

    #[tool(description = "Analyze the blast radius of changing a code entity. \
        Shows all entities within N dependency hops that would be affected, \
        ranked by coupling strength.")]
    async fn blast_radius(
        &self,
        #[tool(param, description = "Entity key to analyze")]
        entity: String,
        #[tool(param, description = "Number of hops to traverse (default: 2)")]
        hops: Option<u32>,
    ) -> Result<CallToolResult, McpError> {
        let hops = hops.unwrap_or(2);
        // ... blast radius logic ...
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}
```

#### Step 3: Implement `ServerHandler` with `#[tool_handler]`

```rust
#[tool_handler]
impl ServerHandler for CodeIntelligenceServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_11_25,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: "rust-llm".to_string(),
                version: "2.0.0".to_string(),
            },
            instructions: Some(
                "Code intelligence server for LLMs. Provides architectural analysis, \
                 dependency graphs, blast radius, safety audits, cross-language edge \
                 detection, and token-budgeted context windows for any codebase."
                    .to_string(),
            ),
        }
    }
}
```

The `#[tool_handler]` macro automatically implements request routing: `tools/list` returns tool definitions from the `ToolRouter`, and `tools/call` dispatches to the appropriate `#[tool]`-annotated method.

#### Step 4: Run the Server

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the analysis store
    let store = Arc::new(Mutex::new(TypedAnalysisStore::new()));

    // Create the MCP server
    let server = CodeIntelligenceServer::new(store);

    // Serve over stdio transport
    let service = server.serve(stdio()).await?;

    // Block until the client disconnects
    service.waiting().await?;

    Ok(())
}
```

Sources:
- https://www.shuttle.dev/blog/2025/07/18/how-to-build-a-stdio-mcp-server-in-rust
- https://mcpcat.io/guides/building-mcp-server-rust/
- https://hackmd.io/@Hamze/S1tlKZP0kx

---

### 2.4 Resource Handling in rmcp

Unlike tools, `rmcp` does **not** currently provide a `#[resource]` macro. Resources must be implemented manually by overriding `ServerHandler` methods:

```rust
async fn list_resources(
    &self,
    _request: ListResourcesRequestParam,
    _context: RequestContext<RoleServer>,
) -> Result<ListResourcesResult, McpError> {
    Ok(ListResourcesResult {
        resources: vec![
            Resource {
                uri: "codebase://entities".to_string(),
                name: "Code Entities".to_string(),
                description: Some("All code entities in the analyzed codebase".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "codebase://graph".to_string(),
                name: "Dependency Graph".to_string(),
                description: Some("Full dependency edge list".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ],
    })
}

async fn read_resource(
    &self,
    request: ReadResourceRequestParam,
    _context: RequestContext<RoleServer>,
) -> Result<ReadResourceResult, McpError> {
    match request.uri.as_str() {
        "codebase://entities" => {
            let store = self.store.lock().await;
            let entities_json = serde_json::to_string_pretty(&store.all_entities())?;
            Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(
                    entities_json,
                    "codebase://entities",
                )],
            })
        }
        uri => Err(McpError::resource_not_found(uri)),
    }
}
```

Enable resources in capabilities with `.enable_resources()` in the `ServerCapabilities` builder.

**Alternative**: The third-party `mcpkit` crate provides a `#[resource(uri_pattern = "...")]` macro, but we should use `rmcp` (the official SDK) and implement resources manually.

Sources:
- https://docs.rs/rmcp/latest/rmcp/
- https://github.com/praxiomlabs/mcpkit

---

### 2.5 Prompt Handling in rmcp

Prompts are also implemented manually via `ServerHandler` overrides:

```rust
async fn list_prompts(
    &self,
    _request: ListPromptsRequestParam,
    _context: RequestContext<RoleServer>,
) -> Result<ListPromptsResult, McpError> {
    Ok(ListPromptsResult {
        prompts: vec![
            PromptInfo {
                name: "analyze_architecture".to_string(),
                title: Some("Analyze Codebase Architecture".to_string()),
                description: Some("Comprehensive architectural review: hotspots, \
                    cycles, communities, tech debt scores".to_string()),
                arguments: vec![
                    PromptArgument {
                        name: "focus".to_string(),
                        description: Some("Area to focus on: 'security', 'coupling', 'all'"
                            .to_string()),
                        required: false,
                    },
                ],
            },
        ],
    })
}

async fn get_prompt(
    &self,
    request: GetPromptRequestParam,
    _context: RequestContext<RoleServer>,
) -> Result<GetPromptResult, McpError> {
    match request.name.as_str() {
        "analyze_architecture" => {
            let focus = request.arguments.get("focus")
                .map(|s| s.as_str())
                .unwrap_or("all");
            Ok(GetPromptResult {
                messages: vec![
                    PromptMessage {
                        role: Role::User,
                        content: Content::text(format!(
                            "Analyze this codebase's architecture with focus on '{}'.\n\
                             Use the following tools in order:\n\
                             1. get_architecture() for SCC, communities, hotspots\n\
                             2. find_cycles() for circular dependencies\n\
                             3. get_tech_debt(top=10) for worst offenders\n\
                             Provide a structured report with actionable recommendations.",
                            focus
                        )),
                    },
                ],
            })
        }
        _ => Err(McpError::invalid_params("Unknown prompt")),
    }
}
```

---

### 2.6 Task Lifecycle Support

rmcp implements the experimental Tasks primitive from the 2025-11-25 spec. Long-running operations (like analyzing a large codebase) can be handled asynchronously:

```
Workflow:
  1. CREATE:  Client calls tool with task hint -> Server returns task_id
  2. INSPECT: Client calls tasks/get -> Server returns status/progress
  3. AWAIT:   Client calls tasks/result -> blocks until completion
  4. CANCEL:  Client calls tasks/cancel -> Server terminates the task
```

---

### 2.7 Alternative Rust MCP SDKs

```
SDK                 CRATE              STRENGTHS                          WHEN TO USE
---                 -----              ---------                          -----------
rmcp (official)     rmcp               Official, #[tool] macros, full     Default choice
                                       spec compliance
rust-mcp-sdk        rust-mcp-sdk       Full 2025-11-25, SSE/HTTP, OAuth   If rmcp lacks a feature
mcpkit              mcpkit             #[resource] macro, #[mcp_server]   If resource macro
                                       unified macro                      ergonomics matter
mcp-framework       mcp-framework      Agent framework, multi-LLM         Building agent clients
```

**Our choice**: `rmcp`. Official, well-maintained (136 contributors), best macro ergonomics for tools, supports all transports we need.

Sources:
- https://github.com/rust-mcp-stack/rust-mcp-sdk
- https://github.com/praxiomlabs/mcpkit

---

## 3. How MCP Clients Consume Servers

### 3.1 Claude Desktop

Claude Desktop is the flagship MCP client. Configuration is via a JSON file:

**Config file location**:
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`

**Configuration for rust-llm**:

```json
{
  "mcpServers": {
    "rust-llm": {
      "command": "/usr/local/bin/rust-llm",
      "args": ["mcp"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

Or, if the user already has an analyzed codebase:

```json
{
  "mcpServers": {
    "rust-llm": {
      "command": "/usr/local/bin/rust-llm",
      "args": ["mcp", "--workspace", "/path/to/project/.rust-llm"],
      "env": {}
    }
  }
}
```

**How it works**:
1. Claude Desktop launches `rust-llm mcp` as a subprocess
2. Communication happens over stdio (stdin/stdout)
3. Claude Desktop sends `initialize`, gets back tool/resource/prompt lists
4. User sees a hammer icon in the chat input showing available tools
5. When Claude decides to use a tool, a permission dialog appears
6. User approves, tool executes, result goes back to Claude

**Verification**: After restart, check for the hammer icon. Logs at `~/Library/Logs/Claude/mcp-server-rust-llm.log`.

Sources:
- https://support.claude.com/en/articles/10949351-getting-started-with-local-mcp-servers-on-claude-desktop
- https://modelcontextprotocol.io/docs/develop/connect-local-servers

---

### 3.2 Cursor IDE

Cursor supports MCP servers at two levels: project-level and global.

**Project-level** (recommended for team sharing): `.cursor/mcp.json` in project root.

```json
{
  "mcpServers": {
    "rust-llm": {
      "command": "/usr/local/bin/rust-llm",
      "args": ["mcp", "--workspace", ".rust-llm"]
    }
  }
}
```

**Global** (for tools you want everywhere): `~/.cursor/mcp.json`.

**Transport support**: Cursor supports stdio (local) and Streamable HTTP (remote).

**Usage**: In Cursor's Agent mode, tools from MCP servers appear automatically. Cursor prompts for approval before each tool call. "Yolo mode" auto-approves tool calls.

Sources:
- https://claudefa.st/blog/tools/mcp-extensions/cursor-mcp-setup
- https://natoma.ai/blog/how-to-enabling-mcp-in-cursor

---

### 3.3 VS Code (via GitHub Copilot)

VS Code's MCP support is integrated with GitHub Copilot Chat. Configuration is stored in `.vscode/mcp.json` (shareable via source control).

```json
{
  "servers": {
    "rust-llm": {
      "type": "stdio",
      "command": "/usr/local/bin/rust-llm",
      "args": ["mcp"]
    }
  }
}
```

**MCP Server Gallery**: VS Code's Extensions view has an MCP server gallery (`@mcp` search filter) that lists servers from the GitHub MCP server registry.

**Agent Mode**: Open Copilot Chat in Agent Mode to see and enable MCP tools. VS Code prompts for trust confirmation when starting an MCP server for the first time.

**Transport**: VS Code supports stdio (local) and HTTP/SSE (remote).

Sources:
- https://code.visualstudio.com/api/extension-guides/ai/mcp
- https://code.visualstudio.com/docs/copilot/customization/mcp-servers

---

### 3.4 Claude Code (CLI)

Claude Code supports MCP via project-level `.mcp.json` or the `claude mcp add` command:

```bash
claude mcp add rust-llm /usr/local/bin/rust-llm mcp
```

---

### 3.5 Custom Agents (Programmatic MCP Clients)

#### Using rmcp as a Client (Rust)

```rust
use rmcp::{ServiceExt, transport::{TokioChildProcess, ConfigureCommandExt}};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Launch rust-llm as a subprocess
    let client = ().serve(TokioChildProcess::new(
        Command::new("rust-llm").configure(|cmd| {
            cmd.arg("mcp");
        })?
    )).await?;

    // List available tools
    let tools = client.list_tools().await?;
    println!("Available tools: {:?}", tools);

    // Call a tool
    let result = client.call_tool("analyze_codebase", json!({
        "path": "/home/user/my-project"
    })).await?;

    println!("Result: {:?}", result);
    Ok(())
}
```

#### Using mcp-use (Python)

```python
from mcp_use import McpClient

client = McpClient(command=["rust-llm", "mcp"])
result = client.call_tool("get_context", {
    "entity": "rust:fn:main",
    "token_budget": 4096
})
```

Sources:
- https://github.com/modelcontextprotocol/rust-sdk
- https://github.com/mcp-use/mcp-use

---

## 4. Mapping Parseltongue Capabilities to MCP Primitives

### 4.1 Tools (LLM-Invocable Actions)

These are functions the LLM can call. Each maps to analysis capabilities from the v2.0.0 architecture.

```
TOOL NAME                 DESCRIPTION                                    MAPS TO
---------                 -----------                                    -------
analyze_codebase          Ingest and analyze a codebase at a path        rust-llm-01 + 04 + 05
get_context               Token-budgeted LLM-optimized context window    rust-llm-context
blast_radius              Impact analysis: what breaks if X changes      rust-llm-graph
search_entities           Fuzzy search across all code entities          rust-llm-core
get_entity_detail         Full detail view for a specific entity         rust-llm-05
get_architecture          SCC + communities + hotspots + k-core          rust-llm-graph
find_cycles               Detect circular dependency chains              rust-llm-graph
find_unsafe_chains        Trace all paths reaching unsafe code           rust-llm-safety
get_tech_debt             SQALE tech debt scores + rankings              rust-llm-graph
get_centrality            PageRank / betweenness rankings                rust-llm-graph
get_coupling_metrics      CBO / LCOM / RFC / WMC for entities            rust-llm-graph
detect_cross_lang_edges   Find FFI, WASM, PyO3, gRPC boundaries          rust-llm-crosslang
get_callers               Reverse dependency: who calls this entity?     rust-llm-graph
get_callees               Forward dependency: what does this call?       rust-llm-graph
get_statistics            Codebase overview: counts, languages, files    rust-llm-05
run_rule                  Execute a custom Ascent analysis rule          rust-llm-rules
```

#### Tool Schema: `get_context` (the killer tool)

```json
{
  "name": "get_context",
  "description": "Get the most architecturally relevant code context for a given entity, optimized for your token budget. Uses PageRank, blast radius, SCC membership, Leiden community clustering, and coupling metrics to rank what to include. Returns structured context, not raw file dumps. Call this when you need to understand or modify a specific entity.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "entity": {
        "type": "string",
        "description": "Entity key (e.g., 'rust:fn:handle_request', 'ts:class:UserService')"
      },
      "token_budget": {
        "type": "integer",
        "description": "Maximum tokens to include in context (default: 4096)",
        "default": 4096
      },
      "include_callers": {
        "type": "boolean",
        "description": "Include entities that call this entity (default: true)",
        "default": true
      },
      "include_callees": {
        "type": "boolean",
        "description": "Include entities this entity calls (default: true)",
        "default": true
      }
    },
    "required": ["entity"]
  }
}
```

#### Tool Schema: `blast_radius`

```json
{
  "name": "blast_radius",
  "description": "Compute the impact radius of changing a code entity. Returns all entities within N hops of the dependency graph, ranked by coupling strength. Use this before modifying code to understand what else might break.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "entity": {
        "type": "string",
        "description": "Entity key to analyze"
      },
      "hops": {
        "type": "integer",
        "description": "Dependency hops to traverse (default: 2, max: 5)",
        "default": 2,
        "minimum": 1,
        "maximum": 5
      }
    },
    "required": ["entity"]
  }
}
```

#### Tool Schema: `find_unsafe_chains`

```json
{
  "name": "find_unsafe_chains",
  "description": "Find all call chains in the codebase that reach unsafe code blocks. Uses transitive closure analysis (Ascent Datalog) to trace from any function through its call graph to unsafe blocks. Essential for security audits and safety-critical Rust codebases.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "max_depth": {
        "type": "integer",
        "description": "Maximum call chain depth to trace (default: 10)",
        "default": 10
      },
      "filter_crate": {
        "type": "string",
        "description": "Optional: limit to entities in a specific crate/module"
      }
    },
    "required": []
  }
}
```

#### Tool Schema: `analyze_codebase`

```json
{
  "name": "analyze_codebase",
  "description": "Analyze a codebase at the given path. Extracts all code entities (functions, structs, classes, traits, etc.), dependency edges (calls, uses, implements), and computes architectural metrics (SCC, PageRank, coupling, tech debt). This is typically the first tool to call. Returns a summary; use other tools to drill into specifics.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": {
        "type": "string",
        "description": "Absolute path to the codebase root directory"
      }
    },
    "required": ["path"]
  }
}
```

---

### 4.2 Resources (Application-Readable Data)

Resources are read-only data exposed via URIs. The application (not the LLM) decides when to read these.

```
RESOURCE URI                         DESCRIPTION                      CONTENT TYPE
------------                         -----------                      ------------
codebase://entities                  All code entities (summary)      application/json
codebase://entities/{key}            Single entity detail             application/json
codebase://graph                     Full dependency edge list        application/json
codebase://graph/callers/{key}       Reverse callers for entity       application/json
codebase://graph/callees/{key}       Forward callees for entity       application/json
codebase://metrics                   Analysis metrics summary         application/json
codebase://metrics/tech-debt         SQALE tech debt scores           application/json
codebase://metrics/coupling          Coupling/cohesion metrics        application/json
codebase://metrics/centrality        PageRank/betweenness rankings    application/json
codebase://architecture/scc          Strongly connected components    application/json
codebase://architecture/communities  Leiden community clusters        application/json
codebase://architecture/kcore        K-core decomposition layers      application/json
codebase://unsafe-chains             All unsafe call chains           application/json
codebase://cross-lang-edges          Cross-language boundaries        application/json
codebase://statistics                Codebase overview stats          application/json
```

**Resource templates** (parameterized):

```json
[
  {
    "uriTemplate": "codebase://entities/{key}",
    "name": "Entity Detail",
    "description": "Full detail for a specific code entity including signature, file, line, dependencies"
  },
  {
    "uriTemplate": "codebase://graph/callers/{key}",
    "name": "Entity Callers",
    "description": "All entities that call/depend on the specified entity"
  },
  {
    "uriTemplate": "codebase://graph/callees/{key}",
    "name": "Entity Callees",
    "description": "All entities called/used by the specified entity"
  }
]
```

**Subscriptions**: When file watching detects changes and re-analysis completes, the server sends `notifications/resources/updated` for all affected resource URIs. This enables live-updating context for long conversations.

---

### 4.3 Prompts (User-Initiated Templates)

Pre-built conversation starters that structure how users interact with the code analysis tools.

```
PROMPT NAME                    TITLE                             ARGUMENTS
-----------                    -----                             ---------
analyze_architecture           Analyze Architecture              focus?: string
find_security_concerns         Security Audit                    scope?: string
understand_entity              Explain This Code                 entity: string
review_change_impact           Review Change Impact              entity: string, change?: string
find_tech_debt                 Tech Debt Assessment              top?: integer
onboard_to_codebase            Codebase Onboarding               (none)
find_cross_lang_issues         Cross-Language Boundary Audit      (none)
```

#### Example: `analyze_architecture` prompt response

```json
{
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "Perform a comprehensive architectural analysis of this codebase. Follow these steps:\n\n1. Call get_architecture() to get SCC decomposition, Leiden communities, and coupling hotspots\n2. Call find_cycles() to detect circular dependencies\n3. Call get_tech_debt(top=10) to identify the worst technical debt\n4. Call get_centrality(method='pagerank') to find architectural pillars\n\nFor each finding, explain:\n- WHY it matters (what risk does it create?)\n- WHERE it is (specific entities and files)\n- WHAT to do about it (actionable recommendation)\n\nStructure your report as:\n## Executive Summary (3 bullet points)\n## Architectural Health Score (A-F grade with justification)\n## Critical Findings (ranked by severity)\n## Recommendations (prioritized action items)"
      }
    }
  ]
}
```

#### Example: `find_security_concerns` prompt response

```json
{
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "Perform a security audit of this codebase. Follow these steps:\n\n1. Call find_unsafe_chains() to trace all paths to unsafe code\n2. Call detect_cross_lang_edges() to find FFI boundaries\n3. Call search_entities(query='password OR secret OR token OR key') to find sensitive data handling\n4. Call blast_radius(entity=X) for each unsafe entry point\n\nFor each concern, classify as: CRITICAL / HIGH / MEDIUM / LOW\n\nStructure your report as:\n## Security Summary\n## Unsafe Code Paths (with full call chains)\n## FFI Boundary Risks\n## Sensitive Data Handling\n## Recommendations"
      }
    }
  ]
}
```

---

## 5. Reference MCP Servers to Study

### 5.1 Official Reference Implementations

| Server | Language | Relevance |
|---|---|---|
| **Everything** | TypeScript | Reference test server: tools + resources + prompts |
| **Filesystem** | TypeScript | Secure file operations, access control patterns |
| **Git** | TypeScript | Code repository operations |
| **Memory** | TypeScript | Knowledge graph persistence |
| **Fetch** | TypeScript | Web content fetching for LLMs |

Source: https://github.com/modelcontextprotocol/servers

### 5.2 Code Analysis MCP Servers (Competitors / Inspirations)

| Server | Language | What It Does | Key Takeaway |
|---|---|---|---|
| **ast-mcp-server** | Python | AST/ASG analysis via tree-sitter, incremental parsing | Validates our tree-sitter approach for MCP |
| **code-graph-mcp** | TypeScript | 25+ language AST, dependency graph, ast-grep backend | Similar scope but no Datalog, no cross-lang edges, no token budgeting |
| **codegraph-mcp** | TypeScript | Lightweight code graph (TS+Python), JSON storage | Simple but limited; shows the demand exists |
| **Code Pathfinder** | TypeScript | 5-pass AST indexing, call graph, dataflow tracking | Strong analysis but no architectural metrics |
| **code-analysis-mcp** | Python | Natural language code exploration | High-level NLP focus, not graph/architecture |
| **ast-grep MCP** | TypeScript | Pattern matching via AST | Complementary tool, not a competitor |

Sources:
- https://github.com/angrysky56/ast-mcp-server
- https://github.com/entrepeneur4lyf/code-graph-mcp
- https://codepathfinder.dev/mcp
- https://github.com/punkpeye/awesome-mcp-servers

### 5.3 Rust-Built MCP Servers (Architecture Reference)

| Server | What It Does | Why Study It |
|---|---|---|
| **rustfs-mcp** | S3-compatible object storage for AI | Production rmcp usage patterns |
| **hyper-mcp** | WASM plugin-based extensibility | Plugin architecture |
| **rust-docs-mcp** | Fetches current Rust crate docs | Similar domain (code intelligence) |
| **terminator** | Desktop automation MCP server | Complex tool orchestration |
| **mcp-rs-template** | Minimal Rust MCP server template | Boilerplate structure |

### 5.4 What No One Has (Our Differentiators)

```
EXISTING MCP CODE SERVERS:        WHAT rust-llm ADDS:
---------------------------       -------------------
AST parsing (tree-sitter)         + Datalog reasoning (Ascent)
Call graph traversal               + 7 graph algorithms (SCC, PageRank, k-core, Leiden, ...)
Symbol search                     + Cross-language edge detection (FFI, WASM, PyO3, gRPC)
Basic code navigation             + Token-budgeted LLM-optimized context windows
                                  + Architectural comprehension (not just navigation)
                                  + Custom rule engine (CodeQL-like, embeddable)
                                  + Safety audit (unsafe chains, taint analysis)
                                  + SQALE tech debt scoring (ISO 25010)
                                  + 12 language support with typed facts
```

**No existing MCP server provides architectural analysis, token-budgeted context, cross-language edge detection, or Datalog-based reasoning.** This is our blue ocean.

---

## 6. MCP vs HTTP: Coexistence Strategy

### 6.1 They Operate at Different Layers

```
MCP and HTTP are NOT alternatives. They operate at different layers:

  HTTP (REST):  App-to-app communication. Stateless. Fixed endpoints.
                Humans read docs, write integration code.

  MCP:          AI-to-tool communication. Stateful (context-aware).
                Runtime discovery. Self-describing.
                LLMs examine tool descriptions and decide what to call.

  MCP often USES HTTP internally (Streamable HTTP transport).
  But MCP ABSTRACTS away HTTP for the AI consumer.
```

### 6.2 When to Use Which

```
USE MCP WHEN:                              USE HTTP WHEN:
  - Consumer is an LLM / AI agent            - Consumer is a human dashboard
  - Consumer needs runtime discovery          - Consumer is a CI/CD pipeline with fixed integration
  - Interactions are context-aware            - Interactions are stateless CRUD
  - Tool selection is dynamic                 - Endpoints are hardcoded
  - You want automatic integration            - You need fine-grained control
    with Claude/Cursor/VS Code

EXAMPLES:
  MCP: Claude asks "What's the blast         HTTP: Grafana dashboard polls
       radius of changing handle_request?"         /metrics every 30 seconds

  MCP: Cursor agent decides to call          HTTP: CI pipeline POSTs to
       get_context() during code review            /analyze and checks exit code

  MCP: Custom agent orchestrates             HTTP: Web UI fetches
       analyze -> get_context -> modify            /entities for rendering
```

### 6.3 Our Architecture: MCP-First, HTTP-Available

```
+-----------------------------------------------------+
|                  rust-llm binary                     |
|                                                      |
|   +----------------------------------------------+  |
|   |           Analysis Engine                     |  |
|   |  (rust-llm-core + context + graph + safety    |  |
|   |   + crosslang + rules)                        |  |
|   +--------------------+-------------------------+  |
|                        |                             |
|                +-------+--------+                    |
|                |  Shared Layer   |                    |
|                |  (query logic,  |                    |
|                |   formatting)   |                    |
|                +---+--------+---+                    |
|                    |        |                         |
|          +---------+--+  +--+---------+               |
|          | MCP Server |  | HTTP Server|               |
|          | (PRIMARY)  |  | (SECONDARY)|               |
|          |            |  |            |               |
|          | stdio      |  | Axum      |               |
|          | + HTTP     |  | REST API  |               |
|          | transport  |  |           |               |
|          +------------+  +-----------+               |
|                                                      |
|   CLI commands:                                      |
|     rust-llm mcp          <- MCP server (stdio)      |
|     rust-llm mcp --http   <- MCP server (HTTP)       |
|     rust-llm serve        <- HTTP REST server         |
|     rust-llm ingest .     <- Analyze codebase         |
+------------------------------------------------------+
```

**Key design principle**: The MCP server and HTTP server share the same analysis engine and query logic. They are different interfaces to the same capabilities. Neither depends on the other. A user can run just MCP, just HTTP, or both simultaneously.

### 6.4 Feature Parity Matrix

```
CAPABILITY                  MCP TOOL              HTTP ENDPOINT
----------                  --------              -------------
Analyze codebase            analyze_codebase      POST /analyze
Get context                 get_context           GET /context?entity=X&tokens=N
Blast radius                blast_radius          GET /blast-radius?entity=X&hops=N
Search entities             search_entities       GET /search?q=pattern
Get entity detail           get_entity_detail     GET /entities/{key}
Architecture overview       get_architecture      GET /architecture
Find cycles                 find_cycles           GET /cycles
Unsafe chains               find_unsafe_chains    GET /unsafe-chains
Tech debt                   get_tech_debt         GET /tech-debt?top=N
Cross-language edges        detect_cross_lang     GET /cross-lang-edges
Run custom rule             run_rule              POST /rules/run
Health check                (via ping)            GET /health
Statistics                  get_statistics        GET /statistics

PLUS (MCP-only):
  Resources                 codebase://entities   (no HTTP equivalent)
  Prompts                   analyze_architecture  (no HTTP equivalent)
  Subscriptions             resource subscribe    (no HTTP equivalent)
  Tasks                     async task handles    (no HTTP equivalent)
```

### 6.5 Why MCP Is Primary

```
1. THE PRIMARY USER IS AN LLM.
   LLMs speak MCP natively. HTTP requires the LLM to know URLs, parse
   responses, handle pagination. MCP gives structured tool lists.

2. ZERO CONFIGURATION FOR THE USER.
   Drop rust-llm config into claude_desktop_config.json. Done.
   No "start a server, get a URL, configure the client, handle CORS."

3. AUTOMATIC DISCOVERY.
   The LLM sees all tools and their descriptions.
   It decides which to call based on the task.
   No hardcoded endpoint integration.

4. CONTEXT-AWARE SESSIONS.
   MCP sessions maintain context. The server knows what the LLM
   has already queried and can optimize subsequent responses.

5. ECOSYSTEM MOMENTUM.
   Claude Desktop, Cursor, VS Code, OpenAI, Google DeepMind --
   all support MCP. Being MCP-native means automatic integration
   with every MCP-compatible client, present and future.
```

Sources:
- https://docs.roocode.com/features/mcp/mcp-vs-api
- https://www.codecademy.com/article/mcp-vs-api-architecture-and-use-cases
- https://www.stainless.com/mcp/from-rest-api-to-mcp-server

---

## 7. Implementation Roadmap for rust-llm MCP Server

### 7.1 Crate Structure

```
crates/rust-llm-07-mcp-server/
  Cargo.toml
  src/
    lib.rs                    # Public API
    server.rs                 # MCP ServerHandler implementation
    tools/
      mod.rs                  # Tool registration
      analyze.rs              # analyze_codebase tool
      context.rs              # get_context tool
      blast_radius.rs         # blast_radius tool
      search.rs               # search_entities tool
      architecture.rs         # get_architecture, find_cycles, etc.
      safety.rs               # find_unsafe_chains tool
      crosslang.rs            # detect_cross_lang_edges tool
      rules.rs                # run_rule tool
    resources/
      mod.rs                  # Resource handlers
      entities.rs             # codebase://entities resources
      graph.rs                # codebase://graph resources
      metrics.rs              # codebase://metrics resources
    prompts/
      mod.rs                  # Prompt handlers
      architecture.rs         # analyze_architecture prompt
      security.rs             # find_security_concerns prompt
      onboarding.rs           # onboard_to_codebase prompt
    transport.rs              # stdio + HTTP transport setup
```

### 7.2 Cargo.toml

```toml
[package]
name = "rust-llm-07-mcp-server"
version = "2.0.0"
edition = "2021"

[dependencies]
# MCP SDK
rmcp = { version = "0.15", features = [
    "server",
    "macros",
    "transport-io",
    "transport-streamable-http-server"
] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Error handling
anyhow = "1"

# Internal dependencies
rust-llm-core = { path = "../rust-llm-core" }
rust-llm-context = { path = "../rust-llm-context", optional = true }
rust-llm-graph = { path = "../rust-llm-graph", optional = true }
rust-llm-safety = { path = "../rust-llm-safety", optional = true }
rust-llm-crosslang = { path = "../rust-llm-crosslang", optional = true }
rust-llm-rules = { path = "../rust-llm-rules", optional = true }

[features]
default = ["full"]
full = ["context", "graph", "safety", "crosslang", "rules"]
context = ["dep:rust-llm-context"]
graph = ["dep:rust-llm-graph"]
safety = ["dep:rust-llm-safety"]
crosslang = ["dep:rust-llm-crosslang"]
rules = ["dep:rust-llm-rules"]
```

### 7.3 Build Order

```
Phase 1: Foundation (MVP)
  1. rust-llm-core       (types, traits, extraction)
  2. rust-llm-07-mcp     (server with analyze_codebase + search + get_statistics)
  3. Wire up stdio transport
  4. Test with Claude Desktop

Phase 2: The Killer Tools
  5. rust-llm-context    (get_context tool)
  6. rust-llm-graph      (blast_radius, architecture, cycles, centrality, tech_debt)
  7. Wire into MCP server

Phase 3: Differentiation
  8. rust-llm-safety     (find_unsafe_chains)
  9. rust-llm-crosslang  (detect_cross_lang_edges)
  10. rust-llm-rules     (run_rule)
  11. Wire into MCP server

Phase 4: Resources + Prompts
  12. Add resource handlers (codebase:// URIs)
  13. Add prompt templates
  14. Add resource subscriptions (file watcher notifications)

Phase 5: HTTP Transport
  15. Add Streamable HTTP transport option
  16. rust-llm-06-http-server (traditional REST, separate crate)
```

### 7.4 Testing Strategy

```
UNIT TESTS:
  - Each tool handler tested in isolation
  - Mock TypedAnalysisStore with known data
  - Verify JSON Schema compliance of tool definitions
  - Verify tool results match expected format

INTEGRATION TESTS:
  - Full MCP lifecycle: initialize -> list_tools -> call_tool -> result
  - Use rmcp client to test against the server
  - Test with real codebases (test fixtures)

CONFORMANCE TESTS:
  - MCP Inspector (official testing tool)
  - Verify protocol compliance with MCP specification
  - Test all three message types: request, response, notification

END-TO-END TESTS:
  - Claude Desktop integration (manual)
  - Cursor integration (manual)
  - Custom agent pipeline (automated)
```

### 7.5 Key Implementation Decisions

```
DECISION                         CHOICE                  RATIONALE
--------                         ------                  ---------
MCP SDK                          rmcp (official)         Most mature, best macros, official
Primary transport                stdio                   Claude Desktop, Cursor, VS Code use it
Secondary transport              Streamable HTTP         Remote/multi-client scenarios
Resource URIs                    codebase:// scheme      Custom scheme, clear namespace
Tool descriptions                LLM-optimized           Explain WHEN to use, not just WHAT
Error handling                   In-result errors        LLM can see and self-correct
Long-running analysis            Tasks primitive         Return immediately, stream progress
State management                 Arc<Mutex<Store>>       Shared across async tool handlers
Logging                          tracing crate           Structured, stderr only (stdout = MCP)
```

### 7.6 Critical Constraint: stdout is Sacred

When running as an MCP stdio server, stdout is exclusively for JSON-RPC messages. All other output (logs, debug, progress) MUST go to stderr. This is the most common mistake in MCP server implementations.

```rust
// WRONG: This breaks MCP
println!("Analyzing codebase...");

// RIGHT: This is safe
eprintln!("Analyzing codebase...");

// BEST: Use tracing with stderr subscriber
tracing::info!("Analyzing codebase...");
// (configured with tracing_subscriber writing to stderr)
```

---

## Summary

### What MCP Gives Us

1. **Automatic integration** with Claude Desktop, Cursor, VS Code, and every future MCP-compatible client
2. **Runtime tool discovery** -- LLMs see our capabilities and decide what to call
3. **Structured protocol** -- JSON-RPC 2.0, capability negotiation, typed schemas
4. **Three primitives** for different interaction patterns: Tools (LLM calls), Resources (app reads), Prompts (user initiates)
5. **Task lifecycle** for long-running codebase analysis
6. **Resource subscriptions** for live-updating context on file changes

### What We Bring That Nobody Else Has

1. **Token-budgeted LLM context** (`get_context`) -- architecturally ranked, not TF-IDF
2. **Cross-language edge detection** -- FFI, WASM, PyO3, gRPC boundaries
3. **Datalog reasoning** (Ascent) -- transitive closure, taint analysis, unsafe chain tracing
4. **7 graph algorithms** -- SCC, PageRank, k-core, Leiden, entropy, CK metrics, SQALE
5. **Custom rule engine** -- institutional knowledge encoded as Ascent rules
6. **12 language support** -- unified typed facts across all languages

### The MCP-First Principle

```
v1.x: parseltongue pt08-http-code-query-server  (HTTP-first, human-optimized)
v2.0: rust-llm mcp                               (MCP-first, LLM-optimized)

The primary user of v2.0.0 is NOT a human.
The primary user is an LLM.
MCP is how LLMs consume tools.
Therefore, MCP is the primary interface.
HTTP exists for humans and dashboards.
```
