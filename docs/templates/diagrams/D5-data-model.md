# D5 Data-model delta — US-XXX Story title

Story: US-XXX
Kind: data-model
Source: hand
Scope: tables touched by migration NNN and their foreign-key neighbours
Status: draft
Reviewed-by: -
Reviewed-at: -

```mermaid
erDiagram
  story ||--o{ story_signal : has
  trace ||--o{ story_signal : has
  story {
    TEXT id PK
  }
  %% added
  story_signal {
    INTEGER id PK
    TEXT story_id FK
    INTEGER trace_id FK
    TEXT type
    TEXT summary
  }
```

## What to review

- Is any data deleted, rewritten, or retained differently? (hard gate)
- Is the migration additive, and is backfill needed?
- Do the constraints match the D4 state set, if one exists?

## Derived next steps

- Migration file and schema-version bump.
- Rebuild-determinism check (two rebuilds print the same dump hash) if any table is rewritten.
