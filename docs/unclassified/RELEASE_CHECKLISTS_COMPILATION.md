# Release Checklists Compilation

This document compiles all release checklists found in the Parseltongue codebase.

---

## 1. S44: Release Checklist - Complete Process

**Source**: `.stable/archive-docs-v2/archive-p1/S44ReleaseCheckList.md`  
**Based on**: v0.9.6 release experience  
**Scope**: Complete end-to-end release process with edge cases, binary naming, verification, and cleanup

### Key Sections:

#### Pre-Flight Checks
- **Version Development Workflow**: Branch protection strategy
- **Clean Working State**: Git status verification
- **Test Verification**: All tests must pass
- **.gitignore Check**: Prevent binary commits
- **Repository Cleanup**: Keep root clean

#### Version Update Phase
- Update workspace version in `Cargo.toml`
- Update README version badge and installation instructions
- Check Mermaid diagram syntax
- Clean repository structure

#### Build Phase
- Clean build process
- Binary verification (size ~49MB, version matching)
- **Critical**: Binary naming convention (ALWAYS `parseltongue`, no version suffix)

#### Testing Phase
- Full test suite execution
- Critical functionality testing
- Subcommand accessibility verification

#### Git Operations
- Stage changes properly
- Comprehensive commit messages
- Tag creation and pushing

#### GitHub Release Phase
- Release creation with detailed notes
- Binary upload with simple naming
- Release verification

#### Install Script Creation
- Versioned install script creation
- URL updates for new version
- Feature descriptions

#### Post-Release Verification
- Clean environment testing
- Install script verification
- Binary functionality testing
- Download verification

#### Success Criteria
- 13-point checklist covering all aspects
- Edge case handling and troubleshooting
- Rollback procedures

---

## 2. TDD Release State Template

**Source**: `.stable/archive-docs-v2/archive-p2/TDD_RELEASE_STATE_v1.4.1.md` (Example)  
**Pattern**: TDD-focused release documentation  
**Status**: GREEN (Ready for Release)  
**Feature**: {FEATURE_DESCRIPTION}

### Release Checklist Components:

#### Version Bumps ({NUMBER} Files)
- `Cargo.toml` (Line 8)
- `README.md` (badge + download URL)
- `CLAUDE.md` (Line 13)
- {OTHER_FILES_TO_UPDATE}

#### Pre-Release Verification
- Clippy checks ({WARNING_COUNT} warnings acceptable)
- Test suite ({TEST_COUNT} passed, 0 failed)
- Clean build verification
- Binary functionality testing

#### Git Operations Template
- Detailed commit message template
- Tag creation commands
- Push procedures

#### GitHub Release Template
- Release notes structure
- Usage examples
- Installation instructions
- Testing documentation

#### Post-Release Verification
- Release appearance verification
- Download link testing
- Smoke testing procedures

---

## 3. Release Readiness Checklist Template

**Source**: `.stable/archive-docs-v2/archive-p1/V143-IMPLEMENTATION-JOURNAL.md` (Example)  
**Pattern**: Implementation journal checklist  
**Context**: {VERSION_CONTEXT}

### Checklist Items:
- [x] {TEST_CATEGORY} tests passing ({TEST_COUNT}/{TEST_COUNT})
- [x] Zero warnings in {COMPONENT_NAME} code
- [x] {ADDITIONAL_READINESS_CRITERIA}

---

## 4. Pre-Release Checklist Template

**Source**: `.stable/archive-docs-v2/archive-p2/Journal2026-01-27-v140-release-verification.md` (Example)  
**Pattern**: Comprehensive pre-release verification  
**Context**: {VERSION_CONTEXT}

### Checklist Sections:
#### 1. Dead Code Cleanup - {STATUS}
#### 2. Version Number Updates - {STATUS}
#### 3. Build Verification - {STATUS}
#### 4. Test Suite - {STATUS}
#### 5. {FEATURE_SPECIFIC_TESTING} - {STATUS}
#### 6. {ADDITIONAL_FEATURE_TESTING} - {STATUS}
#### 7. Documentation Updates - {STATUS}
#### 8. Git Operations - {STATUS}
#### 9. GitHub Release - {STATUS}
#### 10. Post-Release Verification - {STATUS}

---

## Common Patterns Across All Checklists

### 1. Pre-Release Phase
- Clean working state (`git status`)
- Test suite verification (`cargo test --release`)
- Version updates across multiple files
- Build verification (`cargo build --release`)

### 2. Build Phase
- Clean build (`cargo clean`)
- Binary verification (size, version)
- Naming convention enforcement

### 3. Testing Phase
- Unit tests
- Integration tests
- E2E tests
- Feature-specific testing

### 4. Git Operations
- Comprehensive commit messages
- Tag creation
- Push operations

### 5. Release Phase
- GitHub release creation
- Binary upload
- Release notes

### 6. Post-Release Phase
- Clean environment verification
- Install script testing
- Download verification
- Smoke testing

### 7. Quality Gates
- All tests must pass
- Zero critical issues
- Documentation updated
- Binary naming correct

---

## Critical Rules Identified

### Binary Naming
- **ALWAYS** name binary `parseltongue` (no version suffix)
- No platform-specific naming in releases
- Consistent naming across all environments

### Git Workflow
- Use version branches for experimental work
- Protect `main` branch from incomplete features
- Comprehensive commit messages with metrics

### Testing Requirements
- 0 test failures required
- E2E verification for critical features
- Performance benchmarks documented

### Documentation
- README version badges updated
- Installation instructions current
- Mermaid diagrams verified for syntax

### Verification
- Clean environment testing mandatory
- Install script verification
- Download link testing

---

## Templates and Commands

### Commit Message Template
```bash
git commit -m "$(cat <<'EOF'
release: v{NEXT_VERSION} - {TITLE}

## Major Changes

### 1. {FEATURE_NAME} ({METRIC})
- {IMPLEMENTATION_DETAIL_1}
- {IMPLEMENTATION_DETAIL_2}
- {METRIC/BENEFIT}

### 2. {FEATURE_NAME} ({METRIC})
- {IMPLEMENTATION_DETAIL_1}
- {IMPLEMENTATION_DETAIL_2}

## Performance Metrics

{TABLE_OR_LIST_OF_BEFORE/AFTER_METRICS}

## Testing

- Full test suite: {NUMBER} tests passing (0 failures)
- {FEATURE} tested on real codebase
- Binary size and version verified

## Breaking Changes

{NONE_OR_LIST}

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
EOF
)"
```

### Release Creation Template
```bash
gh release create v{NEXT_VERSION} \
  --title "v{NEXT_VERSION}: {TITLE}" \
  --notes "$(cat <<'EOF'
## What's New in v{NEXT_VERSION}

### 🎯 {FEATURE_1} ({METRIC})
{DESCRIPTION}

**Before**: {STATE_BEFORE}
**After**: {STATE_AFTER}
**Result**: {IMPROVEMENT}

### 📦 {FEATURE_2} ({METRIC})
{DESCRIPTION}

## Performance Metrics

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| {METRIC_1} | {VALUE} | {VALUE} | {PERCENTAGE} |
| {METRIC_2} | {VALUE} | {VALUE} | {PERCENTAGE} |

## Installation

```bash
# One-line install (recommended)
curl -fsSL https://raw.githubusercontent.com/that-in-rust/parseltongue/main/parseltongue-install-v{NEXT_VERSION_NO_DOTS}.sh | bash

# Manual download
curl -L https://github.com/that-in-rust/parseltongue/releases/download/v{NEXT_VERSION}/parseltongue -o parseltongue
chmod +x parseltongue
```

## Breaking Changes

{NONE_OR_LIST}

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)" \
  ./target/release/parseltongue
```

### Verification Commands
```bash
# Clean environment test
cd /tmp && mkdir test && cd test && git init
curl -fsSL https://raw.githubusercontent.com/that-in-rust/parseltongue/main/parseltongue-install-v{NEXT_VERSION_NO_DOTS}.sh | bash
./parseltongue --version  # Should show: parseltongue {NEXT_VERSION}
./parseltongue --help     # Verify all commands listed

# Test critical functionality
./parseltongue {CRITICAL_COMMAND} {TEST_ARGS}
```

---

## Evolution of Checklists

### v0.9.6 (S44)
- First comprehensive checklist
- Learned binary naming conventions
- Established Mermaid syntax requirements
- Created repository cleanup standards

### v1.4.0
- Added smart port selection verification
- Streamlined version update process
- Enhanced E2E testing requirements

### v1.4.1
- TDD-focused checklist
- CLI flag verification
- Performance benchmarking integration

### v1.4.3
- File watcher testing requirements
- Code quality standards

#### **CRITICAL MISTAKE IDENTIFIED AND FIXED (2026-02-01)**

**Problem**: Initial v1.4.3 release published **without binaries**
- Released with comprehensive notes and documentation
- BUT: Zero binary assets uploaded to GitHub release
- Users could not download and use v1.4.3 (only source code available)

**Root Cause**: Plan agent's release workflow didn't include multi-platform binary builds
- Line 293 in RELEASE_CHECKLISTS_COMPILATION.md referenced uploading binaries
- BUT: Actual release execution didn't include build step for multiple platforms
- Only uploaded ARM64 binary from local development machine

**Impact**:
- Release incomplete for ~2 hours
- macOS Intel, Linux, Windows users unable to use binaries
- README installation instructions pointed to non-existent downloads

**Fix Applied (Immediate)**:
1. Built macOS x86_64 binary via cross-compilation: `cargo build --release --target x86_64-apple-darwin`
2. Renamed binaries for clarity:
   - `parseltongue-macos-arm64` (Apple Silicon)
   - `parseltongue-macos-x86_64` (Intel)
3. Uploaded both binaries: `gh release upload v1.4.3 parseltongue-macos-arm64 parseltongue-macos-x86_64 --clobber`
4. Updated release notes with:
   - Platform-specific installation instructions
   - Complete Quick Start guide from README (no external links needed)
   - API endpoint table
   - Troubleshooting section

**Lessons Learned**:
1. **Pre-release verification MUST include**: `gh release view {VERSION} --json assets`
2. **Release notes MUST be self-contained**: Include all usage docs inline (no "see README" links)
3. **Multi-platform builds are MANDATORY**: At minimum macOS ARM64 + x86_64
4. **Binary naming convention**: Use platform suffix (`-macos-arm64`, `-macos-x86_64`)

**Updated Release Checklist Item #9** (GitHub Release Phase):
```bash
# Build all platform binaries BEFORE creating release
cargo build --release  # ARM64 (if on Apple Silicon)
cargo build --release --target x86_64-apple-darwin  # macOS Intel
# For Linux: Use GitHub Actions or Docker

# Create platform-named binaries
cp target/release/parseltongue parseltongue-macos-arm64
cp target/x86_64-apple-darwin/release/parseltongue parseltongue-macos-x86_64

# Create release AND upload binaries in one command
gh release create v{VERSION} \
  --title "v{VERSION}: {TITLE}" \
  --notes "{COMPREHENSIVE_NOTES_WITH_USAGE_DOCS}" \
  parseltongue-macos-arm64 \
  parseltongue-macos-x86_64

# MANDATORY VERIFICATION
gh release view v{VERSION} --json assets --jq '.assets[] | {name, size}'
# Expected: 2+ binaries listed with sizes > 50MB
```

**Platform Build Matrix** (v1.4.3 Standard):
| Platform | Architecture | Method | Binary Name |
|----------|--------------|--------|-------------|
| macOS | ARM64 (M1/M2/M3) | Native | `parseltongue-macos-arm64` |
| macOS | x86_64 (Intel) | Cross-compile | `parseltongue-macos-x86_64` |
| Linux | x86_64 | GitHub Actions/Docker | `parseltongue-linux-x86_64` |
| Windows | x86_64 | GitHub Actions | `parseltongue-windows-x86_64.exe` |

**Future Improvement** (v1.5.0+):
- Set up GitHub Actions workflow for automated multi-platform builds
- Trigger on git tag push
- Auto-upload all 4 platform binaries
- Eliminate manual cross-compilation steps

### Future Versions (Template)
- {NEW_FEATURE_VERIFICATION}
- {ADDITIONAL_QUALITY_STANDARDS}
- {EVOLVED_TESTING_REQUIREMENTS}

---

## Best Practices Identified

1. **Never skip clean environment verification**
2. **Always test install scripts before announcing**
3. **Maintain consistent binary naming with platform suffixes** (`-macos-arm64`, `-macos-x86_64`)
4. **Document all metrics and improvements**
5. **Use version branches for experimental work**
6. **Protect main branch stability**
7. **Verify Mermaid diagrams on GitHub**
8. **Keep repository root clean**
9. **Test all critical functionality**
10. **Maintain comprehensive documentation**
11. **MANDATORY: Verify binary uploads after release** (`gh release view {VERSION} --json assets`)
12. **Release notes MUST be self-contained** (include Quick Start, API docs, troubleshooting)
13. **Build multi-platform binaries BEFORE creating release** (minimum: macOS ARM64 + x86_64)
14. **Cross-compile locally or use GitHub Actions** (never release with single-platform binary)

---

## Usage Instructions

### For New Releases

1. **Copy relevant templates** from this compilation
2. **Replace placeholders** with actual values:
   - `{NEXT_VERSION}` → actual version (e.g., "1.5.0")
   - `{TITLE}` → release title
   - `{FEATURE_NAME}` → feature descriptions
   - `{METRIC}` → performance metrics
   - `{NEXT_VERSION_NO_DOTS}` → version without dots (e.g., "150")
3. **Execute checklist** in order
4. **Update this compilation** with new learnings

### Placeholder Reference

| Placeholder | Example | Description |
|-------------|---------|-------------|
| `{NEXT_VERSION}` | "1.5.0" | Full version number |
| `{NEXT_VERSION_NO_DOTS}` | "150" | Version for filenames |
| `{TITLE}` | "Add GraphQL support" | Release title |
| `{FEATURE_NAME}` | "GraphQL parser" | Feature description |
| `{METRIC}` | "50% faster queries" | Performance metric |
| `{TEST_COUNT}` | "25" | Number of tests |
| `{WARNING_COUNT}` | "3" | Acceptable warnings |

---

**Compilation Date**: 2026-02-01 (Updated with v1.4.3 post-release fixes)
**Sources**: 4 documents across archive directories + v1.4.3 lessons learned
**Total Checklist Items**: 60+ comprehensive steps
**Evolution**: From v0.9.6 to v1.4.3 (with critical post-release improvements)
**Template Ready**: Generic for any future version
**Last Major Update**: v1.4.3 multi-platform binary requirement
