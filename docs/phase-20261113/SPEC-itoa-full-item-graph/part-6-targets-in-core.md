# Part 6 — targets in `core`

**State: BLOCKED on a decision. Stub: the work packet is written when this part is picked.**

## Requirement

No requirement is written yet. The decision comes first:

- **Option A — run with the sysroot source.** Targets such as `MaybeUninit`, `Default`,
  `Clone` get real identities. The one measured run with the sysroot did not finish name
  resolution in 2,400 s; that report is deleted and must be re-measured.
- **Option B — accept them as unresolved.** Every such site is stored with
  `external_source_missing`, and the expected graph lists them as unresolved.

REQ-RUST-038.0 already forbids an excluded-sysroot run from being an SOP PASS, so option B
needs that rule revised.

## Blocked on or waiting for

- The decision above ([PRD](../PRD-itoa-full-item-graph-scope.md), Open). It affects 119
  expected edges.
- Some itoa-to-itoa edges need a `core` return type: the signed `write` calls
  `Unsigned::fmt(self.unsigned_abs(), ..)`.
