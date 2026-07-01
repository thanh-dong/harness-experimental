# Combining `repository-harness` with `reverse-tornado-okr` (Okra)

> A design note for combining this repository-level harness with the Okra
> `reverse-tornado-okr` skill. Every claim here is grounded in the source of both
> projects; file:line evidence is collected in the [Evidence appendix](#evidence-appendix).
>
> **Status:** Phases 1–2 implemented. Phase 1 (additive docs): `docs/GOAL_LOOP.md`,
> the `goal-loop-orchestration` entry in `docs/TOOL_REGISTRY.md`, the Goal Loop
> section in `docs/FEATURE_INTAKE.md`, the No-Regression Gate in
> `docs/IMPROVEMENT_PROTOCOL.md`. Phase 2 (worked, verified bridge):
> `docs/demo/goal-loop/`. Phase 3 remains design.
> **Scope:** the wiring (tool registration, a capability policy doc, an intake
> routing paragraph) lands in *this* repo; Okra stays an agent-side skill.

---

## TL;DR

The two projects sit at **different layers and are near-complementary, not
competing**:

- **The harness** governs *the repository and the workflow around a task* —
  durable, installable, task-at-a-time: `intake → story → proof → trace →
  decision → propose`.
- **Okra** governs *how you drive a multi-step goal to a metric without breaching
  a guardrail* — a portable skill: objective + measured anti-goal,
  orchestrator/worker split, three-point anti-goal eval, escalation flags,
  idempotent append-only store, learning memory.

Neither does the other's job. The harness has **no goal loop, no anti-goal wall,
no "steer until the metric reaches target."** Okra is **abstract about where its
metrics and evidence actually live** and is not installable into a repo.

**The combination:** run a harness *initiative* as an **Okra loop**, using the
harness DB/CLI as Okra's deterministic evidence substrate. The harness's
classification output *is* Okra's day-one frame input; Okra's loop is the
execution engine the harness lacks.

The architecture is already native to the harness. `docs/IMPACT_ANALYSIS.md`
states the exact division of labor:

> "The harness records and gates; the agent orchestrates. The two analysis tools
> are agent-side (an MCP server and a skill), so no `harness-cli` subcommand runs
> them."

Okra is simply another agent-side skill provider, exactly like `c3` is for
`impact-analysis`.

---

## 1. What each project is

### repository-harness (this repo)

A repository-level operating harness: Markdown policy docs + a Rust CLI
(`harness-cli`, ~12.8k LOC) backed by a local SQLite DB (`harness.db`). You
install it into any repo so agents get durable project context *before* they
change code. It ships no app of its own. Spine:

```
human intent → intake (risk lane: tiny/normal/high-risk)
            → story packet (verify command + proof booleans)
            → validation (TEST_MATRIX: behavior→proof)
            → trace (scored) → decision (durable ADR)
            → backlog/propose (grow from friction)
```

Key mechanisms it provides: risk classification, a capability-based **inbound
tool registry** (`cli|binary|mcp|skill|http`), deterministic `story verify`,
`score-trace`/`score-context`, `audit` (drift + entropy), and `propose`
(deterministic improvement proposals).

### Okra / reverse-tornado-okr (agent-side skill)

A workflow skill that turns any goal into a self-correcting OKR loop bounded by a
measured **anti-goal**. Core ideas: every objective and anti-goal is a
metric-with-a-number; the anti-goal is evaluated at **three points** per loop
(admissibility before acting, direct read after, paired with the goal at the
progress read); **no cascade** (only the direct metric counts, never completed
tasks); an **authority gradient** (`human owns the frame → orchestrator steers →
workers execute and hand back`); an idempotent, hash-chained, content-addressed
store; and cross-run **learning memory**. Decomposition is exactly three units —
**DKR** (budgeted discovery probe), **CKR** (measurable contribution context, not
worker work), **PKR→task** (pure execution).

### Layer map

```
┌──────────────────────────────────────────────────────────────────┐
│ repository-harness  — the repository operating system              │
│   "what kind of work is this, how dangerous, what proof exists,    │
│    what did we decide"  (classification + durable evidence)        │
│                                                                    │
│   ┌────────────────────────────────────────────────────────────┐  │
│   │ Okra loop  — the goal-execution engine                      │  │
│   │   "how do we drive it to a number without breaching the     │  │
│   │    walls that classification named"  (metricize + steer)    │  │
│   └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 2. Why they compose (complementarity)

| Dimension | Harness | Okra | Combined |
| --- | --- | --- | --- |
| Unit of work | Task / story | Goal / initiative loop | Initiative runs as a loop of stories |
| Success signal | Proof-pass booleans | Metric hits target | Metric target proven by verify commands |
| Guardrail | Risk flags + hard gates (categorical) | Measured anti-goal (metric+threshold+type) | Harness names the wall; Okra measures it |
| Evidence | Durable SQLite records, verify, matrix | "No single-LLM truth" (demands evidence) | Harness *is* the evidence Okra demands |
| Improvement | `propose` (unverified suggestions) | Anti-goal + no-regression gate | Okra hardens the propose loop |
| Reach | Installable into any repo | Portable skill, no install story | Harness hosts Okra as a capability |
| Discovery | none | DKR probes | Okra adds the missing discovery layer |

---

## 3. The hand-off: how the harness prepares Okra's input

Okra's weakest moment is Step 1 — drafting the frame from a bare prompt, where it
must **invent** the metric, the anti-goal, and the action envelope. The harness's
entire job is to turn a bare prompt into a **classified, evidenced work item**
first. So the harness's output is, almost exactly, Okra's missing structured
input.

An Okra frame has seven required parts (`frame_version`, `frame_hash`,
`objective`, `anti_goals`, `metric_contracts`, `action_envelope`,
`human_approval`). The harness already prepares five of them:

| Okra frame element | Harness artifact that prepares it | State |
| --- | --- | --- |
| **Objective** (what success is) | Intake restated work-item + `docs/product/*` + initiative **exit criteria** + `query matrix` proof rows | PARTIAL — success is proof-pass, not always a numeric target |
| **Anti-goals** (what must not break) | **Risk flags** + **hard gates**: auth, authorization, data loss/migration, audit/security, external provider, "removing/weakening validation" + impact-analysis blast radius | READY as *categories*; Okra attaches the metric |
| **Metric contracts** (source + read method) | Tool registry (`query tools`) + story `verify_command` (a deterministic read) + TEST_MATRIX definitions | READY — the harness *is* the read-method registry |
| **Action envelope** (allowed/forbidden/approval) | Risk **lane** (bounds blast radius) + hard gates ("ask human before…") + impact-analysis multi-domain limits | READY — a near-prebuilt envelope |
| **Human ratification** | `decision add` durable ADRs + intake human-confirmation gate | READY — decisions are the ratification store |
| **Learning-memory candidates** | `query friction`, `query interventions`, `query backlog --closed`, `query signals` | READY — Okra's cross-run memory input |
| **Decomposition seed** (DKR/CKR/PKR) | Initiative candidate stories (≈PKR) + matrix rows (≈CKR context) + impact-analysis "components/features touched + proof to re-run" | PARTIAL — no discovery (DKR) layer |

### The concrete flow

Before an Okra loop starts, the agent reads the harness's understanding in this
order (every step is a real command):

```text
1. harness-cli intake --type new_initiative --lane <lane>   → classification
2. read restated work-item + docs/product/* + initiative notes
     (goal, exit criteria, open decisions)
3. harness-cli tool check ; harness-cli query tools          → equipped read-methods;
                                                               is impact-analysis active?
4. (if active) run docs/IMPACT_ANALYSIS.md                   → blast radius =
                                                               anti-goal targets + re-run set
5. harness-cli query matrix --numeric                        → baseline metric reads
6. harness-cli query decisions                               → prior ratified constraints
7. harness-cli query friction | interventions | signals      → candidate learning-memory
   └─► hand all of this to Okra as the candidate-frame raw material
```

Okra then does the two things the harness cannot: **(a) metricize** — turn each
risk-flag category into an anti-goal with a number + threshold + drift/tripwire
type, and turn the objective into a metric + target; **(b) run the
discovery→steer loop** until the objective metric reaches target.

---

## 4. Worked example

Intake of a real initiative and the Okra frame it produces.

**Harness intake (understanding):**

```bash
harness-cli intake --type new_initiative --lane high-risk \
  --summary "Cut checkout p95 latency without breaking auth or dropping audit events" \
  --flags "auth,audit/security,existing behavior,public contracts"
```

Harness output: lane `high-risk` (4 flags → high-risk per `FEATURE_INTAKE.md`
classification), affected docs located, impact-analysis run if a provider is
registered.

**Okra frame draft (metricize + loop), fed by that intake:**

- **Objective:** `checkout_p95_latency_ms ≤ 400` (target from the spec; the
  harness supplied the intent, the human supplied the number).
- **Anti-goals** (each a metricized risk flag):
  - `auth` → `auth_regression_count == 0` (tripwire) — read via the auth story's
    `verify_command`.
  - `audit/security` → `dropped_audit_events == 0` (tripwire).
  - `existing behavior` → `story verify-all stays green` (tripwire).
- **Metric contracts:** read method = `harness-cli story verify <id>` +
  `query matrix --numeric`; freshness `max_age` and lag window added by Okra.
- **Action envelope:** from the high-risk lane + hard gates — "ask human before
  changing auth boundary or removing validation."
- **Ratification:** `harness-cli decision add` records the accepted frame.
- **Decomposition:** DKR "which query/index dominates p95?" (Okra-new discovery)
  → CKR "index hit-rate ≥ 95%" (measurable context) → PKR "add composite index"
  (a harness story with a verify command).

Everything measurable resolves to a harness command; everything discovery/steer
is Okra.

---

## 5. Honest gaps (what the harness does NOT prepare)

These are where Okra earns its keep and where the wiring must add glue, not
assume:

1. **Numeric objective target** — harness success is proof-pass, not always a
   metric. The number still comes from the human/spec.
2. **Metricized anti-goals** — harness gives risk *categories*; Okra attaches
   metric + threshold + drift/tripwire type.
3. **Discovery (DKR) layer** — the harness has no "budgeted probe at an unknown"
   unit; this is Okra's net-new contribution over a flat story list.
4. **Freshness / lag windows** — the harness records `last_verified_at` /
   `last_verified_result` but has **no `max_age` or lag policy**. Okra layers the
   ritual clock on top.
5. **Loop-until-target** — the harness is task-at-a-time; it never keeps
   steering. Okra owns the loop.

Net: the harness prepares **~5 of Okra's 7 frame parts as real artifacts**; Okra
supplies the 2 it lacks (numeric metricization + the discovery/steering loop).

---

## 6. Integration proposals

Ranked by value ÷ effort. All are additive — no rewrite of either project.

### P1 — Okra as the engine for harness initiatives · HIGH confidence

Register Okra in the tool registry exactly the way `c3` registers for
`impact-analysis`, then route initiative/high-risk work into an Okra loop.

```bash
harness-cli tool register \
  --name reverse-tornado-okr --kind skill \
  --capability goal-loop-orchestration \
  --scan ".claude/skills/reverse-tornado-okr" \
  --command "skill:reverse-tornado-okr" \
  --description "Run a goal as a self-correcting OKR loop with anti-goal guardrails" \
  --responsibility "Task specification"
```

- `--kind skill` is valid; presence resolves via the `--scan` path — mirrors
  `c3`'s `--scan ".c3"`.
- `--capability goal-loop-orchestration` is a **new** kebab-case capability; the
  registry is open, so no code change is needed.
- The intake seam already exists as a pattern (the capability-gated "Impact
  Analysis" step). Add an analogous step: *when `goal-loop-orchestration` has a
  provider, run `new_initiative`/high-risk work as an Okra loop.*

**Caveat:** `--responsibility` must be one of the fixed set (`Task
specification, Context selection, Tool access, Project memory, Task state,
Observability, Failure attribution, Verification, Permissions, Entropy auditing,
Intervention recording`). None is a clean fit for a goal-loop engine; `Task
specification` is the least-bad. This taxonomy mismatch is worth noting but not
blocking.

### P2 — Bind Okra's abstract contracts to the harness store · MEDIUM-HIGH confidence

Point each Okra contract at a real harness command:

| Okra abstract contract | Harness concrete substrate |
| --- | --- |
| Objective/CKR metric read | `harness-cli story verify <id>` + `query matrix --numeric` |
| Anti-goal reading | a `story verify-all` regression command that stays green |
| Frame ratification (human-only) | `decision add` + intake hard gates ("ask human before…") |
| Flags (cannot/breaking/pointless/authority-drift) | `intervention add --type … --source …` |
| DKR learning checkpoints / misconceptions | `story signal add --type design_decision\|deviation\|tradeoff\|open_question` → mined by `propose` |
| Learning-memory terminalization | `backlog --predicted` → `--outcome` loop |

**Caveat — bridge, not merge (important):** the two durable stores use different
integrity models. The harness `harness.db` is plain SQLite (WAL, no hash chain);
Okra's `.okra/` is append-only + hash-chained + content-addressed. **Do not merge
them** — merging would weaken Okra's tamper-evidence. The correct seam: a harness
`trace`/`decision` links to an Okra `run-id`; Okra cites harness verify results
as its deterministic evidence.

### P3 — Harden the harness `propose` loop with Okra's anti-goal + no-regression gate · HIGH confidence on the gap

The harness "grows from friction," but `propose` **executes and verifies
nothing**: it emits proposals with a *text* `validation_plan` + `predicted_impact`
+ `confidence`, and `--commit` only writes backlog rows. `audit`'s entropy score
is advisory only. There is no measured wall stopping an improvement that makes
things worse.

Okra fills that gap: treat each accepted harness improvement as an Okra *move*
with

- a **measured anti-goal** (`story verify-all` stays green, `entropy_score` does
  not increase, required trace tier does not drop),
- the **three-point eval** (admissibility before committing → direct read via
  `audit`/`verify-all` after → paired read: friction dropped **and** no
  regression),
- and the backlog `predicted → actual_outcome` loop as Okra's learning-memory
  terminalization.

**Caveat:** the gap is verified; the wiring is a process/doc plus optionally one
"anti-goal check" wrapper command. No blocker found.

### P4 — Cross-pollinate the parts each does better · MEDIUM confidence

- **Harness → Okra: installability & release engineering.** Okra's durable helper
  is a bash script; the harness ships a cross-platform Rust CLI + install script +
  GitHub release workflow. If Okra ever wants to drop into arbitrary repos (not
  only as a Claude/Codex plugin), this is the model.
- **Okra → Harness: adversarial eval rigor.** The harness verifies only via
  `story verify` (runs a command) and `score-trace` (field completeness) — no
  independent/adversarial verification. Okra's sandboxed black-box eval (bwrap +
  golden-calibrated checkers + dual-model review) is the discipline the harness's
  high-risk lane would benefit from.

---

## 7. Results loop back into the harness

An Okra run is not a black box; it writes its footprint back into the harness so
the durable record stays complete:

```text
Okra loop
  ├── each PKR/task        → harness story (with verify_command)
  ├── each metric read     → story verify + query matrix (deterministic evidence)
  ├── each flag raised     → intervention add
  ├── frame ratification   → decision add (durable ADR)
  ├── DKR checkpoints      → story signal add (mined by propose)
  └── run terminalization  → backlog predicted→outcome + trace linked to run-id
```

The harness remains the durable, queryable system of record; Okra is the
execution engine that runs *inside* an initiative and reports through it.

---

## 8. What NOT to do

- **Do not merge the two stores.** Keep Okra's hash-chained `.okra/` and the
  harness `harness.db` separate; bridge by reference (run-id ↔ trace/decision).
- **Do not push project-specific OKR content into the Okra skill repo.** Okra is
  a portable skill; per-project frames live in the target repo's `.okra/` + the
  harness DB.
- **Do not treat `propose` output or an Okra self-report as done.** Both projects
  agree: acceptance needs deterministic evidence (verify exit code, matrix,
  changed-path/hash, or independent review).
- **Do not make `goal-loop-orchestration` a hard dependency.** Follow the harness
  rule: an unregistered capability is *inactive → clean skip*, never a failure.

---

## 9. Phased rollout

1. **Phase 1 (P1 + P3, additive docs) — DONE:** `docs/GOAL_LOOP.md` policy doc
   (modeled on `docs/IMPACT_ANALYSIS.md`); `goal-loop-orchestration` added to the
   capability vocabulary + registration seed in `docs/TOOL_REGISTRY.md`; Goal Loop
   section in `docs/FEATURE_INTAKE.md`; No-Regression Gate in
   `docs/IMPROVEMENT_PROTOCOL.md`. The registration command is verified against the
   CLI (`tool register` / `tool check` / `query tools` accept the new capability).
2. **Phase 2 (P2 bridge) — DONE (worked demo):** `docs/demo/goal-loop/` shows an
   Okra frame whose `metric_contracts` resolve through `story verify` /
   `verify-all` / `query matrix`, with the three-point eval mapped to real
   commands and traces/decisions carrying the Okra run-id. Verified both ways: the
   harness CLI runs the scenario end-to-end, and the demo `frame.v1.json` /
   `tree.v1.json` pass Okra's `okra-store.sh` schema gate + hash-chain `verify`.
3. **Phase 3 (P4 borrow):** adopt harness release engineering for Okra
   distribution (Direction A, not started) and/or Okra eval rigor for harness
   high-risk verification (**Direction B — DONE**). Direction B improves the
   harness standalone: `scripts/calibrate-harness.sh` (black-box golden
   calibration of `audit` + `score-trace`, wired into the CI verify job and
   documented in `docs/HARNESS_AUDIT.md`), and an independent-verification gate on
   the high-risk lane (`docs/FEATURE_INTAKE.md`) and behavior-changing proposals
   (`docs/IMPROVEMENT_PROTOCOL.md`).

---

## Evidence appendix

Grounding for the claims above (paths relative to each repo).

**repository-harness (this repo):**

- Division of labor "harness records and gates; the agent orchestrates" —
  `docs/IMPACT_ANALYSIS.md:6-8`.
- `impact-analysis` is a recommended open capability, gated on a registered
  provider, inactive when none — `docs/TOOL_REGISTRY.md:123-132`,
  `docs/FEATURE_INTAKE.md:149-163`; base-mechanism note in
  `docs/stories/US-027-inbound-tool-registry.md:16`.
- Tool kinds `cli|binary|mcp|skill|http`; skill presence via `scan_target` —
  `docs/TOOL_REGISTRY.md:32-92`; `crates/harness-cli/src/infrastructure.rs:1747-1759`;
  skill-provider test at `:2496-2508`.
- `--responsibility` fixed list — `crates/harness-cli/src/domain.rs:92-104`.
- Intake input types incl. `new_initiative`; risk lanes; hard gates; classification
  thresholds — `scripts/schema/001-init.sql:26-33`, `docs/FEATURE_INTAKE.md:34-46,166-205`.
- Initiative notes shape (goal, exit criteria, open decisions) — `docs/HARNESS.md:187-190`.
- `story verify` deterministic pass/fail + exit 0/1 — `crates/harness-cli/src/interface.rs:536-545`,
  `docs/HARNESS.md:247-269`.
- `query matrix --numeric` mirrors CLI input — `crates/harness-cli/src/interface.rs:1038-1060`.
- `story signal` types design_decision/deviation/tradeoff/open_question, mined by
  `propose` — `crates/harness-cli/src/interface.rs:131-156`, `docs/FEATURE_INTAKE.md:128-133`.
- `propose` executes nothing; `--commit` only writes backlog —
  `crates/harness-cli/src/application.rs:312-314`, `crates/harness-cli/src/interface.rs:897-926`.
- Trace quality tiers; `audit` entropy score (advisory) —
  `crates/harness-cli/src/domain.rs:676-691`, `crates/harness-cli/src/interface.rs:869-887`.
- SQLite/WAL, no hash chain — `scripts/schema/001-init.sql:7`.

**Okra / reverse-tornado-okr:**

- Frame draft, candidate objective+anti-goal, action envelope, ratification —
  `skills/reverse-tornado-okr/SKILL.md` Step 1.
- Required frame/tree keys — `skills/reverse-tornado-okr/references/integrity-store.md`.
- Three-point anti-goal eval — `SKILL.md` Step 5.
- Append-only hash-chained, content-addressed store; `verify` walks the chain —
  `skills/reverse-tornado-okr/scripts/okra-store.sh`.
- Metric freshness contract (source/owner/read method/max_age/lag) —
  `references/operating-loop.md`.
- Learning memory (terminalize/consolidate/continuation) —
  `references/learning-memory.md`.

**Not independently re-verified here (confirm before implementing):** the exact
wording of the intake-routing rule to add in P1, and the precise trace-tier
thresholds referenced in P2, should be read from `docs/FEATURE_INTAKE.md` and
`docs/TRACE_SPEC.md` at implementation time.
