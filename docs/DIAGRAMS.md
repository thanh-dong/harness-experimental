# Change Diagrams

Change diagrams are review artifacts for one change. Each one shows **what the
change does to the system** — never the whole system — in a form a human can
approve faster than prose and an agent can turn into its next step
mechanically. They are separate files in the story packet, each with its own
review status, so a human can approve one diagram without approving the whole
packet, and a stale diagram is visible as stale.

The rule in one line: **a diagram is required when a risk flag requires it,
lives in `<packet>/diagrams/D<n>-<slug>.md`, is reviewed at a named stage, and
is not done until it matches shipped code.**

Which diagrams a change needs is decided by the intake risk flags in
`docs/FEATURE_INTAKE.md`, the same way flags decide lanes and packets. The
human does not pick diagrams; the harness does.

## The Seven Kinds

| Id | Kind | Shows | Human reviews | Agent derives |
| --- | --- | --- | --- | --- |
| D1 | Blast radius | Changed files → dependents → components → stories → product docs | Is the scope what was approved? Anything touched that should not be? | Validation re-run set (`story verify` per affected story), implementation reading list, the `Existing behavior` / `Multi-domain` / `Public contracts` flags |
| D2 | Component delta | C4 level 2/3 before → after, changed nodes highlighted | Architecture direction — one of the explicit human-confirm triggers in `docs/HARNESS.md` | Where new code goes; which `docs/ARCHITECTURE.md` layer rule applies |
| D3 | Sequence | The changed flow: command → handler → repository → event log → views, with error paths | Is the behavior right? Are error paths handled? Does it match the product contract? | Integration / E2E cases (one per conditional arrow), DTO shapes for the interface contract |
| D4 | State | A status set the change introduces or alters | Illegal transitions, missing terminal states | Invariant tests, `CHECK` constraints, accepted-value lists in CLI help |
| D5 | Data-model delta | Tables and relations touched, crow's-foot | Migration safety, retention, deletion — hard-gate territory | Migration file, backfill need, rebuild determinism check |
| D6 | Story DAG | Stories of an initiative and their dependencies | Is the slicing right? What ships first? | Execution order, parallelism, which story is ready |
| D7 | Boundary | Process / provider / platform boundaries the change crosses | External-provider and cross-platform hard gates | Fixture and stub list for `validation.md`, observability points |

Anything else (option trees, intake flowcharts, trace timelines, coverage
"diagrams") is not a change diagram: it either repeats a doc that already
exists or gives the agent nothing mechanical. Do not add kinds without a
decision record.

## When Each Is Required

| Lane / flag | Required diagrams |
| --- | --- |
| Tiny | None. Wanting one means the work is not tiny — re-run the gate. |
| Normal | D1 when the `impact-analysis` capability is active (generated, no authoring cost). D3 when the story crosses more than one component. |
| High-risk (always) | D1, D2, D3. |
| `Data model` flag | D5. |
| Change introduces or alters a status set | D4. |
| `External systems` or `Cross-platform` flag | D7. |
| New spec / new initiative / goal loop | D6 in the spec-intake or initiative notes packet. |

A diagram required by this table and missing at done is an incomplete story,
exactly like a missing `implementation-notes.html`.

## File Standard

One diagram per file, Markdown, Mermaid fenced. Path:

```text
<story packet folder>/diagrams/D<n>-<slug>.md
```

For normal-lane work whose story is still a bare `.md`, create the packet
folder first (the same rule `implementation-notes.html` already applies). For a
new spec, the packet is the folder holding `spec-intake.md`.

Every file has exactly this shape. The header is a fixed key/value block so
`scripts/check-diagrams.sh` can lint it:

````markdown
# D3 Sequence — US-037 Change diagrams

Story: US-037
Kind: sequence
Source: hand
Scope: harness-cli story add --diagram → repository → event log → TEST_MATRIX.md
Status: draft
Reviewed-by: -
Reviewed-at: -

```mermaid
sequenceDiagram
  ...
```

## What to review

- One bullet per thing a reviewer must confirm.

## Derived next steps

- One bullet per mechanical consequence (test case, re-run set, migration, …).
````

Header fields:

| Field | Values | Rule |
| --- | --- | --- |
| `Story` | story id, or `spec:<slug>` for D6 on a new spec | Must exist in `query matrix` (or be the packet being created). |
| `Kind` | `blast-radius` `component-delta` `sequence` `state` `data-model` `story-dag` `boundary` | Must agree with the `D<n>` prefix of the filename. |
| `Source` | `generated:codegraph`, `generated:c3`, `hand` | `hand` on a kind that has a generator means the provider was absent or stale — set `Weak proof` on the intake row. |
| `Scope` | free text | The slice of the system drawn. A diagram with no scope is a system diagram, not a change diagram. |
| `Status` | `draft` `reviewed` `stale` | See Review Stages. |
| `Reviewed-by` | `human:<name>`, `agent:<name>`, `ci:<job>`, or `-` | Must match the `--source` of the recorded intervention. |
| `Reviewed-at` | `YYYY-MM-DD` or `-` | |
| `Coverage` | `N of M` | D1 only, mandatory: blast-radius files with story/trace history. |

Exactly one ```` ```mermaid ```` fence per file. Templates for each kind live in
`docs/templates/diagrams/`; start from them.

Diagram text is the contract; never commit a rendered image in its place.
GitHub renders Mermaid in pull-request review, and agents read the source as
structured text. Do not embed diagrams in `implementation-notes.html` — it is a
no-dependency narrative file and cannot render them; link to the diagram file
instead.

## Drawing Standard

The notation follows Mermaid (the only diagram-as-code syntax GitHub renders
natively) and uses C4 vocabulary for structure. Mermaid's dedicated `C4*`
diagram types are officially experimental, so D2 and D7 use `flowchart` with
C4 semantics — a `subgraph` is a boundary, a node is a container (level 2) or
component (level 3). `C4Component` / `C4Container` are allowed when the author
prefers them; nothing downstream depends on the choice.

Rules that apply to every kind:

- **Delta, not inventory.** Draw only the nodes the change touches plus their
  direct neighbours. A full-system picture belongs in `docs/ARCHITECTURE.md` or
  the C3 model, not in a packet.
- **Highlight the change** with this fixed class set, so every diagram in every
  packet reads the same way:

  ```text
  classDef added   fill:#e6ffed,stroke:#2da44e
  classDef changed fill:#fff8c5,stroke:#bf8700
  classDef removed fill:#ffebe9,stroke:#cf222e,stroke-dasharray:4
  classDef muted   fill:#f6f8fa,stroke:#d0d7de,color:#57606a
  ```

  Unchanged neighbours are `muted`. Kinds that do not support `classDef`
  (D3, D4, D5) mark change with `%% added` / `%% changed` / `%% removed`
  comments on the line and a `Note` where the syntax allows.
- **Name nodes by their real identifier** — file path, C3 component id, table
  name, story id, CLI subcommand — never a paraphrase. The agent joins on these
  names.
- **Twenty-five nodes maximum.** Split by scope into two files before
  exceeding it.
- **No prose in the diagram** beyond labels. Explanation goes in the two
  sections below the fence.

Per-kind notation:

| Kind | Mermaid type | Conventions |
| --- | --- | --- |
| D1 | `flowchart LR` | Five subgraphs in order: `changed`, `dependents`, `components`, `stories`, `docs`. Edges left to right only. Empty subgraph stays present and labelled `none` — an empty join must be visible, not absent. |
| D2 | `flowchart TB` (or `C4Component`) | One `subgraph` per container or layer, named as in `docs/ARCHITECTURE.md`. Show `before` and `after` as two top-level subgraphs only when a node moves; otherwise one picture with `added` / `changed` / `removed` classes. |
| D3 | `sequenceDiagram` with `autonumber` | Participants are components or named adapters, not people. One `alt` / `else` per error path; `opt` for optional branches. A `Note over` for each side effect (event appended, view regenerated, file written). |
| D4 | `stateDiagram-v2` | `[*]` for start and end. Every transition labelled with the command or event that causes it. Forbidden transitions listed under *What to review*. |
| D5 | `erDiagram` | Crow's-foot cardinality. Attributes listed with `PK` / `FK`. Only touched tables plus their foreign-key neighbours. |
| D6 | `flowchart LR` | Nodes are story ids; an edge `A --> B` means *B depends on A*. Classes `ready` / `blocked` / `done` in place of the change set. Must be acyclic. |
| D7 | `flowchart LR` (or `C4Container`) | One `subgraph` per process, provider, or platform boundary. Every crossing edge labelled with protocol and direction. External providers are `muted` unless the change alters the contract with them. |

## Generation

D1 and D2 have generators; the rest are authored.

| Kind | Generator | Command | Degraded (provider absent or stale) |
| --- | --- | --- | --- |
| D1 | codegraph + C3 | `codegraph affected --json` for files and dependents; `c3 lookup <file>` for components; the trace join in `docs/IMPACT_ANALYSIS.md` for stories and docs | `git diff --name-only <base>` for the `changed` subgraph, `dependents` labelled `unknown`, raw paths for components. `Source: hand`, `Weak proof` set. |
| D2 | C3 | `c3 graph <component> --format mermaid`, with `--unit <adr-id>` to preview the after-state from a staged change-unit | Hand-drawn from `docs/ARCHITECTURE.md` layer names. `Source: hand`, `Weak proof` set. |

Generation follows the capability gating in `docs/IMPACT_ANALYSIS.md`: check
`scripts/bin/harness-cli query tools --capability impact-analysis` first. When
the capability is inactive (nothing registered), D1 is not required on normal
lane and is hand-drawn on high-risk.

A generated diagram is re-generated, not hand-edited, after code changes.

## Review Stages

Diagrams are reviewed at the stage where their subject is decided, not all at
the end. Each is a named checkpoint where the human already has the last word.

| Stage | Diagrams | Who | What approval means |
| --- | --- | --- | --- |
| **Intake checkpoint** — before the lane, story, and scope freeze | D1, D6 | Human | "This is the scope I am approving." D1 coverage below half or an unexpected dependent re-opens the restated work item. |
| **Design review** — after the packet is written, before implementation code | D2, D5, D7 | Human (high-risk: mandatory; normal: human or a second agent) | "This direction and these data / boundary changes are acceptable." This is where `docs/HARNESS.md`'s *changing architecture direction* and the data-loss / external-provider hard gates are confirmed. |
| **Design review** — same checkpoint | D3, D4 | Human or a second agent | "This behavior is what the product contract means." The reviewer names any arrow or transition that needs a test. |
| **Done gate** — before `story update --status implemented` | every diagram in the packet | The independent check already required for high-risk (`verify`, CI, or a second reviewer) | "The diagram matches shipped code." Any mismatch flips the file to `stale` and blocks done. |

Recording a review:

1. Set `Status: reviewed`, `Reviewed-by`, `Reviewed-at` in the file.
2. Record it durably:

   ```bash
   scripts/bin/harness-cli intervention add --story US-037 --type review \
     --source human --description "D2 component-delta reviewed: direction accepted"
   ```

   One intervention per diagram, `--source` matching `Reviewed-by`. The
   intervention is the durable record; the header is the readable one. If they
   disagree, the intervention wins.

3. A reviewer who rejects a diagram records `--type correction` instead, with
   what must change, and the file stays `draft`.

A diagram goes `stale` when its subject changes after review — code in its
scope moves, a migration is rewritten, a flow gains a branch. The agent that
makes the change flips the status in the same commit, then re-generates or
re-draws and asks for re-review. `reviewed` on a diagram that no longer matches
the code is the one state this standard exists to prevent.

### Lane minimums

- **Normal**: D1 (when active) and D3 (when required) must reach `reviewed` by
  a human or a second agent before done.
- **High-risk**: D1, D2, D3 and every flag-required diagram must reach
  `reviewed` **by a human** at design review, and pass the done-gate match
  check.

## Agent Consumption

The diagrams are input to the next step, not decoration. After review, the
implementing agent reads them in this order and produces the listed outputs
before writing code:

| Read | Produce |
| --- | --- |
| D1 | Reading list for the implementation phase; the `story verify` re-run set in `validation.md` / the story `Validation` table. |
| D2 | Target file placement per layer; the `Harness Delta` or `Design Notes` section. |
| D3 | One integration or E2E case per `alt` / `opt` branch, listed in `validation.md` before implementation. |
| D4 | Invariant tests for every forbidden transition; the `CHECK` constraint or accepted-value list. |
| D5 | The migration file and, if any table is rewritten, the rebuild-hash check in the validation plan. |
| D6 | The order of story work and the single "ready" story in the handoff. |
| D7 | Fixture and stub list in `validation.md`; observability points in `design.md`. |

The done-gate match check is the reverse read: compare each diagram to the
shipped code and tests, and flip anything that does not match to `stale`.

## Mechanical Check

```bash
bash scripts/check-diagrams.sh            # lint every diagram file in docs/
bash scripts/check-diagrams.sh <path>...  # lint specific files
```

The script checks file naming, the header block, `Kind` ↔ `D<n>` agreement,
`Status` / `Source` values, the single Mermaid fence and that its first line is
the type this standard prescribes for the kind, the D1 `Coverage` line, and
that a `reviewed` file names a reviewer and date. It exits 1 on any failure.
It does not check that a diagram matches the code — that is the done-gate
review's job.

Run it as part of `story verify` for any story that carries diagrams, and
before merge alongside `story verify-all`.

`story verify <id>` and `story verify-all` also enforce the review gate
themselves: before running a story's `verify_command`, the CLI scans
`docs/**/diagrams/D<n>-*.md` for files whose `Story:` names that story, and if
any has a `Status:` other than `reviewed` the verification fails without
running the command, records `last_verified_result = fail`, and lists each
offending file with its status. A drawn diagram is therefore reviewed or
deleted — there is no third option at done.

## Drift

A `reviewed` diagram that no longer matches code is a harness signal, not just
a stale file. When it recurs, record it:

```bash
scripts/bin/harness-cli story signal add --type deviation --story US-037 \
  --summary "D3 reviewed flow diverged during implementation"
```

`harness-cli propose` mines these like any other story signal.
