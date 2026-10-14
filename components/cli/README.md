# xetal-fpga-cli

`xetal-fpga`, the FPGA tool of X_eTaL-fpga: one command per question
about a program in the provisional accelerator IR
([`components/xir`](../xir/README.md)).

| Command | What it prints |
| ------- | -------------- |
| `xetal-fpga check FILE.xir` | each value's type, `%c : i16[8]`, or the first error |
| `xetal-fpga run FILE.xir` | the outputs, as `xetal` prints them (the IR interpreter) |
| `xetal-fpga netlist FILE.xir [--lanes N]` | the scheduled netlist, a cell per line |
| `xetal-fpga verilog FILE.xir [--lanes N]` | the circuit as a Verilog-2005 module |
| `xetal-fpga testbench FILE.xir [--lanes N]` | a self-checking testbench for that module |
| `xetal-fpga sim FILE.xir [--lanes N]` | the netlist simulated: the outputs, and the clocks on stderr |
| `xetal-fpga explain FILE.xir [--lanes N]` | the schedule: units by kind and width, registers, clocks |

Without `--lanes` the schedule is spatial, a unit per element; with
fewer lanes than elements it is temporal (see
[`components/rtl`](../rtl/README.md)).

```bash
just build                                       # target/release/xetal-fpga
just xir-run libs/Datapath/demos/vector-add.xir  # 11 22 33 44 0 0 -32768 -5536
```

`synth`, `pnr`, `pack`, `load` and `detect` now run the open Gowin
tools through an embedded Python 3 wrapper. The first board is Nano
9K; see [board commands](../../boards/tangnano9k/README.md). These
commands require a board top with explicit clock/reset/peripheral
ports and reviewed constraints. SRAM is the load default;
`load --flash` writes persistent flash. Each command accepts
`--help` and `--dry-run`. Tool errors fail the command.
