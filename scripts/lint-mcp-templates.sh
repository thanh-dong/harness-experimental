#!/usr/bin/env bash
# Lint the MCP-flavored context templates.
#
# Three assertions:
#   1. Every harness_* tool name referenced in docs/templates/mcp/*.md is
#      defined as a mapping row in docs/templates/mcp/TOOL_MAPPING.md.
#   2. Every mapping row's CLI command exists in `harness-cli --help` output
#      (best-effort grep; skipped with a warning if the binary is unavailable).
#   3. No generated view (docs/TEST_MATRIX.md, docs/HARNESS_BACKLOG.md,
#      docs/decisions/README.md) is named as something an agent edits, updates,
#      or adds rows to directly — those files are rewritten from the event log,
#      never hand-edited.
#
# Exit non-zero on any failure. Additive, read-only.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MCP_DIR="$REPO_ROOT/docs/templates/mcp"
MAPPING="$MCP_DIR/TOOL_MAPPING.md"

fail=0
note() { printf '%s\n' "$*"; }

[ -f "$MAPPING" ] || { note "FAIL: mapping table not found: $MAPPING"; exit 1; }

# --- Defined tools: first-column `harness_*` code spans in the mapping table ---
defined="$(grep -oE 'harness_[a-z_]+' "$MAPPING" | sort -u)"
[ -n "$defined" ] || { note "FAIL: no harness_* tools defined in mapping table"; exit 1; }

# --- Referenced tools: every harness_* used across the MCP templates ----------
referenced="$(grep -rhoE 'harness_[a-z_]+' "$MCP_DIR" | sort -u)"

note "== Assertion 1: referenced tools are defined in the mapping =="
while IFS= read -r tool; do
  [ -n "$tool" ] || continue
  if grep -qxF "$tool" <<<"$defined"; then
    note "  ok    $tool"
  else
    note "  FAIL  $tool referenced but missing from TOOL_MAPPING.md"
    fail=1
  fi
done <<<"$referenced"

# --- Assertion 2: mapping CLI commands exist in harness-cli --help ------------
note "== Assertion 2: mapping CLI commands exist in harness-cli --help =="
CLI="$REPO_ROOT/scripts/bin/harness-cli"
if [ ! -x "$CLI" ]; then
  note "  WARN  $CLI not executable; skipping CLI-existence check (best-effort)"
else
  # Aggregate top-level help plus one level of subcommand help into one blob.
  help_blob="$("$CLI" --help 2>&1 || true)"
  for sub in intake story decision backlog trace query intervention; do
    help_blob+=$'\n'"$("$CLI" "$sub" --help 2>&1 || true)"
  done
  help_blob+=$'\n'"$("$CLI" story signal --help 2>&1 || true)"

  # Extract the CLI command that follows `harness-cli` in each mapping row,
  # taking the leading subcommand words (e.g. `story signal add`).
  cmds="$(grep -oE 'harness-cli [a-z ]+' "$MAPPING" | sed -E 's/^harness-cli //; s/ +$//' | sort -u)"
  while IFS= read -r cmd; do
    [ -n "$cmd" ] || continue
    # Check each word in the command path appears in the aggregated help.
    ok=1
    for word in $cmd; do
      grep -qw "$word" <<<"$help_blob" || ok=0
    done
    if [ "$ok" = 1 ]; then
      note "  ok    harness-cli $cmd"
    else
      note "  FAIL  harness-cli $cmd not found in harness-cli --help output"
      fail=1
    fi
  done <<<"$cmds"
fi

# --- Assertion 3: generated views aren't named as edit targets ----------------
note "== Assertion 3: generated views aren't named as edit targets =="
generated_edit_re='(TEST_MATRIX\.md|HARNESS_BACKLOG\.md|decisions/README\.md)`?[[:space:]]+(rows|updated|edit)'
offenders="$(grep -rnE "$generated_edit_re" "$MCP_DIR" || true)"
if [ -n "$offenders" ]; then
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    note "  FAIL  $line"
    fail=1
  done <<<"$offenders"
else
  note "  ok    no generated view named as an edit target"
fi

note ""
if [ "$fail" = 0 ]; then
  note "PASS: MCP templates lint clean."
else
  note "FAIL: MCP templates lint found problems (see above)."
fi
exit "$fail"
