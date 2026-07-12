# Project Rules

## Subagents

Use Fable subagents (`model: "fable"` on the Agent tool) when you need more
intelligence — e.g. hard debugging, architecture, or verification steps.

## Working Together Through The Event Log

Durable harness state (intake, stories, decisions, backlog, traces,
interventions, signals) lives in a **git-tracked, append-only event log** at
`.harness/events/`. `harness.db` is a rebuilt cache. This is how multiple
humans and agents share state; treat it as team state, not local scratch. Full
reference: `docs/EVENT_LOG.md`.

Rules of engagement — every agent, every session:

- **Write state through the CLI, never by hand.** `harness-cli intake` /
  `story` / `decision` / `backlog` / `trace` … append events. Do not edit
  `harness.db`, and do not hand-edit the generated views
  (`docs/TEST_MATRIX.md`, `docs/HARNESS_BACKLOG.md`, `docs/decisions/README.md`)
  — they carry a `generated — do not hand-edit` marker and are rewritten from
  the log after every mutation.
- **Pull before you start; commit the log with your work.** `git pull` first —
  the CLI auto-replays new events, so `query matrix` reflects teammates'
  latest. When you commit code, **stage `.harness/events/` and the regenerated
  views in the same commit** so your records travel with the change. Never
  commit `harness.db` (it is gitignored; rebuilt on demand).
- **Your writer identity is per-clone and automatic.** Events append to your
  own `.harness/events/<writer-id>.jsonl` (email hash + per-clone
  disambiguator), so parallel agents/clones never conflict on merge. Set
  `HARNESS_WRITER=<name>` to label a distinct agent on the same machine (e.g.
  `HARNESS_WRITER=reviewer`).
- **After a merge, check for overwrites.** Concurrent edits to the same story
  field resolve last-writer-wins; `harness-cli audit` lists them under
  *Concurrent LWW updates (causal audit)*. If a value you set was overwritten,
  re-apply it — the loser is still in the log.
- **If the cache looks wrong**, run `harness-cli rebuild` (or delete
  `harness.db` and run any query — it auto-rebuilds from the log). Two rebuilds
  print the same dump hash; a mismatch means a corrupt log line.
- **First time on a repo?** Fresh install → `docs/SETUP.md`. Upgrading a
  pre-event-log repo → `harness-cli migrate` then `harness-cli migrate-to-events`
  (`docs/EVENT_LOG.md`). Either way, finish with
  `bash scripts/verify-harness.sh .` — 8/8 means the install works; anything
  less, fix before writing state.

<!-- HARNESS:BEGIN -->
## Harness

Claude Code loads this file into every session, but it does not auto-load
`AGENTS.md`. The bare `@` lines below import the always-required harness
context (the "Must in all lanes" set from `docs/CONTEXT_RULES.md`) at
context-load time. Never wrap them in backticks; that disables the import.

@AGENTS.md

@docs/FEATURE_INTAKE.md

Also run `scripts/bin/harness-cli query matrix` before starting work.

Lane-dependent context (`README.md`, `docs/HARNESS.md`, `docs/ARCHITECTURE.md`,
`docs/CONTEXT_RULES.md`, product docs, stories, decisions) is intentionally not
imported — read it per lane, as `docs/CONTEXT_RULES.md` prescribes.
<!-- HARNESS:END -->
