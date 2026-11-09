# SOP — Using the Z.ai GLM Coding Plan ("zcode") as a second-opinion reviewer and worker from this repo

*Written 2026-11-06 from a working setup in the Devin VM. Everything in §3 (quotas, prices, endpoints) was read live from Z.ai's monitor API and docs.z.ai that day; re-check before relying on numbers.*

## 1. What it is, in one line
A GLM-5.3 / GLM-5.3-Flash agent (Claude Code harness) pointed at Z.ai's Anthropic-compatible endpoint, run **non-interactively** from the repo checkout so it can read the files itself. Used for: independent critique of research notes, summarising long docs, drafting, cheap bulk labelling. Not used for: anything that must be exact (that is the resolver's job) and not inside Parseltongue itself (CPU-only, zero LLM — the LLM is always the *caller*).

## 2. Setup (already done in the VM; repeat on a new machine)
```text
   Node 22 via nvm  →  npm i -g @anthropic-ai/claude-code  (2.1.318 tested)
   ~/bin/zcode      →  launcher below
   ~/.claude/settings.json  →  model slot mapping only; NO token in the file
   key              →  ZAI_API_KEY in the process env, from a session secret; never on disk, never committed
```
`~/bin/zcode`:
```bash
#!/usr/bin/env bash
export NVM_DIR="$HOME/.nvm"; . "$NVM_DIR/nvm.sh" >/dev/null
export ANTHROPIC_BASE_URL="https://api.z.ai/api/anthropic"
export ANTHROPIC_AUTH_TOKEN="${ZAI_API_KEY:?set ZAI_API_KEY}"
export ANTHROPIC_DEFAULT_OPUS_MODEL="glm-5.3" ANTHROPIC_DEFAULT_SONNET_MODEL="glm-5.3" ANTHROPIC_DEFAULT_HAIKU_MODEL="glm-5.3-flash"
export API_TIMEOUT_MS=3000000 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
cd "${ZCODE_DIR:-/home/ubuntu/repos/parseltongue-rust-LLM-companion}"
exec claude "$@"
```
Gotchas met while setting up:
- `${VAR}` inside `settings.json` is **not** expanded by Claude Code → 401 "token expired". Put the token in the env only.
- A pay-as-you-go Z.ai key authenticates (lists models) but every completion returns **1113 "Insufficient balance or no resource package"**. You need a key created under the **Coding Plan** account.
- Claude Code logs `unrecognized_model glm-5.3` / `glm-5.3[1m]` and assumes a 200k window. Cosmetic; the call still works. For real 1M context set `glm-5.3[1m]` **and** `CLAUDE_CODE_AUTO_COMPACT_WINDOW=1000000`.
- `claude -p` with no prompt argument and no stdin errors out: pipe the prompt file in.

## 3. Quota facts (live, Max plan, 2026-11-06)
```text
   endpoint for the agent      https://api.z.ai/api/anthropic           (Claude Code, Goose)
                               https://api.z.ai/api/coding/paas/v4      (OpenAI-compatible tools)
                               https://api.z.ai/api/v1                  (Codex)
   models on the plan          glm-5.3, glm-5.3-flash  (+ [1m] variants; 4.7/5.1/5.2 silently routed to 5.3 / 5.3-flash)
   credits                     5-hour: Lite 2,000 | Pro 12,000 | Max 28,000      weekly: 10,000 | 60,000 | 140,000
   credit formula              (input×6.9 + cached×1.7 + output×24) / 10,000   for GLM-5.3
                               (input×2.3 + cached×0.56 + output×8) / 10,000   for GLM-5.3-Flash   (~3× cheaper)
   off-peak                    50 % of the credit rate. Peak = Mon–Fri 14:00–18:00 Singapore (11:30–15:30 IST). Weekends all off-peak.
   Max plan ≈                  0.7–1.35 B GLM-5.3 tokens/week at 95 %+ cache hit (Z.ai's own estimate)
   MCP tools                   web search / web reader / zread: separate allowance (4,000 calls seen on this account), credit 1.2/call
   no free daily tokens        the "free" things are: off-peak 50 %, the MCP allowance, [1m] context, invite-friends credits (3 paying referrals)
   plan quota only             when quota is exhausted the call fails; it never bills the account balance
```
Live read of what is left (before any big run):
```bash
curl -s -H "Authorization: Bearer $ZAI_API_KEY" https://api.z.ai/api/monitor/usage/quota/limit | python3 -m json.tool
#   TOKENS_LIMIT.percentage = % of the 5-hour token budget used; nextResetTime = ms epoch
#   TIME_LIMIT (usageDetails search-prime/web-reader/zread) = MCP call allowance
S=$(date -u -d '-7 days' '+%Y-%m-%d %H:%M:%S'); E=$(date -u '+%Y-%m-%d %H:%M:%S')
curl -s -G -H "Authorization: Bearer $ZAI_API_KEY" --data-urlencode "startTime=$S" --data-urlencode "endTime=$E" \
     https://api.z.ai/api/monitor/usage/model-usage | python3 -c 'import sys,json;print(json.load(sys.stdin)["data"]["totalUsage"])'
```

## 4. The procedure
```text
   0. CHECK      quota/limit above. If 5-hour percentage > 70 %, wait or use glm-5.3-flash.
   1. TIME IT    big reads (whole devin-research/, 60k+ tokens) outside 11:30–15:30 IST Mon–Fri → half the credits.
   2. PICK MODEL flash for: read-and-summarise, extract, bulk label, grep-like questions.
                 5.3   for: judgement, critique, design opinions, anything you will quote.
                 (slot mapping: ANTHROPIC_DEFAULT_*_MODEL in ~/bin/zcode; override per run with --model)
   3. WRITE THE PROMPT TO A FILE   /home/ubuntu/zcode-runs/<name>.txt  — ask for: restate-the-thesis first (proves it read),
                 numbered sections, cite file names / H-numbers, a ranked "test first" and "cut" list, a verdict.
   4. RUN READ-ONLY
                 export ZAI_API_KEY="$ZAI_CODING_PLAN_API_KEY"
                 ~/bin/zcode -p --allowedTools "Read,Glob,Grep,LS" --output-format text \
                     < zcode-runs/<name>.txt > zcode-runs/<name>.md 2> zcode-runs/<name>.err &
                 (add Edit,Write,Bash only for a task you want it to change files for; never with --dangerously-skip-permissions
                  on a branch you have not committed)
   5. READ       zcode-runs/<name>.md is the final answer; .err has the harness log. Keep both.
   6. RECORD     paste the opinion verbatim into devin-research/ with a one-paragraph "where we disagree"; commit.
                 Never silently merge its claims into our notes — it is one more annotation with creator = glm-5.3@date (H-51 applies to us too).
```

## 5. Run log
| date | model | task | input read | wall time | result |
|---|---|---|---|---|---|
| 2026-11-06 | glm-5.3 | smoke test: question about experiments/public-interface-semver/ | 1 dir | 7.8 s | ok |
| 2026-11-06 | glm-5.3 | critical review of all of devin-research/ (18 md files, ~43.5k words) + experiments/public-interface-semver/ | ~27k tokens billed (Claude Code reads selectively, well under the 60–70k raw size) | 5 m 28 s | 2,262-word review, saved verbatim in `1793968123-second-opinion-glm-5-3-independent-review-of-all-devin-research-with-where-we-disagree.md` |

## 6. Lessons from the first full run
- Claude Code does not ingest the whole folder; it Globs, then Reads what it decides it needs. 43.5k words of notes cost ~27k tokens, so a full-corpus review is cheap (≈ 1 % of the 5-hour Max budget).
- 5½ minutes wall time for 18 files + a 2,300-word answer. Budget 5–10 min for a review of this size; don't poll, wait on the process.
- Waiting with `while pgrep -f "claude -p"` matches the waiting shell's own command line and never exits. Use `pgrep -x node` filtered on the claude path, or `wait` on the backgrounded PID.
- The `[1m]` unrecognized-model warning in `.err` is harmless; the run used glm-5.3.
- The output is a strong independent read when the prompt forces a restatement first and demands file/H-number citations; it caught two real inconsistencies between our own notes that we had not.

## 6. Persisting across Devin sessions (added 2026-11-09)
```text
   key        saved as the PERSONAL Devin secret ZAI_CODING_PLAN_API_KEY (Settings → Secrets → Personal). Injected into every
              new session's env; never on disk, never in settings.json, never committed. Not visible to snapshot builds (personal scope).
   harness    repo blueprint installs nvm → Node 22 → @anthropic-ai/claude-code and writes ~/bin/zcode, so a fresh session has `zcode`.
   key order  the launcher reads ZAI_CODING_PLAN_API_KEY first, ZAI_API_KEY second. Gotcha found 2026-11-09: a session may also carry a
              pay-as-you-go ZAI_API_KEY; if the launcher prefers that one, every call fails with 1113 and `claude -p` appears to hang.
   smoke      echo 'Reply with exactly the word pong.' | ~/bin/zcode -p --output-format text --model glm-5.3-flash   → pong (~5 s)
   quota      curl -s -H "Authorization: Bearer $ZAI_CODING_PLAN_API_KEY" https://api.z.ai/api/monitor/usage/quota/limit
```
