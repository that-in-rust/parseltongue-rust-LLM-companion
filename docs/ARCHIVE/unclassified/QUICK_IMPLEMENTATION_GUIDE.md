# Quick Implementation Guide: Fix Stubbed Reindex

**Time Required**: 30 minutes
**Difficulty**: Easy
**Files Changed**: 1 file (file_watcher_integration_service.rs)

---

## TL;DR

The full implementation already exists in `incremental_reindex_core_logic.rs`. Just call it instead of the stub.

**Change Line 288**:
```rust
// FROM:
match execute_stub_reindex_operation(&file_path_str, &_state).await {

// TO:
match execute_incremental_reindex_core(&file_path_str, &state).await {
```

That's 95% of the fix. The rest is cleanup.

---

## Step-by-Step Implementation

### 1. Open the File

```bash
# File to edit
crates/pt08-http-code-query-server/src/file_watcher_integration_service.rs
```

### 2. Add Import (Line ~34)

**Add this line**:
```rust
use crate::incremental_reindex_core_logic::{
    execute_incremental_reindex_core,
    IncrementalReindexResultData,
};
```

### 3. Delete Stub Function (Lines 76-140)

**Delete this entire function**:
```rust
async fn execute_stub_reindex_operation(
    file_path: &str,
    state: &SharedApplicationStateContainer,
) -> Result<StubReindexResultData, String> {
    // ... 60+ lines ...
}
```

### 4. Delete Stub Struct (Lines 60-67)

**Delete this entire struct**:
```rust
#[derive(Debug, Clone)]
struct StubReindexResultData {
    file_path: String,
    hash_changed: bool,
    entities_added: usize,
    entities_removed: usize,
    processing_time_ms: u64,
}
```

### 5. Update Callback (Line ~244)

**Change from**:
```rust
let _state = state.clone();
```

**To**:
```rust
let state = state.clone();
```

(Remove the underscore)

### 6. Update Function Call (Line ~288)

**Change from**:
```rust
match execute_stub_reindex_operation(&file_path_str, &_state).await {
```

**To**:
```rust
match execute_incremental_reindex_core(&file_path_str, &state).await {
```

### 7. Update Success Log (Line ~292)

**Change from**:
```rust
println!(
    "[FileWatcher] Reindexed {}: +{} entities, -{} entities ({}ms) [STUB]",
    result.file_path,
    result.entities_added,
    result.entities_removed,
    result.processing_time_ms
);
```

**To**:
```rust
println!(
    "[FileWatcher] Reindexed {}: +{} entities, -{} entities ({}ms)",
    result.file_path,
    result.entities_added,
    result.entities_removed,
    result.processing_time_ms
);
```

(Remove `[STUB]`)

### 8. Update "No Change" Log (Line ~300)

**Change from**:
```rust
println!("[FileWatcher] Skipped {} (content unchanged)", result.file_path);
```

**To**:
```rust
println!(
    "[FileWatcher] Skipped {} (content unchanged, {} entities)",
    result.file_path,
    result.entities_before
);
```

### 9. Delete Outdated TODO (Lines ~285-287)

**Delete these lines**:
```rust
// Trigger incremental reindex (v1.4.5 - stub implementation)
// TODO v1.4.6: Replace with full implementation once Bug #3 and Bug #4 are fixed
// See: incremental_reindex_core_logic.rs for complete logic
```

**Replace with**:
```rust
// Trigger incremental reindex (v1.4.6 - full implementation)
```

---

## Build and Test

### Build

```bash
cargo clean
cargo build --release
```

**Expected**: Zero errors, zero warnings

### Test Compilation

```bash
cargo test -p pt08-http-code-query-server --no-run
```

**Expected**: Builds successfully

### Run Unit Tests

```bash
cargo test -p pt08-http-code-query-server
```

**Expected**: All tests pass

---

## Manual E2E Test

### Setup

```bash
# Create test directory
mkdir -p /tmp/reindex_test
cd /tmp/reindex_test

# Initialize database
parseltongue pt01-folder-to-cozodb-streamer .
# Note the workspace path (e.g., parseltongue20260202120000)

# Start server
parseltongue pt08-http-code-query-server \
  --db "rocksdb:parseltongue20260202120000/analysis.db" \
  --port 7777
```

### Test 1: Verify File Watcher Active

```bash
curl http://localhost:7777/server-health-check-status | jq '.data.file_watcher_active'
```

**Expected**: `true`

### Test 2: Create File with 1 Function

```bash
cat > calculator.py << 'EOF'
def add(a, b):
    return a + b
EOF

sleep 1

curl http://localhost:7777/code-entities-list-all | jq '.data.total_count'
```

**Expected**: `1`

**Check server log**:
```
[FileWatcher] Processing Modified: /tmp/reindex_test/calculator.py
[FileWatcher] Reindexed /tmp/reindex_test/calculator.py: +1 entities, -0 entities (12ms)
```

**No `[STUB]` marker!**

### Test 3: Add Second Function

```bash
cat >> calculator.py << 'EOF'

def subtract(a, b):
    return a - b
EOF

sleep 1

curl http://localhost:7777/code-entities-list-all | jq '.data.total_count'
```

**Expected**: `2`

### Test 4: Add Third Function

```bash
cat >> calculator.py << 'EOF'

def multiply(a, b):
    return a * b
EOF

sleep 1

curl http://localhost:7777/code-entities-list-all | jq '.data.total_count'
```

**Expected**: `3`

### Test 5: Verify Language Field

```bash
curl http://localhost:7777/code-entities-list-all | jq '.data.entities[0].language'
```

**Expected**: `"python"` (NOT `"rust"`)

### Test 6: Delete a Function

```bash
cat > calculator.py << 'EOF'
def add(a, b):
    return a + b

def multiply(a, b):
    return a * b
EOF

sleep 1

curl http://localhost:7777/code-entities-list-all | jq '.data.total_count'
```

**Expected**: `2` (subtract removed)

### Test 7: Multi-Language (JavaScript)

```bash
cat > test.js << 'EOF'
function greetUser(name) {
    console.log("Hello, " + name);
}
EOF

sleep 1

curl http://localhost:7777/code-entities-list-all | jq '.data.total_count'
```

**Expected**: `3` (add, multiply, greetUser)

```bash
curl http://localhost:7777/code-entities-list-all | jq '[.data.entities[] | {name, language}]'
```

**Expected**:
```json
[
  {"name": "add", "language": "python"},
  {"name": "multiply", "language": "python"},
  {"name": "greetUser", "language": "javascript"}
]
```

---

## Success Criteria Checklist

```
✅ Build succeeds with zero errors
✅ Build succeeds with zero warnings
✅ Unit tests pass
✅ File watcher stays active
✅ Entity count increases when adding functions
✅ Entity count decreases when removing functions
✅ Language fields are correct (not all "rust")
✅ Multiple languages work (Python, JavaScript, etc.)
✅ Server log shows no [STUB] marker
✅ Processing times are reasonable (<50ms)
✅ Server doesn't crash on edge cases
```

---

## Edge Case Testing (Optional)

### Test: Empty File

```bash
> empty.py
sleep 1
curl http://localhost:7777/code-entities-list-all | jq '.data.entities | map(select(.file_path | contains("empty.py")))'
```

**Expected**: Empty array `[]`

### Test: Syntax Error

```bash
cat > syntax_error.py << 'EOF'
def broken(
EOF

sleep 1
```

**Check server log**:
```
[ReindexCore] Warning: Failed to parse /tmp/reindex_test/syntax_error.py: ...
```

**Server should NOT crash**

### Test: Rapid Saves

```bash
for i in {1..10}; do
  echo "def func$i(): pass" >> rapid.py
done

sleep 1

curl http://localhost:7777/code-entities-list-all | jq '.data.total_count'
```

**Expected**: Entity count reflects final state (debouncing worked)

---

## Common Issues and Solutions

### Issue: Build Error - "Cannot find `execute_incremental_reindex_core`"

**Solution**: Add the import at the top of the file:
```rust
use crate::incremental_reindex_core_logic::execute_incremental_reindex_core;
```

### Issue: Build Error - "Unused variable `_state`"

**Solution**: Remove the underscore:
```rust
let state = state.clone();  // Not _state
```

### Issue: Entity Count Still 0

**Check**:
1. Server log shows reindex message?
2. Database path correct?
3. File in watched directory?
4. File extension in watched list (.py, .js, etc.)?

**Debug**:
```bash
# Check file watcher active
curl http://localhost:7777/server-health-check-status | jq

# Check what extensions are watched
# (Should see in server startup logs)
```

### Issue: Language Field Still "rust"

**This would mean the bug isn't fixed**. But it should be fixed because `incremental_reindex_core_logic.rs` uses correct language detection.

**Debug**:
```bash
# Check the entity key (first part should match language)
curl http://localhost:7777/code-entities-list-all | jq '.data.entities[0].key'
# Should be: "python:fn:add:..." not "rust:fn:add:..."
```

If key is correct but field is wrong → there's a serialization bug (unlikely).

---

## After Implementation

### Update Documentation

1. **CHANGELOG.md**: Add entry for v1.4.6
2. **README.md**: Remove any "stub" warnings
3. **Commit message**:
   ```
   feat: implement full incremental reindex in file watcher (v1.4.6)

   Replaces execute_stub_reindex_operation() with execute_incremental_reindex_core()
   which provides full tree-sitter parsing, entity extraction, and database updates.

   Changes:
   - Remove stub implementation (execute_stub_reindex_operation)
   - Use incremental_reindex_core_logic::execute_incremental_reindex_core
   - Update logs to remove [STUB] marker
   - Fix: Entities now indexed when files change via file watcher

   Test Results:
   - Entity counts update correctly on file save
   - Multi-language support verified (Python, JavaScript, Rust)
   - Language fields correct (not all "rust")
   - Performance: ~15ms for typical file

   Closes: File watcher reindex issue
   ```

### Tag Release

```bash
git tag v1.4.6
git push origin v1.4.6
```

---

## Time Breakdown

```
Code changes:       10 minutes
Build:              2 minutes
Unit tests:         1 minute
Manual E2E test:    15 minutes
Documentation:      5 minutes
──────────────────────────────
Total:              33 minutes
```

---

## What Success Looks Like

### Server Log (Before)

```
[FileWatcher] Reindexed calculator.py: +0 entities, -0 entities (1ms) [STUB]
```

### Server Log (After)

```
[FileWatcher] Reindexed calculator.py: +3 entities, -0 entities (15ms)
```

### API Response (Before)

```json
{
  "total_count": 0,
  "entities": []
}
```

### API Response (After)

```json
{
  "total_count": 3,
  "entities": [
    {
      "key": "python:fn:add:__calculator:T1738483200",
      "name": "add",
      "entity_type": "function",
      "language": "python"
    },
    {
      "key": "python:fn:subtract:__calculator:T1738483201",
      "name": "subtract",
      "entity_type": "function",
      "language": "python"
    },
    {
      "key": "python:fn:multiply:__calculator:T1738483202",
      "name": "multiply",
      "entity_type": "function",
      "language": "python"
    }
  ]
}
```

---

## Done!

You've successfully implemented full incremental reindex in the file watcher. The system now:

1. ✅ Detects file changes across 12 languages
2. ✅ Parses code using tree-sitter
3. ✅ Extracts entities and dependencies
4. ✅ Updates CozoDB database
5. ✅ Returns accurate statistics
6. ✅ Maintains stable entity keys (ISGL1 v2)

Next steps: Celebrate, then tackle v1.4.7 features!
