# Reference Map

Use this file first to route the request before loading heavier backend references.

## 1) Choose the Work Mode

| mode | use when | read next |
| --- | --- | --- |
| `Spec Mode` | Backend feature request is vague, acceptance criteria are missing, or success is not measurable yet | `rust-web-backend-playbook.md`, then `rust-backend-testing-and-fixtures.md` |
| `Delivery Mode` | Work needs endpoint shape, app state, persistence seams, integration boundaries, and implementation sequencing | `rust-web-backend-playbook.md`, then `rust-backend-testing-and-fixtures.md` |
| `Operations Mode` | Main risk is configuration, secrets, tracing, CI, Docker, deployment, migrations, or rollout safety | `rust-backend-runtime-and-ops.md` |
| `Security Mode` | Main risk is credential handling, sessions, cookies, password reset, anti-enumeration, or validation boundaries | `rust-backend-security-and-resilience.md` |
| `Resilience Mode` | Main risk is retries, idempotency, timeout behavior, external API failures, background workers, or transaction scope | `rust-backend-security-and-resilience.md`, then `rust-backend-runtime-and-ops.md` |
| `Review Mode` | The task is a backend review or hardening pass and the main question is production readiness | `rust-backend-security-and-resilience.md`, `rust-backend-runtime-and-ops.md`, then `rust-web-backend-playbook.md` |

## 2) Choose by Task Type

### New Endpoint or Feature

Read in this order:
1. `rust-web-backend-playbook.md`
2. `rust-backend-testing-and-fixtures.md`
3. `rust-backend-security-and-resilience.md` if auth, retries, or external systems are involved

### Auth, Sessions, or Password Flows

Read in this order:
1. `rust-backend-security-and-resilience.md`
   - `1. Typed Credentials And Verification Boundaries`
   - `2. Session, Cookie, And Reset Flows`
   - `3. Failure Shaping And Anti-Enumeration`
2. `rust-backend-testing-and-fixtures.md`
3. `rust-web-backend-playbook.md`

### Database Change, Migration, or Rollout

Read in this order:
1. `rust-backend-runtime-and-ops.md`
   - `5. Deployment And Migration Sequencing`
   - `6. Zero-Downtime Schema Evolution`
2. `rust-backend-testing-and-fixtures.md`
3. `rust-web-backend-playbook.md`

### External API Client or Email Workflow

Read in this order:
1. `rust-web-backend-playbook.md`
   - `5. External API Client Boundary`
2. `rust-backend-testing-and-fixtures.md`
   - `4. HTTP Mocking And Contract Tests`
3. `rust-backend-security-and-resilience.md`
   - `4. Idempotency, Retries, And Transaction Scope`

### Idempotent or Background Workflow

Read in this order:
1. `rust-backend-security-and-resilience.md`
   - `4. Idempotency, Retries, And Transaction Scope`
   - `5. When To Move Work To Background Processing`
2. `rust-backend-runtime-and-ops.md`
3. `rust-backend-testing-and-fixtures.md`

### Backend Review or Hardening Pass

Read in this order:
1. `rust-backend-security-and-resilience.md`
2. `rust-backend-runtime-and-ops.md`
3. `rust-web-backend-playbook.md`
4. `rust-backend-testing-and-fixtures.md` only if verification coverage is unclear

## 3) Default Read Order for Large Backend Tasks

1. Read this file.
2. Read `rust-web-backend-playbook.md`.
3. Read only the relevant sections of `rust-backend-testing-and-fixtures.md`.
4. Read `rust-backend-runtime-and-ops.md` or `rust-backend-security-and-resilience.md` depending on the risk surface.

## 4) Heading Search

- `rg '^##|^###' references/rust-web-backend-playbook.md`
- `rg '^##|^###' references/rust-backend-testing-and-fixtures.md`
- `rg '^##|^###' references/rust-backend-runtime-and-ops.md`
- `rg '^##|^###' references/rust-backend-security-and-resilience.md`
