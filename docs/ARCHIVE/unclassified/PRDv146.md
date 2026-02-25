
From v143 we are now going to ship v145

  🔴 CRITICAL FINDINGS: 5 Showstopper Bugs

  1. File Watcher Reindexing DISABLED (FALSE ADVERTISING)
    - ✅ Verified: Code commented out with TODO
    - Advertised as v1.4.3 key feature
    - Detects changes but DOES NOTHING
    - Action: MAKE THIS WORK
  2. Smart Context Returns ZERO (CORE VALUE PROP BROKEN)
    - ✅ Verified: Live server returns 0 tokens, 0 entities
    - "99% token reduction" claim is BROKEN
    - Silent failure (success: true with empty data)
    - Action: DELETE THIS END POINT AND ITS TRACES
  3. Entity Keys Are NULL (DATA CORRUPTION)
    - ✅ Verified: /code-entities-list-all shows null keys
    - Fundamental data integrity issue
    - Makes entities unqueryable
    - Action: FIX THIS
  4. External Dependencies NOT TRACKED (INCOMPLETE GRAPH)
    - ✅ Verified: Code shows trait_name: None, struct_name: "Unknown"
    - Complexity hotspots ALL external (new, unwrap, to_string)
    - Interface signature graph incomplete
    - Action: FIX THIS
  5. Main Function Search Returns ZERO (ENTRY POINTS UNFINDABLE)
    - ✅ Verified: Search "main" → 0 results (database has 230 entities!)
    - Basic use case broken
    - Action: FIX THIS
  6. Fix ISGL1 v2





Notes at
docs/ISGL1-v2-Stable-Entity-Identity.md




ISGL1 v2 Benefits

  The Core Problem It Solves

  Current ISGL1 v1 keys use line numbers:
  rust:fn:handle_auth:__src_auth_rs:10-50
                                     ↑↑↑↑↑
                                BREAKS WHEN CODE SHIFTS

  What happens when you add 5 lines at the top of a file:
  BEFORE:                          AFTER (add 5 lines above):
  fn handle_auth()  :10-50    →    fn handle_auth()  :15-55   ← NEW KEY!
  fn validate()     :52-80    →    fn validate()     :57-85   ← NEW KEY!
  fn refresh()      :82-100   →    fn refresh()      :87-105  ← NEW KEY!

  Impact: ALL 3 keys changed even though ZERO code changed. This cascades through:
  - ❌ Dependency edges break (keys don't match anymore)
  - ❌ Incremental reindex shows "3 deleted + 3 added" (false positive)
  - ❌ Can't track which entities actually changed
  - ❌ Blast radius analysis fails (can't find entities)

  ---
  ISGL1 v2 Solution

  New format with birth timestamp:
  rust:fn:handle_auth:__src_auth_rs:T1706284800
                                     ↑↑↑↑↑↑↑↑↑↑
                                  NEVER CHANGES

  Same scenario (add 5 lines above):
  BEFORE:                                    AFTER:
  fn handle_auth()  :T1706284800    →    fn handle_auth()  :T1706284800  ✅ SAME KEY!
  fn validate()     :T1706284805    →    fn validate()     :T1706284805  ✅ SAME KEY!
  fn refresh()      :T1706284810    →    fn refresh()      :T1706284810  ✅ SAME KEY!

  Impact: Keys remain stable, system can accurately detect:
  - ✅ Entity is UNCHANGED (content hash matches)
  - ✅ Only line metadata updated (15-55 instead of 10-50)
  - ✅ Incremental reindex shows "0 changes" (accurate)
  - ✅ Dependency graph stays intact

  ---
  Concrete Benefits

  1. Reliable Incremental Indexing

  Before (v1): Re-parsing entire files because can't tell what changed
  After (v2): Only update entities that actually changed content

  Developer adds comment at line 1
  ├─ v1: Deletes all 50 entities, inserts 50 "new" entities (100 DB operations)
  └─ v2: Updates metadata for 50 entities (50 DB operations, 50% faster)

  2. Accurate Change Detection

  Before (v1): Every file edit triggers full dependency graph rebuild
  After (v2): Know exactly which entities changed

  Diff analysis:
  ├─ v1: "100 entities changed" (99 are false positives from line shifts)
  └─ v2: "1 entity changed" (the function you actually modified)

  3. Stable Dependency Graphs

  Before (v1): Adding whitespace breaks all dependency edges
  After (v2): Dependency graph survives refactoring

  Blast radius query for "handle_auth":
  ├─ v1: "Entity not found" (key changed from :10-50 to :15-55)
  └─ v2: Returns accurate impact (key :T1706284800 never changed)

  4. Foundation for Advanced Features

  Before (v1): Can't implement these workflows on unstable keys:
  - ❌ Entity history tracking (key changes on every edit)
  - ❌ Smart context selection (can't track what LLM saw before)
  - ❌ Change impact analysis (false positives everywhere)

  After (v2): Enables workflows from PRD:
  - ✅ Smart context: "Show me entities that changed since last query"
  - ✅ Temporal queries: "What called this function in v1.2.0?"
  - ✅ Incremental analysis: "Re-index only changed entities"

  ---
  Performance Impact

  Database Operations (100-entity file, add 1 comment at top)

  | Operation         | ISGL1 v1        | ISGL1 v2                      | Improvement   |
  |-------------------|-----------------|-------------------------------|---------------|
  | Entities deleted  | 100             | 1 (if actual deletion)        | 99% reduction |
  | Entities inserted | 100             | 1 (if actual addition)        | 99% reduction |
  | Entities updated  | 0               | 99 (metadata only)            | N/A           |
  | Edge updates      | 500 (all edges) | 5 (edges from changed entity) | 90% reduction |
  | Query time        | 500ms (rebuild) | 50ms (incremental)            | 10x faster    |

  Real-World Scenario

  Developer workflow: Edit function, add comment, save file

  ISGL1 v1 (line-based):
  ├─ Parse file: 100ms
  ├─ Delete 100 entities: 200ms
  ├─ Insert 100 entities: 200ms
  ├─ Rebuild 500 edges: 300ms
  └─ Total: 800ms ❌

  ISGL1 v2 (timestamp-based):
  ├─ Parse file: 100ms
  ├─ Match entities: 50ms (hash comparison)
  ├─ Update 1 entity: 10ms
  ├─ Update 5 edges: 20ms
  └─ Total: 180ms ✅ (4.4x faster)
