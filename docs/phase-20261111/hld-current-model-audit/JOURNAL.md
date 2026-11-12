# Journal — HLD-current model audit packet

Append-only. This packet is documentation/setup only; no Rust code or
production HLD was changed.

## 2026-11-12 — documentation packet start

- Base: `31397b39deaa098fdcbb9fcf8e527e4de81d6efb`; isolated branch/worktree.
- Contract: `REQ-RUST-025.0`; named future validator tests remain PENDING.
- Source pin checked: serde `aa7913ad0211fef960df68e7812a6211145a888c`;
  source checkout clean. Bounded Pass A scope only; no production HLD export
  was supplied, so Pass B is BLOCKED/NOT STARTED.
- Skills applied: `rust-executable-specs-01` (Spec/Review/Reliability order)
  and `idiomatic-rust-coder-01` (TDD STUB→RED→GREEN→REFACTOR). Exact named
  TDD plugin was not available in plugin search; root/scoped AGENTS retains
  its required loop. Documentation validation is structural, not TDD RED/GREEN.
- zcode has not yet been invoked. Record its eventual prompt/output/stderr/time
  artifacts, requested and actually reported model identity, or actual auth
  failure in the separate persistent run directory.

## 2026-11-12 — structural checks and zcode launch retries

- The preceding zcode-not-invoked statement is historical for the packet-start
  entry only. The bounded model request was attempted three times after the
  docs-only packet was pushed. No model response was received; no Pass A
  candidate or actual model identity exists. Authentication remains
  unverified, and Pass B remains BLOCKED/NOT STARTED.
- Structural validation output recorded at packet completion:

  JSON_SCHEMA: Draft-07 valid; NOT_STARTED, frozen draft comparison and fully gated PASS records accepted
  SCHEMA_GATES: rejected Pass-A production rows, FROZEN without hash/time, comparison before freeze/export, PASS with S9 NOT_RUN, PASS without export, evidence missing byte_end
  STRUCTURE: 4 local links resolve; 4 unique REQ definitions; 11 four-word PENDING tests map to fixtures
  DIFF_CHECK: exit 0
  SCOPE_CHECK: exactly 8 requested docs changed; root AGENTS.md unchanged

- Docs-only packet commit/push: b7517cdd6511350a5ebb32f64157e16e0c4b860f
  on devin/1794502340-hld-model-audit. The packet worktree was clean after
  push. No mapper, comparator, or SOP-runner files were accessed or changed.
- Zcode/CLI diagnostic facts: Claude Code 2.1.319; wrapper SHA-256
  f0dadaf2abf5fd3c7581a9d333ff43bff3c6a6c347ae30bb54bc403608d97ca3;
  CLI SHA-256
  dabafec82ca1319fa43d8be36a10fc61b1d11821cb98ad5df04ad95e199b8eed.
  The wrapper exports the Z.AI Anthropic endpoint
  https://api.z.ai/api/anthropic and model defaults OPUS/SONNET=glm-5.3,
  HAIKU=glm-5.3-flash; credential values were never read or printed.
  Installed CLI help says --model accepts an alias such as sonnet/opus or a
  full model name. --restricted ignores user/project/local settings; this
  launch still receives the wrapper's environment mapping.
- Official documentation consulted:
  https://docs.z.ai/devpack/latest-model lists GLM-5.3 and GLM-5.3-Flash
  for all plan tiers, gives glm-5.3 as a model example, and documents the
  Claude Code endpoint above.
  https://docs.z.ai/devpack/tool/claude documents
  ANTHROPIC_DEFAULT_OPUS_MODEL, ANTHROPIC_DEFAULT_SONNET_MODEL, and
  ANTHROPIC_DEFAULT_HAIKU_MODEL custom-model mappings; its GLM-5.3 example
  maps OPUS/SONNET to glm-5.3.
- Attempt 1 — launch 2026-11-12T17:11:59Z, shell 444685, CLI
  --model glm-5.3, prompt SHA-256
  b9380130db7dd736a1a2be92039e7fee67e4e6f48874edd0eb7a0b88910980e8.
  Stderr was exactly
  [claude-code:unrecognized_model] {"model":"glm-5.3","query_source":"sdk"}.
  Stdout was empty (SHA-256
  e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855),
  stderr SHA-256
  089dbf616da27f2f327878d5fb2ed70fd63a63b259b7c648536429d4b1c68609;
  time file was empty. Shell stopped after the error; no exit code or elapsed
  time was captured. Attempt metadata:
  /home/ubuntu/zcode-runs/20261112-hld-current-model-audit/run-metadata.json
  (SHA-256
  ebd9a3e0b3b281c30bebe3d532d7e55fa68a3843f586cd4784dda4d21fbe073e).
  Output/stderr/time paths are zcode-output.json, zcode-stderr.log, and
  zcode-time.txt in that run directory.
- Attempt 2 — launch 2026-11-12T17:17:01Z, shell a87479, --model sonnet
  using the wrapper's ANTHROPIC_DEFAULT_SONNET_MODEL=glm-5.3 mapping. The
  identical SDK-tagged model error occurred; stdout was empty, stderr was 74
  bytes with the same SHA-256 as attempt 1, and the time file was empty.
  Attempt artifacts: retry-02-metadata.json (SHA-256
  ad444cfec3143fb648c4d5153bd2ec2be32bf5b669d88da110231a37600ebad6),
  retry-02-result.json (SHA-256
  d8be16d105ff71c2bc0a2a41db34af37a730f62e6dba30581942d161defb3bfe),
  retry-02-zcode-output.json, retry-02-zcode-stderr.log, and
  retry-02-zcode-time.txt.
- Attempt 3 — launch 2026-11-12T17:18:56Z, shell 43fadf, --model opus
  with per-process ANTHROPIC_DEFAULT_OPUS_MODEL=glm-5.3. The identical
  SDK-tagged model error occurred; stdout was empty, stderr was 74 bytes with
  the same SHA-256, and the time file was empty. Attempt artifacts:
  retry-03-metadata.json (SHA-256
  2caf3c2598ba2a874273f174129297637f389e211bd1698472f8f08eb50897c4),
  retry-03-result.json (SHA-256
  30fb01965ab35f2b4456da20b81fdcb313d4440dd2b68a79c4f1b4a828cb6d80),
  retry-03-zcode-output.json, retry-03-zcode-stderr.log, and
  retry-03-zcode-time.txt.
- The unchanged prompt and source/code-graph record remain at
  /home/ubuntu/zcode-runs/20261112-hld-current-model-audit/; prompt SHA-256 is
  b9380130db7dd736a1a2be92039e7fee67e4e6f48874edd0eb7a0b88910980e8,
  and serde-codegraph-index-summary.md SHA-256 is
  0d5c71e1b422e29bf943beadb7bf700ed14ef9b3412ad5f4b12dd38cfb4bf563.
- Diagnosis boundary: direct-name and both documented alias-mapping launches
  all resolved to the configured glm-5.3 string, then emitted unrecognized_model
  with query_source=sdk. Official docs say that model and endpoint are
  supported. This output does not establish whether the rejection came from
  Claude Code's SDK validation or the provider, and is not an authentication
  result. Do not change/downgrade the model or claim authentication until a
  supported model request returns. No debug logging was enabled because it
  could expose authorization data.
- Historical timing limitation: launch timestamps above were printed by the
  harness; /usr/bin/time -v files are zero bytes because each shell was
  stopped after the immediate error, so elapsed durations and exit codes are
  unavailable. Each error was observed in a 10-second check window before
  stopping. No zcode run artifacts contain credentials.
