# Datapath

The vector examples at a stated width: the whole-array operations a
circuit realizes directly (adders per element, a mux per element, a
tree of adders), on Fixed's arithmetic so the evaluator's answer is
the circuit's.

```
"dp:" u_se< "Datapath"
```

| Directory | What |
| --------- | ---- |
| [`src/`](src/) | the library, `Datapath.xtl` |
| [`docs/`](docs/README.md) | the reference: every function, its type, examples, provenance |
| [`demos/`](demos/) | the example programs the circuits realize: vector-add, saxpy, threshold, reduce-sum, pipeline |
| [`tests/`](tests/) | reg-rs baselines: the test programs, the pinned export types (`types.rgt`) and the demos (`demo-*.rgt`, the golden reference for the circuits) |

```bash
just demo-lib Datapath     # run its demos
just run-lib Datapath      # run its test programs
just test-lib Datapath     # check every baseline
```
