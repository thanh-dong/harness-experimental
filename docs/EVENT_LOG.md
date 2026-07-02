# Event Log: Install, Migrate, Collaborate

The durable harness layer is a **git-tracked, append-only event log**
(`.harness/events/`). `harness.db` is a disposable SQLite cache,
deterministically rebuilt from that log. This is what lets a team share story
proof, decisions, backlog outcomes, and telemetry through normal git flow with
no binary merge conflicts (decision `0009`, superseding `0004`).

The rule, one line: **the log is the source of truth; `harness.db` and the
markdown views are generated.**

```text
.harness/
  events/
    <writer-id>.jsonl     # TRACKED IN GIT — team state, append-only
  backup/                 # gitignored — pre-migration DB + archived logs
  shadow.db               # gitignored — scratch rebuild target
harness.db                # gitignored — materialized cache, rebuilt from the log
docs/TEST_MATRIX.md       # generated view — do not hand-edit
docs/HARNESS_BACKLOG.md   # generated view — do not hand-edit
docs/decisions/README.md  # generated index — do not hand-edit
```

## Install (fresh repo)

```bash
# 1. Install the harness (see docs/SETUP.md for the private-fork/gh flow).
curl -fsSL "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.sh?$(date +%s)" \
  | bash -s -- --claude --yes

# 2. Create the durable layer. A fresh database is event-backed from genesis:
#    its (empty) log is the source of truth from the first write.
scripts/bin/harness-cli init

# 3. Commit the event log directory so teammates inherit it.
git add .harness/events .gitignore .gitattributes && git commit -m "chore: init harness event log"
```

`init` leaves you event-backed immediately — no migration step is needed on a
new repo. `.gitignore` already ignores `harness.db*`, `.harness/shadow.db`,
`.harness/backup/`; `.gitattributes` marks `.harness/events/*.jsonl`
`linguist-generated` so chatty trace lines stay out of human diff review.

## Migrate (existing repo on the SQLite-only layer)

If the repo predates the event log (schema ≤ v6, no `.harness/events/`), run the
one-time cutover. It is **safe and proven**: it verifies per-table row-count and
content-hash equality against the live DB before it writes anything, and backs
up the old DB without deleting it.

```bash
# 1. Apply schema migrations (adds ULID ids + cache tables; schema v7).
#    migrate-to-events REQUIRES current schema — it refuses otherwise.
scripts/bin/harness-cli migrate

# 2. Migrate the durable data into the event log.
scripts/bin/harness-cli migrate-to-events

# 3. Verify and commit.
scripts/bin/harness-cli rebuild            # deterministic replay; prints dump hash
scripts/bin/harness-cli query matrix       # confirm state matches the old DB
git add .harness/events docs/TEST_MATRIX.md docs/HARNESS_BACKLOG.md docs/decisions/README.md
git commit -m "chore: cut harness durable layer over to the event log"
```

What `migrate-to-events` does, in order:

1. Refuses if the cache is already event-backed (idempotent no-op on re-run).
2. Synthesizes one genesis event per durable row — `writer = migration`,
   original timestamps preserved, original ids kept in `payload.id`,
   deterministic event ids (re-runs are byte-identical).
3. **Proves before installing**: rebuilds a temp cache from the candidate
   events and asserts per-table count + content-hash equality against the live
   DB. The `tool` table is compared *excluding* the machine-local scan columns
   (`status`, `checked_at`), which are never logged. Any mismatch aborts.
4. Installs: backs up `harness.db` → `.harness/backup/harness.db.pre-migration`,
   archives any pre-cutover shadow logs → `.harness/backup/pre-migration-events/`,
   writes `.harness/events/migration.jsonl`, marks the cache event-backed.

If the proof fails, nothing is installed and the error names the cause. The most
common cause is skipping step 1 (`migrate`): the error will say the schema is
behind. `import brownfield` is now a one-time seed only (decision `0008` Q4); it
refuses on a populated event-backed cache.

## How the read/write paths behave

- **Read** (any query command): compares per-writer-file watermarks (a `stat`
  size+mtime short-circuit), incrementally replays any new events — e.g. after a
  `git pull` — then answers. A missing `harness.db` (fresh clone) triggers a
  full rebuild automatically. You never run `rebuild` by hand for correctness;
  it exists for explicit proof and troubleshooting.
- **Write** (any mutation): validates, appends the event to *your* writer file
  with `fsync`, applies the same event to the cache in one transaction, updates
  the watermark. If the append fails, the command fails and the cache rolls
  back — cache and log cannot drift.

## Command reference (event-log commands)

| Command | What it does |
| --- | --- |
| `harness-cli init` | Create the durable layer; event-backed from genesis. |
| `harness-cli migrate` | Apply schema migrations (needed before `migrate-to-events`). |
| `harness-cli migrate-to-events` | One-time cutover of a pre-event DB, with an equality proof. |
| `harness-cli rebuild [--output <path>]` | Deterministic replay from the log; prints per-table counts + dump hash. |
| `harness-cli query matrix` | Read current state (auto-replays new events first). |

## Recovery

- **Cache looks wrong / after a messy merge**: `harness-cli rebuild` (or just
  delete `harness.db` and run any query — it auto-rebuilds from the log).
- **Rolled back a migration**: the pre-migration DB is at
  `.harness/backup/harness.db.pre-migration`; nothing was deleted.
- **Determinism check**: two `rebuild` runs must print the same dump hash.
