# D3 Sequence — US-037 Change diagrams: review flow

Story: US-037
Kind: sequence
Source: hand
Scope: intake flags → diagram files → human review → intervention record → done gate
Status: draft
Reviewed-by: -
Reviewed-at: -

```mermaid
sequenceDiagram
  autonumber
  participant A as agent
  participant P as packet/diagrams/
  participant H as human reviewer
  participant CLI as harness-cli intervention add
  participant L as scripts/check-diagrams.sh
  A->>A: intake flags select D<n> set (DIAGRAMS.md table)
  A->>P: write D<n>-<slug>.md (Status: draft)
  A->>L: lint
  alt lint fails
    L-->>A: exit 1, fix header/fence
  else lint ok
    A->>H: present diagram at its stage (intake checkpoint / design review)
    alt approved
      H->>P: Status: reviewed, Reviewed-by, Reviewed-at
      H->>CLI: --type review --source human --story US-xxx
      Note over CLI: intervention event appended
    else rejected
      H->>CLI: --type correction
      Note over P: stays draft, agent redraws
    end
  end
  A->>A: implement, deriving tests/placement from reviewed diagrams
  opt subject changed after review
    A->>P: Status: stale (same commit)
    A->>H: re-review
  end
  A->>P: done gate: every diagram matches shipped code
```

## What to review

- Is the review recorded in both places (file header and intervention) and is
  the intervention the one that wins on disagreement?
- Is `stale` set by the agent that changes the subject, in the same commit?
- Does a rejection leave the file `draft` rather than deleting it?

## Derived next steps

- Lint test: wrong Kind / `reviewed` without reviewer fields → exit 1 (covered by `scripts/check-diagrams.sh`).
- Done-gate check: every required diagram `reviewed` — manual until `story verify` can require it.
