# US-028b Exec Plan

## Goal

`teammate_visible_tables == 8/8`: every durable table round-trips through git
via the event log; `harness.db` is a rebuilt cache; two writers merge with
zero conflicts.

## Scope

In scope: ULID row ids, `migrate-to-events`, cutover write path with
watermarks, causal audit, generated views, two-writer e2e, calibration
goldens, gitignore/docs/decision maintenance, dogfood migration of this repo.

Out of scope (US-028c): compaction/snapshots, hash-chain integrity,
`events verify`, telemetry retention policy.

## Stages — each lands with tests green before the next starts

| # | Stage | Proof |
| --- | --- | --- |
| S1 | Schema 007: TEXT/ULID ids + prefix resolution + cache_meta/watermark tables | unit tests: migration on populated v6 DB preserves rows; prefix resolve exact/unique/ambiguous |
| S2 | `migrate-to-events`: genesis synthesis, equality proof, idempotence, backup, shadow-log archive | unit tests mirroring DKR-3 (count+hash equality incl. tool-col exclusion; byte-idempotent re-run) |
| S3 | Cutover write path: append-first (hard fail), apply-from-event, stat-short-circuit watermark, incremental replay, fresh-clone auto-rebuild | unit+integration: mutate → cache==rebuild; simulated pull replays; missing db rebuilds; unwritable log FAILS command |
| S4 | Causal audit: `observed` field, concurrency detection, `lww_audit`, audit surfacing | unit tests from DKR-4 scenarios: concurrent flagged at any skew, sequential (observed) not flagged |
| S5 | Generated views: TEST_MATRIX.md, HARNESS_BACKLOG.md regeneration + migration guard | unit: stable output, generated marker, unimported-row guard |
| S6 | Two-writer git e2e, calibration goldens, bench script, gitignore, docs, decision 0009, dogfood migration of this repo | e2e test green; calibration green; `story verify US-028b`; matrix row updated |

## Risks

- Biggest blast radius is S3 (every write path). Mitigation: `apply_event` is
  already the single replay entry point from US-028a; S3 reroutes writers
  through it rather than duplicating SQL.
- Dogfood migration mutates this repo's own durable layer — run last, after
  fixture proofs, with backup verified.
- Id type change ripples through domain/application/interface record types —
  mechanical but wide; the compiler is the net.

## Done means

All acceptance criteria of the parent spec that belong to cutover hold: the
two-clone merge scenario, fresh-clone auto-replay, migration
hash-equality + idempotence, LWW+causal-audit surfacing, calibration green,
plus `story verify US-028b` passing and this repo actually running
event-backed.
