"""Compile an eight-address recurrent pointer circuit; no repository facts in weights."""
from pathlib import Path

import onnx
from onnx import TensorProto as T, helper as h

N = 8
nodes = []


def emit_single_output_node(op, args, name):
    nodes.append(h.make_node(op, args, [name], name=name))
    return name


def make_float_tensor_info(name, dims):
    return h.make_tensor_value_info(name, T.FLOAT, dims)


op = emit_single_output_node
params = [
    h.make_tensor("one", T.FLOAT, [], [1]),
    h.make_tensor("sum", T.FLOAT, [1, N], [1] * N),
    h.make_tensor("identity", T.FLOAT, [N, N],
                  [float(i == j) for i in range(N) for j in range(N)]),
]
# Rows are addresses. Definitions hold the pointer; aliases redirect it.
op("Mul", ["defs", "identity"], "terminal_rows")
op("Add", ["terminal_rows", "aliases"], "raw")
op("Transpose", ["sum"], "column")
op("MatMul", ["raw", "column"], "count")
op("Sub", ["count", "one"], "extra")
op("Relu", ["extra"], "too_many")
op("Sub", ["one", "too_many"], "allowed")
op("Relu", ["allowed"], "unique_or_empty")
op("Mul", ["raw", "unique_or_empty"], "rows")

q = "query"
for i in range(N):
    addr = op("Transpose", [q], f"addr_{i}")
    both = op("Add", [addr, "rows"], f"both_{i}")
    shifted = op("Sub", [both, "one"], f"shift_{i}")
    # AND(q[row], rows[row,col]) = ReLU(q + row_bit - 1).
    selected = op("Relu", [shifted], f"selected_{i}")
    q = op("MatMul", ["sum", selected], f"next_{i}")

# A cycle still has a pointer after eight steps, but is not a definition.
op("Mul", [q, "defs"], "answer")
graph = h.make_graph(nodes, "compiled_alias_machine", [
    make_float_tensor_info("defs", [1, N]),
    make_float_tensor_info("aliases", [N, N]),
    make_float_tensor_info("query", [1, N]),
], [make_float_tensor_info("answer", [1, N])], params)
model = h.make_model(graph, opset_imports=[h.make_opsetid("", 17)],
                     ir_version=8, producer_name="parseltongue")
onnx.checker.check_model(model, full_check=True)
onnx.save(model, Path(__file__).with_name("resolver.onnx"))
