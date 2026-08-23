# D3 Sequence — US-038 story verify diagram gate

Story: US-038
Kind: sequence
Source: hand
Scope: harness-cli story verify <id> → docs/**/diagrams scan → verify_command → story.verify_result event
Status: draft
Reviewed-by: -
Reviewed-at: -

```mermaid
sequenceDiagram
  autonumber
  participant CLI as harness-cli story verify <id>
  participant R as SqliteHarnessRepository::verify_story
  participant FS as docs/**/diagrams/D<n>-*.md
  participant SH as verify_command (shell)
  participant L as .harness/events
  CLI->>R: id
  R->>R: load verify_command (error if none — unchanged)
  R->>FS: scan files with Story: <id>
  alt any Status != reviewed
    R->>L: story.verify_result {fail}
    R-->>CLI: fail, stderr lists "<path> (<status>)" (exit 1)
    Note over SH: command never runs
  else all reviewed (or no diagrams)
    R->>SH: run in repo root
    SH-->>R: exit code
    R->>L: story.verify_result {pass|fail}
    R-->>CLI: result + stdout/stderr
  end
  Note over R: verify-all applies the same gate per story
```

## What to review

- Is "any diagram file naming this story that is not `reviewed`" the right gate, rather than only flag-required ones? (Chosen: if you drew it, it must be reviewed or deleted.)
- Should the gate record a `fail` event? (Chosen: yes — `last_verified_result` must reflect that verify did not pass.)
- Template files (`Story: US-XXX`) can never match a real id, so they never gate.

## Derived next steps

- Unit test: scan finds only matching story + non-reviewed status, ignores non-`diagrams/` dirs.
- Integration test: gated verify does not run the command, records fail; reviewed → runs and passes.
