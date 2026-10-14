# Tang Nano 9K board flow

Initial target: GW1NR-LV9QN88PC6/I5, GW1N-9C tool family, 27 MHz.
The onboard USB debugger provides JTAG and UART. Connect a USB data
cable and install openFPGALoader's udev rules for unprivileged access.

`board.json` permits only the clock (52), button reset (3), one
active-low LED (10), UART receive (18) and UART transmit (17).
The constraints guard refuses other pins, including JTAG pins, and
checks that the synthesized top never drives a board input. Top
ports must match the constraints exactly. Additional peripherals
need a reviewed board configuration change first.

Sources: Sipeed's [UART constraints](https://github.com/sipeed/TangNano-9K-example/blob/main/uart/src/top.cst)
and [LED constraints](https://github.com/sipeed/TangNano-9K-example/blob/main/led/src/9K_LED_project.cst).
The LED example uses button pin 4; this configuration uses the UART
example's pin 3 consistently for reset. Physical behavior is pending
validation on a connected board.

From the repository root, with the OSS CAD Suite on PATH:

```bash
just board-detect
target/release/xetal-fpga synth design.v --top top --cst boards/tangnano9k/led.cst --out work/board/synth.json
target/release/xetal-fpga pnr work/board/synth.json --cst boards/tangnano9k/led.cst --out work/board/pnr.json
target/release/xetal-fpga pack work/board/pnr.json --out work/board/design.fs
target/release/xetal-fpga load work/board/design.fs --dry-run
```

`load` programs volatile SRAM. Add `--flash` only for a persistent
write. `--dry-run` validates inputs and prints commands without
running tools. `--board` accepts a board name or an absolute path
to board.json when running outside the repository root.

These commands take a board top, not an arbitrary emitted example:
the top must expose the clock, reset and peripheral ports named in
the constraints. Board adapters and X_eTaL blink/echo examples are
the next deliverable. A temporary Verilog fixture validates the
toolchain only; it is not an X_eTaL equivalence demonstration.

## Independent hardware check

While the machine RTL is being developed:

```bash
just board-toolcheck
just board-detect
just board-flow load work/board/toolcheck/toolcheck.fs --dry-run
just board-flow load work/board/toolcheck/toolcheck.fs
```

The toolcheck fixture counts the board clock onto the active-low LED
(about 1.6 blinks per second), with the button on pin 3 resetting the
counter. It wires UART RX directly to TX. With local echo disabled,
a terminal on the debugger's serial port should receive exactly the
bytes it sends. This checks the bridge and FPGA pin routing; a
clocked UART receiver/transmitter is tested separately by the later
X_eTaL echo. For an automated serial test, turn off terminal local
echo, send a known byte sequence and compare the received sequence.

The build recipe does not program hardware. Load commands above
explicitly program SRAM; power cycling restores the previous flash
design. After physical checks pass, an explicit `load --flash` can
verify persistent configuration. No flash write is part of the
toolcheck recipe.
