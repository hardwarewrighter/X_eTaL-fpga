# HEX1000's calculator program

An eight-digit hexadecimal calculator with `+`, `-`, `*`, `=` and
`X` (clear), written in HEX1000's instructions
([`rom/calc.asm`](../rom/calc.asm)) and assembled into its ROM
(`hx:CalcRom`). It behaves as the Radix library's specification,
key for key: [`tests/calc.xtl`](../tests/calc.xtl) types 33 key
sequences at the machine and compares each screen with
`16 rx:c_alc` on the same text.

```
      "hx:" u_se< "Hex1000"
      hx:c_alc "C0DE+BEEF="
   17FCD
      hx:c_ost "FFFFFFFF*FFFFFFFF="
16406
```

## How it talks to the world

Only through the machine's ports, as a board would:

| Port | Use |
| ---- | --- |
| R0 to R7 | the display's digits, digit 0 on the right: a digit takes O when its latch rises (`SETR` then `RSTR` with Y the digit) |
| O | the digit (A, the low four bits) and whether it shows (bit 4, S at `TDO`): leading zeros are blank, a 0 result shows one 0 |
| R8 to R13 | the key columns, selected one at a time while scanning |
| K | the keys: the key with code c is in column `c / 4` and drives K's bit `c mod 4` while its column is selected (codes as Radix's: 0 to 15 the digits, 16 `+`, 17 `-`, 18 `*`, 19 `=`, 20 `X`) |
| R15 | 1 while the program waits for a key, 0 once it has taken one |

R15 is a handshake: a keyboard, a terminal's glue, or the test
harness (`hx:t_ype`) presents the next key only while R15 is 1 and
releases it when R15 drops. Because K 0 means no key, the matrix is
what lets the 0 key and the 21 keys exist at all (decided with the
user, 2026-10-14; `docs/machine.md`, Keys).

## RAM

| File | Words | What |
| ---- | ----- | ---- |
| 0 | 0 to 7 | the accumulator, word 0 the least significant digit |
| 1 | 0 to 7 | the entry (and the multiplicand while multiplying) |
| 2 | 0 to 7 | the multiplier's digits while multiplying |
| 3 | 0 | the column of the key just read |
| 3 | 1 | the pending operation: 0 none, 1 `+`, 2 `-`, 3 `*` |
| 3 | 2 | the phase: 0 after an operator, 1 entering digits, 2 after `=` |
| 3 | 3 | the digit just keyed |
| 3 | 4 | the operator key's row: 0 `+`, 1 `-`, 2 `*`, 3 `=` |
| 3 | 5 | the multiplier digit being worked on |
| 3 | 8 to 13 | the K bits read from that column |

The accumulator, entry, pending operation and phase are Radix's
state, word for word; the display is not stored, it is shown from the
accumulator or the entry.

## The pages

| Page | What |
| ---- | ---- |
| 0 | reset (show 0); wait until every key is up; R15 up; scan the six columns; on a key, R15 down, find its row from the K bit, and branch on its column: digit, operator, clear |
| 1 | a digit: after `=` a new calculation (accumulator, entry and pending operation cleared), after an operator a new entry; a ninth digit is ignored; the entry times 16 plus the digit; phase 1; show the entry |
| 2 | an operator or `=`: settle the pending operation when digits were keyed since (none: the entry is the result; `+`, `-` by subroutine; `*` on page 3), then remember the new operator (phase 0) or, for `=`, nothing (phase 2); clear the entry; show the accumulator |
| 3 | multiply: the product built in file 0 from the multiplier's top digit down, times 16 then the multiplicand added once per unit of the digit; clear (`X`) |
| 4 | subroutines on file X: `show` (the display, leading zeros blank), `shl` (times 16), `zero` |
| 5 | subroutines: `add` (file 0 plus file 1, the carry through S), `sub` (file 0 minus file 1, the borrow through S) |
| 6 | subroutines: copies between files |

The machine has one subroutine level, so the subroutines call
nothing; the main code calls them after `LDP` of their page, and a
`BR` to another page is always `LDP` then `BR`. Digit arithmetic is
the TMS1000's: a 4-bit add or subtract per word, the carry or borrow
in S taken by the next `BR` into a carry-in copy of the loop.

`just demo-lib Hex1000 listing` prints every page as it sits in the
ROM, in the order it runs.

## Cost

Instructions from reset until the program is ready after the last key
(`hx:c_ost`); a board runs 13.5 million a second (two clocks each at
27 MHz):

| Keys | Instructions | On a board |
| ---- | ------------ | ---------- |
| none (reset, show 0) | 135 | 10 microseconds |
| `1` | 400 | 30 microseconds |
| `12345678` | 2013 | 0.15 ms |
| `12345678+87654321=` | 4434 | 0.33 ms |
| `FFFFFFFF*FFFFFFFF=` | 16406 | 1.2 ms |

Multiply is the slow one: up to 15 additions per multiplier digit.

## Size

About 330 instructions in 7 of the ROM's 16 pages, so octal and
decimal (the next step) have room.
