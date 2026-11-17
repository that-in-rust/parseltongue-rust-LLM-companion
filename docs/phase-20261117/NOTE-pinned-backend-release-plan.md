# Pinning, compiling and releasing the non-Rust backend

## The answer

**Propose a Linux/macOS release containing two independently versioned executables: our
Rust `parseltongue` CLI and a pinned `codebase-memory-mcp` non-Rust backend. Build ours
natively per target; normally reuse an exact, verified upstream release binary.**

**Status: proposal, not implementation or validation.** The platform/asset selection below
records the packaging direction; it is not a qualified support matrix. This note adds no
code, lock file, binary, CI workflow, importer or release. The [phase entry document](ReadMe.md)
defines the working method and the boundary with the dataset and consumer CLI phases.

## 1. Linux/macOS assets and our selection

The v0.12.1 tag resolves to
[`a7158ac2f48ca646b4527341b76f930f2aeff87e`](https://github.com/DeusData/codebase-memory-mcp/tree/a7158ac2f48ca646b4527341b76f930f2aeff87e).
Its [packaging script](https://github.com/DeusData/codebase-memory-mcp/blob/a7158ac2f48ca646b4527341b76f930f2aeff87e/scripts/package-release.sh#L33-L44)
declares six Linux/macOS builds. Each has a `.tar.gz` archive:

| Build | Archive | Selection |
| --- | --- | --- |
| `linux-amd64` | `codebase-memory-mcp-linux-amd64.tar.gz` | Skip ordinary dynamic build |
| `linux-amd64-portable` | `codebase-memory-mcp-linux-amd64-portable.tar.gz` | Select |
| `linux-arm64` | `codebase-memory-mcp-linux-arm64.tar.gz` | Skip ordinary dynamic build |
| `linux-arm64-portable` | `codebase-memory-mcp-linux-arm64-portable.tar.gz` | Select |
| `darwin-amd64` | `codebase-memory-mcp-darwin-amd64.tar.gz` | Select only if users need Intel macOS |
| `darwin-arm64` | `codebase-memory-mcp-darwin-arm64.tar.gz` | Select |

The `.mcpb` bundles and `ui-` prefixed files are other packagings, not additional target
builds for this plan. We skip them; CI would select only the exact `.tar.gz` names above,
not a wildcard or a latest-release URL. This is source-declared packaging: the release
asset metadata API returned HTTP 403 during earlier research, so the complete published
asset list was not independently checked in that research.

We would ship four target mappings, with the fourth enabled only on demand:

| Our Rust target | Backend build | Qualification direction |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `linux-amd64-portable` | First integration target; native x86_64 build |
| `aarch64-unknown-linux-gnu` | `linux-arm64-portable` | Native arm64 build and qualification |
| `aarch64-apple-darwin` | `darwin-arm64` | Native Apple Silicon build and qualification |
| `x86_64-apple-darwin` | `darwin-amd64` | Only if users need it; native Intel build and qualification |

Start qualification with Linux x86_64, then Apple Silicon. Ubuntu 22.04/24.04 and macOS 14+
are possible initial OS baselines, not promises: check both executables' actual dependencies
and test the oldest claimed OS before advertising support. A portable backend does not
make our Rust executable or native storage dependencies portable to Alpine/musl or other
targets. No combined target is integration-tested yet.

### Linkage, CPU and signing caveats

Linux portable means **static linkage**, not a newer or older x86 instruction-set tier.
Upstream [builds and verifies static candidates](https://github.com/DeusData/codebase-memory-mcp/blob/a7158ac2f48ca646b4527341b76f930f2aeff87e/.github/workflows/_build.yml#L234-L270).
Its [compatibility notes](https://github.com/DeusData/codebase-memory-mcp/blob/a7158ac2f48ca646b4527341b76f930f2aeff87e/scripts/ci/check-glibc-compat.sh#L1-L22)
describe the ordinary Linux build as requiring glibc 2.38+ / GLIBCXX 3.4.32. No explicit
x86-64-vN CPU baseline was established by the research. Test both executables rather than
inferring our compatibility from the static backend.

Upstream macOS build runners do not prove a deployment minimum. Its
[Darwin signing step](https://github.com/DeusData/codebase-memory-mcp/blob/a7158ac2f48ca646b4527341b76f930f2aeff87e/scripts/ci/prepare-release-candidates.sh#L190-L206)
is ad-hoc signing, not notarization; no notarization step was found in the inspected
workflow. Our downloadable macOS distribution would need its own signing/notarization
decision, credentials and clean-machine Gatekeeper tests before support is advertised.

## 2. Pin a dependency contract, not just a version string

No binary goes into the repository. A future small lock file would map each approved Rust
target to the exact backend asset name and SHA-256 from upstream's signed `checksums.txt`.
No lock file or checksum values are added by this note.

The dependency-update change would also record:

- Upstream release version, resolved source commit and exact asset URL.
- License and third-party notice inventory/snapshot for the selected artifacts.
- Expected adapter protocol, graph-schema compatibility fingerprint and any separately
  authorized import mapping version.
- Our source commit, exact Rust toolchain, `Cargo.lock`, native build environment and tested
  CLI/backend combination.

Approve hashes during dependency review, rather than downloading and trusting a fresh
checksum manifest at installation. Upstream generates
[SHA-256 metadata](https://github.com/DeusData/codebase-memory-mcp/blob/a7158ac2f48ca646b4527341b76f930f2aeff87e/.github/workflows/release.yml#L203-L220)
and [Cosign bundles](https://github.com/DeusData/codebase-memory-mcp/blob/a7158ac2f48ca646b4527341b76f930f2aeff87e/.github/workflows/release.yml#L257-L269).
Verify signed artifact identity against the expected upstream repository/workflow identity
before accepting new hashes. A matching hash proves integrity, not upstream authorship by
itself. Freeze the signature-verification policy in the future executable specification.

**API/schema qualification is still required:** earlier graph/API research inspected
upstream main commit `cdfdf26483fa56f4f7d9d85e7c4c3924133b458b`, not the v0.12.1 commit
`a7158ac2f48ca646b4527341b76f930f2aeff87e`. Verify required MCP tools, response shapes,
pagination/limits and any internal SQLite schema dependency on that exact release. Do not
code against newer main and ship an older binary. v0.12.1 remains an unqualified candidate
pin, even though its asset selection is recorded here.

## 3. Compile Parseltongue independently; reuse the backend binary

Per-target release CI would build our CLI on a native runner, download the matching asset,
verify its approved SHA-256, extract it safely and package **both executables** with their
notices. No build or ordinary query would resolve latest. Preserve upstream's `LICENSE`
and `THIRD_PARTY_NOTICES.md`; they are included in its archives. The pinned backend may
remain unchanged across several releases of our CLI.

Pin a Rust toolchain that passes the research workspace, lock Cargo dependencies, and run
the equivalent of `cargo build --release --locked` for the selected binary/feature set.
Record compiler, linker, native dependencies and OS image; cache keys include target,
toolchain, lockfile and features. Build on the oldest claimed baseline and run the packaged
bytes on that baseline and a current OS. Inspect dynamic dependencies; one static component
does not make the distribution static.

The inspected research workspace is
[edition 2024/resolver 3](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/3e9a78f08779443de89146fd88251aef5a59e787/Cargo.toml#L1-L20),
with a [bundled DuckDB adapter](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/3e9a78f08779443de89146fd88251aef5a59e787/crates/parseltongue-duckdb-analysis-adapter/Cargo.toml#L7-L17)
and [Turso adapter](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/3e9a78f08779443de89146fd88251aef5a59e787/crates/parseltongue-turso-store-adapter/Cargo.toml#L7-L12).
Check the shipping binary's actual dependency/feature graph and parser source/build inputs
before freezing the build environment; follow [repository build-input rules](../../AGENTS.md).
Do not reuse main's declared Rust 1.70 as the research workspace toolchain. Earlier research
found no tracked per-platform release CI or toolchain pin on the inspected refs; creating
those is future implementation work.

Rebuild upstream from source only for a necessary patch, missing platform or source-build
requirement. That would be a separate pipeline with an exact source commit, all vendored
and generated inputs, pinned tools and upstream tests. Its inspected
[Unix build workflow](https://github.com/DeusData/codebase-memory-mcp/blob/a7158ac2f48ca646b4527341b76f930f2aeff87e/.github/workflows/_build.yml#L28-L83)
uses GCC/G++ on Linux, Apple C/C++ on macOS, Node 22 and zlib dependencies, with
[vendored Tree-sitter inputs](https://github.com/DeusData/codebase-memory-mcp/blob/a7158ac2f48ca646b4527341b76f930f2aeff87e/Makefile.cbm#L26-L35).
A home-built artifact has our own checksum/provenance, not the upstream release checksum.
Pinning alone does not establish bit-for-bit reproducibility.

## 4. Qualify the combined package before publishing

The future SPEC/SOP would require, per selected target:

1. Compile/test our chosen Rust binary and exercise its actual storage path.
2. Verify and launch the pinned backend, complete MCP initialization and inspect required
   tools/schema against frozen release-specific expectations.
3. Index pinned real non-Rust source and compare nodes, edges, source ranges, empty results
   and unresolved cases. Test malformed requests, incompatible schema, bad digest, wrong
   architecture, backend absence/failure and subprocess shutdown/stdout/stderr behavior.
4. Test reruns, deletion, stale results and generation replacement. A dedicated backend
   data directory must not inherit a user's unrelated global index; first verify the exact
   release's supported isolation mechanism.
5. If a separate overlay/importer is authorized, freeze and test its snapshot/completeness
   contract, atomic generation replacement, counts and endpoint integrity. Never infer a
   cross-language link without evidence or change frozen Rust facts to make a test pass.
6. Install the final archive on a clean baseline machine, including paths with spaces and
   Unicode, an offline installation and signature/notice verification. Test the packaged
   bytes, not only executables in a CI build tree.

Publish only after every mandatory platform gate passes; a failed target is not silently
included. Sign our release metadata/artifacts, retain the dependency inventory and preserve
Rust-only behavior when the backend is unavailable. No query should silently download it.

The parent session verified that the **ordinary Linux amd64 v0.12.1** archive is
**48,587,785 bytes**, containing a **331,387,792-byte executable**. These sizes were not
independently remeasured here and must not be assumed for the portable variant or other
targets. Index storage is additional and workload-dependent. Keep binaries and indexes out
of Git; account for their footprint in release packages and installation tests.

## 5. Upgrade and roll back one tested compatibility set

A backend bump would be a separate reviewable dependency change: version/source commit,
asset hashes, license and tool/schema diffs, fixture suite on all supported targets and
fresh staging indexes. Releasing our CLI would not automatically bump the backend.

Publish a new CLI/backend/adapter combination only after qualification. Keep the previous
executable, compatibility information and matching index/overlay generation until the new
combination is accepted. Never assume an old backend can open a newly migrated index.
Roll back executable and matching data together, or re-index from source; roll back our
CLI as well when needed. Release notes would name the tested pair and any required re-index.

This is packaging and qualification planning, not permission to implement an importer or
enrich the producer. The Rust dataset and phase-20261116 consumer remain unchanged. No
platform certification, runtime result, build reproducibility or release acceptance is
claimed by this note.
