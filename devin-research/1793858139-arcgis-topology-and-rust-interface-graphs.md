# What a utility map can teach a Rust interface graph

**Decision, 2026-11-05.** Build one source-backed graph of Rust public contracts. Give a maintainer two ways to read it: from a contract to what it depends on, and from a changed type or trait back to the contracts that expose it. Borrow ArcGIS's distinction between a recorded feature, a validated connection, and a diagram. Do not add ArcGIS as a dependency or make a visualizer the product.

## What to keep from Devin

The [decision register](1793853341-rust-public-interface-dependency-graph-usefulness-form-and-open-decisions.md) improves the experiment: use pinned compiler-derived labels on buildable code, retain Codex as a separately identified teacher, and evaluate on repositories outside training. The [scale note](1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md) improves the interface: a small Rust signature view is more useful than a dump of every edge. The [Strata and Clarity note](1793853979-lessons-from-legacycode-strata-and-clarity-cli-for-rust-interface-graph.md) improves trust: validate proposed records and let the reader inspect both ends of a relationship.

Two corrections matter. A failing build is not a win by default: [rust-analyzer explicitly supports partial analysis of broken code](https://rust-analyzer.github.io/book/contributing/architecture.html). And a private helper may be hidden from a diagram, but its call must not be reassigned to the enclosing public function. If a factory returns a callback that later calls `flush`, creating the callback does not call `flush`.

## How ArcGIS constructs a graph

These are **three different Esri mechanisms**, not three mandatory steps in one ArcGIS product.

| Mechanism | What Esri actually does | What to copy for Rust |
|---|---|---|
| [ArcGIS Knowledge: load a table](https://pro.arcgis.com/en/pro-app/3.3/help/data/knowledge/load-data-into-a-knowledge-graph.htm) | Map fields to entity identifiers and properties. Define each relationship by its origin, type and destination. Specify merge rules; optional provenance records document property values. A link chart can be created after loading. | Treat parser captures, compiler records and model output as observations. Resolve endpoints before accepting a fact. Two mentions of `Error` are not the same definition merely because their text matches. Keep each supporting occurrence even when the fact is deduplicated. |
| [Utility Network: build and validate topology](https://pro.arcgis.com/en/pro-app/3.6/help/data/utility-network/about-network-topology.htm) | Network rules govern which features may connect. The topology stores connectivity separately from the editable features. Edits mark affected areas dirty. Esri's [trace example](https://pro.arcgis.com/en/pro-app/3.3/help/data/utility-network/configure-a-trace.htm) shows a deleted connection can still be followed before validation. | Index the inputs that justify an edge. Removing a re-export dirties dependents even in unchanged files. Until reconciliation finishes, identify the last committed capture; never present its path as verified against the new source. |
| [Network diagrams: apply a template](https://pro.arcgis.com/en/pro-app/3.2/help/data/network-diagrams/diagram-rules.htm) | A diagram starts from selected elements. Rules can trace outward, expand containers, or reduce junctions. Different templates yield [different views of the same network](https://pro.arcgis.com/en/pro-app/3.4/help/data/network-diagrams/generate-a-network-diagram.htm). | Render the same facts as a crate overview, a module's Rust signatures, or one expanded witness path. Label a collapsed path as derived and retain its intermediate IDs. Never write a shortcut from the picture back as a direct call. |

The limit of the analogy: pipes can connect at a junction. Rust names do not connect because their spans touch, and a possible callee is not observed runtime flow. A valid record needs more than two existing endpoint IDs: the evidence must support the **kind** of relationship asserted.

## Which graphs earn their place?

| View or index | What to put in it | Question it can answer |
|---|---|---|
| **Public contract graph: ship first** | Canonical items, public paths pointing to definitions, `SIGNATURE_USES` with parameter/return/bound/field position, and trait implementations. Scope facts to the crate, features and target examined. | “Which public functions return this storage error, including through an alias?” Reverse the same signature edges; do not search every body that mentions the type. |
| **Direct call graph: keep separate** | Direct caller-to-callee assertions with call-site spans. Keep possible dispatch targets and unresolved sites distinguishable from direct calls. | “What directly calls this method?” Show private helpers on demand. Do not count a trait implementation or a callback reference as a call. |
| **Ownership and overview: derive** | Workspace, crate and module ownership; project contracts onto these levels for orientation. Retain source witnesses for aggregate edges. | “Which crate should I inspect next, and which exact contract caused that connection?” A Cargo dependency alone is not an answer to the second question. |
| **Evidence dependencies: internal only** | Reverse index from imports, definitions, configuration and source snapshots to the assertions they support. | “What must be checked again after this edit?” A comment can move an occurrence without changing the contract. A changed feature can invalidate an edge without changing that file's bytes. |

Keep the [PRD's three HTTP reads](../docs/research004/prd-v20260710.md): fuzzy search, direct callees and direct callers. The CLI export can carry the richer contract graph. Defer shortest paths, centrality and whole-program control/data-flow graphs: none is needed to distinguish a public return type from a private implementation detail.

## Archives worth using

| Primary source | Use it for | Do not use it for |
|---|---|---|
| [Esri's archived ArcGIS 9 guide](https://content.esri.com/support/documentation/ao_/1003getting_started_with_arcgis.pdf), “Coverage topology” | The historical arc-node model records the endpoints that establish connectivity. It is a clear precedent for storing a relationship independently of its drawing. | Choosing a GIS file format or implying source coordinates establish identity. |
| [Präzi paper](https://arxiv.org/abs/2101.09563) and [authors' Zenodo dataset](https://zenodo.org/records/4478981) | Use the release and build-status CSVs to select historical Rust crates; compare item-level call results with archived static call graphs for matched versions. | Ground truth for public signatures, macro provenance or runtime dispatch. These are different relations. Inspect the CSVs before fetching the 40.2 GB call-graph corpus. |
| [Stack graphs: Name resolution at scale](https://arxiv.org/abs/2211.01224) | Precedent for per-file graph fragments linked by valid name-resolution paths. Use a deterministic linker as a baseline for the student. | Assuming per-file text settles every Rust trait or macro question. |
| [GraphCodeBERT](https://arxiv.org/abs/2009.08366) | Precedent for training with edge prediction and alignment between source and graph structure. Test whether those objectives help a small student identify signature roles. | Evidence that a CPU model fits under 100 MB or constructs accurate Rust public graphs. The paper evaluated other tasks. |
| [Ferrante et al., program dependence graphs](https://www.csa.iisc.ac.in/~raghavan/CleanedPav2011/ferrante-pdg-1987.pdf) | A precise boundary for the term *dependence*: control and data dependencies between operations are a different analysis. | Claiming that a return type identifies which input value reached the return statement. |

## The experiment that would change my mind

Use a held-out workspace where a facade re-exports `backend::Fault` as `Fault`, a public `read` returns it, and a private logger mentions the same type. Ask which public contracts expose the error and require the return-position span plus the re-export witness. The logger must not appear.

1. **Insert an ordinary comment.** The source hash and offsets may change; the public identity and reverse answer must not.
2. **Break an unrelated function body.** Measure whether each analyzer retains the independently supported contract, rather than rewarding it merely for returning something.
3. **Remove the only re-export but leave the return spelling unchanged.** The old exposure must disappear from the next committed capture. If the target is no longer defensible, preserve the unresolved occurrence rather than rebinding it to a same-named type. Compare the incremental result with a clean index of that exact checkout.

Run the shipped Tree-sitter pipeline, a deterministic import-aware resolver, build-aware Rust tooling where available, and the local student on the same inputs. Report per-relation precision/recall, confident false edges, appropriate abstentions, agent answer correctness and tokens, peak CPU inference RAM, and whole-tool memory separately. The preferred sub-100 MB worker and roughly 5% export/source token ratio are **targets**, not findings. Keep the model only where it improves useful answers at an acceptable cost; the graph remains useful whichever producer wins.
