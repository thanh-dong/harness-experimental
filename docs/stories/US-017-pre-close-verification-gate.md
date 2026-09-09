# US-017 Pre-Close Verification Gate

## Status

implemented

## Lane

tiny

## Product Contract

When a trace links to a story with a `verify_command`, the CLI warns if that
story's latest verification result has not passed. The warning is advisory and
the trace is still recorded.

## Relevant Product Docs

- `docs/HARNESS.md`
- `docs/GLOSSARY.md`

## Acceptance Criteria

- `trace --story <id>` warns when the story has an unverified command.
- No warning appears after the story verification has passed.
- Failed verification results still warn.
- Stories with no `verify_command` do not warn.
- Traces without `--story` do not check story verification.

## Design Notes

- Commands: `scripts/bin/harness-cli trace --story <id>`.
- Queries: read story `verify_command` and `last_verified_result`.
- Tables: `trace`, `story`.
- Domain rules: pre-close verification is advisory in Phase 4.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | Repository tests cover story verification status. |
| Integration | CLI smoke for unverified, passed, failed, no-command, and no-story traces. |
| E2E | Not applicable; CLI-only story. |
| Platform | Repo-local `scripts/bin/harness-cli trace --story` warns correctly. |
| Release | `cargo clippy --workspace -- -D warnings`. |

## Harness Delta

Trace recording now surfaces missing story proof before agents close work.

## Evidence

- `cargo fmt --check`
- `cargo test --workspace` passed with 18 tests.
- `cargo clippy --workspace -- -D warnings`
- CLI smoke verified warnings for unverified and failed story commands, and no
  warning for passed, no-command, and no-story traces.
