#!/usr/bin/env bash
# Black-box calibration for the harness `audit` drift detector and `score-trace`
# quality scorer.
#
# It borrows Okra's golden pass/fail `--calibrate` discipline: drive the *shipped*
# binary against known-good and known-bad states and assert its observable
# verdicts. Unit tests check internals; this checks that the drift detector and
# trace scorer a human actually runs still behave, so they cannot silently rot.
#
# Usage:
#   scripts/calibrate-harness.sh            # uses scripts/bin/harness-cli
#   HARNESS_CLI=target/release/harness-cli scripts/calibrate-harness.sh
#
# Exits 0 when every golden holds, 1 otherwise.
set -uo pipefail

SELF_DIR="$(cd "$(dirname "$0")" && pwd)"
CLI="${HARNESS_CLI:-$SELF_DIR/bin/harness-cli}"
SCHEMA_SRC="$SELF_DIR/schema"
PASS=0
FAIL=0

if [ ! -x "$CLI" ]; then
  echo "harness-cli not found or not executable at: $CLI" >&2
  echo "set HARNESS_CLI to a built binary (e.g. target/release/harness-cli)" >&2
  exit 2
fi

setup() { # -> prints a fresh, initialized, isolated workspace dir
  local d
  d=$(mktemp -d /tmp/harness-cal-XXXXXX)
  mkdir -p "$d/scripts"
  cp -R "$SCHEMA_SRC" "$d/scripts/schema"
  HARNESS_REPO_ROOT="$d" HARNESS_DB="$d/harness.db" "$CLI" init >/dev/null
  printf '%s' "$d"
}

hc() { HARNESS_REPO_ROOT="$WS" HARNESS_DB="$WS/harness.db" "$CLI" "$@"; }

check() { # name  expected-substring  actual-text
  if printf '%s' "$3" | grep -qF -- "$2"; then
    PASS=$((PASS + 1))
  else
    FAIL=$((FAIL + 1))
    echo "  FAIL [$1]: expected to find: $2"
    printf '%s\n' "$3" | sed 's/^/        | /'
  fi
}

echo "=== audit calibration (golden drift states) ==="

# G0 — clean install: zero drift, zero entropy.
WS=$(setup)
out=$(hc audit)
check "clean/orphaned-0" "Orphaned stories (planned/in-progress, no traces): 0" "$out"
check "clean/broken-0" "Broken tools: 0" "$out"
check "clean/entropy-0" "Entropy score: 0/100" "$out"
rm -rf "$WS"

# G1 — orphaned story: planned story with no linked trace (weight 10).
WS=$(setup)
hc story add --id US-1 --title "orphan story" --lane normal >/dev/null
out=$(hc audit)
check "orphaned/count-1" "Orphaned stories (planned/in-progress, no traces): 1" "$out"
check "orphaned/entropy-10" "Entropy score: 10/100" "$out"
rm -rf "$WS"

# G2 — unverified story: verify_command set, implemented, never verified (weight 5).
WS=$(setup)
hc story add --id US-2 --title "unverified story" --lane normal --verify "true" >/dev/null
hc story update --id US-2 --status implemented >/dev/null
out=$(hc audit)
check "unverified-story/count-1" "Unverified stories: 1" "$out"
check "unverified-story/entropy-5" "Entropy score: 5/100" "$out"
rm -rf "$WS"

# G3 — unverified decision: verify_command set, never verified (weight 5).
WS=$(setup)
hc decision add --id 0001-x --title "unverified decision" --verify "true" >/dev/null
out=$(hc audit)
check "unverified-decision/count-1" "Unverified decisions: 1" "$out"
check "unverified-decision/entropy-5" "Entropy score: 5/100" "$out"
rm -rf "$WS"

# G4 — broken tool: cli command that resolves nowhere (weight 8).
WS=$(setup)
hc tool register --name brk --kind cli --command "/nonexistent/xyz-calibration" \
  --description "broken tool used for calibration only" \
  --responsibility Verification --force >/dev/null
out=$(hc audit)
check "broken-tool/count-1" "Broken tools: 1" "$out"
check "broken-tool/entropy-8" "Entropy score: 8/100" "$out"
rm -rf "$WS"

# G5 — additivity: orphaned (10) + broken tool (8) = 18.
WS=$(setup)
hc story add --id US-5 --title "orphan again" --lane normal >/dev/null
hc tool register --name brk2 --kind cli --command "/nonexistent/also-missing" \
  --description "another broken tool for calibration" \
  --responsibility Verification --force >/dev/null
out=$(hc audit)
check "additivity/entropy-18" "Entropy score: 18/100" "$out"
rm -rf "$WS"

echo "=== score-trace calibration (quality tiers) ==="

# S0 — incomplete: summary present but no outcome.
WS=$(setup)
hc trace --summary "no outcome recorded" >/dev/null
check "trace/incomplete" "Tier achieved: incomplete (0/3)" "$(hc score-trace)"
rm -rf "$WS"

# S1 — minimal: summary >= 10 chars + outcome.
WS=$(setup)
hc trace --summary "minimal tier trace" --outcome completed >/dev/null
check "trace/minimal" "Tier achieved: minimal (1/3)" "$(hc score-trace)"
rm -rf "$WS"

# S2 — standard: + agent, actions, files_read, files_changed, errors-or-friction.
WS=$(setup)
hc trace --summary "standard tier trace" --outcome completed --agent claude \
  --actions "read,edit" --read "a.rs,b.rs" --changed "a.rs" --friction "none" >/dev/null
check "trace/standard" "Tier achieved: standard (2/3)" "$(hc score-trace)"
rm -rf "$WS"

# S3 — detailed: + decisions, errors, friction, duration, token_estimate.
WS=$(setup)
hc trace --summary "detailed tier trace" --outcome completed --agent claude \
  --actions "read,edit" --read "a.rs,b.rs" --changed "a.rs" \
  --decisions "chose X,rejected Y" --errors "none" --friction "none" \
  --duration 120 --tokens 4000 >/dev/null
check "trace/detailed" "Tier achieved: detailed (3/3)" "$(hc score-trace)"
rm -rf "$WS"

echo
if [ "$FAIL" -eq 0 ]; then
  echo "calibration ok: $PASS checks passed"
  exit 0
fi
echo "calibration FAILED: $FAIL failed, $PASS passed"
exit 1
