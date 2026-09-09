# US-009 Enriched Friction Query

## Status

implemented

## Lane

tiny

## Product Contract

`scripts/bin/harness-cli query friction` shows the task lane and input type from a
linked intake record alongside each friction entry.

## Relevant Product Docs

- `docs/TRACE_SPEC.md`
- `docs/HARNESS_COMPONENTS.md`

## Acceptance Criteria

- `query friction` output includes `risk_lane` and `input_type` columns.
- Values come from `trace.intake_id -> intake.id`.
- Traces without linked intake, or with missing intake rows, show `-`.
- The query still only returns traces with non-null `harness_friction`.
- `cargo test` covers joined output, null-intake output, and the friction
  filter.

## Design Notes

- Commands: existing `scripts/bin/harness-cli query friction`.
- Queries: LEFT JOIN from `trace` to `intake`.
- API: extend `FrictionRecord`.
- Tables: no schema migration.
- Domain rules: missing intake context is displayed as `-`.
- UI surfaces: terminal output only.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | Rust tests for joined and missing intake context. |
| Integration | CLI smoke for `query friction`. |
| E2E | Not applicable; CLI-only story. |
| Platform | `scripts/bin/harness-cli` continues to run. |
| Release | `cargo clippy --workspace -- -D warnings`. |

## Harness Delta

Friction records become easier to group by lane and input type during active
observability reviews.

## Evidence

- `mise x rust@stable -- cargo fmt --check`
- `mise x rust@stable -- cargo test --workspace` passed with 15 tests.
- `mise x rust@stable -- cargo clippy --workspace -- -D warnings`
- CLI smoke: `scripts/bin/harness-cli query friction` showed `risk_lane` and
  `input_type` for linked traces and `-` for unlinked traces.
