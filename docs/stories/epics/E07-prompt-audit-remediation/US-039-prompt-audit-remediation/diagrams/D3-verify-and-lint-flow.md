# D3 Sequence — US-039 Prompt-audit remediation: verify and lint flow

Story: US-039
Kind: sequence
Source: hand
Scope: agent edits skill / shim / MCP template → okra-verify-artifact.py by hand on the fixture → story verify runs verify-harness.sh (check 9 reading blocks, check 10 lint-skills), lint-mcp-templates.sh, check-diagrams.sh
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
  participant V as okra-verify-artifact.py (manual)
  participant H as harness-cli story verify
  participant VH as verify-harness.sh
  participant M as lint-mcp-templates.sh
  participant D as check-diagrams.sh
  A->>S: edit SKILL.md / references
  A->>C: mirror edit byte-for-byte
  A->>X: edit contract tokens (no benchmark literal)
  A->>V: run by hand on the fixture artifact (Task 1 step, not part of story verify)
  alt every requirement present
    V-->>A: complete (exit 0)
  else token missing
    V-->>A: INCOMPLETE + missing ids (exit 1)
    Note over A,X: fix contract or skill text, never the fixture, unless the fixture disobeyed SKILL.md
  end
  A->>H: story verify US-039
  H->>VH: run verify-harness.sh .
  Note over VH: check 9 compares the Harness reading block in AGENTS.md and the two installers
  Note over VH: check 10 runs lint-skills.sh (mirror parity, contract sentences in prose, no grader vocabulary)
  alt all checks pass
    VH-->>H: SCORE 10/10 (exit 0)
  else drift
    VH-->>H: failing check name, e.g. skills lint (exit 1)
  end
  H->>M: run lint-mcp-templates.sh
  alt no generated view named as an edit target
    M-->>H: MCP templates lint clean (exit 0)
  else wording drift
    M-->>H: assertion 3 failure (exit 1)
  end
  H->>D: run check-diagrams.sh
  alt every diagram file well-formed
    D-->>H: N file(s) ok (exit 0)
  else bad header, status, or fence
    D-->>H: FAIL file: reason (exit 1)
  end
  H-->>A: pass / fail recorded as event
  Note over H: .harness/events appended; TEST_MATRIX regenerated
```

## What to review

- Does the happy path match the product contract (contract wins over prose, lints replace hand checks)?
- Is every error path drawn: verifier INCOMPLETE, lint drift, diagram lint failure, story verify fail?
- Is the nesting right: `lint-skills.sh` runs as `verify-harness.sh` check 10, not as a peer, and `okra-verify-artifact.py` is a manual Task 1 step that `story verify` never runs?
- Are all side effects shown: the `.codex` mirror write, the event append, the regenerated matrix?

## Derived next steps

- One check per `alt` branch: verifier complete on the fixture (Task 1), lint fails on a scratch divergence (Tasks 3 and 5), diagram lint ok (Task 8), story verify passes (Task 8).
- The lint scripts' exit codes and messages are the interface contract for `verify-harness.sh` and for the story's `--verify` command.
