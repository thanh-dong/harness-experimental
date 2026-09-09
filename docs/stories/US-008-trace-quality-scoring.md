# US-008 Trace Quality Scoring

## Status

implemented

## Lane

normal

## Product Contract

Agents can run `scripts/bin/harness-cli score-trace` to score the latest trace, or
`scripts/bin/harness-cli score-trace --id N` to score a specific trace, against the
trace quality tiers in `docs/TRACE_SPEC.md`.

## Relevant Product Docs

- `docs/TRACE_SPEC.md`
- `docs/HARNESS_MATURITY.md`
- `docs/HARNESS_COMPONENTS.md`

## Acceptance Criteria

- Scores minimal, standard, and detailed traces using `docs/TRACE_SPEC.md`
  field-presence rules.
- Looks up linked intake lane when `intake_id` is present and reports the
  required tier.
- Lists missing fields for the next tier and for any unmet lane requirement.
- Exits with status 0 when the trace meets its lane requirement or has no lane,
  and status 1 when it falls below the lane requirement.
- `cargo test` covers scoring tiers, lane lookup, missing-field output, and
  exit-code behavior.

## Design Notes

- Commands: `scripts/bin/harness-cli score-trace`, `scripts/bin/harness-cli score-trace --id N`.
- Queries: trace row by id or latest trace; optional intake lane lookup.
- API: add Rust service/repository scoring path and CLI output.
- Tables: read existing `trace` and `intake`; no schema migration.
- Domain rules: tiny requires minimal, normal requires standard, high-risk
  requires detailed.
- UI surfaces: terminal output only.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | Rust tests for all tiers, lane comparison, and missing fields. |
| Integration | CLI smoke against a temporary Harness database. |
| E2E | Not applicable; CLI-only story. |
| Platform | `scripts/bin/harness-cli score-trace` runs through the repo-local binary. |
| Release | `cargo clippy --workspace -- -D warnings`. |

## Harness Delta

`docs/TRACE_SPEC.md` review guidance should reference the scoring command as
the mechanical final check for normal and high-risk work.

## Evidence

- `mise x rust@stable -- cargo fmt --check`
- `mise x rust@stable -- cargo test --workspace` passed with 15 tests.
- `mise x rust@stable -- cargo clippy --workspace -- -D warnings`
- CLI smoke: `scripts/bin/harness-cli score-trace` reported a standard high-risk
  trace below detailed requirement and exited 1.
- CLI smoke: `scripts/bin/harness-cli score-trace` reported a detailed normal trace
  meets the standard requirement and exited 0.
- Benchmark run `phase-3-active-observability` improved average trace quality
  from 2.6 to 2.8 and T4 reached detailed trace quality.
