# US-026 Blast-Radius Plugin: Lane-Gated Impact Analysis At Intake

## Status

implemented

## Lane

normal

## Product Contract

When a change request enters intake on the normal or high-risk lane, the agent
runs a blast-radius impact analysis (CodeGraph code graph + C3 component model +
durable-layer feature join) whose output gates the risk flags, the validation
re-run set, and the implementation reading list. Tiny-lane work skips it.
Activation and presence come from the inbound tool registry (US-027): the agent
looks up the `impact-analysis` capability and reads scanned provider presence
rather than trusting declared intent. Invalid or sparse tooling degrades visibly
(UNKNOWN + `Weak proof` flag), never silently.

## Relevant Product Docs

- `docs/IMPACT_ANALYSIS.md` (policy created by this story)
- `docs/TOOL_REGISTRY.md` (the inbound registry this consumes; US-027)
- `docs/FEATURE_INTAKE.md` (intake hook)
- `docs/HARNESS.md` (task loop this extends)

## Acceptance Criteria

- `docs/IMPACT_ANALYSIS.md` exists and defines: lane-gated trigger, the
  codegraph/c3 dependency set, the three-layer preflight (equipped/valid/live),
  the file-path join pipeline, the coverage signal, and the three gated
  decisions.
- `docs/FEATURE_INTAKE.md` references the impact analysis step for normal and
  high-risk lanes and keys activation on the `impact-analysis` capability.
- Activation is capability-based, not tool-named: the agent runs
  `query tools --capability impact-analysis`; no registered provider means the
  capability is inactive and intake skips the step (trace note only, no
  `Weak proof`).
- Presence is scanned, not trusted: `tool check` runs at intake start, and a
  provider that scans as `missing`/stale degrades per the Degraded Modes table
  and sets `Weak proof`.
- The empty-join rule is explicit: no feature impact data is reported as
  UNKNOWN with the `Weak proof` flag, never as "no features affected".
- Provider kinds are marked (`cli` for codegraph, `skill` for c3) so a
  non-Claude agent treats a skill it cannot run as absent and degrades; a
  `cli` provider needs no agent session and works in headless runners.

## Design Notes

- Commands: `query tools --capability impact-analysis`, `tool check` (both
  from US-027); registration seed lives in `docs/TOOL_REGISTRY.md`.
- Queries: agent-side `codegraph impact` / `codegraph affected`; `c3-audit`.
- Tables: no schema changes in this story; consumes US-027's `tool.kind`,
  `tool.capability`, `tool.scan_target`, `tool.status`, and reuses
  `trace.files_changed`, `trace.story_id`, `story.contract_doc`.
- Domain rules: agent-orchestrated, harness-recorded; degrade-don't-lie;
  no curated feature-to-component mapping artifact; capability-first lookup,
  never tool-name coupling.
- Deliberately excluded until friction recorded: hard CLI enforcement of
  `story_id`/`contract_doc`, new DB tables, CI gates, installer flag.

## Validation

When updating durable proof status, use numeric booleans:
`scripts/bin/harness-cli story update --id US-026 --unit 1 --integration 1 --e2e 0 --platform 0`.

| Layer | Expected proof |
| --- | --- |
| Unit | Verify command: policy doc exists and intake hook references it. |
| Integration | `query tools --capability impact-analysis` resolves the provider set; `tool check` persists status. |
| E2E | Not applicable (docs/process module; no app surface). |
| Platform | Not applicable. |
| Release | Not applicable. |

## Harness Delta

- New policy doc `docs/IMPACT_ANALYSIS.md`, rewritten to consume the inbound
  tool registry (US-027) instead of an impact-analysis-specific activation
  switch.
- `docs/FEATURE_INTAKE.md` gains an impact analysis step for normal/high-risk,
  keyed on the `impact-analysis` capability and `tool check`.
- `docs/templates/story.md` documents the valid durable story statuses.
- Tool registry seed for codegraph and c3 lives in `docs/TOOL_REGISTRY.md`.
- Provider swap gitnexus -> codegraph recorded in decision 0011 (pipeline
  worktree compatibility).
- Follow-up: Backlog proposes `harness-cli impact preflight` for mechanical
  mode detection once friction is recorded.

## Evidence

- `scripts/bin/harness-cli story verify US-026` (doc + hook check).
- `scripts/bin/harness-cli query tools --capability impact-analysis` shows the
  registered provider set; `tool check` reports their scanned status.
