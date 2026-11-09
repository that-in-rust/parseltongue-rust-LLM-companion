# spancut — lexer + ITEM_RECOVERY_SET span cutter (experiment, DI-07 §16)

- `src/lib.rs`, `src/main.rs`, `tests/cutter.rs` — the cutter (no dependencies) and 18 four-word-named tests.
- `truth/` — oracle: `syn::parse_file` item spans (proc-macro2 span-locations).
- `compare.py` — oracle agreement over a file list; `mutate.py` — broken-code mutations; `files.txt` — the 125-file sample (paths relative to a clone dir of cargo-modules, rust-analyzer, flowistry, plus /home/ubuntu/repos/linux/rust).
- Results and the TDD log: devin-research/Deep-Insight-07.md §16.
