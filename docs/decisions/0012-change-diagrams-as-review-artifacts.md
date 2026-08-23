# 0012 Change Diagrams As Separate Review Artifacts

Date: 2026-08-23

## Status

Accepted

## Context

Story packets described changes in prose and SQL only. Human review of scope,
architecture direction, behavior, data changes, and boundary crossings had no
artifact to approve other than the whole packet, and the confirmations
`docs/HARNESS.md` requires (architecture direction, data loss, external
providers) were recorded nowhere durable. The implementing agent likewise had
nothing mechanical to derive tests, re-run sets, or placement from. The repo
already had two diagram generators in its tool registry seed (`codegraph
affected`, `c3 graph --format mermaid`) and no diagram standard.

## Decision

Add **change diagrams** as a harness artifact class, specified in
`docs/DIAGRAMS.md`:

- Seven kinds — D1 blast radius, D2 component delta, D3 sequence, D4 state,
  D5 data-model delta, D6 story DAG, D7 boundary — each defined by what the
  human reviews and what the agent derives. No other kinds without a decision.
- Selected by intake risk flags, never by the human: none on tiny; D1 (when
  `impact-analysis` is active) and D3 (multi-component) on normal; D1, D2, D3
  always on high-risk; D4/D5/D7 by flag; D6 for new specs and initiatives.
- One Mermaid file per diagram at `<packet>/diagrams/D<n>-<slug>.md` with a
  fixed header (`Story`, `Kind`, `Source`, `Scope`, `Status`, `Reviewed-by`,
  `Reviewed-at`, D1 `Coverage`), delta-only content, a fixed highlight class
  set, and a 25-node cap. Mermaid source is the contract; no images.
- Reviewed at three named stages — intake checkpoint (D1, D6), design review
  (D2–D5, D7), done gate (all must match shipped code). A review sets
  `Status: reviewed` in the file and is recorded durably with
  `intervention add --type review`; a changed subject flips the file to
  `stale` in the same commit.
- Linted by `scripts/check-diagrams.sh`; a missing or `stale` required diagram
  blocks done, like a missing `implementation-notes.html`.

## Alternatives Considered

1. Inline Mermaid inside `design.md` / story files — rejected: no per-diagram
   review status, and a stale diagram is invisible inside a reviewed packet.
2. Rendered images (PNG/SVG) — rejected: unreadable to agents, rot silently,
   binary diffs.
3. Mermaid `C4*` diagram types as the mandated notation — rejected as the
   default because Mermaid marks them experimental; allowed as an option.
4. A diagram table in the durable layer plus `story diagram` subcommands —
   deferred: the existing `review` intervention type already records the
   approval; revisit once the rule has run on real packets.

## Consequences

Positive:

- Every human-confirm trigger in the intake gate now has a concrete artifact
  to approve, and the approval is a durable intervention row.
- The implementing agent has a defined derivation (tests from D3/D4, migration
  from D5, re-run set from D1, order from D6, fixtures from D7).
- Both gate flavors (bash and MCP) carry identical diagram obligations.

Tradeoffs:

- D3–D7 are hand-drawn and can drift; the `stale` rule and story signals are
  the only guard until tooling exists.
- Review state lives in two places (file, intervention); the intervention wins.
- Slightly more packet overhead on normal lane when a story spans components.

## Follow-Up

- Consider a `--diagrams` field on `story add/update` so `query matrix` shows
  review state, and a `story verify` hook that fails on a required diagram
  that is not `reviewed`.
- `docs/templates/implementation-notes.html` is missing from the installer
  file list (pre-existing).
