# HEX1000: the 4-bit machine

**Status: decided with the user, 2026-10-07** (the decisions are at
the end). This page is the machine's definition; saga 3's later steps
build to it, and a change to it is a decision of its own.

The machine is the first demo of saga 3 (`docs/plan.md`): a 4-bit
machine in the spirit of the TI TMS1000, our own design of equivalent
size and complexity, not a replica, whose ROM program makes it a
calculator (hexadecimal first, then octal and decimal), sized for a
Tang Nano 1K (`docs/calc-approaches.md`, "Decided"). Facts about the
TMS1000 below are from memory and are to be checked against TI's
data manual before anything relies on them; they are here only to
show what "in the spirit of" borrows.

Its name is **HEX1000** (decision 1): a hexadecimal calculator chip
of the TMS1000's size. The X_eTaL library will be `Hex1000` and the
Verilog module `hex1000`.

## What it borrows from the TMS1000, and what it does not

| TMS1000 (from memory) | Here |
| --------------------- | ---- |
| 4-bit accumulator A, 4-bit Y, 2-bit X | the same |
| 64 by 4-bit RAM, a word chosen by X (file) and Y (word) | the same |
| 1024 by 8-bit ROM: 16 pages of 64 instructions | the same size |
| a 6-bit program counter that steps as a feedback shift register, not a binary counter | a feedback shift register too (decision 2), our own rule, below |
| a page address and a page buffer; a branch or call goes to the buffered page | the same |
| one subroutine level: a return address register and a call latch | the same |
| a status flag: set by compares and carries, and back to 1 after every other instruction; a branch is taken when it is 1 | the same (decision 6) |
| K inputs (4 lines), R output latches (one per Y value), O outputs through a programmable segment table | K, R and O kept as ports; how a board drives them is decision 4 |
| an instruction decoder programmable per customer, about 43 instructions | a fixed decoder, about 40 instructions, our own encoding |
| six oscillator phases per instruction | one instruction per two clocks (decision 3) |

## Registers and memories

| Name | Bits | What |
| ---- | ---- | ---- |
| `A` | 4 | the accumulator |
| `Y` | 4 | the RAM word index, also the R latch index |
| `X` | 2 | the RAM file |
| `S` | 1 | the status flag |
| `PC` | 6 | the address within the page, stepped by the shift-register rule below |
| `PA` | 4 | the page |
| `PB` | 4 | the page buffer (set by `LDP`, taken by `BR` and `CALL`) |
| `SR` | 6 | the return address |
| `CL` | 1 | the call latch (in a subroutine) |
| `O` | 5 | the output register (`TDO`: A and S) |
| `R` | 11 or 16 | the output latches, one per Y value |
| RAM | 64 x 4 | word `16 X + Y` |
| ROM | 1024 x 8 | instruction `64 PA + PC` |

About 50 flip-flops of registers, plus the RAM's 256 bits.

## The program counter

PC does not count; it shifts. After each instruction that does not
branch, with `b5` and `b4` its top two bits:

```
next PC = (2 PC mod 64) + f
f       = 1 - (b5 xor b4), inverted when PC mod 32 = 31
```

From 0 this visits all 64 addresses and comes back to 0: 0, 1, 3, 7,
15, 31, 63, 62, 61, 59, 55, 47, ... (checked when the rule was
chosen; the machine library's tests check it again). The XNOR
feedback alone would visit 63 and stick at 63; the inversion when the
low five bits are all ones makes it visit all 64, so a page holds 64
instructions as on the TMS1000. The hardware is a shift, an XNOR and
a five-input AND, no adder; the assembler places a page's
instructions in this order, so a program is written in reading order
and never sees it. A branch or a call loads PC (and the page) from the
instruction.

## The instruction set

As proposed and kept (decision 7). One byte per instruction. The top
bits choose the group:

| Pattern | Instructions |
| ------- | ------------ |
| `10aa aaaa` | `BR a`: when S is 1, go to `a` in page PB (else S becomes 1 and the program goes on) |
| `11aa aaaa` | `CALL a`: when S is 1, call `a` in page PB (one level) |
| `0111 cccc` | `ACC c`: A + c into A, S the carry (the digit adjust for decimal and octal, decision 7) |
| `0110 cccc` | `YNEC c`: S is Y not equal to c |
| `0101 cccc` | `TCMIY c`: c into RAM, then Y + 1 |
| `0100 cccc` | `TCY c`: c into Y |
| `0011 pppp` | `LDP p`: p into PB |
| `0010 ffbb` | the bit group: `SBIT b`, `RBIT b`, `TBIT b` (S is bit b of the RAM word) and `LDX x` (x into X) |
| `000f ffff` | 32 fixed instructions, below |

The fixed instructions (32 places, 30 used):

| Instruction | What | S |
| ----------- | ---- | - |
| `TAY`, `TYA` | A into Y, Y into A | 1 |
| `CLA` | 0 into A | 1 |
| `TAM`, `TMA`, `TMY` | A into RAM, RAM into A, RAM into Y | 1 |
| `TAMIY` | A into RAM, then Y + 1 | 1 |
| `TAMZA` | A into RAM, then 0 into A | 1 |
| `XMA` | exchange A and the RAM word | 1 |
| `AMAAC` | A + RAM into A | the carry |
| `SAMAN` | RAM - A into A | no borrow |
| `IMAC`, `DMAN` | RAM + 1 into A, RAM - 1 into A | the carry, no borrow |
| `IA` | A + 1 into A | 1 |
| `IYC`, `DYN` | Y + 1, Y - 1 | the carry, no borrow |
| `CPAIZ` | 0 - A into A (two's complement) | A was 0 |
| `ALEM` | compare | A at most RAM |
| `MNEZ`, `YNEA` | compare | RAM not 0, Y not A |
| `KNEZ`, `TKA` | a key present; K into A | K not 0; 1 |
| `SETR`, `RSTR` | R latch Y to 1, to 0 | 1 |
| `TDO`, `CLO` | A and S into O; 0 into O | 1 |
| `COMX` | X inverted | 1 |
| `RETN` | return from a call | 1 |
| `NOP` | nothing | 1 |

Digit arithmetic in RAM (a number is a row of 4-bit words in one
file) is `AMAAC` with the carry carried through S and `ACC 1`, as the
TMS1000's calculators did it; hexadecimal needs no adjustment because
a 4-bit carry is the digit carry.

## Timing

One instruction per **two clocks**, fetch then execute (decision 3).
The ROM is 8 Kbit, which fits one block RAM of the GW1NZ-1, but a
block RAM reads on a clock edge: the instruction is there one clock
after its address. Fetching on one clock and executing on the next
keeps the logic simple; a one-clock design would need the next
address one clock early (or the ROM in LUTs, which the 1K cannot
spare: 1024 x 8 bits is several hundred LUTs). At 27 MHz two clocks
per instruction is 13.5 million instructions a second, far more than
a calculator needs.

## I/O on a board

The machine's ports are K (4 bits in), R (latches out) and O (5 bits
out); how a board presents them is decision 4: first **a terminal over the
UART of saga 2**. A key typed on the host becomes
the K value while it is held (a received byte held for some
milliseconds, translated by a small table from characters to key
codes); the display is sent to the host as text whenever the program
writes it (`TDO` per digit, with R marking which digit). Physical
keys and seven-segment digits (a Pmod or a breadboard) can come
later on the same ports. If the Tang Nano 1K's USB bridge offers no serial port, a USB-serial
adapter on two pins is used (the user has them).

## Size against the Tang Nano 1K (an estimate)

From memory of Gowin's figures for the GW1NZ-LV1, to be checked:
about 1152 4-input LUTs, 864 flip-flops, 72 Kbit of block RAM.

| Part | Estimate | From |
| ---- | -------- | ---- |
| registers | about 50 flip-flops | the table above |
| RAM 64 x 4 | 256 flip-flops, or a few LUTs as distributed RAM, or part of a block RAM | the Clocked library's RAM is a register and a mux per word |
| ROM 1024 x 8 | one block RAM | the program |
| ALU, compares, decoder, next PC | a few hundred LUTs | 4-bit units are small; the decoder is the largest part |
| UART and the terminal glue | about 100 LUTs and 50 flip-flops | the Uart library's loopback has 15 registers (at 16 bits, wider than it needs) |

Total: well under half of the 1K, by this estimate. The machine-rtl
step measures it with Yosys and nextpnr on the Arch machine, and the
1K or 4K choice is made then.

## The calculator program (a sketch)

Two numbers of eight digits each in RAM files 0 and 1, an operation
code and a digit counter in file 2, the display copied out of file 0.
The key loop: wait for a key (`KNEZ`, `BR`), read it (`TKA`), branch
on its class; a digit shifts file 0 left one word and stores the
digit; an operator settles the pending one into file 0 and remembers
the new one; `=` settles; clear zeroes the files. Add and subtract are
digit loops with the carry in S; multiply is repeated addition by
digits; divide is repeated subtraction (decision 5: whether to have
divide at all at first).

## Decided with the user (2026-10-07)

1. **The name: HEX1000.**
2. **The program counter: a feedback shift register**, the rule above.
3. **Two clocks per instruction**: fetch, then execute; the ROM in
   block RAM.
4. **I/O first: a terminal over the UART**; keys and LEDs later. A
   USB-serial adapter if the board's bridge has no serial port.
5. **Eight digits; add, subtract and multiply; divide later.**
6. **The TMS1000's status rule**: S back to 1 after every instruction
   that does not set it; a branch or call taken when S is 1.
7. **The instruction list as proposed.**
