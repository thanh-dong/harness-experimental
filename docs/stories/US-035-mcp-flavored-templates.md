# US-035 MCP-Flavored Context Templates

## Status

planned

## Lane

normal

## Product Contract

An alternate gate/context template set where operating instructions reference
MCP tool names (`harness_intake`, `harness_story_add`, …) instead of bash
commands, alongside the existing bash-flavored set. Gate semantics have one
owner (this repo); MCP hosts like Shuttle generate session context from these
templates and never fork the wording.

Epic: E-shuttle-readiness. Unblocks Shuttle US-MH-04.

## Acceptance Criteria

- `docs/templates/` (or a variant dir) carries the MCP-flavored set.
- Tool-name ↔ CLI-command mapping table included and kept in sync.
- Bash-flavored set unchanged for CLI consumers.

## Validation

| Layer | Expected proof |
| --- | --- |
| Unit | template lint: every referenced tool exists in the mapping |

## Harness Delta

New templates; FEATURE_INTAKE/HARNESS docs mention the two flavors.
