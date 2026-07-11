# US-034 Programmatic Install/Upgrade Contract

## Status

planned

## Lane

normal

## Product Contract

A stable, deterministic install/upgrade entry point for machine consumers:
either `harness-cli install --from <bundle> --merge`, or the existing
`install-harness.sh` `HARNESS_SOURCE_BASE_URL`/`--merge`/`--dry-run` path
hardened as a contract — documented exit codes and a machine-readable summary
of files written/skipped. Detect-and-upgrade on repos that already have a
harness is part of the contract (never overwrites, moves, or deletes existing
stories/decisions/db — the `--merge` promise, now testable).

Epic: E-shuttle-readiness. Hardens Shuttle US-MH-02.

## Acceptance Criteria

- One documented entry point with exit codes + JSON summary.
- Fresh install, merge-upgrade, and already-current cases all deterministic
  and idempotent.
- `--dry-run` output matches what a real run then does.

## Validation

| Layer | Expected proof |
| --- | --- |
| Integration | temp-repo matrix: fresh / existing / re-run |

## Harness Delta

Installer or CLI subcommand; scripts/README.md contract section.
