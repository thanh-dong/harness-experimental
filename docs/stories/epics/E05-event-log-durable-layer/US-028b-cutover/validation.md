# US-028b Validation

verify_command: `cargo test -p harness-cli cutover`

| Layer | Proof | Anti-goal read |
| --- | --- | --- |
| Unit | ULID id minting + prefix resolution (exact/unique/ambiguous/legacy-numeric); migration 007 preserves populated v6 rows; genesis determinism + idempotence; per-table hash equality with tool-col exclusion; watermark stat short-circuit + tail-hash fallback; causal concurrency detection (concurrent flagged, observed-sequential not) | `migrated_row_loss == 0` (fixture) |
| Integration | mutate through the event-backed service → cache equals independent rebuild; simulated `git pull` (file grows externally) → incremental replay before query; deleted `harness.db` → auto-rebuild answers queries; unwritable events dir → mutation FAILS (flipped shadow guarantee); generated views stable + marked | cache/log no-drift |
| E2E | two real git clones, disjoint writes with distinct writers, branch merge → **zero conflicts**, rebuild on merge contains every record from both writers | `merge_conflict_count == 0` |
| Calibration | `scripts/calibrate-harness.sh` extended: golden event log → expected rebuilt state; corrupted log line → rebuild refuses; all pre-existing checks stay green | `calibration stays green` |
| Benchmark | mutation latency at 1k/10k/100k events through the real CLI write path; gate p95 ≤ 100 ms at 100k (DKR-2's regression-gate recommendation, stricter than the parent's 10k acceptance point) | `write_latency_ms <= 100` p95 |
| Dogfood | `migrate-to-events` on this repo's live DB: equality proof output, backup present, `query matrix` identical before/after, `story verify-all` green post-cutover | `migrated_row_loss == 0` (real), `story verify-all stays green` |

Weak-proof note: `pr_reviewability` is read at the next PR (generated-view
diff), not by an automated test here.
