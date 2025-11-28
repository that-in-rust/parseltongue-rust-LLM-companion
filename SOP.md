# SOP: Standard Operating Procedures Journal

> Learning journal for meta-patterns discovered during development. Append-only.

---

## Entry 001: 4-Word Naming Convention Violation (2025-11-28)

**Encountered**: Planning HTTP server crate structure with file names like `cli.rs`, `server.rs`, `router.rs`, `health.rs`

**Error/Mistake**: Violated the 4-word naming convention documented in S01-README-MOSTIMP.md and S06-design101-tdd-architecture-principles.md. Files had 1 word instead of 4.

**Meta Pattern**: The 4-word rule applies to EVERYTHING:
- Crate names (hyphens): `pt08-http-code-query-server`
- File names (underscores): `server_health_check_handler.rs`
- Folder names (underscores): `http_endpoint_handler_modules/`
- Function names (underscores): `handle_server_health_check_status()`
- Struct names (PascalCase): `HttpServerStartupConfig`
- Endpoints (hyphens): `/server-health-check-status`

**What We Did**: Renamed all planned files from 1-word to 4-word names. `cli.rs` → `command_line_argument_parser.rs`, `health.rs` → `server_health_check_handler.rs`, etc.

**Meta Pattern Added**: Before creating ANY new file, folder, function, struct, or endpoint - COUNT THE WORDS. Must be exactly 4. Exceptions only for Rust conventions: `lib.rs`, `main.rs`, `mod.rs`, `Cargo.toml`.

---
