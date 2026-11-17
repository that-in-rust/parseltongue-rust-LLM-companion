# CLI phase — inform coding agents from the dataset

## The answer

**This phase is about using the dataset to inform coding agents via the CLI. It is NOT
about improving the dataset further.** The producer and its dataset backlog remain in
[phase-20261113](../phase-20261113/immediate-backlog.md).

The product is a read-only query surface over published production graphs: help an agent
find relevant code, inspect evidence and follow relationships without loading the whole
dataset into its context. Production never reads the frozen expected dataset as its input;
that dataset is the acceptance oracle for the producer.

**State:** phase entry document only. No new query command, query schema, executable part or
CLI acceptance run is delivered by this document. The initial target remains pinned itoa
1.0.18, not general multi-repository coverage.

## Scope and boundary

- Read published generations and their existing items, sites, relationships, ownership and
  evidence records. Derive navigation views from those facts without changing their meaning.
- Return bounded, deterministic results an agent can act on, including exact source
  locations and evidence where available. Reading a matching source range is navigation,
  not permission to run a second semantic extraction pipeline.
- Preserve unresolved reasons, source gaps, exclusions and unsupported capabilities. A
  missing fact is not a negative fact; an empty result is not proof that code is unused.
- Keep the producer's acceptance state visible. A query test passing does not make an
  unaccepted producer generation accepted.
- Do not change extraction, resolution, graph schema or frozen graph expectations here.
  No invented edges, name-matched substitutes or hidden enrichment to make a query succeed.
- No new database, embeddings, learned summaries, HTTP/MCP implementation or source-editing
  commands in this phase. Such surfaces are not prerequisites for the CLI.

Ideation does not require the exact final dataset. Start from agent questions and the
conceptual model; verify the actual schema and published rows before committing to an
executable contract. Classify ideas as supported by existing facts, unavailable because a
fact is absent, or deferred. An absent fact does not authorize producer work in this phase:
report the limitation and continue with supported queries. Dataset proposals stay in the
previous phase's backlog and are not prerequisites for CLI ideation.

## Read the previous phase before writing a part

Read in this order, for method rather than a copy of its implementation scope:

1. [PRD](../phase-20261113/PRD-itoa-full-item-graph-scope.md): scope, decisions and explicit
   exclusions before implementation.
2. [SPEC index](../phase-20261113/SPEC-itoa-full-item-graph/README.md): one target per part,
   stable requirement ids, expected results before code and unselected parts kept as stubs.
3. [Architecture](../phase-20261113/ARCH-itoa-full-item-graph-pipeline.md): existing ownership
   and persisted records; reuse them rather than build another mapper.
4. [Acceptance SOP](../phase-20261113/SOP-itoa-full-item-graph-acceptance.md): frozen inputs
   and executable, production readback, exact comparison, determinism and measurements.
5. [Log](../phase-20261113/LOG-itoa-full-item-graph-progress.md): one current-state summary,
   then append-only measured entries. Built, tested and accepted are different claims.

The old SOP decides producer acceptance; the CLI needs its own query acceptance procedure.
Do not reuse its S1–S4 labels as evidence that a CLI query has passed.

## Replicate the structure of thinking

Only this entry document exists in the new phase. Create the following documents as their
work is picked; these are planned responsibilities, not links to existing files:

| Document | Owns |
| --- | --- |
| PRD | Agent questions, CLI scope, non-goals and acceptance journeys. |
| ARCH | Query path from dispatch through pure query logic and store/source adapters. |
| SPEC index and numbered parts | Build order and executable requirements for one selected target at a time. |
| Query expectations | Frozen request/result cases, tied to the producer generation and source inputs. Test-only. |
| SOP | Reproducible CLI acceptance, exact output comparison, read-only checks and resource measurements. |
| LOG | The single current-state summary, followed by measured progress entries. |
| BACKLOG | Deferred CLI ideas and interface decisions, not dataset enrichment. |

Read the PRD, selected SPEC part and ARCH before implementing that target; use the SOP to
judge it and the LOG to report it. This entry document is a reading map, not a second log.

### One target through the loop

1. State the agent question. Inspect the existing production records with codebase-memory
   as a navigation aid, then confirm every claim in source and published rows.
2. Pick one target and write its SPEC part using `rust-executable-specs-01`: work mode,
   executable requirements, design by layer, verification matrix, implementation plan,
   quality gate results and open questions. Unselected parts remain stubs.
3. Freeze expected query results independently of the query implementation. Use stable
   requirement ids without colliding with the producer's ids; retain four-word naming.
4. Write a failing real-source test: build/publish through production from pinned itoa,
   query that published graph with the production binary, and assert stdout, stderr and
   exit status. Expected data stays in the test, never in production.
5. Make the smallest general change, then refactor. Keep core query logic pure; side effects
   stay in adapters behind existing trait boundaries. No itoa-specific production cases.
6. Compare results and evidence exactly, verify that the published graph is unchanged,
   check deterministic ordering, run applicable repository gates and measure time/memory.
7. Record commands, requirement/test ids, nonzero selected-test counts, codebase-memory
   queries, results, commit and next target in the log. Report unrun gates explicitly.

### Define the CLI boundary before coding it

Each selected part fixes operands and disambiguation, snapshot selection, versioned JSON
fields, ordering, scope, limits/pagination, evidence, and unsupported/unresolved states.
Separate semantic incompleteness from truncation. Specify stdout data, stderr diagnostics
and exit statuses, including malformed requests and operational failures. Text output is a
rendering of the same result, not a different interpretation of the graph.

## First navigation journey

Proposed first slice: inspect status/capabilities → find a symbol → inspect its source →
follow recorded references/callers/callees. For example, an agent investigating
`Buffer::format` should find it and follow its stored call sites, while seeing where the
available evidence stops instead of receiving guessed implementation bodies.

This is a starting proposal, not frozen command names or a claim that all those capabilities
exist. The 80-repository research supplies ideas, not 30 implementation commitments. Paths,
impact candidates and budgeted context bundles can remain stubs until selected. First make
one useful journey precise and test it against the actual published data.
