# Deep Insight 01 — The key differentiation: a rust-analyzer-trained small model wins on cold start, RAM and "any file, no setup"; rust-analyzer keeps the warm incremental regime

*Written 2026-11-05. Branch `research/tiny-rust-code-mapper`. Companion to ledger entry H-46 (`1793881000-hypotheses-ledger-…md`) and the ELI5 `1793941498-eli5-ascii-train-a-500mb-model-on-rust-analyzer-output-only-…md`. All rust-analyzer numbers are measured on this machine (`1793864191-…`, `1793873685-…`); all model numbers are estimates for a Mac mini M4 Pro GPU (~8 TFLOPS fp16, 273 GB/s; small-model decode is memory-bandwidth-bound, so decode speed ≈ bandwidth ÷ weight bytes). Verify with one real run before quoting.*

## 1. The question

If we post-train a ~500 MB model purely on rust-analyzer's views of the top-100 Rust repos (H-46), what can it honestly claim against rust-analyzer itself: faster? less RAM? It depends entirely on the regime.

## 2. CPU-only (the aim as originally stated): the model does not beat rust-analyzer on a loadable workspace

| | rust-analyzer (measured) | 500 MB student, CPU (est.) | 135M / 100 MB student, CPU (est.) |
|---|---|---|---|
| whole 42k-LOC workspace, cold | **29 s, 2.3 GB** (≈80 % of both in std + deps; cacheable) | 30 files × 15–25 s ≈ **10 min**, 0.6–1 GB | ≈ 2–3 min, ~0.2 GB |
| one file changed, index loaded | **milliseconds** | 15–25 s | 3–5 s |
| accuracy | ~100 % (0.02 % unknown types) | ≤ RA by construction; 78–95 % open | lower |
| needs | `cargo metadata`, dependency sources, build step for proc macros | the file | the file |

On CPU the only clean wins are "runs where RA cannot start" and "zero setup".

## 3. On the Mac mini GPU: cold start and RAM become real claims

| | rust-analyzer (measured) | 500 MB student on GPU (est.) | 135M / 100 MB student on GPU (est.) |
|---|---|---|---|
| one file: prefill 2k tokens + ~500 output tokens | ms (warm) / 29 s (cold load) | ~0.5 s prefill + ~2 s decode at 200–300 tok/s ≈ **2.5 s** | ≈ **0.7 s** |
| whole 42k-LOC workspace (30 files), cold, files batched | **29 s, 2.3 GB** | prefill compute-bound ≈ 15 s, batched decode nearly free → **~20–30 s, ~0.7 GB** | **~8 s, ~0.25 GB** |
| 10 M LOC, one file changed, nothing loaded | 10+ min load, then ms | **2.5 s, no load** | 0.7 s |
| accuracy | ~100 % | the open question (78–95 %) | lower |

```text
                 cold start          RAM              warm incremental      accuracy      setup
  rust-analyzer  29 s (42k LOC)      2.3 GB           ms        ◄ wins      ~100 % ◄ wins cargo + deps + build step
  500 MB on GPU  2.5 s per file ◄    ~0.7 GB ◄        2.5 s                 78–95 % ?     none ◄ wins
  135M on GPU    0.7 s per file ◄    ~0.25 GB ◄       0.7 s                 lower         none ◄ wins
```

## 4. What is defensible, and what is not

Defensible:
1. **Cold start / first answer** — seconds vs rust-analyzer's load time, and the gap grows with workspace size (10 M LOC: minutes vs 2.5 s).
2. **RAM** — 3–4× less for the 500 MB model, ~10× for the 135M model. Caveat: rust-analyzer with a pre-built std/deps index (H-33) would also shrink; compare against *that*, not against RA cold.
3. **Any file, no setup, no dependency download** — snippets, diffs, trees without `Cargo.toml`, Bazel/Buck, repos whose deps will not fetch; std and the popular crates live in the weights (44 % of references go there).
4. **Constant cost per file, independent of workspace size** — a startup property, not a steady-state one.
5. **Whole-workspace cold indexing** — a tie for 500M, a win for 135M, *but* comparing an approximate graph with an exact one.

Not defensible:
- "Faster than rust-analyzer" in the warm, incremental, editor regime: RA re-derives a changed body in milliseconds; the model needs seconds. This stays RA's.
- Any accuracy claim above RA's. The student is bounded by its teacher.

## 5. The headline sentence

> **"A 500 MB model gives you X % of rust-analyzer's graph for any Rust file in ~2.5 s on a Mac mini GPU, with no project setup, no dependency download, and a quarter of the memory."**

X is what the H-46 pilot measures. If X ≥ 95 this is a genuinely new tool (the per-file, no-setup rung of the ladder, usable in chat, in diffs, in CI on partial checkouts). If X ≈ 80 it is the fallback rung and the measured Cassini ceiling for Rust. The RAM and cold-start claims hold either way on GPU; the speed claim holds only for cold start.

## 6. Why this is the differentiation

Everything else in the stack (rust-analyzer-backed store, search tool, seven-event protocol) is exact engineering that others can also do. The thing nobody has is a toolchain-distilled model that **draws most of the exact graph for a file nobody can build**, in seconds, on a desktop GPU — with the exact resolver underneath for the cases where the project *does* load. "Resolver builds, student imitates anywhere, LLM steers."

## 7. Cheapest ways to turn estimates into measurements

- Tok/s and RSS of SmolLM2-135M and Qwen2.5-Coder-0.5B on the Mac mini under MLX, int8, 2k prefill + 500 decode (10 minutes).
- rust-analyzer cold vs with a pre-built std/deps cache on the same workspace (the fair RAM baseline).
- The H-46 pilot for X.
