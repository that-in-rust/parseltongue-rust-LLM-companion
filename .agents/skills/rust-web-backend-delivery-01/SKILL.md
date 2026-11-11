---
name: rust-web-backend-delivery-01
description: Plan, build, harden, and review Rust HTTP backends and APIs with executable requirements, integration-first testing, persistence and migration safety, config and secrets discipline, structured telemetry, auth and session handling, external API clients, deployment safety, idempotency, retries, and background processing. Use for Actix-, Axum-, or similar Rust web services, not generic Rust libraries, CLIs, parsers, or FFI code.
---

# Rust Web Backend Delivery 01

Use this skill to turn Rust backend requests into decision-complete work packets that are spec-first, service-aware, and production-biased.

## Mode Selection

Choose one or more modes before planning or coding:

- `Spec Mode`: use for vague backend requests, missing acceptance criteria, or greenfield service work that needs `REQ-WEB-*` contracts first.
- `Delivery Mode`: use for endpoints, app state, persistence, external integrations, and implementation sequencing.
- `Operations Mode`: use for configuration, secrets, tracing, CI, Docker, deployment, migrations, and rollout safety.
- `Security Mode`: use for validation, credentials, password handling, session and cookie flows, reset flows, and anti-enumeration behavior.
- `Resilience Mode`: use for retries, idempotency, timeout handling, transactions, external API failures, and background workers.
- `Review Mode`: use for backend reviews where the main question is production readiness, not generic Rust style.

Default combinations:

- new endpoint or feature: `Spec Mode` + `Delivery Mode`
- auth or session work: `Spec Mode` + `Security Mode`
- migration or deployment work: `Delivery Mode` + `Operations Mode`
- external integration or idempotent workflow: `Delivery Mode` + `Resilience Mode`
- backend review: `Review Mode` + one or more of `Security Mode`, `Operations Mode`, or `Resilience Mode`

## Workflow

1. Classify the backend task and choose the active mode set.
- Name the service slice first: endpoint, auth flow, external API client, migration, deployment, worker flow, or review.
- Decide whether the main risk is ambiguity, delivery sequencing, operational safety, security, or resilience.

2. Write executable requirements.
- Use `REQ-WEB-<NNN>.<REV>` identifiers.
- Express behavior as `WHEN...THEN...SHALL`.
- Reject vague claims such as "faster", "safer", or "more reliable" without thresholds or observable outcomes.

3. Design the service boundaries before coding.
- Separate request handler, application state, domain types, persistence boundary, and external integration boundary.
- Prefer typed inputs and typed domain objects over raw strings or loosely shaped payloads crossing multiple layers.
- Decide early when synchronous request handling should stay inline versus hand off to durable background processing.

4. Choose the verification mix deliberately.
- Pick from unit, integration, spawned-app tests, database-backed tests, HTTP mock tests, property tests, migration tests, and rollout checks.
- Prefer integration-first coverage for endpoint behavior, wiring, and persistence side-effects.

5. Apply backend delivery rules.
- Keep domain validation out of transport-only types.
- Keep secrets redacted and configuration layered by environment.
- Treat tracing, request correlation, health checks, and failure classification as design concerns, not polish.
- Treat idempotency, retries, and transaction scope as correctness problems when external systems are involved.

6. Run delivery and operational quality gates before handoff.
- Require format, lint, tests, and build success.
- Require requirement coverage and measurable evidence for non-functional claims.
- Require migration and rollout safety review when schema or deployment behavior changes.

## Output Contract

Return results in this order:

1. `Backend Work Mode`
2. `Executable Requirements`
3. `Service Design`
4. `Verification Matrix`
5. `Implementation Plan`
6. `Quality Gates`
7. `Open Questions`

Use this traceability shape:

| req_id | test_id | test_type | assertion | target |
| --- | --- | --- | --- | --- |

## Guardrails

- Do not use this skill as the default for generic Rust libraries, CLIs, parsers, protocol crates, or FFI boundaries. Prefer `rust-executable-specs-01` or `rust-coder-02` there.
- Do not treat transport validation as the whole validation story; keep domain invariants explicit.
- Do not recommend synchronous request handling for work that needs durable retries, deduplication, or long-running external calls unless the tradeoff is explicit.
- Do not recommend password or session flows without addressing storage, failure behavior, and async-safe verification boundaries.
- Do not treat deployment and migrations as postscript tasks when schema or background workflow behavior changes.

## Context Strategy

Load progressively:

1. Read [Reference map](./references/reference-map.md) first.
2. Read [Rust web backend playbook](./references/rust-web-backend-playbook.md) for service anatomy, handlers, domain boundaries, persistence, and external API integration choices.
3. Read [Rust backend testing and fixtures](./references/rust-backend-testing-and-fixtures.md) for endpoint verification, isolated test setup, and HTTP mocking.
4. Read [Rust backend runtime and ops](./references/rust-backend-runtime-and-ops.md) for configuration, secrets, tracing, CI, containerization, deployment, and migration safety.
5. Read [Rust backend security and resilience](./references/rust-backend-security-and-resilience.md) for auth, sessions, idempotency, retries, transactions, and background processing.

For large files, prefer heading search such as:

- `rg '^##|^###' references/rust-web-backend-playbook.md`
- `rg '^##|^###' references/rust-backend-testing-and-fixtures.md`
- `rg '^##|^###' references/rust-backend-runtime-and-ops.md`
- `rg '^##|^###' references/rust-backend-security-and-resilience.md`

## References

- [Reference map](./references/reference-map.md)
- [Rust web backend playbook](./references/rust-web-backend-playbook.md)
- [Rust backend testing and fixtures](./references/rust-backend-testing-and-fixtures.md)
- [Rust backend runtime and ops](./references/rust-backend-runtime-and-ops.md)
- [Rust backend security and resilience](./references/rust-backend-security-and-resilience.md)
