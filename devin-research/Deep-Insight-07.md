# Deep Insight 07 — Span-first: the table before the graph

*A standalone thesis. The map's leaf record is a span of source text — one public item, with its kind, name, path, visibility, file, line range, text and digest — written into one embedded database table by a tiny tagger before any dependency is computed. Edges are derived afterwards: by SQL joins on names, by a model on the ambiguous rows, by an agent reading the span. This inverts the usual order — analyse first, store second — and that inversion is the point. Vertical sections throughout.*

---

## The one-line answer

**Index the public spans first, in minutes, on a CPU; derive the dependency graph from them later, lazily, and only as far as a question requires. The span table is the floor product — exact at the span level, searchable, incrementally updatable — and every later layer (edges, model, proposals) is a view or a lazy write on top of it.**

```text
 the usual order        text ──model──► edges ──► store ──► views
 span-first             text ──tiny tagger──► SPANS ──► one table ──► edges derived at query time or in the background
```

---

## What most people get wrong

Most people build the graph first and the index second, because "dependency graph" is the thing they want. So the expensive, uncertain step — resolving what refers to what — stands between the user and any result at all. If resolution is slow, nothing is available; if it is wrong, everything downstream is wrong.

The real object the agent needs first is not the graph. It is the set of public items, as text, with exact boundaries, searchable by name and by content. An agent that can read the five public spans relevant to its question is already better off than one with a grep result; it does not need the edges between them to begin. Edges make it cheaper and let it ask "what breaks" — they are the optimisation, not the foundation.

The second thing people get wrong is to think the model must do the whole job. Segmenting source into public spans is a syntax-shaped task a 2–5 MB tagger does at tens of thousands of tokens a second; it needs no judgment, no cross-file knowledge, no compiler. Judgment is spent later, per ambiguous row, on demand.

---

## Three things that make it hold

### 1. Public spans are small, exact and cheap

Public items are about one percent of a Rust codebase's lines. For a kernel-sized tree that is roughly three million spans and 100–300 MB of text — one database file. For a 42k-line crate, a few thousand rows. The tagger's output is exact where it matters: kind, name, boundaries, visibility, the signature text itself. Identity is the span's content digest, so an unchanged item is the same row across commits, a moved item is the same digest at a new path, and an incremental update is a row replace. On an 8 GB laptop a kernel-sized tree segments in about twenty minutes; a real Rust workspace in seconds to minutes.

### 2. Edges fall out of joins, and the leftover is exactly the model's job

A `use crate::policy::RetryPolicy` in one span and a span named `RetryPolicy` under `policy/` make a candidate edge in one SQL join. When the name is unique in scope the edge is exact. When it is not, the join returns the candidate set — which is precisely the input a binder model needs: a closed list to choose from, or to decline. So the span table does not merely postpone the model; it narrows its task to the rows the join could not settle, and it hands them over with their candidates attached.

### 3. The agent can read spans directly

Full-text search over span text — signatures and doc comments — makes `find` grounded: the agent receives the real signature, not a paraphrase. For a large class of questions, a language model reading five public spans is a better analyst than any edge we could compute. Span-first makes that the default path and the graph the acceleration. It also makes the thesis cheaper to test: a pilot can run with spans only as its middle arm, before a single edge exists.

---

## 4. The schema

```text
 span
   id            = content digest of the span text (identity; stable across moves)
   file_id       the file's path @ digest
   kind          fn · struct · enum · union · trait · type-alias · const · static · macro · impl · assoc-fn · assoc-const ·
                 assoc-type · field · variant · extern-fn · extern-static · inline-mod
   name          the item's name
   path          crate::module::Name  (what the item is called from outside)
   visibility    pub · pub(crate) · pub(super) · pub(in …) · private   (private rows are LIGHT: no text — see §6)
   start_line · end_line
   text          the signature span (public items); for L3 later, the body span, lazily
   doc           the doc comment, first line and full
   cfg           the cfg / feature gate, if any
   creator       tagger@rev · scanner@rev · model@rev · feed@publisher
   confidence    1.0 for scanner boundaries; the tagger's score otherwise
   snapshot      the snapshot in which this row was produced

 name
   span_id · token          sub-tokens of name and path (auth, provider, retry …) — for joins and for BM25

 edge   (materialised lazily — never required for the table to be useful)
   src · dst · kind · status (extracted · inferred · abstained · proposed · realized · stale) · confidence · evidence_span · snapshot
   produced by:  SQL join on name/path (exact when unique, inferred otherwise) · model on ambiguous rows · propose from an LLM or human
```

One embedded SQLite-compatible database — SQLite itself or its Rust reimplementation — with recursive CTEs for the k-hop views and built-in full-text search over `text` and `doc`. One file. No service.

---

## 5. What the analysis becomes

```text
 find        BM25 over span text + trie/trigram over names → real spans with real signatures, ranked, budgeted
 who-uses    SQL: join `use` paths and type names against span names → exact or candidate edges, in milliseconds
 bind        model, only on the rows the join left ambiguous, with the candidates in its input; writes inferred or abstained
 impact      recursive CTE over the edge table, bounded by depth and token budget; roll-ups by file and folder are sums
 read        the agent is handed the spans themselves when that is cheaper than computing anything
 propose     an LLM or a human inserts proposed spans and edges; they realize when the tagger later finds the real span
```

The order of operations in a session: table in minutes → `find` works immediately → edges accrue in the background and on demand → the model is spent where joins stopped → the agent reads spans for everything else.

---

## 6. What it does not change

- **The method-call problem is untouched.** `self.policy.next_delay(n)` names no type; resolving it still needs the receiver typed, still needs a model, still happens lazily. Span-first makes the public levels nearly free; it does not make the body level free.
- **Private glue is still needed for anchors.** A hit inside a private function must be walked up to the public item that encloses it. Public-only text would lose that path. So private items are kept as *light* rows — kind, name, file, range, no text — enough to walk, nothing to store.
- **Name joins produce false positives.** Two items named `Sender` in scope are not one edge. A join result that is not unique is stored as `inferred` with a confidence from uniqueness, never as `extracted`.
- **Macros remain opaque.** Items produced by proc-macros are not in the text; their spans are the macro invocation, their kind `macro_generates`, their status `abstained` until a model or the feed says otherwise.

---

## 7. Non-goals

- No graph before the table. If the table is not useful on its own, the design has failed.
- No span text for private items by default. Boundaries and names only.
- No model in the segmentation path. The tagger is a tagger; judgment is spent on rows, later.
- No edge stored as `extracted` from a name join that was not unique.
- No second store. Edges, proposals and annotations live in the same file as the spans.

---

## The one risk this does not remove

**Boundaries.** Everything rests on the tagger cutting spans at the right places — where a `pub fn` starts and ends, that an `impl` block's associated items are its children, that a cfg'd duplicate is two rows and not one. A tagger that is 99 % right on boundaries silently merges or splits one item in a hundred, and every edge joined to that row inherits the error. Boundaries are also the one thing a deterministic scanner gets essentially right for well-formed Rust, so the risk is concentrated where the product most wants to work: broken and half-written code. The mitigation is to score boundaries separately from everything else, on broken inputs specifically, and to let the scanner's boundaries override the tagger's wherever the code parses.
