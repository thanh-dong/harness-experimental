# Delegated OKR Loop: Support Ticket First-Response Time

Run id: `okra-frt-2026-09-09`
Drafted: 2026-09-09
Frame state: **candidate** (waiting for human ratification; nothing state-changing runs before that)

Goal as given by the human: cut median support ticket first-response time from `9 hours` to `4 hours` without raising the reopen rate above `8%`.

```
  wide guessing                 narrow known work
  +---------------------------------------------+
  | DKR probes  -> CKR context -> PKR tasks      |   objective: median FRT 9 hours -> 4 hours
  |   zig-zag: learn a slice, act, learn again   |
  +---------------------------------------------+
  ==== wall: reopen rate must stay <= 8% =======   anti-goal (checked 3x per loop)
  human owns frame | orchestrator steers | workers execute
```

Summary in three lines: the objective is a number (median FRT `4 hours`). The wall is a number (reopen rate `8%`). The loop only trusts direct metric reads from the helpdesk data, never a count of finished tasks.

---

## 1. Frame

### 1.1 Objective (candidate until ratified)

| Field | Value |
| --- | --- |
| `metric_id` | `frt_median_hours` |
| Definition | Median, in calendar hours, of `first_substantive_agent_reply_at - ticket_created_at` for all tickets created in the trailing 7-day window. Auto-acknowledgements, bot greetings, and "we received your ticket" macros do **not** count as a first response. |
| Baseline | `9 hours` (human-stated at intake; must be confirmed by a governed source read before round 1) |
| Target | `4 hours` or lower, held for 2 consecutive weekly reads |
| Source of truth | Helpdesk ticket table / weekly FRT report export (owner: support operations lead) |
| Reasoning | Median FRT is the number the human named. A median resists a few very fast or very slow tickets. Excluding auto-replies closes the cheapest way to fake the number. |

### 1.2 Anti-goal (primary, required)

| Field | Value |
| --- | --- |
| `metric_id` | `reopen_rate_pct` |
| Definition | Tickets reopened by the customer within 14 days of being marked solved, divided by tickets marked solved in the same trailing 28-day window, as a percentage. |
| Wall | `8%` (must not go above) |
| Type | **Drift gauge with a tripwire**: warn band starts at `7.0%`; crossing `8%` is the tripwire and pauses committing moves. |
| Baseline | Unknown at intake. DKR-2 reads it first. The human's wording ("not above 8%") implies today's value is at or below `8%`; this is treated as unconfirmed until read. |
| Lag window | 14 days after the solve date. A reopen reading for a period is provisional until 14 days after that period closes. A 7-day early reopen read is used as the drift gauge in the meantime. |

Why this wall: the fastest way to cut first-response time is to answer quickly and badly, or to close tickets on the first reply. Both raise reopens. The reopen rate is where that damage shows up.

### 1.3 Candidate secondary anti-goals (proposed; human may add, remove, or alter)

| Candidate | Metric | Threshold | Type | Why it fits this goal |
| --- | --- | --- | --- | --- |
| Auto-ack gaming | `auto_ack_counted_as_first_response_count` | `== 0` | tripwire | Canned auto-replies would make FRT look like minutes. |
| Customer satisfaction | `csat_pct` | `>= baseline - 2 points` | drift | Fast but shallow replies can hurt CSAT before they show in reopens. |
| Agent overload | `weekly_overtime_hours_per_agent` | `<= 2` | drift | Hitting 4 hours by burning out agents is not a win. |
| Spend | `incremental_monthly_spend_usd` | cap to be set by human (proposed: `0` without approval) | tripwire | No new headcount or tooling without an approval gate. |
| Data boundary | `pii_export_count` | `== 0` | tripwire | No customer data leaves the helpdesk for analysis. |
| DKR budget | `dkr_turns_spent` | `<= 40` per run | tripwire | Unmeasurable work has no natural end. |
| Storage governance | `ungoverned_direct_read`, `ungoverned_direct_write`, `single_llm_truth` | each `== 0` | tripwire | Keeps the run store honest. |
| Memory governance | `unratified_memory_promotion_count == 0`, `single_llm_truth_acceptance_count == 0`, `eval_regression_count == 0` | each `== 0` | tripwire | Keeps learned guardrails from silently entering the frame. |
| Steering quality | `no_value_checkin_count` | `== 0` | tripwire | A check-in that only says "continue" is not steering. |

### 1.4 Anti-goal coverage review

- **Harms considered**: bad or rushed answers (reopens), auto-reply gaming, agent burnout, unapproved spend, customer data leaving the helpdesk, CSAT drop, SLA policy changes that redefine the metric, dropping hard tickets to make the median look better.
- **Selected guardrails**: `reopen_rate_pct <= 8%` (required, ratified set), plus the candidate rows in 1.3.
- **Rejected guardrails, with reason**: "full resolution time" was rejected as a second wall because it overlaps with reopen rate and would slow the loop with two lagging metrics; "ticket volume must not drop" was rejected because volume is not under the loop's control. Both are reconsidered if a DKR shows a gap.
- **Non-negotiable tripwires**: `reopen_rate_pct > 8%`, `auto_ack_counted_as_first_response_count > 0`, `pii_export_count > 0`, any change to the FRT or reopen definition without human ratification.
- **What the wall does not cover**: it does not catch a slow decline in answer quality that never becomes a reopen. CSAT is the candidate cover for that gap.
- **Owners**: support operations lead owns both metric definitions and their reads. The human sponsor owns the frame.
- **Review cadence**: coverage is re-read at every weekly steering round and at end of run.

### 1.5 Action envelope

Action envelope: allowed moves are helpdesk routing and assignment rule changes tested first in staging or dry-run, vetted macro drafting with reviewer sign-off, aged-ticket sweep reports, rota proposals, read-only helpdesk exports with no PII outside the helpdesk; forbidden actions are sending auto-acknowledgements that count as first response, closing or solving tickets to clear the queue, changing SLA or metric definitions, contacting customers outside normal ticket replies, deleting tickets or queues, exporting customer PII, and any new recurring spend; approval gates are required before any staffing or rota change goes live, before any routing rule leaves staging, before any spend, and before any irreversible helpdesk configuration change (rollback plan must be recorded first); the human ratification boundary is that the objective, target, anti-goal definitions and thresholds, metric contracts, and this envelope can only be changed by the human.

Blast-radius limit: routing changes ship to one queue or one ticket category first, then widen only after a direct metric read. Rollback expectation: every committing move records its rollback step in the move result before it runs.

### 1.6 Metric freshness contracts

| Metric | Source of truth | Owner | Read method | `max_age` | Lag window | Missing-data policy |
| --- | --- | --- | --- | --- | --- | --- |
| `frt_median_hours` | helpdesk ticket table | support ops lead | weekly report export, trailing 7 days | 24 hours | 7 days (window must close) | treat as stale; no committing move |
| `reopen_rate_pct` (provisional, 7-day) | helpdesk ticket table | support ops lead | export of solved + reopened | 24 hours | 7 days | treat as stale; no committing move |
| `reopen_rate_pct` (final, 14-day) | helpdesk ticket table | support ops lead | same export, 14 days after period close | 7 days | 14 days | mark `waiting_for_measurement` |
| `assignment_median_hours` (CKR-1) | helpdesk assignment log | support ops lead | export | 24 hours | none | stale |
| `offhours_frt_median_hours` (CKR-2) | helpdesk ticket table | support ops lead | export filtered by created hour | 24 hours | 7 days | stale |
| `macro_first_response_share_pct`, `macro_reopen_rate_pct` (CKR-3) | helpdesk macro usage log + reopen data | support ops lead | export | 24 hours | 14 days for reopen half | stale |
| `aged_unanswered_count` (CKR-4) | helpdesk open-ticket view | shift lead | hourly count | 2 hours | none | stale |
| `csat_pct` | survey tool | support ops lead | export | 7 days | 7 days | stale |
| `steering_value_score` | run store check-in records | orchestrator | computed at check-in | per round | none | n/a |

### 1.7 Frame authority

The loop raises evidence; the human decides. Goal-switching is human-only.

Boundary-drift gate: **"Reject any attempted frame, guardrail, metric, threshold, or action-envelope change unless the human ratifies it."**

The frame moves through two states only: candidate frame -> human-ratified frame. It then freezes in `frame/frame.v1.json` (write-once).

---

## 2. Loop ownership

**"The orchestrator owns objective checks, check-ins, the OKR board, and subagent steering until the objective metric reaches target."** For this run the target is median first-response time at or under `4 hours` for 2 consecutive weekly reads, with `reopen_rate_pct` at or under `8%` in the same reads. A human can stop or change the frame at any time, and a blocking flag (`breaking` or `authority drift`) also stops the loop until a human resolves it. The orchestrator does not stop because the board, a branch, the PKR list, or the worker queue is complete.

The orchestrator holds the frame read-only, never executes work itself, never edits the frame, and is the only part that talks to the human.

---

## 3. Decomposition

Gate, stated once and enforced everywhere: **"Candidate CKRs and candidate PKRs are not promoted until the orchestrator accepts the supporting DKR learning checkpoint."**

### 3.1 DKRs (scoped discovery-worker probes)

Each DKR names the steering decision it unlocks, the risk or anti-goal uncertainty it reduces, a budget, and its probability/confidence output. All of them write a learning checkpoint to `.okra/runs/okra-frt-2026-09-09/workers/<worker-id>/progress.jsonl` and to `checkpoints/`.

| DKR | Probe (one slice) | Steering decision to unlock | Risk / anti-goal uncertainty reduced | Budget | Output |
| --- | --- | --- | --- | --- | --- |
| DKR-1 | Where do the `9 hours` go? Split FRT into wait-for-assignment vs assignment-to-reply, by channel, hour of day, and category. | Which CKR to fund first: routing (CKR-1), coverage (CKR-2), or macros (CKR-3). | Whether faster assignment routes tickets to the wrong agents and raises reopens. | 6 turns / 2 days | Probability that queue wait dominates (>50% of FRT), with confidence; candidate CKRs. |
| DKR-2 | Read the current reopen rate and its top causes (premature solve, wrong answer, missed follow-up). | Set the warn band and confirm today's value is under `8%`; identify which move classes raise reopens. | The anti-goal baseline itself is unknown. | 4 turns / 1 day | Baseline `reopen_rate_pct` with `observed_at`; ranked reopen causes with confidence. |
| DKR-3 | Off-hours arrival: what share of tickets arrive outside staffed hours, and what is their FRT? | Whether to draft a rota proposal (PKR-4, needs approval) or drop CKR-2. | Overtime / agent-overload candidate anti-goal. | 4 turns / 1 day | Probability that off-hours tickets drive the median above `4 hours`. |
| DKR-4 | Which top categories could be answered by a vetted macro without a quality loss? Sample past tickets and their reopen outcomes. | Fund CKR-3 or veto macros. | Macro answers raising reopens; auto-ack gaming boundary. | 6 turns / 2 days | Share of tickets macro-answerable, with per-category reopen risk. |
| DKR-5 (mid-execution) | After the first committing PKR: is the FRT gain holding or decaying week over week? | Continue, pause, or re-aim the funded branch. | Flat metric after finished work (`pointless` trap). | 3 turns | Confidence that the movement is real, not noise. |

A DKR may return empty. Empty is still a result: it tells the orchestrator not to fund that path.

### 3.2 CKRs (measurable contribution context, orchestrator-owned; not worker jobs)

**CKR-1 — Faster assignment.** `assignment_median_hours`: median time from ticket creation to first agent assignment, from current value (DKR-1 reads it) to `<= 1 hour`.
CKR-level discovery/delivery balance: discovery side is DKR-1 (does queue wait dominate, and does fast routing misroute); delivery path becomes PKR-1 (priority routing rule) only after the DKR-1 checkpoint is accepted.

**CKR-2 — Off-hours coverage.** `offhours_frt_median_hours`: median FRT for tickets created outside staffed hours, from current value to `<= 6 hours`.
CKR-level discovery/delivery balance: discovery side is DKR-3 (share and FRT of off-hours arrivals, overtime risk); delivery path becomes PKR-4 (rota proposal behind a human approval gate) only after the DKR-3 checkpoint is accepted.

**CKR-3 — Vetted macro first responses.** `macro_first_response_share_pct` from `0%` to `>= 30%` of first responses in the top categories, paired with `macro_reopen_rate_pct <= 8%`.
CKR-level discovery/delivery balance: discovery side is DKR-4 (which categories are macro-answerable, and their reopen risk); delivery path becomes PKR-3 (macro set with reviewer sign-off) only after the DKR-4 checkpoint is accepted.

**CKR-4 — No aged unanswered tickets.** `aged_unanswered_count`: tickets older than `4 hours` with no substantive reply, read at the start of each shift, from current value to `0`.
CKR-level discovery/delivery balance: discovery side is DKR-1 (whether aged tickets cluster in a queue or a time band); delivery path becomes PKR-2 (aged-ticket sweep) only after the DKR-1 checkpoint is accepted.

A CKR metric moving does not count as objective progress. Only `frt_median_hours` does.

### 3.3 PKRs (progression-worker execution units; candidate until promoted)

| Field | PKR-1 Priority routing rule | PKR-2 Aged-ticket sweep | PKR-3 Vetted macro set | PKR-4 Off-hours rota proposal |
| --- | --- | --- | --- | --- |
| `linked_ckr` | CKR-1 | CKR-4 | CKR-3 | CKR-2 |
| `source_dkr_checkpoint` | DKR-1 checkpoint (pending) | DKR-1 checkpoint (pending) | DKR-4 checkpoint (pending) | DKR-3 checkpoint (pending) |
| `contribution_metric` | `assignment_median_hours <= 1 hour` | `aged_unanswered_count == 0` | `macro_first_response_share_pct >= 30%` and `macro_reopen_rate_pct <= 8%` | `offhours_frt_median_hours <= 6 hours` |
| Done check | Rule live on one category; dry-run on last 7 days shows `assignment_median_hours` under 1 hour; direct read after 7 days confirms | Hourly report exists; two shifts in a row start at `0` aged tickets by direct count | 10 macros signed off by a reviewer; usage logged; 14-day reopen read for macro tickets `<= 8%` | Proposal document with cost and overtime effect delivered to the human; not live until approved |
| Allowed actions | Edit routing rules in staging; dry-run against historical tickets; ship to one category after approval | Build and schedule a read-only report; assign tickets within existing queues | Draft macros; request reviewer sign-off; enable macros for the vetted categories | Read staffing and arrival data; write the proposal |
| Forbidden actions | Changing SLA policy; deleting queues; touching more than one category before a direct read | Solving or closing tickets; sending customer messages | Auto-sending macros; macros that close tickets; macros counted as auto-acks | Changing any shift live; promising hours to agents; any spend |
| Hand-back rule | If the rule needs a skill or category mapping that does not exist, hand back | If aged tickets keep coming from one unknown source, hand back | If a category has no clear correct answer, hand back | If arrival data is missing for a channel, hand back |
| Progress signals reported at check-ins | off-track, quality drift, churn (rule rewritten more than twice), late discovery, stale metric, scope or authority concern | same | same | same |

PKR discovery hand-back: PKRs hand back on unknown discovery instead of researching or resolving unknowns inside execution; the orchestrator then decides whether to spawn a discovery worker.

### 3.4 DKR-to-DKR handoff

Every DKR worker packet that follows another DKR carries these fields, filled from the accepted checkpoint, never from chat memory:

| Field | Example for the DKR-1 -> DKR-4 handoff |
| --- | --- |
| `previous_dkr_checkpoint` | `checkpoints/dkr-1.v1.json` (by content hash) |
| `decision_target` | Fund CKR-3 (macros) or veto it |
| `evidence_refs_or_hashes` | sha256 of the FRT split export; sha256 of the category list |
| `questions_answered` | Which categories carry most of the queue wait |
| `questions_unanswered` | Whether those categories are macro-answerable without raising reopens |
| `confidence_probability_update` | P(queue wait dominates) moved from 0.5 prior to the DKR-1 posterior |
| `risk_or_anti_goal_implications` | Macros in categories with high reopen causes are a `reopen_rate_pct` risk |
| `orchestrator_decision` | Accept checkpoint; spawn DKR-4 with the top 5 categories as scope |
| `next_dkr_scope` | DKR-4, budget 6 turns, those 5 categories only |

---

## 4. Roles and the worker prompt packet

Authority gradient: human owns the frame -> orchestrator works inside it and makes the loop's calls -> workers execute inside their scope and hand back at their edge.

Workers are disposable and parallel. There are two kinds: discovery workers (one DKR each) and progression workers (one PKR or task each). There is no CKR worker. A worker cannot screen its own moves against the anti-goal, cannot call the human, and cannot change its scope.

### 4.1 Worker prompt packet contract

Every dispatch is a fresh packet with exactly these fields; it is never a continuation of a previous worker's chat.

```json
{
  "frame.objective": "frt_median_hours from 9 hours to <= 4 hours (ref frame/frame.v1.json by frame_hash)",
  "frame.anti_goals": "reopen_rate_pct <= 8% (drift, tripwire at 8%), plus ratified secondary set",
  "frame.action_envelope": "see section 1.5; copied verbatim from frame/frame.v1.json",
  "frame.human_ratification_boundary": "objective, target, anti-goals, thresholds, metric contracts, envelope: human-only",
  "current_state": "current_round, open flags, last metric reads with freshness, board snapshot ref",
  "previous_dkr_checkpoint": "checkpoints/<dkr-id>.v1.json hash, or null",
  "assignment": "one DKR probe or one PKR task, scoped as in section 3",
  "budget_and_stop_rule": "N turns / T hours; stop at budget, at done check, or at unknown",
  "hand_back_rule": "on any unknown, stop and write a hand-back record; do not improvise",
  "output_schema": "learning checkpoint (DKR) or move result + progress signals (PKR)"
}
```

### 4.2 In-progress influence rule

**"In-progress worker narrative is not evidence; only worker progress, check-ins, metric reads, flags, or accepted checkpoints can influence the next dispatch."** A worker's own final answer can point to evidence (a file hash, an export, a query result); it is not the evidence.

---

## 5. Storage: run store, idempotency, resume

Storage is set up before any move runs, so the run is safe to interrupt and resume. Without it, a re-run could ship a routing rule twice or count a reopen period twice against the wall.

### 5.1 Layout

```
.okra/
  content/sha256/<hash>            shared, content-addressed evidence (exports, macros, proposals)
  runs/okra-frt-2026-09-09/
    frame/frame.v1.json            write-once; freezes the ratified frame
    tree/tree.v1.json              board: orchestrator, dkrs, ckrs, pkrs
    checkpoints/<dkr-id>.v1.json   accepted DKR learning checkpoints
    moves/<idempotency-key>.json   write-once per key; result + rollback step
    ledger.jsonl                   append-only metric and anti-goal reads
    flags.jsonl                    append-only flag records with lifecycle
    checkins.jsonl                 append-only steering check-in records
    workers/<worker-id>/progress.jsonl   file-based worker progress + heartbeat
    status.json                    GENERATED VIEW ONLY; rebuilt from the records above
```

Integrity rule: append-only records are the source of truth; `status.json` and any progress summary are generated views. A stale or contradictory status file is a signal, not evidence. The store is verified before every resume and before any success report. If more than one OKRA loop runs in this workspace, each keeps its own `runs/<run-id>/`; only `content/sha256` is shared.

### 5.2 Frame schema (`frame/frame.v1.json`)

```json
{
  "frame_version": 1,
  "frame_hash": "<sha256 of canonical JSON, computed by write-frame at ratification>",
  "objective": {
    "metric_id": "frt_median_hours",
    "baseline": "9 hours",
    "target": "4 hours",
    "hold_rule": "2 consecutive weekly reads",
    "definition": "median calendar hours to first substantive agent reply, trailing 7 days; auto-acks excluded"
  },
  "anti_goals": [
    { "metric_id": "reopen_rate_pct", "threshold": "8%", "type": "drift_with_tripwire", "warn_band": "7.0%", "lag_window_days": 14 },
    { "metric_id": "auto_ack_counted_as_first_response_count", "threshold": 0, "type": "tripwire", "status": "candidate" },
    { "metric_id": "pii_export_count", "threshold": 0, "type": "tripwire", "status": "candidate" },
    { "metric_id": "ungoverned_direct_read", "threshold": 0, "type": "tripwire" },
    { "metric_id": "ungoverned_direct_write", "threshold": 0, "type": "tripwire" },
    { "metric_id": "single_llm_truth", "threshold": 0, "type": "tripwire" },
    { "metric_id": "no_value_checkin_count", "threshold": 0, "type": "tripwire" }
  ],
  "metric_contracts": "section 1.6, one entry per metric_id with source, owner, definition, read_method, max_age, lag_window, missing_data_policy",
  "action_envelope": "section 1.5, verbatim",
  "human_ratification": { "status": "pending", "ratified_by": null, "ratified_at": null, "evidence_ref": null }
}
```

### 5.3 Tree schema (`tree/tree.v1.json`)

```json
{
  "tree_version": 1,
  "frame_version": 1,
  "orchestrator": {
    "owns": ["objective checks", "check-ins", "OKR board", "subagent steering", "admissibility screening", "DKR learning gate"],
    "stop_conditions": ["objective target reached and held", "human stops or changes frame", "blocking flag open"]
  },
  "dkrs": [ { "id": "DKR-1", "scope": "worker", "status": "candidate", "budget_turns": 6 }, "..." ],
  "ckrs": [ { "id": "CKR-1", "kind": "measurable context", "metric_id": "assignment_median_hours", "status": "candidate" }, "..." ],
  "pkrs": [ { "id": "PKR-1", "scope": "worker", "linked_ckr": "CKR-1", "source_dkr_checkpoint": null, "status": "candidate" }, "..." ]
}
```

Both files are written through `write-frame` and `write-tree` when the helper is available, and `verify` runs before any success report.

### 5.4 Idempotency keys and move results

Every committing move gets a stable key, for example `move:pkr-1:routing-rule:category=billing:frame_v1`. The orchestrator checks `moves/<key>.json` before dispatch and writes the outcome after. A known key returns the stored result; the routing rule is not shipped twice. Dry-run (propose-cost) workers have no side effect, so they need no key; dry-run is the default for any move whose reopen cost cannot be known up front.

Resume sequence: verify store -> load frame by hash -> load tree -> replay ledger and flags -> rebuild `status.json` -> resume only inside recorded flag resolutions.

### 5.5 Ledger reads

Objective and anti-goal readings go through `metric-read`, not generic append. Example record:

```json
{ "type": "anti_goal_metric_read", "metric_kind": "anti_goal", "metric_id": "reopen_rate_pct",
  "value": null, "unit": "percent", "observed_at": null, "recorded_at": "2026-09-09T00:00:00Z",
  "source": "pending DKR-2 export", "freshness": "stale against max_age=24h (no source read yet)" }
```

Storage-governance reads are appended as zero-valued metric reads each round: `ungoverned_direct_read = 0`, `ungoverned_direct_write = 0`, `single_llm_truth = 0`. Memory-governance reads are appended the same way: `unratified_memory_promotion_count == 0`, `single_llm_truth_acceptance_count == 0`, `eval_regression_count == 0`. Important content is read by content hash or a recorded check-in, and written through the store helper with target path plus content hash.

---

## 6. Eval Points

- **Admissibility before action**: the orchestrator screens objective moves against fresh anti-goal readings or a dry-run before dispatch. For this goal: move "auto-acknowledge every new ticket within 5 minutes" -> would trip `auto_ack_counted_as_first_response_count` and is outside the envelope -> VETOED, off the menu. Move "solve tickets on first reply to clear the queue" -> dry-run projects `reopen_rate_pct` at `14%` -> VETOED. Move "priority routing rule on the billing category" -> dry-run on the last 7 days projects `assignment_median_hours` at `0.8 hours`, FRT median at `6.5 hours`, reopen unchanged -> ADMITTED, dispatched to a progression worker.
- **Direct read after action**: the loop reads the real objective, CKR, and anti-goal metrics from source records after workers return. For this goal: after PKR-1 ships, the orchestrator reads `frt_median_hours` from the helpdesk export (say `6.8 hours`), `assignment_median_hours` (say `0.9 hours`), and the provisional 7-day `reopen_rate_pct` (say `6.9%`, in band under the `7.0%` warn line). The worker saying "it stayed safe" is not a read.
- **Paired goal/anti-goal eval**: the loop checks objective progress and anti-goal hold together; success requires both the objective target and every anti-goal threshold to hold. For this goal: `frt_median_hours` at `4.0 hours` with `reopen_rate_pct` at `9.1%` is not a win; it opens `breaking`. `frt_median_hours` at `4.0 hours` with `reopen_rate_pct` at `7.4%` is a win only after the 14-day lag closes and the second weekly read agrees.

---

## 7. Flags

All four run at once. Lifecycle for every flag: `open` -> `acknowledged` -> `resolved` or `waived`. The orchestrator resumes only inside the recorded resolution.

| Flag | Opens when | Domain example | Behavior |
| --- | --- | --- | --- |
| **Cannot** | DKR budget exhausted or learning flatlined; effort in, nothing back. | DKR-1 spends 6 turns and still cannot split FRT because the helpdesk export lacks assignment timestamps. | Stops the affected branch; hands the human the spent budget, the tree, and the evidence gap. |
| **Breaking** | An anti-metric drifted into the warn band or tripped the wall. | Provisional `reopen_rate_pct` reads `7.6%` (drift warn) or `8.3%` (tripped). | Pauses committing moves by default; dry-runs may continue; human resolves. |
| **Pointless** | **"Pointless opens when work finished or a CKR metric moved, but the objective metric stays flat / does not move after the lag window."** | PKR-2 brings `aged_unanswered_count` to `0` and PKR-1 brings `assignment_median_hours` to `0.9 hours`, but `frt_median_hours` still reads `8.7 hours` after the 7-day lag window closes. | Stops the affected branch; the funnel narrowed toward the wrong tip; the orchestrator hands up the evidence and the human re-aims (for example toward CKR-2 or CKR-3). |
| **Authority drift** | The loop or a worker tries to change the frame, relax `8%`, redefine FRT, expand scope, bypass an approval gate, contact the human directly, or act outside the envelope. | A worker proposes counting the auto-ack as first response "because it is technically a reply", or ships the routing rule to all categories at once. | Stops the proposed move; goes to the human; governance breaker, not just an invalid move. |

Before the lag window closes, a flat objective read marks the branch `waiting_for_measurement` and schedules the next read; it does not open `pointless` yet.

---

## 8. Operating Loop

### 8.1 Cadence and current position

- `current_round`: 0 (frame candidate; ratification pending)
- Steering round cadence: weekly, plus event-based check-ins on worker completion, unknown discovery, and flag opening
- Heartbeat cadence and next_check_at: 10-minute heartbeat for every live worker (human has not set a different cadence); `next_check_at = 2026-09-10T09:00 local`, which is the ratification check-in; after that, the first weekly steering round is `2026-09-16T09:00 local`.

### 8.2 Current metric freshness classification

| Metric | Value | `observed_at` | `recorded_at` | Status | `max_age` |
| --- | --- | --- | --- | --- | --- |
| `frt_median_hours` | `9 hours` (human-stated) | `observed_at=unknown (stated at intake 2026-09-09, not a source read)` | 2026-09-09 | `stale` against `max_age=24h` until the first governed export read | 24h |
| `reopen_rate_pct` | not yet read | `observed_at=none` | 2026-09-09 | `stale` against `max_age=24h`; DKR-2 reads it | 24h |
| `assignment_median_hours` | not yet read | `observed_at=none` | 2026-09-09 | `stale` against `max_age=24h` | 24h |
| `aged_unanswered_count` | not yet read | `observed_at=none` | 2026-09-09 | `stale` against `max_age=2h` | 2h |

Example row shape once reads exist: `observed_at=2026-09-15T08:00 -> status=fresh against max_age=24h`.

### 8.3 Stale-data policy

No committing move is dispatched on a stale objective or anti-goal read. Dry-runs and DKR probes may run. The human may waive a stale state explicitly; the waiver is recorded in `flags.jsonl` with the reason.

### 8.4 The ritual clock (every turn)

1. Start of turn: freshness check on every objective, CKR, and anti-goal metric.
2. Pre-dispatch: admissibility screen or dry-run for each candidate move.
3. Post-move: direct read of the real metrics from the helpdesk source.
4. End of turn: write `current_round`, open flags, last metric read, `next_check_at` to the records; regenerate `status.json`.
5. Idle heartbeat when no worker finishes: read worker progress files, re-check freshness, write the check-in.

### 8.5 Steering check-in value record

Each check-in records: inbound signal consumed, decision delta (continue / spawn discovery / hold / promote / veto / pause / escalate), affected CKR / PKR / DKR or allocation, expected or direct effect on the objective, the anti-goal, uncertainty, or waste, and a freshness or evidence reference (hash or ledger line). Each check-in also appends a ledger metric read: `steering_value_score >= 0.75` per check-in and `valuable_steering_decision_count >= 1` per round, with the anti-goal `no_value_checkin_count == 0`. Example: inbound = DKR-1 checkpoint accepted; delta = promote CKR-1, fund PKR-1 in dry-run mode; effect = expected `assignment_median_hours` from `3.2 hours` to `<= 1 hour`; evidence = `checkpoints/dkr-1.v1.json` hash.

---

## 9. Where the loop is: funnel width

Funnel width is the share of discovery in recent loops. Round 0 is all discovery (DKR-1 through DKR-4 queued, no PKR promoted): wide, far from goal. As checkpoints get accepted and PKRs take over, the funnel narrows. Narrowing counts as progress only while `frt_median_hours` moves with it; the `pointless` flag protects that coupling.

---

## 10. OKRA Learning Memory

- **Previous-run inputs scanned**: no `.okra/runs/` from a previous support-FRT loop was available when this artifact was drafted; `candidate-anti-goals.v1.json` starts empty and is seeded with the candidates below. All entries are automatic candidates; none is ratified for this run yet.
- **Traps (candidate, from domain reasoning, not yet evidenced)**: auto-ack gaming; solve-on-first-reply; median improved by dropping hard tickets to a side queue.
- **Avoidances / vetoes that should work**: excluding auto-acks in the metric definition; dry-run routing on historical tickets before shipping; one category at a time.
- **Misconceptions to watch**: "assignment time equals response time" (it is only part of it); "a finished macro set equals faster responses" (only the direct read says so).
- **Optimization candidates**: run DKR-2 and DKR-1 in parallel since they read the same export; reuse the FRT split query across rounds by content hash.
- **Reusable candidate anti-goals with metrics**: `auto_ack_counted_as_first_response_count == 0`, `macro_reopen_rate_pct <= 8%`, `weekly_overtime_hours_per_agent <= 2`, `pii_export_count == 0`.
- **Evidence / hashes**: none yet; each entry gets a source ref and hash when a DKR checkpoint or metric read backs it.
- **Confidence**: low until run-local evidence exists.
- **Context fit**: support queues with a helpdesk that logs creation, assignment, first reply, solve, and reopen timestamps.
- **Ratification status**: all candidate; `unratified_memory_promotion_count == 0`.
- **Terminalization / continuation packet**: not applicable yet; at end of run the terminal state, objective and anti-goal metric refs, unresolved flags, accepted checkpoints, retained trace manifest, consolidation output, continuation packet, and second-opinion evidence are recorded before any learning is reused.
- **Trace refs and review-set refs**: to be filled from `checkpoints/`, `ledger.jsonl`, and review artifacts tied to prompt and source hashes.
- **No-regression / no-single-LLM-truth evidence**: any learned guardrail is accepted only with deterministic evidence, store records, hashes, human ratification, or at least two independent review artifacts; `single_llm_truth_acceptance_count == 0`, `eval_regression_count == 0`.

---

## 11. The human-only line

The human owns the objective (`9 hours` -> `4 hours`), the anti-goal definitions and thresholds (`8%` reopen wall and any ratified secondary walls), the metric contracts, the action envelope, and the call that this goal is wrong. The loop is best-effort against the goal it was given: when effort goes in and `frt_median_hours` stays flat, it reports the gap with the evidence (DKR budget spent, tree built, CKR movement, flat metric) and the human decides whether to re-aim.

Open questions for ratification (with recommendations):

1. Calendar hours or business hours for FRT? Recommendation: calendar hours, because the source baseline `9 hours` is most likely calendar-based; confirm with the support ops lead.
2. Reopen window of 14 days? Recommendation: 14 days; shorten to 7 only if the helpdesk already reports it that way.
3. Spend cap for the secondary anti-goal? Recommendation: `0` new recurring spend without approval.
4. Hold rule of 2 consecutive weekly reads at `<= 4 hours`? Recommendation: keep it; one good week can be noise.

---

## 12. The four things that must hold

- The objective (`frt_median_hours`, `4 hours`) and every anti-goal (`reopen_rate_pct`, `8%`, plus ratified secondaries) each have a metric with a number.
- Progress is the direct metric read from the helpdesk source, never a roll-up of finished PKRs.
- The anti-goal is checked at all three points: before the move (admissibility or dry-run), after it (direct read), and paired with the goal (both must hold).
- The frame belongs to the human; the loop raises evidence and never changes the goal itself.
