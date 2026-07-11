# US-033 Headless / Non-Interactive Guarantees

## Status

planned

## Lane

normal

## Product Contract

Every command an unattended operator calls (`init`, `migrate`,
`tool register|check`, and all US-032 commands) runs with no prompts and no
TTY assumptions; where confirmation exists, a `--yes`/env equivalent is added.
The guarantee is documented.

Epic: E-shuttle-readiness. Hardens Shuttle US-MH-02 (provisioner runs inside
the server and on remote runners).

## Acceptance Criteria

- Audit of listed commands recorded (prompt/TTY findings + fixes).
- CI job runs the command set with stdin closed and no TTY; all pass.
- docs/SETUP.md notes the headless guarantee.

## Validation

| Layer | Expected proof |
| --- | --- |
| Integration | CI headless matrix over the command set |

## Harness Delta

Possible CLI flags; SETUP.md update.
