# Archived: Parseltongue v1.7.2 crates

Frozen copy of the previous Parseltongue implementation (tree-sitter parsing, CozoDB storage, HTTP query server), moved here unchanged with `git mv` so history is preserved:

- `crates/parseltongue` — CLI entrypoint
- `crates/parseltongue-core` — core graph logic and entities
- `crates/pt01-folder-to-cozodb-streamer` — ingestion into CozoDB
- `crates/pt08-http-code-query-server` — HTTP query API

It is no longer built from the repository root. To build it as it was:

```bash
cd archive/parseltongue-v1-crates
cargo build --release
```

The Rust-only redesign (pinned `ra_ap_syntax` parser, Turso tables, tiny LLMs) is described in `devin-research/Deep-Insight-08.md`.
