# Agent Instructions

Durable harness state is a git-tracked event log at `.harness/events/`;
`harness.db` is a rebuilt cache. Agents collaborate by writing through the CLI
and committing the log with their work — see the "Working Together Through The
Event Log" section in `CLAUDE.md` and the full reference in `docs/EVENT_LOG.md`.

<!-- HARNESS:BEGIN -->
## Harness

This repo uses Harness. First-time setup (install + wire the capability tools):
`docs/SETUP.md`. Before work, in every lane:

- read `docs/FEATURE_INTAKE.md`
- run `scripts/bin/harness-cli query matrix` on macOS/Linux, or `.\scripts\bin\harness-cli.exe query matrix` on Windows

Then read the lane-dependent docs that `docs/CONTEXT_RULES.md` prescribes for
your lane (`README.md`, `docs/HARNESS.md`, `docs/ARCHITECTURE.md`,
`docs/TOOL_REGISTRY.md`, `docs/GOAL_LOOP.md`, product docs, stories, decisions).

Use the Rust Harness CLI at `scripts/bin/harness-cli` on macOS/Linux or
`scripts/bin/harness-cli.exe` on Windows as the main operational tool. Before a
step that could use an external tool, run `scripts/bin/harness-cli query tools
--capability <name> --status present` to see what is equipped; an absent
capability is a clean skip.
<!-- HARNESS:END -->
