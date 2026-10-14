# Mac and Linux development

Decided with the user on 2026-10-14: develop the software machine
and the board flow in parallel. The first hardware target is the
Tang Nano 9K, with onboard USB-JTAG and USB-UART. The Nano 20K is
an alternative; sizing for the Nano 1K comes later.

## Mac handoff

Pull main; the Mac software lane continues on main. The inherited saga
is already named `sw-nibble-machine`, with prefixed step
directories. The machine and hexadecimal ROM have landed through 6efc13e.
Resume live-demo with `agentrail next`, then implement radix modes.

Own `libs/Hex1000/` and any calculator library directories, including
their X_eTaL source, assembler, demos, baselines and documentation.
Maintain the pure machine and hexadecimal ROM tests; finish the
interactive terminal/browser demo and octal and decimal modes
against Radix's reference. Own
`docs/machine.md` and `docs/tuple-plan.md`. Keep the machine's ISA
and state contract there; notify Linux through committed handoff
notes when the state layout or ROM interface is ready.

Mac also owns components/xir, components/rtl and machine-rtl, as
recorded by the Mac in c6d0a26. Implement computed-address RAM/ROM
cells and the machine twin there. Linux consumes the emitted circuit.

## Linux responsibilities

Create `board/flow` from this shared main commit. Archive the
inherited saga on this branch with agentrail, identifying it as an
incomplete copy retained on the Mac branch, then initialize a distinct
`board-fpga-board` saga. This does not complete or discard Mac's work.

Own `boards/`, new board-flow scripts and tests, components/cli
board command integration and hardware tests. Coordinate additions
to shared integration files: `justfile`, Cargo manifests, gate
scripts and root README. Own
`docs/linux-nvidia.md`, `docs/hardware.md`, and `docs/boards.md`.
Mac supplies requested shared-file additions in its handoff.

Order: verify the OSS CAD Suite, establish USB permissions and
read-only board detection, implement board configuration and build
wrappers, prove blink and UART echo, then test the Mac-emitted machine and
calculator circuit on 9K. Mac owns memory cells and machine lowering.
SRAM loading is the default; persistent flash is an explicit action.
No connected board means hardware validation remains pending.

## Shared contracts and integration

`docs/machine.md` defines the ISA, state, reset, K/R/O ports and
two-clock instruction timing. Mac must document assembler opcode
values and ROM image export for RTL and board integration.
Agree on UART key encoding, display framing and baud rate before
calculator integration. Board clock and pins belong to board data,
never the X_eTaL program or IR. Linux can start with existing
Clocked and Uart examples while Mac develops the machine.

Both lanes update CHANGES and `docs/plan.md`; resolve overlapping
entries by keeping both. Coordinate asks in `docs/xetal-asks.md`.
Neither lane moves dependency pins without an agreed integration
point. Neither modifies sibling repositories or vendored checkouts.

Push feature branches regularly. Linux is the integration owner:
merge one completed lane at a time, rerun the gate, then coordinate
site publication with Mac. Mac owns the site and publishes it;
Linux supplies board documentation additions.
Before merging a lane, complete its actual work, archive its saga
using agentrail, and commit the metadata. Prefixed step directories
alone do not solve conflicts in a live `saga.toml`. Preserve both
archives and the common inherited history when resolving metadata.
Rename a finished feature branch to `pr/<slug>` for handoff; do not
continue developing on it. Start later work from updated main.

The first milestone is a tested software machine plus a 9K blink
and UART echo. The second is calculator traces matching across the
evaluator, XIR, netlist and Verilog. The third is calculator UART
results matching on the board. Only then optimize for the 1K.

## Mac notes for Linux

- 2026-10-14: `main` has the software HEX1000 and its calculator
  program (`libs/Hex1000`, `rom/calc.asm`). The I/O contract is in
  `libs/Hex1000/docs/calc.md` (R0 to R7 display strobes with O's bit
  4 as show, R8 to R13 key columns with K the row, R15 a ready
  handshake); the codes are `hx:Names` in order (`hx:k_ind`,
  `docs/machine.md`). The board lane needs nothing from the Mac for
  blink and the UART echo: `libs/Uart/rtl/loopback.v` and the other
  pinned circuits are on `main`.
- Shared-file additions from the Mac in this commit: `justfile`
  recipes `calc`, `web`, `tape`; `scripts/test-libs.sh` reads an
  optional `NAME.keys` as a program's typed input; `scripts/build-web.sh`.
- Next on the Mac: machine-rtl (the machine as a circuit, memory cells,
  the ROM image as a `$readmemh` file) ahead of the octal and decimal
  modes, so the board lane gets the circuit sooner. Proposed for the
  board's terminal glue, to agree before integration: 115200 baud, 8N1;
  a received character is one key (Radix's characters, Enter as =);
  the display sent as 8 characters and a carriage return whenever R7
  to R0 have all been strobed.
