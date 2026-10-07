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
adders, a calculator written as a step function over a state vector
becomes registers, a decoder and an ALU. A small accelerator IR, a
netlist with a cycle simulator, a Verilog-2005 emitter, a schedule
that decides how much of the computation is space and how much is
time (lanes), and the open Gowin flow (Yosys, nextpnr, Apicula,
openFPGALoader) to a $20 Tang Nano 4K.

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
| `libs/Datapath/` | the vector examples at a stated width (`dp:`): add, saxpy, threshold, the reduction tree, the acceptance pipeline; its demos are the programs the circuits realize |
| `libs/Calc/` | a TMS1000-like four-function calculator as a state vector and a pure step function (`calc:`); the first sequential circuit |
| `components/xir/` | the provisional accelerator IR (typed values with widths and shapes, elementwise maps, select, reductions, later registers and state), its text form and a reference interpreter |
| `components/rtl/` | the netlist, the lanes schedule and the reduction tree, the cycle simulator, the Verilog-2005 emitter |
| `components/cli/` | the `xetal-fpga` tool: `check`, `run`, `netlist`, `verilog`, `sim`, `explain` |
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

Saga 1 (fpga-foundation) in progress: the process, the pinned X_eTaL
(75e6a5c) and X_eTaL-libraries (8adcd0d), the library tooling (step
1), the Fixed library (step 2) and the Datapath library with the
example programs (step 3) are in place. Next: the Calc library (4),
the provisional IR and interpreter (5), the netlist, the schedule, the
simulator and the Verilog emitter (6). See
[`docs/plan.md`](docs/plan.md) for the roadmap and the architecture
decisions.

Nothing here needs an FPGA or the FPGA tools yet: the gate runs the
Verilog through iverilog and Yosys when they are installed and says
so when it skips them.

## Build

Prerequisites:

- [Rust](https://rustup.rs) (stable) and
  [`just`](https://github.com/casey/just)
  (`brew install just` or `cargo install just`)
- optional, for the Verilog checks: `iverilog` (Icarus Verilog),
  `yosys`, `verilator` (`brew install icarus-verilog yosys verilator`)
- optional, for the board: nextpnr-himbaechel with Gowin support,
  Apicula (`gowin_pack`) and openFPGALoader
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
