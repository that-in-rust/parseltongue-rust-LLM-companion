# Pinned macro expansion source

The implementation will copy-adapt the pure matcher/transcriber and token-to-parser-input
algorithms from rust-analyzer commit:

6e37b2d62127d445d77069031b416c48c9b0b4f2

Original sources:

- crates/mbe/src/lib.rs
- crates/mbe/src/parser.rs
- crates/mbe/src/expander.rs
- crates/mbe/src/expander/matcher.rs
- crates/mbe/src/expander/transcriber.rs
- crates/syntax-bridge/src/to_parser_input.rs

Upstream license: MIT OR Apache-2.0. Copyright rust-analyzer contributors.
The production port must not link the upstream mbe crate, salsa, HIR or IDE databases.
