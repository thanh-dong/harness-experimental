# US-032 JSON Output Mode on the CLI

## Status

planned

## Lane

normal

## Product Contract

A `--json` flag (or `HARNESS_OUTPUT=json`) on the commands an MCP wrapper
needs — `intake`, `story add|update|signal`, `decision add`, `trace`,
`backlog add`, `query *`, `tool check` — emitting stable field names and
meaningful exit codes. No consumer ever scrapes human-formatted stdout.

Epic: E-shuttle-readiness. The single biggest enabler for Shuttle's MCP tool
surface (US-MH-03).

## Acceptance Criteria

- Every listed command supports JSON output; success and error shapes stable.
- Exit codes documented (0 ok; distinct codes for validation vs. IO failure).
- Human output unchanged by default (no breakage for existing docs/flows).

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | per-command JSON snapshot tests |
| Integration | round-trip: add via JSON → query via JSON |

## Harness Delta

CLI change; version bump; document in scripts/README.md.
