# Release Checklist - Parseltongue

This checklist ensures safe, tested releases with proper verification at each stage.

## Pre-Release: Version & Code Updates

- [ ] **Update version in workspace Cargo.toml**
  - Path: `Cargo.toml` (workspace root)
  - Set `version = "X.Y.Z"`

- [ ] **Update version references in documentation**
  - README.md: Installation commands, binary names
  - Agent files: `.claude/agents/*.md`
  - Install script: `parseltongue-install-vXYZ.sh`

- [ ] **Update CHANGELOG or release notes**
  - Document what changed
  - Include performance metrics if applicable
  - Note breaking changes (if any)

## Build & Local Testing

- [ ] **Clean build**
  ```bash
  cargo clean
  cargo build --release
  ```

- [ ] **Verify binary size and location**
  ```bash
  ls -lh target/release/parseltongue
  # Should be ~49MB for single binary architecture
  ```

- [ ] **Run full test suite**
  ```bash
  cargo test --release
  # All tests must pass (0 failures)
  ```

- [ ] **Test critical commands locally**
  ```bash
  # Test PT01 with test exclusion
  ./target/release/parseltongue pt01-folder-to-cozodb-streamer <codebase> --db "rocksdb:test.db"

  # Verify test exclusion message appears
  # Verify entity counts are CODE only
  ```

- [ ] **Test help and version commands**
  ```bash
  ./target/release/parseltongue --help
  ./target/release/parseltongue --version
  ```

## Git Operations

- [ ] **Commit all changes**
  ```bash
  git add .
  git commit -m "release: v0.9.6 - <concise description>

  <detailed changes>

  🤖 Generated with [Claude Code](https://claude.com/claude-code)
  Co-Authored-By: Claude <noreply@anthropic.com>"
  ```

- [ ] **Create and push tag**
  ```bash
  git tag -a v0.9.6 -m "v0.9.6: <release title>"
  git push origin main
  git push origin v0.9.6
  ```

- [ ] **Verify tag on GitHub**
  - Visit https://github.com/amuldotexe/parseltongue/tags
  - Confirm v0.9.6 appears

## GitHub Release Creation

- [ ] **Create GitHub release via gh CLI**
  ```bash
  gh release create v0.9.6 \
    --title "v0.9.6: <Release Title>" \
    --notes "$(cat <<'EOF'
  ## What's New

  ### Major Changes
  - <feature 1>
  - <feature 2>

  ### Performance
  - <metric 1>
  - <metric 2>

  ### Bug Fixes
  - <fix 1>

  ## Installation

  ```bash
  curl -fsSL https://raw.githubusercontent.com/amuldotexe/parseltongue/main/parseltongue-install-vXYZ.sh | bash
  ```

  🤖 Generated with [Claude Code](https://claude.com/claude-code)
  EOF
  )"
  ```

- [ ] **Upload binary to release**
  ```bash
  # Rename binary with version and platform
  cp target/release/parseltongue parseltongue-v0.9.6-macos-arm64

  # Upload to GitHub release
  gh release upload v0.9.6 parseltongue-v0.9.6-macos-arm64
  ```

- [ ] **Verify release on GitHub**
  - Visit https://github.com/amuldotexe/parseltongue/releases/tag/v0.9.6
  - Confirm binary is attached
  - Confirm release notes are correct

## Install Script Update

- [ ] **Update install script version**
  - Create new file: `parseltongue-install-v096.sh` (or update existing)
  - Update GITHUB_RELEASE_URL to point to v0.9.6
  - Update VERSION variable
  - Update binary name to `parseltongue-v0.9.6-macos-arm64`

- [ ] **Commit and push install script**
  ```bash
  git add parseltongue-install-v096.sh
  git commit -m "chore: Update install script for v0.9.6"
  git push origin main
  ```

## Post-Release Verification (CRITICAL)

- [ ] **Test installation in clean temp directory**
  ```bash
  cd /tmp
  rm -rf parseltongue-test-install
  mkdir parseltongue-test-install
  cd parseltongue-test-install

  # Download and run install script
  curl -fsSL https://raw.githubusercontent.com/amuldotexe/parseltongue/main/parseltongue-install-v096.sh | bash
  ```

- [ ] **Verify downloaded binary version**
  ```bash
  ./parseltongue --version
  # Should show: parseltongue 0.9.6
  ```

- [ ] **Test critical functionality with downloaded binary**
  ```bash
  # Test PT01 with test exclusion
  ./parseltongue pt01-folder-to-cozodb-streamer <test-codebase> --db "rocksdb:verify.db"

  # Verify:
  # - Test exclusion message appears
  # - Correct entity counts
  # - No errors
  ```

- [ ] **Test all subcommands are accessible**
  ```bash
  ./parseltongue --help
  # Verify all pt01-pt07 commands listed
  ```

- [ ] **Cleanup test directory**
  ```bash
  cd /tmp
  rm -rf parseltongue-test-install
  ```

## Post-Release Communication (Optional)

- [ ] **Update README with latest version**
  - Update installation command if changed
  - Update version badges if applicable

- [ ] **Update documentation**
  - Update any docs that reference old version
  - Update agent descriptions if needed

- [ ] **Announce release**
  - GitHub discussions (if applicable)
  - Discord/Slack (if applicable)

## Rollback Plan (If Issues Arise)

If post-release verification fails:

1. **Delete the GitHub release**
   ```bash
   gh release delete v0.9.6 --yes
   ```

2. **Delete the git tag**
   ```bash
   git tag -d v0.9.6
   git push origin :refs/tags/v0.9.6
   ```

3. **Fix the issue locally**
   - Debug and fix the problem
   - Re-run local testing

4. **Start release process again**
   - Follow checklist from top
   - Increment patch version if needed (v0.9.7)

## Success Criteria

Release is considered successful when:
- ✅ All tests pass locally
- ✅ Binary builds without errors
- ✅ GitHub release created with binary attached
- ✅ Install script downloads and runs successfully
- ✅ Downloaded binary passes functionality tests
- ✅ All subcommands work correctly

## Notes

- **Platform**: Currently supports macOS ARM64 only
- **Binary size**: ~49MB for single-binary architecture
- **Testing**: Always test on clean system (temp directory) to catch environment-specific issues
- **Version consistency**: Ensure version matches everywhere (Cargo.toml, tags, install script, binary name)
