# US-030 Distribution Bundle Per Release

## Status

planned

## Lane

normal

## Product Contract

Every CLI release additionally publishes one `harness-dist-<version>.tar.gz`
asset containing the harness operating files (AGENTS.md shim, docs/ skeleton,
templates, schema) alongside the existing per-platform binaries — everything
the installer fetches, in a single checksummed artifact.

Epic: E-shuttle-readiness (Shuttle consumes the harness as a pinned
dependency; see decision 0010). Unblocks Shuttle US-MH-01.

## Acceptance Criteria

- Release workflow builds and attaches the bundle + `.sha256`.
- Bundle contents == what `install-harness.sh` would install (parity check).
- A consumer can install fully offline from the bundle.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | bundle packaging script parity test |
| Integration | offline install from bundle on a temp repo |

## Harness Delta

Release workflow (`build-harness-cli-release.sh`, GH Actions) gains a bundle step.
