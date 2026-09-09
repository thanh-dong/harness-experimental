---
name: reverse-tornado-okr
description: >
  A workflow for turning goals, objectives, missions, measurable improvements, or decision-oriented
  research into a self-correcting OKR loop with measured anti-goal guardrails. Use when the user is
  setting OKRs, planning toward a metric, asking how to make progress without breaking a constraint
  (budget, quality, risk, trust), or wants delegated loop execution with human control. Trigger even
  for "help me set a goal", "what should my OKRs be", "I want to grow X", or open-ended research
  that needs DKR discovery to reduce uncertainty. Produces candidate objective metrics and anti-goals
  for ratification, then the structured loop: objective, anti-goal, decomposition, eval points,
  flags, and operating cadence.
---

# The Reverse Tornado - running a goal as a self-correcting loop

This skill is a workflow. Given any goal, it sets the goal up so an LLM can drive most of the work
while a human keeps the direction. The core picture is a reverse tornado: wide guessing on day
one, narrowing loop by loop into known work, bounded the whole way by a wall it cannot cross
(the anti-goal), stopping when the metric hits target. Another useful picture: the objective is the
maze exit, anti-goals are traps, and discovery maps enough of the maze that the loop can keep moving
without blindly sprinting into danger.

Apply the steps below in order. Do not skip the anti-goal - it is what makes the rest safe.

## Step 1 - Draft the frame (metric + target, and the wall)

On receiving a goal-like, mission-like, improvement, or research request, first draft the frame.
Do not wait for the user to phrase it as an OKR. Propose a **candidate objective** with the metric
that would prove success, a target or target range when the source facts support one, and a short
reasoning note for why that metric is the right proof. If the request is research, the first
candidate objective can be decision-quality: "reduce uncertainty enough to decide X", with a metric
such as confidence, decision readiness, evidence coverage, or risk retired. Mark it candidate until
the human ratifies it.

Pin two things before any planning:

- **Objective**: a metric with a target number. Not a vibe. "Grow monthly sales to $500k", not
  "do better at sales". If the user gives a vague goal, your first job is to propose the metric
  that would prove it, state that it is a candidate, and get human ratification before any
  state-changing move.
- **Anti-goal**: the thing that must not be sacrificed, expressed with **its own metric**. "Keep
  monthly expense at or under $80k." It can be continuous (a drift gauge you watch) or binary
  (a tripwire that halts). State which.

Anti-goals can be reusable across runs. When the user has not named one, propose candidate
anti-goals from the current skill and any available past run context: budget/spend, quality, trust,
safety, privacy/data boundaries, authority drift, time/DKR budget, storage integrity, and
eval regression, stale or unproven learning-memory reuse, and single-LLM-truth acceptance. Pick the
ones that match the current goal and give each a metric. The user may add, remove, or alter
anti-goals; treat that as healthy frame negotiation, then freeze only the human-ratified set.

Why the anti-goal matters: the cheapest way to hit almost any objective is to wreck something
unmeasured. Sales rises fastest by blowing the budget. The anti-goal is the wall that stops the
loop from narrowing toward that disaster. A goal without a named anti-goal is a goal you can hit
in a way you will regret.

The frame always moves through two states: **candidate frame -> human-ratified frame**. If the frame
is not fully knowable on day one (often true - you may not yet know what to track), that is expected.
The **first discovery** surfaces candidate key results and anti-goals; the human **ratifies** them,
then they freeze. Authority here is the human's call, not the loop's invention.

For delegated or automated loops, also ratify the **action envelope**: allowed move classes,
forbidden actions, spend caps, data boundaries, blast-radius limits, irreversible-action gates,
rollback expectations, and which moves need approval. A move can be metric-safe and still outside
authority.

Do an **anti-goal coverage review** for real stakes: list the candidate harms considered, selected
guardrails, rejected guardrails with rationale, non-negotiable tripwires, owners, and review cadence.
One measured wall is required; documenting what it does not cover keeps it honest.

## Step 2 - Know the three units

Decompose work into exactly three kinds. Keep them distinct; blurring them is where these systems rot.

- **DKR - Discovery.** Unmeasurable. A *scoped probe* at one unclear slice ("which channel
  converts?"). Its aim is not generic research; it is **intentional uncertainty reduction for a
  named steering decision**. A valid DKR says which decision it will unlock or improve: whether to
  promote a CKR, fund a PKR, spawn more discovery, admit/veto a risky move, pause, or re-aim. It
  also names the risk or anti-goal uncertainty it is meant to reduce, because over-focusing on the
  objective is how the loop runs into traps. It is plural - many fire per level, some mid-execution.
  It has a **resource budget** (turns/time), because unmeasurable work has no natural stopping point.
  It returns structure (one or many CKRs) with probabilities/confidence, or returns empty - empty is
  still useful when it tells the orchestrator not to fund a path. A DKR is complete only when it
  writes a **learning checkpoint**: decision target, evidence collected, questions answered/
  unanswered, probability/confidence updates, risk or anti-goal implications, candidate CKRs, and
  the next unknowns. CKR/PKR entries stay candidate-only until the orchestrator accepts that
  checkpoint.
  In delegated artifacts, write the gate explicitly: **Candidate CKRs and candidate PKRs are not
  promoted until the orchestrator accepts the supporting DKR learning checkpoint.**
- **CKR - Contribution / Key Result.** Measurable, has its own metric. This is what counts toward
  the objective. A CKR is **context and measurement**, not a worker job. It tells the orchestrator
  which contribution would matter and what direct metric proves it; it is not dispatched as work.
  Each CKR still has a mini reverse-tornado context: what discovery would make the contribution
  meaningful, what direct CKR metric proves movement, and what delivery path becomes PKR work only
  after the uncertainty is reduced.
- **PKR -> task - Progression.** Pure breakdown, then execution. A unit becomes a **task** when no
  DKR remains under it: no discovery, no judgment, just do-and-check. PKRs report progress signals
  for steering - off-track work, quality drift, churn, late discovery, stale metrics, and scope or
  authority concerns.

## Step 2b - Two roles: orchestrator and workers

The loop runs as an **orchestrator** directing disposable **workers**. This split is not cosmetic -
it carries the authority lines. Each tier hands control *up* when it reaches the edge of its authority.
The orchestrator does not stop when a board, branch, or worker queue is complete; it keeps steering
until the objective metric reaches target, a human changes or stops the frame, or a blocking flag
needs a human.

The goal is that no tier ever acts past its own authority, and the gate before any dispatch is that
the orchestrator holds the frame read-only, has accepted the supporting DKR learning checkpoint for
anything it is about to promote, and sends a self-contained worker prompt packet to a worker that
will hand back rather than improvise when it reaches the edge of its scope.

## Step 2c - Make the run idempotent (set up storage first)

Before running any move, set up storage so the loop is **safe to interrupt and resume** - the human
can step in anytime, a worker can crash, a run can restart. Without it, re-running replays side
effects: the discount applies twice, the expense double-counts against the wall.

The goal is that replaying the run never replays its side effects, and the gate before the first
committing move is that the run store exists with a write-once frame, a tree, per-key move results,
an append-only metric and flag ledger, and a checked idempotency key for that move; the schema, key
construction, resume sequence, and a bash helper are in `references/storage-idempotency.md` and
`references/integrity-store.md`.

## Step 2d - Keep the run fresh (the ritual clock)

If the run continues across turns or time, define the update ritual before dispatching work. The
orchestrator needs a **metric freshness contract** for every objective, CKR, and anti-goal metric:
source of truth, owner, exact definition, read method, `observed_at`, `recorded_at`, `max_age`, lag
window, and missing-data policy.

The goal is that every steering decision rests on a reading recent enough to trust, and the gate
before dispatching committing work is that each objective, CKR, and anti-goal metric reads fresh
against its `max_age` and the round has recorded `current_round`, open flags, the last metric read,
and `next_check_at`; the operating-loop fields, lag handling, and flag lifecycle are in
`references/operating-loop.md`.

## Step 2e - Learn, heal, and optimize from OKRA memory

A serious OKRA loop should improve under stress. Here, **stress** means time, turn, budget, or
attention pressure while the loop must still avoid anti-goal violations and either reach the
objective or produce clear evidence that it cannot. The orchestrator owns allocation under that
stress: it spends DKR budget where uncertainty blocks safer steering, holds or promotes candidate
CKRs/PKRs only after accepted learning checkpoints, enforces PKR progress signals, and vetoes moves
whose anti-goal cost is too high. It may re-rank, fund, hold, or stop work inside the ratified frame;
it does not change the frame.

The goal is that the loop gets better across runs without letting memory quietly take authority, and
the gate before any learned anti-goal, optimization, or reused memory is used is that it is backed by
deterministic evidence and human ratification and its source run was terminalized; the record shape
and no-regression gates are in `references/learning-memory.md`.

## Step 3 - The cardinal rule: no cascade

The tree of work is **scaffolding, not scoreboard**. The only score that counts is the **direct
metric** - the objective's number and each CKR's number - read fresh from the source.

A finished subtree with a flat objective metric is **not** success. If the metric's lag window is
still open, mark the branch `waiting_for_measurement` and schedule the next read. Once the lag window
has closed and fresh reads still show no movement, the flat metric is a signal the breakdown was
wrong. Never infer progress from completed tasks. Measure the world directly. The same applies to the
anti-goal: measure breakage where it manifests, never roll it up.

## Step 4 - Run the zig-zag (discovery <-> execution)

This is not waterfall (discover everything, then build everything). It is a **zig-zag** that narrows:
learn a slice -> act on it -> that action surfaces the next unknown -> discover that -> act again.
The swings shrink as guess turns into known work. That narrowing *is* progress.

Keep every bend clean: when an **execution task hits an unknown mid-run, it hands back up** -
"this is not execution anymore, this is discovery" - and the loop decides whether to fund a fresh
probe before resuming. Never let a task quietly muddle through a discovery it cannot see the end of.
You are always either executing known work or running a scoped probe - never pretending one is the
other.

## Step 5 - Evaluate the anti-goal at THREE points every loop

The anti-goal is not a single end-of-loop check. It fires three times, each doing a different job.
This is the heart of the skill - get the timing right.

1. **Admissibility - before acting.** When the orchestrator picks the next move, it screens it
   against the anti-goal *before dispatching a worker*. A move that would breach the wall never
   reaches a worker. This is the guardrail *steering* - it removes disaster moves from the menu.
   The orchestrator judges a move's anti-goal cost up front; for moves whose cost is unknowable
   without running them, it can dispatch a worker in a propose-cost (dry-run) mode that returns a
   projected anti-metric *without committing*, then admit or veto.
   *Example: move "blanket 40% discount" -> projected expense $96k -> VETOED, off the menu.*
2. **Direct read - after acting.** Read the actual anti-metric from the source, not "the task said
   it stayed safe." Drift toward the wall warns early; crossing it trips the breaker.
   *Example: ran "targeted email" -> expense reads $71k -> in band.*
3. **Paired with the goal - at the progress read.** Success is two-sided: **objective up AND
   anti-goal held.** A loop that moved the metric by breaching the wall is a failed loop that looks
   like a win - only the paired read catches it.
   *Example: sales $420k up but expense $88k failed -> not a win -> FLAG breaking.*

## Step 6 - Escalate on the flags

The loop runs around 80% on its own. It calls the human on three outcome conditions, each a distinct
failure:

- **Cannot** - discovery budget exhausted or learning flatlined. Effort in, nothing back.
- **Breaking** - an anti-metric drifted or tripped. The loop started making it worse.
- **Pointless** - work finished or a CKR metric moved, but the objective metric did not budge. This
  also guards the tornado's deepest trap: the funnel can narrow toward the **wrong tip** - converging
  beautifully on a target that will not move the goal. Narrowing without the metric moving -> re-aim.

Run all three outcome flags at once. Drop any one and a class of silent failure slips through.

For delegated loops, also raise **Authority drift** when the loop or a worker tries to change the
frame, relax a threshold, expand scope, bypass approval, contact a human directly, or act outside the
ratified action envelope. This is a governance breaker, not just an invalid move.

Flags have lifecycle. They are `open`, `acknowledged`, `resolved`, or `waived`. `breaking` pauses
committing moves by default; `cannot` and `pointless` stop the affected branch; `authority drift`
stops the proposed move and goes to the human. The orchestrator may resume only inside the recorded
resolution.

## Step 7 - Hold the human-only line

The human owns the **frame**: the objective and target, the CKR and anti-goal definitions and
thresholds, the metric contracts, the action envelope, and the call that a goal is wrong.
**Goal-switching is human-only.**

This is load-bearing. If goal switching sits inside the loop, the loop can satisfy anything by
quietly retreating to a goal it is already hitting, and every guardrail becomes theater. Keeping goal
switching with the human is what makes the anti-goal mean something. This is **best-effort**: the
loop must try against the goal it was given; when effort goes in and the metric stays flat, it
reports the gap and hands up the evidence (budget spent, tree built, contributions done, flat
metric). The human decides.

In team terms: no matter how much runs on its own, eventually someone makes the call - and the call
belongs to a person.

## A read on where you are

The **width of the funnel** - the ratio of discovery to execution in recent loops - tells position.
Wide, still guessing -> early, far from goal. Narrow, mostly known work -> close. This is a progress
signal that is not the direct metric: the metric says *if* you have arrived; the funnel width says
*how close* on the way. It is honest only while "more known" and "closer to goal" stay coupled -
which is what the pointless flag protects.

## Scale the apparatus to the goal

Match depth to the stakes. Not every goal needs the full machinery. For a light or personal goal,
the load-bearing core is just: **a metric+target objective, a measured anti-goal, and the no-cascade
habit of reading the real metric instead of counting tasks done.** Lead with that.

Bring in the heavier parts - orchestrator/worker split, idempotent storage, the formal three-point
eval, the flags - when the goal is being run as an actual automated loop, has real side effects
(spend, sends, deploys), or the user asks how to operationalize it. Offer them rather than front-load
them on someone who just wants help shaping a goal. The reverse tornado is the same shape at every
size; you do not always need to draw the whole funnel.

## Output

Deliver the structured loop: objective + target, the named anti-goal with its metric and type
(drift/tripwire), the CKR/DKR/PKR decomposition, the three eval points instantiated for *this* goal,
the flags, and the human-only frame. Use the user's real domain throughout - do not leave the example
abstract. Preserve exact metric literals from the source material in addition to any explanation; if
the source says `12 per 100`, include that exact phrase instead of only a paraphrase.

When the user wants to run the goal over time, also deliver the Operating Loop: cadence, current
round, metric freshness contracts, lag windows, `next_check_at`, stale-data policy, flag lifecycle,
and what gets updated at every turn or timed heartbeat. Include the current metric freshness
classification, keeping `observed_at`, `recorded_at`, `fresh` or `stale`, and `max_age` in the same
table row or sentence.

If the user wants a visual or shareable explainer, produce a self-contained HTML artifact. See
`references/artifact-guide.md` for how (and how to keep the artifact within its own anti-goal:
single file, no external runtime, no decoration that does not carry meaning).

For a delegated loop the artifact must also satisfy every line under `## Contract` below; run the
gate before handing it over.

## The four things that must hold

- The objective and every anti-goal each have a metric with a number.
- Progress is the direct metric read from the source, never a roll-up of finished tasks.
- The anti-goal is checked at all three points: before the move, after it, and paired with the goal.
- The frame belongs to the human; the loop raises evidence and never changes the goal itself.

## Contract

A delegated-loop artifact must satisfy the completeness contract in
`contracts/handoff-contract.v2.json`. That file is the checked source of truth for the exact keys and
sentences below; if this list and the contract ever differ, the contract wins and this file needs
fixing. Run the gate before handing an artifact over and repair anything it reports missing:

```bash
python3 .claude/skills/reverse-tornado-okr/scripts/okra-verify-artifact.py <artifact.md>
```

The nineteen requirements, one line each, with the tokens the artifact must contain:

- `worker_prompt_packet` - all of: `frame.objective`, `frame.anti_goals`, `frame.action_envelope`, `frame.human_ratification_boundary`, `current_state`, `previous_dkr_checkpoint`, `assignment`, `budget_and_stop_rule`, `hand_back_rule`, `output_schema`.
- `in_progress_rule` - the exact sentence: **"In-progress worker narrative is not evidence; only worker progress, check-ins, metric reads, flags, or accepted checkpoints can influence the next dispatch."**
- `dkr_to_dkr_fields` - all of: `previous_dkr_checkpoint`, `decision_target`, `evidence_refs_or_hashes`, `questions_answered`, `questions_unanswered`, `confidence_probability_update`, `risk_or_anti_goal_implications`, `orchestrator_decision`, `next_dkr_scope`.
- `dkr_to_dkr_worked` - a concrete worked instantiation, six groups, one token from each: "previous dkr learning checkpoint" / "prior dkr" / "previous dkr checkpoint"; "decision target" / "decide whether to"; "confidence" / "probability" / "posterior"; "prior " / "posterior" / "->" / "before and after" / "before/after"; "orchestrator decision" / `orchestrator_decision` / "checkpoint accepted" / "checkpoint held"; "accepted" / "held" / "rejected".
- `ckr_pkr_trace` - every PKR carries: `linked_ckr`, `source_dkr_checkpoint`, `contribution_metric`.
- `ckr_not_worker_work` - one of: "not worker work", "not a worker job", "not dispatched as work", "not subagent work", "measurable contribution context, not", "context and measurement, not".
- `pkr_handback` - one of: "hand back on unknown", "hands back on unknown", "hand-back on unknown", "hand back on newly discovered", "hand back when new uncertainty", "hand back instead of researching".
- `candidate_antigoal_library` - name `candidate-anti-goals.v1.json`; all of `metric_id`, `threshold`, `type`, `candidate_status`; one of `applies_when` / `does_not_apply_when` / `invalidates_when` / `recertify_by`; one of `no_regression_evidence` / "no regression"; one of `source_refs` / "source refs" / "source references"; one of "trace evidence" / `trace_evidence` / "trace refs" / `trace_manifest_ref`.
- `accumulated_governance` - all three, stated literally: `unratified_memory_promotion_count == 0`, `single_llm_truth_acceptance_count == 0`, `eval_regression_count == 0`.
- `four_flags` - all of `cannot`, `breaking`, `pointless`, plus `authority drift` or `authority_drift`.
- `flag_lifecycle` - all of `open`, `acknowledged`, `resolved`, `waived`, plus one of "owner" / "blocking" / "pauses" / "pause" / "status".
- `operating_heartbeat` - one of "heartbeat" / "cadence"; one of `next_check_at` / "next check"; one of "ten-minute" / "10-minute" / "time-based" / "every ten minutes" / "every 10 minutes".
- `worker_progress_reports` - one of `/workers/`, `progress.jsonl`, "worker progress file", "file-based progress report" (workers write under `.okra/runs/<run-id>/workers/`).
- `freshness_contract` - one of `observed_at` / "observed at"; one of `max_age` / "max age" / "stale".
- `orchestrator_ownership` - the exact sentence: **"The orchestrator owns objective checks, check-ins, the OKR board, and subagent steering until the objective metric reaches target."**
- `eval_points` - "admissibility"; "direct read" or "direct metric"; "paired".
- `no_cascade` - one of "no cascade" / "no-cascade" / "direct metric read".
- `frame_tree_schema` - all of `frame_version`, `tree_version`, `orchestrator`, `dkrs`, `ckrs`, `pkrs`, plus the phrases "objective checks" and "subagent steering".
- `human_only` - one of "human owns the frame" / "human-only" / "goal-switching is human" / "goal switching is human" / "reject any attempted frame".

Matching is case-insensitive substring over whitespace-collapsed text; multi-word tokens tolerate
line wrapping.
