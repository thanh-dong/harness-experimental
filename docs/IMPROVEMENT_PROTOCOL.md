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
  hierarchy, architecture direction, validation requirements, or risk policy.
- Completed proposal work must close the backlog item with actual outcome
  evidence.

## Validation

After implementation, compare the predicted impact with:

- `scripts/bin/harness-cli audit`,
- `scripts/bin/harness-cli query friction`,
- `scripts/bin/harness-cli query interventions`,
- benchmark trace quality and harness compliance when benchmark proof applies.
