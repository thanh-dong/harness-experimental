# Change Diagram Templates

One template per change-diagram kind. Copy the file for the kind you need into
`<story packet>/diagrams/` and keep the filename prefix (`D1-` … `D7-`); the
slug after it is yours. Rules, review stages, and the lint are in
`docs/DIAGRAMS.md`.

| File | Kind | Required by |
| --- | --- | --- |
| `D1-blast-radius.md` | blast-radius | normal (capability active), high-risk |
| `D2-component-delta.md` | component-delta | high-risk |
| `D3-sequence.md` | sequence | normal (multi-component), high-risk |
| `D4-state.md` | state | any lane that introduces or alters a status set |
| `D5-data-model.md` | data-model | `Data model` flag |
| `D6-story-dag.md` | story-dag | new spec, new initiative, goal loop |
| `D7-boundary.md` | boundary | `External systems` or `Cross-platform` flag |

Lint before review: `bash scripts/check-diagrams.sh <file>`.
