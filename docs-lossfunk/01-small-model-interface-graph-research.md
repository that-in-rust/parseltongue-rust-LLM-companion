# Can a tiny local model build a better Rust public-interface dependency graph?

Lossfunk Batch 9 research brief. Proposed experiment, not a model card or a benchmark result.

**The artifact we want is a map a Rust systems programmer can hand to another programmer or a code harness.** It says which contracts a downstream crate can name, what each contract depends on, and where in the source to check. The post-trained local model has one job: build and refine that map on code it has not seen during post-training.

The [existing CLI](../README.md) gives us both a distribution path and a serious control. It already indexes code with Tree-sitter and serves dependency queries. The question is not whether a small model can print plausible edges. It is whether the model produces a more accurate *whole graph* than that pipeline on new large Rust OSS repositories, at a cost that makes it reasonable to ship.

## What the application commits us to

| Commitment | What would count |
|---|---|
| Corpus | Select the top 50 OSS Rust libraries under a published, dated ranking rule; pin repository commits and record license, owner, workspace and configurations. Do not count multiple crates from one workspace as independent examples. |
| Training target | Generate reference graphs with the application's proposed teacher, **Codex 5.6 xhigh**. Record actual available model ID, settings, prompt, source revision and evidence per assertion. A teacher answer is a training label, not a fact or a blind test answer. |
| Student | Post-train an open-weight CPU-local model that discovers declarations and emits typed dependency assertions. The preferred deployment target is **under 100 MB peak inference RAM**, not merely a small download. |
| Independent test | Reserve *additional* Rust OSS repositories outside the 50, including cross-crate systems workspaces. Freeze the test before tuning and independently adjudicate a stratified sample including missed edges. |
| Release | Open weights on Hugging Face, redistributable dataset plus generation/training/evaluation code, and a Parseltongue CLI that actually consumes the generator. Clearly label a negative result rather than advertising an improvement. |

## What the model predicts

The unit is not a summary paragraph. It is a declaration or a claim such as "the public result type of this function refers to that definition." For each claim, attach the source range, build configuration, relation kind, proposed target and evidence state. Effective external exposure, including re-exports, matters more than counting occurrences of `pub`. See the [visibility rules](https://doc.rust-lang.org/reference/visibility-and-privacy.html).

Separate three views: public signatures and exposed types; source-backed implementation relations, including private witnesses; and a compact export intended for a human or agent. A private helper can explain a path but does not become part of a crate's public API. A possible dispatch target is not a proven runtime call.

## A fair contest

1. Freeze the released Parseltongue extractor, grammar, options and edge adapter. Report which relations it does not represent; do not silently relabel every `Uses` as a public signature dependency.
2. Include a deterministic scope/import resolver and a Rust-aware build-configured reference. Compare like with like on available inputs; disclose when macro expansion or a successful build was available to only one arm.
3. Train a source-to-record student. Let a second student request bounded source and graph neighborhoods before emitting corrections. Give both an explicit unknown outcome.
4. Compare both with a parser-supplied candidate classifier. Report candidate recall separately: a perfect ranker cannot emit a declaration that parsing never proposed. A hybrid win is useful but not evidence for independent model construction.

## Hugging Face Precedents

Approximate weight-only floors below use `parameters * bits / 8`, in decimal MB. They exclude quantization metadata, tokenizer, buffers, model runtime, KV cache, activations, allocator overhead and graph/parser state. File size is not measured process RAM. Architectures, licenses and model revisions must be pinned before training.

| Candidate | Published evidence | 4-bit ideal weight floor | Proposed role and caveat |
|---|---|---:|---|
| [AMD-Llama-135m-code](https://huggingface.co/amd/AMD-Llama-135m-code) | Card describes 135M, 2,048-token context, code model; Apache 2.0. | 67.5 MB | Most direct small code-model starting point. Rust training share and task performance require verification. 4-bit feasibility under full-process budget is unproven. |
| [SmolLM2-135M](https://huggingface.co/HuggingFaceTB/SmolLM2-135M) | 135M small language-model family. | 67.5 MB | General small-model control. Compare base and instruct variants separately; Rust suitability is not established by the card. |
| [CodeRankEmbed](https://huggingface.co/nomic-ai/CodeRankEmbed) | 137M bi-encoder for code retrieval; card reports 8,192 context. | 68.5 MB | Candidate-retrieval or ranking precedent, not an edge resolver. Verify license, quantization support and RAM independently. |
| [GraphCodeBERT](https://huggingface.co/microsoft/graphcodebert-base) | Code model trained with data-flow information; card reports 512-token maximum. | Determine from pinned configuration | Architecture precedent for code plus structure, not evidence for Rust graph extraction or budget compliance. |
| [CodeBERT](https://huggingface.co/microsoft/codebert-base) | Code-oriented pretrained encoder. | Determine from pinned configuration | Encoder baseline for candidate classification; verify model size, license and Rust coverage. |
| [CodeParrot-small](https://huggingface.co/codeparrot/codeparrot-small) | [Hugging Face's model overview](https://huggingface.co/spaces/codeparrot/code-generation-models/blob/main/architectures/codeparrot.md) identifies 110M and Python-focused training. | 55 MB | Useful negative/control precedent for domain mismatch; not a Rust specialist. |
| [Qwen2.5-Coder-0.5B](https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B) | Card reports 0.49B parameters and Apache 2.0. | 245 MB | Teacher or unconstrained accuracy ceiling. The official [instruct GGUF files](https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF/tree/main) list Q2_K at 415 MB; not a 100 MB deployment candidate. |
| [TinyForge-60M](https://huggingface.co/Juanillaberia/TinyForge-60M) | Card reports ~64.7M total parameters and 1,024-token context; trained on general educational text. | 32.4 MB | Memory-first research control, with substantial task/domain uncertainty; inspect license and maturity. |

The LoRA adapter alone is not the deployed model: base weights, adapter, runtime and buffers must all count. Quantization can reduce weight memory but may affect model quality. A task-specific encoder or smaller distilled student may fit more comfortably than an autoregressive model, but that is a hypothesis to test.

### What does 100 MB mean?

Pre-register two measurements. Only the first is the application's preferred sub-100 MB target:

- **Model-process cap:** peak resident memory of a fresh inference process including model, tokenizer, buffers, context and runtime; parser/DB processes measured separately.
- **End-to-end footprint:** peak memory attributable to the complete local pipeline, including parser, candidate index, model and database. Measure overlapping processes as well as device memory; report shared/mapped pages and measurement method.

Report MB versus MiB explicitly, hardware, OS, context length, batch size, thread count, model format, cold/warm state and peak measurement. Weights-only numbers are feasibility estimates. The evidence above does not demonstrate the inference target on this task.

## Evidence and Verification

The [signature-extraction study](https://docs.google.com/document/d/10ZGBTTTMrLplfdDnNcRlkZpvLSgXD5_zoEcZpwpwyy0/edit) explains why identifying a declaration and resolving its types need separate scores. The [Google Docs source register](02-learned-graph-compiler-thesis.md#google-docs-research-sources) links the rest of the project research.

### The baseline must resolve symbols

The repository [README](../README.md) describes Tree-sitter extraction and forward/reverse graph APIs. It does not prove semantic call accuracy. A fair benchmark needs the current Parseltongue pipeline at a pinned commit, a strengthened deterministic Tree-sitter resolver, and a Rust-aware reference. Use the same source, build configuration and output schema.

[rust-analyzer](https://rust-analyzer.github.io/book/features.html) supports references and macro-related navigation. [SCIP](https://github.com/scip-code/scip) is a code-intelligence format with Rust bindings; [scip-rust](https://github.com/scip-code/scip-rust) documents `rust-analyzer scip .`. SCIP references are not automatically call edges: an adapter must classify call sites and normalize owners. Rust-analyzer is a strong reference, not an infallible oracle, especially for incomplete code, macros, dynamic dispatch and build configurations.

### A concrete ambiguous example

```rust
mod alpha { pub fn run() {} }
mod beta  { pub fn run() {} }
use alpha::run as execute;
pub fn start() { execute(); }
```

The target is `alpha::run` via the alias, not whichever `run` appears first. A Tree-sitter system can solve this deterministically; a small model must beat an implementation that actually handles aliases, not a name-only lookup. Further fixtures should add same-named methods, `self` versus trait methods, generic bounds, re-exports, `cfg`, macros, partial files and external symbols.

### Verification questions

| Question | Answer from available evidence |
|---|---|
| Has an under-100 MB model beaten a strong Rust graph pipeline here? | No benchmark or local experiment has been inspected that establishes this. |
| Is Tree-sitter itself a symbol resolver? | Its role in the proposed pipeline is syntax parsing; edge resolution is additional system logic. Compare end-to-end pipelines. |
| Does `pub` guarantee external reachability? | No. Ancestors, restricted visibility and re-exports matter, per the Rust reference. |
| Does a 4-bit weight estimate prove runtime fits? | No. Measure peak total memory with the actual runtime and workload. |
| Is a SCIP reference automatically a call edge? | No. A reference-to-call adapter and labeled validation are needed. |
| Does a content hash establish entity identity? | No. Local [ISGL1-v2 matching](../crates/parseltongue-core/src/isgl1_v2.rs) uses content and positional heuristics; edits and duplicates make matching ambiguous. |
| Can the model prove complete source coverage? | No. Exact byte accounting must be independently enforced by the capture system. |

## Score the product, not just the selected edges

| Question | Measurement and failure mode |
|---|---|
| Can it find the public surface? | Precision and recall of externally reachable definitions and exposure paths, per repository/configuration. An alias and its definition must not be counted as two independent definitions. |
| Does it tell the truth about dependencies? | Exact typed-edge precision, recall and F1 by relation; separately count wrong confident targets, abstentions, false paths and missing nodes. Macro-average across repositories and show hard cases. |
| Does the graph save investigation work? | On blind downstream-impact and flow-navigation tasks, compare graph-assisted versus baseline source lookup with the same harness, prompts and token accounting. Verify final answers and source citations, not just tool usage. |
| Is it compact? | Tokenize the full canonical public export and the same in-scope Rust sources with a named tokenizer. Report the ratio per repository, what was omitted, and task accuracy. **About 5%** is the application hypothesis, not the cost of one query. |
| Will it run locally? | Measure cold and warm peak RSS of a fresh CPU inference worker at the largest supported window, alongside end-to-end application peak, time, threads and working-set limits. A quantized weight floor is not a RAM result. |

Train first on declarations and exposure paths, then on linking, then on corrections using retrieved evidence. Preserve unresolved labels, duplicate-name negatives, failed builds, re-exports through private modules, feature-specific traits and cross-crate edges in the corpus. Freeze family-level train/validation/test splits before deriving windows; never tune thresholds on the final holdout.

**Decision rule:** claim a graph-quality win only when the predeclared held-out metric improves over the frozen CLI without trading away the agreed false-edge budget. Report the stronger semantic baseline even if it wins. Publish model RAM and graph/source token ratio as separate results, including misses. The [construction thesis](02-learned-graph-compiler-thesis.md) specifies the action loop; the [five timelines](03-five-product-timelines-and-ramifications.md) describe how to put the same graph to work.
