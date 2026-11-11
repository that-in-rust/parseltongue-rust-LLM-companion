# Rust Backend Security And Resilience

Use this file for credentials, session flows, anti-enumeration behavior, idempotency, retries, transaction scope, and background processing choices.

## 1. Typed Credentials And Verification Boundaries

- Keep raw credentials at the edge for as little time as possible.
- Convert to typed wrappers or validated domain objects before deeper application logic.
- Store password hashes, never raw passwords.
- Make the verification boundary explicit so expensive or blocking work does not accidentally happen on the wrong executor path.

Representative ecosystem example:
- `argon2`

## 2. Session, Cookie, And Reset Flows

- Treat login, logout, reset, and session refresh as distinct flows with distinct failure handling.
- Use secure cookie or session defaults appropriate to the deployment model.
- Make success and failure redirects explicit.
- Keep reset tokens, session identifiers, and password change workflows typed and scoped to their exact use.

Representative ecosystem example:
- `redis` for server-side session storage when shared session state is needed

## 3. Failure Shaping And Anti-Enumeration

- Do not leak whether a username, email address, or token exists unless the product explicitly requires it.
- Use generic failure responses for authentication boundaries where enumeration is a risk.
- Separate operator-facing diagnostics from user-facing error messages.
- Treat timing, wording, and redirect behavior as part of the observable contract.

## 4. Idempotency, Retries, And Transaction Scope

When a request can be repeated because of user retries, client retries, or network ambiguity:

- define the idempotency key boundary
- define the stored replay state
- define what counts as “same request”
- define concurrency behavior if two identical requests arrive at once

Transaction scope should answer:
- what must succeed together
- what may happen after commit
- what may be retried independently

Do not mix “save the durable intent” and “call remote side-effects” without a deliberate failure strategy.

## 5. Timeout And Failure-Class Handling

Classify failures at remote boundaries:
- client error
- server error
- timeout
- network transport failure
- malformed response

Each class may deserve a different policy:
- fail fast
- retry
- record for replay
- surface to operator
- move to async recovery

## 6. When To Move Work To Background Processing

Move work out of the request path when:
- remote systems are slow or unreliable
- retries must outlive the request
- duplicate suppression matters
- partial completion is unacceptable
- operator visibility matters more than immediate client completion

When using background workers, define:
- durable queue or table boundary
- worker polling or event model
- retry policy
- deduplication behavior
- operator-visible status transitions

## 7. Review Checklist

For security or resilience reviews, ask:

- Are credentials typed and handled at the right boundary?
- Are hashes, session keys, and tokens kept out of logs?
- Are auth failures shaped to avoid useful attacker feedback?
- Is the transaction boundary explicit?
- Is idempotency defined for repeatable writes?
- Would a timeout or crash leave the workflow in an ambiguous state?
- Should part of this workflow become asynchronous?
