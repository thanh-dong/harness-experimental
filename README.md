<div align="center">

# 🧰 repository-harness

### Turn any software repo into an agent-ready workspace.

*The app is what users touch. The harness is what agents touch.*

[![License: MIT](https://img.shields.io/badge/License-MIT-22c55e.svg)](LICENSE)
[![Harness CLI](https://img.shields.io/badge/Harness%20CLI-v0.1.10-3b82f6.svg)](scripts/README.md)
[![Built in Rust](https://img.shields.io/badge/CLI-Rust-orange.svg)](crates/harness-cli)
[![Platforms](https://img.shields.io/badge/Platforms-macOS%20%7C%20Linux%20%7C%20Windows-6366f1.svg)](#-installation)
[![Agents](https://img.shields.io/badge/Agents-Claude%20Code%20%C2%B7%20Codex%20%C2%B7%20Cursor-8b5cf6.svg)](#-installation)

[**What it is**](#-what-it-is) ·
[**Quickstart**](#-installation) ·
[**Agent self-setup**](#agent-self-setup) ·
[**From a clone**](#-run-from-a-clone) ·
[**The flow**](#-try-the-flow) ·
[**Acknowledgements**](#-acknowledgements)

</div>

> [!NOTE]
> This repository (`thanh-dong/harness-repository-cc`) is a **customized fork**,
> tuned for my own Claude Code projects. The original work is by
> [Hoang Nguyen (@hoangnb24)](https://github.com/hoangnb24/repository-harness) —
> see [Acknowledgements](#-acknowledgements).

---

## 💡 What It Is

`repository-harness` is a **repository-level operating harness** for Claude Code,
Codex, Cursor, and other coding agents. It gives agents the missing project
context they need *before* they change code: where to start, what the product
contract says, how risky the work is, what proof is required, and which decisions
future agents should inherit.

> **In short:** a drop-in set of Markdown docs plus a small Rust CLI that you
> install into any repository so coding agents have a consistent place to learn
> project intent, classify risk, record stories and decisions, and prove their
> work — before they edit code.

It ships **no application of its own**. It is the operating layer you add around
whatever you are building.

This project is exploring a simple idea:

> Coding agents do not only need better prompts. **They need better repositories.**

---

## 🎯 The Problem

Most repos are built for humans reading code in a familiar codebase. Coding
agents usually enter with only a chat prompt and a shallow snapshot of files.
That leads to common failure modes:

- 🔧 The agent edits code before understanding product intent.
- 🗣️ Important constraints live only in chat history or in someone's head.
- ❓ Validation expectations are vague or discovered too late.
- 🔁 Architecture tradeoffs are repeated instead of inherited.
- 📦 Large requests do not get broken into reviewable story-sized work.

---

## 🧭 The Harness Approach

A repository starts to have a harness when it helps an agent answer practical
engineering questions without relying only on chat history:

> *What should I read first? · What type of work is this? · Which product
> contract does it affect? · How risky is the change? · What proof shows the work
> is done? · What decision should future agents inherit?*

In this repo, those answers live in:

| Path | What it provides |
| --- | --- |
| `AGENTS.md` | The stable agent shim with local project notes and Harness doc links. |
| `docs/HARNESS.md` | The human–agent collaboration model. |
| `docs/FEATURE_INTAKE.md` | Tiny, normal, and high-risk work classification. |
| `docs/ARCHITECTURE.md` | Architecture discovery and boundary rules. |
| `docs/TEST_MATRIX.md` | Behavior-to-proof validation expectations. |
| `docs/stories/` | Story packets and backlog items. |
| `docs/decisions/` | Durable decisions and tradeoffs. |
| `docs/templates/` | Reusable spec, story, decision, and validation templates. |

OpenAI describes this shift as an agent-first world where humans steer and agents
execute → <https://openai.com/index/harness-engineering/>

---

## 🚀 Installation

Install the harness into a target project. From that project's directory, run:

```bash
curl -fsSL "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.sh?$(date +%s)" | bash -s -- --yes
```

This applies the Harness operating files (`AGENTS.md`, `docs/`, story and
decision templates) and downloads the prebuilt Harness CLI into
`scripts/bin/harness-cli`. Add `--claude` if the project is driven with Claude
Code so the harness context is auto-loaded into every session.

On **Windows PowerShell**, run:

```powershell
& ([scriptblock]::Create((irm "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.ps1"))) -Yes
```

### Agent Self-Setup

Hand this block to a coding agent to install, wire its tools, and verify the
harness on its own, from inside the target project directory. The full one-pass
playbook is [`docs/SETUP.md`](docs/SETUP.md).

```bash
# 1. Install the harness files and the prebuilt CLI (use --claude for Claude Code)
curl -fsSL "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.sh?$(date +%s)" | bash -s -- --claude --yes

# 2. Create the durable database
scripts/bin/harness-cli init

# 3. Wire the capability tools that are present (register block: docs/SETUP.md), then reconcile
scripts/bin/harness-cli tool check
scripts/bin/harness-cli query tools --summary

# 4. Verify the install: CLI runs and the test matrix is queryable
scripts/bin/harness-cli --version
scripts/bin/harness-cli query matrix

# 5. Read the entry points before changing code
#    AGENTS.md, docs/HARNESS.md, docs/FEATURE_INTAKE.md, docs/GOAL_LOOP.md
```

On Windows the CLI is called as `.\scripts\bin\harness-cli.exe`.

### For Agents: Fresh Install Or Update

> [!IMPORTANT]
> **This fork is private.** The anonymous `curl … raw.githubusercontent.com … | bash`
> one-liners return 404 without credentials. From a machine with `gh` auth, use the
> authenticated flow instead — it is verified end-to-end:
>
> ```bash
> gh repo clone thanh-dong/harness-repository-cc /tmp/harness
> gh release download "$(cat /tmp/harness/scripts/harness-cli-release-tag)" \
>   -R thanh-dong/harness-repository-cc -D /tmp/harness-dist
> HARNESS_CLI_BASE_URL="file:///tmp/harness-dist" \
>   /tmp/harness/scripts/install-harness.sh --claude --yes            # fresh install
> #  … --merge --refresh-agent-shim --claude --yes                    # update
> ```
>
> The curl one-liners below work as written once the repo is made public.

The one-pass playbook for both paths is [`docs/SETUP.md`](docs/SETUP.md). Pick the
branch that matches the target repo:

- **Fresh install** — no `harness.db` yet: run the [Agent Self-Setup](#agent-self-setup)
  block above, then `docs/SETUP.md` §_Wire the tools_ to register the equipped
  capability providers (impact-analysis, `goal-loop-orchestration`, verification),
  and §_Reconcile and verify_.
- **Update an existing install** — already has the harness: use `--merge`, which
  adds new files and **never overwrites, moves, or deletes anything you have** (your
  `docs/product/`, `docs/stories/`, `docs/decisions/`, and `harness.db` are safe).
  Do **not** use `--override` on a working repo — it moves your `docs/` into backup.

  ```bash
  curl -fsSL "https://raw.githubusercontent.com/thanh-dong/harness-repository-cc/main/scripts/install-harness.sh?$(date +%s)" \
    | bash -s -- --merge --refresh-agent-shim --claude --yes
  scripts/bin/harness-cli migrate     # additive, idempotent, data-preserving
  scripts/bin/harness-cli tool check && scripts/calibrate-harness.sh
  ```

  Refreshing already-present policy docs and the full notes are in
  `docs/SETUP.md` §_Updating an existing install_.

- **Goal-loop skill (Okra)** — the `goal-loop-orchestration` provider is the Okra
  `reverse-tornado-okr` skill; install it (plugin or project-local) per
  `docs/SETUP.md` §_Install the goal-loop skill (Okra)_ so the capability resolves.

### Install Options

| Goal | Flag | Command |
| --- | --- | --- |
| Update an existing Harness repo (keep existing files) | `--merge` | `… install-harness.sh … \| bash -s -- --merge --yes` |
| Back up and replace `AGENTS.md`, `docs/`, `scripts/` | `--override` | `… install-harness.sh … \| bash -s -- --override --yes` |
| Refresh an old full `AGENTS.md` into the small shim | `--refresh-agent-shim` | `… \| bash -s -- --merge --refresh-agent-shim --yes` |
| Auto-load harness context in Claude Code | `--claude` | `… \| bash -s -- --claude --yes` |
| Install into a specific path | `--directory <path>` | `… \| bash -s -- --directory /path/to/project --yes` |
| Preview without writing | `--dry-run` / `-DryRun` | `… \| bash -s -- --dry-run` |

PowerShell uses the equivalent switches: `-Merge`, `-Override`,
`-RefreshAgentShim`, `-Yes`, `-Directory`, `-DryRun`.

<details>
<summary><strong>Flag details</strong></summary>

- **`--merge`** — append newly added Harness files without moving the existing
  `AGENTS.md`, `docs/`, or `scripts/` paths into backup. Existing files stay
  untouched; only missing Harness files are created.
- **`--refresh-agent-shim`** — for older installs whose `AGENTS.md` still
  contains the full generated operating guide. The refresh backs up the existing
  file. If it detects the old Harness-generated guide it replaces it with the
  shim; if the file appears custom it appends or updates a marked Harness block
  instead of overwriting the project's local instructions.
- **`--claude`** — Claude Code never auto-loads `AGENTS.md`, so without this the
  installed harness is invisible to fresh sessions. The flag installs (or
  refreshes) a `CLAUDE.md` whose marked Harness block `@`-imports `AGENTS.md` and
  `docs/FEATURE_INTAKE.md` into every session's context. An existing `CLAUDE.md`
  gets the block appended after a backup; plain installs without the flag never
  touch `CLAUDE.md`.

</details>

### About The Prebuilt CLI

The installer downloads the prebuilt Harness CLI for the current platform,
verifies its `.sha256` checksum, and installs it at `scripts/bin/harness-cli`
(macOS/Linux) or `scripts/bin/harness-cli.exe` (Windows). The Rust CLI is the
main Harness tool and stable command path.

Release assets are published from tags by the `Harness CLI Release` GitHub
Actions workflow, with `harness-cli-<platform>` and `harness-cli-<platform>.sha256`
assets for macOS arm64, macOS x64, Linux x64, Linux arm64, and Windows x64 (the
Windows asset is `harness-cli-windows-x64.exe` plus its `.sha256`).

Merged PRs are recorded in `CHANGELOG.md` by the `Post-Merge Maintenance`
workflow; when a merged PR changes the Rust CLI source, schema, Cargo metadata,
or CLI release packaging, that workflow bumps the CLI patch version, updates
`scripts/harness-cli-release-tag`, creates a `harness-cli-v*` tag, and runs the
release build for that tag.

---

## 🛠️ Run From A Clone

To work on the harness itself, or to use the CLI and installer directly from a
checkout instead of the remote one-liner:

```bash
git clone https://github.com/thanh-dong/harness-repository-cc.git
cd harness-repository-cc
```

**Use the prebuilt CLI** that ships in the repo:

```bash
scripts/bin/harness-cli --version      # macOS/Linux
scripts/bin/harness-cli init
scripts/bin/harness-cli query matrix
```

**Or build the CLI from source** (requires the Rust toolchain):

```bash
cargo build --release -p harness-cli
./target/release/harness-cli --version
```

**Run the installer from the local checkout** against a target project, sourcing
the harness files from this clone rather than GitHub:

```bash
HARNESS_SOURCE_BASE_URL="file://$(pwd)" \
  scripts/install-harness.sh --directory /path/to/project --merge --yes
```

Pass `--dry-run` to preview the file changes first. `HARNESS_SOURCE_BASE_URL`
overrides where harness files are fetched from; `HARNESS_CLI_BASE_URL` overrides
where the prebuilt CLI is downloaded from (for example a local `dist/` directory
produced by `scripts/build-harness-cli-release.sh`).

> See [`scripts/README.md`](scripts/README.md) for the full CLI command reference
> and release packaging.

---

## 🔄 Try The Flow

The fastest way to understand the harness is to inspect the tiny demo at
[`docs/demo/README.md`](docs/demo/README.md): it shows how a simple product idea
becomes product docs, stories, validation expectations, and decisions before
implementation starts.

A typical flow looks like this:

```text
human intent or product spec
  ➜ product contract
  ➜ feature intake
  ➜ story packet
  ➜ validation expectations
  ➜ implementation work
  ➜ decision or lesson captured for future agents
```

Implementation prompts do not go straight to code. They first pass through
feature intake, become story-sized work when needed, and then carry both product
validation and harness maintenance expectations.

---

## 🔌 Tool Registry

The harness can use optional external tools (linters, code-graph servers, deploy
checks) **without depending on any of them**. You register a tool as a provider
of a *capability*, the harness scans whether it is actually present, and a
workflow step uses whatever is equipped — an absent tool is a clean skip, never a
failure.

```bash
# register a tool as a provider of a capability
scripts/bin/harness-cli tool register --name deploy-check --kind cli \
  --capability deploy-verification --command ./scripts/deploy-check.sh \
  --responsibility Verification --description "Verify deploy health before release"

# scan presence (writes present/missing/unknown)
scripts/bin/harness-cli tool check

# a step looks up what is equipped for a purpose
scripts/bin/harness-cli query tools --capability deploy-verification --status present
```

Kinds (`cli`, `binary`, `mcp`, `skill`, `http`) make it agent-generic: each agent
runtime uses what it can orchestrate. See
[`docs/TOOL_REGISTRY.md`](docs/TOOL_REGISTRY.md) for the full model, the degrade
ladder, and how to wire a tool into a flow step.

---

## 📍 Current State

This repository is in **Harness v0**.

There is no application implementation and no baked-in product specification yet.
The current work is the reusable project harness: the file structure, agent
operating model, feature intake process, story templates, and validation
expectations that help humans and agents turn a future user-provided spec into
implementation work.

### Product Sources

No product contract is currently defined. When a project specification is
provided, add or reference it as the input spec for the first buildout, then
derive smaller living artifacts from it:

- `docs/product/` — current product contract files, created from the spec.
- `docs/stories/` — story packets and backlog created from selected work.
- `docs/TEST_MATRIX.md` — behavior-to-proof control panel.
- `docs/decisions/` — durable decisions and tradeoffs.

Do not keep a project-specific spec or product breakdown in this harness until a
real project supplies one.

---

## 📂 Repository Structure

```text
project/
├── AGENTS.md
├── README.md
├── docs/
│   ├── HARNESS.md
│   ├── FEATURE_INTAKE.md
│   ├── ARCHITECTURE.md
│   ├── TEST_MATRIX.md
│   ├── HARNESS_BACKLOG.md
│   ├── product/
│   ├── stories/
│   ├── decisions/
│   ├── demo/
│   └── templates/
└── scripts/
    └── README.md
```

---

## 🤝 Contributing

This project is early and benefits most from real-world agent failure cases,
example harness installs, docs improvements, and reusable workflow patterns. See
[`CONTRIBUTING.md`](CONTRIBUTING.md) for contribution ideas.

Useful contributions include:

- Show how the harness works in a real project.
- Add missing templates or improve existing ones.
- Propose validation patterns for different stacks.
- Share failures where an agent made the wrong change because the repo lacked context.
- Compare harness behavior across Claude Code, Codex, Cursor, and other tools.

---

## 🙏 Acknowledgements

Huge thanks to **[Hoang Nguyen (@hoangnb24)](https://github.com/hoangnb24)**, the
original author and creator of `repository-harness`
([upstream repo](https://github.com/hoangnb24/repository-harness)). The harness
model, the operating docs, and the Rust CLI are all his work; this fork only
adapts them for my own use. The project is MIT-licensed — see [`LICENSE`](LICENSE)
(Copyright © 2025 Hoang Nguyen).

If you find this useful, please go ⭐ the
[upstream repository](https://github.com/hoangnb24/repository-harness).

<div align="center">

---

> *An agent-ready repo harness for Claude Code, Codex, Cursor, and other coding
> agents: `AGENTS.md`, product contracts, story packets, validation matrix, and
> decision records.*

**If this idea resonates, please star the repo and share it with someone building with coding agents.**

</div>
