# Clocked

Clocked circuits as X_eTaL step functions: counters, an accumulator,
a shift register, a sequence detector, a RAM, and the harness that
clocks them. Each step is a pure function, state and input in, next
state out: what becomes registers and the logic between them.

```
"ck:" u_se< "Clocked"
```

| Directory | What |
| --------- | ---- |
| [`src/`](src/) | the library, `Clocked.xtl` |
| [`docs/`](docs/README.md) | the reference: every function, its type, examples, provenance |
| [`demos/`](demos/) | the example programs the sequential circuits realize, each with a hand-lowered `.xir` twin |
| [`tests/`](tests/) | reg-rs baselines: the test programs, the pinned export types (`types.rgt`) and the demos (`demo-*.rgt`) |

```bash
just demo-lib Clocked      # run its demos
just run-lib Clocked       # run its test programs
just test-lib Clocked      # check every baseline
```
