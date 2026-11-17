# Markdown review coverage

**The review inventory accounts for all 393 repository Markdown paths and 34 supplemental
research paths frozen for this CLI ideation pass.** The inventory and curated API backlog
are being published in separate, small documentation commits; this page records the
completed coverage reconciliation, not CLI implementation or producer acceptance.

## Frozen inputs and reading treatment

Repository inputs are pinned to
[`fac71702f`](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/commit/fac71702f179c88990914ccd098e0bd52e7e7131).
Sixteen isolated, read-only reviewers returned reports and original ledgers. Repository
editing and synthesis remain in the coordinating session.

| Coverage measure | Reconciled result |
| --- | ---: |
| Original Markdown paths | 427 |
| Canonical nonempty documents | 398 |
| Byte-identical alias paths | 26 |
| Empty files | 2 |
| Symlink, not followed | 1 |
| Assigned ranges with matching ledger records | 430 of 430 |
| Missing, extra or duplicate range records | 0 |
| Range-evidence citations inside assigned bounds | 2,331 of 2,331 |
| API-idea citations with valid source paths and bounds | 469 of 469 |
| Batch-level ideas / document-range ideas retained for curation | 154 / 675 |

All 430 range records report `full_text`. These are the reviewers' reading statuses;
automated correspondence and hash checks are not an independent rereading of the corpus.
Large documents have contiguous assigned ranges. Exact duplicate paths remain listed but
share the review of their canonical bytes. Empty files and the symlink receive explicit
dispositions rather than being silently dropped.

## Reconciliation and later changes

One workflow summary shortened the hash in a batch-13 range ID. The original ledger has
the exact expected ID, path, lines 1–127 and `full_text` status for the community review.
Only the normalized summary was corrected; original reports and ledgers remain unchanged.

Three producer documents changed after the freeze: the scope PRD, acceptance SOP and
immediate backlog. Their changed hunks were reviewed separately at
[`0f4cc3367`](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/commit/0f4cc3367a5f7a352307baf900b6ec050ad38dff).
They record declarations-scope progress, an outstanding acceptance run and the unresolved
`cast` proof question. They do not authorize dataset work in this CLI phase. These are
updated versions of three existing inventory paths, not three additional documents.

This work validates document coverage and citation bounds. It does not validate published
production rows, historical performance claims, current CLI behavior or producer acceptance.
