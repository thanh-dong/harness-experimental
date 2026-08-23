# D1 Blast radius — US-XXX Story title

Story: US-XXX
Kind: blast-radius
Source: generated:codegraph
Scope: files changed by this story and everything that depends on them
Coverage: 0 of 0
Status: draft
Reviewed-by: -
Reviewed-at: -

```mermaid
flowchart LR
  subgraph changed
    f1[path/to/changed-file.rs]:::changed
  end
  subgraph dependents
    f2[path/to/dependent.rs]:::muted
  end
  subgraph components
    c1[component-id]:::muted
  end
  subgraph stories
    s1[US-000]:::muted
  end
  subgraph docs
    d1[docs/product/area.md]:::muted
  end
  f1 --> f2 --> c1 --> s1 --> d1

  classDef added   fill:#e6ffed,stroke:#2da44e
  classDef changed fill:#fff8c5,stroke:#bf8700
  classDef removed fill:#ffebe9,stroke:#cf222e,stroke-dasharray:4
  classDef muted   fill:#f6f8fa,stroke:#d0d7de,color:#57606a
```

## What to review

- Is every node in `changed` inside the approved scope?
- Is any dependent, component, or story here that the request did not mention?
- Coverage below half means feature impact is UNKNOWN — read the product docs
  by hand before approving.

## Derived next steps

- Re-run set: `scripts/bin/harness-cli story verify <id>` for every story node.
- Reading list: every component and doc node, before implementation.
- Flags answered from this diagram: Existing behavior, Multi-domain, Public contracts.
