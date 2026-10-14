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
- The board is a **Tang Nano 4K or 9K**. The circuit as first lowered
  takes about 2220 LUTs and 379 flip-flops (Yosys, before placement):
  a quarter of the 9K, about half of the 4K. The 9K is where the board
  lane brings up the flow first (`docs/parallel-work.md`).
- The Tang Nano 1K was an early target, not a requirement now.

Done when: the calculator, typed at over the UART on a 4K or 9K,
shows the screens the X_eTaL machine shows for the same keys (the
board lane's machine-board step, with the Mac's circuit).

## Long term (and alongside): HEX1000 down to the Tang Nano 1K

Future work, not on the critical path: optimize the circuit that
fits the 4K and 9K until it fits the 1K (1152 LUTs, 864 flip-flops,
4 block RAMs). The known levers: the ROM in block RAM with a
registered read (the two-clock fetch and execute of `docs/machine.md`),
the RAM in LUT RAM, unsigned types in the IR so 4-bit values are 4
bits, dead cells removed, the decoder shared. Each is a schedule or
emitter change; the X_eTaL machine stays the specification.

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

1. HEX1000 on a 4K or 9K board, agreeing with X_eTaL (short term).
2. Serial-data matmul pipelines, a mini GPU or NPU (next).
3. Our own soft CPU, prototyped in X_eTaL, on the 9K, 20K or 25K
   (medium term), with the CDP1802 and the IBM 1130 as inspirations.
4. HEX1000 on the 1K (long term, whenever it is worth it).
