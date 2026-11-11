# Rust Backend Runtime And Ops

Use this file for configuration, secrets, telemetry, CI, containerization, deployment sequencing, and rollout safety.

## 1. Hierarchical Configuration

- Layer configuration by environment instead of hard-coding runtime choices.
- Keep a clear split between checked-in defaults and injected secrets.
- Parse configuration once at startup into typed settings.
- Prefer explicit validation failures at boot over surprising runtime misconfiguration.

Common layers:
- base defaults
- environment-specific overrides
- environment variables or secret injection

## 2. Secrets Handling

- Keep secrets out of logs, panic messages, and debug dumps.
- Pass secrets around in redaction-friendly wrappers where practical.
- Avoid cloning or formatting secrets unnecessarily.
- Treat API tokens, database passwords, session keys, and HMAC keys as separate secret classes with separate rotation concerns.

Representative ecosystem example:
- `secrecy`

## 3. Structured Tracing And Correlation

- Prefer structured tracing over ad-hoc string logs.
- Attach request or operation identifiers so related events can be correlated.
- Record high-value fields once and propagate them through the workflow.
- Make log shape useful for both local debugging and operator triage.

Representative ecosystem examples:
- `tracing`
- `tracing-subscriber`

## 4. Startup, Readiness, Liveness

- Fail fast on invalid configuration, missing migrations, or unavailable core dependencies.
- Distinguish “process started” from “service ready to handle traffic”.
- Expose health surfaces that reflect the real dependency picture when appropriate.
- If the service launches background workers, decide whether readiness requires those workers to be healthy.

## 5. CI Expectations

At minimum for backend delivery:
- formatting
- linting
- tests
- build
- dependency and vulnerability hygiene if the repo already enforces it

When database or migration work is involved:
- run migration-aware tests in CI
- verify schema setup matches application expectations

## 6. Docker And Image Shape

- Use container builds that keep the runtime image small and predictable.
- Keep build caching strategy explicit for Rust dependency compilation.
- Copy only what the runtime image needs.
- Make configuration and secret injection environment-driven, not image-baked.

Representative ecosystem example:
- `docker`

## 7. Deployment And Migration Sequencing

When deployment and schema changes interact, plan the order explicitly:

1. what schema change lands first
2. what code version can run against both old and new states
3. when backfill happens
4. when constraints become strict
5. what rollback still works after each step

Do not ship schema changes that require impossible ordering.

## 8. Zero-Downtime Schema Evolution

Default pattern for risky mandatory-field changes:

1. add the new column or table in a backward-compatible way
2. deploy code that can read and write both shapes
3. backfill existing data
4. tighten constraints only after the new behavior is universal

This is the safe default whenever multiple app instances or staggered rollouts are possible.
