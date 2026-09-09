# D3 Sequence — US-039 Prompt-audit remediation: verify and lint flow

Story: US-039
Kind: sequence
Source: hand
Scope: agent edits skill / shim / MCP template → lints (lint-skills, verify-harness, lint-mcp-templates) → okra-verify-artifact → story verify
Status: draft
Reviewed-by: -
Reviewed-at: -

```mermaid
sequenceDiagram
  autonumber
  participant A as agent (implementer)
  participant S as .claude/skills/reverse-tornado-okr
  participant C as .codex mirror
  participant X as contracts/handoff-contract.v2.json
  participant V as okra-verify-artifact.py
  participant L as lint-skills.sh / verify-harness.sh / lint-mcp-templates.sh
  participant H as harness-cli story verify
  A->>S: edit SKILL.md / references
  A->>C: mirror edit byte-for-byte
  A->>X: edit contract tokens (no benchmark literal)
  A->>V: run on fixture artifact
  alt every requirement present
    V-->>A: complete (exit 0)
  else token missing
    V-->>A: INCOMPLETE + missing ids (exit 1)
    Note over A,X: fix contract or skill text, never the fixture, unless the fixture disobeyed SKILL.md
  end
  A->>L: run lints
  alt trees identical, sentences present, shim blocks equal, no generated-view edit targets
    L-->>A: ok
  else drift
    L-->>A: which pair differs (exit 1)
  end
  A->>H: story verify US-039 (runs all lints)
  H-->>A: pass / fail recorded as event
  Note over H: .harness/events appended; TEST_MATRIX regenerated
```

## What to review

- Does the happy path match the product contract (contract wins over prose, lints replace hand checks)?
- Is every error path drawn: verifier INCOMPLETE, lint drift, story verify fail?
- Are all side effects shown: the `.codex` mirror write, the event append, the regenerated matrix?

## Derived next steps

- One check per `alt` branch: verifier complete on the fixture (Task 1), lint fails on a scratch divergence (Tasks 3 and 5), story verify passes (Task 8).
- The lint scripts' exit codes and messages are the interface contract for `verify-harness.sh`.
