# Impact Analysis

Blast-radius analysis for change requests. It answers, with evidence instead of
judgment: which components (technical perspective) and which features (product
perspective) a change touches, how they are touched, and what proof must re-run.

The harness records and gates; the agent orchestrates. The two analysis tools
are agent-side (a CLI and a skill), so no `harness-cli` subcommand runs
them. The harness's part is the **inbound tool registry** (see
`docs/TOOL_REGISTRY.md`), the durable evidence the feature join depends on, and
the trace where results are stamped.

Impact analysis is the first consumer of the inbound tool registry, not a
dependency of it. It owns no activation switch of its own: it looks up the
`impact-analysis` capability, reads the scanned presence of whatever providers
are registered, and chooses a posture from that. The registry decides what is
equipped; this document decides what to do about it.

## When It Runs

Impact analysis is capability-gated. Context spend stays proportional to risk.

| Lane | Impact analysis |
| --- | --- |
| Tiny | Skip. The risk checklist alone is enough. |
| Normal | Must run before the risk checklist is completed. |
| High-risk | Must run, and its output is required evidence in the story packet. |

Also run it, regardless of provisional lane, when the agent cannot confidently
name the affected files, components, or product docs from the request alone.
If the result escalates the lane, re-enter intake with the new lane.

## Dependencies

Two complementary providers serve the `impact-analysis` capability. Both are
inbound tools; neither is compiled into the harness, and either may be absent
on any machine.

| Provider | Capability role | Kind | Scan target | Agent runtime |
| --- | --- | --- | --- | --- |
| CodeGraph | Code graph: changed files, dependents, call paths, affected tests | `cli` (`codegraph impact`, `codegraph affected`) | `.codegraph` | Any agent or headless runner with the `codegraph` binary |
| C3 | Named component model with declared code locations | `skill` (`c3` skill, `c3-audit` for drift) | `.c3` | Claude Code skill (Claude-specific) |

The `kind` is what lets a non-Claude agent know which dependencies it can
orchestrate: it treats a `skill` it cannot run as absent and degrades, rather
than failing. A `cli` provider needs no agent session at all, which is what
makes the code-graph half viable in pipeline runners working from a fresh
clone or worktree. Registration is described once, in `docs/TOOL_REGISTRY.md`
(the codegraph + c3 examples there are the per-install seed); do not duplicate
the commands here. `harness.db` is local and gitignored by design, so each
install runs that seed once.

Note: the `.codegraph/` index is machine-local and gitignored (never commit
it). A fresh clone or worktree rebuilds it with `codegraph init` (seconds) or
restores it from a CI cache and catches up with `codegraph sync`
(sub-second). The `.c3/` facts are git-tracked and travel with the clone;
only its disposable cache needs `c3 repair` after a branch switch.

## Activation And Skip Rule

There is no impact-analysis-specific switch. Activation is read from the
registry, capability-first:

```bash
scripts/bin/harness-cli query tools --capability impact-analysis
```

- No provider registered for `impact-analysis`: the capability is inactive.
  Skip impact analysis entirely; intake proceeds with the baseline risk
  checklist. Note `impact: skipped, capability inactive` in the trace. Skipping
  an inactive capability is not drift and does not set the `Weak proof` flag.
- One or more providers registered: the capability is active. Run preflight,
  then choose a posture from how many providers are actually present.

This is the generic Degrade Ladder in `docs/TOOL_REGISTRY.md` applied to one
capability. A registered provider that scans as `missing` is a failed validity
gate, not a skip: the repo declared intent to rely on it, so degrade per
Degraded Modes and set the `Weak proof` flag.

## Preflight Validity Gates

Presence is a scanned fact, not a trusted declaration. Reconcile intent with
reality at intake start, then layer the deeper freshness checks the registry
scan cannot see. Three layers, narrowest last:

1. **Equipped** — run `scripts/bin/harness-cli tool check` at intake start so
   each provider's `status` (`present` / `missing` / `unknown`) and
   `checked_at` reflect current reality. `present` means the scan target
   resolves on disk (`.codegraph`, `.c3`); it does not mean fresh or live.
2. **Valid / fresh** — for each `present` provider, confirm it is not stale
   before trusting its output:

   | Provider | Freshness check | On failure |
   | --- | --- | --- |
   | CodeGraph | `codegraph sync` (incremental catch-up to the working tree; sub-second). In a fresh clone/worktree with no `.codegraph/`, `codegraph init` rebuilds it in seconds — a rebuildable index is not a failed gate. | Component impact reported as UNKNOWN, not empty. |
   | C3 | `.c3/` audit (`c3-audit`) clean since the last structural change; `c3 repair` reseals the cache after a branch switch | Degrade component names to raw file paths; the feature join still runs. |

3. **Live** — only the agent runtime can see whether a session-attached
   provider (an `mcp` server, or a `skill` in a non-Claude runtime) is
   actually usable this session. Confirm live usability at call time; a
   provider that is equipped but not usable degrades like a missing one.
   `cli` providers like codegraph skip this layer: runnable on `PATH` is
   live.

Any failed gate at any layer sets the `Weak proof` risk flag on the intake row
and is noted in the trace. Invalid tooling must degrade visibly, never produce
silently stale results.

## Degraded Modes

This specializes the registry's present-provider-count ladder for the two
`impact-analysis` providers. A provider counts as available only when it is
both `present` (layer 1) and fresh/live (layers 2-3).

| Mode | Available | Blast radius | Components | Features | Posture |
| --- | --- | --- | --- | --- | --- |
| Full | codegraph + c3 | changed files + dependents | C3 names | trace join + coverage | Normal operation. |
| Degraded | one of the two | git-diff files only when codegraph is absent | raw file paths when c3 is absent | trace join, lower coverage | Set `Weak proof`. |
| Inactive | neither registered | not computed | not computed | not computed | Skip; trace note only. |

## Pipeline

```text
change request
  -> codegraph impact/affected: changed files + dependent files (blast radius)
       -> map files to C3 components via declared code locations
       -> join files against trace history (trace.files_changed)
            -> story (trace.story_id)
            -> product doc (story.contract_doc)
  -> impact set: components + features + coverage
```

The join key is file paths. No curated feature-to-component mapping exists or
should be created; the durable layer accumulates the feature edge as a side
effect of normal trace discipline.

## Coverage Signal

An empty feature join has two indistinguishable causes: no impact, or no data.
Never report it as empty. Every impact result states coverage:

```text
coverage: N of M blast-radius files have story/trace history
```

When coverage is below half, report feature impact as UNKNOWN, set the
`Weak proof` risk flag, and fall back to reading the relevant `docs/product/*`
manually. Feature-impact quality is emergent: it is weak in a fresh install and
improves only while traces carry `--story` and stories carry `--contract`
(the `story.contract_doc` column).

## Gated Decisions

The impact set exists to change decisions, not to be context. It must feed:

1. Risk flags: `Existing behavior`, `Multi-domain`, and `Public contracts` are
   answered from the impact set, not judgment.
2. Validation scope: stories in the impact set form the required re-run set;
   run `scripts/bin/harness-cli story verify <id>` once per story in the set.
3. Reading list: the affected product docs and components become the
   implementation-phase reading list.

Record the impact summary, coverage, the posture chosen, and any failed
validity gate in the trace notes for the task. If the analysis was skipped
where this document requires it, say so in the trace rather than implying it
ran.
