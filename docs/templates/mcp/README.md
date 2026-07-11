# MCP-flavored context templates

The harness ships its operating instructions in two flavors with **one owner**
(this repo). Gate semantics are identical across both; only the invocation
surface changes.

- **Bash flavor** — the default. Operating instructions reference
  `scripts/bin/harness-cli ...` shell commands. Consumed by CLI users and agents
  that run the harness directly (Claude Code, Codex, Cursor, plain shells). Lives
  in `docs/FEATURE_INTAKE.md`, `docs/HARNESS.md`, and the installer-generated
  `CLAUDE.md` / `AGENTS.md` shim blocks.
- **MCP flavor** — this directory. Operating instructions reference typed MCP
  tool names (`harness_intake`, `harness_story_add`, …) instead of shell
  commands. Consumed by tool-hosting environments (for example Shuttle) that
  expose the harness as MCP tools and generate session gate-context from these
  templates, so no consumer forks the gate wording.

## Contents

| File | Purpose |
| --- | --- |
| `session-context.md` | The gate-context the host injects at session launch: the harness shim block plus the full intake-gate preamble and record-keeping obligations, MCP-flavored. |
| `TOOL_MAPPING.md` | Authoritative `harness_*` tool ↔ `harness-cli` command mapping, with key arguments. |

## Keeping the flavors in lockstep

When the bash-flavored gate wording changes in `docs/FEATURE_INTAKE.md`,
`docs/HARNESS.md`, or the installer shim blocks, mirror the change here — same
obligations, MCP invocation surface. Run `scripts/lint-mcp-templates.sh` to
assert every `harness_*` name used in these templates is defined in
`TOOL_MAPPING.md` and maps to a real `harness-cli` command.
