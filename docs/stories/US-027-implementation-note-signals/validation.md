# Validation

## Proof Strategy

The feature is proven when (a) the schema migration applies cleanly to a fresh
and an existing DB, (b) signals can be added and queried, and (c) `propose`
emits a proposal only when a signal recurs `>= 2` times — never for a single
signal.

## Test Plan

| Layer | Cases |
| --- | --- |
| Unit | `add_story_signal` persists rows; `query_story_signals` filters by type/story; `propose` emits a recurring-signal proposal at count 2 and emits none at count 1; invalid `type` rejected by constraint. |
| Integration | `init` on a v5 DB applies migration 006 and `query signals` works; `propose --commit` writes a `proposed` backlog item from a recurring signal. |
| E2E | CLI end-to-end: `story signal add` x2 with same summary -> `propose` lists the proposal. |
| Platform | n/a (CLI). |
| Performance | n/a. |
| Logs/Audit | Recurring signal appears in `propose` evidence text. |

## Fixtures

- Story `US-AUDIT` (reuse existing test story helper).
- Two `deviation` signals with identical summary "plan omitted migration step".

## Commands

```text
cargo test -p harness-cli
cargo clippy -p harness-cli -- -D warnings
scripts/bin/harness-cli story signal add --type deviation --summary "..." --story US-027-...
scripts/bin/harness-cli query signals
scripts/bin/harness-cli propose
```

## Acceptance Evidence

- `cargo test -p harness-cli` — 28 passed, including
  `propose_mines_recurring_story_signals_only_when_they_repeat` (proves the
  `>= 2` gate: no proposal at count 1, proposal at count 2).
- `cargo clippy -p harness-cli -- -D warnings` — no issues found.
- Migration 006 applied to a fresh DB (schema_version 6) and to an existing v5
  DB via `harness-cli migrate` (`Applied migration 6`), preserving prior records.
- `story verify US-027` — pass.
- Live `propose` emitted the recurring-deviation proposal (Component:
  `Task specification`, Evidence: "2 stories recorded a similar deviation
  signal"). The recurring friction it surfaced (hardcoded `schema_version`
  assertions) is recorded as backlog #1.
- Full narrative in `implementation-notes.html` (this story's dogfood note).
