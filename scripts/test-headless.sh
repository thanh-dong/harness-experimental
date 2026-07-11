#!/usr/bin/env bash
set -euo pipefail

# US-033 — headless / non-interactive guarantee test.
#
# Runs the command set an unattended operator (e.g. Shuttle's provisioner)
# calls — init, migrate, tool register|check, info, and every JSON-mode write
# and query command — with stdin closed (</dev/null) and no controlling TTY
# (setsid where available). Every command must exit 0 without blocking on a
# prompt. Offline and CI-ready: it builds the CLI from source, copies the real
# schema into a throwaway mktemp harness dir, and touches nothing in the repo.

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

# Resolve the binary: prefer an explicit HARNESS_BIN, else build from source.
if [[ -n "${HARNESS_BIN:-}" ]]; then
  bin="$HARNESS_BIN"
else
  echo "building harness-cli (release)..."
  cargo build --release -p harness-cli --manifest-path "$repo_root/Cargo.toml" >/dev/null
  bin="$repo_root/target/release/harness-cli"
fi
[[ -x "$bin" ]] || { echo "FAIL: harness-cli binary not found at $bin"; exit 1; }

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/scripts"
cp -r "$repo_root/scripts/schema" "$work/scripts/schema"

export HARNESS_REPO_ROOT="$work"
export HARNESS_DB="$work/harness.db"

# Detach from any controlling terminal when setsid exists (Linux/CI); always
# close stdin so a stray prompt read would EOF immediately instead of hanging.
if command -v setsid >/dev/null 2>&1; then
  no_tty() { setsid "$@" </dev/null; }
else
  no_tty() { "$@" </dev/null; }
fi

pass=0
fail=0
run() {
  local label="$1"; shift
  if no_tty "$@" >/dev/null 2>&1; then
    echo "PASS  $label"
    pass=$((pass + 1))
  else
    echo "FAIL  $label (exit $?)"
    fail=$((fail + 1))
  fi
}

# Lifecycle + introspection.
run "info (uninitialized)"        "$bin" info
run "info --json (uninitialized)" "$bin" --json info
run "init"                        "$bin" init
run "migrate"                     "$bin" migrate
run "info (initialized)"          "$bin" info

# JSON-mode writes.
run "intake --json"   "$bin" --json intake --type change_request --summary "headless smoke intake" --lane tiny
run "story add --json" "$bin" --json story add --id US-HL --title "headless story" --lane normal
run "story update --json" "$bin" --json story update --id US-HL --status in_progress --unit 1
run "decision add --json" "$bin" --json decision add --id 9001 --title "headless decision"
run "backlog add --json"  "$bin" --json backlog add --title "headless backlog"
run "trace --json"        "$bin" --json trace --summary "headless trace over the command set"

# Tool registry.
run "tool register"     "$bin" tool register --name headless-tool --command true --description "always present" --responsibility verification --force
run "tool check --json" "$bin" --json tool check

# JSON-mode queries.
for view in matrix backlog decisions intakes traces friction stats signals; do
  run "query $view --json" "$bin" --json query "$view"
done
run "query tools --json" "$bin" --json query tools
run "query sql --json"   "$bin" --json query sql "SELECT id FROM story;"

echo "---"
echo "$((pass + fail)) commands run headless: $pass passed, $fail failed"
[[ "$fail" -eq 0 ]]
