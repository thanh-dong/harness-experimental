# Scripts

This directory contains harness automation tools.

## Harness CLI

The Rust Harness CLI is the primary interface for the durable layer. Installed
projects use the prebuilt binary at `scripts/bin/harness-cli` on macOS/Linux or
`scripts/bin/harness-cli.exe` on Windows for normal Harness work.

```bash
scripts/bin/harness-cli init          # Create the database
scripts/bin/harness-cli intake ...    # Record a feature intake classification
scripts/bin/harness-cli story ...     # Add or update a story (test matrix row)
scripts/bin/harness-cli story update --id US-001 --unit 1 --integration 1 --e2e 0 --platform 0
scripts/bin/harness-cli story verify US-001  # Run the story's verify_command
scripts/bin/harness-cli decision ...  # Add a decision or run its verification
scripts/bin/harness-cli backlog ...   # Add or close a backlog item
scripts/bin/harness-cli trace ...     # Record and auto-score an agent execution trace
scripts/bin/harness-cli score-trace   # Score a trace against TRACE_SPEC.md tiers
scripts/bin/harness-cli query ...     # Query harness data, including backlog --open/--closed
scripts/bin/harness-cli query matrix --numeric  # Show proof flags as 1/0
scripts/bin/harness-cli migrate       # Apply pending schema migrations
scripts/bin/harness-cli --version     # Print the installed CLI version
```

Run `scripts/bin/harness-cli help` or `scripts/bin/harness-cli query help` for
full usage. On Windows, use the same commands through
`.\scripts\bin\harness-cli.exe`.

Proof flags on `story update` are numeric booleans: use `1` for yes and `0` for
no. `story verify <id>` runs the configured `verify_command`; it does not accept
proof flags. Configure the command with `story add/update --verify`, run
`story verify <id>`, then update proof flags with `story update`.

Backlog `--risk` uses Harness lanes, not severity words: use `tiny`, `normal`,
or `high-risk`. Use `tiny` instead of `low`. `query matrix` defaults to
human-readable `yes`/`no`; use `query matrix --numeric` when copying values into
`story update`.

The schema lives in `scripts/schema/` and is version-controlled. The database
file (`harness.db`) is `.gitignore`d.

Requires: the prebuilt Rust CLI at `scripts/bin/harness-cli` on macOS/Linux or
`scripts/bin/harness-cli.exe` on Windows.

Direct database inspection may still use SQLite tools, but normal Harness use
should go through the Rust CLI.

### Rust CLI Commands

Current migrated commands:

```bash
scripts/bin/harness-cli init
scripts/bin/harness-cli migrate
scripts/bin/harness-cli info
scripts/bin/harness-cli import brownfield
scripts/bin/harness-cli intake ...
scripts/bin/harness-cli story add ...
scripts/bin/harness-cli story update ...
scripts/bin/harness-cli story verify ...
scripts/bin/harness-cli decision add ...
scripts/bin/harness-cli decision verify ...
scripts/bin/harness-cli backlog add ...
scripts/bin/harness-cli backlog close ...
scripts/bin/harness-cli trace ...
scripts/bin/harness-cli score-trace
scripts/bin/harness-cli query matrix
scripts/bin/harness-cli query backlog
scripts/bin/harness-cli query decisions
scripts/bin/harness-cli query intakes
scripts/bin/harness-cli query traces
scripts/bin/harness-cli query friction
scripts/bin/harness-cli query stats
scripts/bin/harness-cli query sql ...
```

`scripts/bin/harness-cli import brownfield` seeds or refreshes the durable database
from existing Harness v0 markdown in `docs/TEST_MATRIX.md`,
`docs/decisions/`, and `docs/HARNESS_BACKLOG.md`. This keeps already-installed
Harness repos on the Rust CLI path without losing their populated operating
docs.

### JSON output mode

Machine consumers (e.g. the Shuttle MCP wrapper) can request a single
machine-readable JSON object per invocation instead of human-formatted text.
Enable it with the global `--json` flag or by setting `HARNESS_OUTPUT=json`;
either turns JSON on. Human output is unchanged by default, so existing flows
and docs keep working.

```bash
scripts/bin/harness-cli intake --type change_request --summary "..." --lane normal --json
HARNESS_OUTPUT=json scripts/bin/harness-cli query matrix
```

Supported on `intake`, `story add|update|signal`, `decision add`, `trace`,
`backlog add`, every `query *` view, `tool check`, and `info`.

`info` (US-036) reports version and state so a machine consumer can decide
verify / migrate / refuse without parsing files. It works on an uninitialized
repo (reports absence, exits `0`). `--json` emits, under `"data"`: `cliVersion`,
`supportedSchemaVersion`, `availableSchemaVersion`, `eventFormatVersion`,
`initialized`, `dbPath`, `appliedSchemaVersion`, `appliedMigrations`,
`eventBacked`, `eventFiles`, and two action flags — `schemaBehindCli` (the
applied schema is behind the migrations on disk → run `migrate`) and
`cacheBehindLog` (the log holds events the cache has not consumed → any command
replays them).

```json
{"ok":true,"command":"info","data":{"cliVersion":"0.1.13","appliedSchemaVersion":8,"schemaBehindCli":false,"cacheBehindLog":false, "...":"..."}}
```

Every command an unattended operator calls runs headless — no prompts, no TTY
assumptions — with stdin closed and no controlling terminal (US-033). The
`headless-matrix` CI job and `scripts/test-headless.sh` enforce this over the
command set.

Success emits `{"ok":true,"command":"<name>", ...}`. Write commands add the
affected `"id"`; `query *` views wrap their rows under `"data"`:

```json
{"ok":true,"command":"backlog.add","id":"01K..."}
{"ok":true,"command":"query.matrix","data":[{"id":"US-1","status":"planned","unit":true}]}
```

Failure emits `{"ok":false,"error":{"code":"<validation|io>","message":"..."}}`.

Two pre-existing outputs keep their established bare-array shape for backward
compatibility (US-019): `query tools --json` and `tool check --json` still emit a
top-level JSON array, not the envelope. The global `--json` flag and
`HARNESS_OUTPUT=json` both select that same array.

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success. |
| `2` | Validation / bad input / not-found — user-correctable (unknown lane, missing story, empty SQL, tool already exists, unsupported schema version, etc.). |
| `3` | IO or database failure (SQLite error, filesystem IO, corrupt event log, migration verification failure). |

The same mapping applies in human and JSON mode; in JSON mode the `error.code`
field is `"validation"` for exit `2` and `"io"` for exit `3`. Argument-parsing
errors from `clap` (unknown flag, missing required argument) exit with clap's own
code and print to stderr.

## Installer

The upstream installer applies the Harness v0 operating files and folder
structure to a target project directory. It defaults to the current directory,
accepts a target path, and asks interactive users whether to `1. Merge`,
`2. Override`, or `3. Stop` when the target already contains `AGENTS.md`,
`docs/`, or `scripts/`.
Non-interactive installs stop on those protected paths unless `--merge` or
`--override` is provided. Use `--merge` as the safe update path for repositories
that already have Harness: it keeps existing files in place and creates only
missing Harness files. Add `--refresh-agent-shim` when an older install has the
full generated Harness guide in `AGENTS.md` and should move to the small stable
shim. Use `--override` only when replacing the protected Harness surface is
intentional.

```bash
curl -fsSL "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.sh?$(date +%s)" | bash -s -- --yes
```

```powershell
& ([scriptblock]::Create((irm "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.ps1"))) -Yes
```

```bash
curl -fsSL "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.sh?$(date +%s)" | bash -s -- --merge --yes
```

```powershell
& ([scriptblock]::Create((irm "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.ps1"))) -Merge -Yes
```

```bash
curl -fsSL "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.sh?$(date +%s)" | bash -s -- --merge --refresh-agent-shim --yes
```

```powershell
& ([scriptblock]::Create((irm "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.ps1"))) -Merge -RefreshAgentShim -Yes
```

`--refresh-agent-shim` backs up `AGENTS.md` before changing it. If the existing
file is recognized as the old Harness-generated operating guide, the installer
replaces it with the current shim. Otherwise it appends or replaces only the
marked `<!-- HARNESS:BEGIN -->` block so project-specific instructions remain
in place.

The installer must stay limited to harness files. Do not use it to scaffold
application source folders, package scripts, CI, tests, platform shells, or fake
validation commands. The installer script is not part of the installed project
payload.

### Machine install/upgrade contract

For non-interactive / programmatic consumers (e.g. Shuttle) the installer is a
stable contract. Drive it with `--yes` plus, as needed,
`HARNESS_SOURCE_BASE_URL`, `--merge`, `--dry-run`, `--directory`, and
`--summary-json`.

**Exit codes** (same convention as the Harness CLI, US-032):

| Code | Meaning |
| --- | --- |
| `0` | Success. |
| `2` | Validation / bad input — user-correctable: unknown option, missing option value, bad target path, unsupported CLI platform, or a protected-path conflict (`AGENTS.md`/`docs/`/`scripts/` already present) in a non-interactive run without `--merge` or `--override`. |
| `3` | IO / download failure: `curl` download failed, checksum mismatch or empty checksum file, missing local source file, target directory not writable or not creatable, or a required tool (`curl`/`shasum`) is absent. |

**Detect-and-upgrade.** The three consumer cases are deterministic and
idempotent:

- **Fresh install** — empty target: every harness file is created.
- **Merge-upgrade** (`--merge`) over an existing harness: existing files are
  kept in place and only missing harness files are created. `--merge` never
  overwrites, moves, or deletes existing stories, decisions, or `harness.db`.
- **Already-current** — re-running `--merge` produces the same result (0
  created, 0 updated); the run is a no-op you can repeat safely.

`--dry-run` reports exactly the actions a real run then performs; its
`createdFiles` set equals the real run's.

**`--summary-json <path>`** writes one machine-readable JSON object describing
the run (use `-` for stdout). It is emitted only on success (exit 0); on
failure the exit code is the machine signal. Schema:

| Field | Type | Meaning |
| --- | --- | --- |
| `ok` | boolean | Always `true` (only emitted on success). |
| `target` | string | Absolute target directory. |
| `sourceMode` | string | `local` (repo checkout) or `remote` (download). |
| `dryRun` | boolean | Whether this was a `--dry-run`. |
| `conflictAction` | string | `install`, `merge`, or `override`. |
| `created` / `updated` / `skipped` | number | Per-category file counts. |
| `createdFiles` / `updatedFiles` / `skippedFiles` | string[] | Repo-relative paths per category. |

```bash
scripts/install-harness.sh --yes --merge --summary-json /tmp/harness-install.json /path/to/project
# {"ok":true,"target":"/path/to/project","sourceMode":"local","dryRun":false,
#  "conflictAction":"merge","created":0,"updated":0,"skipped":50,
#  "createdFiles":[],"updatedFiles":[],"skippedFiles":["AGENTS.md", ...]}
```

`scripts/test-install-contract.sh` exercises this contract on throwaway
`mktemp` dirs — fresh / merge-over-existing / re-run idempotent / dry-run
parity, asserting the exit codes and JSON summary shape. It is offline
(local source mode + a fake `file://` CLI artifact) and CI-ready:

```bash
scripts/test-install-contract.sh
```

By default the installer also downloads the prebuilt Rust Harness CLI for the
current platform into `scripts/bin/harness-cli` on macOS/Linux or
`scripts/bin/harness-cli.exe` on Windows, then verifies its `.sha256` checksum.
A source branch can pin the release used by the installer through
`scripts/harness-cli-release-tag`; Phase 3 pins `harness-cli-v0.1.4` so branch
installs receive a Phase 3-built CLI. Set `HARNESS_CLI_RELEASE_TAG` to override
that tag, or set `HARNESS_CLI_BASE_URL` to point at an alternate artifact
directory, such as a local `file:///.../dist` directory created by
`scripts/build-harness-cli-release.sh`.

## Schema Migrations

Migration files live under `scripts/schema/` and are named `NNN-description.sql`
where `NNN` is a zero-padded version number. Run `scripts/bin/harness-cli migrate` to
apply pending migrations.

## Future Command Contract

Expected future checks:

```text
validate:quick
  format, lint, typecheck, unit tests, architecture check

test:integration
  backend contract and integration checks

test:e2e
  user-visible end-to-end flows

test:platform
  platform shell smoke checks, if the project has a native shell

test:release
  full suite, log checks, and performance smoke
```

## Release Packaging

Build the current-platform Rust CLI release artifact from the source repo:

```bash
scripts/build-harness-cli-release.sh
```

The script writes `dist/harness-cli-<platform>` plus `.sha256` checksums. The
Windows artifact includes the `.exe` suffix. Supported labels are:

- `macos-arm64`
- `macos-x64`
- `linux-x64`
- `linux-arm64`
- `windows-x64`

For cross-compilation, pass a Cargo target triple:

```bash
scripts/build-harness-cli-release.sh --target x86_64-unknown-linux-gnu
```

GitHub releases are produced by
`.github/workflows/harness-cli-release.yml`. Push a tag matching `v*` or
`harness-cli-v*` to run the verification job, build all supported targets on
native hosted runners, and upload these release assets:

- `harness-cli-macos-arm64`
- `harness-cli-macos-arm64.sha256`
- `harness-cli-macos-x64`
- `harness-cli-macos-x64.sha256`
- `harness-cli-linux-x64`
- `harness-cli-linux-x64.sha256`
- `harness-cli-linux-arm64`
- `harness-cli-linux-arm64.sha256`
- `harness-cli-windows-x64.exe`
- `harness-cli-windows-x64.exe.sha256`

Merged PRs are handled by `.github/workflows/post-merge-maintenance.yml`. The
workflow always prepends a PR summary to `CHANGELOG.md`. If the merged PR
changed `crates/harness-cli/`, `scripts/schema/`, Cargo metadata, or
`scripts/build-harness-cli-release.sh`, it also increments the CLI patch
version, updates `scripts/harness-cli-release-tag`, creates a matching
`harness-cli-v*` tag, and calls the reusable Harness CLI release workflow for
the tagged ref.

## Distribution Bundle

`scripts/build-harness-dist.sh` packages the harness operating files —
everything `install-harness.sh` fetches from the repo, minus the per-platform
CLI binary — into one checksummed artifact:

- `harness-dist-<version>.tar.gz`
- `harness-dist-<version>.tar.gz.sha256`

The bundle contents are derived directly from the file list embedded in
`install-harness.sh`, so bundle-vs-installer parity holds by construction.
`--check` re-verifies that parity (and the checksum) and is the CI gate; the
release workflow's `verify` job runs it on every tagged build.

```bash
scripts/build-harness-dist.sh --version v0.1.13   # build dist/harness-dist-v0.1.13.tar.gz
scripts/build-harness-dist.sh --version v0.1.13 --check
scripts/build-harness-dist.sh --list              # print the packaged file list
```

The bundle mirrors repo-relative paths, so a consumer can install fully
offline by extracting it and pointing the installer at it with `file://`
URLs:

```bash
tar -xzf harness-dist-v0.1.13.tar.gz -C /tmp/harness-src
HARNESS_SOURCE_BASE_URL="file:///tmp/harness-src" \
  HARNESS_CLI_BASE_URL="file:///path/to/binaries" \
  scripts/install-harness.sh --yes /path/to/target
```

`HARNESS_CLI_BASE_URL` must point at a directory holding the matching
`harness-cli-<platform>` binary and its `.sha256`.

## Release Manifest

`scripts/build-manifest.sh` generates `manifest.json` for a release from a
directory of built assets (the per-platform binaries plus the dist bundle,
each with its `.sha256`). It is generated by the release workflow's `publish`
job and **never hand-edited**. A consumer resolves "what do I download, and is
it intact" programmatically from this one file.

```bash
scripts/build-manifest.sh --version v0.1.13 --dist-dir dist --notes "release notes"
scripts/build-manifest.sh --version v0.1.13 --dist-dir dist --check
```

The `--check` mode re-hashes every asset on disk and fails if it disagrees
with the manifest; the workflow runs it after generation as the CI
consistency gate.

### Schema

Stable, machine-readable field names (`manifest.json`):

| Field | Type | Meaning |
| --- | --- | --- |
| `schemaVersion` | number | Version of this manifest format (currently `1`). |
| `harnessVersion` | string | Release version, e.g. `v0.1.13`. |
| `generatedAt` | string | UTC ISO-8601 timestamp of generation. |
| `schema.version` | number | Highest schema migration number shipped. |
| `schema.count` | number | Number of schema migrations shipped. |
| `schema.migrations` | string[] | Shipped migration filenames, sorted. |
| `binaries[].platform` | string | Platform label, e.g. `macos-arm64`, `windows-x64`. |
| `binaries[].name` | string | Binary asset filename. |
| `binaries[].sha256` | string | SHA-256 of the binary asset. |
| `bundle.name` | string | Distribution bundle filename. |
| `bundle.sha256` | string | SHA-256 of the distribution bundle. |
| `migrationNotes` | string | Free-text release/migration notes (may be empty). |

Example:

```json
{
  "schemaVersion": 1,
  "harnessVersion": "v0.1.13",
  "generatedAt": "2026-07-11T08:49:34Z",
  "schema": { "version": 7, "count": 7, "migrations": ["001-init.sql", "..."] },
  "binaries": [
    { "platform": "macos-arm64", "name": "harness-cli-macos-arm64", "sha256": "..." }
  ],
  "bundle": { "name": "harness-dist-v0.1.13.tar.gz", "sha256": "..." },
  "migrationNotes": "Harness CLI release harness-cli-v0.1.13"
}
```

`schema.*` is derived from the same installer file list the bundle is built
from, so the manifest and the bundle always describe the same shipped schema.
