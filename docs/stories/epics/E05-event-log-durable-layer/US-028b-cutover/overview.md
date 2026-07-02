# US-028b Cutover: Event Log Becomes the Source of Truth

## Status

in_progress — human approved the cutover gate 2026-07-02 ("go on US-028b"),
after frame ratification (decision 0008) and accepted discovery checkpoints
DKR-1..4.

## Lane

high-risk. Hard gates: data migration (every durable row), every CLI write
path changes, public CLI contract changes (ULID ids). Full packet per
`docs/FEATURE_INTAKE.md`.

## What changes

The append-only, per-writer event log at `.harness/events/` becomes the
**source of truth** for all durable harness state. `harness.db` is demoted to
a disposable, gitignored materialized cache, deterministically rebuilt from
the log. Concretely:

1. **ULID row ids** — `intake`, `backlog`, `trace`, `intervention`,
   `story_signal` move from auto-increment integers to ULID TEXT ids so
   parallel writers can never mint colliding ids. The CLI accepts unambiguous
   id prefixes; legacy integer ids remain valid values.
2. **`migrate-to-events`** — synthesizes genesis events from an existing DB
   (writer `migration`, deterministic event ids, original timestamps,
   original ids preserved in `payload.id`), proves per-table count and
   content-hash equality against the live DB (tool table excluding
   machine-local `status`/`checked_at`), and only then installs the log and
   demotes the DB. Old DB backed up, never deleted. Byte-idempotent on re-run.
3. **Cutover write path** — mutation = validate → append event (hard fail;
   the shadow guarantee flips) → apply that same event to the cache → update
   watermark. The cache is only ever written from events, so cache and log
   cannot drift.
4. **Watermark reads** — every command compares per-writer-file watermarks
   (stat short-circuit: size + mtime, hash fallback) and incrementally
   replays new events (e.g. after `git pull`) before answering. A fresh clone
   with no `harness.db` auto-rebuilds from the log.
5. **Causal concurrent-update audit** — each event carries an `observed`
   watermark; replay flags cross-writer same-field story updates that are
   causally concurrent. No wall-clock windows (DKR-4).
6. **Generated views** — `docs/TEST_MATRIX.md` and `docs/HARNESS_BACKLOG.md`
   become generated views of the cache, marked "generated — do not hand-edit".
   `import brownfield` is retired to migration-only (decision 0008 Q4).

## Why now

US-028a proved the append+replay contract (`rebuild_determinism == 1` on real
data and in CI). The four DKR checkpoints resolved every uncertainty this
story needed: latency (21× margin + stat short-circuit), migration equality
(8/8 tables), merge semantics (causal audit), log growth (count-triggered
compaction, deferred to US-028c).

## Anti-goals in force (frame 0008)

`merge_conflict_count == 0` · `migrated_row_loss == 0` · `pr_reviewability
does not decrease` · `calibration stays green` · `write_latency_ms <= 100`
p95 · `story verify-all stays green`.
