# Radix

An eight-digit calculator in any radix (hexadecimal, octal, decimal):
the specification HEX1000's ROM program must meet, key for key.

```
"rx:" u_se< "Radix"
```

| Directory | What |
| --------- | ---- |
| [`src/`](src/) | the library, `Radix.xtl` |
| [`docs/`](docs/README.md) | the reference: numbers, keys, the calculator's rules, every function, examples |
| [`demos/`](demos/) | a hexadecimal session; one key sequence in three radixes |
| [`tests/`](tests/) | reg-rs baselines: the test programs, the pinned export types (`types.rgt`) and the demos (`demo-*.rgt`) |

```bash
just demo-lib Radix        # run its demos
just run-lib Radix         # run its test programs
just test-lib Radix        # check every baseline
```
