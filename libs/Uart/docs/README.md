# Uart

A serial port (UART: 8 data bits, no parity, 1 stop bit) as X_eTaL
step functions: a transmitter, a receiver, and the two looped back.

```
"ua:" u_se< "Uart"
```

Put each library's `src/` directory on `XETAL_PATH` (`just path`
prints them; see the [README](../../../README.md)); `ua:` is the
recommended alias. The steps are clocked with the
[Clocked](../../Clocked/docs/README.md) library's harness.

## The frame and the clocks per bit

A line idles at 1. A byte is sent as a frame of ten bits, each held
for `d` clocks: a start bit (0), the eight data bits, least
significant first, and a stop bit (1). `d`, the clocks per bit, is
the clock rate over the baud rate: 234 for a 27 MHz Tang Nano at
115200 baud, 3 in the tests. It is the first argument of every step,
bound once by applying the step to it, then the bound step is clocked:

```
u:t_x := ua:t_x 3
ua:TxIdle 'u:t_x ck:r_un 3 ua:c_ommands 72 105
```

In the circuit `d` is a constant, schedule data a board sets: the
twin marks the three constants that follow from it.

## The states

| State | Registers |
| ----- | --------- |
| transmitter, `ua:TxIdle` | busy (0 or 1), clocks left in this bit, bits left in the frame, the frame (bit 0 on the line) |
| receiver, `ua:RxIdle` | phase (0 idle, 1 receiving, 2 waiting for the stop bit), clocks to the next sample, bits left, the bits so far, the last byte received, valid (1 for one clock when a byte arrives) |
| looped back, `ua:LoopIdle` | the transmitter's four, the receiver's six, a log of four bytes, the count |

The receiver takes a fall to 0 while idle as a start bit and samples
each data bit in its middle: `d + d/2` clocks after the fall, then
every `d` clocks. In the loopback the receiver samples the line as
the transmitter drives it on the same clock, then both step.

## Functions

| Function | Type | What |
| -------- | ---- | ---- |
| `ua:TxIdle`, `ua:RxIdle`, `ua:LoopIdle` | `Int` | the states at reset (all 0) |
| `d ua:t_x s c` | `Int -> Int -> Int -> Int` | the transmitter after one clock; a command `c` of 0 to 255 starts that byte when idle, -1 is none, a command while busy is ignored |
| `ua:l_ine s` | `Int -> Int` | the transmitter's line: 1 when idle, else the frame's bit 0 |
| `ua:l_ines t` | `Int -> Int` | the line for every row of a trace (of the transmitter or the loopback) |
| `d ua:r_x s l` | `Int -> Int -> Int -> Int` | the receiver after one clock of the line `l` |
| `ua:r_eceived s` | `a -> a` | the last byte received |
| `ua:v_alid? s` | `(Num a, Truthy b) => a -> b` | whether it arrived on the last clock |
| `d ua:l_oopback s c` | `Int -> Int -> Int -> Int` | the transmitter wired to the receiver, one clock, the byte logged |
| `ua:l_og s` | `Int -> Int` | the bytes received so far, from a looped-back state |
| `d ua:c_ommands bytes` | `(Num a, Truthy a) => Int -> a -> a` | one command per clock: each byte at the start of a frame of `10 d + 1` clocks, -1 elsewhere, `d` more clocks at the end |

## Examples

From `../tests/basics.xtl`, at 3 clocks per bit:

```
      u:t_x := ua:t_x 3
      ua:TxIdle u:t_x 65
1 2 10 642
      3 ua:c_ommands 65
65 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1 -1
      u:l_oop := ua:l_oopback 3
      ua:l_og ua:LoopIdle 'u:l_oop ck:r_un 3 ua:c_ommands 65
65
```

`../tests/checks.xtl` checks, with the Check library: four bytes (0,
255, 85, 170) come back at 3 and 4 clocks per bit, two at 8; the line
idles at 1; the frame of 77 is start, data low bit first, stop, four
clocks each; a command while busy is ignored; valid lasts one clock
per byte.

## Demos

- [`demos/loopback.xtl`](../demos/loopback.xtl): "Hi" (72 105)
  through the looped-back pair: the line clock by clock, the bytes
  logged, their count. Its twin, `demos/loopback.xir`, writes each
  guard of the steps as a chain of selects; it reproduces the demo
  interpreted and as a clocked circuit of fifteen registers (`just
  equiv`), and its Verilog is pinned in [`rtl/`](../rtl/).

## Provenance

Written for X_eTaL-fpga, saga 2 (fpga-state): the serial line the
4-bit machine of saga 3 will talk to a host through on a Tang Nano.
