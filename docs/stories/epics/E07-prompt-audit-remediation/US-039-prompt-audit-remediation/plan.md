# US-039 implementation plan — prompt-audit remediation

Spec: `US-039-prompt-audit-remediation.md` (same folder). Audit source: the
`/claude-api prompt-audit` report of 2026-09-07; its findings are restated
inside each task so no task needs the report.

## Global Constraints

- Every edit under `.claude/skills/reverse-tornado-okr/` is mirrored
  byte-for-byte into `.codex/skills/reverse-tornado-okr/` in the same commit.
- Never hand-edit `docs/TEST_MATRIX.md`, `docs/HARNESS_BACKLOG.md`,
  `docs/decisions/README.md`, or `harness.db`. Write harness state only through
  `scripts/bin/harness-cli`.
- Gate semantics do not change. Only wording, facts, and enforcement change.
- Prose style for anything the model reads: plain sentences, the reason next
  to the rule, no caps emphasis, no "never/always" clusters without a reason.
- Each task ends by running its listed verification commands and pasting the
  output into its report file. A task whose verification fails is not done.
- Commits: one task per commit unless the task says otherwise; stage
  `.harness/events/` and regenerated views with the commit. Commit messages end
  with `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>`.
- Verify-harness baseline before any task: `bash scripts/verify-harness.sh .`
  must print 8/8 and must still print all checks passing after every task.

---

### Task 1: Baseline fixtures and OKR verifier contract (Phase 0 + Phase 1)

Files: `.claude/skills/reverse-tornado-okr/contracts/handoff-contract.v1.json`
(rename to `handoff-contract.v2.json`), `.claude/skills/reverse-tornado-okr/scripts/okra-verify-artifact.py`
(DEFAULT_CONTRACT path), `.claude/skills/reverse-tornado-okr/SKILL.md`,
`.claude/skills/reverse-tornado-okr/references/integrity-store.md`, and the
`.codex` mirror of each.

Step A, baseline. Write a fixture artifact `/tmp/us039/fixture-okr-artifact.md`:
a delegated-loop OKR artifact for the goal "raise weekly newsletter open rate
from 31% to 40% without raising unsubscribe rate above 0.5%", written by
following SKILL.md's Output section literally (include every exact sentence
and key SKILL.md asks for, the four flags, eval points, frame/tree schema,
worker prompt packet contract, DKR-to-DKR handoff, candidate anti-goal
library with `trace_manifest_ref`, and worker progress reports under
`.okra/runs/<run-id>/workers/`). Run the verifier on it with the v1 contract
and save the JSON output to `/tmp/us039/baseline-verify.json`. Expected:
INCOMPLETE, with `target_instantiation` and at least `orchestrator_ownership`
or `worker_progress_reports` in `missing`. Record the exact missing list in
the report.

Step B, contract fixes (new file `handoff-contract.v2.json`, `contract_version`
`okra.handoff-contract.v2`; delete the v1 file; point `DEFAULT_CONTRACT` at v2):

1. Delete the requirement with id `target_instantiation` entirely.
2. In `orchestrator_ownership`, drop the trailing period from the token so it
   reads `... until the objective metric reaches target` (no period).
3. In `worker_progress_reports`, replace token `.okra/workers` with
   `/workers/`; in its hint replace `under .okra/workers/` with
   `under .okra/runs/<run-id>/workers/`.
4. In `candidate_antigoal_library`, add `trace_manifest_ref` to the group whose
   tokens are `trace evidence`, `trace_evidence`, `trace refs`.
5. Delete the top-level keys `coherence_review` and `coherence_review_source`.
6. In the top-level `description` and `source` strings, remove every mention of
   hidden evals, blindbox, rubric, regex adjacency, scoring, or "case prompt".
   Replace `source` with: `skills/reverse-tornado-okr/SKILL.md (Output section
   for delegated loops). Every requirement cites a public source.` For each
   requirement whose `source` mentions "case prompt", change it to the SKILL.md
   section that documents it (Step 2e for `accumulated_governance`; Output
   for the rest).
7. Update the verifier's module docstring: remove the sentences about the
   hidden eval rubric and blindbox; keep the usage and exit-code lines.

Step C, SKILL.md wiring (Output section):

1. Replace lines 406-408 (the bullet starting "The orchestrator owns objective
   checks") with:
   `- Write this sentence once, exactly: **"The orchestrator owns objective checks, check-ins, the OKR board, and subagent steering until the objective metric reaches target."** Follow it with the domain target and the note that a human or a blocking flag can also stop the loop.`
2. After the artifact-guide paragraph that ends the Output section (the one
   ending "no decoration that does not carry meaning)."), insert:

   ```
   Before you hand over a delegated-loop artifact, run the skill's own completeness gate and repair
   anything it reports missing:

   ```bash
   python3 .claude/skills/reverse-tornado-okr/scripts/okra-verify-artifact.py <artifact.md>
   ```

   It checks the artifact against `contracts/handoff-contract.v2.json`. That file is the checked
   source of truth for the exact keys and sentences listed above; if this prose and the contract
   ever differ, the contract wins and this file needs fixing.
   ```
3. In Step 2c, after the sentence ending "`ungoverned_direct_write`, and
   `single_llm_truth`." append: `For memory-governance anti-goals (Step 2e),
   state these three in the artifact as `unratified_memory_promotion_count == 0`,
   `single_llm_truth_acceptance_count == 0`, and `eval_regression_count == 0`.`
   Append the same sentence in `references/integrity-store.md` after the line
   that names `single_llm_truth` (around line 127).

Step D, mirror to `.codex/skills/reverse-tornado-okr/` and verify:

- `diff -r .claude/skills/reverse-tornado-okr .codex/skills/reverse-tornado-okr` prints nothing.
- `python3 .claude/skills/reverse-tornado-okr/scripts/okra-verify-artifact.py /tmp/us039/fixture-okr-artifact.md --json` reports `"complete": true`. If a requirement still fails, fix the fixture only if the fixture disobeyed SKILL.md; otherwise fix the contract or SKILL.md and say which in the report.
- `python3 -c "import json;json.load(open('.claude/skills/reverse-tornado-okr/contracts/handoff-contract.v2.json'))"` succeeds.
- `grep -rn 'case prompt\|blindbox\|hidden eval\|rubric' .claude/skills/reverse-tornado-okr/` prints nothing.
- `bash scripts/verify-harness.sh .` prints 8/8.

---

### Task 2: One reading rule in three places, plus its lint (Phase 2, tasks 8 and 26)

Files: `AGENTS.md`, `scripts/install-harness.sh` (the heredoc block around
line 270), `scripts/install-harness.ps1` (the here-string around line 139),
`scripts/verify-harness.sh`.

In all three, replace the block from "This repo uses Harness." through the
`query matrix` bullet with exactly this (the ps1 keeps its Windows path
alternative on the matrix bullet, as today):

```
This repo uses Harness. First-time setup (install + wire the capability tools):
`docs/SETUP.md`. Before work, in every lane:

- read `docs/FEATURE_INTAKE.md`
- run `scripts/bin/harness-cli query matrix` on macOS/Linux, or `.\scripts\bin\harness-cli.exe query matrix` on Windows

Then read the lane-dependent docs that `docs/CONTEXT_RULES.md` prescribes for
your lane (`README.md`, `docs/HARNESS.md`, `docs/ARCHITECTURE.md`,
`docs/TOOL_REGISTRY.md`, `docs/GOAL_LOOP.md`, product docs, stories, decisions).
```

Keep the paragraph that follows ("Use the Rust Harness CLI ... an absent
capability is a clean skip.") in all three; the ps1 currently lacks the
`query tools --capability` sentence, so add it there so the paragraph is
identical to the sh copy apart from the Windows path alternative.

Lint: add a check to `scripts/verify-harness.sh` (as a new numbered check,
raising the total the script prints, e.g. 9/9) that extracts the text between
`## Harness` and the end of the reading paragraph from `AGENTS.md` and from
each installer template, normalizes the Windows path alternative and the
`.exe` suffix away, and fails if the three texts differ. Print which pair
differs.

Verify: `bash scripts/verify-harness.sh .` prints all checks passing;
`bash scripts/test-install-contract.sh` passes; `grep -c 'docs/TOOL_REGISTRY.md' AGENTS.md scripts/install-harness.sh scripts/install-harness.ps1` gives 1 for each.

---

### Task 3: MCP flavor wording and register, plus its lint (Phase 2, tasks 9, 10, 28)

Files: `docs/templates/mcp/session-context.md`, `scripts/lint-mcp-templates.sh`.

1. Replace the Normal bullet's "plus planned `docs/TEST_MATRIX.md` rows;" with
   "which renders the story's row into the generated `docs/TEST_MATRIX.md`
   (never hand-edit that file);". In the High-risk bullet replace
   "`docs/TEST_MATRIX.md` rows;" with "the story row via `harness_story_add`;".
   Replace the done-gate bullet "`docs/TEST_MATRIX.md` rows current, and
   validation commands were actually run." with "The story's proof flags are
   current via `harness_story_update` (the matrix view is regenerated from
   them), and validation commands were actually run." In the preamble template,
   replace `docs/TEST_MATRIX.md,` in the `Docs:` line with `docs/product/...,`.
2. Replace the bold opening of "## Harness Intake Gate":
   `**The intake gate is non-negotiable. Run it BEFORE any tool call that mutates the repo.** User approval ("go ahead", "do it", an approved design) moves work *through* the gate, not *around* it. If you are drafting code before the gate output exists, stop and back up.`
   with:
   `Run the intake gate before any tool call that mutates the repo. User approval ("go ahead", "do it", an approved design) moves work *through* the gate, not *around* it, because the gate is what records lane, flags, and proof for the team. If you are drafting code before the gate output exists, stop and back up.`
   and the heading `### Emit this preamble first, every time` with
   `### Emit this preamble first`. Keep the "If you catch yourself skipping the
   gate" section unchanged.
3. Lint: extend `scripts/lint-mcp-templates.sh` to fail if any file under
   `docs/templates/mcp/` contains a generated view named as an edit target
   (regex: `(TEST_MATRIX\.md|HARNESS_BACKLOG\.md|decisions/README\.md)\` (rows|updated|edit)`).
   Print the offending line.

Verify: `bash scripts/lint-mcp-templates.sh` passes; temporarily inserting
"TEST_MATRIX.md rows" into a scratch copy under `/tmp` and running the lint
logic against it fails (show the output); `bash scripts/verify-harness.sh .`
passes.

---

### Task 4: Stale-fact sweep and PHASE plan removal (Phase 3, tasks 11, 11a-c, 12)

Group commits as you like but keep the PHASE removal as its own commit.

1. `README.md`: line 10 badge `v0.1.10` -> `v0.1.23`. Delete the
   `> [!IMPORTANT]` callout that begins "**This fork is private.**" (all of its
   quoted lines) and the line "The curl one-liners below work as written once
   the repo is made public." Keep the plain curl one-liner.
2. `docs/SETUP.md`: delete the sentence beginning "**While this fork is
   private**" and the `gh`-authenticated alternative block it introduces, keeping
   the anonymous curl flow. Replace the comment "# 1. Add the new files
   (GOAL_LOOP.md, SETUP.md, calibrate-harness.sh, schema\n#    005/006, and any
   docs you were missing) and refresh the agent shim block." with "# 1. Add any
   harness files you are missing and refresh the agent shim block." Replace the
   comment "# 2. Apply new schema — additive and idempotent (adds tool
   kind/capability/scan\n#    columns, story_signal, and schema v7: ULID ids +
   event-log cache tables;\n#    existing rows and data are preserved)." with
   "# 2. Apply schema migrations — additive and idempotent, through the current\n#    version in scripts/schema/ (existing rows and data are preserved)."
3. `docs/TRACE_SPEC.md`: row `id` -> type `TEXT (ULID)`, note "ULID assigned by
   the CLI (schema 007). Do not set manually.", example
   `01JCEXAMPLEULID000000000`; row `intake_id` -> type `TEXT (ULID)`, note "ULID
   of the related `intake` row.", same example. Replace the lines 7-8 sentence
   about `001-init.sql` and "not changed by Phase 2" with "The trace schema is
   `scripts/schema/001-init.sql` as amended by `scripts/schema/007-ulid-ids.sql`."
   In the example command near line 171-172, replace `PHASE2.md` with
   `docs/HARNESS.md` in both `--actions` and `--read`.
4. `docs/GOAL_LOOP.md` lines 45-47: replace "`harness.db` is local and
   gitignored, so each install runs that seed once." with "`tool register`
   appends a `tool.register` event to the tracked log, so one teammate's
   registration travels with `git pull`; only the scan result (`status`,
   `checked_at`) is machine-local, so run `tool check` on each machine."
   `docs/IMPACT_ANALYSIS.md` lines 50-51: replace "`harness.db` is local and
   gitignored by design, so each install runs that seed once." with
   "Registration is an event in the tracked log and travels with `git pull`;
   only the scan status is machine-local, so run `tool check` on each machine."
5. `docs/CONTEXT_RULES.md` line 96: replace
   "Read `docs/decisions/0004-sqlite-durable-layer.md`, `scripts/schema/`, and relevant CLI code before planning."
   with "Read `docs/decisions/0009-event-log-source-of-truth.md` (supersedes 0004), `docs/EVENT_LOG.md`, `scripts/schema/`, and relevant CLI code before planning."
6. `docs/HARNESS_COMPONENTS.md` line 45: replace the row with
   "| Sub-agents | `CLAUDE.md` (Fable subagent rule); Claude Code Agent tool | Partial | Subagents are used for hard debugging, architecture, and verification; no repo-defined agent files yet. |".
   Delete the four rows for `PHASE2.md` to `PHASE5.md` (lines 61-64). Delete
   the duplicate `scripts/bin/harness-cli` row (one of lines 129-130).
7. `docs/GLOSSARY.md` Durable Layer entry (around line 105): replace "The
   SQLite database and CLI (`scripts/bin/harness-cli`) that stores operational
   records" with "The git-tracked, append-only event log at `.harness/events/`
   plus the CLI (`scripts/bin/harness-cli`) that writes it. `harness.db` is a
   rebuilt cache that stores those records".
8. `.claude/skills/reverse-tornado-okr/references/learning-memory.md` lines
   69-70: replace "check-ins, worker progress, move results, and content
   hashes." with "check-ins, worker progress, and move results. Content blobs
   live in the shared `.okra/content/sha256/`; the run dir holds only their
   hashes." Mirror to `.codex`.
9. PHASE removal (own commit): `git rm PHASE2.md PHASE3.md PHASE4.md PHASE5.md`.
   Then: `docs/decisions/0007-story-signal-mining.md` line 66 replace the
   `PHASE5.md` citation with `docs/stories/US-024-improvement-proposal-pipeline.md`;
   in `docs/stories/US-008-trace-quality-scoring.md`, `US-009-enriched-friction-query.md`,
   `US-011-backlog-outcome-workflow.md` (PHASE3.md) and `US-012-story-verify-command-field.md`,
   `US-015-story-verify-command.md`, `US-016-auto-trace-scoring-on-write.md`,
   `US-017-pre-close-verification-gate.md` (PHASE4.md) delete the bullet naming the
   PHASE file; in `docs/stories/epics/E02-phase-2-observability-taxonomy/phase-2-progress.md`
   line 5 replace "from `PHASE2.md`" with "planned in `PHASE2.md` (removed; see git
   history before June 2026)". Add a `CHANGELOG.md` entry under the top Unreleased
   or newest section: "Removed `PHASE2.md`–`PHASE5.md` plans; their content lives in
   `docs/HARNESS_COMPONENTS.md`, `docs/HARNESS_MATURITY.md`, `docs/TRACE_SPEC.md`,
   and the shipped stories US-008 to US-024." Do not touch the Rust test strings
   that use "PHASE3.md" as sample data.
10. Backlog items for flag-only findings, via
    `scripts/bin/harness-cli backlog add --title "<t>" --pain "<p>" --risk tiny`
    for each: GLOSSARY.md defines five terms twice; HARNESS_COMPONENTS.md inventory
    stops before decisions 0008-0012 and schema 005-008; two decisions numbered 0007;
    DIAGRAMS.md line 71 example uses nonexistent `story add --diagram`;
    TOOL_MAPPING.md has no per-tool description text for MCP hosts.

Verify: `grep -rn 'PHASE[2-5]\.md' --include='*.md' . | grep -v CHANGELOG.md | grep -v '^./target'` prints nothing; `grep -n 'fork is private\|made public' README.md docs/SETUP.md` prints nothing; `grep -n 'INTEGER' docs/TRACE_SPEC.md` shows no id rows; `bash scripts/verify-harness.sh .` passes; `bash scripts/lint-mcp-templates.sh` passes; `scripts/bin/harness-cli query backlog | grep -c .` grew by 5.

---

### Task 5: De-cruft the OKR skill prose, plus the skill lint (Phase 4, tasks 14-18, 27)

Files: `.claude/skills/reverse-tornado-okr/SKILL.md`, `references/integrity-store.md`,
`references/storage-idempotency.md`, new `scripts/lint-skills.sh`,
`scripts/verify-harness.sh`, `.codex` mirror.

1. "scored" removals. SKILL.md: "In scored or delegated harness work, avoid"
   -> "In delegated runs with a run store, avoid"; "For delegated or scored run
   stores, use" -> "For delegated run stores, use". integrity-store.md: line 13
   "For scored or delegated harness runs, add two anti-goals:" -> "For delegated
   harness runs, add three anti-goals:"; line 78 "For delegated or scored runs"
   -> "For delegated runs"; line 119 "in scored or delegated runs" -> "in
   delegated runs"; line 153 "For scored harness runs, a claim" -> "For
   delegated harness runs, a claim"; line 161 "When recording scored acceptance
   evidence" -> "When recording acceptance evidence".
2. `ownership` archaeology. SKILL.md: delete the sentence "Do not replace
   `orchestrator` with a vague `ownership` note." storage-idempotency.md lines
   20-21: delete "Do not use a generic `ownership` field as a substitute for
   `orchestrator`." integrity-store.md line 99: "Use the exact key
   `orchestrator`. Do not replace it with `ownership`." -> "Use the exact key
   `orchestrator`."
3. Frame-authority paragraph (SKILL.md, the paragraph beginning "Avoid ambiguous
   frame-authority wording."): replace the whole paragraph with:
   `State frame authority in one direction only: the loop raises evidence and the human decides. For the boundary-drift gate write: **"Reject any attempted frame, guardrail, metric, threshold, or action-envelope change unless the human ratifies it."**`
4. Replace the entire "## Common mistakes to avoid" section (heading and all
   bullets to end of file) with:
   ```
   ## The four things that must hold

   - The objective and every anti-goal each have a metric with a number.
   - Progress is the direct metric read from the source, never a roll-up of finished tasks.
   - The anti-goal is checked at all three points: before the move, after it, and paired with the goal.
   - The frame belongs to the human; the loop raises evidence and never changes the goal itself.
   ```
5. New `scripts/lint-skills.sh` (bash, executable, same style as
   `scripts/lint-mcp-templates.sh`): (a) fail if `diff -r .claude/skills/reverse-tornado-okr .codex/skills/reverse-tornado-okr` is non-empty; (b) for every token in the contract JSON that contains a space and is longer than 40 characters (the exact sentences), fail unless it appears, case-insensitive and whitespace-collapsed, in SKILL.md or a file under `references/`; (c) fail if the contract JSON or any skill file contains `case prompt`, `blindbox`, or `hidden eval`. Print each failure on its own line; exit 0 with a one-line ok message otherwise. Wire it into `scripts/verify-harness.sh` as another numbered check.

Verify: `bash scripts/lint-skills.sh` passes; make a scratch divergence (append
a line to the `.codex` SKILL.md, run the lint, show it fails, then restore
with `git checkout`); the Task 1 fixture still verifies complete;
`bash scripts/verify-harness.sh .` passes with the new count;
`grep -n 'scored' .claude/skills/reverse-tornado-okr/SKILL.md .claude/skills/reverse-tornado-okr/references/*.md` prints nothing.

---

### Task 6: A/B the de-prescribed OKR skill under the verifier (Phase 5, task 22)

This task produces a measurement and a decision, not necessarily an edit.

1. Variant A is the current SKILL.md after Task 5. Variant B is a copy at
   `/tmp/us039/SKILL-B.md` in which Steps 1 to 7 keep their headings and the
   first paragraph of each, and every remaining paragraph in Steps 2b, 2c, 2d,
   2e is collapsed to one sentence stating the goal and the gate, with the
   exact keys and sentences moved to a single "Contract" list at the end that
   points at `contracts/handoff-contract.v2.json`.
2. For each variant, write the delegated-loop artifact for a fresh goal, "cut
   median support ticket first-response time from 9 hours to 4 hours without
   raising reopen rate above 8%", following only that variant's text. Save as
   `/tmp/us039/artifact-A.md` and `/tmp/us039/artifact-B.md`.
3. Run the verifier on both; save JSON. Also count, for each artifact: total
   words, number of contract requirements satisfied, and whether the four
   flags, three eval points, and worker prompt packet contract are present.
4. Decision rule: adopt B only if it satisfies every requirement A satisfies
   and is not longer. Otherwise keep A. Write the numbers and the decision to
   the report. If B is adopted, apply it to SKILL.md and mirror to `.codex`,
   then rerun `scripts/lint-skills.sh` and the Task 1 fixture.

Verify: both verifier JSON files exist; the report contains the comparison
table; `bash scripts/lint-skills.sh` passes.

---

### Task 7: Model-fit guidance for Opus 5 and Fable 5.1 (Phase 5, tasks 19, 20, 21, 23, 24, 25)

Files: `CLAUDE.md`, `docs/FEATURE_INTAKE.md`, `docs/templates/mcp/session-context.md`,
new `docs/decisions/0013-keep-verification-steps.md`.

1. `CLAUDE.md`: replace the "## Subagents" section with a section that keeps
   the Fable rule and adds a short policy, in plain prose with reasons:
   delegate independent, sizeable work (wide multi-file investigation,
   genuinely parallel tracks); do not delegate verification or work that takes
   a handful of tool calls, because each subagent re-establishes context and
   you re-read its report; brief once and commit to the result; launch parallel
   agents in one message and keep working while they run; prefer one subagent
   over several for one modest job. Two short paragraphs, no bullets.
2. `CLAUDE.md`: add a section "## Scope and tests" with: report pre-existing
   bugs or unrelated improvements as follow-ups rather than fixing them in the
   same change; keep scratch checks outside the repository (for example under
   `/tmp`) and delete any you added; commit tests only where the story asks for
   them or the neighboring files already keep tests for that kind of change,
   sized like those files. One paragraph.
3. `CLAUDE.md`: add two sentences to the event-log section naming the memory
   surface: lessons from a story go to `harness-cli story signal add` and
   friction to `harness-cli backlog add`; at intake, consult `query signals` and
   `query backlog` before planning.
4. `docs/FEATURE_INTAKE.md`: in both "Done gate" paragraphs (implementation
   notes and change diagrams), add one sentence: "Before reporting progress or
   done, check each claim against a tool result from this session; report only
   work you can point to evidence for, and say plainly what was skipped or
   failed." Add the same sentence to `session-context.md` under "### Required
   before declaring done" as its first bullet.
5. New decision `docs/decisions/0013-keep-verification-steps.md` from
   `docs/templates/decision.md`: title "Keep deterministic verification steps
   under Fable 5.1"; context: Opus 5 guidance removes prompt-level "verify your
   work" scaffolding, Fable 5.1 guidance keeps it, and this harness's done gate
   is deterministic (`story verify`, `check-diagrams.sh`, the independent-check
   rule); decision: keep them, revisit at the next model release; then
   `scripts/bin/harness-cli decision add --id 0013 --title "Keep deterministic verification steps under Fable 5.1" --doc docs/decisions/0013-keep-verification-steps.md`.
6. Check that nothing in the harness prints remaining-context or token
   countdowns to the model: `grep -rn 'tokens remaining\|context remaining\|remaining tokens' CLAUDE.md AGENTS.md docs/ .claude/ scripts/*.sh` must print nothing; record the result.

Verify: `bash scripts/verify-harness.sh .` passes; `scripts/bin/harness-cli query decisions | grep 0013`; `wc -c CLAUDE.md` grew by less than 2500 bytes; `bash scripts/lint-mcp-templates.sh` passes.

---

### Task 8: Story done gate (Phase 6, task 29, plus packet close-out)

Files: the story packet folder, `.harness/events/`.

1. Fill `implementation-notes.html` (from `docs/templates/implementation-notes.html`)
   in the packet folder with the design decisions, deviations, tradeoffs, and
   open questions recorded in every task report under the SDD workspace, plus
   the verification output summary per task.
2. Mark `diagrams/D3-verify-and-lint-flow.md` `reviewed` only if a second
   agent has reviewed it; the controller records that with
   `scripts/bin/harness-cli intervention add --story US-039 --type review --source agent --description "D3 verify-and-lint-flow reviewed: ..."`.
   Run `bash scripts/check-diagrams.sh`.
3. `scripts/bin/harness-cli backlog add --title "Re-run /claude-api prompt-audit at each Claude model release" --pain "Prompts are per-model artifacts; cruft returns silently" --risk tiny`.
4. Record signals: `scripts/bin/harness-cli story signal add --story US-039 --type design_decision --summary "Contract file wins over skill prose; lint enforces"`, and one `tradeoff` for keeping numbered steps if Task 6 kept variant A.
5. `scripts/bin/harness-cli story update --id US-039 --status implemented --unit 1 --integration 1 --e2e 1 --platform 0 --verify "bash scripts/verify-harness.sh . && bash scripts/lint-skills.sh && bash scripts/lint-mcp-templates.sh && bash scripts/check-diagrams.sh"`
   then `scripts/bin/harness-cli story verify US-039`.
6. `scripts/bin/harness-cli trace --summary "US-039 prompt-audit remediation" --story US-039 --intake 01M227KVAS6T3HC9751Z0ZCJ0F --outcome success --actions "<list>" --read "<list>" --changed "<list>"`.

Verify: `story verify US-039` passes; `bash scripts/check-diagrams.sh` ok;
`git status` shows `.harness/events/` and regenerated views staged with the
final commit.
