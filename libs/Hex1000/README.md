# Hex1000

HEX1000, a 4-bit machine in the spirit of the TMS1000, in X_eTaL: its
state as a tuple, one instruction as a pure step, a run harness, an
assembler, and its ROM program, an eight-digit hexadecimal calculator ([`docs/machine.md`](../../docs/machine.md)).

```
"hx:" u_se< "Hex1000"
```

| Directory | What |
| --------- | ---- |
| [`src/`](src/) | the library, `Hex1000.xtl` |
| [`rom/`](rom/) | ROM programs in HEX1000's instructions: `calc.asm`, the calculator |
| [`docs/`](docs/README.md) | the reference: the state, the assembler, every function, examples; [`calc.md`](docs/calc.md), the calculator program |
| [`demos/`](demos/) | a counter traced; keys read and stored; the calculator's screens; its ROM listing |
| [`tests/`](tests/) | reg-rs baselines: every instruction, small programs, assembler errors, the pinned export types (`types.rgt`) and the demos (`demo-*.rgt`) |

```bash
just demo-lib Hex1000      # run its demos
just run-lib Hex1000       # run its test programs
just test-lib Hex1000      # check every baseline
```
