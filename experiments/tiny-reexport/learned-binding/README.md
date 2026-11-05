# Can a tiny learned model pick the right Rust binding?

**Result, November 5, 2026:** yes on familiar syntax, unevenly on withheld syntax, and **yes under 100 MB on CPU**. A trained 64-weight ranker followed by the original exact ONNX lookup produced 115 correct public return-type edges, one false edge and 64 abstentions across 180 held-out files. A simple deterministic matcher picked the right module more often. Do not use the model's score as verification.

## One example

Both modules in [return_alias.rs](examples/return_alias.rs) define a public `Header`. The return type is not `Header`:

```rust
use self::files::Header as OnDisk;
use self::frames::Header as OnWire;
pub fn make() -> OnWire { loop {} }
```

The model picked `frames`. The whole pipeline emitted `make --return--> frames::Header`, with public spelling `OnWire`. Rust-analyzer independently navigated from the return token to that struct declaration. The body supplies no clue.

## What was learned?

1. A Rust CLI parses a raw source file with `syn`. It enumerates two inline modules exporting the same type name and reads the return path and the syntactic import trees. Rust does **not** tell the model which import resolves the return mention.
2. For each candidate, the front end replaces identifiers with role markers: this module, the other module, queried return name, qualifier, common export name, or unrelated identifier. It keeps local and outer scope separate. Features are windows of one to six tokens over those sequences. Candidate order and original names are not features. This is structured feature engineering, not an LLM trained on raw tokens.
3. Logistic regression learns a shared scoring function from the *difference* between the two candidate feature vectors. There is no intercept or positional prior. The 64 nonzero weights occupy **2,737 bytes** of JSON. A pre-set score threshold of 0.80 determines whether to propose an edge or abstain; this score is not calibrated under distribution shift.
4. The chosen module's local export is resolved to a canonical struct using the earlier [fixed-weight ONNX circuit](../README.md). Rust formats the result; it never runs the deterministic baseline or rust-analyzer in prediction mode. Control tests show that zeroing the learned weights forces abstention and negating them reverses a binding on otherwise identical source.

## What counts as unseen?

The generator produces paired files: candidate declarations stay in the same order, while the import or return path changes which module is right. Declarations and import order vary across pairs. All source hashes are distinct, and identifier sets are disjoint across generated splits. Training uses **192 files**, three forms, and rust-analyzer labels. We reconstructed labels using actual LSIF definition destinations at the return-type token, not the generator's choice. The teacher is rust-analyzer 1.93.1; the generated corpus lives in one dependency-free Cargo crate. Four separately written probes were added *after* freezing the weights; the model SHA did not change.

| Held-out slice | Files | Correct edges | Wrong edges | Abstained | Simple rules: correct edges |
|---|---:|---:|---:|---:|---:|
| New names/order, familiar qualified paths and direct imports | 72 | 72 | 0 | 0 | 72 |
| Six withheld source forms | 96 | 40 | 0 | 56 | 64 |
| Deliberately unbound bare name | 8 | 0 | 0 | 8 | 0 |
| Four separate probes, one with no definition | 4 | 3 | 1 | 0 | 3 |
| **Total** | **180** | **115** | **1** | **64** | **139** |

**The honest novelty check:** after names are normalized, the 192 training files collapse to just **six distinct pairs of feature vectors**, ignoring candidate order. All 72 familiar cases map to seen feature inputs. Of the 96 withheld-form cases, 16 local re-export cases also map to seen ranker inputs: their novel part is handled by the fixed executor. On the remaining **80 actually novel feature inputs** (case count, not 80 independent patterns), the model resolved **24** and abstained on **56**. This is not evidence of generalization to unseen OSS workspaces.

The 40 successes in the withheld-form slice were 16/16 for inner imports shadowing outer ones, 16/16 for candidate-local re-exports, and 8/16 for explicit imports competing with globs. It abstained on **all** grouped imports, module aliases and two-hop consumer alias chains. Picking the first declaration would get half the resolvable paired cases right. The stronger rules control matches explicit imports or qualifiers in the immediate query scope and otherwise abstains; it is not a full Rust resolver. Its comparison is scored on module identity. The model's edges are scored on canonical module **and** struct identity.

**The most useful failure:** in [missing_prefix.rs](examples/missing_prefix.rs), `Choice` is a real imported alias but the function returns `nonexistent::Choice`. Rust-analyzer has no target. The model proposed `reader::Ticket` at **0.993**. The fixed executor correctly resolved the *wrong query*; it cannot prove the module prefix is in scope. Keep this as a negative fixture. Use deterministic path validation before promoting model proposals into verified graph facts.

## Memory and reproduction

Five fresh release CLI processes on macOS 15.3.1 arm64 each streamed **all 180 held-out Rust files**. Highest process RSS: **23,707,648 bytes**, measured by macOS `/usr/bin/time -l`; the hard gate is 100,000,000 bytes. This includes reading source, parsing, candidate construction, learned weights, scoring, loading ONNX Runtime 1.24.4, fixed-model inference and graph serialization. The Python measurement driver, offline training and rust-analyzer are not shipped inference steps. These small files do not establish whole-workspace memory behavior.

Use the [parent README](../README.md#reproduce) to set `ORT_DYLIB_PATH`. From this directory:

```sh
cargo build --manifest-path ../Cargo.toml --release --locked
../target/release/learned-binding predict weights.json examples/return_alias.rs
python3 tests.py
# Requires rust-analyzer with the lsif subcommand. Rebuilds labels, trains,
# measures RSS, and rewrites the published weights and evaluation artifacts.
uv run --no-project --python 3.12 --with scikit-learn==1.7.2 --with numpy==2.2.6 python run.py
```

Inspect the [aggregate metrics and model hash](results.json), [per-case source hashes, oracle targets, outputs and scores](predictions.jsonl), [training and evaluation code](run.py), [source generator](corpus.py), and [LSIF reader](oracle.py). The CLI is intentionally bounded: two inline candidate modules, one unit struct and at most one local renamed re-export in each, one public zero-argument return query at root or one module deeper, and at most 64 KiB of source per file. It does not load package dependencies, resolve arbitrary module files, expand macros or evaluate features. Use the learned component as an experimental proposal generator, not as a replacement for Rust name resolution.
