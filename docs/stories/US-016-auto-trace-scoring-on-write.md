# US-016 Auto Trace Scoring On Write

## Status

implemented

## Lane

tiny

## Product Contract

`scripts/bin/harness-cli trace` records the trace and immediately prints the
trace quality score. Scoring is advisory and does not change the trace command
exit code.

## Relevant Product Docs

- `docs/TRACE_SPEC.md`
- `docs/HARNESS.md`

## Acceptance Criteria

- Trace output includes `Trace #N recorded.` and `Tier achieved:`.
- Linked intakes show the required lane tier.
- Below-requirement traces print missing fields.
- Trace rows are still recorded and command exit code remains 0.

## Design Notes

- Commands: `scripts/bin/harness-cli trace`, `scripts/bin/harness-cli score-trace`.
- Queries: score the trace id returned by `record_trace`.
- Tables: `trace`, `intake`.
- Domain rules: advisory scoring does not block trace writes.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | Existing trace scoring tests cover minimal, standard, and detailed tiers. |
| Integration | CLI smoke for minimal and high-risk linked traces. |
| E2E | Not applicable; CLI-only story. |
| Platform | Repo-local `scripts/bin/harness-cli trace` auto-scores. |
| Release | `cargo clippy --workspace -- -D warnings`. |

## Harness Delta

Agents receive trace-quality feedback at the point of recording.

## Evidence

- `cargo fmt --check`
- `cargo test --workspace` passed with 18 tests.
- `cargo clippy --workspace -- -D warnings`
- CLI smoke verified `trace` records rows and prints `Tier achieved` with
  advisory missing-field output.
