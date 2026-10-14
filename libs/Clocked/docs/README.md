# Clocked

Clocked circuits as X_eTaL step functions: counters, an accumulator,
a shift register, a sequence detector, a RAM, and the harness that
clocks them.

```
"ck:" u_se< "Clocked"
```

Put each library's `src/` directory on `XETAL_PATH` (`just path`
prints them; see the [README](../../../README.md)); `ck:` is the
recommended alias. The library imports Fixed itself.

A clocked circuit is `state' = step(state, input)` at every clock.
Here a step is an ordinary pure dyadic function, `s f_ x`: the state
on the left, one clock's input on the right, the next state out. That
function is what becomes hardware: registers for the state, the
function as the logic between them. The harness, `ck:r_un` and
`ck:t_race`, applies it once per input, as a clock would; it is the
testbench, never lowered. It is a loop, `p_ower` over a tuple of the
clock count and the state, because X_eTaL has no left fold yet (ask F5
in [`docs/xetal-asks.md`](../../../docs/xetal-asks.md)); with one,
`s0 'step ck:r_un xs` becomes a built-in fold. Numbers are
8-bit two's complement ([Fixed](../../Fixed/docs/README.md)).

## Functions

| Function | Type | What | In hardware |
| -------- | ---- | ---- | ----------- |
| `s0 'step ck:r_un xs` | `Any a => (a -> b -> a) -> a -> b -> a` | the state after every input (an item, or a row of a matrix, per clock) | the testbench: reset, then one input per clock |
| `s0 'step ck:t_race xs` | `(a -> b -> a) -> a -> b -> a` | every state from `s0` on, a row per clock (a vector for a scalar state) | the registers after reset and after each clock |
| `s ck:t_ick e` | `Int -> Int -> Int` | an 8-bit counter that counts when its enable `e` is 1 | an adder and a register |
| `s ck:a_ccumulate x` | `Int -> Int -> Int` | an 8-bit accumulator | an adder and a register |
| `s ck:s_hift b` | `a -> a -> a` | the bits move one place toward the front and `b` comes in at the end | flip-flops and wiring, no logic |
| `ck:Next101` | `Int` | the next-state table of the 1 0 1 detector, item `1 + s + 4 * b` | a ROM of eight 2-bit words |
| `s ck:d_etect b` | `Int -> Int -> Int` | the detector's next state: 0 nothing yet, 1 seen 1, 2 seen 1 0, 3 seen 1 0 1 | a ROM read and a register |
| `ck:f_ound? s` | `(Num a, Truthy b) => a -> b` | whether the state says 1 0 1 was just seen (the Moore output) | a comparator |
| `m ck:w_rite c` | `Int -> Int -> Int` | the memory after one clock of a write port; `c` is write-enable, address (from 0), data | a register per word, a mux in front of each |
| `m ck:r_ead a` | `a -> Int -> a` | the word at address `a` (from 0) | a read port, a mux |

The step functions are the ones whose hand-lowered twins
(`demos/*.xir`) run as clocked circuits: the intermediate form's
`state`, `next`, `stream`, `lookup`, `put` and `cast`
([`components/xir`](../../../components/xir/README.md)).

## Examples

From `../tests/basics.xtl`:

```
      127 ck:t_ick 1
-128
      (0 0 0 1) ck:s_hift 1
0 0 1 1
      2 ck:d_etect 1
3
      (4 r_eshape 0) ck:w_rite 1 2 9
0 0 9 0
      0 'ck:a_ccumulate ck:r_un 1 2 3 4
10
      (0 0 0) 'ck:s_hift ck:t_race 1 1 0
0 0 0
0 0 1
0 1 1
1 1 0
```

`../tests/checks.xtl` checks, with the Check library: the
accumulator's last state and trace against the wrapped sum and running
sum, the counter against the count of enables and its wrap, the shift
register against the last bits, the detector against every 1 0 1
(overlapping ones too), a trace's shape, the RAM's disabled write, its
addresses and the last write winning, and its 8-bit wrap.

## Demos

Each demo's output is the reference for its clocked circuit; the
twin beside it (`demos/NAME.xir`) is checked to reproduce it, as
interpreted and as a simulated circuit (`just equiv`), and its
Verilog is pinned in [`rtl/`](../rtl/).

- [`demos/counter.xtl`](../demos/counter.xtl): an 8-bit counter with an
  enable, ten clocks from 120, wrapping past 127.
- [`demos/accumulator.xtl`](../demos/accumulator.xtl): the final sum
  and the running sum, wrapped.
- [`demos/shift.xtl`](../demos/shift.xtl): a 4-bit shift register fed
  six bits.
- [`demos/detect.xtl`](../demos/detect.xtl): the 1 0 1 detector's
  states on a bit stream, and where it fires.
- [`demos/ram.xtl`](../demos/ram.xtl): eight 8-bit words written by
  five commands, one disabled, one wrapping.

## Provenance

Written for X_eTaL-fpga, saga 2 (fpga-state): the sequential pieces
the 4-bit machine of saga 3 is built from (registers, a ROM, a RAM, a
state machine), each a pure X_eTaL step function first.
