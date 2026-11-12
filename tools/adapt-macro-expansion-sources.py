#!/usr/bin/env python3
"""Mechanical pinned-source adaptation; subsequent safety edits are reviewed manually."""

from pathlib import Path
import re
import subprocess


def adapt_macro_expansion_sources():
    repository = Path(__file__).resolve().parents[1]
    upstream = repository.parent / "rust-tools/rust-analyzer"
    pin = subprocess.check_output(["git", "-C", str(upstream), "rev-parse", "HEAD"], text=True).strip()
    if pin != "6e37b2d62127d445d77069031b416c48c9b0b4f2":
        raise RuntimeError("the macro source checkout is not pinned")
    destination = repository / "crates/parseltongue-port-macro-expander/src"
    if (destination / "macro-expansion-rule-engine.rs").exists():
        raise FileExistsError("initializer refuses to overwrite reviewed manual adaptations")
    paths = {
        "lib.rs": "macro-expansion-rule-engine.rs",
        "parser.rs": "macro-expansion-rule-parts/rule-pattern-parsing-logic.rs",
        "macro_call_style.rs": "macro-expansion-rule-parts/macro-call-style-values.rs",
        "expander.rs": "macro-expansion-rule-parts/rule-match-expansion-logic.rs",
        "expander/matcher.rs": "macro-expansion-rule-parts/macro-expansion-match-parts/macro-rule-matching-logic.rs",
        "expander/transcriber.rs": "macro-expansion-rule-parts/macro-expansion-match-parts/macro-token-transcription-logic.rs",
    }
    for source, target in paths.items():
        text = (upstream / "crates/mbe/src" / source).read_text()
        text = text.replace("#[cfg(test)]\nmod benchmark;\n", "").replace("#[cfg(test)]\nmod tests;\n", "")
        if source == "lib.rs":
            text = text.replace("mod expander;", '#[path = "macro-expansion-rule-parts/rule-match-expansion-logic.rs"]\nmod expander;')
            text = text.replace("mod parser;", '#[path = "macro-expansion-rule-parts/rule-pattern-parsing-logic.rs"]\nmod parser;')
            text = text.replace("mod macro_call_style;", '#[path = "macro-expansion-rule-parts/macro-call-style-values.rs"]\nmod macro_call_style;')
        if source == "expander.rs":
            text = text.replace("mod matcher;", '#[path = "macro-expansion-match-parts/macro-rule-matching-logic.rs"]\nmod matcher;')
            text = text.replace("mod transcriber;", '#[path = "macro-expansion-match-parts/macro-token-transcription-logic.rs"]\nmod transcriber;')
        text = text.replace("crate::", "crate::macro_expansion_rule_engine::")
        text = text.replace("use syntax_bridge::to_parser_input;", "use crate::macro_token_parser_input::build_macro_parser_input;")
        text = text.replace("to_parser_input(", "build_macro_parser_input(")
        text = text.replace("db: &dyn salsa::Database", "ctx_edition: impl Copy + Fn(span::SyntaxContext) -> span::Edition")
        text = re.sub(r"\bdb\b", "ctx_edition", text)
        text = re.sub(r"([A-Za-z_][A-Za-z_0-9.]*)\.edition\(ctx_edition\)", r"ctx_edition(\1)", text)
        output = destination / target
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text("// Copied and adapted from rust-analyzer " + pin + "; MIT OR Apache-2.0.\n" + text)
    bridge = (upstream / "crates/syntax-bridge/src/to_parser_input.rs").read_text()
    bridge = bridge.replace("use syntax::", "use ra_ap_syntax::").replace("to_parser_input(", "build_macro_parser_input(")
    (destination / "macro-token-parser-input.rs").write_text("// Copied and adapted from rust-analyzer " + pin + "; MIT OR Apache-2.0.\n" + bridge)


if __name__ == "__main__":
    adapt_macro_expansion_sources()
