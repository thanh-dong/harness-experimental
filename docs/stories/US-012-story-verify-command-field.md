# US-012 Story verify_command Field

## Status

implemented

## Lane

normal

## Product Contract

Stories can store an optional `verify_command` plus the last verification
timestamp and result.

## Relevant Product Docs

- `docs/HARNESS.md`
- `scripts/schema/002-story-verify.sql`

## Acceptance Criteria

- Migration `002-story-verify.sql` adds `verify_command`,
  `last_verified_at`, and `last_verified_result` to `story`.
- Fresh `init` and existing `migrate` databases end at schema version 2.
- `story add --verify` stores the command.
- `story update --verify` updates the command.
- Rust tests cover the migration and fields.

## Design Notes

- Commands: `scripts/bin/harness-cli story add --verify`, `scripts/bin/harness-cli story update --verify`.
- Tables: `story`.
- Domain rules: `last_verified_result` is `pass`, `fail`, or null.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | Rust tests for fresh init, migration, and verify field storage. |
| Integration | CLI smoke with temporary Harness database. |
| E2E | Not applicable; CLI-only story. |
| Platform | Repo-local `scripts/bin/harness-cli` supports the new flags. |
| Release | `cargo clippy --workspace -- -D warnings`. |

## Harness Delta

Stories now carry mechanical proof commands in the durable layer.

## Evidence

- `cargo fmt --check`
- `cargo test --workspace` passed with 18 tests.
- `cargo clippy --workspace -- -D warnings`
- CLI smoke verified fresh schema version 2, story verification columns, and
  `story add/update --verify`.
