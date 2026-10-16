# HEX1000 hardware demo and handoff

The captures below are historical evidence from 2026-10-14, using
the board image built before the memory-backed core update. They do
not validate the newer core now on main or any Nano 1K image. The
interactive CLI reads whichever configuration is currently loaded.
Rebuilding with current main requires fresh hardware validation.

## HEX1000 on actual Nano 9K (2026-10-14)

The complete calculator runs on the attached Tang Nano 9K in SRAM.
The FPGA receives ASCII keys over its built-in USB UART at 115200 8N1
and returns eight display characters followed by carriage return.
The host forwards keys and checks answers; HEX1000 executes the ROM
and computes the results on the FPGA.

| Keys | Actual FPGA display |
| ---- | ------------------- |
| `12+34=` | `46` |
| `A*B=` | `6E` |
| `C0DE+BEEF=` | `17FCD` |

The complete design passed synthesis, placement/routing and packing:
2828 of 8640 LUT4s and 791 of 6480 flip-flops. Timing passed at
27 MHz for UART and 27/28 MHz for the calculator. SRAM loading passed
its CRC check. The calculator core is the emitted, pinned
libs/Hex1000/rtl/hex1000.v; the board adapter handles the key matrix,
R15 handshake, digit strobes, synchronization and registered UART TX.

Repeat with OSS CAD Suite on PATH:

```bash
just board-calc-check   # evaluator references and full serial simulation
just board-calc-build   # build work/board/hex1000/hex1000.fs
just board-calc-demo    # open UART, load SRAM, show three FPGA answers
just board-calc-video   # VHS records the hardware terminal session
```

Recording: work/tapes/hex1000-9k.mp4 (8.12 seconds) and
work/tapes/hex1000-9k.gif. These generated files are gitignored.
The user has copied the MP4 to the blog agent. VHS records the terminal;
it does not emulate the calculator. The initial recording used the
same three requests in a temporary host script; the committed tape
uses the reproducible board-calc-demo recipe.

Known issues and remaining work:

- Reopening/reconfiguring serial left the calculator unresponsive in
  one recording attempt. Reset/reload restored it. The demo opens UART
  before SRAM loading and keeps one connection for all calculations.
  The underlying reconnect behavior still needs investigation.
- The key queue holds 32 characters and silently drops input when full.
  The demo paces keys at 80 ms. Arbitrarily fast or long paste is not
  supported yet; add flow control or an overflow indication.
- Display updates have one pending frame; intermediate screens can be
  replaced during rapid typing. The final screen is the demo's check.
- HEX1000 has been verified in SRAM only. Explicit calculator flash
  programming and calculator boot after unplug/replug remain to test.
  Earlier counter/fixture flash results do not prove calculator boot.
- The interactive CLI is now available. Subtraction, wrap and chaining
  have additional observed hardware results below; broader coverage
  and reset/reconnect behavior still need verification.
- The 9K was the recording target. Mac has since delivered block-memory
  ROM/RAM mapping and a [Nano 1K handoff](handoff-1k.md); its physical
  validation is separate from these recordings. No SD card is used.

Linux owns boards/tangnano9k, scripts/board-hex1000.py and its recipes.
Mac owns the machine/library, browser and site. Mac can use the three
verified results and recording in the blog without waiting for flash
boot, an interactive terminal or Nano 1K optimization.

## Try your own input and inspect the evidence

```bash
just board-calc                  # interactive; current image, no reload
# At HEX1000>: X123*4= then Enter; X clears; q quits.
just board-calc-query 'X5-7='     # one request; print raw received bytes
```

Both commands read the current FPGA configuration without loading a
bitstream. The terminal accepts a line of keys and paces them at 80 ms;
it displays the board's final eight-character screen between vertical
bars. It has no expected-answer lookup or host arithmetic. The query
also prints every received byte, including intermediate displays.
If no display frame arrives, it reports an error; it does not invent
an answer. If this is an older image after power loss, use
`just board-calc-demo` to load the calculator into SRAM first.

A new interactive hardware recording is work/tapes/hex1000-9k-live.mp4
(7.68 seconds), with a GIF alongside it. Unlike the original demo,
this session does not load a bitstream. Its source is
boards/tangnano9k/hex1000-live.tape (`just board-calc-live-video`).
The session returned 48C for X123*4=, FFFFFFFE for X5-7=, and 0 for
XFFFFFFFF+1=. A separate interactive session returned A for X2*3+4=.
Raw received bytes for the recorded session are committed in
[the evidence file](../boards/tangnano9k/evidence/hex1000-live-2026-10-14.json).
Local per-session reports remain in work/board/hex1000.

### Why the LED still blinks

The calculator top explicitly retains a heartbeat LED. In
boards/tangnano9k/hex1000-top.v, `led = ~heartbeat[23]`, and the
button resets both the heartbeat and machine. A blinking LED that
pauses while reset is held therefore also occurs with the calculator
loaded. It cannot identify an image by itself. The no-reload UART
responses above establish that the calculator was running in that
session. A power cycle can still restore the older probe image from
flash, because calculator flash boot has not been verified.

The evidence supports these observed calculations on this board;
it does not establish complete correctness for all inputs. Serial
reader tests additionally check that an arbitrary device response is
returned unchanged and that silence or local input echo cannot supply
a calculator answer. Interactive CLI is now delivered; reconnect,
flow control and calculator flash/power-cycle checks remain open.
