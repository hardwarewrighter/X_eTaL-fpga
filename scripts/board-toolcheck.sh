#!/usr/bin/env bash
# Toolchain fixture, not an X_eTaL example. Build only; never load.
set -euo pipefail
trap 'echo "board-toolcheck failed; inspect work/board/toolcheck/*.log" >&2' ERR
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
cat > work/board/toolcheck/top.tb.v <<'VERILOG'
`timescale 1ns/1ps
module tb;
reg clk = 0;
reg rst_n = 0;
reg uart_rx = 1;
wire uart_tx, led;
toolcheck dut(clk, rst_n, uart_rx, uart_tx, led);
always #5 clk = ~clk;
integer i;
initial begin
    @(posedge clk); #1;
    if (dut.count !== 0 || led !== 1) $fatal(1, "reset failed");
    @(negedge clk); rst_n = 1;
    repeat (5) @(posedge clk);
    #1;
    if (dut.count !== 5) $fatal(1, "clock counter failed");
    @(negedge clk); dut.count = 24'h7fffff;
    @(posedge clk); #1;
    if (led !== 0) $fatal(1, "LED transition failed");
    for (i = 0; i < 32; i = i + 1) begin
        uart_rx = i % 2; #2;
        if (uart_tx !== uart_rx) $fatal(1, "UART pin loopback failed");
    end
    @(negedge clk); rst_n = 0;
    @(posedge clk); #1;
    if (dut.count !== 0 || led !== 1) $fatal(1, "button reset failed");
    $display("PASS: fixture clock, reset, LED transition, UART pin loopback");
    $finish;
end
endmodule
VERILOG
if command -v iverilog >/dev/null && command -v vvp >/dev/null; then
    iverilog -g2005-sv -s tb -o work/board/toolcheck/top.vvp work/board/toolcheck/top.v work/board/toolcheck/top.tb.v
    vvp work/board/toolcheck/top.vvp > work/board/toolcheck/sim.log 2>&1
    cat work/board/toolcheck/sim.log
else
    echo "board-toolcheck: SKIPPED fixture simulation (iverilog/vvp missing)"
fi
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
