# D7 Boundary — US-XXX Story title

Story: US-XXX
Kind: boundary
Source: hand
Scope: process, provider, and platform boundaries this story crosses
Status: draft
Reviewed-by: -
Reviewed-at: -

```mermaid
flowchart LR
  subgraph local["this process"]
    app[application]:::changed
  end
  subgraph provider["external provider"]
    api[provider API]:::muted
  end
  subgraph ci["CI runner"]
    job[pipeline job]:::added
  end
  app -- "HTTPS, outbound" --> api
  job -- "harness-cli, shell" --> app

  classDef added   fill:#e6ffed,stroke:#2da44e
  classDef changed fill:#fff8c5,stroke:#bf8700
  classDef removed fill:#ffebe9,stroke:#cf222e,stroke-dasharray:4
  classDef muted   fill:#f6f8fa,stroke:#d0d7de,color:#57606a
```

## What to review

- Does any crossing edge change an external provider contract? (hard gate)
- Is every crossing labelled with protocol and direction?
- Which boundary needs a deterministic fixture or stub for proof?

## Derived next steps

- Fixture / stub list in `validation.md`.
- Observability point per crossing edge in `design.md`.
