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
| `lookup %table %i` | the item of a vector at each index (from 0; out of range gives 0) | a ROM when the table is constant, else a mux tree |
| `put %v %i %x` | the vector with the item at one index replaced (out of range: unchanged) | a write port: a mux per word on an address compare |
| `cast E %x` | each item as element type `E`: an integer wrapped (narrower) or sign-extended (wider), a Bool as 0 or 1, to Bool whether it is not 0 | wiring, or a compare with 0 |

## Clocked programs

A program with state is clocked: it runs from reset for a number of
clocks, its nodes computed once per clock from the current state and
the clock's inputs, every state taking its `next` value between
clocks.

```
%x = stream i8 10 20 30 40 50 -60 70
%s = state i8 0
%t = map add %s %x
next %s %t
output %s
trace %s
```

| Form | Meaning | In hardware |
| ---- | ------- | ----------- |
| `%s = state T init...` | registers and their value at reset | flip-flops with a synchronous reset |
| `next %s %v` | what `%s` holds after each clock (one per state, of its type) | the register's input |
| `%x = stream T items...` | one item of type `T` per clock; the clocks are the stream's length (several streams must agree) | an input port; the items are the testbench's |
| `clocks N` | the clocks to run a program with state but no stream | the testbench's clock count |
| `output %v` | the value after the last clock | an output port |
| `trace %v` | the value after reset and after each clock, stacked along a new first axis (a scalar's trace prints as one vector line, a vector's as a matrix) | the port watched every clock |

An output or trace may not depend on a stream: it is read after the
last clock, when no item is present. `run` of the accumulator above
prints `-96`, then `0 10 30 60 100 -106 90 -96`.

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

`matvec`, reductions along an axis of a matrix, and unsigned
integers when an example needs them (the 4-bit machine may: a nibble
is 0 to 15).
