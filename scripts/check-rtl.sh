#!/usr/bin/env bash
# The generated circuits: for every hand-lowered twin,
# libs/<Lib>/demos/NAME.xir, the Verilog of the spatial schedule and of
# one lane is pinned as libs/<Lib>/rtl/NAME.v and NAME.lanes1.v (a
# clocked twin, one with state, only NAME.v), and
# must be what xetal-fpga writes today (XETAL_BLESS=1 rewrites them:
# review the diff). Then, when the tools are installed (plan A10; they
# are optional and never installed from here):
#   iverilog: each pinned module with its self-checking testbench
#             (xetal-fpga testbench) must print PASS;
#   yosys:    each pinned module must synthesize (synth, generic).
# A missing tool is reported as skipped, never as passed.
#   scripts/check-rtl.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
(cd "$root/components" && cargo build -q -p xetal-fpga-cli)
tool="$root/target/debug/xetal-fpga"
scratch="$root/work/rtl"; mkdir -p "$scratch"
have() { command -v "$1" >/dev/null; }
n=0; fail=0; sims=0; synths=0
for twin in "$root"/libs/*/demos/*.xir; do
  [ -e "$twin" ] || continue
  dir="$(dirname "$(dirname "$twin")")/rtl"; mkdir -p "$dir"
  name="$(basename "$twin" .xir)"
  scheds=(spatial lanes1)
  grep -q '= state ' "$twin" && scheds=(spatial)   # a clocked twin has one schedule
  for sched in "${scheds[@]}"; do
    if [ "$sched" = lanes1 ]; then opts=(--lanes 1); suffix=.lanes1; else opts=(); suffix=; fi
    pinned="$dir/$name$suffix.v"
    rel="${pinned#$root/}"
    n=$((n + 1))
    got="$("$tool" verilog "$twin" ${opts[@]+"${opts[@]}"})"
    if [ "${XETAL_BLESS:-}" = 1 ]; then
      printf '%s\n' "$got" > "$pinned"
    elif [ ! -f "$pinned" ]; then
      echo "FAIL: $rel: not pinned (XETAL_BLESS=1 scripts/check-rtl.sh)"; fail=1; continue
    elif [ "$got" != "$(cat "$pinned")" ]; then
      echo "FAIL: $rel differs from what xetal-fpga writes"
      diff "$pinned" <(printf '%s\n' "$got") | head -20 | sed 's/^/  /' || true
      fail=1; continue
    fi
    tb="$scratch/$name.$sched.tb.v"
    "$tool" testbench "$twin" ${opts[@]+"${opts[@]}"} > "$tb"
    if have iverilog && have vvp; then
      out="$scratch/$name.$sched.vvp"
      if iverilog -g2005 -o "$out" "$pinned" "$tb" 2>"$scratch/iverilog.log" && vvp -n "$out" | grep -q '^PASS'; then
        sims=$((sims + 1))
      else
        echo "FAIL: $rel: iverilog testbench"; cat "$scratch/iverilog.log"; vvp -n "$out" 2>&1 | head -5; fail=1
      fi
    fi
    if have yosys; then
      top="$(sed -n 's/^module \([A-Za-z0-9_]*\) (.*/\1/p' "$pinned")"
      if yosys -q -p "read_verilog $pinned; synth -top $top" >"$scratch/yosys.log" 2>&1; then
        synths=$((synths + 1))
      else
        echo "FAIL: $rel: yosys synth"; tail -5 "$scratch/yosys.log"; fail=1
      fi
    fi
  done
done
have iverilog || echo "check-rtl: SKIPPED iverilog simulation (not installed: brew install icarus-verilog)"
have yosys || echo "check-rtl: SKIPPED yosys synthesis (not installed: brew install yosys)"
note=; [ "${XETAL_BLESS:-}" = 1 ] && note=' (rewritten)'
echo "check-rtl: $n circuit(s) pinned$note, $sims simulated by iverilog, $synths synthesized by yosys$([ $fail = 0 ] && echo ', ok' || echo ', FAILURES')"
exit $fail
