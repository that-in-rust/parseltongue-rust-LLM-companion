# Thesis: Which ≤100 MB model can be edited to act like rust-analyzer? What weight-space programming can and cannot do for a tiny Rust graph extractor

*Written 2026-11-05. Branch `research/tiny-rust-code-mapper`. Prompted by "Sparks of Weight-Space Programming" (Takehi, Teoh, Zhang, Khattab; MIT CSAIL; 2 Nov 2026) [1]. Builds on `1793864191-…` (rust-analyzer as substrate, measured), `1793873685-…` (its limits), `1793874432-…` (interface graphs with one small program) and `experiments/tiny-reexport/` (an exact alias lookup compiled into a 2,460-byte ONNX network).*

---

## 0. The thesis in six sentences

1. Weight-space programming, as demonstrated in [1], does two things: it **reads** what a network computes from its weights, and it **writes** small, exactly specified programs into small networks or **removes** a single failure direction from a larger one. It demonstrably does **not** add an ability a model lacks ([1], Gemma 3 270M: "editing weights could remove an obstacle that hid an ability, but it could not add an ability the model lacked").
2. rust-analyzer is a 200k-LOC program with fixed-point algorithms (import resolution, macro expansion, trait solving) over a 1–3 GB mutable database (measured in `1793864191-…`). That is not an "ability hidden in a model" to be surfaced by an edit; it is a program plus a working set, and the working set is the repository's facts, which cannot live in 100 MB of weights.
3. Therefore no base model can be *edited into* rust-analyzer. The question is well-posed only in a weaker form: which ≤100 MB model can be **trained** (gradient descent, rust-analyzer as teacher) into a per-file Rust graph extractor, and then **edited** to remove its systematic failures and to **host compiled exact circuits** for the bounded sub-operations we know how to compile (alias chasing, path joining, visibility reach), Qwenputer-style [2].
4. Under that form the candidate is a model with ≤ ~200 M parameters at 4-bit or ≤ ~100 M at 8-bit. Of today's public checkpoints the realistic pick is **SmolLM2-135M** [3]; the alternative is a **purpose-built 20–60 M parameter Rust model trained from scratch**, which leaves headroom for grafted circuits and is the regime where [1] showed weights are readable (7.2 M-param character transformers, Exp. 3).
5. The expected end state is **"Cassini for Rust, with abstention"** [4]: a 75–85 % edge-F1 prior that is good enough as a fallback and a ranker, with its failure modes surgically removed; not a replacement for the deterministic ladder, which stays underneath and remains the source of truth and of training labels.
6. The payoff of weight-space programming for this project is therefore not the extractor itself but three cheaper things: **auditing** what the tiny extractor learned (reading heads/directions), **zero-data repair** of its failure directions (looping JSON, hallucinated edges, cfg-blindness), and **grafting** exact lookups into spare width so they need not be learned.

---

## 1. What the post actually shows (and what it does not) [1]

| Exp. | Network | Operation | Result | Lesson for us |
|---|---|---|---|---|
| 1 | MLP 500→32→32→5, 17,253 weights, PPO | *read*: which game? | 4/4 correct, from weights as text | small nets are legible |
| 2 | MLP 160→64→64→144, 23,824 weights, DDPM | *read*: which shape? | correct, found in output layer geometry | reads structure, not forward pass |
| 3 | char-transformer 4×6 heads, width 384, **7.16 M weights**, middle blocks only | *read*: which language, which heads copy? | 3/3 language; every induction head found, none mis-named | a ~7 M-param transformer's circuits can be audited from weights; rediscovered the Elhage et al. method |
| 4 | Kinetix actor, 171,664 weights | *fix*: car stuck at ramp | 0 % → 100 % success, 3.5 % of weights edited, one shot | adding a small side-path for a missing behaviour works at this scale |
| 5 | 600→32→32→6, from zeros | *write*: MinAtar policies | beats PPO@100M steps in 3/3 games, no rollouts | exact small programs can be written into small MLPs |
| 6 | 243→512×6→82, **1.48 M weights**, from zeros | *write*: 9×9 Go | with 50 forward passes beats equal-size PPO trained on 20 M games; written as a 336-line Python program layer by layer | "compile a program into weights" scales to ~1.5 M params with a sandbox |
| 7 | Gemma 3 270M ×2 | *read*: which is sycophantic? | **failed from weights alone** (4/4); succeeded only with attention traces | at 270 M params, pure weight reading breaks down |
| 8 | Qwen3.5-0.8B (873 M params [5]) | *fix*: math | MATH-500 7.9 → 48.1 %, GSM8K 13.8 → 67.1 %, IFEval 21.0 → 62.3 %, by subtracting one direction from 64 output columns in MLPs of layers 15 and 19 (131,072 weights, 0.016 %) | removing a looping direction unlocks what the model already knew |
| extra | Gemma 3 270M (no looping) | *improve*: math | **no gain** (6.8 → ≤ 7.7 %, within noise); failures were "ordinary reasoning errors … with no single component behind them" | **editing cannot add an ability** |

Three structural facts follow:

- **Reading and writing are bounded by the size of the thing being read or written.** Everything that worked from weights alone is ≤ ~7 M parameters (reading) or ≤ ~1.5 M parameters (writing). At 270 M, reading needed activations.
- **Fixing is subtraction, not addition.** The 0.8 B result is a single-direction ablation; the 270 M null result is the control. An ability that is distributed ("ordinary reasoning errors") is not editable in.
- **Writing is compilation.** Exp. 5 and 6 are a frontier model acting as a compiler from an explicit program (ball landing position → paddle; liberties → groups in atari → move score) into weights. That is the same operation as `compile_model.py` in `experiments/tiny-reexport/`, performed by an LLM instead of a Python script, at ~1.5 M parameters instead of 73 constants.

## 2. Why "edit a model into rust-analyzer" is the wrong frame

What rust-analyzer *is* (from `1793864191-…`, `crates/hir-def/src/nameres.rs`, `architecture.md`):

```text
  component                size / cost                       nature
  ───────────────────────  ────────────────────────────────  ─────────────────────────────────────
  parser (rowan)           25k LOC                           deterministic program
  ItemTree + DefMap        47k LOC; fixed-point: collect →    deterministic program with an
  (name resolution,        resolve imports → expand macros → unbounded loop until no progress
   macro expansion)        repeat
  hir-ty (inference,       63k LOC; trait solver             deterministic program, search
   method resolution)
  salsa database           1–3 GB on a 42k-LOC workspace,    the FACTS of this repository + std +
                           ~80 % of it std and deps          every dependency, mutable per edit
```

A model edit changes the *program* (rules) but not the *facts*; in-weight memory is fixed at training time and a `git pull` must not require retraining (the point already made in `1793874432-…`). Where could the facts go? In activations (context) — at most tens of thousands of tokens, i.e. one or a few files, which is exactly the regime where the measured ceiling is ~40 % of call edges (`1793864191-…` §2.3) because receiver types live in other files and 44 % of references leave the workspace entirely. Or in an external store — at which point the model is a client of the store, and the "rust-analyzer" is the store plus its deterministic update rules, not the model.

Qwenputer [2] is the limiting case and it proves the point: a RISC-V CPU compiled into 11 layers of a 4 B model (9.36 B parameters including the added layers [6]), with mutable memory in the DeltaNet state. It can *execute* a program exactly; the program, inputs and memory still have to be supplied, and the whole thing is 40–90× over a 100 MB budget. Our own `tiny-reexport` is the 2,460-byte version of the same idea with the same lesson: exact, 8–10× the memory and 10–18× the time of the native code (Linux repro: native 3.5 MB / 3.2 ms, model 27.5 MB / 28.7–55.9 ms), facts table supplied from outside.

So: **"like rust-analyzer" has to be re-read as "a tiny model that, per file, produces what rust-analyzer would say about that file, with calibrated abstention where the answer is not in the file".** That is Cassini's architecture [4] (Qwen3-4B, LoRA rank 1, six JSON fields per file, 77.6 % F1 on Python/JS/TS, deterministic resolver downstream) at 1/40th the size and with a 100 %-exact teacher.

## 3. The budget, and the candidates

100 MB of process RSS (the target stated in `1793874432-…`) must hold weights, runtime, tokenizer and working buffers. ONNX Runtime alone cost ~24 MB in `tiny-reexport`. Call it **≤ 70–80 MB of weights**: ≤ ~70 M parameters at int8, ≤ ~150 M at int4 (embeddings often stay at 8-bit, which hurts small-vocabulary-heavy models the most).

| Model | Params (verified on HF 2026-11-05) | ≈ weights at int4 / int8 | Fits? | Assessment |
|---|---|---|---|---|
| **HuggingFaceTB/SmolLM2-135M** [3] | 134.5 M (Llama arch, bf16) | ~75 MB / ~135 MB | **yes at int4** | Apache-2.0, trained on code-including mix, modern tokenizer. Best public starting point. Risk: int4 at 135 M params degrades; must measure extractor F1 at int4, not bf16. |
| bigcode/tiny_starcoder_py | 164.1 M (GPTBigCode, f32) | ~90 MB / ~165 MB | borderline (int4) | Python-only pretraining; Rust tokenization and syntax would be learned from scratch anyway. |
| google/gemma-3-270m | 270 M | ~140 MB / ~270 MB | **no** | Also the model [1] *could not* improve by editing. |
| Qwen/Qwen2.5-Coder-0.5B | 494.0 M | ~260 MB | no (2.5–3×) | Best small code model; would be the right choice at a 300 MB budget. |
| Qwen/Qwen3-0.6B | 751.6 M | ~400 MB | no | — |
| Qwen/Qwen3.5-0.8B [5] | 873.4 M | ~450 MB | no (4–5×) | The model edited in [1] Exp. 8. |
| Qwen3-4B (Cassini base [4]) / Qwen3.5-4B-Qwenputer [6] | 4 B / 9.36 B | 2–5 GB | no (20–50×) | Cassini and Qwenputer are both far outside the budget. |
| **purpose-built Rust extractor, 20–60 M params** | 20–60 M | 20–60 MB at int8 | **yes, with headroom** | Trained from scratch on Rust source → rust-analyzer labels. Same scale as [1] Exp. 3 (7.2 M) × 3–8, i.e. still in the regime where weights can be read and circuits grafted. Needs its own tokenizer (Rust-aware BPE or bytes). |

Verdict: **SmolLM2-135M at int4 is the pragmatic student; a from-scratch 30–60 M model is the research student.** Train both on the same data; keep whichever reaches higher edge-F1 per MB.

## 4. The plan: train, then read, then cut, then graft

```text
  step  what                                              who does it                       evidence it works
  ────  ────────────────────────────────────────────────  ────────────────────────────────  ─────────────────────────────
  1     TRAIN the extractor: file (+ optional compact      gradient descent (SFT/LoRA),      Cassini [4]: 77.6 % F1 at 4 B
        cross-file index) → L2–L4 records as JSON,        rust-analyzer as teacher on       on Py/JS/TS; our teacher is
        with "unresolved" as a first-class answer         crates that load                  100 % exact (rustdoc cross-check)
  2     READ the weights: which heads track scope /       frontier model + Python sandbox   [1] Exp. 3: induction heads
        imports / receiver bindings; where does            over the weights (no forward       found from weights at 7 M params;
        abstention live; what is memorised vs computed    passes needed at this size)       breaks down by 270 M (Exp. 7)
  3     CUT failure directions: looping/repeated JSON,     frontier model, ≤ 50 forward       [1] Exp. 8: 0.016 % of weights,
        hallucinated edge instead of null, ignoring       passes on held-out crates, no     MATH-500 ×6; Exp. 4: 3.5 % of
        #[cfg], over-confidence on method calls           training                           weights, 0 → 100 %
  4     GRAFT exact circuits into spare MLP width:         compiler script (like              tiny-reexport: 648/648 agree,
        alias chase (done), module-path join, visibility  compile_model.py) or the           2,460 bytes; Qwenputer [2]: whole
        reach, prelude table, cfg-predicate evaluation    frontier model as compiler          CPU in 11 layers
  5     MEASURE per level, per callee class, at int4,      scripts from 1793864191-… +        kill/promote threshold from
        peak RSS < 100 MB including runtime; compare to   `measure.py`                       1793864191-… §8: ≥ 95 % L4
        tree-sitter + heuristics and to rust-analyzer                                        CALLS F1 at ≤ 2× RA cost
```

What each step is *allowed* to achieve, going by [1]: step 1 provides the ability (nothing else can); step 2 is cheap and reliable at this size; step 3 may be large if the extractor has a dominant failure mode (small models loop) and will be ~0 if its errors are diffuse; step 4 is exact and small but each circuit must have its facts delivered in the prompt/context (the bindings table), which is the real design constraint.

## 5. Predictions (falsifiable)

1. A SmolLM2-135M extractor fine-tuned on rust-analyzer labels reaches 70–85 % edge-F1 on L2–L4 for held-out crates in bf16 and loses 3–8 points at int4. (Prior: Cassini 77.6 % at 30× the size on easier-to-parse languages, but with a noisier teacher.)
2. Its dominant failures will be (a) emitting a concrete target for method calls on untyped receivers instead of `null` (over-confidence), (b) repetition/looping in long JSON, (c) treating `#[cfg]`-inactive items as live. (a) and (b) are single-direction-ish and editable; (c) is a knowledge gap and is not.
3. Weight-space edits will move F1 by at most a few points overall but will cut the *confidently-wrong* rate by a large factor, which is the metric that matters for an agent (`1793874432-…`: report confidently wrong, missed and abstained separately).
4. Grafted circuits will be exact but will not raise F1 much, because the bottleneck is supplying the right facts table from the file, not executing the lookup (the `tiny-reexport` README's own conclusion).
5. On any crate rust-analyzer can load, the deterministic ladder will still beat the edited model on every level; the model's measurable wins will be confined to the no-cargo regime, latency per single file, and L6.

If (1) comes in below 60 % or (3) shows no reduction in confidently-wrong edges, the weight-space-programming step should be dropped and the model kept purely as a ranker/summariser over the deterministic graph.

## 6. What this changes in the earlier documents

- `1793864191-…` §7 ("model + CLIs + markdown"): unchanged; add "weight-space audit and repair" as the maintenance loop for the model component.
- `1793874432-…` "Could we put the computation inside the model?": the answer is now split — *rules*, yes, by compilation (Exp. 5/6, Qwenputer, tiny-reexport); *facts*, no; *abilities*, only by training.
- The 100 MB gate: keep it, but state it as **weights ≤ ~75 MB at int4 + runtime**, and choose the runtime for footprint (ORT cost 24 MB in `tiny-reexport`; a llama.cpp/ggml or candle build of a 135 M model should be measured the same way).

## 7. References

1. R. Takehi, J. Teoh, A. Zhang, O. Khattab. "Sparks of Weight-Space Programming: LLMs can natively read & write neural network weights." Blog post, MIT CSAIL, 2 Nov 2026. https://rikiyat.github.io/blog/weight-space-programming/ — logs at https://github.com/RikiyaT/lm_understand-logs. (Read in full today; all numbers in §1 are quoted from it.)
2. divagr18. "Qwenputer" — a RISC-V CPU compiled into 11 layers of Qwen3.5-4B. https://github.com/divagr18/qwenputer ; checkpoint https://huggingface.co/divagr1925/Qwen3.5-4B-Qwenputer (9.36 B parameters per the safetensors index).
3. HuggingFaceTB/SmolLM2-135M. https://huggingface.co/HuggingFaceTB/SmolLM2-135M — 134.5 M parameters, LlamaForCausalLM, bf16 (HF API, 2026-11-05).
4. LatentForce-ai/Cassini-1.0. https://huggingface.co/LatentForce-ai/Cassini-1.0 — Qwen3-4B base, LoRA r=1, six-field per-file JSON extraction, 77.6 % F1 on Python/JS/TS (model card, read 2026-11-05).
5. Qwen/Qwen3.5-0.8B. https://huggingface.co/Qwen/Qwen3.5-0.8B — 873.4 M parameters (HF API, 2026-11-05).
6. Other sizes checked via the HF API on 2026-11-05: Qwen/Qwen2.5-Coder-0.5B 494.0 M; Qwen/Qwen3-0.6B 751.6 M; bigcode/tiny_starcoder_py 164.1 M; HuggingFaceTB/SmolLM2-360M 361.8 M. Gemma 3 270M size taken from [1].
7. N. Elhage et al. "A Mathematical Framework for Transformer Circuits." Anthropic, 2021 — the induction-head method [1] reports Astra rediscovering.
8. In this repository: `devin-research/1793864191-can-an-llm-parse-rust-better-than-the-compiler-…md` (measurements); `devin-research/1793873685-can-rust-analyzer-draw-everything-accurately-…md` (limits); `devin-research/1793874432-rust-interface-graphs-explained-with-one-small-program.md`; `experiments/tiny-reexport/README.md` (compiled alias lookup; Linux repro numbers in this note's §2).
