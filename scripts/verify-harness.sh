#!/bin/bash
# verify-harness.sh <repo-root> — 9 checks that a harness install works.
# Read-only checks run against the real repo; write checks run in a temp clone.
REPO="${1:?usage: verify-harness.sh <repo-root>}"
H="$REPO/scripts/bin/harness-cli"
CL=$(mktemp -d)/clone
SCORE=0; TOTAL=9
pass() { SCORE=$((SCORE+1)); echo "PASS  $1"; }
fail() { echo "FAIL  $1 -- $2"; }
T0=$(date +%s)

# 1. binary present + expected version
V=$("$H" --version 2>/dev/null)
[ -n "$V" ] && pass "binary present ($V)" || fail "binary" "missing/not executable"

# 2. schema files 001..008 present, no duplicate version prefixes
N=$(ls "$REPO"/scripts/schema/*.sql | wc -l | tr -d ' ')
D=$(ls "$REPO"/scripts/schema/*.sql | xargs -n1 basename | cut -d- -f1 | sort | uniq -d | wc -l | tr -d ' ')
[ "$N" -ge 8 ] && [ "$D" -eq 0 ] && pass "schema files complete, no duplicates ($N files)" || fail "schema files" "count=$N dups=$D"

# 3. event log exists and is git-tracked
if [ -d "$REPO/.harness/events" ] && (cd "$REPO" && [ -n "$(git ls-files .harness/events)" ]); then
  pass "event log present and git-tracked"; else fail "event log" "missing or untracked"; fi

# 4. query matrix works on real repo (auto-rebuild path)
(cd "$REPO" && "$H" query matrix >/dev/null 2>&1) && pass "query matrix" || fail "query matrix" "errored"

# 5. rebuild deterministic on real repo
H1=$(cd "$REPO" && "$H" rebuild 2>/dev/null | grep -i hash)
H2=$(cd "$REPO" && "$H" rebuild 2>/dev/null | grep -i hash)
[ -n "$H1" ] && [ "$H1" = "$H2" ] && pass "rebuild deterministic ($H1)" || fail "rebuild" "$H1 vs $H2"

# 6. schema version == binary max (migrate is a no-op)
M=$(cd "$REPO" && "$H" migrate 2>&1)
echo "$M" | grep -q "Applied 0\|up to date\|Current schema version: 8" && ! echo "$M" | grep -q "Applying" \
  && pass "schema at binary max (no pending migrations)" || fail "migrate" "$M"

# 7. fresh clone: auto-replay + write path (intake) works, appends a writer file
git clone -q "$REPO" "$CL" 2>/dev/null
mkdir -p "$CL/scripts/bin" && cp "$H" "$CL/scripts/bin/"
if (cd "$CL" && scripts/bin/harness-cli intake --type maintenance_request --summary "verify write" --lane tiny >/dev/null 2>&1 \
    && ls .harness/events/*.jsonl >/dev/null 2>&1 && [ -n "$(git status --porcelain .harness/events)" ]); then
  pass "fresh clone: replay + event-backed write"; else fail "clone write" "intake failed or no event appended"; fi

# 8. intervention --type review accepted in clone (schema 008 live)
(cd "$CL" && scripts/bin/harness-cli intervention add --type review --source agent --description "verify" >/dev/null 2>&1) \
  && pass "intervention --type review (008)" || fail "intervention review" "rejected"

# 9. the "## Harness" reading rule is one text in three places: AGENTS.md and
# the two installer templates (scripts/install-harness.sh, .ps1) each embed a
# copy inside <!-- HARNESS:BEGIN/END --> markers. They must read identically
# once the macOS/Linux-vs-Windows path alternative and the .exe suffix --
# the one deliberate platform difference -- are normalized away.
harness_block() {
  awk '/<!-- HARNESS:BEGIN -->/ { f=1; next } /<!-- HARNESS:END -->/ { if (f) exit } f' "$1"
}
normalize_harness_block() {
  # The only platform difference the three copies are allowed to have is the
  # macOS/Linux-vs-Windows path alternative (on the matrix bullet and in the
  # CLI paragraph), which also carries the only legitimate ".exe" mentions.
  # Drop that whole clause -- .exe included -- rather than stripping ".exe"
  # everywhere, so an unrelated ".exe" mention elsewhere would still surface
  # as a real difference.
  tr -s '[:space:]' ' ' | sed -E 's/ on macOS\/Linux,? or `[^`]*\.exe[^`]*` on Windows//g' | sed -E 's/^ +| +$//'
}
A_RAW="$REPO/AGENTS.md"
S_RAW="$REPO/scripts/install-harness.sh"
P_RAW="$REPO/scripts/install-harness.ps1"
A_TXT="$(harness_block "$A_RAW" | normalize_harness_block)"
S_TXT="$(harness_block "$S_RAW" | normalize_harness_block)"
P_TXT="$(harness_block "$P_RAW" | normalize_harness_block)"
if [ -n "$A_TXT" ] && [ "$A_TXT" = "$S_TXT" ] && [ "$A_TXT" = "$P_TXT" ]; then
  pass "Harness reading rule identical (AGENTS.md, install-harness.sh, install-harness.ps1)"
else
  DIFFS=""
  [ "$A_TXT" != "$S_TXT" ] && DIFFS="$DIFFS AGENTS.md!=install-harness.sh"
  [ "$A_TXT" != "$P_TXT" ] && DIFFS="$DIFFS AGENTS.md!=install-harness.ps1"
  [ "$S_TXT" != "$P_TXT" ] && DIFFS="$DIFFS install-harness.sh!=install-harness.ps1"
  fail "Harness reading rule" "texts differ:$DIFFS"
fi

rm -rf "$(dirname "$CL")"
T1=$(date +%s)
echo "----------------------------------------"
echo "REPO: $REPO  SCORE: $SCORE/$TOTAL  runtime: $((T1-T0))s"
[ "$SCORE" -eq "$TOTAL" ]
