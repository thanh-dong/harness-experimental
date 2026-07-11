#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: build-harness-dist.sh [options]

Package the Harness operating files (everything install-harness.sh fetches
from the repo, minus the per-platform CLI binary) into a single checksummed
distribution bundle:

  dist/harness-dist-<version>.tar.gz
  dist/harness-dist-<version>.tar.gz.sha256

The bundle lets a consumer install fully offline. Extract it to a directory
and point the installer at it:

  tar -xzf harness-dist-<version>.tar.gz -C /tmp/harness-src
  HARNESS_SOURCE_BASE_URL="file:///tmp/harness-src" \
    HARNESS_CLI_BASE_URL="file:///path/to/binaries" \
    scripts/install-harness.sh --yes /path/to/target

Options:
      --version <v>    Bundle version label. Defaults to the tag in
                       scripts/harness-cli-release-tag (with the
                       "harness-cli-" prefix stripped).
      --out-dir <dir>  Output directory. Defaults to dist.
      --check          Verify that a previously built bundle's contents match
                       the installer file list, then exit. No rebuild.
      --list           Print the derived file list and exit.
  -h, --help           Show this help.

Parity: the bundle contains exactly the files install-harness.sh copies from
the repo (its embedded file list). --check enforces this and is the CI gate.
EOF
}

fail() {
  printf 'Error: %s\n' "$*" >&2
  exit 1
}

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
installer="$repo_root/scripts/install-harness.sh"
version=""
out_dir="$repo_root/dist"
mode="build"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      [ "$#" -ge 2 ] || fail "$1 requires a value"
      version="$2"
      shift 2
      ;;
    --out-dir)
      [ "$#" -ge 2 ] || fail "$1 requires a path"
      out_dir="$2"
      shift 2
      ;;
    --check)
      mode="check"
      shift
      ;;
    --list)
      mode="list"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "Unknown option: $1"
      ;;
  esac
done

[ -f "$installer" ] || fail "Installer not found: $installer"

# The single source of truth for bundle contents is the file list embedded in
# install-harness.sh (the `done <<'EOF' ... EOF` heredoc feeding copy_file).
# Deriving from it here guarantees bundle == installer parity by construction.
derive_file_list() {
  awk '
    /^done <<.?EOF.?$/ { capture = 1; next }
    capture && /^EOF$/ { capture = 0; next }
    capture { print }
  ' "$installer"
}

resolve_version() {
  if [ -n "$version" ]; then
    printf '%s\n' "$version"
    return
  fi
  local tag_file="$repo_root/scripts/harness-cli-release-tag"
  local tag=""
  [ -f "$tag_file" ] && tag="$(awk 'NF && $1 !~ /^#/ { print $1; exit }' "$tag_file")"
  [ -n "$tag" ] || fail "Could not resolve version: pass --version or populate scripts/harness-cli-release-tag"
  printf '%s\n' "${tag#harness-cli-}"
}

sha256_write() {
  local file="$1"
  local dir base
  dir="$(dirname "$file")"
  base="$(basename "$file")"
  if command -v shasum >/dev/null 2>&1; then
    (cd "$dir" && shasum -a 256 "$base" > "$base.sha256")
  elif command -v sha256sum >/dev/null 2>&1; then
    (cd "$dir" && sha256sum "$base" > "$base.sha256")
  else
    fail "shasum or sha256sum is required to write checksums"
  fi
}

files="$(derive_file_list)"
[ -n "$files" ] || fail "Derived an empty file list from $installer"

if [ "$mode" = "list" ]; then
  printf '%s\n' "$files"
  exit 0
fi

if [ "$mode" = "check" ]; then
  version="$(resolve_version)"
  tarball="$out_dir/harness-dist-$version.tar.gz"
  [ -f "$tarball" ] || fail "Bundle not found for check: $tarball"

  expected="$(printf '%s\n' "$files" | LC_ALL=C sort)"
  actual="$(tar -tzf "$tarball" | grep -v '/$' | LC_ALL=C sort)"

  if [ "$expected" != "$actual" ]; then
    printf 'Parity check FAILED: bundle contents != installer file list\n' >&2
    printf -- '--- only in installer list ---\n' >&2
    comm -23 <(printf '%s\n' "$expected") <(printf '%s\n' "$actual") >&2
    printf -- '--- only in bundle ---\n' >&2
    comm -13 <(printf '%s\n' "$expected") <(printf '%s\n' "$actual") >&2
    exit 1
  fi

  # Checksum integrity when a sidecar exists.
  if [ -f "$tarball.sha256" ]; then
    if command -v shasum >/dev/null 2>&1; then
      (cd "$out_dir" && shasum -a 256 -c "harness-dist-$version.tar.gz.sha256" >/dev/null) \
        || fail "Checksum verification failed for $tarball"
    elif command -v sha256sum >/dev/null 2>&1; then
      (cd "$out_dir" && sha256sum -c "harness-dist-$version.tar.gz.sha256" >/dev/null) \
        || fail "Checksum verification failed for $tarball"
    fi
  fi

  printf 'Parity OK: %s matches the installer file list (%s files).\n' \
    "$(basename "$tarball")" "$(printf '%s\n' "$files" | wc -l | tr -d ' ')"
  exit 0
fi

# mode = build
version="$(resolve_version)"

missing=0
while IFS= read -r relative; do
  [ -n "$relative" ] || continue
  if [ ! -f "$repo_root/$relative" ]; then
    printf 'Missing source file: %s\n' "$relative" >&2
    missing=1
  fi
done <<EOF
$files
EOF
[ "$missing" -eq 0 ] || fail "Bundle sources are incomplete; refusing to build."

mkdir -p "$out_dir"
tarball="$out_dir/harness-dist-$version.tar.gz"

# Deterministic member ordering (sorted) so the bundle is reproducible.
file_args="$(printf '%s\n' "$files" | LC_ALL=C sort)"

# Use a file list to avoid arg-length limits and keep paths repo-relative.
list_tmp="$(mktemp)"
printf '%s\n' "$file_args" > "$list_tmp"
tar -czf "$tarball" -C "$repo_root" -T "$list_tmp"
rm -f "$list_tmp"

sha256_write "$tarball"

printf 'Built %s\n' "$tarball"
printf 'Wrote %s.sha256\n' "$tarball"

# Self-verify parity immediately after building.
"$0" --version "$version" --out-dir "$out_dir" --check
