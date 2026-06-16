# Design

## Domain Model

`StorySignal` — an agent-recorded design fact about a story.

- `signal_type`: `design_decision | deviation | tradeoff | open_question`.
- `summary`: normalized, mineable text (the recurrence key).
- `story_id`, `trace_id`: optional links.
- `component`: optional harness component the signal implicates.

Invariant: a signal is the agent's own design narrative, distinct from an
`Intervention` (an external correction) and from trace `harness_friction`
(per-run friction).

## Application Flow

- Command `story signal add` -> `StorySignalAddInput` -> `repository.add_story_signal`.
- Query `query signals` -> `StorySignalFilter` -> `repository.query_story_signals`.
- `propose` calls `repeated_story_signals(connection)` and emits one proposal
  per recurring `(type, summary)` group with `count >= 2`.

## Interface Contract

```text
harness-cli story signal add --type <design_decision|deviation|tradeoff|open_question>
    --summary <text> [--story <id>] [--trace <id>] [--component <c>] [--notes <n>]
harness-cli query signals [--type <t>] [--story <id>]
```

Errors: invalid `--type` rejected by the CHECK constraint and by accepted-value
help, consistent with `intervention add`.

## Data Model

Migration `006-story-signal.sql` (additive):

```sql
CREATE TABLE story_signal (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    story_id    TEXT REFERENCES story(id),
    trace_id    INTEGER REFERENCES trace(id),
    type        TEXT NOT NULL CHECK(type IN
                  ('design_decision','deviation','tradeoff','open_question')),
    summary     TEXT NOT NULL,
    component   TEXT,
    notes       TEXT
);
INSERT INTO schema_version (version) VALUES (6);
```

Auto-applies via `apply_pending_migrations` (version 6 > current 5). No backfill,
no deletion, no retention change.

## UI / Platform Impact

CLI only. New `story signal` subcommand and `query signals` view.

## Observability

Recurring signals surface in `propose` evidence and, with `--commit`, as
`proposed` backlog items — same path as the other three sources.

## Alternatives Considered

1. Parse HTML in `propose` — rejected (non-deterministic; decision 0007).
2. Reuse `harness_friction` — rejected (per-trace, untyped).
3. Overload `intervention` — rejected (different actor and semantics).
