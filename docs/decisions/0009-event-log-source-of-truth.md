# 0009 Event Log Becomes the Durable Source of Truth

Date: 2026-07-02

## Status

Accepted (supersedes `0004-sqlite-durable-layer`)

## Context

Decision 0004 made SQLite (`harness.db`, gitignored) the durable layer. That
is correct for one machine and wrong for a team: story proof, decisions,
backlog outcomes, and all telemetry never reached another clone, and a
committed SQLite binary cannot merge. US-028 specified the fix; the goal-loop
frame was ratified in decision 0008 and all four discovery checkpoints
(DKR-1..4) were accepted with deterministic evidence.

## Decision

The source of truth for all durable harness state is the append-only,
per-writer event log tracked in git at `.harness/events/` (US-028b):

- `harness.db` is demoted to a disposable materialized cache, rebuilt
  deterministically from the log. Fresh clones auto-rebuild; every command
  incrementally replays new events (stat short-circuit watermarks, measured
  p95 ≈ 6.8 ms per mutation at 100k events against the 100 ms wall).
- Mutations apply in a transaction (SQLite constraints validate the event),
  append to the writer's file with fsync, then commit — a crash leaves the
  cache behind the log, healed by replay. The cache is only written from
  events, so cache and log cannot drift.
- Row ids are ULIDs (schema 007); writer identity is the user's email hash
  plus a per-clone untracked disambiguator (decision 0008 Q3).
- Concurrent same-field story updates resolve last-writer-wins by
  `(event_id, writer)` with a **causal** audit (`observed` watermarks), never
  a wall-clock window (DKR-4). Late-arriving older events cannot overwrite —
  incremental replay converges to the full-rebuild state.
- `TEST_MATRIX.md`, `HARNESS_BACKLOG.md`, and the decision index are
  generated views of the cache.
- `migrate-to-events` migrates pre-event databases behind a per-table count +
  content-hash equality proof (tool scan columns excluded — machine-local
  state is never logged); old DB backed up, never deleted. `import
  brownfield` is a one-time seed (0008 Q4).

Proof at acceptance: 55 crate tests + two-writer git-merge e2e
(`merge_conflict_count == 0`, same-email clones), 19 calibration goldens,
benchmark at 1k/10k/100k events, and this repository's own migration (29
rows, matrix byte-identical pre/post, fresh-clone rebuild identical).

## Alternatives Considered

1. Shared database service (Turso/Postgres/rqlite) — rejected in the spec:
   breaks the drop-in property, moves truth out of the repository.
2. Committing the SQLite file — unmergeable binary conflicts.
3. Wall-clock audit window for concurrent updates — rejected by DKR-4
   evidence: any window W silently misses skew > W.

## Consequences

Positive:

- Teammates share all eight durable tables through normal git flow;
  `propose`/`audit` mine team-wide evidence.

Tradeoffs:

- Log grows without bound until US-028c (compaction at ~5k events,
  telemetry-age snapshotting per DKR-1); bytes are a non-issue for 24+ months.
- Tool scan status resets to `unknown` on rebuild (machine-local by design;
  re-run `tool check`).

## Follow-Up

- US-028c: compaction/snapshots, hash-chain integrity, `events verify`,
  telemetry retention ratification.
