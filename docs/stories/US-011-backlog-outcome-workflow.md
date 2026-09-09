# US-011 Backlog Outcome Workflow

## Status

implemented

## Lane

tiny

## Product Contract

Harness backlog items record predicted impact when proposed and actual outcome
when closed, and backlog queries can filter open and closed items.

## Relevant Product Docs

- `docs/HARNESS.md`
- `docs/GLOSSARY.md`

## Acceptance Criteria

- `docs/HARNESS.md` documents the predicted-impact to actual-outcome loop.
- `scripts/bin/harness-cli query backlog --open` shows only proposed and accepted
  items.
- `scripts/bin/harness-cli query backlog --closed` shows only implemented and rejected
  items.
- `scripts/bin/harness-cli query backlog` without a filter still shows all items.
- `docs/GLOSSARY.md` defines "backlog outcome loop".
- `cargo test` covers open and closed backlog filters.

## Design Notes

- Commands: `scripts/bin/harness-cli query backlog --open`, `scripts/bin/harness-cli query
  backlog --closed`.
- Queries: optional backlog status filter.
- API: add filter argument to backlog query path.
- Tables: no schema migration.
- Domain rules: open means proposed or accepted; closed means implemented or
  rejected.
- UI surfaces: terminal output only.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | Rust tests for open, closed, and unfiltered backlog queries. |
| Integration | CLI smoke for filtered backlog queries. |
| E2E | Not applicable; CLI-only story. |
| Platform | `scripts/bin/harness-cli` continues to run. |
| Release | `cargo clippy --workspace -- -D warnings`. |

## Harness Delta

Harness improvements gain an explicit feedback loop: prediction before work,
measured outcome after closure.

## Evidence

- `mise x rust@stable -- cargo fmt --check`
- `mise x rust@stable -- cargo test --workspace` passed with 15 tests.
- `mise x rust@stable -- cargo clippy --workspace -- -D warnings`
- CLI smoke: `scripts/bin/harness-cli query backlog --open` returned only proposed
  backlog items.
- CLI smoke: `scripts/bin/harness-cli query backlog --closed` returned only implemented
  backlog items with predicted and actual outcome columns visible.
