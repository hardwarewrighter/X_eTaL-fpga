#!/usr/bin/env bash
# HEX1000's calculator ROM as a $readmemh image: 1024 lines, one
# two-digit hexadecimal byte each, address 64 PA + PC in order, from the
# pinned evaluator (hx:CalcRom, the assembler's output). Pinned as
# libs/Hex1000/rom/calc.hex; --check fails when it is stale (the gate).
#   scripts/rom-hex.sh [--check]
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/libs/Hex1000/rom/calc.hex"
tmp="$(mktemp)"; trap 'rm -f "$tmp" "$tmp.xtl"' EXIT
printf '%s\n' '"hx:" u_se< "Hex1000"' 'hx:CalcRom' > "$tmp.xtl"
"$root/scripts/xt" run "$tmp.xtl" | tr ' ' '\n' | grep -v '^$' | while read -r b; do printf '%02x\n' "$b"; done > "$tmp"
[ "$(wc -l < "$tmp" | tr -d ' ')" = 1024 ] || { echo "rom-hex: expected 1024 bytes" >&2; exit 1; }
if [ "${1:-}" = --check ]; then
  cmp -s "$tmp" "$out" || { echo "FAIL: libs/Hex1000/rom/calc.hex is stale (scripts/rom-hex.sh)"; exit 1; }
  echo "rom-hex: libs/Hex1000/rom/calc.hex matches hx:CalcRom"
else
  cp "$tmp" "$out"; echo "rom-hex: wrote libs/Hex1000/rom/calc.hex"
fi
