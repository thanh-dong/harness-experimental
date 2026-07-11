# 0010 — Shuttle Consumption Contract (E-shuttle-readiness)

## Status

accepted (2026-07-11)

## Context

Shuttle (a web UI for Claude Code/Codex) will auto-provision and operate this
harness for every repo registered in it, consuming this repo as a pinned
external dependency (Shuttle decision 0053). Shuttle explicitly does NOT own
this repo, and this repo stays agent- and host-generic (Codex, Cursor, plain
Claude Code) with a live upstream fork relationship.

## Decision

1. Support machine consumers through additive release/CLI changes — the
   E-shuttle-readiness epic: US-030 dist bundle, US-031 manifest.json,
   US-032 `--json` output, US-033 headless guarantees, US-034 install/upgrade
   contract, US-035 MCP-flavored templates, US-036 `info --json`.
2. Canonical state stays in each target repo (event log + docs); no feature
   in this epic may introduce a server-canonical or out-of-repo store.
3. Private-fork access: the supported private-consumption path is consumers
   vendoring the release bundle (US-030); making releases public remains an
   open follow-up, not a blocker.

## Consequences

- All changes are additive → the fork stays mergeable with upstream and
  benefits non-Shuttle consumers.
- US-030..US-032 gate Shuttle's v1 slice; ship them first in one tagged
  release.
- Gate wording gains a second (MCP) flavor with a single owner: this repo.
