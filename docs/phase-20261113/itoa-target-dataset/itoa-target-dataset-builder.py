#!/usr/bin/env python3
"""Builds the itoa full-graph target dataset (LLM_PARSED_DRAFT, not production input).

Candidate identities remain symbolic. This script checks every cited source anchor against the
pinned source and writes the candidate nodes, edges and unresolved sites. It writes no
production rows.
Usage: itoa-target-dataset-builder.py --source-root PATH --pin-file PATH --output-directory PATH
"""
import argparse, collections, hashlib, importlib.util, json, pathlib, subprocess

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
RULE_SPEC = importlib.util.spec_from_file_location("itoa_reference_graph_rules", HERE / "itoa-reference-graph-rules.py")
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

def edge(src, rel, dst, f, line, anchor, dkind="node", skind="node", via=None, gen=None, flags=()):
    check(f, line, anchor)
    e = edges.setdefault((src, rel, dst), dict(source=src, source_kind=skind, relation=rel,
        target=dst, target_kind=dkind, via_use_node=via, generated_by=gen, sites=[], flags=set()))
    e["sites"].append(dict(file=f, line=line, anchor=anchor))
    e["flags"].update(flags)

EXT, BI, OD7, OD8 = "external_identity_unverified", "builtin", "OD-7_containment", "OD-8_non_function_source"
USE_HINT, USE_MEM, USE_STR = "use core::hint", "use core::mem::{self, MaybeUninit}", "use core::str"
MU = "core::mem::MaybeUninit"
def ext(src, rel, dst, f, line, anchor, **k):
    k.setdefault("flags", ()); k["flags"] = tuple(k["flags"]) + (EXT,)
    edge(src, rel, dst, f, line, anchor, dkind="external", **k)
def prim(src, name, f, line, anchor, **k):
    edge(src, "uses_type", name, f, line, anchor, dkind=BI, **k)

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
impl(76, 81, "core::default::Default", "Buffer", "impl Default for Buffer")
impl(83, 83, "core::marker::Copy", "Buffer", "impl Copy for Buffer {}")
impl(86, 91, "core::clone::Clone", "Buffer", "impl Clone for Buffer")
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
ext(USE_MEM, "imports", "core::mem", L, 57, "mem::{self")
ext(USE_MEM, "imports", MU, L, 57, "MaybeUninit}")
ext(USE_STR, "imports", "core::str", L, 58, "core::str")

# ---------------- implements ----------------
ext("Buffer", "implements", "core::default::Default", L, 76, "impl Default for Buffer")
ext("Buffer", "implements", "core::marker::Copy", L, 83, "impl Copy for Buffer")
ext("Buffer", "implements", "core::clone::Clone", L, 86, "impl Clone for Buffer")
for i in impls:
    if i["self_type"] != "Buffer" and i["trait"]:
        edge(i["self_type"], "implements", i["trait"], L, i["start_line"], "impl ", skind=BI, gen=i["generated_by"], flags=["builtin_source_never_seeds_interface_edge"])

# ---------------- uses_type: signatures, fields, bounds, supertraits ----------------
edge("Buffer", "uses_type", MU, L, 73, "MaybeUninit<u8>", dkind="external", via=USE_MEM, flags=[EXT]); prim("Buffer", "u8", L, 73, "<u8>")
edge("Buffer", "reads", mx("i128"), L, 73, "i128::MAX_STR_LEN", flags=[OD8])
edge("<Buffer as Default>::default", "uses_type", "Buffer", L, 78, "-> Buffer")
edge("<Buffer as Clone>::clone", "uses_type", "Buffer", L, 88, "-> Self", flags=["Self_resolved_to_impl_self_type"])
edge("Buffer::new", "uses_type", "Buffer", L, 98, "-> Buffer")
edge("Buffer::format", "uses_type", "Integer", L, 106, "<I: Integer>"); prim("Buffer::format", "str", L, 106, "-> &str")
edge("Buffer::format", "uses_type", "private::Sealed::Buffer", L, 107, "cast::<I::Buffer>()", flags=["body_type_mention"])
edge("Integer", "uses_type", "private::Sealed", L, 119, ": private::Sealed"); prim("Integer::MAX_STR_LEN", "usize", L, 122, ": usize")
ext("private::Sealed", "uses_type", "core::marker::Copy", L, 128, ": Copy", flags=["via_prelude"])
edge("private::Sealed::write", "uses_type", "private::Sealed::Buffer", L, 131, "Self::Buffer"); prim("private::Sealed::write", "str", L, 131, "-> &str")
prim("DecimalPairs", "u8", L, 219, "[u8; 200]"); edge("DECIMAL_PAIRS", "uses_type", "DecimalPairs", L, 222, ": DecimalPairs", flags=[OD8])
prim("divmod100", "u32", L, 231, "u32")
edge("slice_buffer_to_str", "uses_type", MU, L, 247, "MaybeUninit<u8>", dkind="external", via=USE_MEM, flags=[EXT])
edge("slice_buffer_to_str", "uses_type", MU, L, 252, "*const [MaybeUninit<u8>]", dkind="external", via=USE_MEM, flags=[EXT, "body_type_mention"])
for p in ("u8", "usize", "str"): prim("slice_buffer_to_str", p, L, 247, p)
edge("Unsigned", "uses_type", "Integer", L, 255, ": Integer")
edge("Unsigned::fmt", "uses_type", "private::Sealed::Buffer", L, 256, "Self::Buffer", flags=["through_supertraits_Integer_Sealed"]); prim("Unsigned::fmt", "usize", L, 256, "-> usize")
edge("enc_16lsd", "uses_type", MU, L, 408, "MaybeUninit<u8>", dkind="external", via=USE_MEM, flags=[EXT])
for p in ("usize", "u8", "u64"): prim("enc_16lsd", p, L, 408, p)
for p in ("u128", "u64"): prim("div_rem_1e16", p, L, 450, p)
prim("u128_ext::mulhi", "u128", X, 7, "u128")
edge("<u128 as Unsigned>::fmt", "uses_type", sb("u128"), L, 336, "Self::Buffer"); prim("<u128 as Unsigned>::fmt", "usize", L, 336, "-> usize")

# ---------------- calls and reads in hand-written bodies ----------------
edge("<Buffer as Default>::default", "calls", "Buffer::new", L, 79, "Buffer::new()")
edge("<Buffer as Clone>::clone", "calls", "Buffer::new", L, 89, "Buffer::new()")
ext("Buffer::new", "calls", MU + "::uninit", L, 99, "MaybeUninit::<u8>::uninit()", via=USE_MEM)
edge("Buffer::new", "reads", mx("i128"), L, 99, "i128::MAX_STR_LEN")
ext("Buffer::format", "calls", "core::slice::<impl [T]>::as_mut_ptr", L, 107, ".as_mut_ptr()")
ext("Buffer::format", "calls", "core::ptr::<impl *mut T>::cast", L, 107, ".cast::<")
edge("Buffer::format", "calls", "private::Sealed::write", L, 108, "i.write(", flags=["generic_dispatch_one_edge_through_bound_I:Integer:Sealed"])
ext("Buffer::format", "calls", "core::str::<impl str>::len", L, 109, "string.len()")
edge("Buffer::format", "reads", "Integer::MAX_STR_LEN", L, 109, "I::MAX_STR_LEN", flags=["generic_trait_const"])
ext("Buffer::format", "calls", "core::hint::unreachable_unchecked", L, 110, "hint::unreachable_unchecked()", via=USE_HINT)
ext("slice_buffer_to_str", "calls", "core::slice::<impl [T]>::get_unchecked", L, 249, "buf.get_unchecked(")
ext("slice_buffer_to_str", "calls", "core::str::from_utf8_unchecked", L, 252, "str::from_utf8_unchecked(", via=USE_STR)
edge("div_rem_1e16", "calls", "u128_ext::mulhi", L, 463, "u128_ext::mulhi(n, M_HIGH)")
for ln in (14, 15, 19, 21):
    ext("u128_ext::mulhi", "calls", "<u128 as core::convert::From<u64>>::from", X, ln, "u128::from(", flags=["concrete_trait_impl_in_core"])

def table_sites(src, lines, f=L):
    for ln in lines:
        ext(src, "calls", MU + "::write", f, ln, ".write(")
def pairs_sites(src, lines):
    for ln in lines:
        edge(src, "reads", "DECIMAL_PAIRS", L, ln, "DECIMAL_PAIRS.0")
        ext(src, "calls", "core::slice::<impl [T]>::get_unchecked", L, ln, ".get_unchecked(")

F = "<u128 as Unsigned>::fmt"
ext(F, "calls", "core::slice::<impl [T]>::len", L, 340, "buf.len()")
table_sites(F, (341, 373, 374, 375, 376, 387, 388, 399)); pairs_sites(F, (373, 374, 375, 376, 387, 388))
for ln in (345, 353): edge(F, "calls", "div_rem_1e16", L, ln, "div_rem_1e16(")
for ln in (347, 350, 355, 358, 360): edge(F, "reads", mx("u128"), L, ln, "u128::MAX_STR_LEN")
for ln in (350, 358): edge(F, "calls", "enc_16lsd", L, ln, "enc_16lsd::<")
for ln in (371, 384): edge(F, "calls", "divmod100", L, ln, "divmod100(")
E = "enc_16lsd"
ext(E, "calls", "core::iter::Iterator::rev", L, 413, ".rev()", flags=["trait_method"])
for ln in (417, 431): edge(E, "calls", "divmod100", L, ln, "divmod100(")
table_sites(E, (420, 422, 424, 426, 433, 434, 435, 436)); pairs_sites(E, (420, 422, 424, 426, 433, 434, 435, 436))

# ---------------- macro-generated bodies ----------------
for s, u, inv in PAIRS:
    g = G("impl_Integer", inv); c = f"const _ #impl_Integer({s},{u})"
    for ln, t, k in ((138, s, "MIN"), (139, u, "MIN"), (140, s, "BITS"), (140, u, "BITS")):
        ext(c, "reads", f"core::num::<impl {t}>::{k}", L, ln, f"::{k}", gen=g, flags=[OD8])
    for t, ln, sgn in ((u, 144, "$Unsigned"), (s, 160, "$Signed")):
        prim(mx(t), "usize", L, ln, ": usize", gen=g)
        ext(mx(t), "reads", f"core::num::<impl {t}>::MAX", L, ln, f"{sgn}::MAX", gen=g, flags=[OD8])
        ext(mx(t), "calls", f"core::num::<impl {t}>::ilog10", L, ln, ".ilog10()", gen=g, flags=[OD8])
        bl = 148 if t == u else 164
        edge(sb(t), "uses_type", MU, L, bl, "MaybeUninit<u8>", dkind="external", via=USE_MEM, gen=g, flags=[EXT]); prim(sb(t), "u8", L, bl, "<u8>", gen=g)
        edge(sb(t), "reads", mx(t), L, bl, "Self::MAX_STR_LEN", gen=g, flags=[OD8])
        wl = 152 if t == u else 168
        edge(sw(t), "uses_type", sb(t), L, wl, "Self::Buffer", gen=g); prim(sw(t), "str", L, wl, "-> &str", gen=g)
    edge(sw(u), "calls", uf(u), L, 153, "Unsigned::fmt(self, buf)", gen=g, flags=["concrete_impl_chosen_from_argument_type"])
    edge(sw(u), "calls", "slice_buffer_to_str", L, 155, "slice_buffer_to_str(buf, offset)", gen=g)
    edge(sw(s), "reads", mx(s), L, 169, "Self::MAX_STR_LEN", gen=g)
    edge(sw(s), "reads", mx(u), L, 169, "$Unsigned::MAX_STR_LEN", gen=g)
    edge(sw(s), "calls", uf(u), L, 170, "Unsigned::fmt(", gen=g, flags=["concrete_impl_needs_core_return_type_of_unsigned_abs"])
    ext(sw(s), "calls", f"core::num::<impl {s}>::unsigned_abs", L, 171, "self.unsigned_abs()", gen=g)
    ext(sw(s), "calls", "core::convert::TryInto::try_into", L, 172, ".try_into()", gen=g, flags=["trait_method"])
    ext(sw(s), "calls", "core::result::Result::unwrap", L, 172, ".unwrap()", gen=g)
    ext(sw(s), "calls", MU + "::write", L, 176, ".write(b'-')", gen=g)
    edge(sw(s), "calls", "slice_buffer_to_str", L, 179, "slice_buffer_to_str(buf, offset)", gen=g)
for t, p, inv in SIZES:
    g = G("impl_Integer_size", inv)
    prim(mx(t), "usize", L, 195, ": usize", gen=g)
    edge(mx(t), "reads", mx(p), L, 195, "<$primitive as Integer>::MAX_STR_LEN", gen=g, flags=[OD8])
    edge(sb(t), "uses_type", sb(p), L, 200, "<$primitive as private::Sealed>::Buffer", gen=g, flags=[OD8])
    edge(sw(t), "uses_type", sb(t), L, 204, "Self::Buffer", gen=g); prim(sw(t), "str", L, 204, "-> &str", gen=g)
    edge(sw(t), "calls", sw(p), L, 205, "(self as $primitive).write(buf)", gen=g, flags=["concrete_receiver_from_cast"])
for u, inv in UNS:
    g = G("impl_Unsigned", inv); f = uf(u)
    edge(f, "uses_type", sb(u), L, 263, "Self::Buffer", gen=g); prim(f, "usize", L, 263, "-> usize", gen=g)
    ext(f, "calls", "core::slice::<impl [T]>::len", L, 265, "buf.len()", gen=g)
    ext(f, "calls", "core::mem::size_of", L, 271, "mem::size_of::<Self>()", via=USE_MEM, gen=g)
    for ln in (274, 281): ext(f, "calls", "core::convert::TryInto::try_into", L, ln, ".try_into()", gen=g, flags=["trait_method"])
    for ln in (275, 282): ext(f, "calls", "core::result::Result::expect", L, ln, ".expect(", gen=g)
    for ln in (285, 302): edge(f, "calls", "divmod100", L, ln, "divmod100(", gen=g)
    for ln in (288, 290, 292, 294, 306, 308, 319): ext(f, "calls", MU + "::write", L, ln, ".write(", gen=g)
    for ln in (288, 290, 292, 294, 306, 308):
        edge(f, "reads", "DECIMAL_PAIRS", L, ln, "DECIMAL_PAIRS.0", gen=g)
        ext(f, "calls", "core::slice::<impl [T]>::get_unchecked", L, ln, ".get_unchecked(", gen=g)

# ---------------- sites expected to stay unresolved or inactive ----------------
unresolved = []
def site(f, line, anchor, kind, reason, count=1):
    check(f, line, anchor); unresolved.append(dict(file=f, line=line, anchor=anchor, kind=kind, reason=reason, occurrences=count))
for ln in (138, 139, 140): site(L, ln, "assert!(", "body_macro", "depends_on_AD-1: core macro; unresolved(external_source_missing) without sysroot", 5)
site(L, 232, "debug_assert!(", "body_macro", "depends_on_AD-1: core macro; unresolved(external_source_missing) without sysroot")
site(L, 60, "use no_panic::no_panic;", "use", "cfg_inactive(feature no-panic off)"); site(X, 2, "use no_panic::no_panic;", "use", "cfg_inactive(feature no-panic off)")
for ln in (211, 212, 213, 214): site(L, ln, "impl_Integer_size!(", "item_macro", "cfg_inactive(target_pointer_width is 64)")

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
    candidate["source_site_evidence_status"] = "EXACT" if candidate["source_evidence_sites"] and all(
        site["range_status"] == "EXACT_ANCHOR" for site in candidate["source_evidence_sites"]) else "AMBIGUOUS_OR_EMPTY"
    candidate["evidence_key"] = "|".join(f"{site['source_path']}:{site['line']}:{site['byte_range']}" for site in candidate["source_evidence_sites"])
    candidate["evidence_ref_kind"] = "SYMBOLIC_SOURCE_SITE_REF_NOT_CANONICAL"
    edge_rows.append(candidate)
node_rows.sort(key=lambda row: (row["file"], row["start_line"], row["label"]))
edge_rows.sort(key=lambda row: (row["source"], row["relation"], row["target"]))
for row in impls:
    row["source_evidence"] = source_evidence(row["file"], row["start_line"], row["source_anchor"])
for row in unresolved:
    row["source_evidence"] = source_evidence(row["file"], row["line"], row["anchor"])
    row.update(RULES.classify_diagnostic_site({"reason_candidate": row["reason"], "reason_verified": False}))

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
                        "known_uncovered_constructs": ["operators", "body_types", "literals", "repeated_same_line_sites"]},
                    "canonical_identity_status": "SYMBOLIC_NONCANONICAL", "production_input": False,
                    "source_commit": source_commit, "source_pin_file": PIN_PATH.name}
dump("itoa-full-graph-nodes.json", {**candidate_header, "row_count": len(node_rows), "rows": node_rows})
dump("itoa-full-graph-edges.json", {**candidate_header, "row_count": len(edge_rows),
                                     "relation_kind_counts": RULES.relation_counts(edge_rows), "rows": edge_rows})
dump("itoa-local-verification-subset.json", {"artifact_status": "ELIGIBLE_LOCAL_ONLY_DIAGNOSTIC",
     "candidate_provenance": candidate_provenance,
     "nodes": eligible_nodes, "edges": classified["eligible_edges"],
     "counts": {"candidate_nodes": len(node_rows), "eligible_nodes": len(eligible_nodes),
                "candidate_edges": len(edge_rows), "eligible_edges": len(classified["eligible_edges"])}})
blocked_rows = {"candidate_provenance": candidate_provenance,
                "nodes": classified["blocked_nodes"], "edges": classified["blocked_edges"],
                "unresolved_or_inactive_sites": unresolved,
                "blocker_counts": dict(sorted(collections.Counter(blocker for row in
                    classified["blocked_nodes"] + classified["blocked_edges"] for blocker in row["blockers"]).items()))}
dump("itoa-blocked-candidate-index.json", blocked_rows)
dump("itoa-impl-block-records.json", {"artifact_status": "LLM_PARSED_DRAFT_NOT_PRODUCTION",
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
               "known_uncovered_constructs": ["operators", "body_types", "literals", "repeated_same_line_sites"]},
           "open_decisions": ["OD-2", "OD-7", "OD-8"], "production_comparison_status": "NOT_RUN"}
dump("itoa-target-dataset-summary.json", summary)
md = "# itoa expected full item graph: summary\n\n"
md += "**Status: LLM_PARSED_DRAFT_NOT_FROZEN.** Written by Devin from reading the pinned source. Not reviewed, not frozen, never a production input. Identities are symbolic labels, not production keys.\n\n"
md += "Every cited source anchor is checked against the pinned bytes. Whole-source coverage is not claimed: operators, body types, literals and repeated same-line sites are not covered. Rows that depend on OD-2, OD-7 or OD-8 are marked blocked.\n\n"
md += "```text\n" + "\n".join(f"{key}: {value}" for key, value in counts.items()) + "\n"
md += "\n".join(f"relation {key}: {value}" for key, value in summary["relation_kind_counts"].items()) + "\n"
md += "\n".join(f"target {key}: {value}" for key, value in target_kind_counts.items()) + "\n"
md += f"source commit: {source_commit}\n```\n"
(OUT / "itoa-target-dataset-summary.md").write_text(md)
print(md)
