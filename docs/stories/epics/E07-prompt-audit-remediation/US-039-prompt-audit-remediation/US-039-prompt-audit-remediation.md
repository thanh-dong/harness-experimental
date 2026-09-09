# US-039 Prompt-audit remediation and model fit for Opus 5 / Fable 5.1

## Status

implemented

## Lane

normal

## Product Contract

The harness's model-facing text (always-loaded context, skills, MCP flavor,
lane docs) states each rule once, matches the code it describes, and is tuned
for Claude Opus 5 and Claude Fable 5/5.1. The OKR skill's own completeness
gate passes for real goals. Drift between copies of the same rule fails a lint
instead of being found by hand.

Source: `/claude-api prompt-audit` run on 2026-09-07 (31 findings: 12 high,
11 medium, 8 flag-only). The full task list is in `plan.md` next to this file.

## Relevant Product Docs

- `docs/FEATURE_INTAKE.md`
- `docs/CONTEXT_RULES.md`
- `docs/DIAGRAMS.md`
- `docs/templates/mcp/README.md`

## Acceptance Criteria

- `okra-verify-artifact.py` reports complete on a real (non-benchmark) OKR
  delegated-loop artifact that follows SKILL.md; no requirement cites a hidden
  eval "case prompt".
- `AGENTS.md`, `scripts/install-harness.sh`, and `scripts/install-harness.ps1`
  carry the same Harness reading block, which defers to `docs/CONTEXT_RULES.md`
  per lane; a lint fails when they differ.
- `docs/templates/mcp/session-context.md` never names a generated view as a
  hand-edit target and uses the bash flavor's register.
- Every stale fact listed in `plan.md` Phase 3 is corrected; `PHASE2.md` to
  `PHASE5.md` are removed and nothing outside `CHANGELOG.md` references them.
- `.claude/skills/reverse-tornado-okr` and `.codex/skills/reverse-tornado-okr`
  are identical, and the contract's exact sentences appear in the skill text;
  a lint fails when either stops being true.
- `CLAUDE.md` carries a subagent policy, a scope/test-sprawl rule, and a memory
  surface pointer; the done gates carry progress-claim grounding.
- Each phase ends with its verification commands run and recorded in
  `implementation-notes.html`.

## Design Notes

- Commands: none new; `scripts/lint-skills.sh` and an extension of
  `scripts/verify-harness.sh` are added.
- Domain rules: contract file wins over skill prose; CONTEXT_RULES.md wins
  over any reading list.

## Change Diagrams

| Diagram | File | Status | Reviewed at |
| --- | --- | --- | --- |
| D1 blast radius | not required (impact-analysis provider absent) | — | — |
| D3 sequence | `diagrams/D3-verify-and-lint-flow.md` | reviewed (agent, 2026-09-09, after one correction round) | design review |

## References

- `.claude/skills/reverse-tornado-okr/scripts/okra-verify-artifact.py` —
  substring matching rules the contract tokens must satisfy.
- `scripts/lint-mcp-templates.sh` — shape to match for the new lints.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | `bash scripts/check-diagrams.sh`; `bash scripts/lint-mcp-templates.sh`; new `bash scripts/lint-skills.sh` |
| Integration | `bash scripts/verify-harness.sh .` 10/10 (8/8 before this story added checks 9 and 10); `bash scripts/test-install-contract.sh` |
| E2E | verifier passes on the Phase 0 fixture artifact after Phase 1; A/B of OKR skill variants under the verifier |
| Platform | n/a |
| Release | n/a |

## Harness Delta

- `scripts/verify-harness.sh` check 9: the `## Harness` reading block in
  `AGENTS.md`, `scripts/install-harness.sh`, and `scripts/install-harness.ps1`
  must read identically once the macOS/Linux-vs-Windows path alternative is
  normalized away.
- `scripts/verify-harness.sh` check 10: runs the new `scripts/lint-skills.sh`
  (`.claude`/`.codex` mirror parity, the contract's exact sentences present in
  the skill prose, no grader vocabulary in shipped instructions). The score is
  now 10, not 8.
- `scripts/lint-mcp-templates.sh` assertion 3: no generated view
  (`docs/TEST_MATRIX.md`, `docs/HARNESS_BACKLOG.md`,
  `docs/decisions/README.md`) may be named as something an agent edits.
- `docs/decisions/0013-keep-verification-steps.md` — keep deterministic
  verification steps under Fable 5.1.

## Evidence

Verification commands and their final results on the branch:

| Command | Result |
| --- | --- |
| `bash scripts/verify-harness.sh .` | `SCORE: 10/10` (was 8/8 before checks 9 and 10 existed) |
| `bash scripts/lint-skills.sh` | `PASS: skills lint clean.` — mirror parity, both contract sentences found in prose, no grader vocabulary |
| `bash scripts/lint-mcp-templates.sh` | `PASS: MCP templates lint clean.` — all three assertions |
| `bash scripts/check-diagrams.sh` | `check-diagrams: 11 file(s) ok` |
| `okra-verify-artifact.py evidence/fixture-okr-artifact.md` | 19 / 19, `complete: true`, exit 0 (baseline before the contract fix was 16 / 20) |

A/B of the OKR skill variants under the verifier: variant A scored 16 / 19,
variant B scored 18 / 19 as a strict superset, so **variant B was adopted**
(commit `a7f74b4`). Variant B's skill text is 32.9% shorter; its artifact is
29 words (0.55%) longer, which is writer noise between two independently
written artifacts. Both variants missed `dkr_to_dkr_worked`.

Durable copies live in `evidence/` — the Task 1 fixture, the baseline verifier
JSON, and both A/B artifacts with their verifier JSON. See
`evidence/README.md` for what each file proves, and
`implementation-notes.html` for the per-task narrative.
