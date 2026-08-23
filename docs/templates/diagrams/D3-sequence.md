# D3 Sequence — US-XXX Story title

Story: US-XXX
Kind: sequence
Source: hand
Scope: <command or entry point> → <handler> → <storage> → <side effects>
Status: draft
Reviewed-by: -
Reviewed-at: -

```mermaid
sequenceDiagram
  autonumber
  participant CLI as harness-cli <subcommand>
  participant H as handler
  participant R as repository
  participant L as .harness/events
  CLI->>H: parsed input
  H->>R: validate + write
  alt valid
    R->>L: append event
    Note over L: view regenerated
    R-->>CLI: id
  else invalid input
    R-->>CLI: error (exit 1)
  end
```

## What to review

- Does the happy path match the product contract?
- Is every error path the code can hit drawn as an `alt` / `else`?
- Are all side effects (events, generated views, files) shown as Notes?

## Derived next steps

- One integration or E2E case per `alt` / `opt` branch, listed in the validation plan before implementation.
- DTO / argument shapes for the interface contract.
