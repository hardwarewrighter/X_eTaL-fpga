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

## First connection on Linux

```bash
export PATH="/disk1/tmp/oss-cad-suite/bin:$PATH"  # this machine's installation
just board-prepare                           # simulate and build; does not use USB
just board-check                             # read-only JTAG detection and serial listing
just board-test                              # load SRAM and compare UART bytes
```

The diagnostic selects one FT2232-compatible USB probe and the UART
on interface B. With multiple probes, use `--busdev BUS:DEV`; an
explicit `--port /dev/serial/by-id/...` must belong to that probe.
Close terminals using that serial port before the automated test.
It sends 542 bytes at 115200 8N1, including every possible byte value,
with terminal echo and newline conversion disabled. It fails on
missing, changed or extra bytes. Python's standard library handles
serial I/O; no pyserial package is needed.

Results are written to `work/board/first-board.json`: USB identity,
JTAG/load output, serial port and exact-byte result.
The last result for each stage is also saved as `first-board-test.json`,
`first-board-check.json` or `first-board-prepare.json` so a later
read-only check does not replace the test evidence. Observe the
LED and button yourself; those checks are never marked passed by
software. An inserted SD card is not used by the fixture.

If USB access is denied, the Arch rule supplied here grants access
to the existing uucp group:

```bash
sudo install -m 0644 boards/tangnano9k/70-xetal-tang9k.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
```

Unplug and replug the board, then rerun `just board-check`. If your
account is not in uucp, run `sudo usermod -aG uucp "$USER"` and log
out and back in. The setup does not update debugger firmware or
write FPGA flash. Persistent configuration is a later explicit test.

First hardware run on 2026-10-14 passed JTAG identification, SRAM
load/CRC and all 542 UART bytes. The user confirmed LED blinking
and the button pausing it while held. Power-cycle recovery is unchecked.


## Persistent configuration verified

On 2026-10-14 the user authorized a persistent flash test. The verified
fixture was written with openFPGALoader -f; writing completed with
CRC success. The user unplugged and replugged the board and confirmed
LED blinking and button reset resumed. A 541-byte UART test, including
every byte value, passed afterward without any bitstream load.
This proves the fixture boots from flash after power loss. The SD
card was not used. Raw evidence is in work/board/first-flash.log and
work/board/power-cycle.json; the durable result is recorded here.
