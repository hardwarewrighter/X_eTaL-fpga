# Uart

A serial port (UART: 8 data bits, no parity, 1 stop bit) as X_eTaL
step functions: a transmitter, a receiver, and the two looped back.
The clocks per bit are the first argument of every step: the clock
rate over the baud rate, 234 for a 27 MHz Tang Nano at 115200 baud.

```
"ua:" u_se< "Uart"
```

| Directory | What |
| --------- | ---- |
| [`src/`](src/) | the library, `Uart.xtl` |
| [`docs/`](docs/README.md) | the reference: the frame, the states, every function, examples |
| [`demos/`](demos/) | the loopback, and its hand-lowered `.xir` twin |
| [`tests/`](tests/) | reg-rs baselines: the test programs, the pinned export types (`types.rgt`) and the demos (`demo-*.rgt`) |

```bash
just demo-lib Uart         # run its demos
just run-lib Uart          # run its test programs
just test-lib Uart         # check every baseline
```
