# OKRA Delegated Loop — Weekly Newsletter Open Rate

Run id: `2026-09-nl-open-rate`. Run store root: `.okra/runs/2026-09-nl-open-rate/`.
Artifact status: delegated-loop handoff contract, ratified frame, round 3 of an open loop.

## 1. Frame (candidate frame -> human-ratified frame)

**Objective.** Raise the weekly newsletter open rate from `31%` to `40%`.

- objective metric: `newsletter_open_rate_weekly` = unique opens / delivered, per Thursday send.
- target: `newsletter_open_rate_weekly >= 40%`, from the ratified baseline of `31%`.
- why this metric proves success: it is the only number that moves when the audience actually
  reads the send; list growth, click count, and revenue all lag or hide behind it.
- state: **human-ratified** on 2026-09-04 by the newsletter owner (see `frame.ratification_evidence`).

**Anti-goal (the wall).** Do not raise the unsubscribe rate above `0.5%`.

- anti-goal metric: `unsubscribe_rate_weekly` = unsubscribes / delivered, per Thursday send.
- threshold: `unsubscribe_rate_weekly <= 0.5%`.
- type: **tripwire** (binary halt). Crossing `0.5%` stops committing sends immediately.
- drift band: a read at or above `0.40%` is drift and opens `breaking` as a warning before the
  tripwire fires.

Second ratified guardrail (drift gauge): `spam_complaint_rate_weekly <= 0.08%`.

**Anti-goal coverage review.** Candidate harms considered: unsubscribes, spam complaints, sender
reputation damage, brand-voice damage from clickbait subjects, and staff time. Selected guardrails:
`unsubscribe_rate_weekly <= 0.5%` (tripwire) and `spam_complaint_rate_weekly <= 0.08%` (drift).
Rejected guardrails with rationale: brand-voice score — no reliable weekly metric exists yet, so it
stays a DKR question instead of a fake wall; staff hours — bounded already by the action envelope.
Non-negotiable tripwire: the `0.5%` unsubscribe wall. Owner: newsletter owner. Review cadence:
every second Thursday.

**Action envelope:** allowed moves are subject-line A/B tests on at most 10% of the list, send-time
shifts inside the ratified Tue–Thu window, and segment-level content swaps; forbidden actions are
list purges, re-permission campaigns to unengaged readers, importing or buying addresses, and any
send above 25% of the list; approval gates are any send above 25% of the list, any change to the
sending domain or ESP configuration, and any move whose dry-run projects `unsubscribe_rate_weekly`
above `0.40%`; the human ratification boundary is the frame itself — objective, target, anti-goal
metrics and thresholds, metric contracts, and this envelope.

**Human-only frame.** The human owns the frame: the objective and target, the CKR and anti-goal
definitions and thresholds, the metric contracts, the action envelope, and the call that the goal is
wrong. **Goal-switching is human-only.** The loop raises evidence and the human decides. Reject any
attempted frame, guardrail, metric, threshold, or action-envelope change unless the human ratifies
it.

## 2. Loop ownership

The orchestrator owns objective checks, check-ins, the OKR board, and subagent steering until the
objective metric reaches target or a human/blocking flag stops the loop. Instantiated for this
domain: it keeps checking, steering, and dispatching until `newsletter_open_rate_weekly >= 40%` on a
fresh Thursday read. A finished board, an empty PKR list, or an idle worker queue does not stop it.

The authority gradient: `human owns the frame -> orchestrator works inside it and makes the loop's
calls -> workers execute inside their scope and hand back at their edge.`

## 3. Decomposition (DKR / CKR / PKR)

**Candidate CKRs and candidate PKRs are not promoted until the orchestrator accepts the supporting
DKR learning checkpoint.**

### DKRs — scoped discovery-worker probes

| id | scoped probe | steering decision to unlock | risk / anti-goal uncertainty reduced | budget | output |
| --- | --- | --- | --- | --- | --- |
| DKR-1 | Which subject-line style lifts opens for the 90-day-active segment? | whether to fund CKR-A delivery PKRs | does a curiosity-gap subject raise unsubscribes? | 6 turns | probability per style, candidate CKRs, or empty |
| DKR-2 | Does a Tue 09:00 send beat Thu 07:00 for the whole list? | whether to promote CKR-B off candidate status | send-time shifts can spike complaints in one timezone | 4 turns | confidence interval on open-rate delta |
| DKR-3 | Is the 31% baseline depressed by dead addresses rather than content? | whether to re-aim away from content work entirely | list hygiene moves are close to the forbidden purge action | 5 turns | evidence refs, or empty |

Each DKR is complete only when it writes a learning checkpoint: decision target, evidence collected,
questions answered and unanswered, probability/confidence update, risk or anti-goal implications,
candidate CKRs, and the next unknowns.

### CKRs — measurable contribution context

CKRs are measurable contribution context with mini reverse-tornado discovery/delivery balance, and
are **not subagent work**. A CKR is never dispatched as a worker job.

- **CKR-A — subject-line lift.** Metric `open_rate_delta_subject_variant >= +4pp` on the tested
  segment. **CKR-level discovery/delivery balance:** discovery side is DKR-1 (which style wins, and
  at what unsubscribe cost); delivery path is the PKR that ships the winning style to the full
  Thursday send after the checkpoint is accepted.
- **CKR-B — send-time lift.** Metric `open_rate_delta_send_time >= +2pp`. **CKR-level
  discovery/delivery balance:** discovery side is DKR-2 (does the shift hold across timezones);
  delivery path is the PKR that moves the recurring schedule in the ESP.
- **CKR-C — engaged-segment share.** Metric `active_90d_share >= 62%` (now `54%`). **CKR-level
  discovery/delivery balance:** discovery side is DKR-3 (is the denominator the real problem);
  delivery path is a re-engagement PKR that stays inside the action envelope.

### PKRs — progression-worker execution units

| PKR | linked_ckr | source_dkr_checkpoint | contribution_metric | done check | allowed actions | forbidden actions | hand-back rule |
| --- | --- | --- | --- | --- | --- | --- | --- |
| PKR-A1 ship winning subject style to full send | CKR-A | DKR-1/checkpoint-2 | `open_rate_delta_subject_variant` | one Thursday send shipped and read after the 48h lag window | edit subject template, schedule send | change segment size, edit ESP domain settings | hand back on unknown discovery |
| PKR-B1 move recurring schedule to Tue 09:00 | CKR-B | DKR-2/checkpoint-1 | `open_rate_delta_send_time` | schedule changed and two sends read | edit send schedule | change list membership | hand back on unknown discovery |
| PKR-C1 re-engagement sequence for 90-day-inactive | CKR-C | DKR-3/checkpoint-1 | `active_90d_share` | sequence sent to <=10% of list, metrics read | send sequence to ratified slice | purge or re-permission the list | hand back on unknown discovery |

**PKR discovery hand-back.** PKRs hand back on unknown discovery instead of researching or resolving
unknowns inside execution. A progression worker that meets an unknown mid-run stops and returns to
the orchestrator, which decides whether to fund a discovery worker.

PKRs report progress signals at every check-in: off-track work, quality drift, churn, late
discovery, stale metrics, and scope or authority concerns.

## 4. Worker prompt packet contract

Every dispatch is a worker prompt packet, never a raw continuation of a previous worker's chat.
Packet keys:

- `frame.objective` — `newsletter_open_rate_weekly >= 40%` from `31%`.
- `frame.anti_goals` — `unsubscribe_rate_weekly <= 0.5%` (tripwire), `spam_complaint_rate_weekly <= 0.08%` (drift).
- `frame.action_envelope` — allowed moves, forbidden actions, approval gates.
- `frame.human_ratification_boundary` — only the human may change the frame.
- `current_state` — round, board state, last metric reads, open flags.
- `previous_dkr_checkpoint` — ref to the accepted checkpoint this dispatch builds on.
- `assignment` — the one scoped DKR probe or one PKR task.
- `budget_and_stop_rule` — turn/time budget and the exact stop condition.
- `hand_back_rule` — hand back at the edge of scope; do not improvise.
- `output_schema` — the exact record shape the worker must return.

## 5. In-progress influence rule

**In-progress worker narrative is not evidence; only worker progress, check-ins, metric reads,
flags, or accepted checkpoints can influence the next dispatch.**

Long-running workers write file-based progress reports under
`.okra/runs/2026-09-nl-open-rate/workers/<worker-id>/progress.jsonl`, written at each finish, when an
unknown is hit, and on a timed heartbeat.

## 6. DKR-to-DKR handoff

Fields carried from one discovery probe to the next: `previous_dkr_checkpoint`, `decision_target`,
`evidence_refs_or_hashes`, `questions_answered`, `questions_unanswered`,
`confidence_probability_update`, `risk_or_anti_goal_implications`, `orchestrator_decision`, and
`next_dkr_scope`.

Worked instantiation (round 2 -> round 3):

- previous DKR learning checkpoint: `DKR-1/checkpoint-2` (subject-line style probe).
- decision target: decide whether to fund PKR-A1 and ship the curiosity-gap subject style to the
  full Thursday list.
- evidence_refs_or_hashes: `sha256:9c41...ab` (variant read export), `sha256:41ee...07` (unsubscribe
  read export).
- questions_answered: curiosity-gap subjects lifted opens by `+5.1pp` on the tested 10% slice.
- questions_unanswered: whether the lift survives at full-list scale, and whether it holds for the
  inactive segment.
- confidence_probability_update: P(style lifts full-send opens by >= +4pp) moved `0.35 -> 0.72`
  after the second read.
- risk_or_anti_goal_implications: the tested slice read `unsubscribe_rate_weekly = 0.21%`, below the
  `0.40%` drift band, so admissibility passes; the spam-complaint read stayed at `0.03%`.
- orchestrator_decision: checkpoint **accepted**; CKR-A promoted off candidate status, PKR-A1 funded.
- next_dkr_scope: DKR-4 — does the same style hold for the 90-day-inactive segment at full scale?

## 7. Eval Points

- **Admissibility before action**: before dispatching PKR-A1, the orchestrator screens the move
  against a fresh `unsubscribe_rate_weekly` read or a dry-run projection. A move projected above
  `0.40%` is vetoed and never reaches a worker. *Example: move "clickbait subject to full list" ->
  projected `unsubscribe_rate_weekly = 0.61%` -> VETOED, off the menu.*
- **Direct read after action**: after a worker returns, the loop reads the real
  `newsletter_open_rate_weekly`, the CKR metrics, and `unsubscribe_rate_weekly` from the ESP export,
  not from what the worker said. *Example: shipped subject style -> open rate reads `35.4%`,
  unsubscribe reads `0.24%`.*
- **Paired goal/anti-goal eval**: success requires both the objective target and every anti-goal
  threshold to hold. Open rate at `40%` with unsubscribes at `0.6%` is a failed loop that looks like
  a win. *Example: open rate `38.9%` up but unsubscribe `0.52%` -> not a win -> FLAG breaking.*

## 8. No cascade

The tree is scaffolding, not scoreboard. The only score that counts is the **direct metric read**
from the ESP export: the objective's number and each CKR's number. A finished PKR subtree with a
flat `newsletter_open_rate_weekly` is not success. While the 48-hour open lag window is still open,
the branch is marked `waiting_for_measurement` and the next read is scheduled. Once the lag window
has closed and fresh reads still show `31%`, the flat metric is a signal the breakdown was wrong.
The anti-goal is measured where it manifests — per-send unsubscribes — and never rolled up.

## 9. Flags

- **Cannot** — discovery budget exhausted or learning flatlined. Example: DKR-3 spends its 5 turns
  and still cannot separate dead addresses from weak content.
- **Breaking** — an anti-metric drifted or tripped. Example: `unsubscribe_rate_weekly` reads `0.44%`
  (drift) or `0.52%` (tripwire). `breaking` pauses committing sends by default.
- **Pointless** — Pointless opens when work finished or a CKR metric moved, but the objective metric
  stays flat / does not move after the lag window. Example: `open_rate_delta_subject_variant` reads
  `+5.1pp` on the slice but `newsletter_open_rate_weekly` stays at `31%` two sends after the lag
  window closed -> stop that branch and re-aim.
- **Authority drift** — the loop or a worker tries to change the frame, relax the `0.5%` threshold,
  expand scope beyond the 10% test slice, bypass an approval gate, contact a human directly, or act
  outside the ratified action envelope. This is a governance breaker: the proposed move stops and
  goes to the human.

**Flag lifecycle.** Every flag is `open`, `acknowledged`, `resolved`, or `waived`, each with a named
owner and a blocking status. `breaking` pauses committing moves; `cannot` and `pointless` stop the
affected branch; `authority drift` stops the proposed move and goes to the human. The orchestrator
may resume only inside the recorded resolution.

| flag | status | owner | blocking | opened_at | resolution record |
| --- | --- | --- | --- | --- | --- |
| breaking-01 | resolved | newsletter owner | yes | 2026-08-27 | slice reduced to 8%, unsubscribe back to `0.19%` |
| pointless-01 | acknowledged | orchestrator | branch only | 2026-09-03 | waiting on the second post-lag read |

## 10. Operating Loop

**Heartbeat cadence and next_check_at:** ten-minute heartbeat for long-running workers (the default,
since the human set no other cadence), plus event-based check-ins on worker completion, unknown
discovery, and flag opening; `next_check_at = 2026-09-07T09:20:00Z`.

- `current_round`: 3.
- Ritual clock: start-of-turn freshness check -> pre-dispatch admissibility -> post-move direct
  metric read -> end-of-turn status write -> idle heartbeat when no worker finishes.
- Every round writes `current_round`, open flags, last metric read, and `next_check_at`.
- Stale-data policy: do not dispatch committing sends on stale metrics unless the human explicitly
  waives that stale state.

**Metric freshness contracts.**

| metric | source of truth | owner | read method | observed_at | recorded_at | freshness | max_age | lag window | missing-data policy |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `newsletter_open_rate_weekly` | ESP export | newsletter owner | CSV export -> checker | `observed_at=2026-09-06T08:00:00Z -> status=fresh against max_age=72h` | 2026-09-06T08:11:00Z | fresh | 72h | 48h open lag | hold dispatch, open `cannot` after two misses |
| `unsubscribe_rate_weekly` | ESP export | newsletter owner | CSV export -> checker | `observed_at=2026-09-06T08:00:00Z -> status=fresh against max_age=72h` | 2026-09-06T08:11:00Z | fresh | 72h | 24h | block committing sends |
| `spam_complaint_rate_weekly` | ESP + postmaster tools | deliverability lead | API read | `observed_at=2026-09-02T08:00:00Z -> status=stale against max_age=72h` | 2026-09-02T08:05:00Z | stale (2026-09-02, stale against the 72-hour max_age) | 72h | 72h | no committing send until refreshed |

**Steering check-in value.** Each check-in records the inbound signal it consumed, the decision or
allocation change it made, the affected CKR/PKR/DKR, the expected or direct metric effect, and a
freshness or evidence reference. Round-3 check-in: inbound signal = `DKR-1/checkpoint-2` accepted;
decision delta = promote CKR-A, fund PKR-A1, hold CKR-C; expected effect = `+4pp` on
`newsletter_open_rate_weekly` after the 48h lag; evidence ref = `sha256:9c41...ab`. Ledger metric
read appended: `steering_value_score >= 0.75`.

## 11. Run store (idempotency and schema)

Every state-changing send gets a stable idempotency key; the store records whether it ran and what it
produced; the orchestrator checks the store before dispatch and writes the outcome after. Dry-run
admissibility has no side effect and needs no key. Append-only records are the source of truth;
status and progress files are generated views.

`frame/frame.v1.json` keys: `frame_version`, `frame_hash`, `objective`, `anti_goals`,
`metric_contracts`, `action_envelope`, and human ratification evidence.

`tree/tree.v1.json` keys: `tree_version`, `frame_version`, `orchestrator`, `dkrs`, `ckrs`, `pkrs`.
The `orchestrator` entry owns **objective checks** and **subagent steering**; `dkrs` and `pkrs` are
worker scopes; `ckrs` are measurable context.

```json
{"tree_version":"tree.v1","frame_version":"frame.v1","orchestrator":{"owns":["objective checks","subagent steering","OKR board","check-ins"]},"dkrs":["DKR-1","DKR-2","DKR-3"],"ckrs":["CKR-A","CKR-B","CKR-C"],"pkrs":["PKR-A1","PKR-B1","PKR-C1"]}
```

Ledger reads are appended through `metric-read`, with `type: "metric_read"`, `metric_kind`,
`metric_id`, `value`, `observed_at`, `source`, and `freshness`. Storage-governance anti-goals are
recorded as zero-valued metric reads for `ungoverned_direct_read`, `ungoverned_direct_write`, and
`single_llm_truth`.

## 12. OKRA Learning Memory

**Previous-run inputs scanned:** `.okra/runs/2026-05-nl-reactivation/` (terminalized) and
`.okra/runs/2026-07-nl-subject-tests/` (terminalized).

- **Traps hit or nearly hit:** the 2026-05 run nearly purged the inactive segment to make the open
  rate look better; the denominator trick would have moved the metric without moving the goal.
- **Avoidances / vetoes that worked:** the dry-run veto of a clickbait subject at projected
  `unsubscribe_rate_weekly = 0.61%`.
- **Misconceptions corrected:** "opens are content-only" — send time explained about a third of the
  variance in the 2026-07 run.
- **Optimization candidates:** run subject and send-time probes in one week instead of two, using
  disjoint slices.
- **Reusable candidate anti-goals with metrics:** see the candidate guardrail library below.
- **Evidence / hashes:** `sha256:9c41...ab`, `sha256:41ee...07`, `sha256:0b7d...c3`.
- **Confidence:** medium-high on send-time effect, medium on subject-style transfer.
- **Context fit:** same list, same ESP, same weekly cadence; the 2026-05 run used a different
  template system, so its template learning is held as candidate only.
- **Ratification status:** all previous-run memories are automatic candidate inputs, not automatic
  authority. The human ratified only the `unsubscribe_rate_weekly <= 0.5%` guardrail for this run.
- **Memory-governance anti-goals (zero-valued):** `unratified_memory_promotion_count == 0`,
  `single_llm_truth_acceptance_count == 0`, and `eval_regression_count == 0`.
- **Terminalization / continuation packet:** both source runs are terminalized with terminal state,
  objective and anti-goal metric refs, unresolved flags, accepted DKR checkpoints, retained trace
  manifest, consolidation output, continuation packet, and second-opinion evidence.
- **Trace manifest:** `trace_manifest_ref: sha256:0b7d...c3` for the 2026-07 run,
  `trace_manifest_ref: sha256:5a10...9f` for the 2026-05 run.
- **Review-set refs:** `review-set/2026-07-nl-subject-tests/{reviewer-a.json,reviewer-b.json}` — two
  independent review artifacts tied to prompt and source hashes.
- **No-regression evidence:** replaying the 2026-07 fixture with this run's guardrails still passes
  the deterministic checker.
- **No-single-LLM-truth evidence:** every promoted learning is backed by a store record, a hash, or
  human ratification; an agent's own final answer is not accepted as proof.

### Candidate guardrail library — `candidate-anti-goals.v1.json`

| field | entry 1 | entry 2 |
| --- | --- | --- |
| `metric_id` | `unsubscribe_rate_weekly` | `spam_complaint_rate_weekly` |
| `threshold` | `<= 0.5%` | `<= 0.08%` |
| `type` | tripwire | drift |
| `applies_when` | any send to more than 5% of the list | any send from the shared sending domain |
| `does_not_apply_when` | transactional receipts | internal test sends |
| `invalidates_when` | the list is re-permissioned or the ESP changes | postmaster reporting changes definition |
| `recertify_by` | 2026-12-01 | 2026-11-01 |
| `source_refs` | `.okra/runs/2026-05-nl-reactivation/ledger.jsonl#seq=412` | `.okra/runs/2026-07-nl-subject-tests/ledger.jsonl#seq=118` |
| `trace_manifest_ref` | `sha256:5a10...9f` | `sha256:0b7d...c3` |
| `candidate_status` | ratified for this run | candidate |
| `no_regression_evidence` | 2026-07 replay passes with this wall in place | no regression observed across two runs |
| ratification boundary | human-only; the orchestrator may propose, never promote | human-only |

At run start the orchestrator may propose these entries as candidate frame inputs, DKR probes, PKR
progress signals, or action-envelope concerns; it must not silently promote them into the active
frame.
