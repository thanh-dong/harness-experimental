# MCP Tool ↔ CLI Command Mapping

This table is the single source of truth binding each harness MCP tool to its
exact `harness-cli` equivalent. An MCP host (for example Shuttle) exposes the
tools in the left column; each call spawns the pinned `harness-cli` with the
command in the right column, in the calling session's worktree, so writes land
in that branch's committed event log. The gate semantics are identical to the
bash flavor — only the invocation surface changes.

`scripts/lint-mcp-templates.sh` enforces this table: every `harness_*` name used
in the MCP templates must appear here, and every CLI command here must exist in
`harness-cli --help` output.

## Write / record tools

| MCP tool | CLI command | Key arguments |
| --- | --- | --- |
| `harness_intake` | `harness-cli intake` | `--type <input-type>` `--summary <text>` `--lane <tiny\|normal\|high-risk>` (opt: `--flags`, `--docs`, `--story`, `--notes`) |
| `harness_story_add` | `harness-cli story add` | `--id <id>` `--title <text>` `--lane <tiny\|normal\|high-risk>` (opt: `--contract`, `--verify`, `--notes`) |
| `harness_story_update` | `harness-cli story update` | `--id <id>` (opt: `--status`, `--evidence`, `--unit <0\|1>`, `--integration <0\|1>`, `--e2e <0\|1>`, `--platform <0\|1>`, `--verify`) |
| `harness_story_signal` | `harness-cli story signal add` | `--type <design_decision\|deviation\|tradeoff\|open_question>` `--summary <text>` (opt: `--story`, `--trace`, `--component`, `--notes`) |
| `harness_intervention_add` | `harness-cli intervention add` | `--type <correction\|override\|review>` `--description <text>` `--source <human\|agent\|ci>` (opt: `--trace`, `--story`, `--impact`) |
| `harness_decision_add` | `harness-cli decision add` | `--id <id>` `--title <text>` (opt: `--status` [default accepted], `--doc`, `--verify`, `--predicted`, `--notes`) |
| `harness_trace` | `harness-cli trace` | `--summary <text>` (opt: `--intake`, `--story`, `--agent`, `--outcome`, `--duration`, `--tokens`, `--friction`, `--actions`, `--read`, `--changed`, `--decisions`, `--errors`, `--notes`) |
| `harness_backlog_add` | `harness-cli backlog add` | `--title <text>` (opt: `--while`, `--pain`, `--suggestion`, `--risk <tiny\|normal\|high-risk>`, `--predicted`, `--notes`) |

## Query / read tools

Each read tool maps to a `harness-cli query <view>` subcommand.

| MCP tool | CLI command | Key arguments |
| --- | --- | --- |
| `harness_query_matrix` | `harness-cli query matrix` | opt: `--numeric` (render proof flags as `1`/`0`) |
| `harness_query_tools` | `harness-cli query tools` | opt: `--capability <name>`, `--status <present\|missing\|unknown>`, `--json`, `--summary`, `--responsibility` |
| `harness_query_backlog` | `harness-cli query backlog` | — |
| `harness_query_decisions` | `harness-cli query decisions` | — |
| `harness_query_intakes` | `harness-cli query intakes` | — |
| `harness_query_traces` | `harness-cli query traces` | — |
| `harness_query_friction` | `harness-cli query friction` | — |
| `harness_query_interventions` | `harness-cli query interventions` | — |
| `harness_query_signals` | `harness-cli query signals` | — |
| `harness_query_stats` | `harness-cli query stats` | — |

## Rules for keeping this in sync

- Add a row here before referencing a new `harness_*` tool in any MCP template.
- The CLI command must be a real `harness-cli` subcommand path (verified by the
  lint script against `harness-cli --help`).
- Proof flags stay numeric booleans (`--unit 1 --integration 0`), never
  `yes`/`no` — same rule as the bash flavor.
- Lanes are `tiny | normal | high-risk`; use `tiny`, never `low`.
