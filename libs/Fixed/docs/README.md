# Fixed

Fixed-width two's complement numbers at a stated width: the
arithmetic a w-bit circuit does.

```
"fx:" u_se< "Fixed"
```

Put each library's `src/` directory on `XETAL_PATH` (`just path`
prints them; see the [README](../../../README.md)); `fx:` is the
recommended alias.

X_eTaL's Int is 64 bits wide; hardware is 8, 16 or 32. Until X_eTaL
has sized integer types (ask F3 in
[`docs/xetal-asks.md`](../../../docs/xetal-asks.md)), the width is
written in the program: `16 fx:w_rap x + y` is what a 16-bit adder
gives, the evaluator computes exactly that, and the circuit generated
from the program must match it bit for bit. Every function takes the
width first, one number, and works on whole arrays of any shape. A
function of three arguments is used curried: bind the width once,
`u:a_dd := fx:a_dd 16`, then `x u:a_dd y`.

## Functions

| Function | Type | What |
| -------- | ---- | ---- |
| `fx:l_owest w` | `Int -> Int` | the lowest signed value of `w` bits, `-2^(w-1)` |
| `fx:h_ighest w` | `Int -> Int` | the highest, `2^(w-1) - 1` |
| `w fx:u_wrap x` | `Int -> Int -> Int` | `x` reduced to the unsigned range `0` to `2^w - 1`: the bits a `w`-bit register holds, read unsigned |
| `w fx:w_rap x` | `Int -> Int -> Int` | `x` reduced to the signed range: the same bits read as two's complement |
| `w fx:o_verflow? x` | `Int -> Int -> Bool` | whether `x` does not fit in `w` signed bits |
| `w fx:s_at x` | `Int -> Int -> Int` | `x` clamped to the signed range (saturating arithmetic) |
| `w fx:a_dd x y` | `Int -> Int -> Int -> Int` | the sum wrapped to `w` bits: what a `w`-bit adder gives |
| `w fx:s_ub x y` | `Int -> Int -> Int -> Int` | the difference wrapped |
| `w fx:m_ul x y` | `Int -> Int -> Int -> Int` | the product wrapped |
| `w fx:n_eg x` | `Int -> Int -> Int` | the negation wrapped (the lowest value is its own negation) |
| `w fx:b_its x` | `Int -> Int -> Int` | the two's complement bits of each number, most significant first: a vector for one number, a row per number for a vector |
| `fx:v_alue bits` | `Int -> Int` | the signed number each row of bits (or the vector) stands for; the width is the row's length |
| `w fx:h_ex x` | `Int -> Int -> Char` | the bit pattern of each number as `w/4` hexadecimal digits: text for one number, a row per number for a vector |

The arithmetic is the arithmetic of the ring of integers modulo `2^w`:
wrapping the exact sum, difference or product gives the same bits as
a `w`-bit unit computing on the wrapped inputs, so a circuit may wrap
at every unit or only at the end and agree. `../tests/checks.xtl`
checks that, and the rest, on all 256 patterns of 8 bits.

## Examples

From `../tests/basics.xtl`:

```
      8 fx:w_rap 100 200 -1 -129 256
100 -56 -1 127 0
      8 fx:u_wrap 100 200 -1 -129 256
100 200 255 127 0
      u:a_dd := fx:a_dd 8
      100 u:a_dd 100
-56
      8 fx:s_at 300 -300 5 127 -128
127 -128 5 127 -128
      8 fx:b_its -1 -128 127
1 1 1 1 1 1 1 1
1 0 0 0 0 0 0 0
0 1 1 1 1 1 1 1
      fx:v_alue 0 0 0 0 0 1 0 1
5
      16 fx:h_ex -1 4096 48879
FFFF
1000
BEEF
```

## Demos

- [`demos/overflow.xtl`](../demos/overflow.xtl): what an 8-bit adder
  does with 100 + 100 (wrap, bits, saturation, and 16 bits for
  comparison).
- [`demos/counter.xtl`](../demos/counter.xtl): a 16-bit counter near
  the top of its range, read unsigned and signed, and where the carry
  out happens.

## Provenance

Written for X_eTaL-fpga as the stand-in for sized integer types
(ask F3): when X_eTaL has `Int16` and friends, the widths become
types and this library a shim. The bit functions follow the shape of
X_eTaL-libraries' Bits (`b:b_its`, `b:v_alue`), which reads numbers
unsigned; this library's read them as two's complement.
