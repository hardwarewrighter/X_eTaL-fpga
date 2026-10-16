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

- 2026-10-14, for the board top and glue (the core lands next, from
  `libs/Hex1000/demos/hex1000.xir` as `libs/Hex1000/rtl/hex1000.v`):
  - The ROM image: `libs/Hex1000/rom/calc.hex`, 1024 lines of one
    hexadecimal byte, address 64 PA + PC, for `$readmemh`
    (`scripts/rom-hex.sh`; the gate checks it matches `hx:CalcRom`).
    The first core has the ROM inside it as a constant table.
  - The core, module `hex1000`: `input clk`, `input rst` (synchronous,
    high: the reset state), `input [4:0] in_k` (K in bits 3 to 0, bit 4
    tied 0), `output [5:0] out_o` (O: the digit in bits 3 to 0, bit 4
    set to show it, clear to blank; bit 5 always 0), `output [15:0]
    out_r` (the R latches, R0 in bit 0). One instruction per clock;
    no enable yet, so drive `clk` from a divided clock if timing at
    27 MHz fails (the calculator needs only thousands of instructions
    a second). The ports are a bit wider than the values because the
    provisional IR has only signed integers (unsigned types come with
    the 1K sizing).
  - The glue around it (board data, not the core), as the X_eTaL harness
    `hx:t_ype` models it: the key with code c (0 to 9, A to F as 10 to
    15, + 16, - 17, * 18, = 19, X 20; Enter as =) is held as K bit
    `c mod 4` while R(8 + c / 4) is 1; present the next key only
    while R15 is 1 and release it when R15 drops. Digit p (R p, 0 the
    rightmost) takes `out_o` when R p rises; after R0 rises the
    display is complete: send the 8 characters (digit or space) and a
    carriage return at 115200 8N1.

- 2026-10-14 12:11: the core has landed on `main`: `libs/Hex1000/rtl/hex1000.v`,
  module `hex1000`, the ports as above (`clk`, `rst`, `in_k[4:0]`,
  `out_o[5:0]`, `out_r[15:0]`), the ROM inside it. It reproduces the
  X_eTaL machine's ports clock for clock over the 1752 clocks of
  12+34= under iverilog (`just rtl`); the testbench is
  `xetal-fpga testbench libs/Hex1000/demos/hex1000.xir`. Yosys
  (`just fit hex1000`): 2220 LUTs, 379 flip-flops, a quarter of the
  9K. Timing at 27 MHz is not yet known: run it on a divided clock
  first. The board top and the UART glue are the board lane's, as in
  the contract above.


Linux board handoff: the emitted Clocked/Uart counter probe now
passes 512 physical queries (two 8-bit wraps) and 255 echoes on 9K.
Register the UART TX pin: the mapped-cell simulation found glitches
on the combinational line. Keep serial query/echo phases in one raw
connection. The landed HEX1000 core synthesizes here at about 2256
LUTs and 379 flip-flops; its board adapter is the next Linux step.


## Linux calculator handoff (2026-10-14)

HEX1000 is verified on actual Nano 9K SRAM over USB UART: 12+34=
gives 46, A*B= gives 6E, C0DE+BEEF= gives 17FCD. The user copied
the 8.12-second MP4 to the blog agent. Mac can publish these results
now. [Hardware handoff](hex1000-hardware.md) lists commands, evidence
and limitations. Linux retains reconnect, key-queue flow control,
interactive terminal and calculator flash/power-cycle work; Mac
retains library/core, browser/site and blog work.

The user now requests both agents work on main. Linux board work is
pushed directly to main; fetch/pull main for this handoff. Continue
the file ownership above to avoid conflicts, and preserve both saga
histories when reconciling metadata.


## Mac handoff: the glue, checked (2026-10-15)

The board's glue is now modeled in X_eTaL and emitted with the
machine as one checked module, `libs/Hex1000/rtl/hex1000-board.v`,
module `hex1000_board`:

- `input clk`, `input rst` (synchronous, high), `input [7:0] in_c`: a
  received character for one machine clock, 0 for none (ASCII; the
  module decodes keys itself: digits, A to F and a to f, + - * =, X
  and x, CR and LF as =).
- `output [0:0] out_f`: 1 for the one clock after a frame completes.
- `output [47:0] out_d`: the eight digit latches, digit p (0 the
  rightmost) in bits 6p+5 to 6p; bit 4 of a digit set means shown,
  clear means blank.
- Inside: a 16-key queue (a key arriving when 16 wait is dropped), the
  key matrix and R15 handshake, the digit latches on R0 to R7 rising.

It reproduces `hx:w_ires` (demo `libs/Hex1000/demos/board.xtl`) clock
for clock under the IR interpreter, the netlist simulator and iverilog,
and `libs/Hex1000/tests/board.xtl` checks the X_eTaL model's frames
against `hx:c_alc`. Size: 2338 LUTs, 540 flip-flops (Yosys).

What stays in the board top: the UART, handing each received byte to
`in_c` for one clock of the machine clock (a pulse synchronized across
the divider), and sending `out_d` as text when `out_f` is 1. Replacing
the hand-written key queue, matrix, handshake and digit capture in
`boards/tangnano9k/hex1000-top.v` with this module is the board lane's
call; nothing of the board lane's was changed.

The Mac lane's saga moved to `lanes/sw/.agentrail` (run agentrail
with `--saga lanes/sw`): both lanes now share `main`, and `.agentrail/`
is the board lane's. The archived copy in `.agentrail-archive/` is
left as it was.


## Mac handoff: HEX1000 fits the Tang Nano 1K (2026-10-15)

`libs/Hex1000/rtl/hex1000.v` (and `hex1000-board.v`) now read the ROM
and the RAM through registers, which Yosys maps to block RAM:
`just fit hex1000` gives 438 LUTs, 81 flip-flops and 2 block RAMs (the
1K has 1152, 864 and 4; it was 2220 LUTs and 379 flip-flops). The
ports are unchanged (`clk`, `rst`, `in_k`, `out_o`, `out_r`) and the
module still reproduces the X_eTaL trace clock for clock under
iverilog.

For the board:

- Hold `rst` high for at least one clock (the startup reset does): at
  reset the ROM's read is of address 0, which loads the first
  instruction.
- The RAM is a block RAM: loaded with zeros at configuration, **not
  cleared by `rst`**. The calculator expects a cleared RAM, so after a
  later reset (a button) it can show stale digits until the program
  clears its own RAM (Mac step calc-rules). Reconfigure (or power
  cycle) to start clean meanwhile.
- The 1K has no USB serial port and no LUT RAM; the design needs
  neither (block RAM only).
- `hex1000-board.v` (with the UART glue) is 1138 LUTs: it also fits,
  barely; the physical calculator's panel (Mac step panel) replaces the
  glue's key queue with the keypad wired to the machine's key scan.

The full procedure for the 1K, what the first test must show and what
comes next: [docs/handoff-1k.md](handoff-1k.md).
Interactive hardware handoff: `just board-calc` and
`just board-calc-query 'X123*4='` use the current board image without
loading it and expose actual UART display data. Raw evidence is
committed under boards/tangnano9k/evidence. The additional live MP4
is work/tapes/hex1000-9k-live.mp4 (7.68 seconds); Mac needs to copy
that generated artifact separately. The earlier MP4 remains valid.
LED blink/reset is intentionally retained in the calculator top.
