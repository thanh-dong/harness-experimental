# US-XXX Story Title

## Status

planned

Valid durable statuses: `planned`, `in_progress`, `implemented`, `changed`,
`retired`. Keep this section and `story update --status` in agreement.

## Lane

tiny | normal | high-risk

## Product Contract

Describe the behavior this story must make true.

## Relevant Product Docs

- `docs/product/...`

## Acceptance Criteria

- Criterion 1.
- Criterion 2.
- Criterion 3.

## Design Notes

- Commands:
- Queries:
- API:
- Tables:
- Domain rules:
- UI surfaces:

## Change Diagrams

Required by lane and flags (`docs/DIAGRAMS.md`); files live in
`<packet>/diagrams/`. List each with its status, or write `none required`.

| Diagram | File | Status | Reviewed at |
| --- | --- | --- | --- |
| D1 blast radius | `diagrams/D1-....md` | draft | intake checkpoint |
| D3 sequence | `diagrams/D3-....md` | draft | design review |

## References

Source code that already implements the wanted behavior or semantics — the best
spec is a pointer. List paths (in-repo or vendored, any language) and what to
match in each.

- `path/to/reference` — what to match.

## Validation

When updating durable proof status, use numeric booleans:
`scripts/bin/harness-cli story update --id <id> --unit 1 --integration 1 --e2e 0 --platform 0`.

| Layer | Expected proof |
| --- | --- |
| Unit | |
| Integration | |
| E2E | |
| Platform | |
| Release | |

## Harness Delta

Document any harness updates made or proposed because of this story.

## Evidence

Add commands, reports, screenshots, or links after validation exists.
