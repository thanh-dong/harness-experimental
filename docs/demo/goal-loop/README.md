# Demo: Goal Loop bridged to the harness store (Phase 2)

This demo proves the P2 bridge from `COMBINATION.md`: an Okra loop whose metric
contracts resolve through **real harness commands**, and whose three-point
anti-goal eval maps to `story verify` / `story verify-all` / `query matrix`.

Both sides are verified, not just described:

- the harness CLI runs the whole scenario (transcript below);
- the Okra `frame.v1.json` and `tree.v1.json` in this folder pass Okra's own
  schema gate and hash-chain check (`okra-store.sh write-frame` / `write-tree` /
  `verify` all succeed).

## The scenario

Initiative: *ship a checkout retry without breaking existing checkout behavior.*

- **Objective:** `checkout_retry_unit_proof == 1`, read from harness story
  `US-101`'s `verify_command`.
- **Anti-goal** (derived from the `existing behavior` risk flag): `existing
  behavior stays green`, a **tripwire**, read from `story verify-all` (baseline
  `US-100` must never go red).

The objective and anti-goal are not hand-authored: intake classified the work
`high-risk` with flags `existing behavior, public contracts`, and the agent
metricized those flags into the frame (see `docs/GOAL_LOOP.md`).

## The mapping this demo establishes

| Okra concept | Harness command it resolves to |
| --- | --- |
| Objective / CKR metric read | `harness-cli story verify US-101` · `query matrix --numeric` |
| Anti-goal reading | `harness-cli story verify-all` (baseline stays green) |
| Admissibility (before acting) | read anti-goal via `verify-all` before working a story |
| Direct read (after acting) | `story verify US-101` from source, not from task-done |
| Paired goal/anti-goal eval | objective `verify` passes **and** `verify-all` green |
| Frame ratification | `harness-cli decision add 0001-checkout-retry-frame` |
| Escalation flag | `harness-cli intervention add` |
| Run footprint | `harness-cli trace --story US-101` (notes carry `okra_run_id`) |

## Transcript (real output)

```text
# 3. BASELINE metric read
US-100  Existing checkout stays green  planned  0 ...
US-101  Checkout retry implemented     planned  0 ...

# 4. round 1 — before work
-- admissibility: anti-goal read (verify-all) --
Story US-100: pass
Story US-101: fail
2 stories verified: 1 passed, 1 failed
-- direct read: objective (US-101) --
Running: test -f .../retry.done
Story US-101 verification: fail
>> objective NOT met, anti-goal held -> loop continues (not pointless yet)

# 5. progression worker executes PKR-1  (retry.done created; unit proof recorded)

# 6. round 2 — paired
-- objective direct read --   Story US-101 verification: pass
-- anti-goal direct read --   2 stories verified: 2 passed, 0 failed
>> objective MET *and* anti-goal held -> paired eval PASSES

# 7. final matrix
US-100  Existing checkout stays green  planned      no  ...
US-101  Checkout retry implemented     implemented  yes ...
```

Round 1 shows the loop refusing to declare success on a flat objective metric
(no cascade). Round 2 shows the paired eval: the objective moved **and** the
anti-goal held, which is the only shape that counts as a win.

## Reproduce it

```bash
H=scripts/bin/harness-cli
WS=$(mktemp -d); mkdir -p "$WS/scripts"; cp -R scripts/schema "$WS/scripts/schema"
export HARNESS_REPO_ROOT="$WS" HARNESS_DB="$WS/harness.db"

"$H" init
"$H" intake --type new_initiative --lane high-risk \
  --summary "Ship checkout retry without breaking existing checkout behavior" \
  --flags "existing behavior,public contracts"
"$H" story add --id US-100 --title "Existing checkout stays green" --lane normal --verify "true"
"$H" story add --id US-101 --title "Checkout retry implemented" --lane high-risk --verify "test -f $WS/retry.done"

"$H" story verify-all || true          # anti-goal read: US-100 green, US-101 not yet
"$H" story verify US-101 || true        # objective read: fail

touch "$WS/retry.done"                   # progression worker does the PKR
"$H" story update --id US-101 --status implemented --unit 1

"$H" story verify US-101                 # objective: pass
"$H" story verify-all                    # paired: all green
```

Validate the Okra artifacts against Okra's own store checker (requires the Okra
skill checkout):

```bash
S=/path/to/okra/skills/reverse-tornado-okr/scripts/okra-store.sh
OK=$(mktemp -d); "$S" init "$OK/.okra"
RUN=$("$S" init-run okr-checkout-retry-001 "$OK/.okra")
"$S" write-frame docs/demo/goal-loop/frame.v1.json "$RUN"
"$S" write-tree  docs/demo/goal-loop/tree.v1.json  "$RUN"
"$S" verify "$RUN"     # -> OKRA store verified
```

## Files

- `frame.v1.json` — the ratified frame; `metric_contracts[].read_method` are
  harness commands. Passes `okra-store.sh write-frame` + `verify`.
- `tree.v1.json` — orchestrator/DKR/CKR/PKR; `PKR-1.story = US-101` binds the
  execution unit to a harness story. Passes `okra-store.sh write-tree` + `verify`.

`frame_hash` is a placeholder here; a real run computes it via `okra-store.sh`.
