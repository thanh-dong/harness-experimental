# US-037 Change Diagrams As Reviewable Artifacts

## Status

implemented

## Lane

normal

## Product Contract

Normal and high-risk work carries change diagrams — separate Mermaid files
under `<packet>/diagrams/D<n>-<slug>.md`, one of seven kinds (D1 blast radius,
D2 component delta, D3 sequence, D4 state, D5 data-model delta, D6 story DAG,
D7 boundary). The intake risk flags select which are required; each file has
its own `draft | reviewed | stale` status; a human reviews them at a named
stage (intake checkpoint, design review, done gate) and the review is recorded
with `intervention add --type review`; the implementing agent derives tests,
re-run sets, migrations, and placement from the reviewed diagrams. A required
diagram that is missing or `stale` blocks done.

## Relevant Product Docs

- `docs/DIAGRAMS.md` (new — the standard)
- `docs/FEATURE_INTAKE.md` (Change Diagrams section, lane requirements, done gate)
- `docs/HARNESS.md` (done definition, change policy)
- `docs/CONTEXT_RULES.md` (per-phase reading rows)
- `docs/IMPACT_ANALYSIS.md` (D1 as gated output)
- `docs/templates/mcp/session-context.md`, `TOOL_MAPPING.md` (MCP flavor mirror)

## Acceptance Criteria

- `docs/DIAGRAMS.md` defines the seven kinds, the flag → diagram table, the
  file standard (header fields, one Mermaid fence, two trailing sections), the
  drawing standard (Mermaid type per kind, fixed change classes, delta-only,
  25-node cap), the generators and degraded modes, the three review stages and
  how a review is recorded, the agent-consumption table, and drift handling.
- `docs/templates/diagrams/D1..D7` exist, each passes the lint, each starts
  with the prescribed Mermaid type.
- `scripts/check-diagrams.sh` lints header, kind/prefix agreement, Source and
  Status values, reviewer fields on `reviewed`, D1 coverage, single fence with
  the allowed type, and the two sections; exits 1 on failure.
- Both gate flavors (bash in `FEATURE_INTAKE.md`, MCP in
  `docs/templates/mcp/`) carry the same diagram obligations;
  `harness_intervention_add` added to the tool mapping; MCP lint passes.
- Installer file list ships the new doc, templates, and script.

## Design Notes

- Commands: `bash scripts/check-diagrams.sh [files]`
- Queries: `harness-cli query interventions --story <id>` shows diagram reviews.
- Tables: none — review is an `intervention` row (`type=review`), no schema change.
- Domain rules: flags select diagrams; status lives in the file, durable record
  in the intervention; `stale` is set by whoever changes the subject.

## Change Diagrams

| Diagram | File | Status | Reviewed at |
| --- | --- | --- | --- |
| D1 blast radius | — | not required: `impact-analysis` capability inactive on this clone | — |
| D3 sequence | `diagrams/D3-diagram-review-flow.md` | reviewed (human:thanh-dong, 2026-08-23) | design review |
| D4 state | `diagrams/D4-diagram-status.md` | reviewed (human:thanh-dong, 2026-08-23) | design review |

## References

- `docs/IMPACT_ANALYSIS.md` — capability gating and degraded-mode pattern the
  D1/D2 generation section mirrors.
- `docs/FEATURE_INTAKE.md` Implementation Notes — the done-gate wording the
  diagram done gate copies.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | `bash scripts/check-diagrams.sh` passes on all templates and packet diagrams; rejects a file with wrong Kind or unreviewed `reviewed` status |
| Integration | `bash scripts/lint-mcp-templates.sh` passes with the new tool row |
| E2E | — |
| Platform | — |
| Release | installer list includes the new files |

## Harness Delta

This story is a harness delta. Open follow-ups: `story verify` has no way to
require diagram review for a lane mechanically; and
`docs/templates/implementation-notes.html` was already absent from the
installer file list before this change (pre-existing, not fixed here).

## Evidence

- `bash scripts/check-diagrams.sh` → `9 file(s) ok` (7 templates + 2 packet diagrams).
- `bash scripts/lint-mcp-templates.sh` → `PASS`.
