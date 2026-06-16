# Overview

## Current Behavior

`harness-cli propose` mines three structured sources at a `>= 2` recurrence
threshold: repeated trace friction, repeated interventions, and audit findings.
Implementation notes (`implementation-notes.html`, added to the lanes in
`docs/FEATURE_INTAKE.md`) are free-form HTML and are invisible to `propose`.

## Target Behavior

Agents can record typed, mineable story signals
(`design_decision | deviation | tradeoff | open_question`) into the durable
layer. `propose` mines recurring signals as a fourth source, turning the
implementation-note categories into self-evolution fuel.

## Affected Users

- Agents doing normal / high-risk work (record signals).
- Humans reviewing `propose` output (gain a new proposal source).

## Affected Product Docs

- `docs/FEATURE_INTAKE.md`
- `docs/IMPROVEMENT_PROTOCOL.md`
- `docs/decisions/0007-story-signal-mining.md`

## Non-Goals

- Parsing HTML inside `propose` (explicitly rejected in decision 0007).
- Requiring a signal for every note heading — only recurring categories.
- Changing the existing friction / intervention / audit propose loops.
