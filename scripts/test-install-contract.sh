#!/usr/bin/env bash
set -euo pipefail

# US-034 — programmatic install/upgrade contract test.
#
# Exercises install-harness.sh's machine contract on throwaway mktemp dirs:
#   fresh install / merge over existing / re-run idempotent / dry-run parity,
# asserting documented exit codes (0 ok, 2 validation, 3 IO) and the shape of
# the --summary-json output. Offline: the harness file payload is copied from
# this repo (local source mode) and the CLI binary download is satisfied from a
# fake file:// artifact dir, so no network is required. CI-ready.

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
installer="$repo_root/scripts/install-harness.sh"

PASS=0
FAIL=0
WORK=""

cleanup() {
  [ -n "$WORK" ] && rm -rf "$WORK"
}
trap cleanup EXIT

pass() { PASS=$((PASS + 1)); printf 'ok   - %s\n' "$*"; }
die()  { FAIL=$((FAIL + 1)); printf 'FAIL - %s\n' "$*" >&2; }

# Assert the exit code of a command run under `set -e`.
expect_exit() {
  local want="$1"; shift
  local desc="$1"; shift
  local got=0
  "$@" >/dev/null 2>&1 || got=$?
  if [ "$got" -eq "$want" ]; then
    pass "$desc (exit $got)"
  else
    die "$desc: expected exit $want, got $got"
  fi
}

# --- fake CLI artifact dir so the installer's binary download stays offline ---
detect_platform() {
  case "$(uname -s):$(uname -m)" in
    Darwin:arm64)  printf 'macos-arm64' ;;
    Darwin:x86_64) printf 'macos-x64' ;;
    Linux:x86_64)  printf 'linux-x64' ;;
    Linux:aarch64|Linux:arm64) printf 'linux-arm64' ;;
    *) printf 'unsupported' ;;
  esac
}

sha256_of() {
  if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | awk '{print $1}'
  else sha256sum "$1" | awk '{print $1}'; fi
}

WORK="$(mktemp -d)"
PLATFORM="$(detect_platform)"
[ "$PLATFORM" != "unsupported" ] || { echo "unsupported platform for test"; exit 1; }

CLI_DIR="$WORK/cli"
mkdir -p "$CLI_DIR"
printf '#!/bin/sh\necho fake-harness-cli\n' > "$CLI_DIR/harness-cli-$PLATFORM"
sha256_of "$CLI_DIR/harness-cli-$PLATFORM" > "$CLI_DIR/harness-cli-$PLATFORM.sha256"
# Normalize to "<hash>  <name>" so the installer's awk-first-field parse works.
HASH="$(sha256_of "$CLI_DIR/harness-cli-$PLATFORM")"
printf '%s  harness-cli-%s\n' "$HASH" "$PLATFORM" > "$CLI_DIR/harness-cli-$PLATFORM.sha256"
CLI_URL="file://$CLI_DIR"

run_install() {
  HARNESS_CLI_BASE_URL="$CLI_URL" "$installer" "$@"
}

# jq-free JSON readers (work for our flat, machine-generated summary).
json_scalar() { # portable: extract "key":<number>
  local file="$1" key="$2"
  sed -n "s/.*\"$key\":\([0-9][0-9]*\).*/\1/p" "$file" | head -1
}
json_array() { # extract "key":[...] contents as sorted newline list
  local file="$1" key="$2"
  grep -o "\"$key\":\[[^]]*\]" "$file" | head -1 \
    | sed "s/\"$key\":\[//;s/\]//;s/\",\"/\n/g;s/\"//g" \
    | grep -v '^$' | LC_ALL=C sort
}

echo "== US-034 install/upgrade contract =="
echo "platform: $PLATFORM"
echo

# ---------------------------------------------------------------------------
# 1. Fresh install into an empty dir → exit 0, JSON summary, key files present.
# ---------------------------------------------------------------------------
FRESH="$WORK/fresh"
FRESH_JSON="$WORK/fresh.json"
if run_install --yes --summary-json "$FRESH_JSON" "$FRESH" >/dev/null 2>&1; then
  pass "fresh install exits 0"
else
  die "fresh install failed"
fi

[ -f "$FRESH_JSON" ] && pass "fresh install wrote summary JSON" || die "no summary JSON"
grep -q '"ok":true' "$FRESH_JSON" && pass "summary ok:true" || die "summary not ok"

created="$(json_scalar "$FRESH_JSON" created)"
[ "${created:-0}" -gt 0 ] && pass "fresh created > 0 ($created)" || die "fresh created not > 0"

for f in AGENTS.md docs/HARNESS.md scripts/schema/008-intervention-review.sql; do
  [ -f "$FRESH/$f" ] && pass "fresh installed $f" || die "missing $f after fresh install"
done

# 008 must be in the created file list of the JSON summary.
json_array "$FRESH_JSON" createdFiles | grep -qx 'scripts/schema/008-intervention-review.sql' \
  && pass "summary createdFiles includes 008 schema" \
  || die "008 schema missing from summary createdFiles"

# ---------------------------------------------------------------------------
# 2. Merge over existing harness → never overwrites user stories/decisions/db.
# ---------------------------------------------------------------------------
# Seed user-owned artifacts the --merge promise must protect.
USER_STORY="$FRESH/docs/stories/US-USER-local.md"
USER_DECISION="$FRESH/docs/decisions/0099-user-local.md"
USER_DB="$FRESH/harness.db"
printf 'user story body\n' > "$USER_STORY"
printf 'user decision body\n' > "$USER_DECISION"
printf 'SQLITE-DB-BYTES\n' > "$USER_DB"
before_story="$(sha256_of "$USER_STORY")"
before_decision="$(sha256_of "$USER_DECISION")"
before_db="$(sha256_of "$USER_DB")"

MERGE_JSON="$WORK/merge.json"
if run_install --yes --merge --summary-json "$MERGE_JSON" "$FRESH" >/dev/null 2>&1; then
  pass "merge over existing exits 0"
else
  die "merge over existing failed"
fi

m_created="$(json_scalar "$MERGE_JSON" created)"
m_updated="$(json_scalar "$MERGE_JSON" updated)"
[ "${m_created:-x}" = "0" ] && pass "merge created 0 files" || die "merge created ${m_created} (expected 0)"
[ "${m_updated:-x}" = "0" ] && pass "merge updated 0 files" || die "merge updated ${m_updated} (expected 0)"

[ "$(sha256_of "$USER_STORY")" = "$before_story" ] && pass "merge left user story untouched" || die "merge modified user story"
[ "$(sha256_of "$USER_DECISION")" = "$before_decision" ] && pass "merge left user decision untouched" || die "merge modified user decision"
[ "$(sha256_of "$USER_DB")" = "$before_db" ] && pass "merge left harness.db untouched" || die "merge modified harness.db"

# ---------------------------------------------------------------------------
# 3. Re-run idempotent → a second identical --merge produces the same summary.
# ---------------------------------------------------------------------------
MERGE2_JSON="$WORK/merge2.json"
run_install --yes --merge --summary-json "$MERGE2_JSON" "$FRESH" >/dev/null 2>&1 || die "second merge failed"

# Compare the counts + file lists (ignore the volatile "target" absolute path,
# which is identical here anyway).
if [ "$(json_scalar "$MERGE_JSON" created)" = "$(json_scalar "$MERGE2_JSON" created)" ] &&
   [ "$(json_scalar "$MERGE_JSON" skipped)" = "$(json_scalar "$MERGE2_JSON" skipped)" ] &&
   diff <(json_array "$MERGE_JSON" skippedFiles) <(json_array "$MERGE2_JSON" skippedFiles) >/dev/null; then
  pass "re-run merge is idempotent (identical summary)"
else
  die "re-run merge summary differs from first merge"
fi

# ---------------------------------------------------------------------------
# 4. Dry-run parity → dry-run's planned created set == a real run's created set.
# ---------------------------------------------------------------------------
DRY_JSON="$WORK/dry.json"
REAL_JSON="$WORK/real.json"
DRY_DIR="$WORK/dry-target"
REAL_DIR="$WORK/real-target"

run_install --yes --dry-run --summary-json "$DRY_JSON" "$DRY_DIR" >/dev/null 2>&1 || die "dry-run failed"
[ ! -d "$DRY_DIR" ] || { [ -z "$(ls -A "$DRY_DIR" 2>/dev/null)" ] && true; }
if [ -d "$DRY_DIR" ] && [ -n "$(ls -A "$DRY_DIR" 2>/dev/null)" ]; then
  die "dry-run wrote files into target"
else
  pass "dry-run wrote no files"
fi

run_install --yes --summary-json "$REAL_JSON" "$REAL_DIR" >/dev/null 2>&1 || die "real run failed"

if diff <(json_array "$DRY_JSON" createdFiles) <(json_array "$REAL_JSON" createdFiles) >/dev/null; then
  pass "dry-run createdFiles matches real-run createdFiles"
else
  die "dry-run/real-run createdFiles diverge"
  diff <(json_array "$DRY_JSON" createdFiles) <(json_array "$REAL_JSON" createdFiles) >&2 || true
fi

# Every file the real run created must actually exist on disk.
real_missing=0
while IFS= read -r rel; do
  [ -n "$rel" ] || continue
  [ -e "$REAL_DIR/$rel" ] || { real_missing=1; echo "  real run summary lists $rel but it is absent" >&2; }
done < <(json_array "$REAL_JSON" createdFiles)
[ "$real_missing" -eq 0 ] && pass "real run createdFiles all exist on disk" || die "real run summary lists absent files"

# ---------------------------------------------------------------------------
# 5. Exit-code contract.
# ---------------------------------------------------------------------------
# 2 — bad input: unknown option.
expect_exit 2 "unknown option → validation exit" run_install --bogus-flag "$WORK/nope"
# 2 — bad input: --summary-json without a value.
expect_exit 2 "missing --summary-json value → validation exit" run_install --summary-json
# 2 — validation: protected paths present, non-interactive, no --merge/--override.
expect_exit 2 "protected-path conflict without --merge → validation exit" run_install --yes "$FRESH"

# 3 — IO: target directory not writable (skip when running as root, which bypasses perms).
if [ "$(id -u)" -ne 0 ]; then
  RO_PARENT="$WORK/readonly"
  mkdir -p "$RO_PARENT"
  chmod 555 "$RO_PARENT"
  expect_exit 3 "unwritable target → IO exit" run_install --yes "$RO_PARENT/child"
  chmod 755 "$RO_PARENT"
else
  echo "skip - unwritable-target IO test (running as root)"
fi

echo
echo "== results: $PASS passed, $FAIL failed =="
[ "$FAIL" -eq 0 ]
