# Radix

An eight-digit calculator in any radix (hexadecimal, octal, decimal):
the specification HEX1000's ROM program must meet, key for key.

```
"rx:" u_se< "Radix"
```

Put each library's `src/` directory on `XETAL_PATH` (`just path`
prints them; see the [README](../../../README.md)); `rx:` is the
recommended alias.

The radix `r` is the first argument of every function: `16 rx:c_alc
"12+34="`. A function with more arguments is bound to the radix by
applying it, then used as a step: `u:k_ey := rx:k_ey 16`, then
`s u:k_ey k`. The machine is defined in
[`docs/machine.md`](../../../docs/machine.md).

## Numbers

A number is eight digits in radix `r`, as a calculator chip keeps it:
a whole number from 0 to `r^8 - 1`, shown most significant digit
first. Arithmetic wraps like an odometer, modulo `r^8`: in
hexadecimal `FFFFFFFF + 1` is 0 and `5 - 7` is `FFFFFFFE`, the r's
complement a digit-serial subtract leaves.

## The keys and the rules

| Key | Code | What it does |
| --- | ---- | ------------ |
| `0` to `9`, `A` to `F` | 0 to 15 | a digit, appended to the entry; ignored when not below the radix (8 and 9 in octal, `A` to `F` in decimal) or when the entry already has eight digits |
| `+` `-` `*` | 16 17 18 | the pending operation settles, this one is pending, the display shows the result so far |
| `=` | 19 | the pending operation settles and the result is shown; the next digit starts a new calculation, an operator continues from the result |
| `X` | 20 | clear (not `C`, a hexadecimal digit) |

Operations apply left to right as keyed (`2*3+4=` is A). An operator
before any digit uses 0; a second operator in a row replaces the
first; `=` with nothing pending shows the entry. Divide is not there
yet (decided with the user: later). Other characters are ignored.

The screen is eight characters: the digits, leading zeros blank, zero
itself one `0`.

## Functions

| Function | Type | What |
| -------- | ---- | ---- |
| `rx:Width` | `Int` | the digits a number has, 8 |
| `rx:m_odulus r` | `Int -> Int` | `r^8`, one more than the largest number |
| `r rx:d_igits n` | `Int -> Int -> Int` | the eight digits of `n` (wrapped), most significant first |
| `r rx:v_alue ds` | `Num a => a -> a -> a` | the number digits stand for |
| `r rx:w_rap n` | `Int -> Int -> Int` | `n` modulo `r^8` |
| `rx:k_eys text` | `Char -> Int` | the key codes of a text (spaces skipped, other characters 21) |
| `rx:KeyPlus`, `rx:KeyMinus`, `rx:KeyTimes`, `rx:KeyEquals`, `rx:KeyClear` | `Int` | the codes 16 to 20 |
| `rx:Reset` | `Int` | the state at power on: accumulator, entry, pending operation, phase, display, all 0 |
| `r rx:a_lu ops` | `Int -> Int -> Int` | the operation `ops[1]` (1 +, 2 -, 3 *) of `ops[2]` and `ops[3]`, wrapped (the operands as one vector) |
| `r rx:k_ey s k` | `Int -> Int -> Int -> Int` | the state after one key: the calculator's step |
| `rx:d_isplay s` | `a -> a` | the number displayed |
| `r rx:s_creen s` | `Int -> Int -> Char` | the display as eight characters |
| `r rx:r_un keys` | `Int -> Int -> Int` | the state after the keys, from Reset (the harness, a loop over the keys) |
| `r rx:c_alc text` | `Int -> Char -> Char` | the screen after the keys of a text, from Reset |

## Examples

From `../tests/basics.xtl`:

```
      16 rx:d_igits 255
0 0 0 0 0 0 15 15
      rx:k_eys "12+34="
1 2 16 3 4 19
      16 rx:a_lu 2 5 7
4294967294
      16 rx:r_un rx:k_eys "12+34="
70 0 0 2 70
      16 rx:c_alc "12+34="
      46
```

`../tests/checks.xtl` checks 23 calculations and identities with the
Check library: hexadecimal sums, products, wraps above and below,
chaining, the ninth digit ignored; octal and decimal, their digits
outside the radix ignored and their wraps; continuing after `=`,
starting over, clear, `=` alone, an operator first, a replaced
operator, the display at an operator; digits and value as inverses.

## Demos

- [`demos/session.xtl`](../demos/session.xtl): a hexadecimal session,
  the screens HEX1000's ROM program must show.
- [`demos/radixes.xtl`](../demos/radixes.xtl): one key sequence in
  the three radixes.

## Provenance

Written for X_eTaL-fpga, saga 3 (nibble-machine), as the reference
for HEX1000's calculator program. The arithmetic is defined on whole
numbers; HEX1000 does it a digit at a time, and its program is
checked against this library for every test sequence.
