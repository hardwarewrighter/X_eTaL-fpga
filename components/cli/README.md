# xetal-fpga-cli

`xetal-fpga`, the FPGA tool of X_eTaL-fpga: one command per question
about a program in the provisional accelerator IR
([`components/xir`](../xir/README.md)).

| Command | What it prints |
| ------- | -------------- |
| `xetal-fpga check FILE.xir` | each value's type, `%c : i16[8]`, or the first error |
| `xetal-fpga run FILE.xir` | the outputs, as `xetal` prints them |

```bash
just build                                       # target/release/xetal-fpga
just xir-run libs/Datapath/demos/vector-add.xir  # 11 22 33 44 0 0 -32768 -5536
```

The netlist, the Verilog, the simulation and the explanation of a
schedule (`netlist`, `verilog`, `sim`, `explain`, with `--lanes N`)
come with the RTL component.
