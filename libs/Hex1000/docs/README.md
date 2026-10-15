# Hex1000

HEX1000, a 4-bit machine in the spirit of the TI TMS1000, in X_eTaL:
its state, one instruction as a pure step, a run harness and an
assembler. The machine is defined in
[`docs/machine.md`](../../../docs/machine.md); this library is that
definition made executable, and the circuit (a later step) must
reproduce it.

```
"hx:" u_se< "Hex1000"
```

Put each library's `src/` directory on `XETAL_PATH` (`just path`
prints them; see the [README](../../../README.md)); `hx:` is the
recommended alias.

## The state

A tuple of twelve parts, `(A, Y, X, S, PC, PA, PB, SR, CL, O, R, RAM)`:
in the circuit, one register per part, each its own width
([`docs/tuple-plan.md`](../../../docs/tuple-plan.md)).

| Part | Bits | What |
| ---- | ---- | ---- |
| A | 4 | the accumulator |
| Y | 4 | the RAM word, and the R latch SETR and RSTR change |
| X | 2 | the RAM file |
| S | 1 | the status: set by compares and carries, 1 after every other instruction |
| PC | 6 | the address in the page, stepped by the shift-register rule (`hx:n_extPc`) |
| PA, PB | 4, 4 | the page and the page buffer (`LDP`; taken by `BR` and `CALL`) |
| SR, CL | 6, 1 | the return address and the call latch |
| O | 5 | the outputs: A, and S as bit 4 (`TDO`) |
| R | 16 x 1 | the output latches |
| RAM | 64 x 4 | word `16 X + Y` |

The ROM is an Int vector of 1024 bytes, instruction `64 PA + PC`.
`hx:r_egs` shows A to O as ten numbers, the view a trace prints.

## The instructions

The 40 kinds, their codes and what each does are in
[`docs/machine.md`](../../../docs/machine.md) (and `hx:Names`,
`hx:k_ind`, `hx:d_isasm`). The fixed instructions are coded 0 to 28 in
`hx:Names`'s order, so code 0 is `NOP` and an empty ROM runs NOPs;
29 to 31 are unused and act as `NOP`.

## The assembler

```
      rom := hx:a_ssemble "CLA\nTCY 5\nloop: IA\nBR loop"
```

A line is an optional `label:`, an instruction and its operand (a
decimal number, or a label: `BR` and `CALL` go to it, `LDP` takes its
page), and an optional `; comment`. `PAGE n` starts page `n`. A page's
instructions are placed in the program counter's visiting order
(`hx:Order`), so a program runs in the order it is written; a `BR` or
`CALL` with a number gives the PC value itself. A mistake (an unknown
instruction or label, an operand out of range, a full page) stops the
program with `error[panic]` and the line.

## Functions

| Function | Type | What |
| -------- | ---- | ---- |
| `hx:n_extPc pc` | `Int -> Int` | the address after `pc`: shifted left, the XNOR of the top two bits in, inverted when the low five bits are all 1 |
| `hx:Order` | `Int` | the 64 addresses in the order PC visits them from 0 |
| `hx:Names` | `Box Char` | the mnemonics, in the order of their kinds |
| `hx:k_ind op` | `Int -> Int` | the kind of each instruction code (its place in `Names` from 0) |
| `hx:d_isasm op` | `Int -> Char` | an instruction code as text |
| `hx:Reset` | a 12-part tuple | the state after reset: all 0 but S |
| `hx:r_egs st` | tuple `-> Int` | A Y X S PC PA PB SR CL O as numbers |
| `hx:r_am st` | tuple `-> Int` | the RAM as 4 files of 16 words |
| `hx:l_atches st` | tuple `-> Int` | the 16 R latches |
| `rom hx:s_tep st k` | `Int -> tuple -> Int -> tuple` | one instruction with `k` on the K inputs (bind the ROM: `u:s_tep := rom hx:s_tep`) |
| `rom hx:r_un (st, k, n)` | | the state after `n` instructions with `k` held |
| `rom hx:r_unKeys (st, ks)` | | the state after a run of presses, a row of `ks` each: K and how many instructions |
| `rom hx:t_race (st, k, n)` | | `hx:r_egs` after reset and after each of `n` instructions, a row each |
| `hx:a_ssemble text` | `Char -> Int` | the 1024-byte ROM of a program |
| `p hx:l_isting rom` | `Int -> Int -> Char` | page `p` in the order it runs, to its last instruction: PC, code, instruction |
| `names hx:t_able m` | `Char -> Int -> Char` | rows of numbers (or one vector) under column names, five characters a column |
| `hx:RegNames` | `Char` | `A Y X S PC PA PB SR CL O`, `hx:r_egs`'s columns |
| `hx:Calc` | `Char` | the calculator program's source, `rom/calc.asm` |
| `hx:CalcRom` | `Int` | the calculator program assembled |
| `hx:m_atrix (c, r)` | | the K inputs with key `c` held and latches `r`: column `c / 4` selected by R8 to R13, row `c mod 4` a K bit |
| `rom hx:t_ype (st, keys, n)` | | the machine typed at as a terminal would (each key held from R15 up to R15 down), the display latched from O on R0 to R7 rising: (state, digit latches, done, instructions) |
| `rom hx:t_ypeOn (st, digits, keys, n)` | | `t_ype` from a display already showing `digits` (a terminal keeps its screen: an ignored key changes nothing) |
| `rom hx:r_ecord (st, keys, n)` | | `t_ype`'s instructions at the ports: per instruction the K given, O and the R latches after: (ks, os, rs) |
| `hx:s_creen digits` | `Int -> Char` | latched digits as text, blank where O's bit 4 was clear |
| `hx:c_alc text` | `Char -> Char` | the screen after typing a text at the calculator from reset: the machine's `16 rx:c_alc text` |
| `hx:c_ost text` | `Char -> Int` | the instructions that took |
| `hx:Keymap` | `Int` | the key code of each ASCII character (0 to 127) as the board's glue decodes it: digits, `A` to `F` and `a` to `f`, `+ - * =`, `X` and `x`, Enter as `=`; 31 for a character that is not a key |
| `hx:GlueReset` | a 9-part tuple | the glue's state at reset: (queue of 16 key codes, write place, read place, count, held, key, digit latches, R0 to R7 as seen, frame) |
| `hx:g_lueK (g, r)` | | the K the glue gives the machine with latches `r`: the held key's row bit while its column is selected (combinational) |
| `hx:g_lue (g, c, o, r)` | | the glue's next state with character `c` arriving (0: none), O and the latches: a key queued (dropped when 16 wait), the next held from R15 up to R15 down, digit p latching O when R p rises, R0 rising completing a frame |
| `rom hx:b_oard (st, g, c)` | | one clock of the board: the machine with K from the glue, and the glue, both reading the ports as they were before the clock |
| `rom hx:s_erial (text, n)` | | the board typed at over its serial line, a character a clock: each completed frame as a row of eight digits, digit 7 first (`hx:s_creen` shows them) |
| `rom hx:w_ires (text, n)` | | `s_erial`'s run at the glue's ports: the characters, and after reset and each clock the frame flag and the digit latches |

`just types Hex1000` prints the full types; `tests/types.rgt` pins them. The run harness
loops with `p_ower` over a tuple state; nothing recurses.

## Examples

```
      "hx:" u_se< "Hex1000"
      12 t_ake hx:Order
0 1 3 7 15 31 63 62 61 59 55 47
      rom := hx:a_ssemble "TCY 3\nDYN\nBR 1"
      rom hx:t_race (hx:Reset, 0, 3)
0 0 0 1 0 0 0 0 0 0
0 3 0 1 1 0 0 0 0 0
0 2 0 1 3 0 0 0 0 0
0 2 0 1 1 0 0 0 0 0
      0 hx:l_isting hx:a_ssemble "loop: TDO\nIA\nBR loop"
  PC code  instruction
   0   25  TDO
   1   14  IA
   3  128  BR 0
```

## The calculator

The ROM program [`rom/calc.asm`](../rom/calc.asm) makes HEX1000 an
eight-digit hexadecimal calculator that behaves as the Radix library,
key for key; how it works, its RAM, its pages and its cost are in
[`calc.md`](calc.md).

```
      hx:c_alc "C0DE+BEEF="
   17FCD
```

## Tests and demos

- [`tests/instructions.xtl`](../tests/instructions.xtl): every
  instruction against its definition (58 checks).
- [`tests/programs.xtl`](../tests/programs.xtl): a counter, two digits
  added with the carry, keys scanned.
- [`tests/calc.xtl`](../tests/calc.xtl): the calculator against
  Radix on 33 key sequences (every hexadecimal one of Radix's tests and
  more: borrows through all eight digits, FFFFFFFF times FFFFFFFF,
  operators in a row, ignored keys).
- [`tests/errors.xtl`](../tests/errors.xtl): the assembler stopping at
  a mistake.
- [`tests/board.xtl`](../tests/board.xtl): the machine with the
  board's glue, typed at over the serial line: the last frame equals
  `hx:c_alc`'s screen (lowercase, `x` and Enter as keys; a text of 19
  keys sent at once keeps its first 16).
- [`demos/serial.xtl`](../demos/serial.xtl): the frames the board
  sends, as a terminal attached to it shows them.
- [`demos/board.xtl`](../demos/board.xtl): the machine and glue at the
  glue's ports, clock for clock: what `rtl/hex1000-board.v` must
  reproduce.
- [`demos/counter.xtl`](../demos/counter.xtl): the machine counting on
  its outputs: its listing, then every register under a header,
  after reset and after each instruction.
- [`demos/terminal.xtl`](../demos/terminal.xtl): the calculator at a
  terminal (`just calc`), a key at a time; its scripted session is
  `terminal.keys`.
- [`demos/calc.xtl`](../demos/calc.xtl): calculations typed at the
  machine, their screens and costs.
- [`demos/listing.xtl`](../demos/listing.xtl): the calculator's ROM,
  page by page.
- [`demos/keys.xtl`](../demos/keys.xtl): keys read from K and stored:
  the listing, the presses, the registers and RAM at the end.

## The circuit

`demos/hex1000.xir` is HEX1000 as a circuit, written by
`scripts/gen-hex1000-xir.py` (one rule per instruction), and
`rtl/hex1000.v` its Verilog: one instruction per clock, ports `in_k`,
`out_o`, `out_r`. `demos/circuit.xtl` (`hx:r_ecord`) is what it must
reproduce at its ports, clock for clock, and does: in the IR
interpreter, the netlist simulator and iverilog (`just equiv`,
`just rtl`). `just fit hex1000`: about 2220 LUTs and 379 flip-flops.

`demos/hex1000-board.xir` is the machine with the board's glue
(`hx:b_oard`), from the same script, and `rtl/hex1000-board.v` its
Verilog, module `hex1000_board`: `in_c[7:0]` a character for one
clock (0: none), `out_f` 1 for the clock after a frame is complete,
`out_d[47:0]` the eight digit latches, digit p in bits `6p+5` to `6p`
(the digit in the low four bits, bit 4 set to show it). A board top
adds the UART, hands each received byte to `in_c` in the machine's
clock domain, and sends the digits as text when `out_f` is 1.
`demos/board.xtl` (`hx:w_ires`) is what it must reproduce, and does,
clock for clock, in the IR interpreter, the netlist simulator and
iverilog. `just fit hex1000-board`: about 2340 LUTs and 540
flip-flops, the glue about 120 LUTs and 160 flip-flops.

## Not yet

Octal and decimal (a radix mode in the program); the ROM and RAM as
block and LUT RAM, to fit the Tang Nano 1K.

## Provenance

Written for X_eTaL-fpga. The machine is our own design in the spirit
of the TMS1000, not a replica.
