# D6 Story DAG — spec:<slug> or initiative title

Story: spec:<slug>
Kind: story-dag
Source: hand
Scope: stories of <initiative> and their dependencies
Status: draft
Reviewed-by: -
Reviewed-at: -

```mermaid
flowchart LR
  A[US-001 first slice]:::ready
  B[US-002 second slice]:::blocked
  C[US-003 third slice]:::blocked
  A --> B
  A --> C

  classDef ready   fill:#e6ffed,stroke:#2da44e
  classDef blocked fill:#f6f8fa,stroke:#d0d7de,color:#57606a
  classDef done    fill:#ddf4ff,stroke:#0969da
```

## What to review

- Is the slicing right — is each node one bounded story?
- Does `A --> B` really mean B cannot start before A ships?
- Is there a cycle? (must be acyclic)

## Derived next steps

- Execution order and which stories can run in parallel.
- The single "ready" story named in the implementation handoff.
- Story hierarchy / dependency records in the durable layer.
