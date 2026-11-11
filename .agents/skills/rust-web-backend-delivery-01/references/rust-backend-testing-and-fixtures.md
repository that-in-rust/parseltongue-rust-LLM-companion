# Rust Backend Testing And Fixtures

Use this file for endpoint verification, test harness structure, isolated infrastructure setup, and HTTP client mocking.

## 1. Integration-Test-First Mindset

For HTTP backends, start with tests that exercise the service from the outside:

- request in
- response out
- persistence side-effects checked where relevant
- remote integration calls mocked or isolated

This catches wiring errors, serialization mismatches, routing mistakes, and state setup issues earlier than unit tests alone.

Unit tests still matter for:
- domain constructors
- validation logic
- parsing edge cases
- small pure decision functions

## 2. Spawned-App Test Harness

Use a reusable harness that:
- boots the application with test configuration
- binds to a random port
- returns the base URL plus handles to shared dependencies
- lets tests call the service through its public HTTP interface

The harness should make the test read like a user journey, not a dependency graph.

## 3. Random Port And Isolated Database Setup

- Bind to a random free port to avoid test collisions.
- Provision database state per test or per suite with clear isolation semantics.
- Apply migrations automatically in test setup so tests see the real schema.
- Make cleanup strategy explicit: ephemeral database, isolated schema, or explicit teardown.

If tests share infrastructure, document the isolation boundary and why it is safe.

## 4. HTTP Mocking And Contract Tests

Use HTTP mocking when the backend integrates with external APIs:

- assert the outbound request shape
- assert auth headers or tokens are attached
- assert the client handles status codes, timeouts, and malformed bodies

Good mock tests state intent clearly:
- what request is expected
- what remote behavior is simulated
- what local behavior must result

Representative ecosystem example:
- `wiremock`

## 5. Maintainable Helper Structure

- Keep one helper module for startup and environment setup.
- Keep domain-specific fixtures close to the tests that use them.
- Prefer helpers that encode intent, not generic bags of convenience functions.
- When a helper hides too much logic, it stops helping.

Good helpers usually do one of:
- create valid test inputs
- boot the app
- query persistent state for assertions
- stub remote systems

## 6. Property-Based Tests For Domain Types

Property-based tests are especially useful for:
- domain constructors with invariants
- normalization rules
- round-trip parsing and formatting
- “accept all valid, reject all invalid” boundaries

Use them selectively. They are strongest at the domain edge, not as a substitute for end-to-end request tests.

## 7. Test Matrix Expectations

For substantial backend work, the verification mix should usually include:

| concern | preferred tests |
| --- | --- |
| endpoint behavior | integration tests |
| domain invariants | unit or property tests |
| database writes and reads | integration tests with real migrations |
| remote API interaction | HTTP mock tests |
| auth/session flows | integration tests plus selected unit tests |
| migration safety | schema/setup tests and rollout checks |
