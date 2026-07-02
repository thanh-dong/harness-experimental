# US-028 Event-Log Durable Layer: Git-Native Team State

## Status

proposed — awaiting human ratification of the frame (see Open Questions)

## Lane

high-risk. Hard gates hit: data migration (every durable row moves), removing or
weakening validation is possible if replay is wrong, and the change touches every
CLI write path. Per `docs/FEATURE_INTAKE.md`, implementation must not start until
a human confirms direction, and the work ships as a full high-risk story packet
(`execplan.md`, `overview.md`, `design.md`, `validation.md`) derived from this
spec.

## Problem

`harness.db` is SQLite, local, and gitignored by design. That is correct for one
machine and wrong for a team:

- **Teammates share nothing.** Story proof flags, verify results, decisions,
  backlog outcomes recorded via CLI never reach another clone. Only the three
  markdown surfaces (`TEST_MATRIX.md`, `docs/decisions/`, `HARNESS_BACKLOG.md`)
  travel, and only via the one-way `import brownfield`.
- **Telemetry is invisible team-wide.** `trace`, `intervention`, `story_signal`,
  `intake` exist per-machine, so `propose` and `audit` mine one agent's evidence
  instead of the team's. The self-improvement loop is blind to shared friction.
- **Tracking the binary is not an option.** A committed SQLite file cannot be
  merged; two writers conflict unmergeably on every parallel change.

The durable layer needs to be *shareable through git* without giving up SQLite's
local query power.

## Product Contract

The source of truth for all durable harness state becomes an **append-only,
per-writer event log tracked in git**. `harness.db` is demoted to a disposable,
gitignored **materialized cache**, deterministically rebuilt from the log. Two
teammates working in parallel branches merge their durable state as a trivial
file union — no binary conflicts, no lost records. Human-readable surfaces
(`TEST_MATRIX.md` and friends) become **generated views** of the log, so PR
reviewability does not decrease.

This is the same integrity rule the goal-loop engine already uses (see
`docs/GOAL_LOOP.md` and Okra's integrity store): *append-only records are the
source of truth; human-readable status is generated.*

## Goal-Loop Frame (candidate, per docs/GOAL_LOOP.md)

- **Objective**: `teammate_visible_tables == 8/8` — every durable table
  (`intake`, `story`, `decision`, `backlog`, `trace`, `tool`, `intervention`,
  `story_signal`) round-trips through git, proven by: a fresh clone runs
  `harness-cli rebuild` and its cache answers every query identically to the
  writer's (`rebuild_determinism == 1`, byte-stable dump comparison).
- **Anti-goals** (each a wall, read by a deterministic check):
  - `merge_conflict_count == 0` (tripwire) — two clones performing disjoint
    writes merge with zero conflicts. Read: the two-writer e2e scenario below.
  - `migrated_row_loss == 0` (tripwire) — migration reproduces every existing
    row. Read: row-count + content-hash comparison, old DB vs rebuilt DB.
  - `pr_reviewability does not decrease` (drift) — generated markdown views
    still render current state in diffs. Read: generated-view freshness check.
  - `calibration stays green` (tripwire) — `scripts/calibrate-harness.sh`
    passes unchanged against the rebuilt cache. Read: CI verify job.
  - `write_latency_ms <= 100` p95 per CLI mutation (drift) — append + cache
    apply must not make the CLI feel slower. Read: benchmark in validation.

## Relevant Product Docs

- `docs/HARNESS.md` (task loop whose records this carries)
- `docs/TOOL_REGISTRY.md` (tool table semantics; scan status stays local)
- `docs/GOAL_LOOP.md` (shared integrity philosophy; okra bridge)
- `docs/decisions/0004-sqlite-durable-layer.md` (the decision this supersedes —
  a new decision record is required)
- Epic `E01-durable-layer` (the architecture this evolves)

## Design

### Event log

```text
.harness/
  events/
    <writer-id>.jsonl        # tracked in git; append-only; one file per writer
  snapshots/                 # phase 3: compacted state + watermark
harness.db                   # gitignored; materialized cache; disposable
```

- **Writer identity**: stable per human/agent — derived from `git config
  user.email` (hashed short form) or `HARNESS_WRITER` override. Per-writer files
  make concurrent appends conflict-free by construction: parallel branches touch
  different files, merges are unions.
- **Event shape** (one JSON object per line):

```json
{
  "event_id": "01J...ULID",
  "writer": "a1b2c3",
  "recorded_at": "2026-07-02T04:00:00Z",
  "op": "story.update",
  "schema": 1,
  "payload": { "id": "US-028", "unit": true, "evidence": "..." }
}
```

- **Ops** mirror the existing CLI surface one-to-one: `intake.record`,
  `story.add|update|verify_result`, `decision.add|verify_result`,
  `backlog.add|close`, `tool.register|remove`, `trace.record`,
  `intervention.add`, `signal.add`. Tool **scan status** (`tool check`) is
  machine-local reality, not team state — it stays cache-only, never logged.

### Ordering and conflict semantics

- Total order for replay: `(event_id ULID, writer)` — ULIDs are time-ordered
  with per-writer monotonic tiebreak, so clock skew between writers reorders
  only causally-unrelated events.
- Inserts commute. Field updates are **last-writer-wins by that order**, with
  the losing event still visible in the log (provenance is free). A new `audit`
  category surfaces concurrent updates to the same story field within a merge
  window, so LWW is observable rather than silent.
- IDs: `story`/`decision` already use human-assigned TEXT keys (unchanged).
  Auto-increment integer IDs (`backlog`, `trace`, `intake`, `intervention`,
  `signal`) become ULIDs; the CLI accepts unambiguous short prefixes and prints
  short forms, preserving `backlog close --id <x>` ergonomics.

### Write and read paths

- **Write**: command → validate → append event to own writer file → apply the
  *same event* to the local cache in one operation. The cache is only ever
  written from events, so cache and log cannot drift.
- **Read**: unchanged — SQL against `harness.db`. The cache stores a
  **watermark** (count + hash of consumed events per writer file). Every
  command first compares watermarks; new events (e.g. after `git pull`) are
  incrementally replayed before the query runs. `harness-cli rebuild` does the
  full deterministic replay from genesis.

### Generated views

`TEST_MATRIX.md`, `HARNESS_BACKLOG.md`, and the decision index become generated
views (regenerated after mutations, marked "generated — do not hand-edit"),
following the `status.md` pattern. `import brownfield` remains as the one-time
seed for pre-event-log repos and becomes a migration tool, not a sync tool.

### Migration

`harness-cli migrate-to-events` reads the existing DB and synthesizes genesis
events (writer = `migration`, original timestamps preserved), then verifies:
rebuilt cache row counts and per-table content hashes equal the original DB.
Only after that proof does the tool write the log and demote the DB. The old DB
is backed up, never deleted.

### Integrity (phase 3, optional)

Per-file hash chain (`prev_hash`/`record_hash`, the `okra-store.sh` record
shape) plus `harness-cli events verify`. Deferred: git history already provides
tamper evidence for a team; the chain adds standalone verifiability.

## Phasing (candidate stories)

| Story | Scope | Risk |
| --- | --- | --- |
| US-028a | Shadow mode: emit events alongside current writes; `rebuild` command; determinism proof in CI | normal |
| US-028b | Cutover: cache demoted, watermark replay, ULID ids, generated views, `migrate-to-events` | high-risk |
| US-028c | Compaction/snapshots, hash-chain integrity, `events verify` | normal |

Shadow mode first is the safety property: US-028a changes no behavior and lets
determinism be proven on real data before anything depends on it.

## Acceptance Criteria

- Two clones make disjoint durable writes on branches; `git merge` completes
  with zero conflicts; `rebuild` on the merge result contains every record from
  both writers.
- A fresh clone with no `harness.db` runs any query command and gets current
  team state (auto-replay from events).
- `migrate-to-events` on a populated v6 DB reproduces all rows byte-equivalently
  (per-table hash proof) and is idempotent.
- Replay is deterministic: two independent rebuilds of the same log produce
  identical dumps.
- `scripts/calibrate-harness.sh` passes unchanged against the event-backed CLI.
- Concurrent same-field updates are resolved LWW and surfaced by the new audit
  category, not silently.
- p95 CLI mutation latency ≤ 100 ms with a 10k-event log (incremental apply).

## Validation

| Layer | Proof |
| --- | --- |
| Unit | event serialization round-trip; ULID ordering with skewed clocks; LWW resolution; watermark math |
| Integration | append→apply consistency; incremental replay after simulated pull; migration hash-equality on fixture DBs |
| E2E | two-writer git-merge scenario (the `merge_conflict_count == 0` anti-goal read) |
| Calibration | extend `calibrate-harness.sh`: golden event logs → expected rebuilt state; corrupted log line → rebuild refuses |
| Benchmark | mutation latency at 1k/10k/100k events |

## Risks And Mitigations

| Risk | Mitigation |
| --- | --- |
| Clock skew reorders events across writers | ULID + per-writer monotonic counter; only causally-unrelated events can reorder; LWW is audited |
| Log grows unbounded (traces are chatty) | phase 3 snapshots + watermark; telemetry retention policy decided at ratification |
| Agent queries stale cache after pull | watermark check on every command — staleness is detected, not trusted |
| Event schema evolves | `schema` field per event + upcasters in replay; never rewrite old events |
| Partial adoption breaks teammates | shadow phase proves determinism first; cutover is one atomic story with migration proof |
| ULID ergonomics regress numeric ids | short-prefix acceptance + short display forms; help text updated |

## Non-Goals

- A shared database service (Turso/Postgres/rqlite) — rejected: breaks the
  drop-in property and moves truth out of the repository.
- Real-time sync between machines — git remains the transport.
- Multi-repo/org-wide aggregation — out of scope for this epic.
- Logging `tool check` scan results — machine-local reality stays local.

## Open Questions (human ratification required)

1. **Telemetry in git: yes or no?** Tracking `trace`/`intervention`/`signal`
   events enables team-wide `propose` mining (the main prize) but adds commit
   noise and grows the log fastest. Options: track all (default proposed),
   track with rotation, or keep telemetry local and log only contract tables.
2. **LWW vs field-level merge** for concurrent `story.update` — LWW + audit is
   proposed; field-merge is more correct and more complex.
3. **Writer identity source** — `git config user.email` hash (proposed) vs
   explicit `HARNESS_WRITER` requirement.
4. **Retire `import brownfield` as sync** — becomes migration-only. Confirm no
   workflow still depends on markdown → DB seeding post-cutover.

## Harness Maintenance

On implementation: new decision record superseding `0004-sqlite-durable-layer`;
update `docs/HARNESS.md`, `docs/TOOL_REGISTRY.md` (outbound manifest: `rebuild`,
`events verify`, `migrate-to-events`), installer `.gitignore` rules (keep
ignoring `harness.db*`, never ignore `.harness/events/`), both installer
manifests if new shipped files appear, and `scripts/calibrate-harness.sh`
goldens.
