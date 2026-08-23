# US-038 Story Verify Fails On Unreviewed Change Diagrams

## Status

implemented

## Lane

normal

## Product Contract

`harness-cli story verify <id>` (and `story verify-all`) fails before running
the story's `verify_command` when any change diagram file
(`docs/**/diagrams/D<n>-*.md`) whose `Story:` header names the story has a
`Status:` other than `reviewed`. The failure is recorded as a
`story.verify_result` fail event and the stderr lists each offending file with
its status. This makes the `docs/DIAGRAMS.md` done gate mechanical instead of
prompt-enforced.

## Relevant Product Docs

- `docs/DIAGRAMS.md` — Mechanical Check section
- `docs/HARNESS.md` — Story Verification section

## Acceptance Criteria

- A story with a `draft` or `stale` diagram: `story verify` → `fail`, command
  not executed, `last_verified_result = fail`, stderr names the file.
- Same story after the diagram is `reviewed`: command runs normally.
- Stories without diagrams are unaffected; templates (`Story: US-XXX`) never gate.
- `verify-all` applies the same rule per story.

## Design Notes

- Commands: `story verify <id>`, `story verify-all` (behavior change; no new flags).
- Tables: none. Event: existing `story.verify_result`.
- Domain rules: `unreviewed_diagrams(repo_root, id)` in `infrastructure.rs`;
  scan is filesystem-only, no DB linkage.

## Change Diagrams

| Diagram | File | Status | Reviewed at |
| --- | --- | --- | --- |
| D3 sequence | `diagrams/D3-verify-gate.md` | reviewed (human:thanh-dong, 2026-08-23) | design review |

## References

- `crates/harness-cli/src/infrastructure.rs` `verify_story` — the pre-existing
  command runner this gate wraps.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | `cargo test -p harness-cli unreviewed_diagrams` |
| Integration | `cargo test -p harness-cli story_verify_fails_on_unreviewed_diagram` |
| E2E | live: `story verify US-038` fails while its own D3 is draft, passes after review |

## Harness Delta

`docs/DIAGRAMS.md` Mechanical Check and `docs/HARNESS.md` Story Verification
updated to state the gate. CLI patch version bump required (US-025 rule).

## Evidence

- `cargo test -p harness-cli` → 69 passed.
- Live gate run recorded in `implementation-notes.html`.
- After D3 review: `story verify US-038` → pass.
