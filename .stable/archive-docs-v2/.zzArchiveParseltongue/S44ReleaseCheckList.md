# S44: Release Checklist - Complete Process

Based on v0.9.6 release experience. Covers edge cases, binary naming, verification, and cleanup.

---

## Pre-Flight Checks (Before Starting)

### 0. Version Development Workflow (Branch Protection)

**Rule**: Always create a version branch for experimental work. This protects `main` from incomplete features.

```bash
# Create and switch to version branch
git checkout -b v0XY  # e.g., v097 for v0.9.7

# Verify you're on the version branch
git branch --show-current
# Should show: v0XY (not main)
```

**Why this matters**:
- **Protection**: `main` stays stable while you experiment
- **Isolation**: Breaking changes, failed experiments, and WIP commits stay in the branch
- **Rollback**: Easy to discard entire branch if approach doesn't work
- **Parallel work**: Can have multiple version branches (v097, v098-experimental)

**Workflow**:
1. **Start version work**: `git checkout -b v097`
2. **Experiment freely**: Make commits, break things, try approaches
3. **When ready for release**: Merge to main via PR or direct merge
4. **After release**: Delete version branch or keep for reference

**Branch naming**:
- Format: `vXYZ` (lowercase, no dots)
- Examples: `v097`, `v098`, `v100`
- NOT: `v0.9.7`, `version-097`, `feature-v097`

**Merge strategy** (when features are ready):
```bash
# Option A: Direct merge (if all changes are good)
git checkout main
git merge v097
git push origin main
git branch -d v097  # Delete local branch

# Option B: Cherry-pick specific commits (if some experiments failed)
git checkout main
git cherry-pick <commit-hash>
git push origin main
```

**Remote branching** (optional - for collaboration or backup):
```bash
# Push version branch to remote (like "ultrathink" pattern)
git push origin v097

# Others can checkout and continue work
git fetch origin
git checkout v097
```

**Edge case - Uncommitted changes when creating branch**:
```bash
# If you have uncommitted changes and want to move them to version branch
git stash
git checkout -b v097
git stash pop

# Now changes are in the version branch
```

---

### 1. Clean Working State
```bash
# Verify no uncommitted changes
git status
# Should show: "nothing to commit, working tree clean"

# Verify on main branch
git branch --show-current
# Should show: main

# Pull latest
git pull origin main
```

### 2. Verify Tests Pass
```bash
cargo test --release
# All tests must pass - 0 failures
```

### 3. Check .gitignore for Binaries
```bash
# Verify .gitignore excludes binaries BEFORE building
cat .gitignore | grep -E "parseltongue$|parseltongue-v"

# If missing, add:
echo "parseltongue" >> .gitignore
echo "parseltongue-v*" >> .gitignore

# Also exclude test databases (but NOT query directories!)
echo "*.db/" >> .gitignore
```

**Critical**: This prevents accidentally committing binaries (we did this in v0.9.6).

**⚠️ IMPORTANT - DO NOT DELETE OR .GITIGNORE THESE DIRECTORIES**:
```bash
entity_queries/       # ← Required: .scm files compiled into binary via include_str!()
dependency_queries/   # ← Required: .scm files compiled into binary via include_str!()
```

These contain tree-sitter `.scm` query files for 12 languages. The code uses:
```rust
include_str!("../../../entity_queries/rust.scm")
include_str!("../../../dependency_queries/rust.scm")
```

**If you delete these directories, compilation will fail with "file not found" errors.**

**What you CAN delete**:
- Test database directories: `*.db/` (parseltongue-demo.db, temp-analysis.db, etc.)
- Old install scripts: `parseltongue-install-v*.sh` (keep only latest)
- Binaries in root: `parseltongue`, `parseltongue-v*-macos-arm64`

---

## Version Update Phase

### 1. Update Workspace Version
**File**: `Cargo.toml` (root)
```toml
[workspace.package]
version = "X.Y.Z"  # Update this line
```

### 2. Update README Version Badge
**File**: `README.md` (line 3)
```markdown
> **vX.Y.Z** - Brief description of release (80 chars max)
```

**Examples**:
- ✅ `v0.9.6 - Test exclusion (90% token reduction) + single-binary architecture (80% disk reduction)`
- ❌ `v0.9.6 - Amazing new features!` (Too vague, hype language)

### 3. Update Installation Instructions
**File**: `README.md`

**Three locations to update**:

**Location 1: One-line install script reference**
```markdown
curl -fsSL https://raw.githubusercontent.com/that-in-rust/parseltongue/main/parseltongue-install-vXYZ.sh | bash
```

**Location 2: Install script description**
```markdown
1. Downloads `parseltongue` binary (vX.Y.Z with <features>)
```

**Location 3: Manual download**
```markdown
curl -L https://github.com/that-in-rust/parseltongue/releases/download/vX.Y.Z/parseltongue -o parseltongue
```

### 4. Check for Mermaid Diagram Syntax Errors
```bash
# Search for common Mermaid syntax errors
grep -n "ends$" README.md  # Should be "end"
grep -n "stroke-width:" README.md  # Not supported in GitHub Mermaid

# Verify Mermaid blocks are valid
# Look for: subgraph matching end, proper arrow syntax
```

**Common errors we hit**:
- `ends` instead of `end` (line 18 in v0.9.6)
- Invalid styling: `stroke-width:3px` (not supported)

### 5. Repository Cleanup - Keep Root Clean
```bash
# Check for clutter in root
ls -la *.md

# Only these should be in root:
# - README.md
# - RELEASE_CHECKLIST.md (or this file: S44ReleaseCheckList.md)

# Move version-specific docs to docs/vXYZ/
mkdir -p docs/vXYZ
git mv ARCHITECTURE_vX.Y.Z.md docs/vXYZ/
git mv PRD_vX.Y.Z.md docs/vXYZ/
git mv RoadmapVXYZtoVABC.md docs/vXYZ/

# Clean test databases (safe to delete)
rm -rf *.db/

# Remove old install scripts (keep only latest vXYZ)
git rm parseltongue-install-v*.sh  # Remove old versions
# (Keep only parseltongue-install-vXYZ.sh for current release)
```

**⚠️ CRITICAL - NEVER DELETE THESE**:
```bash
entity_queries/       # Required for compilation
dependency_queries/   # Required for compilation
crates/              # Source code
target/              # Build artifacts (can clean with cargo clean)
```

**Root directory structure should be**:
```
parseltongue/
├── README.md                         ✅ Keep
├── RELEASE_CHECKLIST.md              ✅ Keep
├── Cargo.toml                        ✅ Keep
├── Cargo.lock                        ✅ Keep
├── parseltongue-install-vXYZ.sh      ✅ Keep (latest only)
├── .gitignore                        ✅ Keep
├── crates/                           ✅ Keep (source code)
├── entity_queries/                   ✅ Keep (REQUIRED - compiled into binary)
├── dependency_queries/               ✅ Keep (REQUIRED - compiled into binary)
├── docs/                             ✅ Keep (documentation)
├── target/                           ✅ Keep (build artifacts)
├── *.db/                             ❌ Delete (test databases)
├── parseltongue                      ❌ Delete (should be .gitignored)
├── parseltongue-v*-macos-arm64       ❌ Delete (should be .gitignored)
└── parseltongue-install-vOLD.sh      ❌ Delete (old versions)
```

---

## Build Phase

### 1. Clean Build
```bash
cargo clean
cargo build --release
```

**Expected output**:
- Build succeeds with 0 errors
- Warnings are acceptable (note for future fixes)
- Duration: ~2-3 minutes

### 2. Verify Binary
```bash
# Check size and version
ls -lh target/release/parseltongue
./target/release/parseltongue --version

# Expected:
# Size: ~49MB (single binary architecture)
# Version: parseltongue X.Y.Z (must match workspace version)
```

**Edge case**: If version doesn't match, you forgot to update Cargo.toml.

### 3. Binary Naming Convention - CRITICAL

**Rule**: Binary is ALWAYS named `parseltongue` (no version suffix).

```bash
# Create release binary with SIMPLE NAME
cp target/release/parseltongue parseltongue

# Verify
ls -lh parseltongue
# Should show: parseltongue (not parseltongue-vX.Y.Z-platform)
```

**Why**:
- Agents always invoke `./parseltongue` (consistent)
- Users expect simple name
- Download URLs stay clean
- Version info lives in tags/release notes, not binary names

**Wrong approach** (we did this initially in v0.9.6):
```bash
❌ cp target/release/parseltongue parseltongue-v0.9.6-macos-arm64
```

---

## Testing Phase

### 1. Run Full Test Suite
```bash
cargo test --release 2>&1 | grep "test result"
```

**Success criteria**:
- All test suites show "ok"
- 0 failures
- Example: `test result: ok. 24 passed; 0 failed`

### 2. Test Critical Functionality Locally
```bash
# Test PT01 with test exclusion (if that's a feature)
./target/release/parseltongue pt01-folder-to-cozodb-streamer <test-codebase> --db "rocksdb:test.db"

# Verify expected behavior (e.g., test exclusion message)
# Check output for feature-specific messaging
```

### 3. Test All Subcommands Accessible
```bash
./target/release/parseltongue --help

# Verify all pt01-pt07 commands listed
# Verify help text is accurate
```

---

## Git Operations

### 1. Stage All Changes
```bash
git add -A
git status
```

**Verify**:
- Version updates in Cargo.toml
- README updates
- No binaries staged (should be in .gitignore)
- No build artifacts

**Edge case**: If you see `parseltongue` or `target/` staged, fix .gitignore immediately:
```bash
git reset HEAD parseltongue
echo "parseltongue" >> .gitignore
git add .gitignore
```

### 2. Commit with Comprehensive Message
```bash
git commit -m "$(cat <<'EOF'
release: vX.Y.Z - <Title>

## Major Changes

### 1. <Feature 1 Name> (<Metric>)
- <Implementation detail 1>
- <Implementation detail 2>
- <Metric/benefit>

### 2. <Feature 2 Name> (<Metric>)
- <Implementation detail 1>
- <Implementation detail 2>

## Performance Metrics

<Table or list of before/after metrics>

## Testing

- Full test suite: X tests passing (0 failures)
- <Feature> tested on real codebase
- Binary size and version verified

## Breaking Changes

None. (or list them)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
EOF
)"
```

**Template guidelines**:
- Major Changes: Group by feature, include metrics
- Performance Metrics: Quantify improvements
- Testing: Prove everything works
- Breaking Changes: Be honest

### 3. Create and Push Tag
```bash
# Create annotated tag
git tag -a vX.Y.Z -m "vX.Y.Z: <Short title>

- <Key feature 1>
- <Key feature 2>
- <Key metric>"

# Push commit and tag together
git push origin main
git push origin vX.Y.Z
```

**Verify tag pushed**:
```bash
git ls-remote --tags origin | grep vX.Y.Z
```

---

## GitHub Release Phase

### 1. Create GitHub Release with Notes
```bash
gh release create vX.Y.Z \
  --title "vX.Y.Z: <Title>" \
  --notes "$(cat <<'EOF'
## What's New in vX.Y.Z

### 🎯 <Feature 1> (<Metric>)
<Description>

**Before**: <State before>
**After**: <State after>
**Result**: <Improvement>

### 📦 <Feature 2> (<Metric>)
<Description>

## Performance Metrics

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| <metric 1> | <value> | <value> | <percentage> |
| <metric 2> | <value> | <value> | <percentage> |

## Installation

```bash
# One-line install (recommended)
curl -fsSL https://raw.githubusercontent.com/that-in-rust/parseltongue/main/parseltongue-install-vXYZ.sh | bash

# Manual download
curl -L https://github.com/that-in-rust/parseltongue/releases/download/vX.Y.Z/parseltongue -o parseltongue
chmod +x parseltongue
```

## Breaking Changes

None. (or list them)

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

**Verify release created**:
```bash
gh release view vX.Y.Z
```

### 2. Upload Binary with Simple Name
```bash
# Upload parseltongue (simple name, no version suffix)
gh release upload vX.Y.Z parseltongue --clobber

# Verify upload
gh release view vX.Y.Z --json assets
```

**Critical**: Binary must be named exactly `parseltongue` (not `parseltongue-vX.Y.Z-macos-arm64`).

**Edge case**: If you uploaded wrong name:
```bash
# Delete wrong asset
gh release delete-asset vX.Y.Z parseltongue-vX.Y.Z-macos-arm64 --yes

# Upload correct name
gh release upload vX.Y.Z parseltongue
```

---

## Install Script Creation

### 1. Create Versioned Install Script
**File**: `parseltongue-install-vXYZ.sh`

**Key sections to update**:

**Header**:
```bash
#!/bin/bash

# Parseltongue vX.Y.Z Install Script
# <Brief feature description>

echo "🐍 Parseltongue vX.Y.Z Installer"
echo "Features: <Feature 1> + <Feature 2>"
```

**Binary URL** (CRITICAL):
```bash
case $PLATFORM in
    darwin)
        if [ "$ARCH" = "arm64" ]; then
            # ✅ Simple URL - just "parseltongue"
            BINARY_URL="https://github.com/that-in-rust/parseltongue/releases/download/vX.Y.Z/parseltongue"
        else
            echo "❌ Error: macOS x86_64 not supported in this release"
            exit 1
        fi
        ;;
esac
```

**Feature descriptions**:
```bash
echo "🎯 vX.Y.Z Features:"
echo "   • <Feature 1>: <Benefit>"
echo "   • <Feature 2>: <Benefit>"
```

### 2. Make Executable and Commit
```bash
chmod +x parseltongue-install-vXYZ.sh

git add parseltongue-install-vXYZ.sh
git commit -m "chore: Add install script for vX.Y.Z

- Updated binary URL to vX.Y.Z release
- Updated feature descriptions
- Updated quick start examples

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

git push origin main
```

---

## Post-Release Verification (CRITICAL - DO NOT SKIP)

This is where we catch issues before users do.

### 1. Create Clean Test Environment
```bash
cd /tmp
rm -rf parseltongue-release-verify
mkdir parseltongue-release-verify
cd parseltongue-release-verify

# Initialize git repo (required for install script)
git init
echo "# Test" > README.md
git add .
git commit -m "init"
```

### 2. Test Install Script from GitHub
```bash
# Test the EXACT command users will run
curl -fsSL https://raw.githubusercontent.com/that-in-rust/parseltongue/main/parseltongue-install-vXYZ.sh | bash
```

**Watch for**:
- Download starts immediately (no 404 errors)
- Binary downloads successfully (~49M)
- Agent and docs download
- Success message appears

**If it fails**:
- Check GitHub release has `parseltongue` asset
- Check install script is pushed to main branch
- Check binary URL in install script matches release

### 3. Verify Downloaded Binary
```bash
# Check version
./parseltongue --version
# Output: parseltongue X.Y.Z

# Check size
ls -lh parseltongue
# Output: ~49M

# Check it's executable
file parseltongue
# Output: Mach-O 64-bit executable arm64
```

### 4. Test Critical Functionality
```bash
# Test help
./parseltongue --help
# Verify: All pt01-pt07 commands listed

# Test PT01 (or other critical command)
./parseltongue pt01-folder-to-cozodb-streamer <test-codebase> --db "rocksdb:verify.db"

# Verify: Expected behavior (e.g., test exclusion messaging)
```

### 5. Verify Downloads from GitHub
```bash
# Test direct binary download
curl -L https://github.com/that-in-rust/parseltongue/releases/download/vX.Y.Z/parseltongue -o test-binary
chmod +x test-binary
./test-binary --version
# Should show: parseltongue X.Y.Z

# Test agent download
curl -fsSL https://raw.githubusercontent.com/that-in-rust/parseltongue/main/.claude/agents/parseltongue-ultrathink-isg-explorer.md | head -5
# Should show: agent file content

# Test README download
curl -fsSL https://raw.githubusercontent.com/that-in-rust/parseltongue/main/README.md | head -5
# Should show: README content with correct version
```

### 6. Cleanup
```bash
cd /tmp
rm -rf parseltongue-release-verify
```

---

## Success Criteria Checklist

Before announcing release, verify ALL these:

- [ ] **Binary uploaded to GitHub release**: Check release page shows `parseltongue` asset
- [ ] **Binary has simple name**: NOT `parseltongue-vX.Y.Z-platform`, just `parseltongue`
- [ ] **Version matches**: `./parseltongue --version` shows correct version
- [ ] **Install script works**: Tested in clean environment, downloads successfully
- [ ] **All subcommands accessible**: `--help` shows all pt01-pt07 commands
- [ ] **Critical features work**: Tested main functionality (e.g., PT01 with test exclusion)
- [ ] **Test suite passes**: `cargo test --release` shows 0 failures
- [ ] **Documentation updated**: README shows correct version, install commands, features
- [ ] **Git tag pushed**: `git ls-remote --tags origin | grep vX.Y.Z` shows tag
- [ ] **GitHub release exists**: Release page shows notes, metrics, installation instructions
- [ ] **No binaries in repo**: `git status` shows no binaries, .gitignore excludes them
- [ ] **Root directory clean**: Only README.md and essential files in root
- [ ] **Mermaid diagrams render**: Check README on GitHub, no syntax errors

---

## Edge Cases and Troubleshooting

### Edge Case 1: Binary Won't Download (404 Error)

**Symptom**: Install script fails with "404 Not Found"

**Causes**:
1. Binary uploaded with wrong name (e.g., `parseltongue-v0.9.6-macos-arm64`)
2. Binary not uploaded to release
3. Install script has wrong URL

**Fix**:
```bash
# Check release assets
gh release view vX.Y.Z --json assets

# If binary has wrong name, delete and re-upload
gh release delete-asset vX.Y.Z <wrong-name> --yes
gh release upload vX.Y.Z parseltongue

# If URL wrong in install script, fix and push
# Edit parseltongue-install-vXYZ.sh, then:
git add parseltongue-install-vXYZ.sh
git commit -m "fix: Correct binary URL in install script"
git push origin main
```

### Edge Case 2: Version Mismatch

**Symptom**: Binary shows different version than expected

**Causes**:
1. Forgot to update workspace Cargo.toml
2. Built old version
3. Uploaded wrong binary

**Fix**:
```bash
# Update Cargo.toml
# Edit [workspace.package] version = "X.Y.Z"

# Clean rebuild
cargo clean
cargo build --release

# Verify version
./target/release/parseltongue --version

# Re-upload
cp target/release/parseltongue parseltongue
gh release upload vX.Y.Z parseltongue --clobber
```

### Edge Case 3: Mermaid Diagram Doesn't Render on GitHub

**Symptom**: README shows "Unable to render rich display" error

**Common causes**:
- `ends` instead of `end`
- Invalid styling (`stroke-width:`, `fill:` with complex values)
- Unclosed subgraphs
- Invalid arrow syntax

**Fix**:
```bash
# Find syntax errors
grep -n "ends$" README.md
grep -n "stroke-width:" README.md

# Fix and push
git add README.md
git commit -m "fix: Correct Mermaid syntax in README"
git push origin main
```

### Edge Case 4: Binary Accidentally Committed to Repo

**Symptom**: `git status` shows 49M binary staged or committed

**Fix immediately**:
```bash
# If not yet committed
git reset HEAD parseltongue
echo "parseltongue" >> .gitignore
git add .gitignore
git commit -m "fix: Add parseltongue to .gitignore"

# If already committed
git rm parseltongue
echo "parseltongue" >> .gitignore
git add .gitignore
git commit -m "fix: Remove binary from repo, update .gitignore"
git push origin main
```

### Edge Case 5: Install Script Downloads from Wrong Branch

**Symptom**: Install script works locally but not from GitHub

**Cause**: Changes not pushed to main branch

**Fix**:
```bash
# Verify install script is on main
curl -fsSL https://raw.githubusercontent.com/that-in-rust/parseltongue/main/parseltongue-install-vXYZ.sh | head -20

# If missing or wrong, push again
git push origin main

# Wait 1-2 minutes for GitHub CDN to update
```

---

## Rollback Plan (If Critical Issues Found)

If verification reveals critical issues:

### 1. Delete GitHub Release
```bash
gh release delete vX.Y.Z --yes
```

### 2. Delete Git Tag (Local and Remote)
```bash
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
```

### 3. Fix Issues Locally
```bash
# Fix the problem
# Re-run tests
cargo test --release

# Re-verify locally
./target/release/parseltongue <test-command>
```

### 4. Increment Version and Re-Release
```bash
# Update to X.Y.Z+1 (patch increment)
# Follow checklist from top
```

**Note**: Don't re-use a version number. If v0.9.6 had issues, fix and release v0.9.7.

---

## Announcement (Optional)

After successful verification:

### Update GitHub README Badge (if exists)
```markdown
![Version](https://img.shields.io/badge/version-X.Y.Z-blue)
```

### Announce in Channels (if applicable)
- GitHub Discussions
- Discord/Slack
- Twitter/social media

**Message template**:
```
Parseltongue vX.Y.Z released

• <Feature 1>: <Benefit>
• <Feature 2>: <Benefit>

Install: curl -fsSL https://raw.githubusercontent.com/that-in-rust/parseltongue/main/parseltongue-install-vXYZ.sh | bash

Release notes: https://github.com/that-in-rust/parseltongue/releases/tag/vX.Y.Z
```

**Tone**: Low-drama (per S05 - no hype, just facts).

---

## Post-Release Monitoring

### First 24 Hours
- Check GitHub issues for installation problems
- Monitor release download counts: `gh release view vX.Y.Z`
- Verify install script works across different environments (if multiple testers available)

### First Week
- Watch for bug reports
- Check if documentation is clear (users asking basic questions = docs need improvement)

---

## Lessons from v0.9.6 Release

### What Went Well ✅
1. Systematic approach with RELEASE_CHECKLIST.md
2. Test suite verification caught issues early
3. End-to-end verification in clean environment
4. Clear commit messages with metrics

### What Didn't Go Well ❌
1. **Binary naming confusion**: Started with `parseltongue-v0.9.6-macos-arm64`, had to fix to `parseltongue`
2. **Mermaid syntax errors**: `ends` vs `end`, invalid styling
3. **Repository clutter**: Docs in root instead of docs/ directory
4. **Binary committed**: Forgot to update .gitignore before building

### Key Takeaways 🎯
1. **Binary name is ALWAYS just `parseltongue`** - no exceptions
2. **Verify in clean environment BEFORE announcing** - catches 90% of issues
3. **Keep root clean** - move version docs to docs/vXYZ/
4. **Test Mermaid on GitHub** - push to branch first, verify render, then merge
5. **.gitignore first** - add binary patterns before building

---

## Quick Reference Commands

### Pre-Release
```bash
git status                           # Clean state
cargo test --release                 # All tests pass
cat .gitignore | grep parseltongue   # Binary excluded
```

### Build
```bash
cargo clean && cargo build --release
cp target/release/parseltongue parseltongue  # Simple name!
./parseltongue --version             # Verify version
```

### Release
```bash
git tag -a vX.Y.Z -m "..."
git push origin main && git push origin vX.Y.Z
gh release create vX.Y.Z --title "..." --notes "..."
gh release upload vX.Y.Z parseltongue
```

### Verify
```bash
cd /tmp && mkdir test && cd test && git init
curl -fsSL https://.../parseltongue-install-vXYZ.sh | bash
./parseltongue --version             # Must match X.Y.Z
./parseltongue --help                # All commands listed
```

---

## Version History

- **v0.9.6**: First release using this checklist - Learned binary naming, Mermaid syntax, repo cleanup
- **Future releases**: Update this section with learnings

---

**Remember**: A release is only done when verification in a clean environment succeeds. No shortcuts.
