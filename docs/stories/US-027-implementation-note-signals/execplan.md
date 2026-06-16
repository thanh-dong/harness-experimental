# Exec Plan

## Goal

Make the implementation-note categories feed `harness-cli propose`, so the
richest agent-produced design signal becomes self-evolution input.

## Scope

In scope:

- Additive schema migration `006-story-signal.sql`.
- `story signal add` command and `query signals` view.
- One new recurring-signal loop in `propose`.
- Cargo unit tests + docs (`IMPROVEMENT_PROTOCOL.md`).

Out of scope:

- HTML parsing or any LLM in `propose`.
- Backfilling existing stories.
- Benchmark-repo wiring (separate work).

## Risk Classification

Risk flags:

- Data model (new durable table).
- Existing behavior (`propose` output changes — additive).
- Public contracts (new CLI surface).

Hard gates:

- Data model -> high-risk. Scope narrowed: additive table only, no migration of
  existing data, no deletion/retention change.

## Work Phases

1. Discovery — read decision 0004, intervention schema, propose code. (done)
2. Design — schema + command + propose rule. (decision 0007)
3. Validation planning — see `validation.md`.
4. Implementation — schema 006, domain/app/infra/interface, propose loop.
5. Verification — cargo test + clippy; seed signals; run propose.
6. Harness update — `IMPROVEMENT_PROTOCOL.md`, dogfood note for this story.

## Stop Conditions

Pause for human confirmation if:

- The change would require migrating or deleting existing rows.
- `propose` would need to become non-deterministic to read signals.
- Validation requirements would be weakened.
