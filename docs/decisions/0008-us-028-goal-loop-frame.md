# 0008 US-028 Goal-Loop Frame Ratification

Date: 2026-07-02

## Status

Accepted

## Context

US-028 (event-log durable layer) was `proposed — awaiting human ratification of
the frame`. The `goal-loop-orchestration` capability is active
(reverse-tornado-okr, scanned `present`), the lane is high-risk and
metric-driven, so the initiative runs as a goal loop per `docs/GOAL_LOOP.md`.
The spec carried a candidate frame and four open questions that set frame and
action-envelope boundaries the loop may not decide itself.

## Decision

The human ratified the frame on 2026-07-02:

- **Objective**: `teammate_visible_tables == 8/8` with
  `rebuild_determinism == 1` (fresh-clone rebuild answers every query
  identically; byte-stable dump comparison).
- **Anti-goals** (frozen set): `merge_conflict_count == 0` (tripwire),
  `migrated_row_loss == 0` (tripwire), `pr_reviewability does not decrease`
  (drift), `calibration stays green` (tripwire), `write_latency_ms <= 100` p95
  per CLI mutation (drift), `story verify-all stays green` (tripwire).
- **Open Question 1 — telemetry in git**: track all telemetry tables
  (`trace`, `intervention`, `story_signal`) as events. DKR-1 measures real
  growth before cutover; phase-3 compaction bounds it.
- **Open Question 2 — merge rule**: LWW + audit, ordered by
  `(event_id ULID, writer)`; losing events stay visible; concurrent same-field
  updates surfaced by a new audit category. DKR-4 validates under clock skew.
- **Open Question 3 — writer identity**: `git config user.email` hash **plus a
  per-clone disambiguator** auto-generated on first write and stored untracked
  (e.g. `.git/harness-writer`). The human uses the same email on two machines;
  a bare email hash would make both clones append to one writer file and
  reintroduce tail-append merge conflicts. The suffix keeps per-writer files
  conflict-free by construction while the shared email prefix preserves
  human-level provenance. `HARNESS_WRITER` remains an override.
- **Open Question 4 — `import brownfield`**: retired as sync; migration-only
  after cutover.

Frame changes from here require re-ratification: reject any attempted frame,
guardrail, metric, threshold, or action-envelope change unless the human
ratifies it.

## Alternatives Considered

1. Telemetry local / contract tables only — rejected: blinds team-wide
   `propose` mining, the initiative's main prize.
2. Field-level merge — rejected for now: more complex replay; LWW + audit is
   observable rather than silent. Revisit if DKR-4 shows silent-loss risk.
3. Bare email-hash writer id — rejected: same email on two machines collides
   into one writer file (see Decision, Q3).
4. Required `HARNESS_WRITER` env var — rejected: setup burden on every
   machine/agent and a hard-fail mode for a default the email hash covers.

## Consequences

Positive:

- The loop can dispatch discovery (DKR-1..4) inside a frozen, human-owned frame.
- Two-machine single-human usage is conflict-free by construction.

Tradeoffs:

- Tracking all telemetry grows the log fastest; retention policy is deliberately
  deferred to DKR-1 evidence and ratified at the US-028c boundary.
- One human appears as multiple writer files (one per clone); tooling that
  groups by writer must group by email prefix.

## Follow-Up

- On implementation, record the superseding decision for
  `0004-sqlite-durable-layer` as the spec's Harness Maintenance section
  requires (this record ratifies the frame; it does not itself supersede 0004).
- Run store for the loop: `.okra/runs/us-028-event-log/`.
