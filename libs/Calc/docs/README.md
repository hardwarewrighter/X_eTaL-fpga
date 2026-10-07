# Calc

A four-function calculator as a state vector and a pure
step function.

```
"calc:" u_se< "Calc"
```

Put each library's `src/` directory on `XETAL_PATH` (`just path`
prints them; see the [README](../../../README.md)); `calc:` is the
recommended alias. The library imports Fixed itself.

A calculator the way a chip has it: a few registers (the state), a
keyboard (one key code per step), a decoder, an ALU and a display.
`calc:s_tep` is pure, one key in and the next state out: it is the
circuit's next-state logic, what saga 2 lowers to registers, a
decoder and an ALU. `calc:r_un` folds it over a key sequence and is
only the harness, the testbench's clock (X_eTaL has no left fold yet,
ask F5 in [`docs/xetal-asks.md`](../../../docs/xetal-asks.md), so it
recurses).

## The state

One Int vector of five registers, every number 16-bit two's
complement ([Fixed](../../Fixed/docs/README.md)), as the chip's
registers would be:

| Register | Holds |
| -------- | ----- |
| `acc` | the accumulator: the first operand, then the running result |
| `entry` | the number being keyed in |
| `op` | the pending operation: 0 none, 1 `+`, 2 `-`, 3 `*`, 4 `/` |
| `phase` | 0 after an operator (the next digit starts an entry), 1 entering digits, 2 after `=` (the result is shown) |
| `display` | the number shown |

`calc:Reset` is `0 0 0 0 0`, the state at power on.

## The keys

A key is a code: the digits are 0 to 9, then `+` 10, `-` 11, `*` 12,
`/` 13, `=` 14, `C` 15 (a key's code is its place in
`"0123456789+-*/=C"`, less 1). The decoder gives a key's class: 0 a
digit, 1 an operator, 2 equals, 3 clear, 4 none.

| Key class | What the step does |
| --------- | ------------------ |
| digit | appended to the entry (`entry * 10 + d`, wrapped); after `=` it starts a new calculation |
| operator | the pending operation settles (the ALU takes the entry, or the entry alone becomes the accumulator when none is pending), this one is pending, the display shows the accumulator |
| `=` | the pending operation settles, the result is shown, the next digit starts over |
| `C` | every register 0 |

Operations apply left to right as keyed, as a four-function
calculator does (`2 * 3 + 4 =` is 10). A division is whole-number,
and a division by zero gives 0 rather than trapping. The ALU computes
the four results side by side and keeps the one the op selects, a
mux written as a masked sum.

## Functions

| Function | Type | What |
| -------- | ---- | ---- |
| `calc:k_eys text` | `Char -> Int` | the key codes of a text such as `"12+34="` (spaces skipped; another character gives 16, which the step ignores) |
| `calc:d_ecode key` | `(Num a, Truthy a) => Int -> a` | the class of every key, 0 to 4 |
| `calc:Reset` | `Int` | the state at power on |
| `op calc:a_lu acc entry` | `Num a => a -> Int -> Int` | the operation applied to the two operands, a 2-item vector (`1 calc:a_lu 12 34` is 46), 16 bits |
| `s calc:s_tep key` | `Int -> Int -> Int` | the next state (pure) |
| `s calc:r_un keys` | `Int -> Int -> Int` | the state after every key |
| `s calc:t_race keys` | `Int -> Int -> Int` | every state, a row per key, the start first |
| `calc:d_isplay s` | `a -> a` | the number shown |
| `calc:c_alc text` | `Char -> Int` | the display after the keys of a text, from Reset |

## Examples

From `../tests/basics.xtl`:

```
      calc:k_eys "12+34="
1 2 10 3 4 14
      1 calc:a_lu 12 34
46
      calc:Reset calc:r_un calc:k_eys "12+34="
46 0 0 2 46
      calc:Reset calc:t_race calc:k_eys "2+3="
0 0 0 0 0
0 2 0 1 2
2 0 1 0 2
2 3 1 1 3
5 0 0 2 5
      calc:c_alc "123-27="
96
      calc:c_alc "60000+10000="
4464
```

`../tests/checks.xtl` checks the calculations of the Check library's
report: the four operations, chains, a negative result, 16-bit wrap,
continuing from a result, clear, and that `r_un` is `s_tep` folded.

## Demos

- [`demos/session.xtl`](../demos/session.xtl): a session, each line
  a key sequence and its display.
- [`demos/trace.xtl`](../demos/trace.xtl): the registers key by key
  for `12 + 34 =`, what the circuit's registers will hold on each
  clock.

## Provenance

Written for X_eTaL-fpga after research7's PoC 1 ("Wrighting a
calculator") as a behavioral model: what a calculator displays, with
16-bit binary registers and a one-clock ALU. It is not the demo
machine. The demo is a 4-bit machine in the spirit of the TMS1000,
our own design whose calculator is a program in its ROM (see
[`docs/calc-approaches.md`](../../../docs/calc-approaches.md) and
saga 3 of [`docs/plan.md`](../../../docs/plan.md)); this library stays
as an example of a sequential model until the user decides its fate.
