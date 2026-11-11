# Rust Web Backend Playbook

Use this file for service anatomy, request flow boundaries, persistence seams, and external API integration choices.

## 1. Service Anatomy

Treat a backend request path as five layers:

1. transport boundary: HTTP method, route, auth context, request parsing
2. handler boundary: orchestration and error mapping
3. domain boundary: validated types and business rules
4. persistence boundary: database queries, transactions, and migrations
5. integration boundary: calls to email, payment, or other remote systems

Keep each layer narrow enough that failures have an obvious owner.

## 2. Request Handlers And App State

- Keep handlers thin. They should parse inputs, call application logic, and map failures to responses.
- App state should hold long-lived shared resources such as connection pools, HTTP clients, or configuration snapshots.
- Avoid burying business rules inside extractor glue or response formatting code.
- Prefer one place to wire dependencies for the service rather than constructing clients inside handlers.

Representative ecosystem examples:
- HTTP: `actix-web`, `axum`
- async runtime: `tokio`
- shared state: framework data extractors or explicit dependency containers

## 3. Domain Types And Validation Boundaries

- Use transport types for input parsing and shape validation.
- Use domain types for invariants that must hold everywhere after parsing.
- Convert at the edge. Do not let unchecked strings or loosely typed payloads travel through the service core.
- Prefer newtypes and `TryFrom`/constructor-based validation for values such as email addresses, passwords, tokens, and identifiers.

This is where executable specs matter:
- invalid input paths should become explicit `REQ-WEB-*` contracts
- empty, malformed, and duplicate cases should have named tests

## 4. Persistence Boundaries

- Keep SQL and migration concerns behind a small persistence interface or module boundary.
- Prefer one transaction owner per workflow.
- Make side-effects and durability order explicit when multiple writes must succeed together.
- Treat migrations as part of delivery, not an afterthought.
- If a schema change alters runtime assumptions, pair the code change with rollout sequencing and test fixtures that reflect the new shape.

Representative ecosystem examples:
- SQL + migrations: `sqlx`
- database: `Postgres`

## 5. External API Client Boundary

- Wrap remote APIs behind a typed client with one responsibility and an explicit error surface.
- Reuse a shared HTTP client instead of creating one per request.
- Separate request construction, credential injection, response handling, and failure classification.
- Treat timeout behavior, retry policy, and status-code handling as part of the client contract.
- Keep the client mockable with transport-level tests or HTTP mocking.

Representative ecosystem examples:
- HTTP client: `reqwest`
- HTTP mocking: `wiremock`

## 6. Endpoint-To-Worker Split

Keep work inline when:
- the request is short
- the response must reflect immediate success or failure
- deduplication and retry concerns are minor

Move work behind a queue or worker loop when:
- the request fans out to slow or failure-prone remote systems
- retries must be durable
- idempotency or replay matters
- the request would otherwise hold open a client connection while doing long-running work

When splitting:
- define what the endpoint commits durably
- define what the worker consumes
- define what “already processed” means
- define what state transitions are observable for operators
