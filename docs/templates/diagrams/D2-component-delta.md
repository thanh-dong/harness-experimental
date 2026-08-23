# D2 Component delta — US-XXX Story title

Story: US-XXX
Kind: component-delta
Source: generated:c3
Scope: containers and components this story adds, changes, or removes
Status: draft
Reviewed-by: -
Reviewed-at: -

```mermaid
flowchart TB
  subgraph interface
    cli[harness-cli]:::muted
  end
  subgraph application
    h1[existing-handler]:::changed
    h2[new-handler]:::added
  end
  subgraph infrastructure
    repo[repository]:::muted
    old[retired-adapter]:::removed
  end
  cli --> h1
  cli --> h2
  h1 --> repo
  h2 --> repo

  classDef added   fill:#e6ffed,stroke:#2da44e
  classDef changed fill:#fff8c5,stroke:#bf8700
  classDef removed fill:#ffebe9,stroke:#cf222e,stroke-dasharray:4
  classDef muted   fill:#f6f8fa,stroke:#d0d7de,color:#57606a
```

## What to review

- Does the direction match `docs/ARCHITECTURE.md` layering (domain <- application <- infrastructure <- interface)?
- Is any `added` node a new boundary that needs a decision record?
- Is every `removed` node orphaned by this change, not pre-existing dead code?

## Derived next steps

- File placement per layer for each `added` / `changed` node.
- Design Notes / Harness Delta section entries.
