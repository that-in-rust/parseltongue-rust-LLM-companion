# v200-Decision-log-01
Status: Active
Purpose: Record binding scope and architecture decisions for V200.

## 2026-02-17 — Defer `rust-llm-context-packer` to V210
Decision:
- Remove `rust-llm-context-packer` from active V200 scope.
- Track it only in `docs/v210-backlog.md` with explicit re-entry criteria.

Why this decision was made:
1. V200 objective is dependency-graph-grounded context adequacy, not token-minimization strategy.
2. Context-packing is an optimization layer that can mask weak retrieval/fidelity in core graph and read-path contracts.
3. Deferring this crate reduces coupling, pass count, and contract surface so high-risk foundations converge faster.
4. This preserves optionality: packing is still available later, but only when evidence shows read-path-only orchestration is insufficient.

Decision principle:
- Prove graph fidelity and read-path determinism first; add context packing only after measured insufficiency and explicit product need.

Immediate scope impact:
- V200 active scope is 8 crates (context-packer excluded).
- Pass ledger/probe sequencing remains contiguous with no context-packer pass in V200.

## 2026-02-17 — Promote lifecycle + companion readiness bundle to V200 requirements
Decision:
- Promote the following `PRD_v173` ideas to explicit V200 pre-PRD requirements:
  - `#7` route prefix nesting
  - `#8` auto port + port file lifecycle
  - `#10` shutdown CLI command
  - `#25` XML-tagged response categories
  - `#27` project slug in URL path
  - `#28` slug-aware port file naming
  - `#29` token count at ingest
  - `#35` data-flow tree-sitter queries (`assign` / `param` / `return`)

Scope/architecture stance:
- These are requirement upgrades, not topology changes.
- V200 remains the same 8-crate core dependency graph.
- Tauri app work is treated as a companion consumer of stable gateway contracts (external app track), not as a new core crate.

Crate mapping:
- `rust-llm-interface-gateway`: `#7`, `#8`, `#10`, `#25`, `#27`, `#28`
- `rust-llm-store-runtime`: `#29`
- `rust-llm-tree-extractor`: `#35`

Decision principle:
- Lifecycle contracts and context clarity are first-class product requirements when they improve deterministic operation, discoverability, and LLM usability without introducing new core crates.

## 2026-02-22 — JSON-RPC 2.0 as the MCP wire protocol

Decision:
- Use JSON-RPC 2.0 for all MCP tool calls in `rust-llm-interface-gateway` (MCP server mode).
- This is a protocol compliance requirement, not an open design choice — Anthropic's MCP spec mandates JSON-RPC 2.0.

Alternatives evaluated and rejected:

### REST/HTTP
Pro: universally understood, great tooling.
Con: stateless by design. An MCP server holds session state (loaded graph, workspace path, ingest status). REST requires the client to re-send that state on every request. More critically, REST has no standard way to correlate multiple async tool responses to the requests that triggered them. The `id` field in JSON-RPC solves this directly.
Verdict: rejected — statelessness assumption conflicts with persistent workspace model.

### gRPC / protobuf
Pro: strongly typed, high performance, code-generated clients in any language.
Con: requires `.proto` schema files and a code-generation step. Binary encoding means responses are not human-readable — you cannot inspect them with `jq` or in a terminal. Debugging requires extra toolchain. gRPC is not in the MCP spec. Building a gRPC-to-MCP bridge would add a translation layer with no benefit.
Verdict: rejected — tooling overhead and spec incompatibility outweigh performance gains at this scale (tool calls, not bulk streaming data).

### GraphQL
Pro: flexible field selection, great for open schemas with many optional fields.
Con: designed for open-ended querying over a schema. Our interface is a fixed set of ~16 named tools with defined input/output shapes — that is RPC, not graph querying. GraphQL would require a schema definition language, a resolver layer, and a query parser — all overhead with no benefit over a simple tool-dispatch table.
Verdict: rejected — wrong abstraction; adds schema management cost with no flexibility gain on a fixed tool surface.

### Raw WebSocket + custom protocol
Pro: bidirectional, low latency.
Con: you would be implementing request/response correlation (the `id` field), error codes, batch requests, and notifications from scratch. That is inventing JSON-RPC. Transport-level WebSocket is fine; JSON-RPC 2.0 is the application-level protocol that rides on top.
Verdict: rejected — reinventing a solved protocol.

### Plain HTTP/SSE (without JSON-RPC framing)
Pro: simple, works for streaming.
Con: no standard request/response correlation. If Claude sends two tool calls simultaneously, SSE events have no canonical way to say "this response is for tool call id=5." JSON-RPC `id` solves this.
Verdict: rejected — insufficient for concurrent tool call resolution.

Why JSON-RPC 2.0 wins on its own merits (beyond spec compliance):
1. Transport-agnostic — the same message format works over stdio (local process), HTTP+SSE (remote), and WebSocket. One protocol, three transports.
2. Request/response correlation — the `id` field maps every response to its originating request. Essential when Claude batches tool calls.
3. Human-readable — JSON is inspectable with `jq`, loggable as plain text, debuggable without a schema compiler.
4. Notification support — `id: null` means fire-and-forget (progress events, log lines). No extra protocol needed.
5. No schema compilation — add a new tool by adding a tool descriptor struct; no `.proto` regeneration cycle.
6. Ecosystem fit — every MCP host (Claude Desktop, Claude.ai, future LLM clients) already speaks JSON-RPC 2.0. Zero translation layer.

Decision principle:
- When a protocol is mandated by the spec you implement against, evaluate it critically but adopt it unless a concrete deficiency (not theoretical) demands deviation. JSON-RPC 2.0 has no concrete deficiency for this use case.

Crate impact:
- `rust-llm-interface-gateway`: implements JSON-RPC 2.0 message framing, id correlation, and tool dispatch over stdio.
- No other crates are aware of the wire protocol — they receive typed Rust structs from the gateway.

## 2026-02-22 — Language priority tiers for Rust-aligned development

Decision:
- V200 supports 12 languages via tree-sitter. For a Rust-primary developer, the 5 file families below represent the highest-value subset and should receive the most careful extraction testing.

Tier 1 — Primary (always in scope, deeply tested):
- **Rust** (`.rs`) — primary language; rust-analyzer enrichment on top of tree-sitter.
- **C / C++** (`.c`, `.h`, `.cpp`, `.hpp`) — Rust's native FFI boundary. `bindgen` / `cbindgen` / `cxx` crate are common in any Rust systems project. `ffi_boundary` edges between Rust and C/C++ are a first-class graph concept.
- **TypeScript / TSX** (`.ts`, `.tsx`) — the dominant Rust backend + web frontend pairing (Axum + React/TS). Tauri itself is Rust + TypeScript WebView. `http_boundary` edges between Rust and TS are the canonical cross-language edge in V200.
- **JavaScript / JSX** (`.js`, `.jsx`) — same family as TypeScript; also the primary consumer of Rust-compiled WASM (`wasm-bindgen` emits JS/TS bindings). Any project with a React frontend will mix `.ts` and `.js` files.

Tier 2 — Secondary (supported, lower test priority for Rust-aligned devs):
- **Python** (`.py`) — PyO3 makes Rust-in-Python or Python-in-Rust a real use case. High value if the team mixes Rust extensions with Python orchestration. Lower priority than C/TS for pure Rust-first shops.
- **Go** (`.go`) — increasingly common as a Rust peer in polyglot microservice architectures. Less FFI coupling than C; more likely to appear as a service-boundary peer.

Tier 3 — Peripheral (tree-sitter coverage present, not a Rust-adjacent boundary pattern):
- **Ruby / Rails** (`.rb`) — no natural Rust FFI or WASM boundary. Useful for repos that happen to have Ruby scripts or tooling alongside Rust. Not a priority for Rust-primary devs.
- **Java** (`.java`), **C#** (`.cs`), **Swift** (`.swift`), **PHP** (`.php`) — same story: tree-sitter extraction works, but cross-language edges to Rust are rare in practice.

Scope impact:
- No change to V200 crate scope — all 12 languages remain supported.
- Extraction test harness (`rust-llm-test-harness`) should weight Tier 1 languages most heavily.
- Language-tier metadata can be surfaced in `rust-llm-interface-gateway` docs/API reference so users know which edges carry deep semantic enrichment vs structural extraction only.

Decision principle:
- Support all 12 languages uniformly at the tree-sitter layer; invest in depth (rust-analyzer, confidence scoring, boundary detection) proportional to actual Rust ecosystem coupling patterns.
