# Setup

One pass an agent runs after dropping the harness into a repo. It installs the
operating layer, creates the durable store, and **wires the capability tools that
are actually present**. A tool that is absent is a clean skip, never a failure —
the core works with zero tools registered.

## 1. Install

```bash
curl -fsSL "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.sh?$(date +%s)" | bash -s -- --claude --yes
scripts/bin/harness-cli init
git add .harness/events .gitignore .gitattributes && git commit -m "chore: init harness event log"
```

`--claude` imports the harness context into every Claude Code session. Drop it for
other agents. Windows: `scripts/bin/harness-cli.exe`.

The durable layer is a git-tracked event log at `.harness/events/`; `init`
leaves a fresh repo event-backed from genesis, so **commit `.harness/events/`**
so teammates inherit team state. `harness.db` is a rebuilt cache and stays
gitignored. Full install/migration/collaboration reference: `docs/EVENT_LOG.md`.

## 2. Wire the tools

Tools are **capability providers**; a workflow step looks them up by capability,
never by name. Register only what this environment actually has, so nothing shows
a false `missing`. Re-run a line with `--force` to update an existing entry.

```bash
H=scripts/bin/harness-cli

# impact-analysis — blast radius before edits (docs/IMPACT_ANALYSIS.md)
# .codegraph is machine-local (gitignored); on a fresh clone/worktree run
# `codegraph init` first (seconds), then register.
command -v codegraph >/dev/null 2>&1 && "$H" tool register --name codegraph --kind cli --capability impact-analysis \
  --scan ".codegraph" --command "codegraph" \
  --description "Code-graph blast radius for impact analysis" --responsibility Verification
[ -d .c3 ] && "$H" tool register --name c3 --kind skill --capability impact-analysis \
  --scan ".c3" --command "skill:c3" \
  --description "Component model and drift audit for impact analysis" --responsibility Verification

# goal-loop-orchestration — run initiatives as OKR loops (docs/GOAL_LOOP.md)
[ -d .claude/skills/reverse-tornado-okr ] && "$H" tool register --name reverse-tornado-okr --kind skill \
  --capability goal-loop-orchestration --scan ".claude/skills/reverse-tornado-okr" \
  --command "skill:reverse-tornado-okr" \
  --description "Run a goal as a self-correcting OKR loop with anti-goal guardrails" \
  --responsibility "Task specification"

# verification — deterministic evidence the loop reads instead of trusting a claim
command -v bh >/dev/null 2>&1 && "$H" tool register --name bh --kind cli \
  --capability browser-verification --command "bh" \
  --description "Isolated-Chrome browser checks for UI verification" --responsibility Verification
```

Add any other equipped tool the same way: a linter, a deploy check, a coverage or
security scan. Pick a kebab-case capability from `docs/TOOL_REGISTRY.md`.

### Install the goal-loop skill (Okra)

`goal-loop-orchestration` needs the Okra `reverse-tornado-okr` skill present. If it
is not, install it, then run the goal-loop register line above.

```bash
# Option A — Claude Code plugin (lightest; no files added to the repo)
claude plugin marketplace add lagz0ne/okra
claude plugin install okra@okra-marketplace
#   then register with --scan pointing at the installed plugin skill directory

# Option B — project-local skill (Claude + Codex; deterministic, matches the
#   default --scan ".claude/skills/reverse-tornado-okr")
git clone --depth 1 https://github.com/lagz0ne/okra /tmp/okra
mkdir -p .claude/skills .codex/skills
cp -R /tmp/okra/skills/reverse-tornado-okr .claude/skills/reverse-tornado-okr
cp -R /tmp/okra/skills/reverse-tornado-okr .codex/skills/reverse-tornado-okr
```

`tool check` marks the provider `present` once its scan target resolves; until
then goal-loop is inactive and initiatives run as a plain story list.

## 3. Reconcile and verify

```bash
scripts/bin/harness-cli tool check            # scan presence -> present/missing/unknown
scripts/bin/harness-cli query tools --summary # the equipped tool menu (built-ins + registered)
scripts/bin/harness-cli query matrix          # the proof board is queryable
scripts/calibrate-harness.sh                  # audit + score-trace behave (black-box goldens)
bash scripts/verify-harness.sh .              # 8-check install self-verify (expect 8/8)
```

`verify-harness.sh` is the one-command answer to "is the harness well set up":
binary, complete schema set (no duplicate version prefixes), git-tracked event
log, deterministic rebuild, no pending migrations, and a working write path.
Write checks run in a throwaway clone, so it never dirties the team event log.

`present` for an `mcp`/`skill` means equipped on disk, not live this session —
confirm the tool actually runs before trusting its output. A registered tool that
scans `missing` is a failed gate (set `Weak proof`), not a silent skip.

## Headless / non-interactive operation

Every command an unattended operator calls — `init`, `migrate`, `info`,
`tool register|check`, and every JSON-mode write and query — runs with **no
prompts and no TTY assumptions**. It is safe to invoke with stdin closed and no
controlling terminal (inside a server, a remote runner, or CI). No command
blocks waiting for confirmation: where a destructive action needs an override
it takes an explicit flag (`tool register --force`) rather than prompting, and a
missing prerequisite is a non-zero exit with a message, never a question.

`info` (optionally `--json`) reports CLI version, applied schema vs. the
migrations on disk, event-log format version, and whether the cache is behind
the log or the schema is behind the CLI — so a provisioner can decide
verify/migrate/refuse without parsing files. It works on an uninitialized repo,
reporting absence and exiting `0`.

The guarantee is enforced in CI: `scripts/test-headless.sh` runs the whole
command set with `</dev/null` and (where available) `setsid`, asserting each
exits `0`. Run it locally the same way:

```bash
bash scripts/test-headless.sh            # builds the CLI, exercises it headless
```

## 4. Read before changing code

`AGENTS.md` · `docs/HARNESS.md` · `docs/FEATURE_INTAKE.md` · `docs/GOAL_LOOP.md`

## Done when

- `harness-cli --version` runs and `query matrix` returns.
- `query tools --summary` lists every equipped provider as `present`.
- `calibrate-harness.sh` exits `0`.

## Updating an existing install

Run from the target repo root. `--merge` adds new harness files and **never
overwrites, moves, or deletes an existing file** — so your `docs/product/`,
`docs/stories/`, `docs/decisions/`, `harness.db`, and any customized doc are all
safe. Do not use `--override`: it moves the whole `docs/` (your stories and
decisions included) into a backup and restores only the stock files.

```bash
# 1. Add any harness files you are missing and refresh the agent shim block.
#    Existing files are kept as-is. Drop --claude if this is not a Claude Code repo.
curl -fsSL "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.sh?$(date +%s)" \
  | bash -s -- --merge --refresh-agent-shim --claude --yes

# 2. Apply schema migrations — additive and idempotent, through the current
#    version in scripts/schema/ (existing rows and data are preserved).
scripts/bin/harness-cli migrate

# 3. Cut the durable layer over to the git-tracked event log (one-time, proven:
#    verifies row-count + content-hash equality before writing; backs up the old
#    DB). Skip if already event-backed. Full detail: docs/EVENT_LOG.md.
scripts/bin/harness-cli migrate-to-events
git add .harness/events docs/TEST_MATRIX.md docs/HARNESS_BACKLOG.md docs/decisions/README.md

# 4. Wire the newly available capabilities, then verify (see "Wire the tools").
scripts/bin/harness-cli tool check
scripts/bin/harness-cli query tools --summary
scripts/calibrate-harness.sh
```

`--merge` keeps existing files, so **updated policy docs are not pulled in.** To
refresh a specific harness-owned doc to the new version — safe because these are
harness policy docs, not your content — move your copy aside and re-run merge to
recreate it fresh, then diff if you had local edits:

```bash
mkdir -p .harness-backup
for f in docs/FEATURE_INTAKE.md docs/TOOL_REGISTRY.md docs/HARNESS_AUDIT.md docs/IMPROVEMENT_PROTOCOL.md; do
  [ -f "$f" ] && mv "$f" ".harness-backup/$(basename "$f").bak"
done
curl -fsSL "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.sh?$(date +%s)" \
  | bash -s -- --merge --yes            # recreates the moved docs at the new version
# review .harness-backup/*.bak vs the refreshed docs if you had customized them
```

The `--refresh-agent-shim` in step 1 already updates the marked
`<!-- HARNESS:BEGIN/END -->` block in `AGENTS.md` (and `--claude` the `CLAUDE.md`
block), preserving your project-specific instructions outside the markers.
