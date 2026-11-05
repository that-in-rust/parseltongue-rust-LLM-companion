"""Paired synthetic Rust sources. No answer labels are returned by the generator."""
import hashlib
import random
import re
from pathlib import Path


KNOWN = ["qualified", "rename_pair", "plain_import"]
NOVEL = ["grouped", "module_alias", "glob", "shadow", "reexport", "alias_chain"]


def generate_paired_source_cases(root):
    src = root / "src"
    src.mkdir(parents=True, exist_ok=True)
    (root / "Cargo.toml").write_text('[package]\nname="bindingbench"\nversion="0.1.0"\nedition="2021"\n[workspace]\n')
    manifest = []
    for split, kinds, pairs, seed in [
        ("train", KNOWN, 32, 71), ("test", KNOWN, 12, 119),
        ("novel", NOVEL, 8, 241), ("missing", ["unbound"], 4, 301),
    ]:
        rng = random.Random(seed)
        for kind in kinds:
            for pair in range(pairs):
                suffix = f"{split}{kind.replace('_','')}{pair}"
                mods = ["wire_" + suffix, "store_" + suffix]
                ident, alias, other = "Packet" + suffix, "Ticket" + suffix, "Other" + suffix
                exported = "Export" + suffix if kind == "reexport" else ident
                decls = [f"pub mod {m} {{ pub struct {ident};" +
                         (f" pub use self::{ident} as {exported};" if kind == "reexport" else "") +
                         " }" for m in mods]
                rng.shuffle(decls)
                reverse = rng.choice([False, True])
                prefix = rng.choice(["", "self::"])
                for flip in range(2):
                    chosen, decoy = mods[flip], mods[1 - flip]
                    module = f"case_{split}_{kind}_{pair}_{flip}"
                    ret = alias
                    uses = []
                    if kind == "qualified":
                        ret = f"{prefix}{chosen}::{ident}"
                    elif kind in ("rename_pair", "reexport", "unbound"):
                        uses = [f"use {prefix}{chosen}::{exported} as {alias};",
                                f"use {prefix}{decoy}::{exported} as {other};"]
                        if kind == "unbound":
                            ret = "Unavailable" + suffix
                    elif kind == "plain_import":
                        uses = [f"use {prefix}{chosen}::{ident};", f"use {prefix}{decoy}::{ident} as {other};"]
                        ret = ident
                    elif kind == "grouped":
                        uses = [f"use {prefix}{chosen}::{{{ident} as {alias}}};",
                                f"use {prefix}{decoy}::{{{ident} as {other}}};"]
                    elif kind == "module_alias":
                        uses = [f"use {prefix}{chosen} as selected;", f"use {prefix}{decoy} as ignored;"]
                        ret = "selected::" + ident
                    elif kind == "glob":
                        uses = [f"use {prefix}{decoy}::*;", f"use {prefix}{chosen}::{ident};"]
                        ret = ident
                    elif kind == "shadow":
                        uses = [f"use self::{decoy}::{ident} as {alias};",
                                f"pub mod client {{ use super::{chosen}::{ident} as {alias};"]
                    elif kind == "alias_chain":
                        uses = [f"use {prefix}{chosen}::{ident} as Stage;",
                                f"use {prefix}{decoy}::{ident} as Reserve;", f"use self::Stage as {alias};"]
                    if reverse and kind != "shadow":
                        uses.reverse()
                    code = "\n".join(decls + uses + [f"pub fn make() -> {ret} {{ loop {{}} }}"] +
                                     (["}"] if kind == "shadow" else [])) + "\n"
                    (src / (module + ".rs")).write_text(code)
                    manifest.append(dict(case=module, split=split, kind=kind, pair=pair,
                                         sha256=hashlib.sha256(code.encode()).hexdigest()))
    for f in sorted(Path(__file__).with_name("examples").glob("*.rs")):
        module = "manual_" + f.stem
        code = f.read_bytes()
        (src / (module + ".rs")).write_bytes(code)
        manifest.append(dict(case=module, split="manual", kind=f.stem, pair=None,
                             sha256=hashlib.sha256(code).hexdigest()))
    (src / "lib.rs").write_text("#![no_std]\n" + "".join(f"pub mod {r['case']};\n" for r in manifest))
    return manifest


def identify_definition_query_positions(path):
    """Map source positions, NOT guessed links. Generated source is ASCII."""
    defs = {}
    query = None
    for line, s in enumerate(path.read_text().splitlines()):
        decl = re.search(r"pub mod (\w+) \{ pub struct (\w+);", s)
        if decl:
            defs[line, decl.start(2)] = "::".join(decl.groups())
        fn = re.search(r"pub fn make\(\) -> ([\w:]+)", s)
        if fn:
            query = line, fn.end(1) - len(fn[1].split("::")[-1])
    assert len(defs) == 2 and query is not None
    return defs, query
