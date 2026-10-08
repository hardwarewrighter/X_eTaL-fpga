#!/usr/bin/env bash
# A waveform of one twin's circuit: its Verilog and self-checking
# testbench run under iverilog, the waveform written to
# work/rtl/NAME.vcd (open it with gtkwave). Needs iverilog and vvp.
#   scripts/wave.sh libs/Clocked/demos/detect.xir        # spatial (or clocked)
#   scripts/wave.sh libs/Datapath/demos/reduce-sum.xir 1 # one lane
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
twin="${1:?usage: wave.sh FILE.xir [LANES]}"
[ -f "$twin" ] || twin="$root/$twin"
opts=(); [ -n "${2:-}" ] && opts=(--lanes "$2")
for t in iverilog vvp; do
  command -v "$t" >/dev/null || { echo "wave: $t is not installed (brew install icarus-verilog; on Arch: pacman -S iverilog)" >&2; exit 1; }
done
(cd "$root/components" && cargo build -q -p xetal-fpga-cli)
tool="$root/target/debug/xetal-fpga"
scratch="$root/work/rtl"; mkdir -p "$scratch"
name="$(basename "$twin" .xir)"
"$tool" verilog "$twin" ${opts[@]+"${opts[@]}"} > "$scratch/$name.v"
"$tool" testbench "$twin" ${opts[@]+"${opts[@]}"} > "$scratch/$name.tb.v"
iverilog -g2005 -o "$scratch/$name.vvp" "$scratch/$name.v" "$scratch/$name.tb.v"
(cd "$scratch" && vvp -n "$name.vvp")
module="$(sed -n 's/^module \([A-Za-z0-9_]*\) (.*/\1/p' "$scratch/$name.v")"
echo "wave: work/rtl/$module.vcd (gtkwave work/rtl/$module.vcd)"
