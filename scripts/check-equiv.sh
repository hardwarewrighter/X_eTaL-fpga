#!/usr/bin/env bash
# The equivalence checks: every hand-lowered twin, libs/*/demos/*.xir,
# run by the XIR interpreter (xetal-fpga run) and as a simulated
# circuit (xetal-fpga sim) in the spatial schedule and with 1, 2 and 3
# lanes (a clocked twin, one with state: clocked from reset, its one
# schedule), must print exactly the lines of its demo's reg-rs baseline
# that it names in a comment,
#   # baseline: demo-NAME LINES      (LINES: 2, 1-3, 2,4,5)
# where demo-NAME.out is the pinned evaluator's output of the .xtl
# beside it. Ints are compared exactly (plan A2); a twin's schedule
# data, "# schedule: OPTIONS" (as --memories), is passed to its
# simulated circuit. A twin without a
# baseline line, or naming a missing baseline, fails.
#   scripts/check-equiv.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Generated inputs are current: HEX1000's IR (scripts/gen-hex1000-xir.py)
# and its ROM image (scripts/rom-hex.sh).
"$root/scripts/gen-hex1000-xir.py" --check
"$root/scripts/rom-hex.sh" --check
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
  # A clocked twin (one with state) has one schedule; the lanes are for vectors.
  sched="$(sed -n 's/^# schedule: *//p' "$twin" | head -1)"
  hows=("run" "sim" "sim --lanes 1" "sim --lanes 2" "sim --lanes 3")
  grep -q '= state ' "$twin" && hows=("run" "sim $sched")
  for how in "${hows[@]}"; do
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
echo "check-equiv: $n twin(s), the XIR interpreter and the simulated circuits (spatial; 1, 2, 3 lanes; clocked) against the evaluator$([ $fail = 0 ] && echo ', all equal' || echo ', DIFFERENCES')"
exit $fail
