# xetal-fpga-xir

The provisional accelerator IR of X_eTaL-fpga: a typed dataflow
program of named values, its text form, checking, and a reference
interpreter. It says what is computed; the schedule and the netlist
([`components/rtl`](../rtl/README.md)) say how a circuit computes it. It knows
nothing of Verilog.

It stands in for X_eTaL's own accelerator IR (ask F1 in
[`docs/xetal-asks.md`](../../docs/xetal-asks.md)) and is, written as
code and tests, what this repository asks X_eTaL for. Until X_eTaL
lowers its Core IR to it (ask F2), each example program's `.xtl` has
a hand-lowered `.xir` twin beside it, and `scripts/check-equiv.sh`
proves the twin prints exactly what the evaluator printed.

## Values and types

Every value has an element type and a static shape:

| Type | Meaning |
| ---- | ------- |
| `i16` | a signed two's complement integer of 16 bits (any width 1 to 64), a scalar |
| `i16[8]` | a vector of eight |
| `i32[2,3]` | a 2 by 3 matrix |
| `bool[8]` | eight 1-bit values, 0 or 1 (what comparisons give) |

Arithmetic wraps to the width, as Fixed's `w fx:w_rap` does in the
X_eTaL program, so `map add` on `i16` is what a 16-bit adder gives.
The width is the program's (plan A3), never the schedule's.

## The text form

One node per line, `%name = operation operands`, `#` comments, then
the outputs in order:

```
# vector-add: eight 16-bit adders
%a = input i16[8] 1 2 3 4 100 -100 32767 30000
%b = input i16[8] 10 20 30 40 -100 100 1 30000
%c = map add %a %b
output %c
```

| Operation | Gives | In hardware |
| --------- | ----- | ----------- |
| `input T data...` | a program input with its data | input ports, or a ROM in a testbench |
| `const T data...` | a constant | wires tied high or low |
| `map OP %x %y` | elementwise; a scalar operand goes with every element | a unit per element (or fewer, scheduled) |
| `select %m %a %b` | `a` where the Bool `m` is 1, `b` elsewhere | a mux |
| `reduce OP %v` | a vector to a scalar | a tree of units (or an accumulator) |
| `shl %x k`, `shr %x k` | a shift by a constant (`shr` is arithmetic: floor division by 2^k) | wiring |
| `slice %x start len` | part of a vector (start from 0) | wiring |
| `concat %a %b` | two vectors joined | wiring |

`map` operations: `add sub mul min max and or xor` (two integers or,
for `and or xor eq ne`, two Bools), the comparisons `eq ne lt le gt
ge` (giving Bool), and the unary `neg abs` (integers) and `not`
(Bool). `reduce` operations: `add max min` on integers, `and or` on
either. Operands must have one element type and width; there is no
implicit widening (a width change will be its own operation).

Printing a program (`print`) gives the canonical text, comments
dropped, and parsing it again gives the same program.

## Checking and running

`check` gives every value's type or the first error (an undefined
name, a value defined twice, data that does not fit its type, shapes
or element types that disagree, a mask that is not Bool, a reduce of
a non-vector). `run` checks, then interprets, and gives the outputs;
a `Value` prints as `xetal` prints (a scalar, a vector on one line, a
matrix a row per line with right-aligned columns, Bools as 0 and 1).

```bash
just xir-check libs/Datapath/demos/threshold.xir   # each value's type
just xir-run libs/Datapath/demos/threshold.xir     # its outputs
just equiv                                         # every twin against its baseline
```

## Not yet

Registers, ROMs, lookups and the state machine (the Calc library's
step) come in saga 2; widening and narrowing casts, `matvec`, and
reductions along an axis of a matrix when an example needs them.
