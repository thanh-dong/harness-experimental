# 0013 Keep Deterministic Verification Steps Under Fable 5.1

Date: 2026-09-09

## Status

Accepted

## Context

Anthropic's behavioral guidance differs between the two models this harness is
written for. The Claude Opus 5 guidance removes prompt-level "verify your work"
scaffolding, because that model checks its work without being told. The Claude
Fable 5.1 guidance keeps it, because that model can report progress it has not
yet confirmed with a tool result. A prompt audit on this repo asked whether the
harness should follow the Opus 5 direction and drop its verification prompts.

The harness's done gate is not prompt scaffolding. It is deterministic:
`harness-cli story verify` runs the story's `verify_command`,
`scripts/check-diagrams.sh` lints every change diagram, and the high-risk rule
requires one independent check (a passing proof, CI, or a second reviewer)
before a story is done. These steps produce a recorded pass or fail that a
human can audit later, regardless of which model ran the session.

## Decision

Keep every deterministic verification step in the done gate as it is:
`story verify`, `check-diagrams.sh`, and the independent-check rule for
high-risk work. Add one sentence to the done gates asking the agent to ground
each progress claim in a tool result from the current session, since that is
the Fable 5.1 gap the deterministic steps do not close on their own.

Revisit this decision at the next Claude model release. If the next model's
guidance also drops verification scaffolding and the session-level evidence
sentence stops catching anything, remove the sentence; the deterministic steps
stay in any case because they protect the record, not the model.

## Alternatives Considered

1. Follow the Opus 5 guidance and remove verification prompts from the gate —
   rejected: the gate also serves Fable 5.1 sessions, Codex sessions, and human
   reviewers reading the record, and none of those benefit from a weaker gate.
2. Keep the deterministic steps but skip the evidence sentence — rejected: a
   story can pass `story verify` and still ship with an inaccurate progress
   report; the sentence targets the report, not the proof.
3. Branch the gate wording by model — rejected: two gate flavors (bash and
   MCP) already exist; a third axis would drift.

## Consequences

Positive:

- The done gate reads the same for every model and tool surface.
- Progress reports must cite session evidence, which matches the harness rule
  that a story is done only when its proof has actually run.

Tradeoffs:

- Opus 5 sessions carry one sentence of guidance they may not need.
- The revisit point depends on someone reading the next model's guidance.

## Follow-Up

- At the next model release, re-read both models' behavioral guidance and
  either confirm this decision or supersede it.
