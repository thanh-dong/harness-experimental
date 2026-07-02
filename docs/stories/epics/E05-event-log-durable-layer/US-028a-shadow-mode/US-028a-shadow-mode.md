# US-028a Event-Log Shadow Mode: Emit, Rebuild, Prove Determinism

## Status

implemented

## Lane

normal (per the US-028 phasing table: shadow mode changes no behavior; cutover
US-028b is the high-risk story)

## Product Contract

Every durable CLI mutation appends an event to a per-writer, append-only,
git-trackable log at `.harness/events/<writer-id>.jsonl` **alongside** the
existing SQLite write, and `harness-cli rebuild` deterministically replays that
log into a fresh cache. No existing behavior changes: `harness.db` remains the
authoritative read path, all commands print exactly what they printed before,
and a failed event append warns on stderr but never fails the command.

Goal-loop context: this story is PKR-028a in run `us-028-event-log` (CKR-1,
`rebuild_determinism == 1`), promoted from accepted checkpoints DKR-1
(`6cd587dc`) and DKR-2 (`7a204902`). Frame: decision
`docs/decisions/0008-us-028-goal-loop-frame.md`.

## Relevant Product Docs

- `docs/stories/epics/E05-event-log-durable-layer/US-028-event-log-durable-layer.md`
  (parent spec: event shape, op names, writer identity, ordering)
- `docs/GOAL_LOOP.md` (integrity rule this implements)

## Acceptance Criteria

- Every mutating command (`intake`, `story add|update|verify|verify-all|signal
  add`, `decision add|verify`, `backlog add|close`, `tool register|remove`,
  `intervention add`, `trace`, `propose --commit`) appends exactly one event
  per durable write, in the parent spec's event shape and op names.
- `tool check` scan results (`status`, `checked_at`) are never logged —
  machine-local reality stays local.
- Writer id = `HARNESS_WRITER` override, else hash of `git config user.email`
  **plus a per-clone disambiguator** stored untracked in the git dir (decision
  0008 Q3: same email on two machines must not share a writer file).
- Event ids are ULIDs (26-char Crockford), time-ordered, monotonic within a
  process for same-millisecond events.
- `harness-cli rebuild` replays `.harness/events/*.jsonl` in
  `(event_id, writer)` order into a fresh SQLite cache and prints per-table
  counts and a deterministic dump hash.
- Two rebuilds of the same log produce identical dump hashes
  (`rebuild_determinism == 1`) — proven by a unit test that runs in CI.
- Event append failure does not fail the command (shadow guarantee) — warning
  on stderr only.
- `scripts/calibrate-harness.sh` and `story verify-all` stay green.

## Design Notes

- Commands: new `harness-cli rebuild [--output <path>]`; all mutating commands
  gain shadow emission (no flag — always on).
- Emission seam: `HarnessService` (application layer) — every CLI mutation
  passes through it; repository-level tests stay event-free. `propose --commit`
  emits `backlog.add` from the returned committed proposals. `import
  brownfield` is deliberately event-silent: it is the legacy seed that
  `migrate-to-events` (US-028b) replaces; shadow events capture post-shadow
  mutations only.
- Tables: none in `harness.db` — the log is the new surface. Rebuild output
  applies the full schema (001–006) then replays events with explicit
  timestamps taken from the event (`recorded_at`), never `datetime('now')`,
  so replay is time-independent.
- Domain rules: payloads carry the full row including the SQLite integer id
  (parent spec / DKR-3: original ids preserved under `payload.id`; ULID row
  ids are a US-028b concern).
- New module `crates/harness-cli/src/events.rs`; new dependency `serde_json`
  (event serialization + rebuild parsing).
- Not in this story (US-028b): watermark/incremental replay, ULID row ids,
  generated views, `migrate-to-events`, causal concurrent-update audit.
  Per DKR-2, the stat-based watermark lands in US-028b; shadow mode only
  proves the append+replay contract.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | ULID format/ordering/monotonicity; writer-id override + disambiguator; event emission shape; rebuild determinism (two rebuilds → identical dump hash); shadow no-fail on unwritable events dir |
| Integration | service-level: mutate through `HarnessService`, rebuild, compare per-table counts to `harness.db` state written in the same run |
| E2E | deferred to US-028b (two-writer merge is the cutover anti-goal read) |
| Platform | n/a |

verify_command: `cargo test -p harness-cli events`

## Harness Delta

- Parent spec design sections already updated with accepted DKR evidence
  (causal audit, stat short-circuit watermark, migration clauses, count-based
  compaction) — see story signals #5–8.
- On completion: installer `.gitignore` guidance (never ignore
  `.harness/events/`) moves to US-028b cutover; shadow logs are droppable.

## Evidence

- Goal-loop run store: `.okra/runs/us-028-event-log/` (ledger metric reads,
  accepted checkpoints, round-2 board `17909a8d`).
- `cargo test -p harness-cli`: 40 passed, of which 12 cover events/rebuild
  (ULID format/order/monotonicity, writer override, emission shape, shadow
  no-fail, one-event-per-mutation op sequence, rebuild determinism +
  content reproduction, corrupt-line refusal, empty-log rebuild). CI runs
  `cargo test --workspace` (`.github/workflows/harness-cli-release.yml`).
- Real-log proof on this repo (dogfooding, machine-local binary refreshed):
  two `harness-cli rebuild` runs produced identical `dump hash
  992b6d499764942f`; live shadow log at `.harness/events/b6b55092-5630.jsonl`
  (writer id = email hash + per-clone suffix per decision 0008 Q3).
- Walls: `scripts/calibrate-harness.sh` 16 checks green; `story verify-all`
  2 passed, 0 failed, 1 skipped.
