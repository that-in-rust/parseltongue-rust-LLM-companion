# Pinned macro expansion source

The implementation copy-adapts the pure matcher/transcriber and token-to-parser-input
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
Matcher/transcriber state-machine loops stay close to upstream for diffability.
The production port must not link the upstream mbe crate, salsa, HIR or IDE databases.

Adaptations replace database arguments with explicit edition callbacks, preserve token source
ranges with definition/invocation anchors, use typed public errors and reject partial results,
and replace unchecked token lookahead with recoverable errors. The initialization helper
refuses to overwrite the subsequently reviewed source.

The TT/intern/stdx/parser leaves are version-pinned to 0.0.362. The minimal safe span
compatibility package in ../parseltongue-macro-span-records retains the external ra_ap_span
package name for Cargo patch compatibility. Unlike upstream's salsa-disabled opaque context,
its record constructors need no unsafe operation. It contains no HIR, IDE, database, interner,
or query state. The root Cargo patch selects it for TT; the span dependency disables defaults.
