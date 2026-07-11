# US-036 Version & Compatibility Introspection

## Status

planned

## Lane

normal

## Product Contract

`harness-cli info --json` reports CLI version, schema version, applied
migrations, and event-log format version, so an operator can decide
verify / migrate / refuse on an existing install without parsing files.

Epic: E-shuttle-readiness. Feeds Shuttle's provisioner decisions and /health.

## Acceptance Criteria

- `info --json` stable shape; works on uninitialized repos (reports absence).
- Distinguishes "cache behind log" (needs replay) from "schema behind CLI"
  (needs migrate).

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | info output across init states |

## Harness Delta

CLI subcommand; scripts/README.md.
