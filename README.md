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
openFPGALoader) to Sipeed's Tang FPGA boards, from the Tang Nano 1K
up (see [Boards](#boards)).

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
| `libs/Hex1000/` | HEX1000 itself (`hx:`): the machine's state as a tuple, one instruction as a pure step, a run harness and an assembler |
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

The X_eTaL evaluator, XIR interpreter and netlist simulator agree on
all existing circuit examples. All 18 pinned Verilog circuits pass
Icarus simulation, Verilator lint and Yosys synthesis. HEX1000 runs
in software and on the attached Tang Nano 9K over USB UART.

### Actual Nano 9K results (2026-10-14)

| Check | Result on the attached board |
| ----- | ---------------------------- |
| USB/JTAG | passed; Gowin 9C identified as 0x0100481b |
| Synthesis, placement/routing, packing | passed with the Linux OSS CAD Suite |
| SRAM programming | passed; loader CRC check succeeded |
| USB-UART | 542 bytes returned exactly at 115200 8N1, including every byte value |
| Clock and LED | blinking observed by the user |
| Button reset | holding a button pauses the blink, confirmed by the user |
| Persistent flash programming | passed; flash write completed with CRC success |
| Boot after power loss | passed; user unplugged/replugged, LED/reset resumed, 541 UART bytes passed without loading a bitstream |
| X_eTaL on the board | passed; 512 counter queries match the evaluator through two full wraps, plus 255 exact UART echoes |
| HEX1000 on the board | passed in SRAM over UART: 12+34= gives 46, A*B= gives 6E, C0DE+BEEF= gives 17FCD |

Repeat the initial checks with `just board-prepare`, `just board-check`
and `just board-test`. The fixture does not use the inserted SD card.
[First-board instructions](boards/tangnano9k/README.md) and
[hardware verification](docs/hardware.md) describe the tests. The
initial target is Nano 9K; Nano 1K sizing is deferred.

Actual calculator results, repeatable commands, recording and known limits:
[HEX1000 hardware handoff](docs/hex1000-hardware.md).

## HEX1000

The machine, ROM and emitted core, with each stage checked against
the earlier one. See [project goals](docs/goals.md).

| File | What |
| ---- | ---- |
| [`docs/machine.md`](docs/machine.md) | the machine's definition: registers, memories, instruction set, timing, I/O |
| [`libs/Hex1000/src/Hex1000.xtl`](libs/Hex1000/src/Hex1000.xtl) | the machine in X_eTaL, the executable specification: one instruction as a pure step, the assembler, the run harness ([reference](libs/Hex1000/docs/README.md)) |
| [`libs/Hex1000/rom/calc.asm`](libs/Hex1000/rom/calc.asm) | the calculator program in HEX1000's instructions ([how it works](libs/Hex1000/docs/calc.md)) |
| [`libs/Hex1000/rom/calc.hex`](libs/Hex1000/rom/calc.hex) | that program assembled: the 1024-byte ROM image, for `$readmemh` (`scripts/rom-hex.sh`) |
| [`libs/Radix/src/Radix.xtl`](libs/Radix/src/Radix.xtl) | the calculator's specification: the screens the program must show ([`libs/Hex1000/tests/calc.xtl`](libs/Hex1000/tests/calc.xtl) checks 33 key sequences) |
| [`libs/Hex1000/demos/circuit.xtl`](libs/Hex1000/demos/circuit.xtl) | the machine at its ports: K, O and the R latches at every clock of a calculation, what the circuit must reproduce |
| [`scripts/gen-hex1000-xir.py`](scripts/gen-hex1000-xir.py) | the hand lowering: one rule per instruction |
| [`libs/Hex1000/demos/hex1000.xir`](libs/Hex1000/demos/hex1000.xir) | the circuit in the intermediate form (generated; the gate checks it is current) |
| [`libs/Hex1000/rtl/hex1000.v`](libs/Hex1000/rtl/hex1000.v) | the circuit in Verilog-2005, module `hex1000`: the core a board top wraps (generated and pinned; iverilog, Verilator and Yosys in the gate) |
| [`libs/Hex1000/demos/terminal.xtl`](libs/Hex1000/demos/terminal.xtl) | the calculator at a terminal (`just calc`); [`web/hex1000`](web/hex1000) the same in a browser ([live](https://hardwarewrighter.github.io/X_eTaL-fpga/calc/)) |
| [`docs/parallel-work.md`](docs/parallel-work.md) | the core's ports and the UART glue a board needs |

```bash
just demo-lib Hex1000 calc        # calculations typed at the machine in X_eTaL
just equiv                        # the IR and the simulated circuit against X_eTaL
just rtl                          # the Verilog under iverilog, Verilator, Yosys
just fit hex1000                  # its size on a Gowin GW1N (Yosys)
```


## Boards

The target is Sipeed's family of Gowin FPGA boards, all of which the
user has. The board is schedule data (a constraints file per board:
its clock, pins, serial lines), never part of a program, so a design
stays buildable for several boards and goes to the smallest that
holds it. [`docs/boards.md`](docs/boards.md) has each board's family,
resources, processors and the software it needs, with sources.

| Board | Family | LUT4 | Processor besides the fabric | Open flow | Used for |
| ----- | ------ | ---- | ---------------------------- | --------- | -------- |
| Tang Nano 1K | GW1N (GW1NZ) | 1,152 | none | yes | HEX1000 optimization later |
| Tang Nano 4K | GW1N (GW1NSR) | 4,608 | a hard ARM Cortex-M3 in the FPGA | yes (the fabric) | HEX1000 alternative; lane counts |
| Tang Nano 9K | GW1N (GW1NR) | 8,640 | none | yes | HEX1000 now; the CDP1802; the neural layer |
| Tang Nano 20K | GW2A (GW2AR) | 20,736 | a BL616 microcontroller as programmer | yes | the IBM 1130, larger designs |
| Tang Primer 20K | GW2A | 20,736 | a BL702 on its Dock, as programmer | yes | as the 20K, with DDR3 |
| Tang Primer 25K | GW5A | 23,040 | a BL616 as programmer | not yet: Gowin IDE V1.9.9 Beta-4 or newer | the largest; Gowin IDE only for now |

HEX1000 currently runs on the 9K; optimization for the 1K is later.
Which other Tang
Primer boards the user has is still to record.

## Build

Prerequisites:

- [Rust](https://rustup.rs) (stable) and
  [`just`](https://github.com/casey/just)
  (`brew install just` or `cargo install just`)
- optional, for the Verilog checks: `iverilog` (Icarus Verilog),
  `yosys`, `verilator`, and `gtkwave` to look at waveforms (macOS:
  `brew install icarus-verilog yosys verilator`; Arch: `pacman -S
  iverilog yosys verilator gtkwave`)
- optional, for the boards (on the Arch machine, where the boards
  are): nextpnr with the Gowin target (`nextpnr-himbaechel`), Apicula
  (`gowin_pack`) and openFPGALoader; see [Boards](#boards) for which
  board does what
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
just demo-lib Hex1000 counter                      # HEX1000 counting on its outputs, every register traced
just demo-lib Hex1000 keys                         # HEX1000 reading keys and storing them in RAM
just demo-lib Hex1000 calc                         # HEX1000 calculating: screens, and instructions taken
just demo-lib Hex1000 listing                      # its calculator program, page by page in the ROM
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

## Documentation and the live site

Every library documents itself in X_eTaL's doc comments: a `##` block
above each definition, `###` section headings, and for every export a
`## >>` example with its output, which `just doctest` runs.
`xetal doc` turns them into a browsable site with a search, the
built-ins and each source file:

```bash
just doc                  # the documentation into pages/doc
just pages                # the whole site into pages/ (not tracked): landing page, documentation, Verilog
just serve-pages          # look at it: http://127.0.0.1:8417/
just publish              # make pages/ the gh-pages branch's only commit (maintainers)
```

The site is published at
<https://hardwarewrighter.github.io/X_eTaL-fpga/> once GitHub Pages is
enabled for the repository (it is private for now).

## Documents

- [`docs/plan.md`](docs/plan.md) -- architecture decisions, the
  subset, the roadmap
- [`docs/hardware.md`](docs/hardware.md) -- Verilog first, the
  checks from the evaluator to the bitstream, testing on a board,
  bricking risks
- [`docs/linux-nvidia.md`](docs/linux-nvidia.md) -- setting up on
  the Arch Linux machine: the tools, USB access to the boards, the
  rebuild
- [`docs/boards.md`](docs/boards.md) -- the Tang boards: families,
  resources, processors, software
- [`docs/tuple-plan.md`](docs/tuple-plan.md) -- where tuples are used,
  how they map to registers, the gaps and who fixes them
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

Hardware bring-up starts on the Nano 9K with onboard USB-JTAG and
USB-UART; Nano 1K sizing comes later. Development ownership is in
[parallel work](docs/parallel-work.md).

The X_eTaL hardware proof uses `just board-probe-build` and
`just board-probe-test`; `just board-probe-flash` explicitly writes
flash. Its UART and counter are emitted from checked XIR twins of
ordinary X_eTaL programs. [Proof and recording](boards/examples/README.md).
