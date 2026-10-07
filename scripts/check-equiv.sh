#!/usr/bin/env bash
# The equivalence checks: every hand-lowered twin, libs/*/demos/*.xir,
# run by the XIR interpreter (xetal-fpga run) and as a simulated
# circuit (xetal-fpga sim) in the spatial schedule and with 1, 2 and 3
# lanes, must print exactly the lines of its demo's reg-rs baseline
# that it names in a comment,
#   # baseline: demo-NAME LINES      (LINES: 2, 1-3, 2,4,5)
# where demo-NAME.out is the pinned evaluator's output of the .xtl
# beside it. Ints are compared exactly (plan A2). A twin without a
# baseline line, or naming a missing baseline, fails.
#   scripts/check-equiv.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
(cd "$root/components" && cargo build -q -p xetal-fpga-cli)
tool="$root/target/debug/xetal-fpga"
n=0; fail=0
for twin in "$root"/libs/*/demos/*.xir; do
  [ -e "$twin" ] || continue
  n=$((n + 1))
  lib="$(basename "$(dirname "$(dirname "$twin")")")"
  rel="${twin#$root/}"
  spec="$(sed -n 's/^# baseline: *//p' "$twin" | head -1)"
  [ -n "$spec" ] || { echo "FAIL: $rel: no '# baseline: demo-NAME LINES' comment"; fail=1; continue; }
  read -r base lines <<<"$spec"
  out="$root/libs/$lib/tests/$base.out"
  [ -f "$out" ] || { echo "FAIL: $rel: no baseline libs/$lib/tests/$base.out"; fail=1; continue; }
  want="$(python3 - "$out" "$lines" <<'PY'
import sys
text = open(sys.argv[1]).read().splitlines()
picked = []
for part in sys.argv[2].split(","):
    a, _, b = part.partition("-")
    picked += range(int(a), int(b or a) + 1)
print("\n".join(text[i - 1] for i in picked))
PY
)"
  for how in "run" "sim" "sim --lanes 1" "sim --lanes 2" "sim --lanes 3"; do
    read -r cmd opts <<<"$how"
    # shellcheck disable=SC2086
    if ! got="$("$tool" $cmd "$twin" $opts 2>/dev/null)"; then
      echo "FAIL: $rel: xetal-fpga $how: $("$tool" $cmd "$twin" $opts 2>&1 >/dev/null)"; fail=1; continue
    fi
    if [ "$got" != "$want" ]; then
      echo "FAIL: $rel ($how) does not reproduce $base lines $lines"
      diff <(echo "$want") <(echo "$got") | sed 's/^/  /' || true
      fail=1
    fi
  done
done
echo "check-equiv: $n twin(s), the XIR interpreter and the simulated circuits (spatial; 1, 2, 3 lanes) against the evaluator$([ $fail = 0 ] && echo ', all equal' || echo ', DIFFERENCES')"
exit $fail
