# Deferred Abstractions and Decision Backlog

Updated: 2026-11-14 17:59 IST.

## Active goal: the detailed full map

Phase 1 builds and verifies `full-item-dependency-graph` for pinned itoa, together
with its source, signature, body, resolution and expansion evidence. It does not
need an interface projection or any higher-level abstraction to qualify.

The map retains public and private items, direct dependency facts, exact canonical
identities and source locations, ownership, visibility/reachability metadata,
resolved targets and explicit unresolved reasons. Original and expanded source
must be accounted for. Existing graph-shape decisions remain in force: impls are
records, function-body descendants are evidence, and source containment is distinct
from semantic ownership. “All details” does not silently promote every expression,
field or impl into a new graph-node category.

**Backlog means deferred, not decided, deleted, implemented or passed.** Existing
HLD code, safeguards and test evidence are retained. No source-scope or publication
gate may be weakened to make the full map look complete.

## Abstractions are queries, not separate semantic pipelines

The mega map is one logical graph: canonical node and edge tables plus linked
source, signature, body, resolution and expansion evidence. It does not require
one denormalized physical table. Rust derives semantic facts; Turso durably stores
them; DuckDB derives views over a compatible frozen generation.

Interface selection, dependency slices, caller/callee traversal, private-path
collapse and file/folder summaries are queries over those retained facts. A later
materialized view is a cache of a derivation, not another source of truth or a
second source-analysis pipeline. Queries cannot recover missing type resolution,
macro expansion or call targets. New facts require upstream semantic work; intent
maps require separately authored intent, never a deduction of intent from code.

## A. Higher-level products removed from active Phase 1 scope

| ID | Deferred work | What stays active instead | Resume condition |
| --- | --- | --- | --- |
| BL-01 | Public/interface node selection and `hld-current-interface-nodes` | Every in-scope public/private item; visibility is metadata, not an inclusion filter | Full-map acceptance, then explicit approval to resume abstractions |
| BL-02 | Collapsed HLD dependency edges and private-path compression | Direct, uncollapsed full-graph edges and their call/type/read sites | Same; current six-relation rule remains the retained baseline |
| BL-03 | HLD multi-hop evidence-path construction | Exact source evidence for each direct fact; expansion and ownership provenance | Same; never discard direct proof to save future projection work |
| BL-04 | File-level and folder-level dependency roll-ups | Source paths, file areas, source containers and cross-file direct edges | Same; file identities/containment are not deferred |
| BL-05 | Crate/folder/file/interface summaries and abstraction-level views | Lossless semantic facts available from the frozen full-map generation | Same |
| BL-06 | Producing, publishing and comparing all five HLD outputs | Durable full-map rows, frozen readback, atomic generation safety and exact full-map comparison | Same; full-map publication is not permission to mark HLD complete |
| BL-07 | HLD-oriented query/visualization convenience, intent/future maps and HLD delta views | A usable, versioned full-map read/export contract with identities and evidence | Separate user request after the detailed-map contract is verified |
| BL-08 | Extending or retiring `original-top-level-containment` as a product mode | Keep its bounded regression as historical diagnostic coverage; build missing full-map producers | A later compatibility/CLI decision; do not silently broaden or delete it |

## B. Open decisions that do not block the current full-map target

| ID | Original decision | Deferred portion | Reactivation trigger |
| --- | --- | --- | --- |
| BD-01 | OD-6 | Whether private items used by several files also appear in an interface view | Interface projection is requested; private items already belong in the full map |
| BD-02 | OD-9 | Whether collapse should be restricted to calls/reads rather than the retained six dependency relations | Collapse is requested; no collapse is performed in the full map |
| BD-03 | OD-2 | Which file/folder bucket receives a generated item in a derived roll-up | Roll-ups are requested; expansion identity and actual source/container provenance remain active |
| BD-04 | OD-3 | Whether agent-facing convenience queries use raw tables or a denormalized copy with display text | A consumer API is requested; canonical frozen full-map readback/export remains active |
| BD-05 | OD-5 | Dual `imports`/`reexports` representation for `pub use`, absent from pinned itoa | Before accepting a source scope with `pub use`; no general reexport-completeness claim before then |
| BD-06 | AD-5, projection portion | Independent expected HLD rows, collapsed paths and roll-up counters | Higher-level outputs are requested; the independent full-map reference is required now |

These IDs are the queue for deferred decisions, not a second set of graph rules.
Their alternatives have been removed from the active open-points lists. Existing
HLD executable contracts remain in the SPEC as compatibility/regression obligations
when those paths are invoked; keeping a contract is not scheduling its implementation now.

Retained context for BD-02: the current terminal-hop rule can expose a private
helper's type uses at a public caller. Restricting collapse to calls/reads would
change that result. Neither alternative changes the direct full-map facts.

## C. Follow-on work outside this bounded benchmark

| ID | Work | Why deferred / trigger |
| --- | --- | --- |
| BF-01 | Five-repository qualification, serde first | After pinned-itoa full-map acceptance; preserve applicable corpus thresholds and do not report an all-five milestone from Phase 1 |
| BF-02 | General proc-macro/async/dynamic-dispatch shapes not exercised by pinned itoa | Before accepting a corpus that needs them; absence from itoa is not passing coverage |
| BF-03 | Performance tuning, incremental remapping and convenience CLI verbs | After correctness or a measured blocker; time, whole-process-tree RSS and disk measurements remain required now |
| BF-04 | Formal M8 HLD release qualification | Only when HLD work is explicitly resumed; this backlog does not lower those gates |
| BF-05 | Broad SOP/AGENTS process cleanup beyond this scope correction | The previously proposed six process changes still need approval; do not mix them into the full-map implementation |

## D. Do not move these into the backlog

These affect the detailed map itself. Keep them in the active requirements and
resolve each before claiming completeness for its affected source facts.

| Active item | Why it matters now |
| --- | --- |
| Canonical node/edge schema and frozen full-map export | The detailed map must be inspectable and comparable without HLD tables |
| OD-4: fields/variants as nodes versus retained evidence | itoa's private fields and field type/array-length facts still matter; lack of public fields is not an exemption |
| OD-7 remainder: file-module/area relationship | Affects full-graph containment; the decided one-source-container rule stays in force |
| OD-8: non-function type uses and reads | Needed for associated constants, field types and array-length dependencies |
| OD-2: generated identity, source/container provenance | Required for exact macro-generated nodes/edges; only derived roll-up attribution is deferred |
| AD-1: real core identities and core-dependent typing | Sysroot exclusion is diagnostic, not complete selected-source semantic coverage |
| AD-2: body scope/name resolution | Required for lexical binding, shadowing and exact call targets |
| AD-3: effective visibility computation | Preserve a semantic fact without using it to discard private nodes; unknowns stay explicit |
| AD-4: supported inference and method selection | Determines whether dependency targets are correct; unsupported cases need reasons |
| AD-5: independent full-map reference | Prevents defining correctness by whatever production happens to emit |
| Source/expansion completeness, unresolved and inactive inventories | Prevents silent omissions and fabricated targets |
| Generation consistency, cancellation/failure safety and determinism | Full-map correctness, not optional presentation work |

## Next active checkpoint

1. Persist original item/member nodes and their decided source-container edges.
2. Resolve `<Buffer as Default>::default -> Buffer::new` as one direct full-graph
   call with exact source evidence and frozen readback.
3. Progress to generic `Buffer::format -> private::Sealed::write`, remaining
   dependency kinds and macro/core-dependent source coverage.
4. Compare the complete scoped full map and its detailed evidence against an
   independently reviewed reference. Do not add interface filtering, collapse or
   roll-up acceptance to that checkpoint.

## Maintenance and references

- Add new deferred work here with a stable ID, reason and reactivation trigger.
- Never defer a correctness blocker merely because it is difficult.
- Record scope changes newest-first in the [Phase 1 evolution log](phase1-log-p1.md).
- Keep earlier log entries as history; former subparts 4.6 and 4.7 are deferred,
  while 4.8 concerns full-map publication safety now.
- [PRD: product intent and decisions](PRD-itoa-full-item-graph-scope.md)
- [SPEC: executable contracts and active open points](SPEC-phase-20261113-executable-requirements.md)
- [Architecture: pipeline and table boundaries](ARCH-itoa-full-item-graph-pipeline.md)
- [SOP: source-backed verification](SOP-itoa-full-item-graph-acceptance.md)

This is a scope/decision record, not implementation or test evidence.
