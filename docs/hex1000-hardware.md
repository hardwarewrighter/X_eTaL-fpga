# HEX1000 hardware demo and handoff

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
- Add a persistent interactive terminal, broaden hardware checks to
  subtraction/wrap/chaining, and check reset/reconnect behavior.
- The 9K is the current target. Nano 1K optimization and block-memory
  ROM/RAM mapping are later work. No SD card is used.

Linux owns boards/tangnano9k, scripts/board-hex1000.py and its recipes.
Mac owns the machine/library, browser and site. Mac can use the three
verified results and recording in the blog without waiting for flash
boot, an interactive terminal or Nano 1K optimization.
