# Can a small pretrained model recover real Rust return-type bindings?

For the short version with a real example, see [Which `Sender`? What a tiny model learned about Rust bindings](2026-11-05-minilm-rust-binding-observations.md).

**Yes to putting the weights inside a small model; no to trusting it to add edges from this experiment.** The [ready-to-run checkpoint](model/binding.onnx) is 17,454,319 bytes. When a return type could denote either of two same-named crate-local declarations, the fused model added **zero accepted bindings** on five held-out crate releases. A same-module rule alone found **56** rust-analyzer-confirmed bindings. This is a test of one dependency-graph edge (`signature -> return-type declaration`), not a whole-crate graph generator.

## What is inside the file?

We took the [pinned quantized Xenova ONNX conversion](https://huggingface.co/Xenova/paraphrase-MiniLM-L3-v2/tree/4b544e74dfc3256b2b56849ea5d7064fee1ac846) of the [Sentence Transformers MiniLM encoder](https://huggingface.co/sentence-transformers/paraphrase-MiniLM-L3-v2). Its transformer weights are **frozen**. We appended two scoring heads inside the model graph, not in Rust:

- The **64 original binding weights** score the same candidate-normalized lexical features as the [earlier ranker](../README.md).
- A **384-weight projection**, fitted on the earlier 192 synthetic rust-analyzer-labeled training examples, scores the pretrained encoder's first-token representation of each candidate. Its training objective is binary logistic loss on the difference between candidate scores, with the lexical margin fixed and an L2 penalty. Names and declaration order are not labels.
- One forward pass adds the two scores per candidate. All coefficients travel in **one ONNX file**; the small [metadata file](model/meta.json) contains the feature order and role-token IDs, not weights. [The builder](build.py) compares each output with independent calculations and checks that zeroing the lexical input changes the fused output. This is an encoder with trained heads, not a generative LLM, not a fine-tune of the attention layers, and not inference by rust-analyzer.

The Rust front end reads real source, parses it with `syn`, and supplies a return path and imports in the immediate syntactic module. For each candidate it encodes up to 62 normalized syntax tokens plus two boundary tokens. This removes many original spellings and can truncate imports. It does **not** give the encoder the whole Rust file, a resolved module graph, or the same-module heuristic. The original eight-slot alias executor is not needed here: the two choices already point to concrete declaration locations.

## The blind comparison

The [weight-space programming thesis](../../../../devin-research/1793877276-thesis-weight-space-programming-which-100mb-model-can-be-edited-to-act-like-rust-analyzer.md) separates attaching bounded operations from teaching a model a new capability. This experiment measures an attached learned scoring operation. It does not test editing an existing attention head or compiling rust-analyzer into a transformer.

No code or rust-analyzer output from the five releases was used to train the heads, select the threshold, or supply candidate answers. This is held out from **our supervised training**; we cannot certify what occurred in upstream pretraining. Three fresh single-thread CPU processes per crate produced identical predictions. Only after inference did a separate evaluator join the query's source location to an actual rust-analyzer LSIF definition and compare destination file, line and UTF-16 column. The index uses each crate's own Cargo manifest and the host's default configuration.

| Unmodified crate release | Two-way collisions | Oracle target in pair | Same-module rule confirmed | Fused model accepted after original rules | Max RSS, published sweep |
|---|---:|---:|---:|---:|---:|
| indexmap 2.12.0 | 39 | 33 | 0 | 0 | 80.02 MB |
| hashbrown 0.16.0 | 2 | 2 | 2 | 0 | 92.09 MB |
| aho-corasick 1.1.3 | 18 | 17 | 15 | 0 | 79.82 MB |
| regex-automata 0.4.13 | 45 | 36 | 35 | 0 | 91.67 MB |
| futures-channel 0.3.31 | 4 | 4 | 4 | 0 | 73.45 MB |
| **Total** | **108** | **92** | **56** | **0** | **92.09 MB max** |

Across 161 files there were 2,708 return-type path mentions and no file parse failures. Eligibility required exactly two declarations with the same leaf spelling in distinguishable crate-local modules. Public functions, public inherent methods and methods of public traits were inspected syntactically; effective external visibility, parameter edges, macro expansion, arbitrary re-exports and all feature combinations are **not** covered. Source discovery follows file layout and inline modules, not Cargo's resolved module graph. There is no guarantee that either candidate is the answer. Of the 108 sites, **11** resolved outside the offered pair and **5** had no oracle destination. Missing does not mean wrong.

The original qualifier/import rules abstained on all 108. For a second baseline, an unqualified name with exactly one candidate in the same syntactic module selects that candidate. This baseline emitted 61 proposals: **56 confirmed, 5 unverified, none contradicted**. Of its 47 abstentions, **36** had a correct answer in the candidate pair. The fused model, original lexical head and pretrained-encoder head all abstained at the frozen 0.80 threshold. A system that emits no predictions has no measurable precision at that threshold; it does not have 100% precision.

## What the misses teach us

- **A fact a model needn't guess:** `futures-channel::mpsc::channel` refers to `Sender` at `src/mpsc/mod.rs:349`. Two modules declare that spelling. Rust-analyzer picks the declaration at line 123 of the same file, not `src/oneshot.rs:26`. A deterministic module relationship provides information that the two normalized model prompts did not distinguish.
- **A useful lead, not proof:** in `indexmap`, the model weakly preferred the correct `set::iter::Iter` for `src/set.rs:213`. But for `as_slice` at `src/set/iter.rs:46`, it weakly preferred `map::slice::Slice` over the correct `set::slice::Slice`. Both mistakes and successes matter when a coding agent follows the resulting edges.
- **What if we let it guess?** The old lexical head tied on every real site. The encoder broke 37 ties: its higher score matched 24 oracle targets and missed 13. On just the 47 stronger-rule abstentions, that becomes **11 right, 4 wrong, 32 tied**. The largest lead equates to a two-choice score of about 0.657; none reached 0.80. These are retrospective ranking diagnostics, not accepted edges or calibrated probabilities. We did not tune on these crates.
- **Next experiment:** resolve module ancestry, imports and re-exports in the candidate retriever; retain an explicit “none of these” option. Train on real, repo-disjoint examples where rules abstain, then freeze a threshold on separate validation crates. Compare rule-only, a module-path heuristic, lexical-only and neural models on *fresh* crates at the same wrong-edge budget. Keep this five-crate set for regression, not as a new blind test.

## Run or reproduce

From this directory, set `ORT_DYLIB_PATH` as in the [original experiment](../../README.md#reproduce). The published sweep used macOS 15.3.1 arm64, ONNX Runtime 1.24.4 CPU and rust-analyzer 1.93.1. The 100 MB gate is **100,000,000 bytes of peak resident memory for the whole inference process**, including source parsing, declaration index, runtime, checkpoint and output. It excludes offline training, LSIF generation, scoring in Python, and memory used by other processes. One crate per process; other workspaces, machines and runtime versions need their own measurement. The first exploratory sweep peaked at 92,798,976 bytes; the final published sweep peaked at 92,094,464.

```sh
cargo build --release --locked --manifest-path ../../Cargo.toml
python3 check.py model
../../target/release/learned-binding real model/binding.onnx model/meta.json /path/to/crate

# Rebuild the checkpoint, fetch the unchanged releases, generate oracle indexes and score.
work=$(mktemp -d)
uv run --no-project --python 3.12 --with onnx==1.19.1 --with onnxruntime==1.24.4 --with numpy==2.2.6 python build.py "$work"
cmp model/binding.onnx "$work/binding.onnx"
uv run --no-project --python 3.12 python prepare.py "$work"
uv run --no-project --python 3.12 python evaluate.py "$work"
```

Add `--with scipy` to the `uv run` options and pass `--train` after the work directory to refit the residual instead of using the [published coefficients](residual.json). The data collector requires a working `rust-analyzer lsif` and Cargo. [Per-site predictions](real-predictions.jsonl) include both candidates, all three margins, decisions and independent oracle outcomes; coordinates are zero-based. [Aggregate results](real-results.json) include release hashes, all 15 RSS samples and the checkpoint SHA-256. [Model provenance and licensing](model/README.md).
