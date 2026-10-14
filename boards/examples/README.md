# X_eTaL board proof

`counter.xtl` calls Clocked's 8-bit tick for 512 enables, including
two full wraps. `probe.xtl` uses Uart's receive/transmit steps and
Clocked's tick: each received byte advances a counter from 120.
A question mark returns the new counter byte; every other byte
echoes unchanged. The script uses three clocks per UART bit as a
short test harness; hardware timing comes from board.json.

Each program has a hand-lowered XIR twin and a reviewed evaluator
baseline (.out). `just board-probe-check` regenerates reference
streams from the X_eTaL output, compares every traced clock against
the IR interpreter and netlist simulator, and runs the emitted
Verilog's self-checking testbench when Icarus is installed. It also
decodes the reference UART frames and checks their response bytes.

The Nano 9K top instantiates the emitted cores. Clock division,
startup reset, reset/receive synchronization and pins are board glue;
the UART state machines and fixed-width arithmetic are generated.
The LED counter ticks once per 65536 board clocks, giving about 1.6
blinks per second. UART uses 234 board clocks per bit at 27 MHz.

```bash
just board-probe-check
just board-probe-build
just board-probe-test
```

The hardware test sends 512 queries and compares the answers with
the pinned evaluator's counter trace, then checks all 255 non-query
byte values as UART echoes. It sends one request at a time because
this example has no UART FIFO. `board-probe-test` loads SRAM before
testing so the state starts from its defined reset value.

After a successful SRAM run, `just board-probe-flash` explicitly
writes persistent FPGA configuration. Power-cycle the board, then
`just board-probe-uart` checks the same results without loading a
configuration. Reset before repeating a UART-only test: the tests
advance the counter state. The SD card is not used.

This proves the current hand-lowered process from X_eTaL semantics
through the emitter and open FPGA tools to measured board results.
It does not provide automatic Core-to-XIR lowering (ask F2) or the
HEX1000 circuit.


## Verified results

The physical Nano 9K passes all 512 queries and 255 echoes in SRAM
and after explicit flash writing. The full board-top test also passes
against synthesized Gowin cell models at real clock/baud timing.
UART TX is registered at the pin to suppress combinational glitches.
The raw serial test keeps one connection open for both phases.

`just board-proof-video` records the real SRAM test using VHS into
work/tapes/nano9k-proof.mp4 and nano9k-proof.gif. The tape is
boards/tangnano9k/proof.tape. It records hardware results, not the
software calculator. VHS, ttyd, ffmpeg and Chromium are recording
prerequisites; the test/build scripts never install them. On this
machine VHS 0.12.1 is unpacked under /disk1/tmp/vhs-0.12.1.
