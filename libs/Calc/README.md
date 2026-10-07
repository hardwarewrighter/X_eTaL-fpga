# Calc

A TMS1000-like four-function calculator as a state vector and a pure
step function: registers, a keyboard, a decoder, an ALU and a display,
every number 16-bit two's complement, written as ordinary X_eTaL so
the evaluator can run it and a circuit can be generated from it.

```
"calc:" u_se< "Calc"
```

| Directory | What |
| --------- | ---- |
| [`src/`](src/) | the library, `Calc.xtl` |
| [`docs/`](docs/README.md) | the reference: the state layout, the key codes, every function, examples |
| [`demos/`](demos/) | programs that use it (a session; the registers traced key by key) |
| [`tests/`](tests/) | reg-rs baselines: the test programs, the pinned export types (`types.rgt`) and the demos (`demo-*.rgt`) |

```bash
just demo-lib Calc         # run its demos
just run-lib Calc          # run its test programs
just test-lib Calc         # check every baseline
```
