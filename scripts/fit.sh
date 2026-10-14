#!/usr/bin/env bash
# What each pinned circuit (libs/<Lib>/rtl/*.v) takes on a Gowin GW1N
# part, by Yosys's Gowin synthesis (synth_gowin -family gw1n, no I/O
# pads: a pinned module is not yet a board's top level), against the
# Tang Nano 1K (GW1NZ-1: 1152 LUT4, 864 flip-flops, 4 block RAMs, no
# multipliers). LUTs counts LUT1-LUT4 and the ALU cells of the carry
# chains (each takes a LUT's place). An estimate before placement:
# nextpnr's numbers on the Arch machine are the real ones. Fails if a
# module does not synthesize; yosys missing is reported as skipped
# (plan A10; never installed from here).
#   scripts/fit.sh [NAME...]   (default: every pinned circuit)
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if ! command -v yosys >/dev/null; then
  echo "fit: SKIPPED Gowin utilization (yosys not installed: brew install yosys)"; exit 0
fi
scratch="$root/work/rtl"; mkdir -p "$scratch"
files=()
if [ $# -gt 0 ]; then
  for n in "$@"; do f="$(ls "$root"/libs/*/rtl/"$n".v 2>/dev/null | head -1)"
    [ -n "$f" ] || { echo "fit: no pinned circuit $n"; exit 1; }; files+=("$f"); done
else
  files=("$root"/libs/*/rtl/*.v)
fi
printf '%-28s %6s %6s %6s %6s %6s\n' circuit LUTs FFs ALUs BSRAM MULT
fail=0
for f in "${files[@]}"; do
  rel="${f#$root/libs/}"; name="${rel%.v}"; name="${name/\/rtl\//:}"
  top="$(sed -n 's/^module \([A-Za-z0-9_]*\) (.*/\1/p' "$f")"
  log="$scratch/fit.$(basename "$f" .v).log"
  if ! yosys -p "read_verilog $f; synth_gowin -family gw1n -noiopads -top $top; stat" >"$log" 2>&1; then
    echo "FAIL: $rel: synth_gowin"; tail -5 "$log"; fail=1; continue
  fi
  # The last statistics block is the flattened top.
  stat="$(awk '/Printing statistics/{s=""} {s=s $0 "\n"} END{printf "%s", s}' "$log")"
  count() { printf '%s' "$stat" | awk -v re="$1" '$2 ~ re {n += $1} END {print n+0}'; }
  luts=$(count '^LUT[1-4]$'); alus=$(count '^ALU$'); ffs=$(count '^DFF')
  bram=$(count '^(SP|SPX9|SDPB|SDPX9B|DPB|DPX9B|pROM|pROMX9)$'); mult=$(count '^MULT')
  printf '%-28s %6s %6s %6s %6s %6s\n' "$name" $((luts + alus)) "$ffs" "$alus" "$bram" "$mult"
done
printf '%-28s %6s %6s %6s %6s %6s\n' "Tang Nano 1K holds" 1152 864 - 4 0
exit $fail
