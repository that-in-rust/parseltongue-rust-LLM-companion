# GitHub Actions Workflows

This directory contains automated CI/CD workflows for Parseltongue.

## Workflows

### 1. `release.yml` - Multi-Platform Release Builds

**Trigger**: Push tags matching `v*` (e.g., `v1.4.4`, `v1.5.0`)

**What it does**:
- Builds binaries for 4 platforms in parallel:
  - macOS ARM64 (Apple Silicon)
  - macOS x86_64 (Intel)
  - Linux x86_64
  - Windows x86_64
- Uploads all binaries to GitHub release automatically
- Verifies all 4 binaries uploaded successfully

**Usage**:
```bash
# Create and push a tag
git tag -a v1.5.0 -m "Release v1.5.0"
git push origin v1.5.0

# Workflow runs automatically
# Check: https://github.com/that-in-rust/parseltongue-dependency-graph-generator/actions
```

**Output**: 4 binaries uploaded to release:
- `parseltongue-macos-arm64`
- `parseltongue-macos-x86_64`
- `parseltongue-linux-x86_64`
- `parseltongue-windows-x86_64.exe`

### 2. `test.yml` - Cross-Platform CI Tests

**Trigger**: Push to `main` branch or pull requests

**What it does**:
- Runs full test suite on Linux, macOS, Windows
- Runs clippy checks
- Builds release binaries to verify compilation
- Runs file watcher tests specifically (critical for v1.4.3+)

**Testing Matrix**:
| Platform | Rust Version | Tests |
|----------|--------------|-------|
| Ubuntu Latest | Stable | All tests + file watcher |
| macOS Latest | Stable | All tests + file watcher |
| Windows Latest | Stable | All tests + file watcher |

---

## Setup (First Time)

These workflows were added in **v1.4.3** to prevent the "missing binaries" mistake.

**No additional setup required** - workflows run automatically when:
1. You push a tag (for releases)
2. You push to main or create a PR (for tests)

---

## Benefits

### Before GitHub Actions (v1.4.2 and earlier)
- ❌ Manual cross-compilation required
- ❌ Only tested on developer's machine (macOS ARM64)
- ❌ Risk of missing binaries in release
- ❌ No Windows/Linux validation

### After GitHub Actions (v1.4.3+)
- ✅ **Automatic** multi-platform builds
- ✅ Tested on Linux, macOS, Windows
- ✅ All binaries uploaded automatically
- ✅ Verification step catches missing binaries
- ✅ Cross-platform file watcher validation

---

## Troubleshooting

### Release workflow failed?

Check logs at: https://github.com/that-in-rust/parseltongue-dependency-graph-generator/actions

**Common issues**:
1. **Build failure**: Check Cargo.toml dependencies compile on all platforms
2. **Missing binary**: Check if binary was stripped or renamed incorrectly
3. **Upload failure**: Ensure tag exists and GITHUB_TOKEN has permissions

### Test workflow failing?

1. Run tests locally: `cargo test --all`
2. Check platform-specific failures in workflow logs
3. File watcher tests may fail on Windows due to different FS APIs

---

## Workflow Maintenance

**When to update**:
- Adding new platforms (e.g., Linux ARM64)
- Changing binary names
- Adding new test suites
- Updating Rust version requirements

**Last Updated**: 2026-02-01 (v1.4.3)
**Added By**: Post-release fix for missing binaries
