# The boards

Sipeed's Tang boards, all built on Gowin FPGAs, which the user has:
their families, what is on them, what software they need, and what
each is for here. Gathered 2026-10-12 from Sipeed's wiki pages for
each board and from Project Apicula's README and issue tracker (the
sources are listed at the end). Where a figure was not on those pages
it is marked "to check"; check a board's own page before relying on
a number for a design.

## Current board status (2026-10-14)

On the attached Nano 9K, JTAG identification, SRAM load with CRC,
542-byte exact UART loopback at 115200, LED blink and button reset
are verified. Persistent flash programming also passed with CRC;
after the user unplugged/replugged, LED/reset resumed and 541 UART
bytes passed without a bitstream load. The USB rule is installed.
The fixture proves the tools and board connection; X_eTaL board
execution and HEX1000 hardware are still pending. The inserted SD
card is not used. See [first-board setup](../boards/tangnano9k/README.md).

## Families

Gowin's FPGAs come in families, and the family decides both the
resources and the software:

| Family | Gowin's name | Boards here | Open flow (Yosys, nextpnr-himbaechel, Apicula) | Gowin IDE |
| ------ | ------------ | ----------- | ---------------------------------------------- | --------- |
| GW1N (GW1NZ, GW1NSR, GW1NR) | LittleBee | Tang Nano 1K, 4K, 9K | supported: Apicula lists all three boards | any current version |
| GW2A (GW2AR, GW2A) | Arora | Tang Nano 20K, Tang Primer 20K | supported: Apicula lists both; the device needs nextpnr's `--vopt family` and the family name for `gowin_pack` | any current version |
| GW5A | Arora V | Tang Primer 25K (and the Tang Mega boards, not here) | **in progress, not supported yet**: Apicula's README does not list any GW5A board; its "GW5A family support" issue has been open since 2023-10, a GW5A-25 PLL issue since 2025-12, and the first GW5A flags were merged 2026-10-08 (Apicula 0.34, 2026-10-02, is the latest release) | **V1.9.9 Beta-4 or newer** is required for the 25K (Sipeed) |

So the GW1N and GW2A boards can be built entirely with the open
tools this repository plans to drive (plan A12); the Tang Primer 25K
needs Gowin's own IDE until the open flow covers the GW5A, and is
re-checked when a design first goes to it.

## The boards

| Board | FPGA | LUT4 | Flip-flops | Block SRAM | Multipliers | Memory beside the FPGA | Clock | Programming and serial |
| ----- | ---- | ---- | ---------- | ---------- | ----------- | ---------------------- | ----- | ---------------------- |
| Tang Nano 1K | GW1NZ-LV1QN48C6/I5 | 1,152 | 864 | 72 Kbit (4 blocks) | none listed | 64 Kbit user flash | 27 MHz | USB-JTAG; **no USB serial port listed**: a USB-serial adapter on two pins (the user has them) |
| Tang Nano 4K | GW1NSR-LV4CQN48PC7/I6 | 4,608 | 3,456 | 180 Kbit | 16 | 64 Mbit PSRAM in the package; 32 Mbit flash | 27 MHz | BL702 for JTAG (serial: to check) |
| Tang Nano 9K | GW1NR-LV9QN88PC6/I5 | 8,640 | 6,480 | 486 Kbit (plus 17,280 bits of shadow SRAM) | to check | 64 Mbit PSRAM, 608 Kbit user flash, 32 Mbit flash | 27 MHz | USB-JTAG and USB-UART |
| Tang Nano 20K | GW2AR-LV18QN88C8/I7 | 20,736 | 15,552 | 828 Kbit (46 blocks, plus 41,472 bits of shadow SRAM) | 48 (18 x 18) | 64 Mbit 32-bit SDR SDRAM | 27 MHz (to check) | BL616 microcontroller: bitstream loading and serial over USB-C |
| Tang Primer 20K | GW2A-LV18PG256C8/I7 | 20,736 | 15,552 | 828 Kbit (46 blocks) | 48 (18 x 18) | 128 MB DDR3 | to check | JTAG and UART; on the Dock board a BL702 for bitstreams and serial |
| Tang Primer 25K | GW5A-LV25MG121 | 23,040 | 23,040 | 1,008 Kbit (56 blocks, plus 180 Kbit of shadow SRAM) | 28 (18 x 18) | 64 Mbit NOR flash; SDRAM by an add-on module | 50 MHz | BL616 over USB-C: JTAG and UART |

Which other Tang Primer boards the user has is **to record** here
(for example a Primer 20K on its Dock, or a Primer 15K).

## Processors on the boards

Some boards carry a processor besides the FPGA fabric. None is needed
by this repository's designs, which are built from X_eTaL programs in
the fabric; they matter for how a board is programmed and talked to,
and as later options.

| Board | Processor | What it is | For this repository |
| ----- | --------- | ---------- | ------------------- |
| Tang Nano 4K | **ARM Cortex-M3, a hard core in the FPGA** (up to 80 MHz) | a real CPU inside the GW1NSR-4C, used through Gowin's own tools (the IDE's embedded-processor flow and Gowin's MCU software) | not used: the open flow supports the 4K's fabric, not (to check) its Cortex-M3. A later option: a host program on the M3 talking to an X_eTaL circuit in the fabric |
| Tang Nano 20K, Tang Primer 25K | **BL616**, a RISC-V microcontroller on the board, outside the FPGA | Sipeed's debugger: loads bitstreams and bridges a serial port over USB | used only as the programmer and serial port (openFPGALoader talks to it) |
| Tang Nano 4K, Tang Primer 20K Dock | **BL702**, a RISC-V microcontroller on the board | the same role on these boards | the same |
| Tang Nano 9K | none (a soft RISC-V, PicoRV32, fits in its fabric) | | a soft core is a design like any other; not planned |
| Tang Nano 1K | none | | |

## What each board is for here

| Board | Used for |
| ----- | -------- |
| Tang Nano 1K | HEX1000 first (decided: the smallest project on the smallest board); its block RAM holds the 8 Kbit program ROM; serial through an adapter |
| Tang Nano 4K | HEX1000 if the 1K is too small; the vector datapath's lane counts measured |
| Tang Nano 9K | the RCA CDP1802; the tiny neural layer; its USB-UART needs no adapter |
| Tang Nano 20K | the IBM 1130 and other larger designs in the open flow |
| Tang Primer 20K | as the Nano 20K, with DDR3 when a design needs large memory |
| Tang Primer 25K | the largest here, but **Gowin IDE only** until the open flow supports the GW5A |

A design goes to the smallest board that holds it (plan A12), and a
board's support in the open flow is checked when a design first goes
to it, with the result recorded in this page.

## Sources

- Sipeed, Tang boards overview: https://wiki.sipeed.com/hardware/en/tang/common-doc/summary.html
- Sipeed, Tang Nano 1K: https://wiki.sipeed.com/hardware/en/tang/Tang-Nano-1K/Nano-1k.html
- Sipeed, Tang Nano 4K: https://wiki.sipeed.com/hardware/en/tang/Tang-Nano-4K/Nano-4K.html
- Sipeed, Tang Primer 20K: https://wiki.sipeed.com/hardware/en/tang/tang-primer-20k/primer-20k.html
- Sipeed, Tang Primer 25K: https://en.wiki.sipeed.com/hardware/en/tang/tang-primer-25k/primer-25k.html
- CNX Software on the Tang Nano 9K, 20K and Primer 25K: https://www.cnx-software.com/2022/01/17/tang-nano-9k-fpga-board-can-emulate-picorv32-risc-v-soft-core-with-all-peripherals/, https://www.cnx-software.com/2024/01/03/sipeed-tang-primer-25k-board-23040-logic-cells-fpga-prototyping-development/
- Project Apicula (supported boards; GW5A issues and releases): https://github.com/YosysHQ/apicula

## Initial hardware target (2026-10-14)

The user deferred 1K sizing and selected Nano 9K bring-up first for
onboard USB programming and UART. The Linux toolchain has built a
counter fixture to a bitstream; physical testing is pending. Board
configuration and commands are in [Nano 9K](../boards/tangnano9k/README.md).


The first connected 9K now passes JTAG detection (0x0100481b), SRAM
loading with CRC success and an exact 542-byte USB-UART loopback at
115200 baud. The user installed the supplied USB rule. The SD card
is not used by this fixture; flash configuration is unchanged.

The user also confirmed LED blinking and the button pausing it while
held. This verifies clock/LED/reset alongside the programming and UART
checks. Power-cycle recovery remains unchecked.
