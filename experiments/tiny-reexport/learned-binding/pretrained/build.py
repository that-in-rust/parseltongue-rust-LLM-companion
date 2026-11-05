"""Attach the frozen 64-weight ranker and a fitted residual to real MiniLM."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import urllib.request

import numpy as np
import onnx
from onnx import helper as h, numpy_helper as nh, TensorProto as T
import onnxruntime as ort

HERE = Path(__file__).resolve().parent
PARENT = HERE.parent
sys.path.insert(0, str(PARENT))
BASE = "https://huggingface.co/Xenova/paraphrase-MiniLM-L3-v2/resolve/4b544e74dfc3256b2b56849ea5d7064fee1ac846/"
ARTIFACTS = [
    (
        "base.onnx",
        "onnx/model_quantized.onnx",
        "b190f50dd46296b9895ae8f274c3455762d08610f8788f4a9bd15019f4f7160c",
    ),
    (
        "vocab.txt",
        "vocab.txt",
        "07eced375cec144d27c900241f3e339478dec958f92fddbc551f295c992038a3",
    ),
]
WORDS = {
    "return": "return",
    "use": "import",
    "outer": "outer",
    "M": "candidate",
    "O": "other",
    "Q": "query",
    "P": "prefix",
    "T": "type",
    "id": "word",
    "as": "as",
    "::": ":",
    "{": "{",
    "}": "}",
    ",": ",",
    "*": "*",
    "self": "self",
    "super": "super",
}


def open_cpu_model_session(path):
    options = ort.SessionOptions()
    options.intra_op_num_threads = options.inter_op_num_threads = 1
    return ort.InferenceSession(str(path), options, providers=["CPUExecutionProvider"])


def encode_candidate_prompt_ids(words, tokens):
    seq = [101] + [tokens[w] for w in words[:62]] + [102]
    return seq + [0] * (64 - len(seq))


def pack_candidate_model_inputs(e, keys, tokens):
    ids = np.array(
        [encode_candidate_prompt_ids(w, tokens) for w in e["tokens"]], dtype=np.int64
    )
    bits = np.array([[k in v for k in keys] for v in e["features"]], dtype=np.float32)
    return dict(
        input_ids=ids,
        attention_mask=(ids != 0).astype(np.int64),
        token_type_ids=np.zeros_like(ids),
    ), bits


def train_synthetic_residual_head(work, keys, old, tokens):
    from scipy.optimize import minimize
    from scipy.special import expit
    from corpus import generate_paired_source_cases
    from oracle import collect_rust_analyzer_labels

    base = open_cpu_model_session(work / "base.onnx")
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        train = [r for r in generate_paired_source_cases(root) if r["split"] == "train"]
        collect_rust_analyzer_labels(root, train)
        paths = [root / "src" / (r["case"] + ".rs") for r in train]
        proc = subprocess.run(
            [PARENT.parent / "target/release/learned-binding", "features", *paths],
            text=True,
            check=True,
            capture_output=True,
        )
        x, offsets, labels = [], [], []
        for row, text in zip(train, proc.stdout.splitlines(), strict=True):
            e = json.loads(text)
            ins, bits = pack_candidate_model_inputs(e, keys, tokens)
            cls = base.run(None, ins)[0][:, 0, :]
            x.append(cls[0] - cls[1])
            offsets.append(float((bits[0] - bits[1]) @ old))
            labels.append(e["candidates"][0] == row["ra_target"].split("::")[0])
    x, offsets, y = (
        np.asarray(x, dtype=float),
        np.array(offsets),
        np.array(labels, dtype=float),
    )

    def compute_residual_training_loss(w):
        z = offsets + x @ w
        return np.sum(np.logaddexp(0, z) - y * z) + w @ w / 2, x.T @ (expit(z) - y) + w

    fit = minimize(
        compute_residual_training_loss,
        np.zeros(x.shape[1]),
        method="L-BFGS-B",
        jac=True,
    )
    assert fit.success and np.linalg.norm(fit.x) > 0
    return fit.x.astype(np.float32)


def build_fused_binding_model(work, train):
    work.mkdir(parents=True, exist_ok=True)
    for local, remote, digest in ARTIFACTS:
        if not (work / local).exists():
            urllib.request.urlretrieve(BASE + remote, work / local)
        assert hashlib.sha256((work / local).read_bytes()).hexdigest() == digest
    raw = (PARENT / "weights.json").read_bytes()
    assert (
        hashlib.sha256(raw).hexdigest()
        == "938bf6aa68bef5b2a8655888b5c283db1675b26ddf3b5d8fb756d18f4451254b"
    )
    weights = json.loads(raw)
    keys = sorted(weights)
    old = np.array([weights[k] for k in keys], np.float32)
    vocab = (work / "vocab.txt").read_text().splitlines()
    tokens = {k: vocab.index(v) for k, v in WORDS.items()}
    if train:
        residual = train_synthetic_residual_head(work, keys, old, tokens)
        (HERE / "residual.json").write_text(json.dumps(residual.tolist()) + "\n")
    else:
        residual = np.array(
            json.loads((HERE / "residual.json").read_text()), np.float32
        )
    m = onnx.load(work / "base.onnx")
    g = m.graph
    g.input.append(
        h.make_tensor_value_info("lexical_features", T.FLOAT, [2, len(keys)])
    )
    for name, val in [
        ("cls_position", np.array(0, np.int64)),
        ("frozen_binding_weights", old[:, None]),
        ("trained_encoder_weights", residual[:, None]),
    ]:
        g.initializer.append(nh.from_array(val, name))
    g.node.extend(
        [
            h.make_node(
                "Gather", ["last_hidden_state", "cls_position"], ["binding_cls"], axis=1
            ),
            h.make_node(
                "MatMul", ["binding_cls", "trained_encoder_weights"], ["encoder_score"]
            ),
            h.make_node(
                "MatMul",
                ["lexical_features", "frozen_binding_weights"],
                ["lexical_score"],
            ),
            h.make_node("Add", ["encoder_score", "lexical_score"], ["binding_score"]),
        ]
    )
    del g.output[:]
    for name in ["binding_score", "lexical_score", "encoder_score"]:
        g.output.append(h.make_tensor_value_info(name, T.FLOAT, [2, 1]))
    onnx.checker.check_model(m)
    dest = work / "binding.onnx"
    onnx.save(m, dest)
    meta = {"features": keys, "tokens": tokens}
    (work / "meta.json").write_text(json.dumps(meta, sort_keys=True))
    # Check actual execution, both heads, and a lexical-input intervention.
    test = {
        "tokens": [["return", "M", "Q"], ["return", "O", "Q"]],
        "features": [keys[:5], []],
    }
    ins, bits = pack_candidate_model_inputs(test, keys, tokens)
    rt = open_cpu_model_session(dest)
    fused, lex, enc = rt.run(None, {**ins, "lexical_features": bits})
    cls = open_cpu_model_session(work / "base.onnx").run(None, ins)[0][:, 0, :]
    np.testing.assert_allclose(lex[:, 0], bits @ old, atol=1e-5)
    np.testing.assert_allclose(enc[:, 0], cls @ residual, atol=1e-4)
    np.testing.assert_allclose(fused, lex + enc, atol=1e-5)
    nolex = rt.run(None, {**ins, "lexical_features": bits * 0})[0]
    np.testing.assert_allclose(nolex, enc, atol=1e-5)
    assert not np.allclose(fused, nolex) and abs(enc[0, 0] - enc[1, 0]) > 1e-5
    assert dest.stat().st_size < 100_000_000
    print(dest, dest.stat().st_size, hashlib.sha256(dest.read_bytes()).hexdigest())


if __name__ == "__main__":
    build_fused_binding_model(Path(sys.argv[1]).resolve(), "--train" in sys.argv[2:])
