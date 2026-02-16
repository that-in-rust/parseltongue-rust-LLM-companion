# PRD: Parseltongue v2.0.0

**Date**: 2026-02-16

---

## Requirement #1

**NO BACKWARD COMPATIBILITY NEEDED.**

v2.0.0 is a clean break. New ingestion pipeline, new storage, new server. It does not need to produce the same JSON shapes, use the same endpoint names, accept the same CLI flags, or interoperate with v1.x in any way.

**NO OLD CODE WILL BE DELETED.**

v1.x (pt01, pt08, parseltongue-core, CozoDB) stays in the repo, compiles, and works. v2.0.0 is additive. New crates, new modules, new binaries. The old code is not touched, not deprecated, not removed. Both can coexist in the same workspace.


## Requirement #2

