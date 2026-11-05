# One Rust re-export, two executors

**Result (2026-11-05):** both paths emitted the same return-type graph. The fixed-weight ONNX model executed the alias lookup on CPU in a single Rust process below **24 MB peak RSS** on both fixtures. Native Rust was faster and smaller. This is an isolated experiment, not a change to the production Parseltongue indexer.

```rust
pub struct Circle;
pub use self::Circle as Round;
pub fn make() -> Round { Circle }
```

```json
[{"from":"make","role":"return","spelling":"Round","target":"Circle"}]
```

## What actually ran inside the model?

Both modes parse the same source with `syn`, assign addresses to names, and record literal declarations, one-hop aliases and public return-type mentions. This shared front end does **not** chase aliases, supply the canonical target, or look at the function body to choose an answer. Output formatting is shared as well.

- **Native:** start at the spelling in the return position; follow the unique binding in each row until a definition, missing/conflicting binding or cycle.
- **Model:** run [resolver.onnx](resolver.onnx) on ONNX Runtime's CPU provider, one thread. The network takes three changing tensors: definition counts, alias counts, and a one-hot starting address. The serialized weights are the same for every file. No callbacks into the Rust resolver and no host-side transitive closure.

[compile_model.py](compile_model.py) compiles the lookup rule into a fixed numerical network. It gives definitions a self-loop and zeros any row with conflicting bindings. For each step, a ReLU gate selects the addressed row: for bits, `ReLU(address + binding - 1)` is AND. A fixed all-ones projection sums the selected row into the next address. After eight steps a terminal mask discards pointers stuck in cycles. The output is one-hot for a definition or all zeros for unresolved; the Rust wrapper only decodes that output. The ONNX file is **2,460 bytes** (73 fixed float constants plus the operators and wiring).

This borrows [Qwenputer's](https://github.com/divagr18/qwenputer) idea of *compiling an exact operation into a network*. It is **not** Qwen, a trained language model, a RISC-V emulator or a source-to-graph neural parser. No probabilities or learned parameters are needed for the lookup. A learned controller deciding which query to issue would be a different experiment.

## Measurements

Apple arm64, macOS 15.3.1, Rust 1.93.1 release build, ONNX Runtime 1.24.4. Five fresh processes **per mode per fixture**, alternating execution order. Peak is the maximum RSS reported by macOS `/usr/bin/time -l` in bytes; timings are median end-to-end wall times including process launch and the time wrapper. No attempt to evict the OS file cache. Both modes include parsing and emitting JSON; the model also includes loading the shared library, initializing the session and running inference. No Python interpreter, compiler, GPU or separate worker is on the measured path.

| Fixture | Native peak RSS | Model peak RSS | Native median | Model median |
|---|---:|---:|---:|---:|
| [Round to Circle](fixtures/round.rs) | 2,080,768 B | 23,609,344 B | 4.30 ms | 24.58 ms |
| [A to H, seven alias hops](fixtures/eight.rs) | 2,375,680 B | 23,527,424 B | 3.56 ms | 25.97 ms |

The pass threshold was **100,000,000 bytes**, not checkpoint size. These numbers show ample room for this operation on this host, not a memory guarantee for a real workspace. The end-to-end timings do **not** isolate per-hop compute.

## Correctness and limits

- Both modes matched the expected JSON for the Rust fixtures, changed alias targets, direct returns, raw identifiers, missing definitions, cycles and duplicate bindings. Nightly rustdoc JSON (`nightly-2026-04-01`) independently resolved `make` to the struct `Circle` under spelling `Round` and to struct `H` under spelling `A`.
- A differential test also enumerates **216** three-address tables (six row choices at each address), querying all three addresses: **648** native/model comparisons with zero differences. This tests the numeric circuit independently of parsing. It is not an exhaustive test of Rust or of all eight-address inputs.
- Scope: one flat file up to 64 KiB, eight distinct type names, public unit structs, explicit renamed re-exports, and public zero-argument functions returning a plain type name. No nested modules, globs, generics, feature evaluation, macros, or full name resolution. Unsupported item forms and overflow reject the input; a missing, cyclic or ambiguous binding yields `"target":null`, not a guessed edge. Bodies are syntactically parsed but not type-checked. The invalid-binding fixtures are not required to build; the goal there is abstention.

## Reproduce

From this directory on macOS arm64 or Linux with GNU time, Rust 1.88+ and `uv`:

```sh
# Use the CPU runtime library from the wheel, not Python for inference.
export ORT_DYLIB_PATH="$(uv run --no-project --python 3.12 --with onnxruntime==1.24.4 python -c '
from importlib.util import find_spec
from pathlib import Path
d = Path(find_spec("onnxruntime").origin).parent / "capi"
print(next(p for p in d.iterdir() if p.name.startswith("libonnxruntime.") and (".so" in p.name or p.suffix == ".dylib")))
')"
cargo build --release --locked
cargo test --release --locked
target/release/tiny-reexport native fixtures/round.rs
target/release/tiny-reexport model fixtures/round.rs
python3 measure.py
python3 measure.py fixtures/eight.rs
```

The native mode does not load or require the ONNX shared library. Both CLIs accept `-` for stdin. To regenerate the included network, run `uv run --no-project --python 3.12 --with onnx==1.19.1 python compile_model.py`, then rebuild: the Rust binary embeds it. To check the target with rustdoc independently:

```sh
out=$(mktemp -d)
rustdoc +nightly-2026-04-01 --edition 2021 --crate-type lib -Z unstable-options \
  --output-format json -o "$out" fixtures/round.rs
jq '. as $d | .index[] | select(.name=="make") | .inner.function.sig.output.resolved_path |
    {spelling:.path, target:$d.index[(.id|tostring)].name}' "$out/round.json"
```

**Decision:** use native lookup when the binding table is already known. To test the value of a *learned* component, hold this executor fixed, give a student source with competing in-scope candidates, and score whether it supplies the right evidence or abstains on unseen crates. That would test what this experiment deliberately gives both paths for free.
