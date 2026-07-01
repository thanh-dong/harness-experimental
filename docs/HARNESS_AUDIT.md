# Harness Audit

`scripts/bin/harness-cli audit` detects drift in durable Harness state and
prints an entropy score. Lower is better.

## Checks

| Category | Meaning | Weight |
| --- | --- | --- |
| Orphaned stories | Planned or in-progress stories with no linked trace. | 10 |
| Unverified stories | Stories with `verify_command` but no recorded verification result. | 5 |
| Unverified decisions | Decisions with `verify_command` but no recorded verification result. | 5 |
| Open backlog without outcomes | Implemented backlog items with predicted impact but no actual outcome. | 2 |
| Stale stories | Unimplemented stories whose latest linked trace is more than 30 days old. | 3 |
| Broken tools | Registered tools whose command is not found on disk or `PATH`. | 8 |

## Score

```text
score = orphaned_stories * 10
      + unverified_stories * 5
      + unverified_decisions * 5
      + backlog_without_outcomes * 2
      + stale_stories * 3
      + broken_tools * 8
```

The score is capped at 100.

| Range | Interpretation |
| --- | --- |
| 0 | Perfect: records are traced, verified, and healthy. |
| 1-25 | Healthy: minor housekeeping remains. |
| 26-50 | Attention needed: drift is accumulating. |
| 51-100 | Action required: stale state undermines Harness value. |

Audit findings feed `scripts/bin/harness-cli propose`, which can turn repeated
drift into proposed backlog items.

## Calibration

The audit detector and the `score-trace` scorer are themselves checked with a
black-box calibrator that borrows Okra's golden pass/fail discipline: it drives
the shipped binary against known-good and known-bad states and asserts the
observable verdicts, so drift detection and trace scoring cannot silently rot.

```bash
scripts/calibrate-harness.sh                              # uses scripts/bin/harness-cli
HARNESS_CLI=target/release/harness-cli scripts/calibrate-harness.sh   # a built binary
```

Each golden isolates one signal: a clean install must score entropy `0`; a single
orphaned story must score `10`; an unverified story or decision `5`; a broken
tool `8`; and the four trace tiers (`incomplete`/`minimal`/`standard`/`detailed`)
must each be reached by the expected field depth. The calibrator exits non-zero
if any verdict drifts from its golden, and runs in CI (`Harness CLI Release`
verify job) after `cargo test`. Run it before merges and maturity claims,
alongside `story verify-all`.
