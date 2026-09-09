# US-039 evidence

Durable copies of the artifacts and verifier output the story was proved with.
They were written under `/tmp/us039/` during the work and copied here so the
before/after proof survives a reboot and can be re-run from the repo alone.

| File | What it is |
| --- | --- |
| `fixture-okr-artifact.md` | The OKRA delegated-loop artifact written in Task 1 as a realistic (non-benchmark) goal: raise newsletter open rate from 31% to 40% without pushing the unsubscribe rate above 0.5%. It follows `SKILL.md` literally, so it is the fixture the completeness contract is measured against. |
| `baseline-verify.json` | `okra-verify-artifact.py` run on that fixture against the **old** `handoff-contract.v1.json`, before any fix: 16 of 20 requirements, `complete: false`. This is the "before" number the story exists to fix. |
| `artifact-A.md` | Task 6 A/B: the artifact a blind Fable writer produced from skill variant A (the prescriptive, numbered-step `SKILL.md` that Task 5 left in place). |
| `artifact-B.md` | Task 6 A/B: the artifact a second blind Fable writer produced from skill variant B (the de-prescribed `SKILL.md` with a compact `## Contract` list). Neither writer saw the other variant. |
| `verify-A.json` | Verifier output for `artifact-A.md`: **16 / 19**. Missing `dkr_to_dkr_worked`, `ckr_not_worker_work`, `candidate_antigoal_library`. |
| `verify-B.json` | Verifier output for `artifact-B.md`: **18 / 19**. Missing only `dkr_to_dkr_worked`. |

## A/B result

Variant B satisfies every requirement variant A satisfies and two more, so it
is a strict superset (18/19 against 16/19). Variant B's skill text is 32.9%
shorter (3389 words against 5048) while its artifact is 29 words longer
(5285 against 5256, +0.55%). **Variant B was adopted** (commit `a7f74b4`):
the story's criterion is completeness under the verifier, and a 29-word gap
between two independently written 5,000-word artifacts is writer noise, not a
measured verbosity cost. Both variants missed `dkr_to_dkr_worked`, a skill gap
the de-prescription neither caused nor fixed.

## Re-running the proof

```bash
python3 .claude/skills/reverse-tornado-okr/scripts/okra-verify-artifact.py \
  docs/stories/epics/E07-prompt-audit-remediation/US-039-prompt-audit-remediation/evidence/fixture-okr-artifact.md --json
```

Against the shipped `handoff-contract.v2.json` this prints
`"satisfied_count": 19, "required_count": 19, "complete": true` and exits 0.
