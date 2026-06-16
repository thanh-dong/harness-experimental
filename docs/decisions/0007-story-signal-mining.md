# 0007 Story Signal Mining

Date: 2026-06-16

## Status

Accepted

## Context

Normal and high-risk work now keeps a running `implementation-notes.html` in the
story packet (decision recorded in `docs/FEATURE_INTAKE.md`). That note captures
four recurring categories — design decisions, deviations, tradeoffs, and open
questions — as human-readable HTML.

The self-improvement engine (`harness-cli propose`, see
`docs/IMPROVEMENT_PROTOCOL.md`) is deliberately rule-based: it mines *structured,
countable* rows (repeated trace friction, repeated interventions, audit
findings) at a `>= 2` recurrence threshold. It cannot read free-form HTML prose.

So the note, as a pure artifact, advances project memory and reviewability but
contributes nothing to self-evolution: the richest signal an agent produces
about where the harness is ambiguous or under-specified is invisible to
`propose`. The note already names the exact categories that, when they recur
across stories, indicate a harness gap (a missing plan rule, an ambiguous spec,
an under-documented tradeoff). The gap is purely that these signals are not
recorded in the durable layer.

## Decision

Add a durable, mineable signal extracted from implementation notes, and teach
`propose` to mine it.

1. **Schema migration `006-story-signal.sql`** adds an additive `story_signal`
   table, modeled on the existing `intervention` table:
   - `id`, `created_at`, `story_id` (-> `story`), `trace_id` (-> `trace`),
   - `type` constrained to
     `design_decision | deviation | tradeoff | open_question` (the four note
     headings),
   - `summary` (the normalized, mineable text), `component` (optional harness
     component the signal implicates), `notes` (optional).
   - The migration is additive and auto-applies to existing databases via the
     existing `apply_pending_migrations` path, so no backfill or data loss.

2. **CLI surface** `harness-cli story signal add` records one signal, and
   `harness-cli query signals` lists them (filterable by `--type` / `--story`),
   mirroring `intervention add` / `query interventions`.

3. **Propose rule.** `propose` gains one loop over recurring story signals
   (same normalized `summary` within a `type`, `count >= 2`), reusing the
   existing `repeated_values` grouping and `confidence_for_count` rule. Each
   proposal is attributed to the `Task specification` component by default (a
   recurring deviation/ambiguity is a spec or plan-template gap), carries the
   recurrence as evidence, and a validation plan of "the signal stops recurring
   in the next stories after the doc/template fix."

The HTML note stays the human narrative; the recorded signal is the
machine-mineable subset. Recording a signal is part of the normal/high-risk
"done" definition only for *recurring* categories — agents are not asked to
log every note heading mechanically.

## Alternatives Considered

1. **Parse the HTML in `propose`.** Rejected. It would make the proposer
   non-deterministic and pull HTML parsing into the engine, contradicting the
   rule-based design that keeps proposals auditable (`PHASE5.md` lists
   "circular / low-quality proposals" as the failure signal to avoid).
2. **Reuse `harness_friction` on the trace.** Rejected. Friction is per-trace
   and tied to a single execution; signals are per-story design facts that
   outlive one trace and carry a typed category friction lacks.
3. **Reuse the `intervention` table with a new `type`.** Rejected. Interventions
   are corrections to agent behavior by an external actor; signals are the
   agent's own design narrative. Overloading the constraint would blur both
   `propose` loops and the audit semantics.

## Consequences

Positive:

- The highest-quality agent-produced signal becomes visible to self-evolution.
- `propose` gains a fourth, typed source with the same `>= 2` discipline.
- Implementation notes now have a durable, queryable counterpart.

Tradeoffs:

- One more thing to record on normal/high-risk work; mitigated by limiting it to
  *recurring* categories, not every heading.
- Possible overlap with trace decisions; the boundary is: trace decision = "what
  I decided this run," story signal = "a design fact that recurs across stories."

## Follow-Up

- Implement under story `US-027-implementation-note-signals`.
- After a few real stories, run `propose` and confirm a recurring signal yields
  an actionable proposal; otherwise tune the threshold or the component mapping.
- Revisit whether `component` should be agent-supplied or inferred per `type`.
