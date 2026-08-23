# Design

## Change Diagrams

Separate files under `diagrams/` (`docs/DIAGRAMS.md`). D1, D2, D3 always;
D4/D5/D7 by flag. Each must be `reviewed` by a human before implementation.

| Diagram | File | Status |
| --- | --- | --- |
| D1 blast radius | `diagrams/D1-....md` | draft |
| D2 component delta | `diagrams/D2-....md` | draft |
| D3 sequence | `diagrams/D3-....md` | draft |

## Domain Model

Describe entities, value objects, and business rules.

## Application Flow

Describe commands, queries, and handlers. The D3 sequence diagram is the
contract; this section explains it.

## Interface Contract

Describe routes, messages, commands, request DTOs, response DTOs, and errors.

## Data Model

Describe tables, indexes, migrations, and retention concerns. With the
`Data model` flag, the D5 diagram is the contract; this section explains it.

## UI / Platform Impact

Describe browser, mobile, desktop, CLI, deployment, or platform-shell impact.

## Observability

Describe logs, audit records, metrics, or traces.

## Alternatives Considered

1. Option.
