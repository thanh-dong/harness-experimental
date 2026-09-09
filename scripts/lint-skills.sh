#!/usr/bin/env bash
# Lint the bundled agent skills.
#
# Three assertions:
#   1. The .claude and .codex copies of every bundled skill are byte-identical.
#      Both trees ship the same instructions to different hosts; a divergence
#      means one host silently reads older prose.
#   2. Every exact sentence the OKRA completeness contract requires (a token
#      that contains a space and is longer than 40 characters) still appears in
#      the skill's own prose -- SKILL.md or a file under references/. The
#      contract is the source of truth; prose that no longer states the
#      sentence cannot teach an agent to produce it.
#   3. No contract or skill file uses grader vocabulary (`case prompt`,
#      `blindbox`, `hidden eval`). These skills are product instructions, not
#      benchmark scaffolding.
#
# Exit non-zero on any failure. Additive, read-only.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKILL_NAME="reverse-tornado-okr"
CLAUDE_SKILL="$REPO_ROOT/.claude/skills/$SKILL_NAME"
CODEX_SKILL="$REPO_ROOT/.codex/skills/$SKILL_NAME"
CONTRACT="$CLAUDE_SKILL/contracts/handoff-contract.v2.json"

fail=0
note() { printf '%s\n' "$*"; }

[ -d "$CLAUDE_SKILL" ] || { note "FAIL: skill not found: $CLAUDE_SKILL"; exit 1; }
[ -d "$CODEX_SKILL" ] || { note "FAIL: mirror not found: $CODEX_SKILL"; exit 1; }
[ -f "$CONTRACT" ] || { note "FAIL: contract not found: $CONTRACT"; exit 1; }

# --- Assertion 1: .claude and .codex skill trees are identical ---------------
note "== Assertion 1: .claude and .codex skill trees are identical =="
divergence="$(diff -r "$CLAUDE_SKILL" "$CODEX_SKILL" 2>&1 || true)"
if [ -z "$divergence" ]; then
  note "  ok    .claude/skills/$SKILL_NAME == .codex/skills/$SKILL_NAME"
else
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    note "  FAIL  $line"
  done <<<"$divergence"
  fail=1
fi

# --- Assertion 2: contract sentences still appear in the skill prose ---------
note "== Assertion 2: contract sentences appear in SKILL.md or references/ =="
if ! command -v python3 >/dev/null 2>&1; then
  note "  WARN  python3 unavailable; skipping contract-to-prose check"
else
  results="$(python3 - "$CONTRACT" "$CLAUDE_SKILL" <<'PY'
import glob, json, os, re, sys

contract_path, skill_dir = sys.argv[1], sys.argv[2]

def normalize(text):
    """Same normalization okra-verify-artifact.py uses: lowercase + collapse whitespace."""
    return re.sub(r"\s+", " ", text.lower())

sources = [os.path.join(skill_dir, "SKILL.md")]
sources += sorted(glob.glob(os.path.join(skill_dir, "references", "*.md")))
haystack = normalize("\n".join(open(p, encoding="utf-8").read() for p in sources))

contract = json.load(open(contract_path, encoding="utf-8"))
seen = set()
for req in contract.get("requirements", []):
    for group in req.get("groups", []):
        for token in group.get("tokens", []):
            # The exact sentences: multi-word tokens long enough to be prose.
            if " " not in token or len(token) <= 40:
                continue
            key = (req.get("id"), token)
            if key in seen:
                continue
            seen.add(key)
            status = "PASS" if normalize(token) in haystack else "MISS"
            print("%s\t%s\t%s" % (status, req.get("id"), token))
PY
)"
  if [ -z "$results" ]; then
    note "  FAIL  no sentence tokens extracted from the contract"
    fail=1
  else
    while IFS=$'\t' read -r status req token; do
      [ -n "$status" ] || continue
      if [ "$status" = PASS ]; then
        note "  ok    $req"
      else
        note "  FAIL  $req sentence missing from SKILL.md and references/: \"$token\""
        fail=1
      fi
    done <<<"$results"
  fi
fi

# --- Assertion 3: no grader vocabulary in contract or skill files ------------
note "== Assertion 3: no grader vocabulary in contract or skill files =="
grader_re='case prompt|blindbox|hidden eval'
offenders="$(grep -rniE "$grader_re" "$CLAUDE_SKILL" "$CODEX_SKILL" || true)"
if [ -n "$offenders" ]; then
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    note "  FAIL  ${line#"$REPO_ROOT"/}"
    fail=1
  done <<<"$offenders"
else
  note "  ok    no grader vocabulary found"
fi

note ""
if [ "$fail" = 0 ]; then
  note "PASS: skills lint clean."
else
  note "FAIL: skills lint found problems (see above)."
fi
exit "$fail"
