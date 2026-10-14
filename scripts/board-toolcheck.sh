#!/usr/bin/env bash
# Toolchain fixture, not an X_eTaL example. Build only; never load.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
for tool in yosys nextpnr-himbaechel gowin_pack; do
    command -v "$tool" >/dev/null || { echo "board-toolcheck: SKIPPED ($tool missing; activate OSS CAD Suite)"; exit 0; }
done
mkdir -p work/board/toolcheck
cat > work/board/toolcheck/top.v <<'VERILOG'
module toolcheck(input clk, input rst_n, input uart_rx, output uart_tx, output led);
reg [23:0] count;
always @(posedge clk)
    if (!rst_n) count <= 0;
    else count <= count + 1'b1;
assign led = ~count[23];
assign uart_tx = uart_rx;
endmodule
VERILOG
cat boards/tangnano9k/uart.cst > work/board/toolcheck/top.cst
cat >> work/board/toolcheck/top.cst <<'CST'
IO_LOC "led" 10;
IO_PORT "led" IO_TYPE=LVCMOS33 PULL_MODE=UP DRIVE=8;
CST
cli=target/release/xetal-fpga
"$cli" synth work/board/toolcheck/top.v --top toolcheck --cst work/board/toolcheck/top.cst --out work/board/toolcheck/synth.json > work/board/toolcheck/synth.log 2>&1
"$cli" pnr work/board/toolcheck/synth.json --cst work/board/toolcheck/top.cst --out work/board/toolcheck/pnr.json > work/board/toolcheck/pnr.log 2>&1
"$cli" pack work/board/toolcheck/pnr.json --out work/board/toolcheck/toolcheck.fs > work/board/toolcheck/pack.log 2>&1
echo "board-toolcheck: built work/board/toolcheck/toolcheck.fs; LED counter plus direct UART loopback; not loaded"
