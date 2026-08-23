#!/usr/bin/env bash
# Lint change-diagram files against docs/DIAGRAMS.md.
#
# Usage:
#   bash scripts/check-diagrams.sh            # every docs/**/diagrams/D*.md
#   bash scripts/check-diagrams.sh <file>...  # specific files
#
# Exits 1 on any failure. Checks structure only; whether a diagram matches the
# code is the done-gate review's job.

set -u

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

if [ "$#" -gt 0 ]; then
  files=("$@")
else
  files=()
  while IFS= read -r f; do files+=("$f"); done < <(find docs -path '*/diagrams/D[1-7]-*.md' -type f | sort)
fi

if [ "${#files[@]}" -eq 0 ]; then
  echo "check-diagrams: no diagram files found"
  exit 0
fi

fail=0
err() { echo "FAIL $1: $2"; fail=1; }

kind_for_prefix() {
  case "$1" in
    D1) echo blast-radius ;;
    D2) echo component-delta ;;
    D3) echo sequence ;;
    D4) echo state ;;
    D5) echo data-model ;;
    D6) echo story-dag ;;
    D7) echo boundary ;;
    *) echo "" ;;
  esac
}

# Accepted first lines of the mermaid fence per prefix (regex).
mermaid_for_prefix() {
  case "$1" in
    D1) echo '^flowchart LR$' ;;
    D2) echo '^(flowchart TB|C4Component)$' ;;
    D3) echo '^sequenceDiagram$' ;;
    D4) echo '^stateDiagram-v2$' ;;
    D5) echo '^erDiagram$' ;;
    D6) echo '^flowchart LR$' ;;
    D7) echo '^(flowchart LR|C4Container)$' ;;
  esac
}

field() { grep -m1 -E "^$1: " "$2" | sed -E "s/^$1: //"; }

for f in "${files[@]}"; do
  base="$(basename "$f")"
  prefix="${base%%-*}"
  expected_kind="$(kind_for_prefix "$prefix")"

  if [ -z "$expected_kind" ] || ! [[ "$base" =~ ^D[1-7]-[a-z0-9][a-z0-9-]*\.md$ ]]; then
    err "$f" "filename must be D<1-7>-<slug>.md"
    continue
  fi
  case "$f" in */diagrams/*) ;; *) err "$f" "must live in a diagrams/ folder" ;; esac

  head -1 "$f" | grep -qE "^# $prefix " || err "$f" "first line must start with '# $prefix '"

  for key in Story Kind Source Scope Status Reviewed-by Reviewed-at; do
    grep -qE "^$key: " "$f" || err "$f" "missing header field '$key'"
  done

  kind="$(field Kind "$f")"
  [ "$kind" = "$expected_kind" ] || err "$f" "Kind '$kind' does not match prefix $prefix ($expected_kind)"

  source="$(field Source "$f")"
  case "$source" in generated:codegraph|generated:c3|hand) ;; *) err "$f" "Source must be generated:codegraph, generated:c3, or hand (got '$source')" ;; esac

  status="$(field Status "$f")"
  case "$status" in draft|reviewed|stale) ;; *) err "$f" "Status must be draft, reviewed, or stale (got '$status')" ;; esac

  by="$(field Reviewed-by "$f")"
  at="$(field Reviewed-at "$f")"
  if [ "$status" = "reviewed" ]; then
    [[ "$by" =~ ^(human|agent|ci):.+ ]] || err "$f" "reviewed file needs Reviewed-by: human:<name>|agent:<name>|ci:<job>"
    [[ "$at" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || err "$f" "reviewed file needs Reviewed-at: YYYY-MM-DD"
  fi

  if [ "$prefix" = "D1" ]; then
    grep -qE '^Coverage: [0-9]+ of [0-9]+$' "$f" || err "$f" "D1 needs 'Coverage: N of M'"
  fi

  fences="$(grep -c '^```mermaid$' "$f")"
  [ "$fences" -eq 1 ] || err "$f" "expected exactly one \`\`\`mermaid fence (found $fences)"
  if [ "$fences" -ge 1 ]; then
    first="$(awk '/^```mermaid$/{getline; print; exit}' "$f" | sed -E 's/^[[:space:]]+//')"
    echo "$first" | grep -qE "$(mermaid_for_prefix "$prefix")" || err "$f" "mermaid type '$first' is not allowed for $prefix"
  fi

  grep -q '^## What to review' "$f" || err "$f" "missing '## What to review' section"
  grep -q '^## Derived next steps' "$f" || err "$f" "missing '## Derived next steps' section"
done

if [ "$fail" -eq 0 ]; then
  echo "check-diagrams: ${#files[@]} file(s) ok"
fi
exit "$fail"
