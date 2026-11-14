# SOP — Using the Z.ai GLM Coding Plan to create labelled data (and when not to)

*2026-11-06. Companion to `SOP-zai-glm-coding-plan-second-opinion.md` (setup, launcher, quota endpoints). Numbers are from Z.ai's live monitor API and docs.z.ai on that date; re-check before a big run.*

## 1. Which labels an LLM may make at all
```text
   label type                                  who makes it                         zcode?
   L1–L4  items, edges, types, cfg, spans      rust-analyzer / rustdoc (exact, free)  NO. H-46: the toolchain is the teacher.
                                                                                      Frontier models omitted 6/14 signature
                                                                                      deps in experiments/public-interface-semver;
                                                                                      an omitted edge looks like independence.
   L6     module purpose, role, invariant,     an LLM                                 YES. Judgement labels; GLM-5.3 for quality,
          hot path, pattern cluster (H-55)                                             GLM-5.3-Flash for bulk.
   residual edges (dyn / cfg / proc-macro /    an LLM proposes, validator judges       YES as proposer only. RA or compile-check
          no-cargo) + explicit abstentions                                             stays the judge (H-32). Store as annotation
                                                                                      with creator = glm-5.3@date (H-51), not as fact.
   RQ1 agent A/B; Luna-style blind-labelling   an agent harness                       YES — this is what the plan is sold for.
   QA sample of any label set                  a second model                         YES, cheap: 50–200 items.
```
Rule: anything the compiler can decide, the compiler labels. The LLM labels meaning and proposals, each tagged `confidence` + `creator`, never merged into the exact graph.

## 2. Capacity on the Max plan (live: 28,000 credits / 5 h, 140,000 / week)
```text
   credit per token   GLM-5.3:  input 6.9   cached 1.7   output 24    (÷ 10,000)
                      Flash:    input 2.3   cached 0.56  output 8
   off-peak           ×0.5  — everything except Mon–Fri 14:00–18:00 SGT (11:30–15:30 IST); weekends all off-peak

   typical label call  2,000 in + 300 out
       GLM-5.3  peak 2.1 credits   off-peak 1.05
       Flash    peak 0.70          off-peak 0.35
       with a cached shared prefix (schema + instructions ≈ 1,500 of the 2,000 input tokens) ≈ −60 % on input

   per 5-hour window     ~27k items (5.3, off-peak)      ~80k items (Flash, off-peak)
   per week              ~130k items (5.3)               ~400k items (Flash)

   corpus sizes          top-100 Rust repos ≈ 15 M LOC ≈ ~50k modules ≈ ~1.5 M items
   → L6 module meaning for the whole corpus: one off-peak week on 5.3
   → one label per item, Flash: ~4 weeks; 5.3: ~12 weeks  (quota-bound, not latency-bound)
```
Concurrency is "dynamic, Max > Pro > Lite, higher off-peak"; expect 5–10 parallel calls, more at night.

## 3. The policy catch — read before any bulk run
From docs.z.ai/devpack/usage-policy and the FAQ (2026-11-06):
- "GLM Coding Plan may only be used within officially supported tools and products. Use in unsupported tools may result in restricted benefits."
- "Users subscribed to the Coding Plan can only make calls via the plan's quota in supported tools. API calls outside the plan are not available."
- Violations → "rate limiting, account freezing"; >3 violations → ban.

So a Python script calling `/api/coding/paas/v4` in a loop is outside the plan's terms. Driving the labeller through headless Claude Code (`claude -p`, a supported tool) is technically inside them but is a grey zone at ~10⁵ calls/week. Compliant paths for bulk:
1. **Pilot through `claude -p`** (≤ a few thousand items, off-peak) to measure credits/item and label quality.
2. **Full run on pay-as-you-go** GLM-5.3-Flash via the standard API (list prices on docs.z.ai/guides/overview/pricing — not extracted here; check), or
3. **Team Plan** with on-demand overage (billed at 10 % off list, per Z.ai), which also excludes data from training by default — relevant if proprietary code is ever labelled.

## 4. Procedure for the pilot (1,000 items)
```text
   0. CHECK quota (SOP-zai… §3). Run off-peak.
   1. SAMPLE 1,000 modules/items from 3 held-out crates (post-base-model-cutoff — contamination rule, 1793853341 D13).
   2. PROMPT FILE per item: fixed prefix (record schema, label definitions, 3 worked examples, "abstain if unsure")
      + the item's source window + the RA-derived exact context (signatures it touches). Keep the prefix byte-identical → cache hits.
   3. RUN   ZAI_API_KEY=… ~/bin/zcode -p --model glm-5.3-flash --allowedTools "" --output-format json < item.txt
      (no tools: the context is in the prompt; JSON output so the label parses). Batch 5–10 in parallel.
   4. VALIDATE every label: schema check; for residual-edge proposals, resolve against rust-analyzer — accepted / rejected / unverifiable.
   5. QA  50 items re-labelled by a frontier model + 50 by GLM-5.3 (not Flash); report agreement per label type.
   6. RECORD credits used (monitor API before/after), items/credit, agreement, abstention rate → decide 5.3 vs Flash and the billing path.
```
Kill criteria for "zcode as labeller": agreement with the frontier model on L6 labels < 80 %, or abstention < 5 % on items RA marks unresolved (means it is guessing).

## 5. What this does *not* change
H-46's dataset for the student model is still rust-analyzer output, not LLM labels. Z.ai labels feed only the L6/annotation layer and the evaluation harness.
