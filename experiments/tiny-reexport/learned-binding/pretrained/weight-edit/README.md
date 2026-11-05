# Editing 740 MiniLM weights to distinguish same-named Rust types

**Verdict (2026-11-05): keep the original checkpoint.** A direct edit to the pretrained encoder changed which Rust type it weakly preferred in one real case. It did not change a single accepted decision across 168 return-type mentions. The original model nevertheless earned seven independently confirmed bindings on new crates where our deterministic rules abstained. Those are seven reference sites, not seven independent resolution mechanisms.

## What we wrote into the model

[Sparks of Weight-Space Programming](https://rikiyat.github.io/blog/weight-space-programming/) describes editing existing weights without an optimizer. Our narrower hypothesis: the upstream token embeddings for "candidate" and "other" might be too similar to distinguish same-named declarations. Unlike the [earlier experiment](../README.md), this time we did not append or train a head.

In the existing quantized word-embedding table, let `a` and `b` be the 384-byte rows for candidate roles M and O. Keep their midpoint fixed before rounding; replace their offsets from it with **four times** the originals. Round and clip to the original unsigned-byte range. This symmetric edit favors neither candidate position by design. It changes 740 scalar weights in two rows, uses no labels or gradients, and adds no parameters. Restoring that tensor reproduces the original checkpoint byte for byte. No examples, imports, features, output heads or acceptance thresholds were changed. We did not try successive multipliers and select a winner.

For comparison we ran the unchanged checkpoint, the edited checkpoint, and a score-only control that multiplies the original output margin by four without changing its ranking. The control is equivalent to lowering the original acceptance threshold and is not evidence of better name resolution.

## Where the original model helped

| Sites left unresolved by the stronger same-module/import rules | Original model | Edited model | Score-only control |
|---|---|---|---|
| 25 fresh sites; Tokio checked with its `full` feature | 7 confirmed, 1 unverified, 17 abstained | Same decisions | 16 confirmed, 1 unverified, 8 abstained |
| 47 sites from the five earlier crates | 47 abstained | 47 abstained | 3 wrong, 44 abstained |

The fresh releases were Tokio 1.48.0 (56 eligible sites), Rayon 1.11.0 (4), and crossbeam-channel 0.5.15 (0). They were picked before evaluating the edit; the five earlier crates are a regression set. After the rules abstained, 22 of the 25 fresh sites had the correct answer among the two candidates, two pointed elsewhere, and one had no unique verified destination. On this host, three fresh CPU processes per checkpoint and release produced deterministic predictions. The edited model occupied **17,454,319 bytes on disk** and the whole edited inference process peaked at **90,456,064 bytes**. This excludes offline rust-analyzer and Python evaluation and is not a whole-workspace guarantee.

Two Tokio examples illustrate the distinction:

- At `src/runtime/task_hooks.rs:71`, the original and edited models both picked the task `Id`, not the runtime `Id`. Rust-analyzer confirmed it. Of the seven confirmed sites, [six needed the original heads' scores combined](head-check.json): neither the lexical nor encoder score alone reached the pre-set threshold. Several sites are related methods; do not extrapolate seven successes to every Rust crate.
- At `src/runtime/task/join.rs:312`, the original model *weakly* favored runtime `Id`. The edit reversed that preference to the correct task `Id` (signed margin +0.021 to -0.117). Neither crossed the acceptance cutoff (absolute margin 1.386). It changed the ranking, not the graph.

**The grading trap:** Tokio disables many features by default. The first rust-analyzer LSIF index verified none of its 56 sites. We froze those predictions, copied the crate unchanged except for setting its default feature to `full` in the oracle's temporary Cargo manifest, and regenerated the definition index. We did not run a second prediction pass to get better answers. These confirmations apply to that feature configuration, not to every build; the other crates retained their defaults. Rust-analyzer logged unnamed enclosing definitions even though indexing exited successfully. Missing answers are recorded as unverified, never as correct or wrong.

**Product decision:** Don't ship the edit or globally lower the threshold. Preserve the seven Tokio sites, especially the six where the two sources of evidence worked together. For the next trial, supply the module/import path to each candidate and check new crates at the same wrong-edge budget. A weight edit can't distinguish candidates whose observable inputs are identical.

## Reproduction

Run the [edited checkpoint](model/binding.onnx) with its [feature and token map](model/meta.json), or regenerate it using the commands below. It is a research comparison, not a replacement for the original model. The [base model credits](../model/README.md) and [Apache-2.0 license](../model/LICENSE.txt) apply.

From this directory with the [release Rust CLI and CPU ONNX Runtime](../../../README.md#reproduce) built and `ORT_DYLIB_PATH` exported:

```sh
work=$(mktemp -d)
uv run --no-project --python 3.12 --with onnx==1.19.1 --with numpy==2.2.6 python run.py "$work"
uv run --no-project --python 3.12 python tokio_full.py "$work"
uv run --no-project --python 3.12 python check_heads.py "$work"
```

To include the five regression releases, pass a second argument to `run.py`: a directory populated by [the previous corpus preparation script](../prepare.py). Output includes the exact edited checkpoint (`binding.onnx`), runtime metadata, hashes, paired scores, decisions, parse failures and all RSS readings. Checkpoint SHA-256: `cda639f2e9b18805c0e39552d87155582a8ebd269a887febbb6e6afd3ce29dd0`. The shipped base is unchanged.

[Published run](report.json) | [Paired site scores](cases.jsonl) | [Feature-aware scores](full-report.json) | [Tokio destinations](full-tokio-cases.jsonl) | [Pinned archives](releases.json)
