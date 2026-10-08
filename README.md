# X_eTaL-fpga

X_eTaL programs as hardware.

[X_eTaL](https://github.com/softwarewrighter/X_eTaL), the eXperimental
Extensible Typed Array Language, states a computation as whole-array
operations: `c := a + b`, `'+ r_/ c`. Those operations already expose
their parallelism and their dataflow, so the same program can run as
sequential software (the X_eTaL evaluator), as thousands of parallel
threads (a GPU, in
[X_eTaL-gpu](https://github.com/softwarewrighter/X_eTaL-gpu)) or as a
circuit. This repository is the circuit part: `c := a + b` on eight
16-bit numbers becomes eight adders, `'+ r_/ v` becomes a tree of
adders, a 4-bit machine written as a step function over a state vector
becomes registers, a decoder and an ALU. A small accelerator IR, a
netlist with a cycle simulator, a Verilog-2005 emitter, a schedule
that decides how much of the computation is space and how much is
time (lanes), and the open Gowin flow (Yosys, nextpnr, Apicula,
openFPGALoader) to a small Tang Nano (1K first, or 4K).

The goal is the concept, one typed array program realized as
sequential time, parallel threads or physical space with the same
answer, not speed. The software organization
([softwarewrighter](https://github.com/softwarewrighter)) maps array
programs onto hardware someone else designed; this organization
([hardwarewrighter](https://github.com/hardwarewrighter)) uses the
array program to describe the hardware itself.

## What this is

| Part | What it holds |
| ---- | ------------- |
| `libs/Fixed/` | fixed-width two's complement numbers (`"fx:" u_se< "Fixed"`): the width written in the program, the evaluator computing what a w-bit adder gives |
| `libs/Clocked/` | clocked circuits as step functions (`ck:`): a counter, an accumulator, a shift register, a sequence detector, a RAM, and the harness that clocks them |
| `libs/Uart/` | a serial port as step functions (`ua:`): a transmitter, a receiver, and the two looped back; the clocks per bit as schedule data |
| `libs/Radix/` | an eight-digit calculator in any radix (`rx:`): the specification HEX1000's ROM program must meet, key for key |
| `libs/Datapath/` | the vector examples at a stated width (`dp:`): add, saxpy, threshold, the reduction tree, the acceptance pipeline; its demos are the programs the circuits realize |
| `components/xir/` | the provisional accelerator IR (typed values with widths and shapes, elementwise maps, select, reductions, later registers and state), its text form and a reference interpreter |
| `components/rtl/` | the netlist, the lanes schedule and the reduction tree, the cycle simulator, the Verilog-2005 emitter and testbenches |
| `components/cli/` | the `xetal-fpga` tool: `check`, `run`, `netlist`, `verilog`, `testbench`, `sim`, `explain` |
| `docs/` | the plan, the asks for X_eTaL, the research notes |

Every example is an X_eTaL program first; the pinned X_eTaL evaluator
is the reference, and every realization must reproduce its answer bit
for bit:

```
                X_eTaL program (.xtl)
                        |
      +---------+-------+--------+-----------+
      v         v                v           v
  evaluator  XIR interpreter  netlist sim  Verilog (iverilog)
      |         |                |           |
      +---------+-------+--------+-----------+
                        v
                  the same arrays
```

The accelerator IR and the lowering from X_eTaL's Core IR to it belong
to X_eTaL and are not there yet (see
[`docs/xetal-asks.md`](docs/xetal-asks.md), asks F1 to F4), so for now
each example's `.xtl` has a hand-lowered `.xir` twin, and a check
proves the twin's interpretation equals the evaluator's output. The
twins go when X_eTaL's lowering lands.

## Status

Saga 1 (fpga-foundation) is done: the process and the pinned X_eTaL
(75e6a5c) and X_eTaL-libraries (8adcd0d); the hardware models as
tested X_eTaL libraries (Fixed, and Datapath with the example
programs); the provisional IR with its
interpreter; the netlist, the lanes schedule (spatial, or a counter
with any number of lanes), the cycle simulator and the Verilog-2005
emitter with self-checking testbenches. Every Datapath example's
answer from the evaluator is reproduced, bit for bit, by the IR
interpreter and by its circuits simulated at several lane counts, and
their Verilog is pinned. Saga 2 (fpga-state) is under way: clocked
programs (state, input streams, ROMs, a RAM's write port) in the IR,
the netlist, the simulator and the Verilog, with the Clocked library
as their specification (step 1); a UART, transmitter and receiver
looped back, from X_eTaL to a clocked circuit (step 2); self-checking
testbenches that check every clock and write waveforms, and Verilator
lint, ready for the Arch machine (step 3). Saga 2 is done. Next,
saga 3: HEX1000, a 4-bit calculator machine in the spirit of the
TMS1000 (our own design, hexadecimal first), sized for a Tang Nano
1K; its architecture is decided ([`docs/machine.md`](docs/machine.md))
and its calculator's specification written (the Radix library).
See
[`docs/plan.md`](docs/plan.md) for the roadmap and the architecture
decisions.

Nothing here needs an FPGA or the FPGA tools yet. The Verilog is
proven through the Rust simulator of the same netlist; the gate also
runs it through iverilog (each module with its self-checking
testbench), Verilator (lint) and Yosys (synthesis) when they are
installed, and says so when it skips them. None has been run yet:
the tools go on the Arch machine with the boards (see the RTL
component's verification status). `just wave FILE.xir` writes a
waveform of one circuit for gtkwave.

## Build

Prerequisites:

- [Rust](https://rustup.rs) (stable) and
  [`just`](https://github.com/casey/just)
  (`brew install just` or `cargo install just`)
- optional, for the Verilog checks: `iverilog` (Icarus Verilog),
  `yosys`, `verilator`, and `gtkwave` to look at waveforms (macOS:
  `brew install icarus-verilog yosys verilator`; Arch: `pacman -S
  iverilog yosys verilator gtkwave`)
- optional, for the board (on the Arch machine, where the boards
  are): nextpnr with the Gowin target (`nextpnr-himbaechel`), Apicula
  (`gowin_pack`) and openFPGALoader
- for the gate (maintainers): `reg-rs` and `sw-markdown-checker`

```bash
just                                 # list the tasks
just xetal                           # clone and build the pinned X_eTaL (bin/xetal)
just libs-get                        # the pinned X_eTaL-libraries (Check, Bits, Strings)
just eval "'+ r_/_2 2 3 r_eshape r_ange 6"   # try it: row sums, 6 15
just test                            # every library's baselines, the components, the equivalence checks
just gate                            # the pre-commit gate
```

X_eTaL is not tracked here: `XETAL_COMMIT` pins a known-good commit
(one line, the full SHA), `just xetal` clones X_eTaL into `work/xetal/`
(gitignored), checks it out, builds the CLI and links it as
`bin/xetal`. The first build takes a few minutes;
`XETAL_SOURCE=../../softwarewrighter/X_eTaL just xetal` clones from a
sibling checkout instead of GitHub. X_eTaL-libraries is pinned the
same way (`XETAL_LIBRARIES_COMMIT`, `work/xetal-libraries/`, the
libraries used linked under `work/libs/`). Maintainers move to a
newer X_eTaL with `just xetal-pin [REF]` (and `just libs-pin [REF]`),
in its own commit after `just gate` passes.

## Try it

```bash
just demo-lib Datapath vector-add                  # the evaluator: a + b, then eight 16-bit adders' sum
just xir-run libs/Datapath/demos/vector-add.xir    # the same sum from the hand-lowered IR
just equiv                                         # every twin and its circuits against the evaluator
just explain libs/Datapath/demos/vector-add.xir    # spatial: eight 16-bit adders
just explain libs/Datapath/demos/vector-add.xir 1  # temporal: one adder, a counter, eight clocks
just verilog libs/Datapath/demos/reduce-sum.xir 1  # the Verilog of a one-lane reduction
just sim libs/Datapath/demos/pipeline.xir 3        # the pipeline circuit with three lanes, simulated
just demo-lib Clocked detect                       # a 1 0 1 detector in X_eTaL: states and where it fires
just sim libs/Clocked/demos/detect.xir             # the same as a clocked circuit: a ROM and a register
just verilog libs/Clocked/demos/ram.xir            # a RAM of eight words as Verilog
just demo-lib Uart loopback                        # "Hi" over a serial line: the line, the bytes received
just explain libs/Uart/demos/loopback.xir          # the same as a circuit: fifteen registers
just demo-lib Radix session                        # the hexadecimal screens HEX1000 must show
```

## Libraries and examples

```bash
just libs                            # the libraries: name, alias, what
just run-lib Fixed                   # a library's test programs
just demo-lib Datapath vector-add    # one of its demos (an example program)
just types Fixed                     # its exported names and types
just test-lib Fixed                  # its reg-rs baselines
```

Each library is its own directory, `libs/<Name>/` (the layout of
X_eTaL-libraries): `src/<Name>.xtl`, `tests/` (reg-rs baselines and
the pinned export types), `demos/` (programs that use it; here, the
example programs the circuits realize) and `docs/README.md` (the
reference). `just path` prints the directories for `XETAL_PATH`,
including the pinned libraries from X_eTaL-libraries.

## Documentation

- [`docs/plan.md`](docs/plan.md) -- architecture decisions, the
  subset, the roadmap
- [`docs/xetal-asks.md`](docs/xetal-asks.md) -- what this repo needs
  from X_eTaL; the blockers first
- `docs/research7.txt` -- the planning conversation (archival, not
  normative)
- [`CHANGES.md`](CHANGES.md) -- every commit
- [`CLAUDE.md`](CLAUDE.md) (also `AGENTS.md`) -- the agent workflow
  (agentrail sagas) and rules

## Development

Development is tracked with agentrail sagas, as in the sibling
repositories: `agentrail status` shows the current step,
`agentrail next` its instructions. Every step ends with the gate
passing, docs updated, a commit to `main` and a push.

## Related Projects

- [X_eTaL](https://github.com/softwarewrighter/X_eTaL) -- the language
  ([try it live](https://softwarewrighter.github.io/X_eTaL/))
- [X_eTaL-gpu](https://github.com/softwarewrighter/X_eTaL-gpu) -- the
  same array programs on old GPUs
- [X_eTaL-libraries](https://github.com/softwarewrighter/X_eTaL-libraries)
  -- reusable `.xtl` and `.xtlm` libraries (Check and Bits are used here)
- [X_eTaL-ML](https://github.com/softwarewrighter/X_eTaL-ML) -- the
  machine-learning demos and libraries (the layer to wright, later)
- [X_eTaL-demos](https://github.com/softwarewrighter/X_eTaL-demos) --
  visual array programs

## Links

- Blog: [Software Wrighter Lab](https://software-wrighter-lab.github.io/)
- Discord: [Join the community](https://discord.com/invite/Ctzk5uHggZ)
- YouTube: [Software Wrighter](https://www.youtube.com/@SoftwareWrighter)

## Copyright

Copyright (c) 2026 Michael A Wright

## License

MIT. See [`LICENSE`](LICENSE) and [`COPYRIGHT`](COPYRIGHT).
