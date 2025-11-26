# Parseltongue v1.0.2 Noob Test Session

**Test Date:** 2025-11-26
**Test Location:** `crates/pt02-llm-cozodb-to-context-writer/`
**Tester Perspective:** Complete noob discovering Parseltongue from GitHub release
**Goal:** Document every friction point, confusion, and UX issue

---

## Step 1: Download Binary

**Action:** Download the v1.0.2 release binary from GitHub

```bash
curl -L -o parseltongue "https://github.com/that-in-rust/parseltongue/releases/download/v1.0.2/parseltongue-v1.0.2-macos-arm64"
chmod +x parseltongue
```

**Result:** ✅ Downloaded 48.9MB binary successfully

**Observation:** Download worked smoothly. Binary size is ~49MB.

---

## Step 2: First Run - Discovery

**Action:** Run binary without arguments to see what it does

**Noob Thought:** "I just downloaded this tool. What does it do? Let me run it."

```bash
./parseltongue
```

**Output:**
```
Parseltongue CLI Toolkit
Ultra-minimalist code analysis toolkit

Use --help for more information

Available commands:
  pt01-folder-to-cozodb-streamer       - Index codebase into CozoDB (Tool 1: Ingest)

  PT02: Export from CozoDB (Progressive Disclosure)
    pt02-level00                       - Pure edge list (~2-5K tokens) [RECOMMENDED]
    pt02-level01                       - Entity + ISG + Temporal (~30K tokens)
    pt02-level02                       - + Type system (~60K tokens)

  pt07                                 - Visual analytics (Tool 7: Visualize)
```

**Observation:**
- ✅ Good: Clean, simple output
- ✅ Good: Clear command hierarchy shown
- ⚠️ **FRICTION**: No quick start example showing the simplest workflow
- ⚠️ **FRICTION**: "Use --help for more information" but unclear which command to start with
- 🤔 **CONFUSION**: "CozoDB" - what is that? No explanation
- 🤔 **CONFUSION**: "Progressive Disclosure" - technical term, unclear to noobs
- 🤔 **CONFUSION**: Multiple levels (level00, level01, level02) - which one do I need?

**Noob Decision:** "Let me try --help to learn more"

---

## Step 3: Getting Help

**Action:** Run with --help flag

```bash
./parseltongue --help
```

**Output:**
```
Ultra-minimalist CLI toolkit for code analysis

Usage: parseltongue [COMMAND]

Commands:
  pt01-folder-to-cozodb-streamer  Tool 1: Stream folder contents to CozoDB with ISGL1 keys
  pt02-level00                    Tool 2a: Export pure edge list (Level 0 - ~2-5K tokens) [RECOMMENDED]
  pt02-level01                    Tool 2b: Export entities with ISG + temporal (Level 1 - ~30K tokens)
  pt02-level02                    Tool 2c: Export entities with type system (Level 2 - ~60K tokens)
  pt07                            Tool 7: Visual analytics for code graphs
  help                            Print this message or the help of the given subcommand(s)
```

**Observation:**
- ✅ Good: Shows all commands clearly
- 🤔 **CONFUSION**: Still no workflow explanation - do I run pt01 first, then pt02?
- 🤔 **CONFUSION**: "ISGL1 keys" - what are these?
- 🤔 **CONFUSION**: "rocksdb:" prefix - is this mandatory?

**Noob Decision:** "pt01 seems to be step 1, let me get help on that"

---

## Step 4: Understanding pt01

**Action:** Get help on pt01 command

```bash
./parseltongue pt01-folder-to-cozodb-streamer --help
```

**Output:**
```
Examples:
  parseltongue pt01-folder-to-cozodb-streamer .            # Index current directory
  parseltongue pt01-folder-to-cozodb-streamer ./src --db rocksdb:analysis.db --verbose

Usage: parseltongue pt01-folder-to-cozodb-streamer [OPTIONS] [directory]

Arguments:
  [directory]  Directory to index [default: current directory] [default: .]

Options:
      --db <db>   Database file path [default: parseltongue.db]
  -v, --verbose   Enable verbose output
  -q, --quiet     Suppress output
  -h, --help      Print help (see a summary with '-h')
```

**Observation:**
- ✅ **EXCELLENT**: Real examples shown! This is super helpful
- ✅ Good: Simple default (just run with `.`)
- ⚠️ **FRICTION**: Example shows `rocksdb:analysis.db` but default is `parseltongue.db` - which one is correct?
- 🤔 **CONFUSION**: What's the difference between `--db parseltongue.db` and `--db rocksdb:analysis.db`?
- ✅ Good: Defaults make sense (current directory, simple db name)

**Noob Decision:** "Let me just run the simplest command and index this directory"

---

## Step 5: First Ingestion Attempt

**Action:** Run pt01 on current directory with defaults

**Noob Thought:** "The example shows `.` so I'll do that"

```bash
./parseltongue pt01-folder-to-cozodb-streamer .
```

**Output:**
```
Running Tool 1: folder-to-cozodb-streamer
  Workspace: parseltongue20251126182052
  Database: rocksdb:parseltongue20251126182052/analysis.db
Starting directory streaming...

Streaming Summary:
Total files found: 52
Files processed: 16
Entities created: 4 (CODE only)
  └─ CODE entities: 4
  └─ TEST entities: 252 (excluded for optimal LLM context)
Errors encountered: 36
Duration: 287ms

✓ Tests intentionally excluded from ingestion for optimal LLM context
✓ Indexing completed
  Files processed: 16
  Entities created: 4

📁 Workspace location:
  parseltongue20251126182052

Next steps:
  Export edges:    parseltongue pt02-level00 --where-clause "ALL" \
                     --output parseltongue20251126182052/edges.json \
                     --db "rocksdb:parseltongue20251126182052/analysis.db"

  Export entities: parseltongue pt02-level01 --include-code 0 --where-clause "ALL" \
                     --output parseltongue20251126182052/entities.json \
                     --db "rocksdb:parseltongue20251126182052/analysis.db"
```

**Observation:**
- ✅ **EXCELLENT**: Automatic workspace creation with timestamp!
- ✅ Good: Clear progress output
- ✅ Good: Helpful "Next steps" with actual commands
- ⚠️ **WARNING**: "Errors encountered: 36" - sounds scary but no explanation
- 🤔 **CONFUSION**: Only 4 entities created from 52 files - is this normal?
- 🔴 **CRITICAL FRICTION #1**: Must manually copy/paste the long workspace path
- 🔴 **CRITICAL FRICTION #2**: Must copy/paste full database path with `rocksdb:` prefix
- 🔴 **CRITICAL FRICTION #3**: Must type `--where-clause "ALL"` every time
- ⚠️ **FRICTION**: The "Next steps" commands are split across multiple lines with backslashes - hard to copy

**Noob Thought:** "OK, so it created a workspace. Now I need to copy this long command to export edges. Let me try..."

---

## Step 6: Attempting First Export

**Action:** Copy and paste the pt02-level00 command from "Next steps"

**Noob Experience:**
1. Select the command text from terminal
2. Try to copy it (backslashes make it awkward)
3. Need to substitute `./parseltongue` instead of `parseltongue`
4. Paste into terminal

**Command Run:**
```bash
./parseltongue pt02-level00 --where-clause "ALL" --output parseltongue20251126182052/edges.json --db "rocksdb:parseltongue20251126182052/analysis.db"
```

**Output:**
```
Running PT02 Level 0: Pure Edge List Export
✓ PT02 Level 0 export completed
  Output files: parseltongue20251126182052/edges.json, parseltongue20251126182052/edges_test.json
  Edges exported: 870
  Token estimate: ~5000
  Fields per edge: 3 (from_key, to_key, edge_type)
```

**Observation:**
- ✅ **SUCCESS**: Export worked! 870 edges from only 4 entities is impressive
- ✅ Good: Clear output with metrics
- ✅ Good: Token estimate helpful for LLM context planning
- ⚠️ **SURPRISE**: Creates TWO files (edges.json AND edges_test.json) but I only specified one
- 🤔 **CONFUSION**: No "Next steps" shown - what do I do now?
- ⚠️ **FRICTION**: Had to copy entire long command manually

**Noob Thought:** "Cool, it worked! Now what? Let me try the visualization tool..."

---

## Step 7: Trying Visualization

**Action:** Try pt07 to see the results visually

**Noob Thought:** "I want to see what I analyzed. Let me try pt07"

```bash
./parseltongue pt07 --help
```

**Output:**
```
Tool 7: Visual analytics for code graphs

Usage: parseltongue pt07 <COMMAND>

Commands:
  entity-count  Entity count bar chart visualization
  cycles        Circular dependency detection visualization
  help          Print this message or the help of the given subcommand(s)
```

**Observation:**
- ✅ Good: Two clear visualization options
- 🤔 **CONFUSION**: No examples shown
- ⚠️ **FRICTION**: Need to get help on subcommand to know what flags are required

**Action:** Get help on entity-count
```bash
./parseltongue pt07 entity-count --help
```

**Output:**
```
Entity count bar chart visualization

Usage: parseltongue pt07 entity-count [OPTIONS] --db <db>

Options:
      --db <db>        Database file path
      --include-tests  Include test entities (default: implementation-only)
  -h, --help           Print help
```

**Observation:**
- 🔴 **CRITICAL FRICTION #4**: --db is REQUIRED again! Must copy the full path AGAIN
- ⚠️ **FRICTION**: No default database path
- ⚠️ **FRICTION**: No example showing the full command

**Noob Thought:** "Ugh, I need to copy that long database path again..."

**Action:** Run visualization (copy/pasting database path for the THIRD time)
```bash
./parseltongue pt07 entity-count --db "rocksdb:parseltongue20251126182052/analysis.db"
```

**Output:**
```
Running Tool 7: Visual Analytics
📊 Generating entity count visualization...
╔═══════════════════════════════════════════╗
║     Entity Count by Type (Impl Only)      ║
╠═══════════════════════════════════════════╣
║ Module     [██████████░░░░]   3  (75%)  ║
║ Enum       [███░░░░░░░░░░░]   1  (25%)  ║
╚═══════════════════════════════════════════╝

Total Implementation Entities: 4
```

**Observation:**
- ✅ **EXCELLENT**: Beautiful ASCII visualization!
- ✅ Good: Clear and helpful
- ✅ Good: Shows the 4 entities we saw earlier
- ⚠️ **FRICTION**: But I had to type that database path THREE TIMES NOW

---

## COMPREHENSIVE FRICTION ANALYSIS

### Critical Friction Points (Must Fix)

**🔴 FRICTION #1: Database Path Repetition (Severity: HIGH)**
- **Issue**: Must copy/paste `rocksdb:parseltongue20251126182052/analysis.db` for EVERY command
- **Occurrences**: 3 times in basic workflow (pt02-level00, pt07 entity-count, pt07 cycles)
- **Impact**: Extremely tedious, error-prone, breaks flow
- **Solution Needed**: Session state or auto-detection

**🔴 FRICTION #2: Workspace Path Repetition (Severity: HIGH)**
- **Issue**: Must manually copy `parseltongue20251126182052` into output paths
- **Occurrences**: Every export command
- **Impact**: Tedious manual copy/paste
- **Solution Needed**: Auto-detect workspace or session state

**🔴 FRICTION #3: Mandatory --where-clause "ALL" (Severity: MEDIUM)**
- **Issue**: Must type `--where-clause "ALL"` for every pt02 command
- **Occurrences**: Every single export (pt02-level00, pt02-level01, pt02-level02)
- **Impact**: Repetitive typing, "ALL" is the common case
- **Solution Needed**: Make "ALL" the default

**🔴 FRICTION #4: Long Commands with Backslashes (Severity: MEDIUM)**
- **Issue**: "Next steps" show multi-line commands with `\` which are hard to copy
- **Impact**: Must carefully select and copy, error-prone
- **Solution Needed**: Show single-line commands or provide copy-friendly format

### Confusion Points (Documentation Needed)

**🤔 CONFUSION #1: "CozoDB" - No Explanation**
- First-time users don't know what CozoDB is
- No link to docs or quick explanation

**🤔 CONFUSION #2: "rocksdb:" Prefix**
- Unclear why some examples show `rocksdb:` and default is just `parseltongue.db`
- When do I need the prefix vs not?

**🤔 CONFUSION #3: "ISGL1 keys"**
- Technical term with no explanation
- Appears in pt01 description

**🤔 CONFUSION #4: "Progressive Disclosure"**
- Jargon that noobs won't understand
- What does "Level 0, 1, 2" actually mean?

**🤔 CONFUSION #5: "Errors encountered: 36"**
- Sounds alarming but no explanation
- Are these bad? Should I worry?

**🤔 CONFUSION #6: Only 4 Entities from 52 Files**
- Seems low - is this normal?
- What about the other 48 files?

### Positive Observations

**✅ Automatic Workspace Creation**
- Creating timestamped workspace is excellent
- Clean isolation per analysis session

**✅ Clear Output Messages**
- Progress messages are helpful
- Metrics (edges exported, token estimates) are valuable

**✅ "Next Steps" Guidance**
- Showing actual commands is very helpful
- Reduces guessing about workflow

**✅ ASCII Visualizations**
- pt07 output is beautiful and clear
- Makes data accessible

**✅ Examples in Help**
- pt01 --help shows real examples
- Very helpful for learning

---

## Summary: The "Yes No" Problem

### What the User Meant by "Press Yes No"

The user's complaint about "pressing yes no" refers to **DECISION FATIGUE**, not literal yes/no prompts:

1. **Decision**: "Do I need to specify --db?"
   **Answer**: Yes (every single time)

2. **Decision**: "What workspace path do I use?"
   **Answer**: Copy/paste manually (3+ times)

3. **Decision**: "Do I need --where-clause?"
   **Answer**: Yes, must type "ALL" every time

4. **Decision**: "What's the database path format?"
   **Answer**: Figure out rocksdb: prefix vs plain path

5. **Decision**: "What command do I run next?"
   **Answer**: Read "Next steps", manually copy command

### The Core Problem

**Every command requires manual decision-making and copy/pasting:**
- No session memory
- No smart defaults
- No command chaining
- Every flag must be explicitly specified

### The User Experience

```
Command 1: ./parseltongue pt01 .
    ↓
Copy workspace path manually
    ↓
Command 2: ./parseltongue pt02-level00 --where-clause "ALL" --output PASTE_PATH/edges.json --db "rocksdb:PASTE_PATH/analysis.db"
    ↓
Copy database path manually AGAIN
    ↓
Command 3: ./parseltongue pt07 entity-count --db "rocksdb:PASTE_PATH/analysis.db"
    ↓
REPEAT for every visualization/export
```

**This is the "yes no" friction** - constant manual decisions and repetitive copy/paste actions.

---

## Recommendations

### Quick Wins (Easy to Implement)

1. **Session state file** - Write `.parseltongue-session` after pt01 with workspace/db paths
2. **Make --where-clause default to "ALL"** - 90% use case
3. **Make --output auto-generate in workspace** - Smart defaults
4. **Make --db optional** - Auto-detect from session
5. **Show single-line commands** - Remove backslashes from "Next steps"

### Medium Effort

6. **Add `parseltongue analyze`** - One command for full workflow: pt01 + pt02-level00 + pt07
7. **Add workspace commands** - `parseltongue ws list`, `parseltongue ws use <name>`

### Long Term

8. **Interactive mode** - `parseltongue interactive` asks questions once, runs pipeline
9. **Config file support** - `.parseltongue.toml` for project defaults

---

## Test Complete

**Total Commands Run:** 5
**Manual Copy/Paste Operations:** 5
**Friction Points Encountered:** 4 critical, 6 confusing
**Time Spent:** ~3 minutes
**Noob Experience:** 😐 Functional but tedious
