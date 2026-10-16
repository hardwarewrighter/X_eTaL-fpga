# Handoff: HEX1000 on the Tang Nano 1K

From the Mac lane to the Linux (board) lane, 2026-10-16. What is
ready on `main`, what to build first on the 1K, what it must show, and
what comes next. Board files (`boards/`, `scripts/board-*.py`, their
recipes) stay the Linux lane's; the Mac lane does not edit them.

## What is ready

| What | Where | State |
| ---- | ----- | ----- |
| The machine as a circuit | `libs/Hex1000/rtl/hex1000.v`, module `hex1000` | checked clock for clock against X_eTaL under iverilog (1752 clocks of 12+34=); Verilator and Yosys clean |
| Its size | `just fit hex1000` | 438 LUTs, 81 flip-flops, 2 block RAMs (the 1K: 1152, 864, 4) |
| Its ports | unchanged since the 9K | `clk`, `rst` (synchronous, high), `in_k[4:0]` (K in bits 3 to 0), `out_o[5:0]` (the digit in bits 3 to 0, bit 4 set to show it), `out_r[15:0]` (R0 in bit 0) |
| The UART variant | `libs/Hex1000/rtl/hex1000-board.v`, module `hex1000_board` | 1138 LUTs before a UART: too big for the 1K with one; use it on the 9K only |

Pull `main`, run `just gate` (with the OSS CAD Suite on `PATH` it also
runs iverilog, Verilator and Yosys on every circuit), then
`just fit hex1000`.

## Two things to know before the board

1. **The RAM is block RAM and `rst` does not clear it.** It is loaded
   with zeros at configuration. The calculator expects a cleared RAM,
   so after a reset other than configuration (a button) it can show
   stale digits until the program clears its own RAM (Mac step
   calc-rules, next). Reload the bitstream or power cycle to start
   clean meanwhile.
2. **Hold `rst` high for at least one clock of the machine's clock**
   at startup (the 9K top's startup counter does). At reset the ROM
   read is of address 0, which loads the first instruction into the
   instruction register; without that clock the first instruction is
   garbage.

## The 1K in the board flow

The flow has no 1K entry yet: `boards/tangnano1k/board.json` and its
`.cst` files are needed, in the 9K's format. From `docs/boards.md`:
the FPGA is the **GW1NZ-LV1QN48C6/I5**, a 27 MHz clock, USB-JTAG, **no
USB serial port** (a 3.3 V USB-serial adapter on two pins when one is
wanted), no LUT RAM (the design uses block RAM only). To confirm
against the tools before relying on them, not guessed here:

- the nextpnr-himbaechel device string and the `family` for
  `--vopt family=` and `gowin_pack -d` (Apicula names the chip
  GW1NZ-1);
- the openFPGALoader board name (`openFPGALoader --list-boards`);
- every pin: the clock, the user button(s), the RGB LED's three pins
  and whether they are active low, from Sipeed's Tang Nano 1K
  schematic or Apicula's 1K example constraints. Keep the board flow's
  guard: no JTAG pin, nothing the board itself drives.

## First test: the machine runs (no keys, no display)

A smoke-test top, `boards/tangnano1k/hex1000-smoke.v` (the Linux
lane's), around the unchanged core:

- the machine's clock divided from 27 MHz as on the 9K (by 28: about
  a million instructions a second);
- `rst` from a startup counter (and a button if the board has one),
  synchronized;
- `in_k` tied to 0 (no key pressed);
- three LEDs (or the RGB LED's three colors): `out_r[15]` (ready),
  `out_o[4]` (the digit is shown), and a sticky latch set when
  `out_r[0]` rises (digit 0's strobe is a single instruction, far too
  short to see; the latch shows it happened). `out_r[8]` toggles as the
  program scans the first key column, a dim glow on an LED if one is
  spare.

What it must show, from the X_eTaL trace (`libs/Hex1000/tests/demo-circuit.out`,
the circuit's specification):

| After instruction | What happens | The LEDs |
| ----------------- | ------------ | -------- |
| 88 | the program strobes digit 0 (R0 high for one instruction) with O = 16: a shown 0, the reset screen | "shown" on; the R0 latch on |
| 135 | it is ready for a key (R15 rises; O stays 16, R0 to R7 stay 0) and scans the key columns while it waits | ready on, steady |

At a million instructions a second that is within 0.2 ms of reset, so
all three come on at once and stay on. A dark ready LED means the ROM
did not load (the reset clock) or the clock is not running; the R0
latch dark with ready on would mean the display strobe went wrong. This proves the ROM and RAM blocks, the
subroutine call (the program calls `show` to strobe the digits) and
the reset path on the 1K. Record the resource and timing results as
on the 9K.

## Next: the physical calculator

The Mac lane's next steps:

1. **calc-rules:** `==` clears to 0, overflow shows `OuErFLO`,
   negatives stay two's complement, and the program clears its own RAM
   at reset (which removes caveat 1 above). Changes `hex1000.v`'s ROM
   only; the ports stay.
2. **mockup-rules:** the 3D mockup updated for those rules.
3. **panel:** the panel modeled in X_eTaL and emitted as a checked
   module around the machine: the 4x4 hexadecimal pad and the four
   operator buttons on the machine's own key scan (debounced), and the
   MAX7219 driver (SPI-style: data, clock, load; no-decode mode, the
   hex font and `OuErFLO`), with `docs/calculator-build.md` (the
   wiring, the parts, the power). Its ports are fixed then; expect
   about: the pad's 4 column outputs and 4 row inputs, the 4 operator
   inputs (active low, to ground), the MAX7219's 3 outputs, the ready
   and overflow LEDs.

The board lane then: the 1K top around that module (the clock
divider, the reset, the pins and pull-ups in the `.cst`:
`PULL_MODE=UP` on the row and operator inputs), the build, flashing
the calculator so it boots on power-up (the 9K's calculator flash boot
is still open too), and the wiring check with the parts.

Electrical notes, to confirm in `docs/calculator-build.md`: the 1K's
pins are 3.3 V and not 5 V tolerant; about 1 kOhm in series on the
pad's column lines; the MAX7219 runs at 5 V and wants a 3.5 V high, so
its three inputs go through a 74AHCT125 powered at 5 V; power is 4 AA
cells through a 5 V buck-boost module into the board's 5 V pin (not at
the same time as USB without a diode).
