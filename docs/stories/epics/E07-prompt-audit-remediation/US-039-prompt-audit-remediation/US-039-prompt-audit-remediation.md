# US-039 Prompt-audit remediation and model fit for Opus 5 / Fable 5.1

## Status

in_progress

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
| D3 sequence | `diagrams/D3-verify-and-lint-flow.md` | draft | design review |

## References

- `.claude/skills/reverse-tornado-okr/scripts/okra-verify-artifact.py` —
  substring matching rules the contract tokens must satisfy.
- `scripts/lint-mcp-templates.sh` — shape to match for the new lints.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | `bash scripts/check-diagrams.sh`; `bash scripts/lint-mcp-templates.sh`; new `bash scripts/lint-skills.sh` |
| Integration | `bash scripts/verify-harness.sh .` 8/8; `bash scripts/test-install-contract.sh` |
| E2E | verifier passes on the Phase 0 fixture artifact after Phase 1; A/B of OKR skill variants under the verifier |
| Platform | n/a |
| Release | n/a |

## Harness Delta

- New lints: `scripts/lint-skills.sh`, reading-block equality in
  `scripts/verify-harness.sh`, generated-view wording check in
  `scripts/lint-mcp-templates.sh`.
- Decision: keep verification scaffolding (Fable 5.1 guidance), see
  `docs/decisions/0013-keep-verification-steps.md`.

## Evidence

Filled in as phases complete; see `implementation-notes.html`.
