# Delegated OKR Loop: Support First-Response Time

**Skill:** reverse-tornado-okr (delegated loop with storage)
**Run id:** `okra-2026-09-09-support-frt`
**Run store:** `.okra/runs/okra-2026-09-09-support-frt/`
**Frame state:** candidate -> awaiting human ratification (nothing below is frozen until the human signs it)
**Prepared:** 2026-09-09

```text
   wide guessing (day 1)
  \                        /
   \   DKR  DKR  DKR      /      <- discovery, many probes
    \    CKR   CKR       /       <- measurable contributions
     \   PKR  PKR       /        <- known work, tasks
      \    task        /
       \__ target ____/          <- median FRT = 4 hours
   |<-- wall: reopen rate <= 8% -->|
```

The funnel narrows from guesses to known work. The wall on both sides is the
anti-goal. The loop stops when the objective metric hits target, or when a flag
hands the decision back to the human.

---

## 1. Frame (candidate, for human ratification)

### 1.1 Objective (candidate)

| Field | Value |
| --- | --- |
| `objective_id` | `OBJ-FRT-MEDIAN` |
| Metric | Median first-response time (FRT) on support tickets |
| Exact definition | Median of `(first_agent_reply_at - ticket_created_at)` over all tickets created in the trailing 7 full days, business and non-business hours both counted, auto-acknowledgement emails excluded |
| Baseline | `9 hours` (source literal: "from 9 hours") |
| Target | `4 hours` (source literal: "to 4 hours") |
| Direction | Lower is better |
| Success condition | Two consecutive fresh reads at or under `4 hours` with the anti-goal held on both reads |
| Reasoning note | Median is the right proof because the user named it. It resists a few extreme outliers, but it can hide a long tail. A tail read (p90 FRT) is tracked as a watch metric, not as the objective, so the loop cannot "win" by answering only easy tickets fast. |

### 1.2 Anti-goal (candidate)

| Field | Value |
| --- | --- |
| `anti_goal_id` | `AG-REOPEN-RATE` |
| Metric | Ticket reopen rate |
| Exact definition | `reopened_tickets / resolved_tickets` over the trailing 14 full days, where reopened means the customer replied on a ticket after it was marked resolved, within 7 days of resolution |
| Wall | `8%` (source literal: "not raising the reopen rate above 8%") |
| Current reading (baseline, to be confirmed by DKR-1) | unknown; must be read from the helpdesk before any committing move |
| Type | **Tripwire** at `8%` (halts committing moves) **with a drift gauge** below it: warn at `7.0%`, warn-hard at `7.5%` |
| Why this wall | The cheapest way to cut FRT is to send fast, empty, or wrong first replies. That would make customers reply again and reopen tickets. The reopen rate is where that damage shows up. |

### 1.3 Secondary anti-goals (candidate, proposed from the library)

| `metric_id` | Metric and threshold | Type |
| --- | --- | --- |
| `AG-CSAT-FLOOR` | CSAT score on resolved tickets, trailing 14 days, must stay >= baseline minus 3 points | drift gauge |
| `AG-SPEND-CAP` | Extra spend on this loop (tooling, contractors, overtime) <= $6,000 per month | tripwire |
| `AG-PRIVACY-BOUNDARY` | Zero customer PII copied outside the helpdesk and the run store | tripwire (binary) |
| `AG-AUTHORITY-DRIFT` | Count of moves proposed outside the action envelope; any count > 0 raises the `authority drift` flag | tripwire (binary) |
| `AG-DKR-BUDGET` | Total DKR budget for the run: 40 worker-turns or 10 calendar days, whichever first | tripwire |

The human may add, remove, or change any row. Only the ratified set freezes.

### 1.4 Action envelope (candidate, for ratification)

| Class | Allowed moves | Needs approval | Forbidden |
| --- | --- | --- | --- |
| Read-only | Read helpdesk reports, ticket metadata, staffing rota, macro library, queue rules | none | Export raw ticket bodies outside the helpdesk |
| Routing and queue | Change ticket routing rules, priority weights, round-robin settings | Human approves any rule that touches more than 30% of ticket volume | Deleting queues |
| Macros and templates | Draft and edit reply templates; add triage macros | Human approves any template that goes to customers before it is enabled | Auto-sending a template to a customer with no agent review |
| Staffing | Propose shift changes; propose on-call coverage | Human approves every staffing change | Hiring, firing, changing pay |
| Automation | Propose auto-triage tags; propose auto-acknowledge with real ETA | Human approves anything customer-visible | Auto-resolve, auto-close, mass-close of tickets |
| Spend | Spend up to $500 per move on tooling trials | Human approves any single spend over $500 or cumulative over $6,000 per month | Signing annual contracts |
| Blast radius | Any change ships first to one queue or one product line (<= 25% of volume) for at least 48 hours | Human approves widening beyond 25% | Whole-queue changes in one step |
| Irreversible gates | none allowed without approval | Human gate on every irreversible action (deleting rules, closing tickets in bulk, contacting customers en masse) | - |
| Rollback | Every committing move records its rollback step before it runs | - | A move with no rollback recorded |

### 1.5 Human ratification boundary

The human owns the frame. The frame is: the objective and its target of `4 hours`,
the anti-goal definitions and the `8%` wall, the metric contracts, the action
envelope, and the call that this goal is the wrong goal. **Goal-switching is
human-only.** The orchestrator may re-rank, fund, hold, or stop work inside the
frame; it never edits the frame. Any proposal to change a threshold, widen the
envelope, or redefine a metric is an `authority drift` flag, not a move. The
orchestrator must reject any attempted frame change coming from a worker or from
memory.

### 1.6 Anti-goal coverage review

| Candidate harm | Decision | Rationale |
| --- | --- | --- |
| Faster but worse replies (customers reply again) | **Selected** as `AG-REOPEN-RATE`, the primary wall | Named by the user; measured directly where the harm shows |
| Customers less happy even without reopening | **Selected** as `AG-CSAT-FLOOR` | Reopen rate misses quiet unhappiness |
| Overtime and tooling spend runs away | **Selected** as `AG-SPEND-CAP` | Cheapest way to cut FRT is to buy it |
| Ticket bodies leak to outside tools | **Selected** as `AG-PRIVACY-BOUNDARY` | Data boundary is a hard gate |
| Loop widens its own authority | **Selected** as `AG-AUTHORITY-DRIFT` | Governance breaker |
| Discovery runs without end | **Selected** as `AG-DKR-BUDGET` | Unmeasurable work has no natural stop |
| Agent burnout | **Rejected as a measured wall** | No trustworthy metric available in 10 days; covered by the human approval gate on every staffing move; revisit at round 5 |
| p90 FRT grows while median falls | **Rejected as a wall, kept as a watch metric** | Human chose the median; a p90 wall would change the frame. Reported every round so the human can promote it. |

Non-negotiable tripwires: `AG-REOPEN-RATE` at `8%`, `AG-PRIVACY-BOUNDARY`, `AG-AUTHORITY-DRIFT`.
Owners: support lead (objective, reopen, CSAT), finance partner (spend), orchestrator (authority drift, DKR budget).
Review cadence: the human reviews the coverage table at round 1 (ratification), round 5, and at terminalization.

---

## 2. Candidate anti-goal library: `candidate-anti-goals.v1.json`

Stored at `.okra/library/candidate-anti-goals.v1.json`. Every entry is
`candidate_status: candidate` until the human ratifies it in this run. Learned
entries from earlier runs may only be reused when their `source_refs` point at a
terminalized run, their `trace_evidence` is attached, and their
`no_regression_evidence` is present.

```json
{
  "library_version": "v1",
  "entries": [
    {
      "metric_id": "AG-REOPEN-RATE",
      "threshold": "8%",
      "type": "tripwire_with_drift_gauge",
      "drift_warn": "7.0%",
      "drift_warn_hard": "7.5%",
      "candidate_status": "candidate",
      "applies_when": "the objective is a speed metric on a customer-facing reply",
      "does_not_apply_when": "the helpdesk has no resolved-state event",
      "invalidates_when": "reopen definition in the helpdesk changes",
      "recertify_by": "2026-12-09",
      "source_refs": ["user request 2026-09-09: 'without raising the reopen rate above 8%'"],
      "trace_evidence": [],
      "no_regression_evidence": null,
      "owner": "support lead"
    },
    {
      "metric_id": "AG-CSAT-FLOOR",
      "threshold": "baseline CSAT minus 3 points",
      "type": "drift_gauge",
      "candidate_status": "candidate",
      "applies_when": "CSAT survey response count >= 30 in the window",
      "invalidates_when": "survey response count < 30",
      "recertify_by": "2026-12-09",
      "source_refs": ["proposed by skill library: quality"],
      "trace_evidence": [],
      "no_regression_evidence": null,
      "owner": "support lead"
    },
    {
      "metric_id": "AG-SPEND-CAP",
      "threshold": "$6,000 per month",
      "type": "tripwire",
      "candidate_status": "candidate",
      "applies_when": "any move has a cost line",
      "recertify_by": "2026-12-09",
      "source_refs": ["proposed by skill library: budget/spend"],
      "trace_evidence": [],
      "no_regression_evidence": null,
      "owner": "finance partner"
    },
    {
      "metric_id": "AG-PRIVACY-BOUNDARY",
      "threshold": "0 records exported",
      "type": "tripwire",
      "candidate_status": "candidate",
      "applies_when": "always",
      "recertify_by": "2026-12-09",
      "source_refs": ["proposed by skill library: privacy/data boundaries"],
      "trace_evidence": [],
      "no_regression_evidence": null,
      "owner": "support lead"
    },
    {
      "metric_id": "AG-AUTHORITY-DRIFT",
      "threshold": "0 out-of-envelope proposals",
      "type": "tripwire",
      "candidate_status": "candidate",
      "applies_when": "always in delegated runs",
      "recertify_by": "2026-12-09",
      "source_refs": ["proposed by skill library: authority drift"],
      "trace_evidence": [],
      "no_regression_evidence": null,
      "owner": "orchestrator"
    },
    {
      "metric_id": "AG-DKR-BUDGET",
      "threshold": "40 worker-turns or 10 calendar days",
      "type": "tripwire",
      "candidate_status": "candidate",
      "applies_when": "always",
      "recertify_by": "2026-12-09",
      "source_refs": ["proposed by skill library: time/DKR budget"],
      "trace_evidence": [],
      "no_regression_evidence": null,
      "owner": "orchestrator"
    }
  ]
}
```

No regression rule: a library entry may not be reused with a looser threshold
than the one it had in its source run unless the human re-ratifies it. A reused
entry with no `trace_evidence` and no `no_regression_evidence` is treated as
unproven and stays candidate.

---

## 3. The three units, instantiated

### 3.1 DKRs (discovery: scoped probes for a named steering decision)

Each DKR has a budget and names the decision it unlocks and the anti-goal
uncertainty it reduces. Candidate CKRs and candidate PKRs are not promoted until
the orchestrator accepts the supporting DKR learning checkpoint.

| DKR id | Probe | Decision it unlocks | Anti-goal uncertainty it reduces | Budget |
| --- | --- | --- | --- | --- |
| `DKR-1` | Read baseline reopen rate, CSAT, p90 FRT, and FRT split by hour-of-day and weekday from the helpdesk | Whether the frame can be ratified with real baselines; whether the `8%` wall already has room | How far the current reopen rate sits from `8%` | 3 turns |
| `DKR-2` | Where do the 9 hours go? Break FRT into: time in unassigned queue, time assigned but untouched, time in wrong queue, off-hours wait | Which CKR to fund first (queue, coverage, or triage) | Whether faster assignment risks wrong-queue replies that reopen | 6 turns |
| `DKR-3` | Which ticket types have long FRT and low reopen risk (safe to speed) versus long FRT and high reopen risk (unsafe to speed)? | Admit or veto template-based fast replies per ticket type | Direct read of reopen risk per type | 6 turns |
| `DKR-4` | Off-hours coverage: what share of tickets arrive outside staffed hours, and what does a 2-person evening rota cost? | Fund or reject `CKR-COVERAGE` | Spend against `AG-SPEND-CAP` | 5 turns |
| `DKR-5` (mid-execution) | After `PKR-1` ships to the pilot queue: did routing change reopen behavior in that queue? | Widen or roll back `PKR-1` | Reopen drift in the pilot | 4 turns |

DKR-2 through DKR-5 stay `candidate` until the human ratifies the frame.

### 3.2 CKRs (measurable contributions: context and measurement, not worker work)

A CKR is not worker work. It tells the orchestrator which contribution would
matter and what direct metric proves it. It is not dispatched as work.

| CKR id | Contribution | Direct CKR metric | Discovery that makes it meaningful | Delivery path (becomes PKR only after the DKR checkpoint is accepted) |
| --- | --- | --- | --- | --- |
| `CKR-QUEUE` | Tickets reach the right agent faster | Median time from `ticket_created_at` to `first_assigned_at`, trailing 7 days; target <= 45 minutes | `DKR-2`, `DKR-3` | Routing rule changes, triage macros |
| `CKR-COVERAGE` | Fewer tickets wait overnight | Median FRT for tickets created 18:00-08:00 local; target <= 6 hours (baseline unknown until `DKR-1`) | `DKR-4` | Evening rota, on-call rotation |
| `CKR-FIRST-TOUCH` | Agents send a real first reply sooner on assigned tickets | Median time from `first_assigned_at` to `first_agent_reply_at`; target <= 90 minutes | `DKR-2`, `DKR-3` | Reply templates with real ETA, agent SLA timer |

Each CKR also reports the paired anti-goal read (reopen rate in its slice), so
a CKR cannot "win" by breaking the wall in its own corner.

### 3.3 PKRs -> tasks (progression: known work only)

A PKR is dispatched only when no DKR remains under it. Every PKR carries its
trace back to the CKR and the DKR checkpoint that justified it. PKR workers
hand back on unknown: if a task hits something it cannot see the end of, it
stops and hands up rather than researching.

| PKR id | Work | `linked_ckr` | `source_dkr_checkpoint` | `contribution_metric` | Blast radius | Rollback |
| --- | --- | --- | --- | --- | --- | --- |
| `PKR-1` | Add skill-based routing rules for the top 3 ticket categories in the pilot queue | `CKR-QUEUE` | `DKR-2-checkpoint-v1` (must be accepted first) | Median created-to-assigned time in pilot queue | Pilot queue only (<= 25% volume), 48 hours | Disable the 3 rules; rule ids stored in the move result |
| `PKR-2` | Enable acknowledgement template with a true ETA for the "safe to speed" ticket types found by `DKR-3` | `CKR-FIRST-TOUCH` | `DKR-3-checkpoint-v1` (must be accepted first) | Median assigned-to-first-reply time for those types | Those types only, pilot queue | Disable the template |
| `PKR-3` | Stand up a 2-person evening rota for 2 weeks (human approves) | `CKR-COVERAGE` | `DKR-4-checkpoint-v1` (must be accepted first) | Median FRT for 18:00-08:00 tickets | Pilot product line | End the rota at 2 weeks; no contract |

Progress signals every PKR must report: off-track, quality drift, churn, late
discovery, stale metrics, scope or authority concerns.

---

## 4. Orchestrator and workers

**The orchestrator owns objective checks, check-ins, the OKR board, and
subagent steering until the objective metric reaches target.** Workers are
disposable. Each tier hands control up when it reaches the edge of its
authority.

```text
   human (frame owner)
      ^  flags, evidence, approvals
      |
   orchestrator (read-only frame, owns board + steering)
      |  worker prompt packet
      v
   worker (DKR probe | PKR task)  -- hands back at edge of scope
      |
      v
   .okra/runs/<run-id>/workers/<worker-id>/progress.jsonl
```

Gate before any dispatch: the orchestrator holds the frame read-only, has
accepted the supporting DKR learning checkpoint for anything it is about to
promote, and sends a self-contained worker prompt packet.

**In-progress worker narrative is not evidence; only worker progress,
check-ins, metric reads, flags, or accepted checkpoints can influence the next
dispatch.**

### 4.1 Worker prompt packet (schema, with `DKR-2` filled in)

```yaml
worker_prompt_packet:
  packet_id: "wpp-DKR-2-r02"
  run_id: "okra-2026-09-09-support-frt"
  frame.objective:
    id: OBJ-FRT-MEDIAN
    metric: "median support ticket first-response time, trailing 7 days"
    baseline: "9 hours"
    target: "4 hours"
  frame.anti_goals:
    - id: AG-REOPEN-RATE
      wall: "8%"
      type: tripwire_with_drift_gauge
      drift_warn: "7.0%"
    - id: AG-SPEND-CAP
      wall: "$6,000 per month"
      type: tripwire
    - id: AG-PRIVACY-BOUNDARY
      wall: "0 records exported"
      type: tripwire
  frame.action_envelope:
    allowed: ["read helpdesk reports", "read ticket metadata", "read rota"]
    forbidden: ["export ticket bodies", "change any rule", "contact a customer", "contact a human directly"]
    spend_cap_this_packet: "$0"
    data_boundary: "helpdesk reporting API and the run store only"
  frame.human_ratification_boundary: >
    You may not propose a change to the objective, the 4 hours target, the 8%
    wall, any metric definition, or the action envelope. If you think the frame
    is wrong, write it in questions_unanswered and hand back.
  current_state:
    current_round: 2
    objective_last_read: {value: "9.1 hours", observed_at: "2026-09-09T06:00Z", fresh: true}
    anti_goal_last_read: {AG-REOPEN-RATE: {value: "6.4%", observed_at: "2026-09-09T06:00Z", fresh: true}}
    open_flags: []
    funnel_width: "wide (4 DKR : 0 PKR)"
  previous_dkr_checkpoint: "DKR-1-checkpoint-v1 (accepted 2026-09-09T07:10Z)"
  assignment: >
    Break the trailing-7-day FRT into four segments per ticket: time in
    unassigned queue, time assigned but untouched, time in a wrong queue
    (reassigned at least once), and off-hours wait. Report the median and p90
    of each segment, and the reopen rate of tickets that were reassigned versus
    not reassigned. Return candidate CKRs with a probability that each segment
    holds at least 2 hours of the 9 hours.
  budget_and_stop_rule: >
    6 worker-turns or 3 hours wall clock. Stop early when every segment has a
    median with n >= 200 tickets. Stop and hand back if the helpdesk report
    cannot give first_assigned_at.
  hand_back_rule: >
    Hand back on unknown. If you meet a question that this assignment does not
    cover, a data gap, a possible anti-goal cost, or anything that needs a rule
    change, stop and write it in questions_unanswered. Do not research past
    the scope. Do not improvise a fix.
  output_schema:
    learning_checkpoint:
      decision_target: string
      evidence_refs_or_hashes: [string]
      questions_answered: [string]
      questions_unanswered: [string]
      confidence_probability_update: {prior: number, posterior: number, basis: string}
      risk_or_anti_goal_implications: [string]
      candidate_ckrs: [{id, metric, probability}]
      next_unknowns: [string]
    progress_signals: [off_track, quality_drift, churn, late_discovery, stale_metrics, scope_or_authority_concern]
    progress_file: ".okra/runs/okra-2026-09-09-support-frt/workers/wpp-DKR-2-r02/progress.jsonl"
```

### 4.2 Worker progress reports

Workers write a file-based progress report, one JSON line per check-in, to
`.okra/runs/<run-id>/workers/<worker-id>/progress.jsonl`. The orchestrator
reads these files at every heartbeat. A worker's chat text is never the source
of a metric or a decision.

```json
{"ts":"2026-09-09T08:05Z","worker":"wpp-DKR-2-r02","turn":2,"status":"running","signal":null,"note":"segment medians for 3 of 4 computed, n=412"}
{"ts":"2026-09-09T08:31Z","worker":"wpp-DKR-2-r02","turn":4,"status":"handback","signal":"late_discovery","note":"first_assigned_at missing for 18% of tickets created by email channel"}
```

---

## 5. DKR-to-DKR handoff (worked)

Every DKR starts from the last accepted checkpoint and ends in a new one.

### 5.1 Handoff fields

| Field | Meaning |
| --- | --- |
| `previous_dkr_checkpoint` | The accepted checkpoint this probe builds on |
| `decision_target` | The one steering decision this probe will unlock |
| `evidence_refs_or_hashes` | Report ids, query hashes, file hashes |
| `questions_answered` | What the probe settled |
| `questions_unanswered` | What it could not settle; hand-back material |
| `confidence_probability_update` | Prior -> posterior for the decision target |
| `risk_or_anti_goal_implications` | What this means for the reopen wall and the other anti-goals |
| `orchestrator_decision` | `accepted`, `held`, or `rejected` |
| `next_dkr_scope` | What the next probe should look at |

### 5.2 Worked instantiation: `DKR-1` -> `DKR-2`

```yaml
dkr_handoff:
  previous_dkr_checkpoint: "DKR-1-checkpoint-v1"
  decision_target: "Ratify the frame with real baselines; decide whether the 8% wall leaves room to move fast"
  evidence_refs_or_hashes:
    - "helpdesk report frt-7d-2026-09-09 sha256:3f9a...c21e"
    - "helpdesk report reopen-14d-2026-09-09 sha256:b71d...880a"
  questions_answered:
    - "Baseline median FRT reads 9.1 hours (matches the stated 9 hours)"
    - "Baseline reopen rate reads 6.4%, so 1.6 points of room under the 8% wall"
    - "p90 FRT reads 31 hours"
  questions_unanswered:
    - "Which part of the 9 hours is queue wait versus agent wait (goes to DKR-2)"
    - "CSAT baseline has only 22 responses in 14 days; AG-CSAT-FLOOR not yet valid"
  confidence_probability_update:
    prior: 0.50   # "the wall has room for a routing change without tripping"
    posterior: 0.80
    basis: "reopen 6.4% vs 8% wall; reassigned tickets reopen at 9.8%, non-reassigned at 5.9%, so wrong-queue is the reopen driver, and better routing should lower it, not raise it"
  risk_or_anti_goal_implications:
    - "AG-REOPEN-RATE: room exists but is small (1.6 points); pilot first"
    - "AG-CSAT-FLOOR: invalid until n >= 30; report only, do not gate on it yet"
  orchestrator_decision: accepted
  next_dkr_scope: "DKR-2: segment the 9 hours into unassigned, assigned-untouched, wrong-queue, off-hours; n >= 200 per segment"
```

Candidate CKRs from `DKR-1` (`CKR-QUEUE`, `CKR-COVERAGE`, `CKR-FIRST-TOUCH`)
move from `candidate` to `accepted-candidate` on this acceptance. They still
need the human's frame ratification before any PKR under them can be dispatched.

---

## 6. Storage and idempotency (set up before the first move)

Storage exists so the run is safe to interrupt and resume. Replaying the run
never replays its side effects: a routing rule is not added twice, a spend line
is not counted twice against `AG-SPEND-CAP`.

```text
.okra/runs/okra-2026-09-09-support-frt/
  frame.json           write-once after ratification (frame_version)
  tree.json            versioned (tree_version); dkrs, ckrs, pkrs
  moves/<key>.json     per-key move result; key checked before commit
  ledger.jsonl         append-only metric reads + flag events
  workers/<id>/progress.jsonl
  checkpoints/DKR-<n>-checkpoint-v<k>.json
```

### 6.1 Frame and tree schema

```json
{
  "run_id": "okra-2026-09-09-support-frt",
  "frame_version": 0,
  "frame_status": "candidate",
  "frame": {
    "objective": {"id": "OBJ-FRT-MEDIAN", "baseline": "9 hours", "target": "4 hours"},
    "anti_goals": [{"id": "AG-REOPEN-RATE", "wall": "8%", "type": "tripwire_with_drift_gauge"}],
    "action_envelope_ref": "section 1.4",
    "ratified_by": null,
    "ratified_at": null
  },
  "tree_version": 0,
  "orchestrator": {
    "owns": ["objective checks", "check-ins", "OKR board", "subagent steering"],
    "may_not": ["edit frame", "relax a threshold", "widen envelope", "switch goal"],
    "current_round": 0,
    "next_check_at": null
  },
  "dkrs": [{"id": "DKR-1", "status": "candidate", "budget_turns": 3, "checkpoint": null}],
  "ckrs": [{"id": "CKR-QUEUE", "status": "candidate", "metric": "median created-to-assigned, trailing 7d"}],
  "pkrs": [{"id": "PKR-1", "status": "blocked_on_checkpoint", "linked_ckr": "CKR-QUEUE", "source_dkr_checkpoint": "DKR-2-checkpoint-v1", "contribution_metric": "median created-to-assigned in pilot queue"}]
}
```

`frame.json` is written once when the human ratifies (`frame_version: 1`).
Any later frame change is a new ratification by the human and a new
`frame_version`; the orchestrator cannot write it. `tree.json` bumps
`tree_version` on every promote, hold, or stop.

### 6.2 Idempotency keys for moves

| Move | Key construction | Side effect guarded |
| --- | --- | --- |
| `PKR-1` add routing rules | `sha256(run_id + "PKR-1" + rule_spec_hash + frame_version)` | Rules created once; rerun finds `moves/<key>.json` and skips |
| `PKR-2` enable template | `sha256(run_id + "PKR-2" + template_hash + ticket_type_list)` | Template enabled once |
| `PKR-3` rota | `sha256(run_id + "PKR-3" + rota_spec_hash + start_date)` | Spend line booked once against `AG-SPEND-CAP` |
| Any metric read | `sha256(metric_id + observed_at)` | Same read not appended twice to `ledger.jsonl` |

Gate before the first committing move: `frame.json` exists with
`frame_status: ratified`; `tree.json` exists; `ledger.jsonl` exists; the move's
key is computed and `moves/<key>.json` does not exist; the rollback step is
written into the move record before the move runs.

### 6.3 Resume sequence

1. Read `frame.json`; refuse to run if `frame_status != ratified`.
2. Read `tree.json`; rebuild the board from it, not from memory.
3. Replay `ledger.jsonl` to get the last metric reads and open flags.
4. For every move in `moves/`, treat `status: committed` as done; treat
   `status: started` with no `committed` as suspect: run its rollback, then
   re-admit it as a fresh move.
5. Recompute `next_check_at`; continue the heartbeat.

---

## 7. Metric freshness contracts

| Metric | Source of truth | Owner | Read method | `observed_at` | `recorded_at` | `max_age` | Lag window | Missing-data policy | Freshness now |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `OBJ-FRT-MEDIAN` (median FRT, trailing 7d) | Helpdesk reporting API | Support lead | Scheduled report `frt-7d` | 2026-09-09T06:00Z | 2026-09-09T06:02Z | 6 hours | 7 days after a move (window must fill with post-move tickets) | If the report is missing, mark `stale`, do not dispatch committing moves, alert owner | `fresh` (age 2h at 2026-09-09T08:00Z) |
| `AG-REOPEN-RATE` (trailing 14d) | Helpdesk reporting API | Support lead | Scheduled report `reopen-14d` | 2026-09-09T06:00Z | 2026-09-09T06:02Z | 6 hours | 7 days after resolution for the reopen to show, so the full effect of a move takes 14 to 21 days | Missing read = treat as `stale`, pause committing moves | `fresh` |
| `CKR-QUEUE` metric (created-to-assigned) | Helpdesk event log | Support lead | Query `q-assign-lag` | 2026-09-09T06:00Z | 2026-09-09T06:03Z | 6 hours | 24 hours | Missing = `stale`, hold PKR-1 widening | `fresh` |
| `CKR-COVERAGE` metric (off-hours FRT) | Helpdesk event log | Support lead | Query `q-offhours-frt` | not yet read | - | 6 hours | 7 days | Missing = `stale` | `stale` (never read; DKR-1 to fill) |
| `CKR-FIRST-TOUCH` metric (assigned-to-reply) | Helpdesk event log | Support lead | Query `q-touch-lag` | 2026-09-09T06:00Z | 2026-09-09T06:03Z | 6 hours | 24 hours | Missing = `stale` | `fresh` |
| `AG-CSAT-FLOOR` | Survey tool | Support lead | Export `csat-14d` | 2026-09-09T06:00Z | 2026-09-09T06:05Z | 24 hours | 14 days | n < 30 = `report-only`, not a gate | `fresh` but `report-only` (n=22) |
| `AG-SPEND-CAP` | Finance ledger + `moves/` spend lines | Finance partner | Sum of committed move spend + finance export | 2026-09-09T00:00Z | 2026-09-09T00:10Z | 24 hours | none | Missing finance export = use `moves/` sum and mark `partial` | `fresh` |
| `AG-PRIVACY-BOUNDARY` | Helpdesk audit log | Support lead | Export audit for run's service account | 2026-09-09T06:00Z | 2026-09-09T06:05Z | 6 hours | none | Missing = `stale`, pause all worker dispatch | `fresh` |

Stale-data policy: a `stale` objective or anti-goal read blocks committing
moves. Read-only DKR probes may continue. A read older than `max_age` is
`stale` even if nothing changed.

---

## 8. Operating loop

### 8.1 Heartbeat and cadence

| Field | Value |
| --- | --- |
| Heartbeat | Time-based, every 10 minutes while workers are active; every 6 hours when only waiting for measurement |
| `current_round` | 0 (frame candidate; round 1 opens on ratification) |
| `next_check_at` | set at ratification to `ratified_at + 10 minutes`; then rolled forward on each heartbeat |
| Round length | One round = one orchestrator decision cycle: read metrics -> screen flags -> admit or veto moves -> dispatch -> record |
| Human check-in | Every round that raises a flag, plus a fixed weekly summary (Mondays 09:00 local) |
| Terminalization | Two consecutive fresh objective reads at or under `4 hours` with `AG-REOPEN-RATE` at or under `8%` on both; or a human `cannot`/`pointless` decision |

### 8.2 What updates on every heartbeat

1. Read every worker `progress.jsonl` since the last heartbeat.
2. Read each metric against its `max_age`; write `fresh`/`stale` to `ledger.jsonl`.
3. Run the three anti-goal eval points for any move in flight (section 9).
4. Raise, advance, or close flags (section 10).
5. Update `tree.json` (`tree_version` + 1 on any change) and the OKR board.
6. Record `current_round`, open flags, last metric read, `next_check_at`.
7. Compute funnel width (DKR turns : PKR turns in the last 3 rounds).

### 8.3 Lag handling

A finished PKR with a flat objective read inside its lag window is marked
`waiting_for_measurement` with a scheduled read. Only after the lag window
closes and the fresh read is still flat does the branch raise `pointless`.

---

## 9. The three eval points, instantiated for this goal

| Point | When | Check for this goal | Example |
| --- | --- | --- | --- |
| 1. Admissibility | Before a worker is dispatched | Project the reopen cost of the move. Any move whose projected reopen rate exceeds `7.5%` (hard drift warn) is vetoed; any move with unknowable reopen cost runs first in dry-run mode that returns a projected reopen rate without committing | Move "auto-reply with a canned 'we are looking into it' and mark ticket pending on all tickets" -> projected reopen rate `11%` (customers reply again) -> **VETOED**, off the menu. Move "skill-based routing in pilot queue" -> projected reopen `6.0%` (wrong-queue reopens fall) -> admitted. |
| 2. Direct read | After the move ships | Read `AG-REOPEN-RATE` from the helpdesk report for the affected slice, never from the worker's note | `PKR-1` shipped to pilot queue -> pilot reopen reads `6.1%` after 7 days -> in band, below `7.0%` warn. |
| 3. Paired with the goal | At every progress read | Success is two-sided: median FRT down AND reopen rate at or under `8%`. A round where FRT falls but reopen crosses the wall is a failed round | Read: median FRT `5.2 hours` (down from `9 hours`) but reopen `8.4%` -> **not a win** -> `breaking` flag opens, committing moves pause. |

No cascade: progress is the direct metric read from the source. Three PKRs done
and a median FRT still at `9 hours` is not progress; it is a sign the breakdown
was wrong. The anti-goal is measured where it manifests (the helpdesk reopen
report), never rolled up from "the worker said it stayed safe".

---

## 10. Flags and lifecycle

### 10.1 The four flags

| Flag | Fires when (this goal) | Default effect |
| --- | --- | --- |
| `cannot` | `AG-DKR-BUDGET` exhausted (40 turns or 10 days) or two rounds of discovery return empty | Stops the affected branch; hands evidence to the human |
| `breaking` | `AG-REOPEN-RATE` reads above `7.5%` (drift hard-warn) or above `8%` (tripwire); or any other anti-goal tripwire fires | Pauses all committing moves; read-only probes may continue |
| `pointless` | A PKR finished, its `contribution_metric` moved, the lag window closed, and median FRT did not move; or the funnel narrowed for 3 rounds with median FRT flat | Stops the affected branch; asks the human to re-aim |
| `authority drift` | Any worker or orchestrator proposal to change the `4 hours` target, the `8%` wall, a metric definition, or the action envelope; any attempt to contact a human directly or bypass an approval gate; any spend over cap | Stops the proposed move; goes straight to the human |

All four run at once every round. Dropping any one lets a class of silent
failure through.

### 10.2 Lifecycle

| Status | Meaning | Who moves it | Blocking? |
| --- | --- | --- | --- |
| `open` | Raised by the orchestrator; recorded in `ledger.jsonl` | Orchestrator | `breaking` pauses committing moves; `cannot` and `pointless` block their branch; `authority drift` blocks the move |
| `acknowledged` | The owner has seen it | Owner (support lead for metric flags; human frame owner for authority drift) | Still blocking |
| `resolved` | A resolution is recorded (rollback done, branch re-aimed, spend reversed) | Owner records; orchestrator verifies with a fresh read | Unblocks only inside the recorded resolution |
| `waived` | The human decides the flag does not apply this time, with a reason | Human only | Unblocks; the waiver is logged with the reason and expires at the next round |

The orchestrator may resume only inside the recorded resolution. It may not
resolve or waive an `authority drift` flag itself.

---

## 11. Learning memory and governance counters

Memory from this run may improve later runs only after this run is
terminalized and the human ratifies what is kept. Every heartbeat asserts:

- `unratified_memory_promotion_count == 0`
- `single_llm_truth_acceptance_count == 0`
- `eval_regression_count == 0`

If any counter is above zero the orchestrator opens an `authority drift` flag.
"Single LLM truth" means a metric or decision taken from a worker's or the
orchestrator's own text with no deterministic read behind it. Every accepted
checkpoint must carry `evidence_refs_or_hashes`. Every learned anti-goal reused
from `candidate-anti-goals.v1.json` must carry `trace_evidence` and
`no_regression_evidence` from a terminalized source run.

---

## 12. Round 0 board (current state)

| Item | Status |
| --- | --- |
| Frame | candidate; `frame_version 0`; awaiting human ratification of objective (`9 hours` -> `4 hours`), anti-goal `8%`, secondary anti-goals, action envelope |
| Objective last read | `9.1 hours`, `observed_at` 2026-09-09T06:00Z, `fresh`, `max_age` 6 hours |
| `AG-REOPEN-RATE` last read | `6.4%`, `observed_at` 2026-09-09T06:00Z, `fresh`, `max_age` 6 hours |
| Funnel width | wide (all discovery, no execution) |
| Open flags | none |
| `next_check_at` | ratification time + 10 minutes |
| Next move on ratification | dispatch `DKR-1` if not yet accepted; otherwise `DKR-2` with packet `wpp-DKR-2-r02` |

### Questions for the human before ratification

1. Confirm the reopen definition (customer reply within 7 days of resolution). If your helpdesk defines it differently, the `8%` wall must use your definition.
2. Confirm the pilot queue or product line for the first 48-hour blast-radius test.
3. Confirm the `$6,000 per month` spend cap, or set your own.
4. Decide whether p90 FRT should become a second anti-goal or stay a watch metric.

The human owns the frame. Goal-switching is human-only. The loop raises
evidence and never changes the goal itself.
