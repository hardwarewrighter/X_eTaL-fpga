# Fixed

Fixed-width two's complement numbers at a stated width: the
arithmetic a w-bit circuit does, computed by the evaluator so it can
be the reference for the circuit.

```
"fx:" u_se< "Fixed"
```

| Directory | What |
| --------- | ---- |
| [`src/`](src/) | the library, `Fixed.xtl` |
| [`docs/`](docs/README.md) | the reference: every function, its type, examples, provenance |
| [`demos/`](demos/) | programs that use it (an 8-bit overflow, a 16-bit counter wrapping) |
| [`tests/`](tests/) | reg-rs baselines: the test programs, the pinned export types (`types.rgt`) and the demos (`demo-*.rgt`) |

```bash
just demo-lib Fixed        # run its demos
just run-lib Fixed         # run its test programs
just test-lib Fixed        # check every baseline
```
