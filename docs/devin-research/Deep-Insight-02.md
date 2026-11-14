# Deep Insight 02 — Shortcutting the training time: the dataset lever beats the model lever, and a scaling ladder beats both

*Written 2026-11-05. Branch `research/tiny-rust-code-mapper`. Follows Deep-Insight-01 and ledger entry H-46 (post-train a ~500 MB model on rust-analyzer output only). All numbers are 6·N·D estimates for a Mac mini M4 Pro (~2.5 TFLOPS effective in training) and one H100 (~400 TFLOPS effective); verify with a 10-minute MLX run before planning around them.*

## 1. Question

H-46 as written costs ~1 B tokens/epoch for a 0.5 B model = 3×10¹⁸ FLOPs ≈ 2 weeks per epoch on the Mac mini. Can we shortcut this by playing with model size or dataset size?

Yes. The dataset lever is much larger than the model lever, and the biggest shortcut is a procedure (a scaling ladder), not a number.

## 2. The levers, largest to smallest

| lever | what you do | time saved | what it costs |
|---|---|---|---|
| **Start from a code-pretrained base** (Qwen2.5-Coder-0.5B, SmolLM2-360M) instead of teaching syntax | skip phase 1 entirely — the base already parses Rust. Train only phase 2: source → compact records | **5–10×** (1 B → 100–200 M tokens) | nothing real; "learn the syntax first" was the expensive part and it is already paid for in the base |
| **Shrink the input, not the labels** | train on item-level windows (one fn + the signatures it references, ~300–500 tokens) instead of whole files; dedupe vendored/generated/test code (top-100 repos are maybe 40 % redundant) | **2–3×** | some cross-item context; fine for L2–L4, which are per-item anyway |
| **Smaller model** 0.5 B → 135 M → 50 M | | 3.7× / 10× | capacity for receiver typing (65 % of call edges) is what is at risk; 50 M probably fails it |
| **Narrow the task first** | phase 2a = item skeleton only (L2); phase 2b adds definitions + types (L3–L4) | the L2 model converges in ~10 M tokens | you get the public-skeleton extractor early and cheaply |
| LoRA, sequence packing, MLX fused kernels | | 1.3–2× | none, but these do not change the order of magnitude |

## 3. Combined

```text
   H-46 as written        0.5 B model, 1 B tokens, phases 1+2       3×10^18 FLOPs   ~2 weeks  Mac M4 Pro   ~2 h  H100
   with levers 1+2        0.5 B model, ~100 M tokens, phase 2 only  3×10^17 FLOPs   ~1.5 days Mac M4 Pro   ~15 min H100
   with levers 1+2+3      135 M model, ~100 M tokens, phase 2 only  8×10^16 FLOPs   ~8 h      Mac M4 Pro   ~4 min  H100
```

## 4. The real shortcut: a scaling ladder instead of a committed dataset size

```text
   step 0   zero training: few-shot the base model with the record schema  →  X₀   (1 h; measures what is already there)
   step 1   1 M tokens    → X₁      (~10 min on the Mac)
   step 2   10 M tokens   → X₂      (~2 h)
   step 3   100 M tokens  → X₃      (~1 day)                       stop as soon as X flattens
   held-out repos fixed across all steps; score at int4 (the shipped artefact)
```

Three points give the log-linear curve. By step 2 you know whether 100 M tokens reaches ~95 % or sits on the ~78 % (Cassini) ceiling — before spending the day. Fine-tuning curves of this kind usually flatten by 10–50 M dense label tokens, and rust-analyzer labels are dense (every output token is supervision), so the knee should come early.

X = held-out exact-match on (a) item skeleton, (b) definition path for 1 k sampled tokens, (c) receiver type for 1 k method calls — the H-46 metrics.

## 5. Do not cut

- Held-out **repos**, not held-out files (otherwise you measure memorisation).
- The `unresolved` / abstain examples (otherwise the model learns to invent edges; silent omission and invention are the failure modes that matter — see H-42).
- Evaluation at int4 — cutting it saves no training time and makes the result unusable for the shipped artefact.

## 6. One-line version

Use a code-pretrained base, train only the record-emission task on item windows, and climb 1 M → 10 M → 100 M tokens while watching the held-out curve; expect to know the answer in ~2 hours of Mac GPU time and to finish in a day, not two weeks.

## 7. ELI5 ASCII — the levers

```text
THE BILL:   time  =  6  ×  (model size)  ×  (tokens you feed it)        ÷  (how fast the chip is)
                                 ▲                 ▲                              ▲
                              lever 3           levers 1, 2, 4                 lever 5
                              (small)           (BIG)                          (fixed: you own one Mac)


LEVER 1 — don't teach the alphabet twice                   saves 5–10×

   old plan:    [ learn Rust syntax ]────────►[ learn to emit records ]      1,000 M tokens
                 ▲ 900 M tokens here
   shortcut:                                  [ learn to emit records ]        100 M tokens
                 "Qwen-Coder already reads Rust. Only teach it OUR output."


LEVER 2 — feed snacks, not the whole fridge                 saves 2–3×

   whole file in:   ┌────────────────────────────────────────┐ 3,000 tokens  → 1 label set
                    └────────────────────────────────────────┘
   item window in:  ┌──────┐ 400 tokens: this fn + the 5 signatures it touches → same labels
                    └──────┘
   + throw out duplicates (vendored / generated / test boilerplate ≈ 40 % of the top-100 repos)


LEVER 3 — a smaller brain                                   saves 3.7× (135 M)  or 10× (50 M)

   500 M  ■■■■■■■■■■  can hold "what type is x?" for 65 % of call edges   ✓
   135 M  ■■■        probably                                             ?
    50 M  ■          probably not                                         ✗   ← the lever with a cliff


LEVER 4 — start with the easy exam                          first useful model after ~10 M tokens

   2a   source  →  item skeleton (what's pub, what's a fn/struct/trait)        easy, converges fast
   2b   source  →  + where each name is defined, + the type of each receiver   hard, needs more tokens
   ship 2a while 2b trains.


LEVER 5 — tune the engine                                   saves 1.3–2× ; doesn't change the order of magnitude
   LoRA, sequence packing, MLX fused kernels


ADD THEM UP (Mac mini M4 Pro)

   H-46 as written           ██████████████ 2 weeks
   + lever 1 + 2 (0.5 B)     ██ 1.5 days
   + lever 3 (135 M)         ▌ 8 hours
   (same, one rented H100)   ▏ 4–15 minutes


THE SHORTCUT THAT ISN'T A LEVER — climb, don't jump

   tokens      1 M        10 M        100 M
   time       10 min      2 h         1 day
   score X    X₁ ───────► X₂ ───────► X₃
                            ▲
               by here you can see whether the line heads to 95 % or flattens at ~78 %
               → stop early if it flattens; only buy the day if the line is still climbing

   same 10 held-out REPOS at every rung    ·    score at int4    ·    keep the "unresolved" examples
```
