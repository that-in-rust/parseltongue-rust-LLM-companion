# Scoped agent instructions — HLD-current model audit

These instructions apply only inside `hld-current-model-audit/`. Root
`AGENTS.md` remains authoritative and unchanged.

## Scope and authority

- Audit **HLD current** from pinned source/configuration. Never audit intended
  HLD future rows or implement/integrate the production M8 builder here.
- Pass A must not read production HLD rows. Freeze its source-grounded expected
  artifact and digest before Pass B.
- Model findings are candidates, not facts or acceptance. PRIMARY and SOP S8/S9
  retain authority; source-confirmed false edges fail, unresolved disputes
  block. Never change a denominator from model output.
- Run and report one repository at a time. Do not pool the five corpus results.

## Skills and graph exploration

Use `@skills:rust-executable-specs-01` for spec-first boundaries and
`@skills:tdd-workflow-for-rust-stub-red-green-refactor` for any later code.
Use the installed code-graph executable at
`/home/ubuntu/code-graph-mcp/target/release/code-graph-mcp`; verify its current
version/help, reindex from a repository root, and check that `.code-graph/` is
gitignored before indexing. Use `health-check`, `search`, `map`, and `stats`
as navigation/coverage evidence. FTS/search results are not semantic ground
truth; cite pinned source bytes and lines for every candidate claim. Do not
install or update runtimes. Keep pinned corpus checkouts read-only; if an index
is needed, use a separate disposable source worktree.

## zcode isolation

Use `/home/ubuntu/bin/zcode` only when explicitly dispatched. Set `ZCODE_DIR`
to the isolated audit worktree or its audit-artifact directory; never use the
shared mapper worktree or resume/continue another session. Use noninteractive
`-p`, `--no-session-persistence`, `--permission-mode plan`, a narrow read-only
tool allowlist, and an external timeout/budget. Do not grant bypass/skip
permissions, shell/git mutation, or edits outside audit artifacts. Never print,
copy, log or commit credential values. Record the actual reported model ID or
`null`; do not infer it from a requested alias.

## Evidence and TDD journal

Record source commit/config/file digests, canonical compilation and occurrence
keys, line evidence, examined scopes, exclusions/unknowns, prompt/tool/model
identity, frozen expected digest, actual export digest, and sampling limits.
If HLD current is absent, mark Pass B BLOCKED/NOT STARTED. Do not claim an
exhaustive golden table without exhaustive independently checked coverage.

For subsequent validator/runner code, follow the exact journaled loop:

1. **STUB:** signatures and `todo!()`; run the scoped `cargo check`; append
   actual output and requirement ID.
2. **RED:** add four-word, REQ-tagged failing tests first; run them and journal
   the observed failure (never call compilation alone RED).
3. **GREEN:** minimum implementation; run the tests and journal exact output.
4. **REFACTOR:** simplify/types/dead-code cleanup or explicitly record no
   refactor needed; re-run changed checks and commit separately.

Documentation edits use structural checks (links, unique IDs, JSON Schema and
fixture shape). They are not Rust STUB/RED/GREEN and must not be journaled as
such. Keep every future validator test PENDING until it exists and runs.
