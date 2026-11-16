#!/usr/bin/env python3
"""REQ-RUST-058.0: the expected target of every method call of pinned itoa that the frozen
expected graph lists without a target (133 sites).

Each row is written by reading the pinned itoa source and the `core` source of one compiler.
The builder checks every citation against those bytes and stops on the first that is wrong.
It writes test data only: production never reads its output (SOP Rule 2).
"""
import argparse
import hashlib
import importlib.util
import json
import pathlib
import re
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[2]
RULE_SPEC = importlib.util.spec_from_file_location("itoa_expected_graph_rules", HERE / "itoa-expected-graph-rules.py")
RULES = importlib.util.module_from_spec(RULE_SPEC)
RULE_SPEC.loader.exec_module(RULES)
VENDORED_SOURCE = ROOT / "crates/parseltongue-cli-dispatcher-binary/tests/pinned-itoa-source-tree"
OUTPUT_NAME = "itoa-expected-core-member-targets.json"
FREEZE_NAME = "itoa-expected-core-target-freeze.json"
# T8: what a stored row must equal. The rest of a row is the proof a reader checks.
COMPARED_FIELDS = ("target_label", "target_crate_name", "target_owner_kind", "core.declaration.file",
                   "core.declaration.line", "assumptions[].assumption_kind")
LIB = "src/lib.rs"

# T5: an assumption is something the row rests on that was not proven. A row may carry only these.
ASSUMPTION_KINDS = {
    "prelude_trait_impl_unrefuted": "a trait of the core prelude declares a method of the same name; that it is not "
                                    "implemented for the types of the earlier lookup steps was not proven by a trait solver",
}
# What the target names. A trait label names the trait's declaration; the impl that runs is not named.
OWNER_KINDS = {
    "inherent_impl": "the method is declared in an inherent impl of core; the label names that declaration",
    "trait_declaration_impl_not_named": "the method is declared by a trait of core; the label names the trait's "
                                        "declaration, and the impl that runs for this receiver is not named",
}
# Where the receiver's type comes from. `inferred_not_written` rows carry no type text.
RECEIVER_TYPE_SOURCE_KINDS = {
    "written_in_itoa": "a parameter, field, static, associated type or return type written in the pinned source",
    "declared_by_cited_core_item": "the type a cited core declaration gives: a return type or a constant's type",
    "inferred_not_written": "no source text states the type; the label does not depend on it (see name_scope_note)",
}
# How core writes the declaration. A fact, not a doubt.
DECLARATION_FORM_KINDS = {
    "plain": "the declaration is written inside the impl or trait that owns it",
    "core_macro_body": "the declaration is written in a macro_rules! body; the owner is the impl whose body calls "
                       "that macro (both lines are cited)",
}
# T4: how the fitting lookup step relates to the receiver's type, and how `self` is taken there.
STEP_KINDS = ("receiver_type", "array_unsized_to_slice")
SELF_ADJUSTMENTS = ("by_value", "shared_reference", "mutable_reference")


def parse_arguments():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", help="pinned itoa checkout; default: the vendored copy in the test tree")
    parser.add_argument("--core-source-root", help="<sysroot>/lib/rustlib/src/rust/library; default: asks rustc")
    parser.add_argument("--output-directory", default=str(HERE))
    parser.add_argument("--write-freeze-manifest", action="store_true",
                        help="write the freeze record (T7); a later change then needs a source citation and a new freeze")
    return parser.parse_args()


def read_compiler_version():
    return subprocess.run(["rustc", "-vV"], check=True, capture_output=True, text=True).stdout.strip().splitlines()


def find_library_root(argument):
    if argument:
        return pathlib.Path(argument)
    sysroot = subprocess.run(["rustc", "--print", "sysroot"], check=True, capture_output=True, text=True).stdout.strip()
    return pathlib.Path(sysroot) / "lib/rustlib/src/rust/library"


def read_core_files(library):
    """Every .rs file under core/src, keyed by its path relative to `library`, in path order."""
    paths = sorted(path.relative_to(library).as_posix() for path in (library / "core/src").rglob("*.rs"))
    return {path: (library / path).read_bytes() for path in paths}


def digest_core_files(files):
    """T6. Equal to: cd library && find core/src -name '*.rs' | LC_ALL=C sort | xargs sha256sum | sha256sum"""
    listing = "".join(f"{hashlib.sha256(data).hexdigest()}  {path}\n" for path, data in files.items())
    return hashlib.sha256(listing.encode()).hexdigest()


class CitationError(Exception):
    pass


def make_line_reader(files):
    lines = {path: data.decode().split("\n") for path, data in files.items()}

    def read(path, line):
        if path not in lines or not 1 <= line <= len(lines[path]):
            raise CitationError(f"{path}:{line} does not exist")
        return lines[path][line - 1]
    return read


def cite(read, path, line, text, says=None):
    """A citation: `text` must be on that line of that file."""
    if text not in read(path, line):
        raise CitationError(f"{path}:{line} does not hold {text!r}; it holds {read(path, line).strip()!r}")
    return {"file": path, "line": line, "text": text, **({"says": says} if says else {})}


HEADER = re.compile(r"\s*(pub(\([a-z]+\))? )?(unsafe )?(const )?(impl\b|trait\b|macro_rules!)")


def find_enclosing_header(read, path, line):
    """The nearest line above `line` with less indentation that opens an impl, a trait or a macro."""
    indent = lambda text: len(text) - len(text.lstrip())
    limit = indent(read(path, line))
    return next((number for number in range(line - 1, 0, -1)
                 if read(path, number).strip() and indent(read(path, number)) < limit and HEADER.match(read(path, number))),
                None)


def read_owner_label(header):
    """T3: the owner as core writes it. An impl gives its self type; a trait gives its name."""
    text = header.strip().rstrip("{").strip()
    trait = re.match(r"(pub )?(unsafe )?(const )?trait (\w+)", text)
    if trait:
        return trait.group(4)
    body = re.match(r"impl(<[^>]*>)? (.+)$", text)
    if not body:
        raise CitationError(f"not an impl or trait header: {header!r}")
    return body.group(2).strip()


def declare(read, path, line, text, owner_line, owner_text, macro_caller=None):
    """A core declaration and the impl or trait that owns it, both checked against the core bytes."""
    declaration = cite(read, path, line, text)
    enclosing = find_enclosing_header(read, path, line)
    if macro_caller is None:
        if enclosing != owner_line:
            raise CitationError(f"{path}:{line}: the enclosing header is line {enclosing}, not {owner_line}")
        owner = cite(read, path, owner_line, owner_text)
        return {"declaration": declaration, "owner": owner, "owner_label": read_owner_label(owner_text)}
    # The declaration is in a macro body: the owner is the impl whose body calls that macro.
    macro = cite(read, path, enclosing, "macro_rules! " + macro_caller["macro"])
    owner_path, call_line = macro_caller["file"], macro_caller["call_line"]
    call = cite(read, owner_path, call_line, macro_caller["macro"] + "!")
    if find_enclosing_header(read, owner_path, call_line) != owner_line:
        raise CitationError(f"{owner_path}:{call_line}: the macro call is not inside the impl at line {owner_line}")
    owner = cite(read, owner_path, owner_line, owner_text)
    return {"declaration": declaration, "declared_in_macro": macro, "macro_call": call, "owner": owner,
            "owner_label": read_owner_label(owner_text)}


def find_impl_line(read, path, text, total):
    found = [line for line in range(1, total + 1) if read(path, line) == text]
    if len(found) != 1:
        raise CitationError(f"{path}: {text!r} occurs {len(found)} times")
    return found[0]


def list_name_declarations(files, read, name):
    """Every `fn NAME` in the core source that is not in a comment, with the header that encloses it:
    the checkable part of a name-scope claim."""
    pattern = re.compile(rf"\bfn {name}\b")

    def enclosing(path, number):
        header = find_enclosing_header(read, path, number)
        return read(path, header).strip() if header else None
    return [{"at": f"{path}:{number}", "enclosing_header": enclosing(path, number)}
            for path, data in files.items()
            for number, line in enumerate(data.decode().split("\n"), 1)
            if pattern.search(line) and not line.lstrip().startswith("//")]


def step(kind, step_type, adjustment):
    """`step_type` is None when the receiver's type is not written."""
    assert kind in STEP_KINDS and adjustment in SELF_ADJUSTMENTS
    return {"step_kind": kind, "step_type": step_type, "self_adjustment": adjustment}


def build_rules(itoa, core, core_files):
    """One rule per (method name, source line of the site). A rule is a function of the site: the caller decides
    the primitive type (macro arguments) and the cited lines."""
    P = "core/src/"
    lib = lambda line, text, says=None: cite(itoa, LIB, line, text, says)
    core_line_count = lambda path: len(core_files[path].decode().split("\n"))

    slice_impl = (101, "impl<T> [T] {")
    maybe_write = declare(core, P + "mem/maybe_uninit.rs", 574, "pub const fn write(&mut self, val: T) -> &mut T {", 384, "impl<T> MaybeUninit<T> {")
    slice_get = declare(core, P + "slice/mod.rs", 640, "pub const unsafe fn get_unchecked<I>(&self, index: I) -> &I::Output", *slice_impl)
    slice_len = declare(core, P + "slice/mod.rs", 116, "pub const fn len(&self) -> usize {", *slice_impl)
    slice_ptr = declare(core, P + "slice/mod.rs", 760, "pub const fn as_mut_ptr(&mut self) -> *mut T {", *slice_impl)
    str_len = declare(core, P + "str/mod.rs", 155, "pub const fn len(&self) -> usize {", 131, "impl str {")
    ptr_cast = declare(core, P + "ptr/mut_ptr.rs", 31, "pub const fn cast<U>(self) -> *mut U {", 8, "impl<T: PointeeSized> *mut T {")
    try_into = declare(core, P + "convert/mod.rs", 625, "fn try_into(self) -> Result<T, Self::Error>;", 618, "pub const trait TryInto<T>: Sized {")
    result_expect = declare(core, P + "result.rs", 1178, "pub fn expect(self, msg: &str) -> T", 574, "impl<T, E> Result<T, E> {")
    result_unwrap = declare(core, P + "result.rs", 1226, "pub fn unwrap(self) -> T", 574, "impl<T, E> Result<T, E> {")
    iterator_rev = declare(core, P + "iter/traits/iterator.rs", 3439, "fn rev(self) -> Rev<Self>", 42, "pub const trait Iterator {")

    exact_size_len = cite(core, P + "iter/traits/exact_size.rs", 116, "fn len(&self) -> usize {",
                          "ExactSizeIterator declares len; the trait is in the prelude")
    prelude_iter = cite(core, P + "prelude/v1.rs", 44, "ExactSizeIterator", "the prelude of every edition exports it")
    prelude_iterator = cite(core, P + "prelude/v1.rs", 44, "Iterator}", "the prelude of every edition exports Iterator")
    prelude_try_into = cite(core, P + "prelude/mod.rs", 67, "pub use crate::convert::{TryFrom, TryInto};",
                            "the 2021 prelude exports TryInto; itoa's edition is 2021")
    try_into_blanket = cite(core, P + "convert/mod.rs", 821, "fn try_into(self) -> Result<U, U::Error> {",
                            "the one impl of TryInto in core is the blanket impl for every T")
    try_into_return = cite(core, P + "convert/mod.rs", 625, "-> Result<T, Self::Error>", "try_into returns a Result")

    no_other_trait = ("No trait in scope declares this name: itoa imports no trait (src/lib.rs lines 56 to 60), its "
                      "own traits in scope declare only `fmt`, and no trait of the core prelude declares it.")
    element_note = ("Indexing an array or a slice gives the element (index of type usize) or a slice (a range). "
                    "core declares no `write` for a slice, so the receiver is the element.")

    def primitive_of(site):
        return re.match(r"<(\w+) as ", site["source"]).group(1)

    def unsigned_of(site):
        return primitive_of(site).replace("i", "u")

    def buffer_alias(line):
        return lib(line, "type Buffer = [MaybeUninit<u8>; Self::MAX_STR_LEN];", "Self::Buffer is an array of MaybeUninit<u8>")

    def target(found, receiver, proven_by, lookup, scope, assumptions=()):
        source_kind, receiver_type = receiver
        assert source_kind in RECEIVER_TYPE_SOURCE_KINDS and (receiver_type is None) == (source_kind == "inferred_not_written")
        owner_is_trait = "trait" in found["owner"]["text"].split()
        return {"expected": "target", "target_label": None, "target_crate_name": "core",
                "target_owner_kind": "trait_declaration_impl_not_named" if owner_is_trait else "inherent_impl",
                "declaration_form_kind": "core_macro_body" if "declared_in_macro" in found else "plain",
                "receiver_type_source_kind": source_kind, "receiver_type_text": receiver_type,
                "receiver_type_proven_by": proven_by, "core": found,
                "lookup_step": lookup, "name_scope_note": scope, "assumptions": list(assumptions)}

    written = lambda text: ("written_in_itoa", text)
    from_core = lambda text: ("declared_by_cited_core_item", text)
    not_written = ("inferred_not_written", None)

    def write_on_element(parameter, alias):
        return lambda site: target(
            maybe_write, written("MaybeUninit<u8>"), [parameter] + ([alias] if alias else []),
            step("receiver_type", "MaybeUninit<u8>", "mutable_reference"),
            element_note + " " + no_other_trait + " `private::Sealed::write` is not in scope outside `mod private`.")

    def lookup_table(site):
        return target(
            slice_get, written("[u8; 200]"),
            [lib(222, "static DECIMAL_PAIRS: DecimalPairs", "the static's type"),
             lib(219, "struct DecimalPairs([u8; 200]);", "field 0 is an array of u8")],
            step("array_unsized_to_slice", "[u8]", "shared_reference"),
            "core declares no `get_unchecked` for an array (core/src/array/mod.rs). " + no_other_trait)

    def buffer_len(parameter):
        return lambda site: target(
            slice_len, written("&mut [MaybeUninit<u8>; Self::MAX_STR_LEN]"), [parameter, buffer_alias(148)],
            step("array_unsized_to_slice", "[MaybeUninit<u8>]", "shared_reference"),
            "core declares no `len` for an array. Earlier steps where the prelude trait could answer: "
            "`&mut [T; N]` taken by `&`, and `[T; N]` taken by `&`. By Rust's rules neither type implements "
            "ExactSizeIterator; the expected row names the target and carries the assumption (decided 2026-11-16).",
            [{"assumption_kind": "prelude_trait_impl_unrefuted", "trait_name": "ExactSizeIterator",
              "trait_method": exact_size_len, "trait_in_prelude": prelude_iter}])

    def literal_try_into(site):
        return target(
            try_into, not_written, [],
            step("receiver_type", None, "by_value"),
            "The receiver is an integer literal; its type is inferred. The label does not depend on that type: `try_into` is declared in core only by the trait "
            "TryInto and its one blanket impl, itoa declares no `try_into`, and itoa has no other dependency.")

    def result_of_try_into(found):
        return lambda site: target(
            found, from_core("Result<T, Self::Error>"), [try_into_return],
            step("receiver_type", "Result<T, Self::Error>", "by_value"),
            "The receiver is the value `try_into` returns. " + no_other_trait)

    def integer_ilog10(signed):
        macro, file, max_line, fn_line = (("int_impl", P + "num/int_macros.rs", 47, 3520) if signed
                                          else ("uint_impl", P + "num/uint_macros.rs", 51, 1833))
        max_text = "pub const MAX: Self = (<$UnsignedT>::MAX >> 1) as Self;" if signed else "pub const MAX: Self = !0;"

        def rule(site):
            name = primitive_of(site)
            owner_line = find_impl_line(core, P + "num/mod.rs", f"impl {name} {{", core_line_count(P + "num/mod.rs"))
            found = declare(core, file, fn_line, "pub const fn ilog10(self) -> u32 {", owner_line, f"impl {name} {{",
                            macro_caller={"macro": macro, "file": P + "num/mod.rs", "call_line": owner_line + 1})
            invocation = site["generated_by"]["invocation_line"]
            return target(
                found, from_core(name),
                [lib(invocation, name, f"the macro argument is the builtin type {name}"),
                 cite(core, file, max_line, max_text, f"{name}::MAX has the type {name}")],
                step("receiver_type", name, "by_value"), no_other_trait)
        return rule

    signed_write = lib(168, "fn write(self, buf: &mut Self::Buffer) -> &str {", "buf is &mut Self::Buffer")
    unsigned_fmt = lib(263, "fn fmt(self, buf: &mut Self::Buffer) -> usize {", "buf is &mut Self::Buffer")
    u128_fmt = lib(336, "fn fmt(self, buf: &mut Self::Buffer) -> usize {", "buf is &mut Self::Buffer")
    enc_buf = lib(408, "buf: &mut [MaybeUninit<u8>]", "buf is a slice of MaybeUninit<u8>")
    lines = lambda numbers, rule: {number: rule for number in numbers}

    return {
        "as_mut_ptr": {107: lambda site: target(
            slice_ptr, written("[MaybeUninit<u8>; i128::MAX_STR_LEN]"),
            [lib(106, "&mut self", "self is &mut Buffer"),
             lib(73, "bytes: [MaybeUninit<u8>; i128::MAX_STR_LEN],", "the field is an array")],
            step("array_unsized_to_slice", "[MaybeUninit<u8>]", "mutable_reference"),
            "core declares no `as_mut_ptr` for an array; `MaybeUninit<T>::as_mut_ptr` is for one MaybeUninit, not an "
            "array of them. " + no_other_trait)},
        "cast": {107: lambda site: target(
            ptr_cast, from_core("*mut T"),
            [cite(core, P + "slice/mod.rs", 760, "-> *mut T", "[T]::as_mut_ptr returns *mut T")],
            step("receiver_type", "*mut T", "by_value"),
            "The receiver is the value `as_mut_ptr` returns. " + no_other_trait)},
        "expect": lines((275, 282), result_of_try_into(result_expect)),
        "unwrap": lines((172,), result_of_try_into(result_unwrap)),
        "get_unchecked": {
            249: lambda site: target(
                slice_get, written("&[MaybeUninit<u8>]"), [lib(247, "buf: &[MaybeUninit<u8>]", "buf is a shared slice")],
                step("receiver_type", "&[MaybeUninit<u8>]", "by_value"), no_other_trait),
            **lines((288, 290, 292, 294, 306, 308, 373, 374, 375, 376, 387, 388, 420, 422, 424, 426, 433, 434, 435, 436), lookup_table)},
        "ilog10": {144: integer_ilog10(False), 160: integer_ilog10(True)},
        "len": {
            109: lambda site: target(
                str_len, written("&str"),
                [lib(108, "let string = i.write(", "string is the value Sealed::write returns"),
                 lib(131, "fn write(self, buf: &mut Self::Buffer) -> &str;", "Sealed::write returns &str")],
                step("receiver_type", "&str", "by_value"),
                "An inherent method that fits at the first step is chosen before any trait method, so the prelude "
                "trait ExactSizeIterator is not consulted."),
            265: buffer_len(unsigned_fmt), 340: buffer_len(u128_fmt)},
        "rev": {413: lambda site: target(
            iterator_rev, not_written, [prelude_iterator],
            step("receiver_type", None, "by_value"),
            "The receiver is a range expression; its type is not written. `rev` is declared exactly once in core, by the trait Iterator; itoa declares no `rev` and has no "
            "other dependency.")},
        "try_into": {
            172: lambda site: target(
                try_into, not_written, [prelude_try_into, try_into_blanket],
                step("receiver_type", None, "by_value"),
                "The receiver is a reference to a range index of `buf`; its type is not written. `try_into` is declared in core only by the trait TryInto and its one blanket impl; itoa declares no "
                "`try_into` and has no other dependency."),
            **lines((274, 281), lambda site: {**literal_try_into(site), "receiver_type_proven_by": [prelude_try_into, try_into_blanket]})},
        "write": {
            176: write_on_element(signed_write, buffer_alias(164)),
            **lines((288, 290, 292, 294, 306, 308, 319), write_on_element(unsigned_fmt, buffer_alias(148))),
            **lines((341, 373, 374, 375, 376, 387, 388, 399), write_on_element(u128_fmt, buffer_alias(148))),
            **lines((420, 422, 424, 426, 433, 434, 435, 436), write_on_element(enc_buf, None))},
    }


def list_untargeted_call_sites(edges):
    """T1: every `calls` site the frozen expected graph lists without a target, in source order."""
    sites = [{"source": row["source"], "method_name": row["callee_name"], "file": site["file"], "line": site["line"],
              "anchor_text": evidence["anchor_text"], "byte_range": evidence["byte_range"],
              "call_expression_text": exact["text"], "generated_by": row.get("generated_by")}
             for row in edges["rows"] if row["relation"] == "calls" and row["target_kind"] == "unresolved"
             for site, evidence, exact in zip(row["sites"], row["source_evidence_sites"], row["site_exact_ranges"])]
    return sorted(sites, key=lambda site: (site["file"], site["byte_range"], site["source"]))


def read_receiver_text(site):
    """The text before the method name, with white space collapsed."""
    text = site["call_expression_text"]
    return " ".join(text[:text.rindex("." + site["method_name"])].split())


def build_rows(sites, rules):
    def build(site):
        rule = rules.get(site["method_name"], {}).get(site["line"])
        if rule is None:
            raise CitationError(f"no rule for .{site['method_name']} at {site['file']}:{site['line']}")
        row = rule(site)
        label = f"{row['core']['owner_label']}::{site['method_name']}"
        return {"site": site, "receiver_text": read_receiver_text(site), **row, "target_label": label}
    return [build(site) for site in sites]


def main():
    args = parse_arguments()
    freeze = json.loads((HERE / "itoa-expected-graph-freeze.json").read_text())
    source_root = pathlib.Path(args.source_root) if args.source_root else VENDORED_SOURCE
    itoa_files = {path: (source_root / path).read_bytes() for path in freeze["source_file_sha256"]}
    RULES.verify_source_digests(itoa_files, freeze["source_file_sha256"])
    library = find_library_root(args.core_source_root)
    core_files = read_core_files(library)
    edges = json.loads((HERE / "itoa-expected-graph-edges.json").read_text())
    try:
        sites = list_untargeted_call_sites(edges)
        read_core = make_line_reader(core_files)
        rules = build_rules(make_line_reader(itoa_files), read_core, core_files)
        rows = build_rows(sites, rules)
        used = {(row["site"]["method_name"], row["site"]["line"]) for row in rows}
        unused = sorted({(name, line) for name, by_line in rules.items() for line in by_line} - used)
        if unused:
            raise CitationError(f"rules with no site: {unused}")
    except CitationError as error:
        raise SystemExit(f"REQ-RUST-058.0 citation check failed: {error}")
    names = sorted({row["site"]["method_name"] for row in rows})
    document = {
        "artifact_status": "FROZEN_EXPECTED_CORE_TARGETS",
        "meaning": "the expected target of each calls site that the frozen expected graph lists without a target",
        "requirement": "REQ-RUST-058.0",
        "compared_fields": list(COMPARED_FIELDS),
        "production_input": False,
        "provenance": {"author": "Devin", "basis": "PINNED_SOURCE_AND_CORE_SOURCE_READING",
                       "review": "AUTHOR_ONLY_NO_SECOND_REVIEWER",
                       # One-off exception to "No oracle" (user, 2026-11-16). The rows were written first; the
                       # compiler's output was then read once and changed no row. It is not their authority.
                       "independent_compiler_oracle_verification": "RUSTC_MIR_CALLEE_COUNTS_BY_LABEL_GROUP_ONCE",
                       "compiler_check": {
                           "date": "2026-11-16", "compiler": "rustc 1.99.1 (58cf8f9c3 2026-10-13)",
                           "command": "RUSTC_BOOTSTRAP=1 rustc --edition 2021 --crate-type lib --crate-name itoa "
                                      "-Zmir-opt-level=0 --emit=mir src/lib.rs",
                           "compared": "for each method name, the callee the MIR names and how many calls name it",
                           "result": "133 of 133 calls agree with the row labels, in 11 groups",
                           "limit": "matched by label and count, not site by site; one compiler, one target"}},
        "source_commit": freeze["source_commit"], "source_file_sha256": freeze["source_file_sha256"],
        "core_source": {"compiler_version_lines": read_compiler_version(), "directory": "<sysroot>/lib/rustlib/src/rust/library/core/src",
                        "file_count": len(core_files), "byte_count": sum(map(len, core_files.values())),
                        "listing_sha256": digest_core_files(core_files),
                        "listing_command": "cd <sysroot>/lib/rustlib/src/rust/library && find core/src -name '*.rs' | LC_ALL=C sort | xargs sha256sum | sha256sum"},
        "vocabulary": {"assumption_kinds": ASSUMPTION_KINDS, "target_owner_kinds": OWNER_KINDS,
                       "receiver_type_source_kinds": RECEIVER_TYPE_SOURCE_KINDS,
                       "declaration_form_kinds": DECLARATION_FORM_KINDS},
        "name_declarations_in_core": {name: list_name_declarations(core_files, read_core, name) for name in names},
        "row_count": len(rows), "rows": rows,
    }
    output = pathlib.Path(args.output_directory) / OUTPUT_NAME
    output.write_text(RULES.serialize_dataset(document))
    print(f"wrote {output.name}: {len(rows)} rows")
    if args.write_freeze_manifest:
        digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
        builders = (pathlib.Path(__file__).resolve(), HERE / "itoa-expected-graph-rules.py")
        (output.parent / FREEZE_NAME).write_text(RULES.serialize_dataset({
            "frozen_file_sha256": {OUTPUT_NAME: digest(output)},
            "builder_file_sha256": {path.name: digest(path) for path in builders},
            "expected_graph_edges_sha256": digest(HERE / "itoa-expected-graph-edges.json"),
            "source_commit": freeze["source_commit"], "source_file_sha256": freeze["source_file_sha256"],
            "core_source": document["core_source"], "row_count": len(rows),
            "provenance": document["provenance"]}))
        print(f"wrote {FREEZE_NAME}")


if __name__ == "__main__":
    main()
