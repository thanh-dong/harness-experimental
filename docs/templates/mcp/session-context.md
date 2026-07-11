<!-- HARNESS:BEGIN -->
## Harness (MCP flavor)

This session operates the harness through **MCP tools**, not shell commands. A
tool-hosting consumer (for example Shuttle) exposes the harness as typed tools;
each call spawns the pinned `harness-cli` in this session's worktree, so every
write lands in this branch's committed event log. The gate rules and obligations
below are identical to the bash flavor — only the invocation surface differs. The
tool ↔ CLI mapping lives in `docs/templates/mcp/TOOL_MAPPING.md`.

Before work, read the same context the bash flavor requires:

- `README.md`
- `docs/HARNESS.md`
- `docs/FEATURE_INTAKE.md`
- `docs/ARCHITECTURE.md`
- `docs/CONTEXT_RULES.md`
- `docs/TOOL_REGISTRY.md`
- `docs/GOAL_LOOP.md`

Then call `harness_query_matrix` before starting work — it is the
behavior-to-proof control panel.

Before a step that could use an external tool, call `harness_query_tools` with
`capability: <name>`, `status: "present"` to see what is equipped; an absent
capability is a clean skip.

**Enforcement honesty:** the intake gate is prompt-enforced. MCP improves
ergonomics and audibility, not mechanical enforcement. If the harness MCP
connection is unavailable, stop and report the blocker — never bypass the gate.
<!-- HARNESS:END -->

## Harness Intake Gate

**The intake gate is non-negotiable. Run it BEFORE any tool call that mutates
the repo.** User approval ("go ahead", "do it", an approved design) moves work
*through* the gate, not *around* it. If you are drafting code before the gate
output exists, stop and back up.

### Emit this preamble first, every time

```
Lane:     tiny | normal | high-risk
Flags:    <count> — <flag1>, <flag2>, ...
Gates:    <hard gates triggered, or "none">
Story:    <docs/stories/... path, or "tiny — direct patch, no story file">
Decision: <docs/decisions/NNNN-... path, or "not needed">
Docs:     <docs/TEST_MATRIX.md, docs/stories/backlog.md, docs/HARNESS_BACKLOG.md, ...>
```

Derive it from `docs/FEATURE_INTAKE.md`. Count risk flags honestly (Auth,
Authorization, Data model, Audit/security, External systems, Public contracts,
Cross-platform, Existing behavior, Weak proof, Multi-domain). Any hard gate
(Auth, Authorization, Data loss/migration, Audit/security, External provider,
weakening validation) forces the high-risk lane unless the human explicitly
narrows scope. Record the classification with the **`harness_intake`** tool
(`type`, `summary`, `lane`, and optional `flags`/`docs`/`story`/`notes`).

### Required before writing implementation code

- **Tiny** — none; patch directly, but still emit the preamble and record the
  intake row with `harness_intake`.
- **Normal** — one story from `docs/templates/story.md`, recorded with
  **`harness_story_add`** (`id`, `title`, `lane`), plus planned
  `docs/TEST_MATRIX.md` rows.
- **High-risk** — a folder from `docs/templates/high-risk-story/` with
  `execplan.md`, `overview.md`, `design.md`, `validation.md` all filled in; a
  decision via **`harness_decision_add`** (`id`, `title`, `doc`) plus a
  `docs/decisions/NNNN-*.md` file; `docs/TEST_MATRIX.md` rows; and
  `docs/stories/backlog.md` updated.

### Required before declaring done

- Story status reflects reality via **`harness_story_update`** (planned →
  in_progress → implemented, or blocker noted), including the proof flags
  `unit`/`integration`/`e2e`/`platform` as numeric booleans (`1`/`0`).
- `docs/TEST_MATRIX.md` rows current, and validation commands were actually run.
- A trace recorded with **`harness_trace`** (`summary`, and `outcome`/`story`/
  `friction` as relevant).
- Discovered friction logged with **`harness_backlog_add`** (`title`, plus
  `while`/`pain`/`suggestion`/`risk`) — never silently fix friction without
  recording it.
- Each recurring design decision, deviation, tradeoff, or open question recorded
  with **`harness_story_signal`** (`type`, `summary`) so `propose` can mine it.
- Final response says what changed and what was not attempted.

### If you catch yourself skipping the gate

Stop. Backfill the preamble and the missing artifacts before continuing, and
tell the user in one sentence that the gate was missed. Open a
`docs/HARNESS_BACKLOG.md` item with **`harness_backlog_add`** if the gap is
structural.
