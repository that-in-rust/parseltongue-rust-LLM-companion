# Assessment: how far the itoa graph is, and whether the process is sound

Dated review at commit `dc08a0bed` (2026-11-16). Not a document in force. Companion to
[README.md](README.md). Nothing was built or run for it: this VM has no `~/repos/rust-tools`.
Every count is quoted from the LOG, the SOP, the expected-graph summary or a source read, and
says which. `AGENTS.md` Rule 1 forbids bare percentages; each one below sits beside its N of M.

## The answer

**By stored rows the graph is about 80% of the goal: all 151 nodes, and 549 of 682 edge rows
with a target. By the SOP it passed at `822da80cf` and has no valid status at HEAD. By work
left I judge it about two thirds done, because the missing fifth is the hardest part: type
inference for 124 method calls into `core`. The acceptance method is sound. Three habits are
working against it: PASS is measured against an expected graph that already contains the gap,
types are resolved by matching text, and state is copied into eight documents that then go stale.**

## 1 — Percent against the goal

Goal (PRD): every item outside a function body is a node, and every source-proven dependency
is an edge with evidence, equal to the frozen expected graph.

```text
measure                               accepted run       part 8, pushed     part 8, HEAD (WIP)
                                      822da80cf          scope declarations 8 targets unchecked
nodes equal to expected               151 of 151  100%   153 (2 new)        not run
edge rows stored                      682 of 682  100%   682                682
edge rows with a target               549 of 682   80%   558 of 682  82%    566 of 682  83%
calls with a target                    64 of 197   32%    73 of 197  37%     81 of 197  41%
method calls into core, named           0 of 133    0%     9 of 133   7%     17 of 133  13%
SOP steps valid for this commit         4 of 4             0 of 4             0 of 4
node fields ARCH requires               0 of 3  (visibility, reachable outside crate, signature)
```

Source: LOG "Where we are" and the stop entry of 19:53 IST.

**What is left, in the order part 8 plans it:**

- **8.3, 87 more calls predicted** (`write`, `get_unchecked`, `as_mut_ptr`, `cast`; the spec
  predicts 96 for the step, not measured). 5 `len` calls may stay without a target by design.
  If it lands: about 155 of 197 calls (79%), 640 of 682 edge rows (94%).
- **8.4, 26 calls** (`try_into`, `expect`, `unwrap`): need the return type of a `core` method.
- **8.5, 11 calls** (`ilog10`, `rev`): need `core`'s own macros expanded.
- **8.6:** re-freeze the expected graph and repeat S1 to S4.
- **Outside part 8:** the three node fields; rows that change with the source directory;
  `test_lang_items_in_sysroot`; two open decisions (fields and variants as nodes, module to
  file-contents edge).

**My judgement of work left: about two thirds done.** 8.3 reuses what exists. 8.4 and 8.5
each need a capability the code does not have (return-type substitution, expansion inside
`core`). Part 8 so far cost 16 commits and 2,173 production lines for 9 checked targets.

## 2 — What the process gets right

- **One objective target.** A frozen expected graph, compared by byte range, on a frozen
  executable, with digests. Few projects have a check this exact.
- **Strict words.** PASS belongs to one executable; a production change voids it. The LOG
  applied this to itself: "The acceptance run covers no commit after it."
- **A missed target over a wrong one.** Unresolved rows are stored with a reason, never guessed.
- **The process audit.** `strace` proves only three child commands run. The code agrees:
  one production `Command::new`.
- **Determinism is measured,** at 1, 4 and 16 workers.
- **The stop entries are honest.** The 19:53 entry has a NOT KNOWN section. That is rare and
  worth keeping.
- **Scope was cut to one small crate** after the five-repository attempt.

## 3 — What is going wrong

### 3.1 PASS measures agreement, not completeness

- The expected graph has 133 calls with no target and no node visibility. The mapper matches
  it, so S1 to S4 pass while the graph is `incomplete`.
- By the written rule (`AGENTS.md`: DONE when the run equals the frozen graph) the phase was
  DONE at `822da80cf`. Work went on into part 8, so the real goal is past the written one.
- No document says how many of the 197 calls must have a target, or which may stay without.
- **Fix:** state DONE as a count: calls with a target N of 197, plus a named list of the
  calls allowed to stay unresolved and why.

### 3.2 The expected graph and the mapper share one author

- The summary file says so itself: `review: AUTHOR_ONLY_NO_SECOND_REVIEWER`,
  `independent_compiler_oracle_verification: NONE`, `whole_source_coverage_claimed: false`
  (operators and literals are not covered).
- Rule 5 (second opinion) was retired on 2026-11-16 and the rust-analyzer oracle is out of
  scope. So a mistake made the same way twice passes.
- **Fix:** one test-only check of the expected call targets against rust-analyzer on itoa.
  It touches expected data only, never production. This needs your decision, because the PRD
  rules the oracle out.

### 3.3 Part 8 writes expected rows after the run, not before

- SOP Rule 1: the expected graph comes from source and is frozen before comparison.
- Part 8 order: build the lookup, see which calls the run names, then check those against source
  ("the 8 new targets are not checked against the source, so the list is not updated").
  The re-freeze is the last step, 8.6.
- The targets are already known from the part 8 measurement table (`MaybeUninit<T>::write`
  for 49 sites, `[T]::get_unchecked` for 39, and so on).
- **Fix:** write the expected target for all 133 calls from source now, freeze, then build.
  The failing rows then choose the work, as SOP Rule 2 says.

### 3.4 Types are resolved by matching text

- `read_self_alias_parts` finds `Self::NAME` with `strip_prefix("Self::")` on the stored
  type text (`core-member-target-records.rs:536-549`).
- The LOG records the stored text of a macro-written type as
  `[MaybeUninit <u8 >; Self ::MAX_STR_LEN ]`, with spaces. A prefix match on `Self::` cannot
  match `Self ::`. This fits the LOG's open question (37 of 46 calls did not reach the rule).
  Read from source, not run.
- The cause is upstream. PRD stage 6 is "lowered signatures", and
  `lowered-signature-type-table` is registered, but nothing writes it. Each resolver re-reads
  type text its own way, so each new receiver shape needs a new requirement (054 to 057 for
  9 targets) and `build_original_call_edge` is now 336 lines.
- **Fix:** before 8.4, build stage 6 once: a structured type row (path, generic arguments,
  reference, array, slice, pointer) written from syntax, not text. 8.4 needs it anyway to
  carry a return type.

### 3.5 Part 8 broke three written rules

- **`AGENTS.md` line 203:** an architecture change updates ARCH, SOP and `AGENTS.md` in the
  same commit. Part 8 added a stage, a table and a flag value; none of the three changed.
- **SOP Rule 2:** "Production reads only the pinned source." Part 8 reads `core` source.
- **SOP S1:** inputs are frozen by digest. The LOG says the run id "does not hash the core
  source, only the compiler version". A new input is outside the freeze.

### 3.6 State is copied into eight documents

- Six of the eight are stale within two days (table in [README.md](README.md), section 4).
  `AGENTS.md` still says "a bounded checkpoint" and that a run with `core` excluded "cannot
  pass"; the SOP says it passed.
- Since 2026-11-13 (227 commits, `git log --numstat`): Markdown under `docs/` and
  `AGENTS.md` gained 38,649 lines and lost 11,303; all of `crates/` gained 37,712. As many
  lines of prose were written as of code and tests together.
- **Fix:** only the LOG's "Where we are" page holds status and numbers. PRD, ARCH, MAP,
  SPEC README and `AGENTS.md` hold rules and link to it.

### 3.7 Smaller points

- **The comparison is narrower than SOP Rule 3 says.** Not compared: lookup evidence on a
  row, the range of a macro-written node, the range of `contains` and `implements` rows,
  visibility and signature.
- **Determinism is proven for one directory only.** Rows depend on the absolute source path,
  and S1 fixes it to `/home/ubuntu/repos/rust-tools/itoa`. The same source in another
  directory stores different ids.
- **The build is not reproducible from the repository.** It needs a patched rust-analyzer
  checkout beside it. This session's VM had none, so no gate could run.
- **Cost.** 15 to 19 s and 370 to 414 MB for 488 lines. A run that reads the whole sysroot
  fails after 171 s at 1,556 MB (part 8 measurement). Performance is out of scope, but an
  itoa-only gate cannot show whether the design scales.
- **Rule 4 costs more than it returns.** codebase-memory is required at nine named points. Its call edges are name matches (it reported 331 calls across a boundary with no
  Cargo dependency) and it parses the SQL files only partly.
- **Out-of-scope code is still in every gate:** about 4,700 lines of HLD and oracle code and
  6 tables compile and test on each run.

## 4 — What I would do next, in order

1. Close or revert the WIP at HEAD so the branch has a passing head.
2. Write the expected targets for the 133 calls from source and freeze them (3.3).
3. Put a number on DONE in the PRD (3.1).
4. Update SOP Rule 2, S1 and ARCH for the `core` read, and hash the `core` files (3.5).
5. Build the structured type rows before step 8.4 (3.4).
6. Move all status to the LOG page and strip it from the other documents (3.6).
