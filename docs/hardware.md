# Hardware: Verilog, checking it in software, and testing on a board

How a circuit from this repo is checked before it reaches a board,
how it gets to a Tang board, and what can and cannot go wrong there.
The boards themselves are in `docs/boards.md`; the plan's steps are in
`docs/plan.md`.

## Current board status (2026-10-14)

On the attached Nano 9K, JTAG identification, SRAM load with CRC,
542-byte exact UART loopback at 115200, LED blink and button reset
are verified. Persistent flash programming also passed with CRC;
after the user unplugged/replugged, LED/reset resumed and 541 UART
bytes passed without a bitstream load. The USB rule is installed.
The first fixture proves tools and board connection. The emitted
X_eTaL counter/UART now also passes 512 evaluator-matched counter
queries and 255 echo bytes in SRAM and after explicit flash writing.
HEX1000 now passes three calculator sequences on actual 9K SRAM over UART.
See [hardware demo and known issues](hex1000-hardware.md). The inserted SD
card is not used. See [first-board setup](../boards/tangnano9k/README.md).

## Verilog first (and VHDL perhaps never)

The emitter writes plain Verilog-2005 (plan A7), and that is the only
hardware language this repo produces. Reasons:

- Yosys, the synthesis step of the open Gowin flow, reads Verilog
  itself; VHDL needs a separate front end (GHDL and its Yosys plugin).
- The simulators used here, Icarus Verilog (iverilog) and Verilator,
  are Verilog simulators.
- The Gowin IDE, needed for the Tang Primer 25K until the open flow
  covers the GW5A (`docs/boards.md`), reads Verilog as well as VHDL.

The emitter prints the netlist (`components/rtl`), not X_eTaL, so a
VHDL printer would be a second small printer over the same netlist,
not a redesign. It is not planned; it is added only if someone asks
for VHDL.

## Checking a circuit in software

Every circuit is checked at increasing depth, each layer against the
one before, all of it without a board:

| Depth | What runs | Tool | Where |
| ----- | --------- | ---- | ----- |
| 1. Specification | the `.xtl` program | the pinned `xetal` evaluator | reg-rs baselines (`just test-lib`) |
| 2. IR | the hand-lowered `.xir` twin | the IR interpreter (`components/xir`) | `just equiv` |
| 3. Netlist | the scheduled cells | our netlist simulator (`components/rtl`) | `just equiv` |
| 4. Verilog | the emitted, pinned `.v` | iverilog with the self-checking testbench; Verilator lint; Yosys generic synthesis | `just rtl` |
| 5. Gowin cells | what Yosys maps to LUTs, flip-flops, ALUs, block RAM | `yosys synth_gowin` | `just fit` |
| 6. Gowin cells, simulated | the mapped netlist | iverilog with Yosys's Gowin cell models, or our own cell simulator | planned (saga fpga-board) |
| 7. Bitstream | the placed, routed and packed `.fs` | Apicula's unpacker back to Verilog, then simulated | planned; to confirm |

The testbench checks every output bit for bit against the evaluator's
answer (for a clocked circuit after reset and after every clock) and
writes a waveform (`just wave FILE.xir`, `work/rtl/NAME.vcd`, viewed
in GTKWave or Surfer).

**Status (2026-10-14).** The tools are installed on the Mac
(`brew install icarus-verilog verilator yosys`). On their first run they
found a bug the netlist simulator could not see: an output that was
also traced (`output %s`, `trace %s`) was declared as two ports of the
same name, which iverilog rejects. It is fixed. All 17 pinned
modules now pass the iverilog testbench, Verilator's lint (warnings
only: unused cells, an index wider than its table) and Yosys's
synthesis.

**What `just fit` shows today** (Yosys's estimate before placement,
GW1N, no I/O pads):

- The clocked circuits are small (8 to 64 flip-flops); the UART
  loopback takes about 225 LUTs and 194 flip-flops, a fifth of the
  Tang Nano 1K.
- For eight elements, one lane (`--lanes 1`) is larger than eight:
  the counter, the selection muxes and the output registers cost more
  than the seven adders they save. A temporal schedule pays off for
  long vectors and wide units, not for eight additions.
- saxpy and pipeline use 8 hardware multipliers spatially; the 1K has
  none, so there they would be built from LUTs.

### The HEX1000 in software

The same layers apply to the machine (`docs/machine.md`):

1. The X_eTaL machine (step machine-lib) is the specification: for a
   ROM program and a key sequence it gives the register trace at
   every instruction (A, Y, X, PC, the status, the outputs) and the
   display.
2. Its `.xir` twin, the IR interpreter and the netlist simulator give
   the same trace.
3. The Verilog, under iverilog, gives the same trace at every clock;
   Verilator compiles it for long runs (an 8-digit multiplication is
   thousands of instructions).
4. The terminal is checked at the bit level: the testbench drives the
   receive line one bit time at a time with the key bytes and decodes
   the transmit line, so the UART is inside the same check.
5. `just fit` says whether it fits the 1K before a board is touched.

One emitter change is needed first: memories with a computed address.
Today a `put` or `lookup` at a computed index becomes comparators and
muxes, and the 64 by 4-bit RAM held in flip-flops would take 256 of
the 1K's 864. The emitter needs a memory cell, written the way Yosys
maps to block RAM (a `reg` array, a synchronous write, a registered
read), and the 1024 by 8-bit ROM the same way (one of the 1K's four
block RAMs). That is part of step machine-rtl.

## Simulators, not emulators

Nothing runs a Gowin bitstream the way QEMU runs a CPU. What exists
are simulators at the depths in the table above:

- **Behavioral**: iverilog (an event-driven Verilog simulator),
  Verilator (Verilog compiled to a fast cycle-accurate C++ program),
  Yosys's CXXRTL.
- **After synthesis**: Yosys can write the netlist of Gowin cells,
  and iverilog simulates it with the cell models Yosys ships
  (`share/yosys/gowin/cells_sim.v`).
- **The bitstream**: Apicula, the open Gowin database, has an
  unpacker that turns a bitstream back into a netlist; simulating
  that checks placement, routing and packing as well. To confirm on
  the Arch machine.

**Building one.** We already have the first: the netlist simulator in
`components/rtl`. The useful next one is a cell-level simulator in
Rust: read Yosys's JSON after `synth_gowin` and simulate its cells
(LUT1-LUT4, the flip-flops, the ALU carry chain, the wide-LUT muxes,
block RAM) against the same traces. It checks Yosys's mapping without
any outside simulator and is a moderate component. A simulator of
the routed fabric from the bitstream is much larger, and the unpack
round trip covers it more cheaply.

## Testing on a board

On the Arch machine, with the open flow (`docs/boards.md`):

```
yosys -p "read_verilog hex1000.v; synth_gowin -top hex1000 -json hex1000.json"
nextpnr-himbaechel --json hex1000.json --write pnr.json \
  --device GW1NZ-LV1QN48C6/I5 --vopt family=GW1NZ-1 --vopt cst=tangnano1k.cst
gowin_pack -d GW1NZ-1 -o hex1000.fs pnr.json
openFPGALoader -b tangnano1k hex1000.fs
```

The commands become `xetal-fpga synth`, `pnr`, `pack` and `load`
(plan A9), and the constraints file (`.cst`: the clock pin, the LEDs,
the serial lines) is the board's schedule data (plan A8), one per
board.

The order, each one a check of the one before:

1. **Blink**: an X_eTaL counter on an LED. Proves the flow and the
   clock.
2. **UART echo**: the loopback through a USB-serial adapter. The Tang
   Nano 1K has no USB serial port of its own; the adapter must be
   **3.3 V TTL**, never 5 V and never an RS-232 port.
3. **HEX1000 self-test**: a built-in ROM program that runs the test
   arithmetic and shows pass or fail on the RGB LED, no host needed.
4. **The calculator**: the terminal over the adapter, driven from the
   host with the same key sequences as the tests, the display compared
   with the reference.

When it misbehaves, a small USB logic analyzer with sigrok/PulseView
shows the pins. Gowin's built-in analyzer (GAO) works only in the
Gowin IDE.

## Bricking: the risks and the recovery

The Tang boards are hard to brick.

- **Loading into SRAM** (`openFPGALoader -b BOARD FILE.fs`) is
  volatile: a power cycle brings back whatever is in flash. It is the
  default for everything here; the flow writes the flash only when
  asked (`-f`).
- **Writing the flash** survives power-off, but the JTAG port still
  configures the FPGA, so a bad design is replaced or erased the same
  way.
- **JTAG pins turned into ordinary I/O** (an option in the Gowin IDE)
  is the one way a design can lock the programmer out; recovery then
  takes extra steps. The open flow does not do this unless told to,
  and the board flow will refuse a constraints file that assigns a
  JTAG pin.
- **Electrical damage** is the real risk: 5 V on a 3.3 V pin, an
  output driven from both sides, a short. The constraints files are
  reviewed so that no output lands on a pin the board already drives
  (the clock, the buttons).
- **The programmer chip's own firmware** (the BL702 or BL616 on some
  boards): it is not updated unless a tool requires it; a failed update
  is usually recovered through its boot button, but it is a needless
  risk.

## The steps (docs/plan.md)

- nibble-machine, verilog-tools (this document; done): the tools in the gate,
  the duplicate-port fix, explicit casts, `just fit`.
- nibble-machine, machine-rtl: memory cells (block RAM for the RAM
  and the ROM), the bit-level UART testbench, Verilator for long runs,
  `just fit` against the 1K.
- fpga-board, flow: the wrappers, a constraints file per board, SRAM
  loading by default, the JTAG-pin guard, blink and echo.
- fpga-board, cell-sim: the cell-level simulator over Yosys's JSON.
- fpga-board, bitstream round trip: unpack and simulate the `.fs`.

## Nano 9K bring-up status (2026-10-14)

The user selected Nano 9K for initial bring-up; 1K sizing is deferred.
Its onboard USB debugger supplies programming and UART. The CLI now
has `synth`, `pnr`, `pack`, `load` and read-only `detect`; see
[board commands](../boards/tangnano9k/README.md). Commands validate
constraints against approved pin assignments and the synthesized
ports before place-and-route. The load default is SRAM; persistent
flash requires `--flash`.

A temporary Verilog counter passed the complete build-to-bitstream
flow on Linux. This is a toolchain check. Board-ready X_eTaL tops,
physical blink/echo and calculator deployment remain pending. Mac
owns the machine RTL; Linux owns board integration. No board has
been detected or loaded yet.


### First Nano 9K physical test (2026-10-14)

USB permissions, JTAG identification, SRAM programming and a direct
UART pin loopback are now verified on the attached 9K. Programming
reported a successful CRC; 542 host bytes returned exactly at 115200
baud. This fixture tests the board connection and tools independently
of HEX1000. It does not prove a clocked UART receiver/transmitter or
an X_eTaL program. The user confirmed LED blinking and the button pausing it while held.
Power-cycle recovery remains unchecked. Neither FPGA flash nor the inserted SD card
was used. Commands: `just board-prepare`, `just board-check`,
`just board-test`; [first connection](../boards/tangnano9k/README.md).


## X_eTaL arithmetic and UART on Nano 9K

The emitted counter/UART probe now passes on the actual board:
512 query results match the pinned evaluator, including two complete
8-bit wraps; all 255 non-query bytes echo exactly at 115200 baud.
The LED heartbeat instantiates the emitted Clocked counter. Its
programs, twins and reviewed baselines are in boards/examples.

The check compares full traces in the evaluator, XIR interpreter,
netlist simulator and emitted Verilog. The build additionally
simulates the board top at 27 MHz/115200 and Yosys's mapped Gowin
cells. Those deeper tests exposed brief extra UART frames when a
combinational output drove TX directly; registering TX at the pin
removes them. Keeping query/echo phases in one raw serial session
also avoids a failure introduced by reconfiguring the bridge between
phases. Both fixes are covered by simulation/serial tests.

The corrected image passed physical SRAM and explicit flash tests.
Power-cycle boot was already verified for the initial fixture; the
new X_eTaL image's power-cycle semantic test is separately pending.
Logs and per-stage reports are in work/board/probe. No host arithmetic
implements the FPGA counter; host calculations only check responses.

The complete HEX1000 adapter now passes timing and three physical
calculator sequences on Nano 9K SRAM. See
[hardware handoff](hex1000-hardware.md) for evidence, commands and limits.
