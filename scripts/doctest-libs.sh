#!/usr/bin/env bash
# The libraries' doc tests: every "## >>" example in libs/*/src runs
# (xetal doc --test, through scripts/xt so every library is on the
# path) and must print the "##" lines under it. Each ## block is one
# session, so an example imports its library itself, as a reader would.
#   scripts/doctest-libs.sh [Name...]
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
if [ $# -gt 0 ]; then names=("$@"); else
  names=(); while IFS= read -r s; do [ -n "$s" ] && names+=("$s"); done < <(scripts/libs.py list)
fi
total=0
for name in ${names[@]+"${names[@]}"}; do
  for f in "libs/$name/src/$name.xtl" "libs/$name/src/$name.xtlm"; do
    [ -f "$f" ] || continue
    out="$(scripts/xt doc --test "$f" 2>&1)" || { echo "$out"; echo "FAIL: doc tests in $f"; exit 1; }
    n="$(printf '%s\n' "$out" | sed -n 's/.*: \([0-9]*\) examples.*/\1/p' | head -1)"
    total=$((total + ${n:-0}))
  done
done
echo "doctest-libs: ${#names[@]} librar$([ ${#names[@]} = 1 ] && echo y || echo ies), $total examples, all passed"
