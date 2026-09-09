# US-015 Story Verify Command

## Status

implemented

## Lane

normal

## Product Contract

Agents can run `scripts/bin/harness-cli story verify <id>` to execute a story's
`verify_command` from the repository root and record pass/fail state.

## Relevant Product Docs

- `docs/HARNESS.md`
- `docs/GLOSSARY.md`

## Acceptance Criteria

- `story verify <id>` runs the story command and prints pass or fail.
- Verification updates `last_verified_at` and `last_verified_result`.
- Passing commands exit 0; failing commands exit 1.
- Stories without `verify_command` return `story <id> has no verify_command`.
- Rust tests cover pass, fail, missing command, and repo-root execution.

## Design Notes

- Commands: `scripts/bin/harness-cli story verify <id>`.
- Queries: select story `verify_command`, update verification result.
- Tables: `story`.
- UI surfaces: terminal output only.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | Rust tests for pass, fail, missing verify command, and repo-root execution. |
| Integration | CLI smoke for passing and failing story verification. |
| E2E | Not applicable; CLI-only story. |
| Platform | Repo-local `scripts/bin/harness-cli story verify` runs. |
| Release | `cargo clippy --workspace -- -D warnings`. |

## Harness Delta

Story validation gains the same mechanical verification pattern as decisions.

## Evidence

- `cargo fmt --check`
- `cargo test --workspace` passed with 18 tests.
- `cargo clippy --workspace -- -D warnings`
- CLI smoke verified `story verify` pass/fail behavior, fail exit status, and
  repo-root command output ordering.
