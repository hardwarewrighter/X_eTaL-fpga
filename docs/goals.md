# Goals

The project's goals, short, medium and long term, as the user set
them on 2026-10-14. The plan (`docs/plan.md`) turns them into sagas
and steps; this page says what they are for and in what order.

## Short term: X_eTaL emulates a design that runs on a board

Show that X_eTaL can emulate, exactly, a design that then runs on a
Tang Nano: the program in X_eTaL is the executable specification, the
same design as a circuit runs on the board, and the two agree.

- The demo is HEX1000 (`docs/machine.md`), the 4-bit calculator
  machine: in X_eTaL (`libs/Hex1000`), as Verilog
  (`libs/Hex1000/rtl/hex1000.v`), and on a board, typed at over a
  serial terminal, every screen the same as the X_eTaL machine's.
- The board is a **Tang Nano 4K or 9K** (done on the 9K). The circuit
  as first lowered took about 2220 LUTs and 379 flip-flops; with its
  memories in block RAM it takes 438 LUTs, 81 flip-flops and 2 block
  RAMs, so it fits the 1K too.

Done when: the calculator, typed at over the UART on a 4K or 9K,
shows the screens the X_eTaL machine shows for the same keys (the
board lane's machine-board step, with the Mac's circuit).

## Now: a physical HEX1000 calculator on the Tang Nano 1K

Decided with the user, 2026-10-15: HEX1000 as a battery-powered hex
calculator in a case, the **Tang Nano 1K the goal, the 4K the
fallback**. This makes the 1K the target again (it had been long
term).

- Keys: a 4x4 matrix of the 16 hexadecimal keys, a 4-button board for
  `+ - * =` (common ground, a wire a button) and a Clear key. They wire
  straight to the machine's own key scan (R8 to R13 select a column,
  K reads its row), debounced in the fabric; an I2C GPIO expander is
  the option if pins run short.
- Display: an 8-digit 7-segment LED module driven by a MAX7219
  (SPI-style, write-only: data, clock, load), written in no-decode
  mode with our own hex font each time the program completes a frame;
  5 V, with a 74AHCT125 shifting the 3.3 V lines up. Raw multiplexed
  digits (16 pins, a transistor per digit) are the alternative.
- Rules (decided with the user): negatives stay two's complement
  (`5-7=` shows `FFFFFFFE`); `=` pressed twice clears to 0, so no Clear
  key and no reset button; overflow (a carry out of digit 7 on `+` or
  `*`) shows `OuErFLO` until the next key.
- LEDs for power and overflow (R14); a power switch.
- Power: 4 AA cells through a 5 V buck-boost module, for the board and
  the MAX7219 (the cells alone sag below the MAX7219's 4.0 V).
- Efficiency: the circuit as first lowered (about 2220 LUTs) had to
  fit 1152. The ROM and the RAM in block RAM, read through registers
  (step rtl-memories, 2026-10-15), took it to 438 LUTs, 81 flip-flops
  and 2 of the 4 block RAMs: it fits, with room for the keypad and
  display logic. Unsigned 4-bit types, dead and duplicate logic removed
  and a shared adder (rtl-narrow) are left for more room. The X_eTaL
  machine stays the specification.

Done when: the calculator runs from batteries on a 1K (or a 4K),
keys in, digits on the display, every screen the X_eTaL machine's.

## Later: HEX1000 on smaller boards still

Superseded by the section above: the 1K is the target now.

## Next: mini GPU/NPU pipelines for serial data

Before any new CPU: small matrix-multiply pipelines for data that
arrives serially (a UART stream, samples), in the style of a tiny GPU
or NPU: multiply-accumulate lanes, weights in a ROM, a schedule that
trades lanes for clocks, results streamed back out. Written in
X_eTaL first, lowered and checked as everything here is, then run on
a 9K, 20K or 25K. This builds on the Datapath library (vector add,
saxpy, the reduction tree, the pipeline) and the neural layer of the
plan's fpga-nn saga, and shares the matmul with X_eTaL-gpu, which
schedules the same layer as GPU kernels.

## Medium term: our own soft CPU, prototyped in X_eTaL

Use X_eTaL to prototype new soft CPUs on the larger boards (Tang Nano
9K and 20K, Tang Primer 25K), the way HEX1000 was built: the machine as
a pure step function in X_eTaL, its programs assembled and run there,
then lowered to a circuit that must agree with it.

- Inspirations: the RCA CDP1802 (COSMAC) and the IBM 1130, for their
  instruction set architectures. They may be implemented one day as
  themselves.
- The medium-term target is **our own ISA** for a soft CPU, informed
  by them, not a replica of either.

## Order

1. HEX1000 on a 4K or 9K board, agreeing with X_eTaL (done on the
   9K, 2026-10-14).
2. The physical calculator on the Tang Nano 1K (fallback 4K): keypad,
   I2C display, batteries; the circuit made efficient enough to fit.
3. Serial-data matmul pipelines, a mini GPU or NPU (next).
4. Our own soft CPU, prototyped in X_eTaL, on the 9K, 20K or 25K
   (medium term), with the CDP1802 and the IBM 1130 as inspirations.
