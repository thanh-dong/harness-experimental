# Improvement Protocol

Phase 5 starts the self-improvement loop:

```text
friction + interventions + audit findings
  -> harness-cli propose
  -> proposed backlog item
  -> human review
  -> implementation with predicted impact
  -> close with actual outcome
```

## Generate Proposals

```bash
scripts/bin/harness-cli propose
```

The command is rule-based. It looks for:

- repeated trace friction,
- repeated intervention patterns,
- repeated story signals (the mineable subset of `implementation-notes.html`,
  recorded with `scripts/bin/harness-cli story signal add`; see decision 0007),
- non-zero audit categories.

Story signals carry the four implementation-note categories
(`design_decision`, `deviation`, `tradeoff`, `open_question`). When the same
signal recurs across stories (`>= 2`), `propose` reads it as a spec, plan, or
template gap and emits a proposal attributed to `Task specification`.

Each proposal includes title, component, evidence, predicted impact, risk,
suggested action, validation plan, and confidence.

## Commit Proposals

```bash
scripts/bin/harness-cli propose --commit
```

Committed proposals become `proposed` backlog items. Humans review them with:

```bash
scripts/bin/harness-cli query backlog --open
```

## Review Rules

- Tiny proposals may be implemented directly when they only clarify docs.
- Normal proposals need a story packet or clear backlog acceptance.
- High-risk proposals need a durable decision record before changing source
  hierarchy, architecture direction, validation requirements, or risk policy,
  plus one independent check (deterministic proof or a second reviewer) recorded
  with `harness-cli intervention add --type review`. Do not accept a
  behavior-changing improvement on the implementing agent's word alone.
- Completed proposal work must close the backlog item with actual outcome
  evidence.

## No-Regression Gate

`propose` predicts impact but verifies nothing, and `--commit` only files a
backlog item. Before *accepting* an improvement that changes agent behavior or
validation results, name a measured anti-goal so the change cannot make the
harness worse while claiming to make it better. This is the goal-loop
admissibility screen (`docs/GOAL_LOOP.md`) applied to harness self-improvement.

Pin one anti-goal before committing the change, and read it three times:

- **Before** — state the wall as a metric, for example `entropy_score does not
  increase`, `story verify-all stays green`, or `required trace tier does not
  drop`.
- **After** — read the actual metric from source with `audit` / `story
  verify-all`, not from the proposal's predicted impact.
- **Paired** — accept only when the friction the proposal targeted actually
  dropped **and** the anti-goal held. A friction drop that raised entropy or
  broke a verification is a failed improvement that looks like a win.

An improvement whose measured anti-goal breaches is reverted or reworked, not
merged. Record the outcome on the backlog item; a self-reported success without a
metric read is not acceptance.

## Validation

After implementation, compare the predicted impact with:

- `scripts/bin/harness-cli audit`,
- `scripts/bin/harness-cli query friction`,
- `scripts/bin/harness-cli query interventions`,
- benchmark trace quality and harness compliance when benchmark proof applies.
