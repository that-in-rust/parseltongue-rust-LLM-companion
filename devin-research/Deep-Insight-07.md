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

---

## 9. The staged pipeline, model-first

*A rewrite of the build order, after one correction: a tiny model sits at every point where a judgment is made, and the deterministic tool only moves bytes. The question that forced it: "what scanner can run a Linux-sized codebase into the table?" — answered below, and then set aside, because once spans exist no later stage needs to read the codebase at all.*

### 9.1 The estimate: can a 100 MB model span a Linux-sized tree in under fifteen minutes?

```text
 100 MB encoder reading ALL 400 M tokens  (2e8 FLOP per token → 8e16 FLOPs)
   8 GB MacBook Air        ~30 h
   Mac mini M4 Pro         ~7 h
   desktop GPU             ~30 min
   datacenter GPU          ~7 min        ← the only hardware that reads everything in fifteen minutes

 fifteen minutes of Air GPU ≈ 6 M tokens through a 100 MB model ≈ 1.5 % of the corpus
```

So the 100 MB model must not read everything. Two ways to stay inside fifteen minutes:

- **Tagger first, heavy model second.** A 2–5 MB boundary tagger reads every token (Air ~20 min, Mac mini ~5 min, desktop GPU ~1 min) and the 100 MB model reads only the grey spans — low confidence, failed parse, macro soup — a few percent of the text. Air ~25 min total; Mac mini ~8 min; desktop GPU ~3 min.
- **Shard.** Two laptops or one desktop GPU split the tree by crate; the table merges by digest.

Verdict: under fifteen minutes for a kernel-sized tree on a Mac mini or a desktop GPU; about half an hour on an Air; for any real Rust workspace — three million lines or fewer — under fifteen minutes everywhere, including the Air.

### 9.2 What the deterministic tool is allowed to do

The tool is a one-pass tokenizer and a row writer. It runs at ripgrep speed — hundreds of megabytes per second per core, so 1.49 GB in about five minutes on four cores — and writes batched rows at hundreds of thousands per second. Scale is not its problem. Judgment is: a regular-grammar pass gets generics, `where` clauses, nested `impl<T: Trait<U>>`, attribute soup and half-written items subtly wrong, and on a Linux-sized tree "subtly wrong two percent" is forty thousand bad rows that poison every edge joined to them.

So the tool decides nothing. It does exactly five things:

```text
 tokenize       bytes → tokens, with positions
 digest         file and span content hashes (identity)
 copy           span text and doc comment into their columns
 insert         batched rows; FTS index build
 join           the SQL that turns names into candidate edges
```

Every boundary, kind, visibility, field, binding and label is a model's output.

### 9.3 The stages

Each stage reads rows from the table and writes columns or rows back. Nothing is held in memory between stages; every stage is resumable and separately scorable against the compiler on held-out repositories.

```text
 stage 0   CONTEXT     cargo metadata (crates, features, cfg) · file digests                 tool            seconds to minutes
 stage 1   SPANS       boundaries · kind · name · visibility, over all text                   2–5 MB tagger   Air ~20 min / mini ~5 min (kernel-size)
 stage 2   TEXT        copy span text + doc into columns · build FTS                           tool            minutes
 stage 3   CLARIFY     grey spans only — fix boundary, kind, visibility; split or merge         100 MB encoder  reads ~2–5 % of text
 stage 4   NODES       fields from span text: signature parts, generics and bounds, flags       30–100 MB       reads span text only —
                       {async, unsafe, const}, derives, cfg, impl trait + self type,           encoder with    40–80 M tokens for kernel-size,
                       field types, variant shapes                                             field heads     not 400 M
 stage 5   EDGES       SQL joins on names → exact or candidate edges; binder on the ambiguous   tool + 100 MB   binder reads only ambiguous rows,
                       rows with candidates in its input → inferred or abstained                binder          candidates attached
 stage 6   LABELS      one-line role or headline per node (opinion layer, optional)             labeller        on demand
```

### 9.4 Why stage 4 belongs to the model — the correction

The first draft gave node fields to the scanner "with a small model on the odd ones". That was wrong twice over. Wrong on quality: fields are exactly where regular grammar fails. And wrong on cost, in the other direction: once spans exist, stage 4 reads span text only — signatures and docs, ten to twenty percent of the corpus — so a model can read all of it cheaply:

```text
 stage 4 input, kernel-sized tree    ≈ 40–80 M tokens
   100 MB field-tagger    Air ~5 h · Mac mini ~1 h · desktop GPU ~5 min
    30 MB field-tagger    Air ~1.5 h · Mac mini ~20 min · desktop GPU ~2 min
     5 MB field-tagger    Air ~10–15 min · Mac mini ~3 min
 any real Rust workspace (≤ 3 M lines): minutes on an Air with any of them
```

The 100 MB "dumbed-down heavy" model is the right one here: it reads `impl<T> Trait<U> for Foo<T> where T: Send` as one unit and tags generics, bounds, self type, trait, flags, derives and cfg correctly — on broken code too — where a regex is hopeless. On an Air it runs in the background; on a Mac mini it finishes over lunch.

### 9.5 Three ways to build it

**A — Relay of specialists.** One tiny model per stage: tagger (2–5 MB) → clarifier (100 MB) → field-tagger (30 MB) → binder (100 MB). Each trained on its own compiler-labelled target; each consumes rows whose status says it is needed.

- Kernel-size: the table is usable after stage 2 (~25 min on an Air, ~10 min on a mini); fully bound L1/L2 after ~2–3 h on an Air, ~45 min on a mini.
- For: each model small, cheap to train, cheap to swap; failures localise to one stage; the table shows progress per stage.
- Against: four artefacts to version; a wrong boundary poisons its node and edges downstream; four passes over the rows.
- Best when measurability matters and one stage must be replaceable without retraining the others.

**B — One model, many heads.** A single 100 MB encoder with heads for boundary, kind, visibility, fields and binding. Pass one reads each grey file once and fills spans and fields together; pass two reruns only the binding head on ambiguous rows with candidates from the join — candidates cannot exist before the table does, so there are always two passes.

- Kernel-size: pass one must still be gated to grey text by the tagger, otherwise it is the thirty-hour read; gated, ~2 h on an Air, ~40 min on a mini.
- For: one artefact, one training run, one leaderboard row; boundary and kind inform each other through a shared representation.
- Against: harder to train well; a regression in one head ships with all of them.
- Best when distributing one model file matters more than isolating failures.

**C — Table-first, models as repair workers.** The tagger and the tool produce a complete, honest table with `abstained` rows wherever judgment is pending — kernel-size in ~25 min on an Air, ~10 min on a mini, seconds for a crate. Models never block it. They consume three queues in the background or on demand: the grey-span queue (clarifier), the field queue (field-tagger), the ambiguous-name queue (binder). Each consumed row flips `abstained` to `extracted` or `inferred`, and the coverage banner climbs while the agent is already working.

- Kernel-size to fully bound: the same ~2–3 h on an Air — but the first useful table arrives at minute twenty-five, not hour three.
- For: fastest to first value; models optional and replaceable; queues are independent, so there is no stage-ordering problem; a model that fails to clear the bar simply leaves its queue unconsumed.
- Against: needs a small job runner; the tool and the models must agree on one schema.
- Best when the product must be usable before any model has earned its place.

### 9.6 Where this lands

C is the frame; A's specialists are its queue workers; B is how the workers are packaged once there is a leaderboard worth winning. The fifteen-minute target for a Linux-sized tree is met by C on a Mac mini for the *table*; the fully bound map follows behind it, row by row, visibly. On any real Rust workspace the whole pipeline — table and bindings — finishes within the fifteen minutes on an Air.

The principle, stated once: **the tiny model makes every decision; the tool only moves bytes; the table is the contract between them.**
