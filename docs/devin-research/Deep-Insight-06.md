# Deep Insight 06 — The multi-level code map

*A standalone definition of the object Parseltongue builds and the agent reads: one set of dependency records over a Rust codebase, viewed at three nested levels — folder, file, public interface — where every node has a kind and metadata, every dependency is a typed edge with provenance, and an edge at an upper level is nothing more than the sum of the edges beneath it. Answer first, then the three things that make it hold, then the definitions, then a worked example.*

---

## The one-line answer

**A code map is not a call graph with folders drawn around it. It is one append-only set of leaf records — items and the typed edges between them — from which the file view and the folder view are computed by a fixed roll-up rule, so that any question can be asked at the cheapest level that answers it and expanded downward, edge by edge, to the exact span that justifies it.**

```text
                 L0  crate     ── what packages, features, cfg am I in?            cargo metadata
                 L1a folder    ── which parts of the tree touch each other?        roll-up of L1b
                 L1b file      ── if I change this file, which files notice?       roll-up of L2/L3 + mod/use/include
                 L2  item face ── what does this file promise to the rest?         scanner + tiny model
                 L3  item body ── who actually calls, constructs, reads this?      tiny model, lazily

      expand ▼ at any edge: folder edge → the file edges that made it → the item edges → the spans
```

---

## What most people get wrong

Most people think "multi-level" means several graphs: a module graph here, a file graph there, a call graph underneath, each built by its own tool and each a little inconsistent with the others. Then the agent is handed three pictures that disagree and told to reason.

The real object is one graph. Folders and files do not have dependencies of their own; they inherit them from the items inside them. A file-level edge *is* the set of item-level edges between the two files, counted. A folder-level edge *is* the set of file-level edges, counted. Once that is the definition — not a convenience — the three views can never disagree, a change at the leaf updates every level in the same transaction, and "why does `client/` depend on `policy/`?" has a mechanical answer: expand.

The second thing people get wrong is to treat the two visible levels as a sketch and the call graph as the truth. For an agent under a token budget the reverse is closer: the folder and file views are where it lives, the public face is what it reasons about, and bodies are a detail it opens for one region at a time. The map is designed so the cheap levels are complete and honest on their own, and the expensive level is optional.

---

## Three things that make it hold

```text
   1. ONE RECORD SET, THREE VIEWS       the roll-up rule is the definition of the upper levels, not an approximation
   2. TYPED NODES, TYPED EDGES          every node has a kind and metadata; every dependency has a kind, a span, a creator,
                                        a confidence, a status — absence is written down as `abstained`
   3. THREE WRITERS, ONE CONTRACT       a deterministic scanner builds most of it; a tiny model raises the ceiling; only an
                                        LLM or a human writes the future — and the map says which did what
```

### 1. One record set, three views

```text
 an edge  A → B  exists at level n   ⇔   at least one edge  a → b  exists at level n+1 with a ⊂ A and b ⊂ B
 weight(A → B)      = number of contributing lower edges
 kinds(A → B)       = the set of lower kinds that contributed          e.g. {uses, field_type, may_call}
 cfg(A → B)         = union of the lower cfgs
 confidence(A → B)  = [min, max] of the lower confidences — a range, never an average
 status(A → B)      = extracted if any contributor is extracted; proposed if every contributor is proposed
```

Three consequences. `impact` at folder level is not a separate analysis; it is the same records summed. Nothing is stored twice: the folder and file views can be recomputed from the leaves at any time, which is what lets the backend be swapped and the schema be versioned without migrating anyone's map. And an upper-level edge carries the *range* of confidences beneath it, so a folder edge that rests on one exact import and one 0.6 guess shows `[0.60, 1.00]`, not `0.80`.

### 2. Typed nodes, typed edges

Every record, at every level, carries the same base: an identity, a kind, a cfg, a creator (scanner at a revision, model at a revision, feed publisher, LLM, human), a confidence, a status (`extracted · abstained · inferred · proposed · realized · stale`), a first- and last-seen snapshot, and machine tags that are annotations rather than identity. The levels add their own kinds and fields (§4). Comments, imports and globals are fields on records, not nodes: a doc comment is the item's `doc`, an import is the evidence span on a `uses` edge, a static is an item of kind `static`.

Where the extractor sees a site but cannot name its target, it writes an edge to a placeholder target with status `abstained` and the span. The roll-up then shows "file A → 3 unresolved sites" rather than nothing. That one rule is what makes the cheap levels honest: an empty neighbourhood means *no dependencies*, never *we did not look*.

### 3. Three writers, one contract

A deterministic scanner over Rust tokens produces the crate level, the folder level, the file level and most of the public face — every `mod`, `use`, `include!`, cfg gate and every item whose signature parses — with `abstained` where regular grammar stops: glob imports, macro-generated items, re-export chains, names that must be bound among candidates. A tiny model takes those sites and either resolves them with a confidence or abstains again; it alone produces the body level. An LLM or a human writes proposed nodes and edges — the structure the code should have — and nothing else may. The map records which writer produced each record, so the agent can ask for the view without the model, or without the proposals, as a filter rather than a rebuild.

The scanner alone is therefore a complete, honest map at L0–L2. The model does not make the map exist; it raises its ceiling.

---

## 4. The definitions

### 4.1 Levels

```text
 level  node          what it is                                  edge "A → B" means                              derived by
 ─────  ────────────  ──────────────────────────────────────────  ──────────────────────────────────────────────  ───────────────────────
 L0     Crate         a Cargo package@version                     A declares a dependency on B (features, cfg)   cargo metadata, no build
 L1a    Folder        a directory under the crate root            some file in A depends on some file in B       roll-up of L1b
 L1b    File          one .rs file, content-digested              A names B — mod, use, include!, cfg — or an    scanner; model for
                                                                  item in A depends on an item in B               ambiguous imports
 L2     Item          one public or crate-visible item            A's signature or face refers to B              scanner (plain cases) ·
                                                                                                                  model (binding)
 L3     Item body     the same items, their bodies                A's body calls / constructs / reads B          model, lazily, per region
```

### 4.2 Node kinds and metadata

```text
 BASE, all levels   id · kind · display name · crate · cfg · creator · confidence · status · first_seen / last_seen · tags

 CRATE   kind ∈ { lib, bin, proc-macro, workspace-member, external, std }
         name · version · edition · features enabled · source (path | registry | git) · licence

 FOLDER  kind ∈ { crate-root, src, module-dir, tests, examples, benches, bin, build, generated }
         path · file count · item count by kind (rolled up) · lines · owning module path

 FILE    kind ∈ { lib-root, bin-root, module, inline-only, test, bench, example, build-script, included, generated }
         path · content digest · module path defined · lines · tokens · pub item count · file-top cfg · first //! line ·
         has_unsafe · defines_macros · extraction coverage (% of sites resolved)

 ITEM    kind ∈ { fn, struct, enum, union, trait, type-alias, const, static, macro, impl, assoc-fn, assoc-const,
                  assoc-type, field, variant, extern-fn, extern-static, inline-mod }
         path (crate::module::Name) · visibility declared (pub | pub(crate) | pub(super) | pub(in …) | private) ·
         visibility effective (after re-exports, when known) · signature text, normalised · generics and bounds ·
         flags {async, unsafe, const, extern-abi} · attributes of interest (derives, cfg, deprecated, must_use, repr) ·
         doc (first line + full) · span (file, line range) · body digest · size · impl: trait + self type ·
         field: type · variant: payload shape
```

The membership test for the public face is one question: *could this line change what another file is allowed to write?*

### 4.3 Edge kinds and metadata

```text
 BASE, all edges   id = hash(src, dst, kind, cfg) · kind · src · dst · cfg · evidence span(s) · creator · confidence · status ·
                   weight (1 for leaves, count for roll-ups) · via (collapsed private path when projected) · first/last seen

 L0  crate          depends_on (normal | dev | build) · enables_feature
 L1  file, folder   declares_mod · uses · includes · cfg_gates · tests · depends_on (roll-up: weight, kinds, cfg, range)
 L2  item face      signature_uses · parameter · returns · field_type · implements · trait_bound · associated_type ·
                    reexports · derives · macro_generates
 L3  item body      calls · may_call (dyn / generic dispatch, candidates ranked) · constructs · reads_field · writes_field ·
                    awaits · panics_via · unsafe_uses · influences
 any level          proposed_* — any kind above with status proposed, authored by an LLM or a human; the tool never creates one
```

The vocabulary is versioned by schema release. A kind is added by a release, never by a model.

---

## 5. Who produces what

```text
                               scanner (deterministic, CPU)              tiny model (CPU or GPU)                  LLM / human
 L0   crates                   cargo metadata                             —                                        —
 L1a  folders                  roll-up                                    —                                        —
 L1b  files                    all plain mod / use / include / cfg        glob imports, imports through macros      —
                                                                          or cfg'd re-export chains → inferred
 L2   item nodes               every item with a parseable signature:     macro-generated items; effective
                               doc, cfg, attributes, span, digest         visibility through re-export chains
 L2   item edges               when the type path is qualified or         when a name must be bound among
                               resolves through a plain use                candidates; trait_bound, associated_type
 L3   bodies                   call-site spans — where a call happens     targets: receiver typing, method
                                                                          resolution, may_call ranking, abstention
 proposed                      —                                          —                                        all of it
 annotations                   —                                          labeller (opinion layer)                 opinions, sightings
```

---

## 6. A worked example

A small crate, `retry`:

```text
 retry/
   src/lib.rs            pub mod policy;  pub mod client;  pub use client::Client;
   src/policy/mod.rs     pub trait RetryPolicy { fn next_delay(&self, attempt: u32) -> Option<Duration>; }
                         mod fixed;  pub use fixed::FixedBackoff;
   src/policy/fixed.rs   pub struct FixedBackoff { pub delay: Duration }
                         impl RetryPolicy for FixedBackoff { … }
   src/client/mod.rs     use crate::policy::RetryPolicy;
                         pub struct Client { policy: Box<dyn RetryPolicy> }
                         impl Client { pub fn send(&self, req: Request) -> Result<Response, Error> { … self.policy.next_delay(n) … } }
```

```text
 L2  ITEM EDGES — the leaf records
   client::Client#struct            ──field_type──►   policy::RetryPolicy#trait          (Box<dyn RetryPolicy>)       scanner  1.00
   client::Client::send#assoc-fn    ──parameter──►    Request#struct                                                   scanner  1.00
   client::Client::send#assoc-fn    ──returns──►      Response#struct · Error#enum                                     scanner  1.00
   policy::fixed::<Impl#1>#impl     ──implements──►   policy::RetryPolicy#trait  (for FixedBackoff)                    scanner  1.00
   policy/mod.rs                    ──reexports──►    policy::fixed::FixedBackoff#struct  (as policy::FixedBackoff)     scanner  1.00
   lib.rs                           ──reexports──►    client::Client#struct               (as retry::Client)            scanner  1.00
   client::Client::send#assoc-fn    ──may_call──►     policy::RetryPolicy::next_delay#assoc-fn  (dyn dispatch)         model    0.93  [L3]
                                      candidates: FixedBackoff::next_delay — sole implementor in the workspace

 L1b FILE EDGES — roll-up plus direct
   src/lib.rs          ──declares_mod──►  src/policy/mod.rs · src/client/mod.rs                           weight 2
   src/lib.rs          ──reexports──►     src/client/mod.rs                                               weight 1
   src/client/mod.rs   ──uses──►          src/policy/mod.rs             (evidence: use crate::policy::RetryPolicy)
   src/client/mod.rs   ──depends_on──►    src/policy/mod.rs             weight 2  kinds {field_type, may_call}  conf [0.93, 1.00]
   src/policy/mod.rs   ──declares_mod──►  src/policy/fixed.rs
   src/policy/fixed.rs ──depends_on──►    src/policy/mod.rs             weight 1  kinds {implements}

 L1a FOLDER EDGES — roll-up
   src/client/  ──depends_on──►  src/policy/      weight 3  kinds {uses, field_type, may_call}  conf [0.93, 1.00]

 L0  retry  ──depends_on──►  (no external crates)
```

What the agent sees, in order, under a budget: the folder view — two folders, one edge, ~60 tokens. Expanding it, the file view — five files, six edges, ~250 tokens. Expanding `client/mod.rs → policy/mod.rs`, the item edges that made it, with the one model-produced `may_call` marked by its confidence and its candidate list. `impact RetryPolicy` reads upward from the trait: one implementor, one field type, one dyn call site — and the coverage banner reports that nothing in this crate was abstained.

A proposed edge sits in the same picture. If the agent plans `pub struct ExponentialBackoff` implementing `RetryPolicy`, `propose` writes `policy::ExponentialBackoff#struct ──implements (proposed, author: agent-LLM)──► RetryPolicy#trait`; it rolls up as a dashed sibling of `fixed.rs` into `src/policy/`, and it becomes `realized` the first time the extractor finds the real impl.

---

## 7. What each level is for

```text
 level   question it answers fastest                      typical size, 300k-line workspace      default token cost
 L0      what crates, features, cfg am I in?               30–80 crates                           ~200
 L1a     which parts of the tree touch each other?         100–400 folders · 1–3k edges           ~500 — top-k by weight
 L1b     if I change this file, which files can notice?    1–5k files · 10–50k edges              ~1k — one neighbourhood
 L2      what does this file or item promise?              10–50k pub items · 50–200k edges       ~3k — one anchor, radius 1
 L3      who actually calls, constructs, reads this?       100k–1M sites                          ≤ 20k — one deep dive
```

---

## 8. Non-goals — so they stay non-goals

- **No separate graphs per level.** Folders and files are views; if a backend cannot compute them from the leaves, it is the wrong backend.
- **No comment, import or global as a node.** They are fields and evidence.
- **No model-invented kinds.** The vocabulary is a schema release.
- **No silent gaps.** An unresolved site is an `abstained` edge with a span; a folder with no edges has no dependencies.
- **No tool-authored future.** Proposed records come from an LLM or a human, or they do not exist.
- **No bodies by default.** L3 is opened per region, on request.

---

## The one risk this does not remove

**Identity under motion.** The roll-up rule is only as stable as the identities beneath it. Rename a module and every item path changes; move a file and every file id changes; the leaves are re-extracted correctly, but the history of the edges above them breaks unless identity survives the move. The map's answer is a second fingerprint — signature shape for items, body skeleton for functions, content digest for files — used to carry an id across a rename and record it as a transition rather than a death and a birth. If that fingerprinting fails on real refactors, the levels still compute, but the first- and last-seen history that makes the map more than a snapshot does not. It is the first engineering problem to solve, and it is testable on any repository's own git log.
