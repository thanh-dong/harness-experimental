# Feature Intake

Every implementation prompt enters the intake gate before code changes. A new
project spec also enters through this gate before it becomes product docs,
stories, or implementation work.

The human does not need to classify risk. The harness does.

The gate wording ships in two flavors with one owner (this repo): the **bash
flavor** in this document and `docs/HARNESS.md` references `harness-cli` shell
commands for CLI consumers, and the **MCP flavor** in `docs/templates/mcp/`
mirrors the same obligations against typed MCP tool names (`harness_intake`,
`harness_story_add`, …) for tool-hosting consumers such as Shuttle. Only the
invocation surface differs; gate semantics are identical.

## Intake Flow

```text
User prompt
    |
    v
Classify input type
    |
    v
Restate as work item
    |
    v
Run unknowns check
    |
    v
Find affected product docs and stories
    |
    v
Run impact analysis when the
impact-analysis capability is active
    |
    v
Run risk checklist
    |
    v
Choose lane: tiny, normal, or high-risk
    |
    v
Select required change diagrams
from the lane and flags
```

## Input Types

Use the input type to decide where the work should land before choosing the risk
lane.

| Type | Use when | Typical artifact |
| --- | --- | --- |
| New spec | Turning a user-provided project spec into harness-ready docs | Product docs, candidate epics, decisions |
| Spec slice | Implementing selected behavior from an accepted spec | Story packet |
| Change request | Changing, fixing, or refining accepted behavior | Story packet or direct patch |
| New initiative | Adding a larger product area that needs multiple stories | Initiative notes plus story packets |
| Maintenance request | Changing technical, operational, or dependency behavior | Story packet, validation report, or decision |
| Harness improvement | Improving how humans and agents collaborate | Direct docs update or `scripts/bin/harness-cli backlog add` |
| Spike / prototype | Exploring an approach, UX, or feasibility before committing to behavior | Throwaway prototype under a story packet `prototypes/` folder |

Do not create or extend a monolithic spec by default after intake. Use product
docs, stories, decisions, and initiative notes as the living surface.

Spikes run with tiny-lane mechanics: record the intake row, keep artifacts under
the related story packet's `prototypes/` folder (or `docs/stories/spikes/` when
no story exists yet), and skip product-doc and proof requirements — a spike is
not accepted behavior. When the spike ends, promote its findings into a story or
delete the artifacts; a spike left in place is drift.

## Unknowns Check

Before choosing a lane, list the top unknowns in the restated work item:
ambiguous intent, missing constraints, and unknown territory (unfamiliar code or
domain). If any answer would change the lane, the architecture, or a public
contract, interview the human — one question at a time, prioritizing questions
whose answer changes the architecture — before proceeding. Record the answers in
the intake `--notes` and the story packet so they become durable known-knowns.
If no unknown clears that bar, proceed without questions; the check is a gate on
ambiguity, not a mandatory interview.

## Lanes

### Tiny

Use for low-risk docs, copy, names, or narrow edits.

Also use for initial project setup when the work is limited to installing
declared dependencies, wiring a server entrypoint, adding a health/smoke
endpoint, or opening a local development database connection without creating
domain schema, CRUD behavior, auth, authorization, provider integration, or
data migration. A health endpoint in a new benchmark or scaffolded project is
smoke proof, not a public contract escalation by itself.

Requirements:

- Record the intake row before implementation; tiny work skips story packet
  overhead, not durable task classification.
- Patch directly.
- Keep affected docs current.
- Run available quick checks.
- Update the harness only if friction was found.

### Normal

Use for story-sized behavior with bounded blast radius.

Requirements:

- Create or update one story file from `docs/templates/story.md`.
- Link relevant product docs.
- Add or update validation expectations.
- Implement the smallest vertical slice when implementation exists.
- Record or update proof status with `scripts/bin/harness-cli story add` and
  `scripts/bin/harness-cli story update`.
- Keep a running `implementation-notes.html` in the story packet folder (see
  [Implementation Notes](#implementation-notes)).
- Add the change diagrams the lane and flags require under
  `<packet>/diagrams/` and get them reviewed at their stage (see
  [Change Diagrams](#change-diagrams)).

### High-Risk

Use when the work can affect security, data, scope, contracts, or multiple
roles/platforms.

Requirements:

- Create a story folder using `docs/templates/high-risk-story/`.
- Fill in `execplan.md`, `overview.md`, `design.md`, and `validation.md`.
- Ask for human confirmation before implementation if direction is ambiguous.
- Record a durable decision when behavior, architecture, authorization, data
  ownership, API shape, or validation requirements change meaningfully. Use a
  `docs/decisions/NNNN-*.md` file from `docs/templates/decision.md`, then add
  or refresh the durable row with `scripts/bin/harness-cli decision add`.
  Decision text in a trace is not a durable decision record.
- Keep a running `implementation-notes.html` alongside `execplan.md` /
  `overview.md` / `design.md` / `validation.md` (see
  [Implementation Notes](#implementation-notes)).
- Add D1, D2, D3 and every flag-required change diagram under
  `<packet>/diagrams/`; each must be reviewed by a human at design review
  before implementation code (see [Change Diagrams](#change-diagrams)).
- Do not accept high-risk work on the implementing agent's word alone. Require one
  independent check before done: a deterministic proof (`story verify` /
  `verify-all` / a passing test), or a second reviewer — human or a different
  agent. Record it with `scripts/bin/harness-cli intervention add --type review
  --source <human|agent|ci>`. A high-risk story with a `verify_command` that has
  never passed is not done.

## Implementation Notes

Normal and high-risk work keeps a running `implementation-notes.html` **inside
the story packet folder** — never at the repo root. The story packet is the
*contract*; this file is the working narrative that explains *how* the
implementation got there. Tiny-lane work has none by definition: wanting one
means the work is not tiny, so re-run the gate and re-lane.

- **Normal lane** — next to the single story `.md`. If the story is currently a
  bare file, create its folder and move the `.md` inside.
- **High-risk lane** — alongside the `docs/templates/high-risk-story/` set.

Start from `docs/templates/implementation-notes.html`. Keep it self-contained
(inline `<style>`, no build, no dependencies) so it opens directly in a browser,
and update it **as you go**, capturing:

- **Design decisions** — choices made where the spec was ambiguous, and why.
- **Deviations** — where you intentionally departed from the plan, and why.
  Default policy when implementation hits an unplanned unknown: pick the
  conservative option, log it here, and keep going. Pause for the human only
  when the deviation touches a hard gate or a high-risk stop condition.
- **Tradeoffs** — alternatives considered and why you picked what you did.
- **Open questions** — anything to confirm, each with your recommendation.

Reference its full path in the final response. These four categories are also
the durable self-improvement signal: when a category recurs across stories it is
a harness gap, so record the recurring ones with
`scripts/bin/harness-cli story signal add` (see `docs/IMPROVEMENT_PROTOCOL.md`)
so `harness-cli propose` can mine them. The HTML is the human-readable narrative;
the recorded signal is the machine-mineable subset.

**Done gate (normal and high-risk).** A story is not done until its
`implementation-notes.html` exists in the story packet folder and reflects the
work as shipped. Before reporting progress or done, check each claim against a
tool result from this session; report only work you can point to evidence for,
and say plainly what was skipped or failed. Before declaring done:

- The file exists in the packet folder (never the repo root) and every section
  matches reality — fix any section that no longer does before continuing.
- Each recurring design decision, deviation, tradeoff, or open question is
  recorded with `scripts/bin/harness-cli story signal add`.
- The final response cites the file's full path.

If the note is missing or stale when you are about to declare done, stop,
backfill it, and say in one sentence that the gate was missed. A missing
implementation note is an incomplete story, not a documentation nicety.

## Change Diagrams

Normal and high-risk work carries **change diagrams**: separate Mermaid files
under `<packet>/diagrams/D<n>-<slug>.md`, one per diagram, each with its own
review status. They show what the change does to the system — never the whole
system — so a human can approve scope, direction, behavior, data, and
boundaries at the stage where each is decided, and the implementing agent can
derive tests, re-run sets, migrations, and file placement from them
mechanically. Full standard, notation, review stages, and lint:
`docs/DIAGRAMS.md`. Templates: `docs/templates/diagrams/`.

The risk flags select the diagrams; the human does not pick them:

| Lane / flag | Required diagrams |
| --- | --- |
| Tiny | None. Wanting one means the work is not tiny — re-run the gate. |
| Normal | D1 blast radius when `impact-analysis` is active; D3 sequence when the story crosses more than one component. |
| High-risk | D1 blast radius, D2 component delta, D3 sequence — always. |
| `Data model` | D5 data-model delta. |
| Introduces or alters a status set | D4 state. |
| `External systems` or `Cross-platform` | D7 boundary. |
| New spec, new initiative, goal loop | D6 story DAG. |

Review stages — each is a point where the human already has the last word:

| Stage | Diagrams | Reviewer |
| --- | --- | --- |
| Intake checkpoint, before lane and scope freeze | D1, D6 | Human |
| Design review, after the packet exists and before implementation code | D2, D5, D7 (direction, data, boundaries); D3, D4 (behavior) | Human on high-risk; human or second agent on normal |
| Done gate, before `story update --status implemented` | every diagram in the packet must match shipped code | The independent check (verify, CI, or second reviewer) |

A review is recorded in two places: `Status: reviewed` with `Reviewed-by` /
`Reviewed-at` in the file, and
`scripts/bin/harness-cli intervention add --story <id> --type review --source <human|agent|ci> --description "D<n> reviewed: …"`.
A diagram whose subject changes after review is flipped to `Status: stale` in
the same commit and re-reviewed. `bash scripts/check-diagrams.sh` lints every
diagram file's structure; add it to the story's `--verify` command.

**Done gate (normal and high-risk).** A story is not done until every required
diagram exists, passes `scripts/check-diagrams.sh`, is `reviewed`, and matches
the shipped code. A missing or `stale` required diagram is an incomplete story,
exactly like a missing `implementation-notes.html`. Before reporting progress
or done, check each claim against a tool result from this session; report only
work you can point to evidence for, and say plainly what was skipped or failed.

## Impact Analysis

On normal and high-risk work, when the `impact-analysis` capability has a
registered provider, run the impact analysis described in
`docs/IMPACT_ANALYSIS.md` before completing the risk checklist. Check activation
with `scripts/bin/harness-cli query tools --capability impact-analysis`, and run
`scripts/bin/harness-cli tool check` at intake start so provider presence is a
scanned fact rather than a trusted declaration. Its output feeds the
`Existing behavior`, `Multi-domain`, and `Public contracts` flags, the
validation re-run set, and the implementation reading list, and is rendered as
the D1 blast-radius diagram (`docs/DIAGRAMS.md`) for the human to approve at
the intake checkpoint. Tiny-lane work skips
it. When no provider is registered, the capability is inactive: skip the step
and note the skip in the trace. A registered provider that scans as missing,
stale, or drifted is not a skip: degrade per the Degraded Modes table in
`docs/IMPACT_ANALYSIS.md` and set the `Weak proof` flag. If the analysis
escalates the lane, re-classify before proceeding.

## Goal Loop

Some intake is not one task but a goal: an initiative or a metric-driven change
that spans multiple stories toward one outcome. When the `goal-loop-orchestration`
capability has a registered provider, `new_initiative` and metric-driven
high-risk work can run as a self-correcting OKR loop with a measured anti-goal,
described in `docs/GOAL_LOOP.md`. Check activation with
`scripts/bin/harness-cli query tools --capability goal-loop-orchestration`. When
no provider is registered, the capability is inactive: skip it and produce a flat
story list. Tiny and normal task-at-a-time work skips it regardless.

The loop does not ask the human to hand-author an anti-goal. The intake risk
flags below are the candidate anti-goal categories; the agent metricizes them
(metric, threshold, drift or tripwire), asks a few targeted tailoring questions,
and the human ratifies the small final set. See the anti-goal mapping table in
`docs/GOAL_LOOP.md`.

## Risk Checklist

Mark one flag for each item that applies:

| Risk flag | Applies when the work touches |
| --- | --- |
| Auth | login, logout, sessions, JWT, password, refresh token |
| Authorization | roles, permissions, tenant or company scope |
| Data model | schema, migrations, uniqueness, deletion, retention |
| Audit/security | audit logs, privacy, sensitive data, access logs |
| External systems | email, payments, cloud services, provider SDKs, queues, webhooks |
| Public contracts | API shape, response envelope, client-visible behavior |
| Cross-platform | desktop/mobile/browser split, native shell behavior, deep links |
| Existing behavior | already implemented or test-covered behavior changes |
| Weak proof | unclear or missing tests around the affected area |
| Multi-domain | more than one product domain changes at once |

## Classification

```text
0-1 flags:
  tiny or normal, based on code impact

2-3 flags:
  normal with stronger validation

4+ flags:
  high-risk

Any hard gate:
  high-risk unless the human explicitly narrows scope
```

Hard gates:

- Auth.
- Authorization.
- Data loss or migration.
- Audit/security.
- External provider behavior.
- Removing or weakening validation requirements.

## Output

At the end of intake, the agent should be able to say:

```text
Lane: normal
Reason: touches authorization, API contract, and audit behavior.
Docs: permissions, account-settings, audit-log.
Story: docs/stories/epics/E02-access-control/US-014-manager-updates-role.md.
Diagrams: D1 blast radius, D3 sequence (reviewed at design review).
Validation: unit, integration, E2E.
```
