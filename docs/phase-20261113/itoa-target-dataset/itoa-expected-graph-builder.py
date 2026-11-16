#!/usr/bin/env python3
"""Builds the itoa full-graph target dataset (LLM_PARSED_DRAFT, not production input).

Candidate identities remain symbolic. This script checks every cited source anchor against the
pinned source and writes the candidate nodes, edges and unresolved sites. It writes no
production rows.
Usage: itoa-expected-graph-builder.py --source-root PATH --pin-file PATH --output-directory PATH
"""
import argparse, collections, hashlib, importlib.util, json, pathlib, re, subprocess

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[2]
parser = argparse.ArgumentParser()
parser.add_argument("source", nargs="?")
parser.add_argument("--source-root")
parser.add_argument("--pin-file", default=str(ROOT / "docs/phase-20261111/phase-one-benchmark-pin.json"))
parser.add_argument("--output-directory", default=str(HERE))
args = parser.parse_args()
PIN_PATH = pathlib.Path(args.pin_file).resolve()
PIN = json.loads(PIN_PATH.read_text())
RULE_SPEC = importlib.util.spec_from_file_location("itoa_expected_graph_rules", HERE / "itoa-expected-graph-rules.py")
RULES = importlib.util.module_from_spec(RULE_SPEC)
RULE_SPEC.loader.exec_module(RULES)
SRC = pathlib.Path(args.source_root or args.source or PIN["benchmark_source_checkout_path"]).resolve()
OUT = pathlib.Path(args.output_directory).resolve()
OUT.mkdir(parents=True, exist_ok=True)
expected_digests = {row["path"]: row["sha256"] for row in PIN["benchmark_source_inventory_records"]}
expected_digests["Cargo.toml"] = PIN["benchmark_manifest_digest_sha256"]
RULES.verify_source_digests({path: (SRC / path).read_bytes() for path in expected_digests}, expected_digests)
source_commit = subprocess.check_output(["git", "-C", str(SRC), "rev-parse", "HEAD"], text=True).strip()
if source_commit != PIN["benchmark_source_commit_hash"]:
    raise SystemExit(f"source pin mismatch: expected {PIN['benchmark_source_commit_hash']}, got {source_commit}")
L, X = "src/lib.rs", "src/u128_ext.rs"
TEXT = {f: (SRC / f).read_text().split("\n") for f in (L, X)}
nodes, edges, impls, errors = {}, {}, [], []

def check(f, line, anchor):
    if anchor not in TEXT[f][line - 1]:
        errors.append(f"{f}:{line} lacks {anchor!r}")

def node(label, kind, vis, f, a, b, anchor, owner, reach=False, gen=None, flags=()):
    check(f, a, anchor)
    nodes[label] = dict(label=label, item_kind=kind, effective_visibility=vis,
        reachable_outside_crate=reach, file=f, start_line=a, end_line=b, owner=owner,
        generated_by=gen, source_anchor=anchor, flags=sorted(flags))

def edge(src, rel, dst, f, line, anchor, dkind="node", skind="node", via=None, gen=None, flags=(), n=1, start=None):
    # "-> ⟦Buffer⟧": the anchor is "-> ⟦Buffer⟧"; the site's exact text (part 7 section 6) is the marked part.
    offset, text = anchor.find("⟦"), None
    if offset >= 0:
        text = anchor[offset + 1:anchor.index("⟧")]; anchor = anchor.replace("⟦", "").replace("⟧", "")
    check(f, line, anchor)
    if n != 1 and TEXT[f][line - 1].count(anchor) != n:
        errors.append(f"{f}:{line} has {TEXT[f][line - 1].count(anchor)} of {anchor!r}, not {n}")
    e = edges.setdefault((src, rel, dst), dict(source=src, source_kind=skind, relation=rel,
        target=dst, target_kind=dkind, via_use_node=via, generated_by=gen, sites=[], flags=set()))
    # A site that starts with a macro parameter starts, after expansion, at the argument written in the macro call.
    first = dict(kind="invocation", line=gen["invocation_line"]) if gen and anchor.startswith("$") else dict(kind="definition", line=start or line)
    # 'start' is the line of the receiver's first token when a method call chain is written over several lines.
    if start is not None and not (start < line and TEXT[f][start - 1].strip() and not TEXT[f][start - 1].strip().startswith(".")):
        errors.append(f"{f}:{line} start line {start} is not an earlier receiver line")
    e["sites"].extend(dict(file=f, line=line, anchor=anchor, occurrence=i, first_token=first, site_text=text, site_text_offset=max(offset, 0))
                      for i in range(n))
    e["flags"].update(flags)

EXT, BI, OD7, OD8 = "external_identity_unverified", "builtin", "OD-7_containment", "OD-8_non_function_source"
USE_HINT, USE_MEM, USE_STR = "use core::hint", "use core::mem::{self, MaybeUninit}", "use core::str"
MU = "core::mem::MaybeUninit"
def ext(src, rel, dst, f, line, anchor, **k):
    k.setdefault("flags", ()); k["flags"] = tuple(k["flags"]) + (EXT,)
    edge(src, rel, dst, f, line, anchor, dkind="external", **k)
def prim(src, name, f, line, anchor, **k):
    edge(src, "uses_type", name, f, line, anchor, dkind=BI, **k)
# A method call whose receiver type itoa's own source does not prove. The row is expected without a target.
# The reason is the SPEC's (part 4): the path does not prove it leaves read source, so it is not external_source_missing.
WHY = {
    "write": "the receiver is an element of the buffer; indexing and the methods of its element type are declared in core",
    "get_unchecked": "the receiver is an array or a slice; its methods are declared in core",
    "len": "the receiver is a slice; its methods are declared in core",
    "as_mut_ptr": "the receiver is an array behind a raw-pointer cast chain; its methods are declared in core",
    "cast": "the receiver is the result of as_mut_ptr, which has no target",
    "rev": "the receiver is a range expression; its type and the trait method are declared in core",
    "ilog10": "the receiver is an associated const of a builtin type; the const's type is declared in core",
    "try_into": "the receiver is the result of a call into core",
    "unwrap": "the receiver is the result of try_into, which has no target",
    "expect": "the receiver is the result of try_into, which has no target",
}
def unres(src, method, f, line, anchor, gen=None, n=1, flags=(), start=None, why=None):
    if "." + method not in anchor.replace("⟦", "").replace("⟧", ""):
        errors.append(f"{f}:{line} anchor {anchor!r} does not call .{method}")
    edge(src, "calls", f"unresolved .{method}", f, line, anchor, dkind="unresolved", gen=gen, n=n, start=start)
    row = edges[(src, "calls", f"unresolved .{method}")]
    row.update(resolution_status="unresolved", unresolved_reason="trait_solver_needed", unresolved_why=why or WHY[method], callee_name=method)

# ---------------- hand-written nodes ----------------
AL, AX = "area:src/lib.rs", "area:src/u128_ext.rs"
node("itoa", "module", "public", L, 1, 466, "//!", "crate_root", reach=True, flags=["crate_root"])
node(AL, "file_public_area", "public", L, 1, 466, "//!", "crate_root")
node(AX, "file_public_area", "public", X, 1, 22, "#[cfg", "crate_root")
node("u128_ext", "module", "private", L, 54, 54, "mod u128_ext;", AL)
node(USE_HINT, "use_statement", "private", L, 56, 56, "use core::hint;", AL)
node(USE_MEM, "use_statement", "private", L, 57, 57, "use core::mem::{self, MaybeUninit};", AL)
node(USE_STR, "use_statement", "private", L, 58, 58, "use core::str;", AL)
node("Buffer", "struct", "public", L, 72, 74, "pub struct Buffer", AL, reach=True)
node("<Buffer as Default>::default", "associated_function", "public", L, 78, 80, "fn default()", "impl@76", reach=True)
node("<Buffer as Clone>::clone", "associated_function", "public", L, 88, 90, "fn clone(&self)", "impl@86", reach=True)
node("Buffer::new", "associated_function", "public", L, 98, 101, "pub fn new()", "impl@93", reach=True)
node("Buffer::format", "associated_function", "public", L, 106, 113, "pub fn format<I: Integer>", "impl@93", reach=True)
node("Integer", "trait", "public", L, 119, 123, "pub trait Integer: private::Sealed", AL, reach=True)
node("Integer::MAX_STR_LEN", "associated_constant", "public", L, 122, 122, "const MAX_STR_LEN: usize;", "Integer", reach=True)
node("private", "module", "private", L, 126, 133, "mod private {", AL)
node("private::Sealed", "trait", "public", L, 128, 132, "pub trait Sealed: Copy", "private")
node("private::Sealed::Buffer", "associated_type", "public", L, 130, 130, "type Buffer: 'static;", "private::Sealed")
node("private::Sealed::write", "associated_function", "public", L, 131, 131, "fn write(self, buf: &mut Self::Buffer) -> &str;", "private::Sealed")
node("DecimalPairs", "struct", "private", L, 219, 219, "struct DecimalPairs([u8; 200]);", AL)
node("DECIMAL_PAIRS", "static", "private", L, 222, 228, "static DECIMAL_PAIRS: DecimalPairs", AL)
node("divmod100", "function", "private", L, 231, 237, "fn divmod100(value: u32)", AL)
node("slice_buffer_to_str", "function", "private", L, 247, 253, "unsafe fn slice_buffer_to_str", AL)
node("Unsigned", "trait", "private", L, 255, 257, "trait Unsigned: Integer", AL)
node("Unsigned::fmt", "associated_function", "private", L, 256, 256, "fn fmt(self, buf: &mut Self::Buffer) -> usize;", "Unsigned")
node("<u128 as Unsigned>::fmt", "associated_function", "private", L, 336, 403, "fn fmt(self, buf: &mut Self::Buffer) -> usize {", "impl@334")
node("enc_16lsd", "function", "private", L, 408, 438, "fn enc_16lsd<const OFFSET: usize>", AL)
node("div_rem_1e16", "function", "private", L, 450, 466, "fn div_rem_1e16(n: u128)", AL)
node("u128_ext::mulhi", "function", "restricted_non_private", X, 7, 22, "pub(crate) fn mulhi", AX)

def impl(line, end, trait, self_ty, anchor, gen=None):
    check(L, line, anchor)
    impls.append(dict(file=L, start_line=line, end_line=end, trait=trait, self_type=self_ty,
                      source_anchor=anchor, generated_by=gen))
impl(76, 81, "Default", "Buffer", "impl Default for Buffer")
impl(83, 83, "Copy", "Buffer", "impl Copy for Buffer {}")
impl(86, 91, "Clone", "Buffer", "impl Clone for Buffer")
impl(93, 114, None, "Buffer", "impl Buffer {")
impl(334, 404, "Unsigned", "u128", "impl Unsigned for u128")

# ---------------- macro-generated nodes ----------------
PAIRS = [("i8", "u8", 185), ("i16", "u16", 186), ("i32", "u32", 187), ("i64", "u64", 188), ("i128", "u128", 189)]
SIZES = [("isize", "i64", 215), ("usize", "u64", 216)]
UNS = [("u8", 329), ("u16", 330), ("u32", 331), ("u64", 332)]
GEN = "macro_generated_file_assignment_open(I4)"
def G(macro, line): return dict(macro=macro, invocation_file=L, invocation_line=line)
def mx(t): return f"<{t} as Integer>::MAX_STR_LEN"
def sb(t): return f"<{t} as private::Sealed>::Buffer"
def sw(t): return f"<{t} as private::Sealed>::write"
def uf(t): return f"<{t} as Unsigned>::fmt"

for s, u, inv in PAIRS:
    check(L, inv, f"impl_Integer!({s}, {u});"); g = G("impl_Integer", inv)
    c = f"const _ #impl_Integer({s},{u})"
    node(c, "constant", "private", L, 137, 141, "const _: () = {", AL, gen=g, flags=[GEN, "unnamed_const"])
    for t, base, sgn in ((u, 143, "$Unsigned"), (s, 159, "$Signed")):
        impl(base, base + 2, "Integer", t, f"impl Integer for {sgn}", g)
        impl(base + 4, 157 if t == u else 181, "private::Sealed", t, f"impl private::Sealed for {sgn}", g)
        node(mx(t), "associated_constant", "public", L, base + 1, base + 1, "const MAX_STR_LEN: usize =", f"impl@{base}", reach=True, gen=g, flags=[GEN])
        node(sb(t), "associated_type", "public", L, base + 5, base + 5, "type Buffer = [MaybeUninit<u8>; Self::MAX_STR_LEN];", f"impl@{base+4}", gen=g, flags=[GEN, "reachability_needs_review"])
        node(sw(t), "associated_function", "public", L, base + 9, 156 if t == u else 180, "fn write(self, buf: &mut Self::Buffer) -> &str {", f"impl@{base+4}", gen=g, flags=[GEN, "reachability_needs_review"])
for t, p, inv in SIZES:
    check(L, inv, f"impl_Integer_size!({t} as {p} #[cfg(target_pointer_width = \"64\")]);"); g = G("impl_Integer_size", inv)
    impl(194, 196, "Integer", t, "impl Integer for $t", g); impl(199, 207, "private::Sealed", t, "impl private::Sealed for $t", g)
    node(mx(t), "associated_constant", "public", L, 195, 195, "const MAX_STR_LEN: usize = <$primitive as Integer>::MAX_STR_LEN;", "impl@194", reach=True, gen=g, flags=[GEN])
    node(sb(t), "associated_type", "public", L, 200, 200, "type Buffer = <$primitive as private::Sealed>::Buffer;", "impl@199", gen=g, flags=[GEN, "reachability_needs_review"])
    node(sw(t), "associated_function", "public", L, 204, 206, "fn write(self, buf: &mut Self::Buffer) -> &str {", "impl@199", gen=g, flags=[GEN, "reachability_needs_review"])
for u, inv in UNS:
    check(L, inv, f"impl_Unsigned!({u});"); g = G("impl_Unsigned", inv)
    impl(261, 325, "Unsigned", u, "impl Unsigned for $Unsigned", g)
    node(uf(u), "associated_function", "private", L, 263, 324, "fn fmt(self, buf: &mut Self::Buffer) -> usize {", "impl@261", gen=g, flags=[GEN])

# ---------------- macro definitions and macro calls (PRD: both are nodes; a call expands its definition and generates items) ----------------
MACRO_DEFS = (("impl_Integer", 135, 183), ("impl_Integer_size", 191, 209), ("impl_Unsigned", 259, 327))
for name, a, b in MACRO_DEFS:
    check(L, b, "}")
    node(f"macro_rules! {name}", "macro_definition", "private", L, a, b, f"macro_rules! {name} {{", AL)
# All six impl_Integer_size! calls are active: the cfg is an argument and applies to the generated impls.
SIZE_CALLS = [(t, p, w, 211 + i) for i, (t, p, w) in enumerate(
    (("isize", "i16", 16), ("usize", "u16", 16), ("isize", "i32", 32), ("usize", "u32", 32), ("isize", "i64", 64), ("usize", "u64", 64)))]
MACRO_CALLS = ([("impl_Integer", f"impl_Integer!({s}, {u})", inv) for s, u, inv in PAIRS]
    + [("impl_Integer_size", f'impl_Integer_size!({t} as {p} #[cfg(target_pointer_width = "{w}")])', inv) for t, p, w, inv in SIZE_CALLS]
    + [("impl_Unsigned", f"impl_Unsigned!({u})", inv) for u, inv in UNS])
generated_nodes = [n for n in nodes.values() if n["generated_by"]]
for macro, call, inv in MACRO_CALLS:
    node(call, "macro_call", "not_applicable", L, inv, inv, call + ";", AL)
    edge(call, "expands", f"macro_rules! {macro}", L, inv, macro + "!(")
    for n in generated_nodes:
        if n["generated_by"]["invocation_line"] == inv:
            edge(call, "generates", n["label"], L, inv, macro + "!(")

# ---------------- contains: decided file areas; inherent impl ownership stays flagged ----------------
inherent_impl_owners = {f"impl@{row['start_line']}" for row in impls if row["trait"] is None}
for n in list(nodes.values()):
    o = n["owner"]
    if o == "crate_root": continue
    container = AL if o.startswith("impl@") else o
    fl = [OD7] if o in inherent_impl_owners else []
    if n["generated_by"]: fl.append(GEN)
    edge(container, "contains", n["label"], n["file"], n["start_line"], "", gen=n["generated_by"], flags=fl)

# ---------------- imports ----------------
ext(USE_HINT, "imports", "core::hint", L, 56, "core::hint")
ext(USE_MEM, "imports", "core::mem", L, 57, "mem::{⟦self⟧")
ext(USE_MEM, "imports", MU, L, 57, "⟦MaybeUninit⟧}")
ext(USE_STR, "imports", "core::str", L, 58, "core::str")

# ---------------- implements ----------------
ext("Buffer", "implements", "Default", L, 76, "impl Default for Buffer")
ext("Buffer", "implements", "Copy", L, 83, "impl Copy for Buffer")
ext("Buffer", "implements", "Clone", L, 86, "impl Clone for Buffer")
for i in impls:
    if i["self_type"] != "Buffer" and i["trait"]:
        edge(i["self_type"], "implements", i["trait"], L, i["start_line"], "impl ", skind=BI, gen=i["generated_by"], flags=["builtin_source_never_seeds_interface_edge"])

# ---------------- uses_type: signatures, fields, bounds, supertraits ----------------
edge("Buffer", "uses_type", MU, L, 73, "MaybeUninit<u8>", dkind="external", via=USE_MEM, flags=[EXT]); prim("Buffer", "u8", L, 73, "<⟦u8⟧>")
edge("Buffer", "reads", mx("i128"), L, 73, "i128::MAX_STR_LEN", flags=[OD8])
edge("<Buffer as Default>::default", "uses_type", "Buffer", L, 78, "-> ⟦Buffer⟧")
edge("<Buffer as Clone>::clone", "uses_type", "Buffer", L, 88, "-> ⟦Self⟧", flags=["Self_resolved_to_impl_self_type"])
edge("Buffer::new", "uses_type", "Buffer", L, 98, "-> ⟦Buffer⟧")
edge("Buffer::format", "uses_type", "Integer", L, 106, "<I: ⟦Integer⟧>"); prim("Buffer::format", "str", L, 106, "-> &⟦str⟧")
edge("Buffer::format", "uses_type", "private::Sealed::Buffer", L, 107, "cast::<⟦I::Buffer⟧>()", flags=["body_type_mention"])
edge("Integer", "uses_type", "private::Sealed", L, 119, ": ⟦private::Sealed⟧"); prim("Integer::MAX_STR_LEN", "usize", L, 122, ": ⟦usize⟧")
ext("private::Sealed", "uses_type", "Copy", L, 128, ": ⟦Copy⟧", flags=["via_prelude"])
edge("private::Sealed::write", "uses_type", "private::Sealed::Buffer", L, 131, "Self::Buffer"); prim("private::Sealed::write", "str", L, 131, "-> &⟦str⟧")
prim("DecimalPairs", "u8", L, 219, "[⟦u8⟧; 200]"); edge("DECIMAL_PAIRS", "uses_type", "DecimalPairs", L, 222, ": ⟦DecimalPairs⟧", flags=[OD8])
edge("DECIMAL_PAIRS", "calls", "DecimalPairs", L, 222, "= DecimalPairs(", flags=["tuple_struct_constructor"])
prim("divmod100", "u32", L, 231, "u32", n=3)
edge("slice_buffer_to_str", "uses_type", MU, L, 247, "MaybeUninit<u8>", dkind="external", via=USE_MEM, flags=[EXT])
edge("slice_buffer_to_str", "uses_type", MU, L, 252, "*const [⟦MaybeUninit<u8>⟧]", dkind="external", via=USE_MEM, flags=[EXT, "body_type_mention"])
prim("slice_buffer_to_str", "u8", L, 247, "<⟦u8⟧>"); prim("slice_buffer_to_str", "usize", L, 247, ": ⟦usize⟧"); prim("slice_buffer_to_str", "str", L, 247, "-> &⟦str⟧")
# ---------------- uses_type: types written in an expression (REQ-RUST-018.3 R10) ----------------
BT = ["body_type_mention"]
SELF_BT = BT + ["Self_resolved_to_impl_self_type"]
for ln in (8, 9, 10, 11, 18): prim("u128_ext::mulhi", "u64", X, ln, "as ⟦u64⟧", flags=BT)
prim("Buffer::new", "u8", L, 99, "::<⟦u8⟧>", flags=BT)
prim("slice_buffer_to_str", "u8", L, 252, "u8", n=2, flags=BT)
for s, u, inv in PAIRS:
    g = G("impl_Integer", inv)
    prim(mx(u), "usize", L, 144, "as ⟦usize⟧", gen=g, flags=BT); prim(mx(s), "usize", L, 160, "as ⟦usize⟧", gen=g, flags=BT)
for t, p, inv in SIZES:
    g = G("impl_Integer_size", inv)
    prim(mx(t), p, L, 195, "⟦$primitive⟧ as Integer", gen=g, flags=BT)
    edge(mx(t), "uses_type", "Integer", L, 195, "as ⟦Integer⟧>", gen=g, flags=BT)
    prim(sw(t), p, L, 205, "⟦$primitive⟧).write(buf)", gen=g, flags=BT)
for u, inv in UNS:
    g = G("impl_Unsigned", inv); f = uf(u)
    prim(f, u, L, 271, "::<⟦Self⟧>", gen=g, flags=SELF_BT); prim(f, u, L, 280, ": ⟦Self⟧", gen=g, flags=SELF_BT); prim(f, u, L, 303, "as ⟦Self⟧", gen=g, flags=SELF_BT)
    for ln in (285, 302): prim(f, "u32", L, ln, "as ⟦u32⟧", gen=g, flags=BT)
    for ln in (288, 290, 292, 294, 306, 308): prim(f, "usize", L, ln, "as ⟦usize⟧", gen=g, flags=BT)
    prim(f, "u8", L, 318, "as ⟦u8⟧", gen=g, flags=BT)
FU = "<u128 as Unsigned>::fmt"
for ln, p in ((360, "u64"), (371, "u32"), (384, "u32"), (385, "u64"), (398, "u8")): prim(FU, p, L, ln, "as ⟦" + p + "⟧", flags=BT)
for ln in (373, 374, 375, 376, 387, 388): prim(FU, "usize", L, ln, "as ⟦usize⟧", flags=BT)
for ln in (417, 431): prim("enc_16lsd", "u32", L, ln, "as ⟦u32⟧", flags=BT)
for ln in (420, 422, 424, 426, 433, 434, 435, 436): prim("enc_16lsd", "usize", L, ln, "as ⟦usize⟧", flags=BT)
for ln in (454, 465): prim("div_rem_1e16", "u64", L, ln, "as ⟦u64⟧", flags=BT)

edge("Unsigned", "uses_type", "Integer", L, 255, ": ⟦Integer⟧")
edge("Unsigned::fmt", "uses_type", "private::Sealed::Buffer", L, 256, "Self::Buffer", flags=["through_supertraits_Integer_Sealed"]); prim("Unsigned::fmt", "usize", L, 256, "-> ⟦usize⟧")
edge("enc_16lsd", "uses_type", MU, L, 408, "MaybeUninit<u8>", dkind="external", via=USE_MEM, flags=[EXT])
for p in ("usize", "u8", "u64"): prim("enc_16lsd", p, L, 408, p)
prim("div_rem_1e16", "u128", L, 450, "u128", n=2); prim("div_rem_1e16", "u64", L, 450, "u64")
prim("u128_ext::mulhi", "u128", X, 7, "u128", n=3)
edge("<u128 as Unsigned>::fmt", "uses_type", sb("u128"), L, 336, "Self::Buffer"); prim("<u128 as Unsigned>::fmt", "usize", L, 336, "-> ⟦usize⟧")

# ---------------- calls and reads in hand-written bodies ----------------
edge("<Buffer as Default>::default", "calls", "Buffer::new", L, 79, "Buffer::new()")
edge("<Buffer as Clone>::clone", "calls", "Buffer::new", L, 89, "Buffer::new()")
ext("Buffer::new", "calls", MU + "::uninit", L, 99, "MaybeUninit::<u8>::uninit()", via=USE_MEM)
edge("Buffer::new", "reads", mx("i128"), L, 99, "i128::MAX_STR_LEN")
unres("Buffer::format", "as_mut_ptr", L, 107, ".as_mut_ptr()")
unres("Buffer::format", "cast", L, 107, ".cast::<")
edge("Buffer::format", "calls", "private::Sealed::write", L, 108, "i.write(", flags=["generic_dispatch_one_edge_through_bound_I:Integer:Sealed"])
# 'string' is the value of i.write(..); private::Sealed::write is declared '-> &str', so the receiver is a '&str'.
# No target (part 7 Q4): a method on a '&str' value is either a '&self' method of 'str' or a by-value method declared
# for '&str' (as 'clone' and 'into' are). Which one 'len' is, is declared in core. Same as 'buf.len()' on a '&[T]' parameter.
unres("Buffer::format", "len", L, 109, "string.len()",
      why="the receiver is a '&str' (declared return type of private::Sealed::write); whether 'len' is declared for 'str' or for '&str' is in core")
edge("Buffer::format", "reads", "Integer::MAX_STR_LEN", L, 109, "I::MAX_STR_LEN", flags=["generic_trait_const"])
ext("Buffer::format", "calls", "core::hint::unreachable_unchecked", L, 110, "hint::unreachable_unchecked()", via=USE_HINT)
unres("slice_buffer_to_str", "get_unchecked", L, 249, "buf.get_unchecked(")
ext("slice_buffer_to_str", "calls", "core::str::from_utf8_unchecked", L, 252, "str::from_utf8_unchecked(", via=USE_STR)
edge("div_rem_1e16", "calls", "u128_ext::mulhi", L, 463, "u128_ext::mulhi(n, M_HIGH)")
for ln, count in ((14, 2), (15, 2), (19, 3), (21, 2)):
    ext("u128_ext::mulhi", "calls", "u128::from", X, ln, "u128::from(", n=count, flags=["which_From_impl_not_claimed"])

def table_sites(src, lines, f=L, chained=()):
    for ln in lines:
        unres(src, "write", f, ln, ".write(", start=ln - 1 if ln in chained else None)
def pairs_sites(src, lines):
    for ln in lines:
        edge(src, "reads", "DECIMAL_PAIRS", L, ln, "⟦DECIMAL_PAIRS⟧.0")
        unres(src, "get_unchecked", L, ln, ".get_unchecked(")

F = "<u128 as Unsigned>::fmt"
unres(F, "len", L, 340, "buf.len()")
table_sites(F, (341, 373, 374, 375, 376, 387, 388, 399)); pairs_sites(F, (373, 374, 375, 376, 387, 388))
for ln in (345, 353): edge(F, "calls", "div_rem_1e16", L, ln, "div_rem_1e16(")
for ln in (347, 350, 355, 358, 360): edge(F, "reads", mx("u128"), L, ln, "u128::MAX_STR_LEN")
for ln in (350, 358): edge(F, "calls", "enc_16lsd", L, ln, "enc_16lsd::<")
for ln in (371, 384): edge(F, "calls", "divmod100", L, ln, "divmod100(")
E = "enc_16lsd"
unres(E, "rev", L, 413, ".rev()", flags=["trait_method"])
for ln in (417, 431): edge(E, "calls", "divmod100", L, ln, "divmod100(")
table_sites(E, (420, 422, 424, 426, 433, 434, 435, 436), chained=(420, 422, 424, 426)); pairs_sites(E, (420, 422, 424, 426, 433, 434, 435, 436))

# ---------------- macro-generated bodies ----------------
for s, u, inv in PAIRS:
    g = G("impl_Integer", inv); c = f"const _ #impl_Integer({s},{u})"
    for ln, t, p, k in ((138, s, "$Signed", "MIN"), (139, u, "$Unsigned", "MIN"), (140, s, "$Signed", "BITS"), (140, u, "$Unsigned", "BITS")):
        ext(c, "reads", f"{t}::{k}", L, ln, f"{p}::{k}", gen=g, flags=[OD8])
    for t, ln, sgn in ((u, 144, "$Unsigned"), (s, 160, "$Signed")):
        prim(mx(t), "usize", L, ln, ": ⟦usize⟧", gen=g)
        ext(mx(t), "reads", f"{t}::MAX", L, ln, f"{sgn}::MAX", gen=g, flags=[OD8])
        unres(mx(t), "ilog10", L, ln, f"{sgn}::MAX.ilog10()", gen=g, flags=[OD8])
        bl = 148 if t == u else 164
        edge(sb(t), "uses_type", MU, L, bl, "MaybeUninit<u8>", dkind="external", via=USE_MEM, gen=g, flags=[EXT]); prim(sb(t), "u8", L, bl, "<⟦u8⟧>", gen=g)
        edge(sb(t), "reads", mx(t), L, bl, "Self::MAX_STR_LEN", gen=g, flags=[OD8])
        wl = 152 if t == u else 168
        edge(sw(t), "uses_type", sb(t), L, wl, "Self::Buffer", gen=g); prim(sw(t), "str", L, wl, "-> &⟦str⟧", gen=g)
    edge(sw(u), "calls", uf(u), L, 153, "Unsigned::fmt(self, buf)", gen=g, flags=["concrete_impl_chosen_from_argument_type"])
    edge(sw(u), "calls", "slice_buffer_to_str", L, 155, "slice_buffer_to_str(buf, offset)", gen=g)
    edge(sw(s), "reads", mx(s), L, 169, "Self::MAX_STR_LEN", gen=g)
    edge(sw(s), "reads", mx(u), L, 169, "$Unsigned::MAX_STR_LEN", gen=g)
    # The impl member cannot be chosen: the first argument's type is the return type of unsigned_abs, declared in core.
    edge(sw(s), "calls", "Unsigned::fmt", L, 170, "Unsigned::fmt(", gen=g, flags=["trait_declaration_impl_member_needs_core_return_type_of_unsigned_abs"])
    # 'self' is taken by value in an impl whose self type is the builtin written in the macro call.
    ext(sw(s), "calls", f"{s}::unsigned_abs", L, 171, "self.unsigned_abs()", gen=g, flags=["receiver_type_is_self_type_of_impl"])
    unres(sw(s), "try_into", L, 172, ".try_into()", gen=g, flags=["trait_method"])
    unres(sw(s), "unwrap", L, 172, ".unwrap()", gen=g)
    unres(sw(s), "write", L, 176, ".write(b'-')", gen=g)
    edge(sw(s), "calls", "slice_buffer_to_str", L, 179, "slice_buffer_to_str(buf, offset)", gen=g)
for t, p, inv in SIZES:
    g = G("impl_Integer_size", inv)
    prim(mx(t), "usize", L, 195, ": ⟦usize⟧", gen=g)
    edge(mx(t), "reads", mx(p), L, 195, "<$primitive as Integer>::MAX_STR_LEN", gen=g, flags=[OD8])
    edge(sb(t), "uses_type", sb(p), L, 200, "<$primitive as private::Sealed>::Buffer", gen=g, flags=[OD8])
    # The same path names the type $primitive (written in the macro call) and the trait private::Sealed.
    prim(sb(t), p, L, 200, "⟦$primitive⟧ as private::Sealed", gen=g)
    edge(sb(t), "uses_type", "private::Sealed", L, 200, "⟦private::Sealed⟧>::Buffer", gen=g, flags=[OD8])
    edge(sw(t), "uses_type", sb(t), L, 204, "Self::Buffer", gen=g); prim(sw(t), "str", L, 204, "-> &⟦str⟧", gen=g)
    edge(sw(t), "calls", sw(p), L, 205, "(self as $primitive).write(buf)", gen=g, flags=["concrete_receiver_from_cast"])
for u, inv in UNS:
    g = G("impl_Unsigned", inv); f = uf(u)
    edge(f, "uses_type", sb(u), L, 263, "Self::Buffer", gen=g); prim(f, "usize", L, 263, "-> ⟦usize⟧", gen=g)
    unres(f, "len", L, 265, "buf.len()", gen=g)
    ext(f, "calls", "core::mem::size_of", L, 271, "mem::size_of::<Self>()", via=USE_MEM, gen=g)
    for ln in (274, 281): unres(f, "try_into", L, ln, ".try_into()", gen=g, flags=["trait_method"], start=ln - 1)
    for ln in (275, 282): unres(f, "expect", L, ln, ".expect(", gen=g, start=ln - 2)
    for ln in (285, 302): edge(f, "calls", "divmod100", L, ln, "divmod100(", gen=g)
    for ln in (288, 290, 292, 294, 306, 308, 319): unres(f, "write", L, ln, ".write(", gen=g, start=None if ln == 319 else ln - 1)
    for ln in (288, 290, 292, 294, 306, 308):
        edge(f, "reads", "DECIMAL_PAIRS", L, ln, "⟦DECIMAL_PAIRS⟧.0", gen=g)
        unres(f, "get_unchecked", L, ln, ".get_unchecked(", gen=g)

# ---------------- sites expected to stay unresolved or inactive ----------------
unresolved = []
def site(f, line, anchor, kind, reason, count=1):
    check(f, line, anchor); unresolved.append(dict(file=f, line=line, anchor=anchor, kind=kind, reason=reason, occurrences=count))
for ln in (138, 139, 140): site(L, ln, "assert!(", "body_macro", "depends_on_AD-1: core macro; unresolved(external_source_missing) without sysroot", 5)
site(L, 232, "debug_assert!(", "body_macro", "depends_on_AD-1: core macro; unresolved(external_source_missing) without sysroot")
site(L, 60, "use no_panic::no_panic;", "use", "cfg_inactive(feature no-panic off)"); site(X, 2, "use no_panic::no_panic;", "use", "cfg_inactive(feature no-panic off)")
for ln in (211, 212, 213, 214): site(L, ln, "impl_Integer_size!(", "generated_impls", "the call is active; the two impls it generates are cfg_inactive(target_pointer_width is 64) and are not nodes")

# ---------------- source evidence and candidate classifications ----------------
for edge_row in edges.values():
    if edge_row["source_kind"] == "node" and edge_row["source"] not in nodes:
        errors.append(f"unknown source {edge_row['source']}")
    if edge_row["target_kind"] == "node" and edge_row["target"] not in nodes:
        errors.append(f"unknown target {edge_row['target']}")
    edge_row["flags"] = sorted(edge_row["flags"])
if errors:
    raise SystemExit("\n".join(errors))

source_hashes = {path: hashlib.sha256((SRC / path).read_bytes()).hexdigest() for path in (L, X)}

def source_evidence(file, line, anchor):
    raw_lines = (SRC / file).read_bytes().splitlines(keepends=True)
    line_bytes = raw_lines[line - 1].rstrip(b"\r\n")
    needle = anchor.encode("utf-8")
    hits, cursor = [], 0
    while needle and (index := line_bytes.find(needle, cursor)) >= 0:
        hits.append(index); cursor = index + max(1, len(needle))
    start = sum(len(value) for value in raw_lines[:line - 1])
    exact = len(hits) == 1
    hit = hits[0] if exact else 0
    return {"source_path": file, "source_sha256": source_hashes[file], "line": line,
            "byte_range": [start + hit, start + hit + len(needle)] if exact else None,
            "anchor_text": anchor, "range_status": "EXACT_ANCHOR" if exact else "AMBIGUOUS_OR_EMPTY"}

RAW = {f: (SRC / f).read_bytes() for f in (L, X)}
MACRO_PARAMS = {"impl_Integer": ("$Signed", "$Unsigned"), "impl_Integer_size": ("$t", "$primitive"), "impl_Unsigned": ("$Unsigned",)}
range_errors = []

def line_span(f, line):
    lines = RAW[f].splitlines(keepends=True)
    start = sum(len(v) for v in lines[:line - 1])
    return start, start + len(lines[line - 1])

def find_all(f, needle, lo, hi):
    out, at = [], RAW[f].find(needle.encode(), lo)
    while 0 <= at < hi:
        out.append(at); at = RAW[f].find(needle.encode(), at + 1)
    return out

ID = set(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$")
WS = set(b" \n\t\r")

def ident_back(f, p):
    while p > 0 and RAW[f][p - 1] in ID: p -= 1
    return p

def group_back(f, p):
    """RAW[f][p-1] closes a group; the index of the token that opens it."""
    close = RAW[f][p - 1]; opener = {41: 40, 93: 91, 62: 60}[close]; depth, i = 0, p - 1
    while True:
        c = RAW[f][i]
        if c == close and not (close == 62 and RAW[f][i - 1] == 45): depth += 1
        elif c == opener: depth -= 1
        if depth == 0: return i
        i -= 1

def atom_back(f, p):
    """The start of the postfix operand that ends at p: a name, a literal, or a group with what it is applied to."""
    c = RAW[f][p - 1]
    if c in b")]":
        p = group_back(f, p)
        if RAW[f][p - 1] in ID or RAW[f][p - 1] in b")]": return atom_back(f, p)
        if RAW[f][p - 1] == 62:
            lt = group_back(f, p)
            if RAW[f][lt - 2:lt] != b"::": raise ValueError("generic arguments without a turbofish")
            return ident_back(f, lt - 2)
        return p
    if c in ID: return ident_back(f, p)
    raise ValueError(f"no operand ends at byte {p}")

def call_start(f, p):
    """E1: the first token of the call expression whose callee name starts at p. A method call starts at its receiver;
    a prefix operator before the receiver is not part of the call. A path call starts at the first segment of its path."""
    while True:
        if RAW[f][p - 2:p] == b"::":
            q = p - 2
            if RAW[f][q - 1] == 62:
                lt = group_back(f, q)
                if RAW[f][lt - 2:lt] != b"::": return lt
                p = ident_back(f, lt - 2)
            else:
                p = ident_back(f, q)
            continue
        # The dot is written directly before the name; a receiver may end on an earlier line.
        if RAW[f][p - 1] == 46 and RAW[f][p - 2] != 46:
            q = p - 1
            while RAW[f][q - 1] in WS: q -= 1
            p = atom_back(f, q)
            continue
        return p

def macro_argument(f, gen, name):
    """The byte range, in the macro call, of the argument substituted for the macro parameter 'name'."""
    lo, hi = line_span(f, gen["invocation_line"])
    line = RAW[f][lo:hi].decode(); inside = line.index("(") + 1
    words = list(re.finditer(r"[A-Za-z0-9_]+", line[inside:]))
    index = MACRO_PARAMS[gen["macro"]].index(name)
    word = words[index * 2 if gen["macro"] == "impl_Integer_size" else index]
    return [lo + inside + word.start(), lo + inside + word.end()]

def exact_site_range(rel, site, gen, callee):
    """The site's byte range by part 7 section 6 (E1 to E6), from the pinned bytes; None with an error when not determined."""
    f = site["file"]; lo, hi = line_span(f, site["line"])
    hits = find_all(f, site["anchor"], lo, hi)
    where = f"{f}:{site['line']} {rel} {site['anchor']!r}"
    if site["occurrence"] >= len(hits):
        range_errors.append(f"{where}: anchor occurrence {site['occurrence']} not on the line"); return None
    at = hits[site["occurrence"]]; a, b = at, at + len(site["anchor"].encode())
    if rel in ("expands", "generates"):
        b = RAW[f].index(b";", a) + 1
    elif rel == "calls":
        found = re.search(r"(?<![A-Za-z0-9_$])" + re.escape(callee) + r"(?![A-Za-z0-9_])", site["anchor"])
        if not found:
            range_errors.append(f"{where}: callee name {callee!r} not in the anchor"); return None
        name_at = at + len(site["anchor"][:found.start()].encode())
        try:
            a = call_start(f, name_at)
        except ValueError as error:
            range_errors.append(f"{where}: {error}"); return None
        depth, i = 0, RAW[f].index(b"(", name_at)
        while True:
            depth += {40: 1, 41: -1}.get(RAW[f][i], 0)
            if depth == 0: break
            i += 1
        b = i + 1
    elif site["site_text"] is not None:
        a = at + len(site["anchor"][:site["site_text_offset"]].encode()); b = a + len(site["site_text"].encode())
    text = RAW[f][a:b].decode()
    params = sorted(MACRO_PARAMS[gen["macro"]], key=len, reverse=True) if gen else ()
    first_param = next((p for p in params if re.match(re.escape(p) + r"(?![A-Za-z0-9_])", text)), None)
    last_param = next((p for p in params if text.endswith(p)), None)
    first = macro_argument(f, gen, first_param) if first_param else None
    last = macro_argument(f, gen, last_param) if last_param else None
    return dict(first_token_kind="invocation" if first else "definition", first_token_start=first[0] if first else a,
                last_token_kind="invocation" if last else "definition", last_token_end=last[1] if last else b, text=text)

node_rows = []
for row in nodes.values():
    candidate = dict(row)
    candidate["source_evidence"] = source_evidence(row["file"], row["start_line"], row["source_anchor"])
    candidate["file_assignment_status"] = "PROPOSED_UNVERIFIED_MACRO_OWNERSHIP" if row.get("generated_by") else "SOURCE_FILE_CANDIDATE"
    node_rows.append(candidate)
edge_rows = []
for row in edges.values():
    candidate = dict(row)
    candidate["source_evidence_sites"] = [source_evidence(site["file"], site["line"], site["anchor"])
                                         for site in row["sites"]]
    callee = row.get("callee_name") or row["target"].rsplit("::", 1)[-1]
    candidate["site_exact_ranges"] = [exact_site_range(row["relation"], site, row["generated_by"], callee) for site in row["sites"]] \
        if row["relation"] not in ("contains", "implements") else None
    candidate["source_site_evidence_status"] = "EXACT" if candidate["source_evidence_sites"] and all(
        site["range_status"] == "EXACT_ANCHOR" for site in candidate["source_evidence_sites"]) else "AMBIGUOUS_OR_EMPTY"
    candidate["evidence_key"] = "|".join(f"{site['source_path']}:{site['line']}:{site['byte_range']}" for site in candidate["source_evidence_sites"])
    candidate["evidence_ref_kind"] = "SYMBOLIC_SOURCE_SITE_REF_NOT_CANONICAL"
    edge_rows.append(candidate)
# One node per name outside itoa that an edge ends at, and one per builtin type an edge starts from or ends at (PRD).
def external_name_kind(name):
    if name.startswith("core::"): return "crate_rooted_path"
    return "builtin_member" if name.split("::")[0] in RULES.BUILTIN_PRIMITIVES and "::" in name else "prelude_name"
endpoints = {}
for row in edge_rows:
    for label, kind in ((row["source"], row["source_kind"]), (row["target"], row["target_kind"])):
        if kind in ("external", BI):
            endpoints[label] = kind
for label, kind in sorted(endpoints.items()):
    node_rows.append(dict(label=label, item_kind=None, node_category="external_name" if kind == "external" else "builtin_type",
        external_name_kind=external_name_kind(label) if kind == "external" else None, source_read=False,
        effective_visibility=None, reachable_outside_crate=None, file="", start_line=0, end_line=0, owner=None,
        generated_by=None, source_anchor=None, flags=[], source_evidence=None, file_assignment_status="NO_SOURCE_FILE_NOT_READ"))
node_rows.sort(key=lambda row: (row["file"], row["start_line"], row["label"]))
edge_rows.sort(key=lambda row: (row["source"], row["relation"], row["target"]))
for row in impls:
    row["source_evidence"] = source_evidence(row["file"], row["start_line"], row["source_anchor"])
for row in unresolved:
    row["source_evidence"] = source_evidence(row["file"], row["line"], row["anchor"])
    row.update(RULES.classify_diagnostic_site({"reason_candidate": row["reason"], "reason_verified": False}))

if range_errors:
    raise SystemExit("\n".join(range_errors))
classified = RULES.classify_candidates(node_rows, edge_rows)
eligible_nodes = classified["eligible_nodes"]

def dump(name, value):
    (OUT / name).write_text(RULES.serialize_dataset(value))

candidate_provenance = {"status": "LLM_PARSED_DRAFT", "author": "Devin",
                        "basis": "PINNED_SOURCE_READING",
                        "independent_compiler_oracle_verification": "NOT_FROZEN"}
candidate_header = {"artifact_status": "LLM_PARSED_DRAFT_NOT_FROZEN",
                    "candidate_provenance": candidate_provenance, "candidate_rows_preserved": True,
                    "source_coverage": {"whole_source_coverage_claimed": False,
                        "known_uncovered_constructs": ["operators", "literals", "repeated_same_line_sites"]},
                    "canonical_identity_status": "SYMBOLIC_NONCANONICAL", "production_input": False,
                    "source_commit": source_commit, "source_pin_file": PIN_PATH.name}
dump("itoa-expected-graph-nodes.json", {**candidate_header, "row_count": len(node_rows), "rows": node_rows})
dump("itoa-expected-graph-edges.json", {**candidate_header, "row_count": len(edge_rows),
                                     "relation_kind_counts": RULES.relation_counts(edge_rows), "rows": edge_rows})
dump("itoa-unblocked-rows-subset.json", {"artifact_status": "ELIGIBLE_LOCAL_ONLY_DIAGNOSTIC",
     "candidate_provenance": candidate_provenance,
     "nodes": eligible_nodes, "edges": classified["eligible_edges"],
     "counts": {"candidate_nodes": len(node_rows), "eligible_nodes": len(eligible_nodes),
                "candidate_edges": len(edge_rows), "eligible_edges": len(classified["eligible_edges"])}})
blocked_rows = {"candidate_provenance": candidate_provenance,
                "nodes": classified["blocked_nodes"], "edges": classified["blocked_edges"],
                "unresolved_or_inactive_sites": unresolved,
                "blocker_counts": dict(sorted(collections.Counter(blocker for row in
                    classified["blocked_nodes"] + classified["blocked_edges"] for blocker in row["blockers"]).items()))}
dump("itoa-blocked-rows-reasons.json", blocked_rows)
dump("itoa-expected-impl-blocks.json", {"artifact_status": "LLM_PARSED_DRAFT_NOT_PRODUCTION",
     "candidate_provenance": candidate_provenance, "rows": impls})
dump("itoa-expected-unresolved-sites.json", {"artifact_status": "BLOCKED_DIAGNOSTIC_EXPECTATIONS", "rows": unresolved})

counts = {"candidate_nodes": len(node_rows), "eligible_nodes": len(eligible_nodes), "blocked_nodes": len(classified["blocked_nodes"]),
          "candidate_edges": len(edge_rows), "eligible_edges": len(classified["eligible_edges"]), "blocked_edges": len(classified["blocked_edges"]),
          "impl_block_records": len(impls), "unresolved_or_inactive_sites": len(unresolved)}
target_kind_counts = dict(sorted(collections.Counter(row["target_kind"] for row in edge_rows).items()))
summary = {"artifact_status": "LLM_PARSED_DRAFT_NOT_FROZEN",
           "candidate_provenance": candidate_provenance, "counts": counts,
           "relation_kind_counts": RULES.relation_counts(edge_rows), "target_kind_counts": target_kind_counts,
           "blocker_counts": blocked_rows["blocker_counts"],
           "source_coverage": {"whole_source_coverage_claimed": False,
               "known_uncovered_constructs": ["operators", "literals", "repeated_same_line_sites"]},
           "open_decisions": ["OD-2", "OD-7", "OD-8"], "production_comparison_status": "NOT_RUN"}
dump("itoa-expected-graph-summary.json", summary)
md = "# itoa expected full item graph: summary\n\n"
md += "**Status: LLM_PARSED_DRAFT_NOT_FROZEN.** Written by Devin from reading the pinned source. Not reviewed, not frozen, never a production input. Identities are symbolic labels, not production keys.\n\n"
md += "Every cited source anchor is checked against the pinned bytes. Whole-source coverage is not claimed: operators, literals and repeated same-line sites are not covered. A type written in an expression is covered (REQ-RUST-018.3 R10). Rows that depend on OD-2, OD-7 or OD-8 are marked blocked.\n\n"
md += "```text\n" + "\n".join(f"{key}: {value}" for key, value in counts.items()) + "\n"
md += "\n".join(f"relation {key}: {value}" for key, value in summary["relation_kind_counts"].items()) + "\n"
md += "\n".join(f"target {key}: {value}" for key, value in target_kind_counts.items()) + "\n"
md += f"source commit: {source_commit}\n```\n"
(OUT / "itoa-expected-graph-summary.md").write_text(md)
print(md)
