# xetal-fpga-rtl

The hardware side of X_eTaL-fpga: an XIR program
([`components/xir`](../xir/README.md)) is scheduled into a word-level
netlist, which a cycle simulator runs and a Verilog-2005 emitter
prints. It knows nothing of X_eTaL syntax.

## The schedule

The schedule is data, never part of the program (plan A8): `lanes`
says how many elements of a vector are computed at once.

| Lanes | Circuit | For eight 16-bit numbers added |
| ----- | ------- | ------------------------------ |
| as many as the longest vector (the default) | spatial: combinational, a unit per element, a reduction a balanced tree | eight 16-bit adders, no clock |
| fewer | temporal: a counter steps through the vector, `lanes` units compute that many elements per clock, vector outputs are collected in registers, a reduction is a tree of `lanes` units into an accumulator | one adder and a counter, eight clocks (one lane) |

The reduction tree adds the first half to the second half, item by
item, down to one (an odd last item is carried), as the Datapath
library's `dp:s_um` does. With a lane count that does not divide the
vector, the lanes past the end contribute the reduction's identity.
The temporal schedule takes programs whose vectors all have one
length and whose reductions feed only scalars and outputs (a
two-pass program, such as `x - mean x`, needs the spatial one); it
refuses the rest with a message saying so.

## The netlist and the simulator

A cell produces one word of an element type (an integer of its
width, or a 1-bit Bool): an input element, a constant, a unit (any
of the IR's `map` operations), a mux, a shift by a constant, a
`select 1 of n` (a mux tree on an address), or a register (its init
on reset, then its `d` at every clock). `Netlist::simulate` resets,
clocks until `done` (at once for a combinational circuit) and gives
the outputs and the clocks taken. The units compute exactly as the
IR interpreter does, so the simulated circuit gives the evaluator's
answer bit for bit.

## Verilog

Plain Verilog-2005 (plan A7): `assign`, one `always @(posedge clk)`
with a synchronous reset, no vendor primitives. A port is a bus of
its elements, element 0 in the low bits; integers are `signed` wires
of their width, so a 16-bit `+` wraps as the program's does; Bools are
1-bit wires. A sequential module has `clk`, `rst` and a `done`
output. `testbench` writes a self-checking testbench that drives the
program's input data, clocks until done and compares every output
with the expected bits, printing `PASS` or `FAIL` lines.

## Use

```bash
just explain libs/Datapath/demos/pipeline.xir      # spatial: 15 adders, 8 max, 8 multipliers
just explain libs/Datapath/demos/pipeline.xir 2    # two lanes, four clocks
just sim libs/Datapath/demos/pipeline.xir 3        # three lanes: the outputs, clocks: 3
just verilog libs/Datapath/demos/vector-add.xir    # eight adders as Verilog
just rtl                                           # the pinned Verilog; iverilog and yosys when installed
```

The Verilog of every twin, spatial and with one lane, is pinned in
`libs/<Lib>/rtl/` and checked by `scripts/check-rtl.sh` (in the
gate); `just bless-rtl` rewrites it after a reviewed change. When
`iverilog` and `yosys` are installed, the same script simulates each
module with its testbench and synthesizes it; when they are not, it
says so.

## Not yet

Registers and state written in the program (a step function over a
state vector, as the 4-bit machine of saga 3 is), ROMs, a UART, the board's pins and clock: saga 2 and 3.
