# Goal Loop

Running a goal, initiative, or metric-driven change as a self-correcting loop
bounded by a measured **anti-goal**. It answers, with a metric instead of a task
list: what number proves success, what must not be sacrificed to reach it, and
when to escalate to a human.

The harness records and gates; the agent orchestrates. The loop engine is
agent-side (the `reverse-tornado-okr` skill / Okra), so no `harness-cli`
subcommand runs it. The harness's part is the **inbound tool registry** (see
`docs/TOOL_REGISTRY.md`), the classification that seeds the loop's frame, and the
durable evidence — stories, verify commands, decisions, traces — that the loop
reads as proof.

Goal loop is a consumer of the inbound tool registry, not a dependency of it. It
owns no activation switch of its own: it looks up the `goal-loop-orchestration`
capability, reads the scanned presence of whatever provider is registered, and
chooses a posture from that.

## When It Runs

Goal loop is capability-gated and lane-gated. Machinery stays proportional to
stakes; most work never needs it.

| Lane / input type | Goal loop |
| --- | --- |
| Tiny | Skip. Task-at-a-time intake is enough. |
| Normal (single story) | Skip by default. Use only if the work is explicitly metric-driven and the human asks to run it as a loop. |
| High-risk | Optional. Use when the change targets a number (latency, coverage, error rate) and has real side effects. |
| `new_initiative` | Preferred. An initiative that spans multiple stories toward one outcome is the natural goal loop. |

If the loop is not active or not warranted, intake proceeds normally: a flat
story list is the correct output for task-at-a-time work.

## Dependency

One provider serves the `goal-loop-orchestration` capability. It is an inbound
tool, not compiled into the harness, and may be absent on any machine.

| Provider | Capability role | Kind | Scan target | Agent runtime |
| --- | --- | --- | --- | --- |
| reverse-tornado-okr (Okra) | Frame the goal + anti-goal, run the discovery→steer loop, raise flags | `skill` | `.claude/skills/reverse-tornado-okr` | Claude Code / Codex skill |

Registration is described once, in `docs/TOOL_REGISTRY.md` (the per-install
seed); do not duplicate the command here. `tool register` appends a
`tool.register` event to the tracked log, so one teammate's registration
travels with `git pull`; only the scan result (`status`, `checked_at`) is
machine-local, so run `tool check` on each machine. A non-Claude agent that cannot run the skill
treats it as absent and skips the loop.

## Activation And Skip Rule

There is no goal-loop-specific switch. Activation is read from the registry:

```bash
scripts/bin/harness-cli query tools --capability goal-loop-orchestration
```

- No provider registered: the capability is inactive. Skip the loop; intake and
  work proceed task-at-a-time. Note `goal-loop: skipped, capability inactive` in
  the trace. Skipping an inactive capability is not drift.
- One provider registered and `present`: the capability is active. When the lane
  or input type warrants it, hand the classification to the loop engine as
  candidate-frame input.

Run `scripts/bin/harness-cli tool check` at intake start so provider presence is
a scanned fact, not a trusted declaration. A registered provider that scans as
`missing` is a failed validity gate, not a skip: set the `Weak proof` flag and
degrade to a flat story list.

## The Anti-Goal: Who Provides It

**The human does not hand-author an anti-goal per intake.** That would contradict
the harness rule that the human does not classify risk — the harness does — and
the loop's own rule that the human *owns* the frame but the agent *drafts* it.
The division is:

1. **The harness derives the candidate anti-goal categories.** The intake risk
   flags and hard gates already name what must not be sacrificed. They are the
   menu of candidate anti-goals; no new input is required to get them.
2. **The agent metricizes and tailors.** For each selected category the agent
   proposes a concrete anti-goal — metric, threshold, and type (drift gauge or
   tripwire) — and asks at most a few targeted questions to fit the project. It
   does not ask the human to invent anti-goals from a blank page.
3. **The human ratifies.** The human adds, removes, or adjusts, then the set
   freezes. Authority is the human's; drafting is the agent's.

Default mapping from risk flag to candidate anti-goal (the agent proposes, the
human ratifies):

| Risk flag / hard gate | Candidate anti-goal (metric · type) | Read method |
| --- | --- | --- |
| Auth | `auth_regression_count == 0` · tripwire | the auth story's `verify_command` |
| Authorization | `authz_regression_count == 0` · tripwire | authz story `verify` |
| Data model / data loss | `destructive_migration_count == 0` · tripwire | migration story `verify` |
| Audit/security | `dropped_audit_events == 0` · tripwire | audit story `verify` |
| External systems | `provider_error_rate <= <baseline>` · drift | integration story `verify` |
| Public contracts | `contract_break_count == 0` · tripwire | contract story `verify` |
| Existing behavior | `story verify-all stays green` · tripwire | `story verify-all` |
| Weak proof | `unproven_change_count == 0` · drift | `query matrix` proof booleans |

Tailoring questions the agent asks only when the answer is not derivable:

- What number, if any, is the objective target? (The harness supplies intent;
  the metric target usually still comes from the human or the spec.)
- For a drift-type anti-goal, what threshold counts as breaching versus warning?
- Is there an existing story `verify_command` that reads this metric, or must one
  be created?

Keep the ratified set small. One measured wall the loop actually checks beats a
long list it does not.

## The Hand-Off: Classification Becomes Frame

Before the loop runs, the agent reads the harness's understanding and hands it to
the engine as candidate-frame input:

```text
1. harness-cli intake --type new_initiative --lane <lane>   -> classification
2. read restated work-item + docs/product/* + initiative notes
     (goal, exit criteria, open decisions)
3. harness-cli tool check ; query tools                     -> equipped read-methods
4. (if impact-analysis active) run docs/IMPACT_ANALYSIS.md  -> blast radius =
                                                              anti-goal targets + re-run set
5. harness-cli query matrix --numeric                       -> baseline metric reads
6. harness-cli query decisions                              -> prior ratified constraints
7. harness-cli query friction | interventions | signals     -> candidate learning-memory
```

Frame elements the harness prepares versus the loop supplies:

| Frame element | Prepared by the harness | Supplied by the loop |
| --- | --- | --- |
| Objective | intent, exit criteria, matrix rows | the numeric metric + target |
| Anti-goals | risk-flag categories | metric + threshold + type |
| Metric contracts | tool registry + verify commands | freshness `max_age` + lag window |
| Action envelope | risk lane + hard gates | — (used as-is) |
| Ratification | `decision add` + human-confirm gates | — (used as-is) |
| Learning memory | friction / interventions / signals | trap/avoidance extraction |
| Decomposition | candidate stories (PKR), matrix (CKR) | the DKR discovery layer |

## Results Loop Back

The loop is not a black box; it writes its footprint into the durable layer so
the record stays complete:

```text
loop run
  each PKR / task        -> harness-cli story add (with --verify)
  each metric read       -> harness-cli story verify ; query matrix
  each flag raised       -> harness-cli intervention add
  frame ratification     -> harness-cli decision add
  DKR learning checkpoint -> harness-cli story signal add
  run terminalization    -> harness-cli backlog (predicted -> outcome), trace linked to run
```

The harness remains the durable, queryable system of record. The loop runs inside
an initiative and reports through it.

## Gated Decisions

Like impact analysis, the loop exists to change decisions, not to be context:

1. The ratified anti-goal set becomes the pre-dispatch admissibility screen: a
   move that would breach a wall is vetoed before any story is worked.
2. The objective and anti-goal metrics are read from source after each move
   (`story verify`, `query matrix`), never inferred from completed tasks.
3. Escalation flags (`cannot`, `breaking`, `pointless`, `authority_drift`) are
   recorded as interventions and stop or pause the affected branch.

Record the frame summary, the ratified anti-goal set, and any raised flag in the
trace and durable records. If the loop was skipped where an initiative warranted
it, say so in the trace rather than implying it ran.
