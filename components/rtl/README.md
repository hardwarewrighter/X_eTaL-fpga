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

## Clocked programs

A program with state takes the spatial schedule (lanes are for
vectors; asking a clocked program for fewer lanes than its longest
vector is refused): each `state` is a register per element with its
reset value, each `next` that register's input, each `stream` an
input port the testbench drives with one item per clock, a constant
`lookup` table a ROM, a `put` a write port (an address compare and a
mux per word; an index too narrow to name a word never writes it). A
constant index is wiring: a `lookup` at a constant is the item
itself, a `put` at a constant replaces that word, so reading and
writing the fields of a state vector costs no logic.
The simulator resets, runs the stream's clocks, and gives the outputs
after the last clock and the traces after reset and every clock.

## The netlist and the simulator

A cell produces one word of an element type (an integer of its
width, or a 1-bit Bool): an input element, a constant, a unit (any
of the IR's `map` operations), a mux, a shift by a constant, a
`select 1 of n` (a mux tree on an address), a ROM read, a cast, or a
register (its init on reset, then its `d` at every clock). `Netlist::simulate` resets,
clocks until `done` (at once for a combinational circuit) and gives
the outputs and the clocks taken. The units compute exactly as the
IR interpreter does, so the simulated circuit gives the evaluator's
answer bit for bit.

## Verilog

Plain Verilog-2005 (plan A7): `assign`, one `always @(posedge clk)`
with a synchronous reset, no vendor primitives. A port is a bus of
its elements, element 0 in the low bits; integers are `signed` wires
of their width, so a 16-bit `+` wraps as the program's does; Bools are
1-bit wires. A module with registers has `clk` and `rst`; a temporal
one also a `done` output. A ROM is a memory initialized in an
`initial` block, read through a range guard (Yosys maps it to LUTs;
block RAM wants a registered read, a later refinement for the 4-bit
machine's program store). `testbench` writes a self-checking testbench that drives the
program's input data, clocks until done and compares every output
with the expected bits, printing `PASS` or `FAIL` lines; for a
clocked module it resets, drives one stream item per clock, and
checks every output (a traced one at its last value).

## Verification status

As of 2026-10-14 every pinned module passes iverilog (its
self-checking testbench, after reset and after every clock), Verilator
(`--lint-only -Wall`; the warnings left are unused cells and an index
wider than its table) and Yosys (generic and Gowin synthesis). Their
first run found one bug the netlist simulator could not: an output
also traced was declared as two ports of the same name (fixed; a test
pins it). Casts now change width explicitly (cut to the low bits,
sign-extended, or a Bool zero-extended) rather than by Verilog's
implicit assignment rules.

| Library | Modules in `libs/<Lib>/rtl/` |
| ------- | ---------------------------- |
| Datapath | `vector-add`, `saxpy`, `threshold`, `reduce-sum`, `pipeline`, each spatial and `.lanes1` (ten) |
| Clocked | `counter`, `accumulator`, `shift`, `detect` (a ROM), `ram` (five) |
| Fixed | `counter` (one) |
| Uart | `loopback` (one) |

`just fit` gives each module's LUTs, flip-flops, ALUs, block RAM and
multipliers on a Gowin GW1N against the Tang Nano 1K. The deeper
checks (Gowin cells simulated, the bitstream round trip) are planned:
`docs/hardware.md`.

## Use

```bash
just explain libs/Datapath/demos/pipeline.xir      # spatial: 15 adders, 8 max, 8 multipliers
just explain libs/Datapath/demos/pipeline.xir 2    # two lanes, four clocks
just sim libs/Datapath/demos/pipeline.xir 3        # three lanes: the outputs, clocks: 3
just verilog libs/Datapath/demos/vector-add.xir    # eight adders as Verilog
just rtl                                           # the pinned Verilog; iverilog, verilator and yosys when installed
just fit                                           # each module on a Gowin GW1N against the Tang Nano 1K (yosys)
just wave libs/Clocked/demos/detect.xir            # a waveform of one circuit, work/rtl/detect.vcd (iverilog)
```

The Verilog of every twin, spatial and with one lane, is pinned in
`libs/<Lib>/rtl/` and checked by `scripts/check-rtl.sh` (in the
gate); `just bless-rtl` rewrites it after a reviewed change. When
`iverilog`, `verilator` and `yosys` are installed, the same script
simulates each module with its testbench, lints it and synthesizes
it; when they are not, it says so.

## Not yet

Memory cells for block RAM (a computed-address RAM, the ROM with a
registered read), the board's pins and clock.
