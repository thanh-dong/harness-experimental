# 0011 CodeGraph Replaces GitNexus As The Code-Graph Impact Provider

Date: 2026-07-12

## Status

Accepted

## Context

The `impact-analysis` capability had two providers: GitNexus (code graph,
kind `mcp`) and C3 (component model, kind `skill`). The pipeline feature runs
in fresh clones and git worktrees, where GitNexus structurally fails three
ways:

1. Its index lives in an untracked `.gitnexus/` directory at the repo root —
   absent in every fresh clone and worktree (worktrees share `.git`, not
   untracked files).
2. Its global registry (`~/.gitnexus/registry.json`) keys repos by absolute
   path, so a worktree at a new path is an unknown repo even when the main
   checkout is indexed.
3. As an `mcp` provider it requires a live agent session, which headless
   runners do not have (the layer-3 "live" preflight gate).

Result: in the pipeline, impact analysis would always run Degraded with
`Weak proof`, losing the blast-radius half exactly where automation needs it.

## Decision

Replace GitNexus with CodeGraph as the code-graph provider. The provider set
becomes codegraph (kind `cli`, scan `.codegraph`) + c3 (kind `skill`, scan
`.c3`). Verified in a real worktree test: CodeGraph's index is path-relative
(a copied `.codegraph/` works at a new path), cold `codegraph init` took ~5s
on an 804-file repo, incremental `codegraph sync` ~0.4s, and
`codegraph impact` / `codegraph affected --json` run headless with no agent
session.

Runner preflight: restore `.codegraph/` from CI cache keyed by merge-base
commit, then `codegraph sync`; on cache miss, `codegraph init`. Never commit
`.codegraph/` — it is a machine-local derived artifact.

## Alternatives Considered

1. Keep GitNexus and add a pipeline index step — blocked by the
   absolute-path registry and the mcp-session requirement, not just the
   missing index.
2. Commit the index to git — churn and merge conflicts on a generated
   binary-ish artifact; rejected for both tools.
3. Accept permanent Degraded mode in pipelines (git-diff files + trace
   join) — loses dependents/call-path analysis where it matters most.
4. SCIP/LSIF indexers — CI-native but per-language and a bigger integration;
   codegraph already covers the need with one CLI.

## Consequences

Positive:

- Full-mode impact analysis works in pipeline runners, worktrees, and fresh
  clones; the layer-3 live gate no longer applies to the code-graph half.
- `codegraph affected` maps directly onto the validation re-run set.

Tradeoffs:

- Interactive sessions lose GitNexus-specific MCP ergonomics; nothing stops
  an install from additionally registering gitnexus for local interactive
  use, but the documented seed is codegraph + c3.
- CodeGraph's index is still machine-local: every runner workspace must run
  the init/sync preflight before the capability scans `present`.

## Follow-Up

- Add the cache-restore + `codegraph sync` preflight step to the pipeline
  runner definition when that feature lands.
