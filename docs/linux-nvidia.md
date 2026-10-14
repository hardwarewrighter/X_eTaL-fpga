# Setting up on Arch Linux (the NVIDIA machine) with the Tang boards

How to clone and rebuild this repository on the Arch Linux machine
(`large12`, the one with the RTX 5060 Ti that X_eTaL-gpu measures on),
install the open FPGA flow, and reach the Tang boards over USB. Written
2026-10-14 on the Mac before the move; the package names are as Arch
names them to my knowledge, so confirm each with `pacman -Ss NAME`
the first time, and fix this page where it is wrong.

## The GPU is not needed here

Nothing in this repository uses the GPU: the simulators run on the
CPU and the FPGA flow is CPU-only. The NVIDIA driver and CUDA/OpenCL
are already installed on this machine for X_eTaL-gpu
(`../../softwarewrighter/X_eTaL-gpu`, its `docs/rtx5060-results.md`).
Leave them as they are; nothing below installs, upgrades or removes
them. The two repositories share only the machine and the pinned
X_eTaL.

## 1. The directory layout

The scripts find the sibling repositories by relative path
(`../../softwarewrighter/...`), so keep the Mac's layout:

```
~/github/softwarewrighter/X_eTaL              # the language (pins move from here)
~/github/softwarewrighter/X_eTaL-libraries    # Check, Bits, Strings
~/github/hardwarewrighter/X_eTaL-fpga         # this repository
```

```bash
mkdir -p ~/github/softwarewrighter ~/github/hardwarewrighter
cd ~/github/softwarewrighter
git clone git@github.com:softwarewrighter/X_eTaL.git             # if not already there for X_eTaL-gpu
git clone git@github.com:softwarewrighter/X_eTaL-libraries.git
cd ~/github/hardwarewrighter
git clone git@github.com:hardwarewrighter/X_eTaL-fpga.git
```

## 2. The build tools

```bash
sudo pacman -S --needed base-devel git rustup just python github-cli
rustup default stable
rustup component add rustfmt clippy
```

The gate also uses three of the user's own tools from the
softwarewrighter organization: `reg-rs` (the baselines),
`sw-markdown-checker` (the docs) and `agentrail` (the sagas). On the
Mac they are in `~/.local/softwarewrighter/bin`. Install them the
usual way (with `sw-install`, run by the user; the agents never run
it) and put that directory on `PATH`.

## 3. The Verilog and FPGA tools

Two ways. Take one; do not mix them on `PATH`.

**A. The OSS CAD Suite (recommended for the FPGA flow).** YosysHQ's
nightly bundle has a matched set: Yosys, nextpnr-himbaechel with
Gowin support, Apicula's `gowin_pack`, openFPGALoader, Icarus
Verilog, Verilator, GTKWave. Download the `linux-x64` archive from
<https://github.com/YosysHQ/oss-cad-suite-build/releases>, unpack it
into `~/opt/oss-cad-suite`, and add to your shell profile:

```bash
export PATH="$HOME/opt/oss-cad-suite/bin:$PATH"
```

**B. Arch packages.** The simulators and Yosys come from the official
repositories; nextpnr with the Gowin (himbaechel) architecture and
Apicula may not, so check first:

```bash
sudo pacman -S --needed iverilog verilator yosys gtkwave openfpgaloader
pacman -Ss nextpnr                       # official? else the AUR (nextpnr-git) or the OSS CAD Suite
pipx install apycula                     # Apicula: gowin_pack, gowin_unpack (sudo pacman -S python-pipx)
```

Check what you got:

```bash
iverilog -V | head -1; verilator --version; yosys -V
nextpnr-himbaechel --help | head -1
gowin_pack -h | head -1
openFPGALoader --Version
```

The Mac has Icarus Verilog 13.0, Verilator 5.052 and Yosys 0.70 (from
Homebrew); a similar or newer version is fine.

## 4. USB access to the boards

The Tang boards' USB programmers appear to Linux as an FTDI-style
JTAG adapter plus, on most boards, a serial port. Without udev rules
only root can open them.

```bash
lsusb                                    # plug a board in: a Sipeed / FTDI-style debugger appears
openFPGALoader --scan-usb                # what openFPGALoader sees
```

- **JTAG**: install openFPGALoader's udev rules if the package did
  not (`ls /usr/lib/udev/rules.d/ /etc/udev/rules.d/ | grep -i fpga`;
  the OSS CAD Suite has a copy under `share/`). Copy the file to
  `/etc/udev/rules.d/`, then `sudo udevadm control --reload-rules &&
  sudo udevadm trigger`, and unplug and replug the board.
- **Serial** (the board's own UART bridge, or the USB-serial adapter
  the Tang Nano 1K needs): Arch gives `/dev/ttyUSB*` and `/dev/ttyACM*`
  to the `uucp` group: `sudo usermod -aG uucp $USER`, then log out and
  in.
- The adapter for the 1K must be **3.3 V TTL** (`docs/hardware.md`).
- A terminal for the HEX1000: `picocom -b 115200 /dev/ttyUSB0`
  (`sudo pacman -S picocom`); the baud rate is the design's, set when
  the board flow exists.

A first check that touches nothing on the board:

```bash
openFPGALoader -b tangnano1k --detect     # or tangnano4k, tangnano9k, tangnano20k
```

It reads the FPGA's ID code over JTAG. Loading a design comes later
(saga fpga-board), into SRAM by default, which a power cycle undoes
(`docs/hardware.md`, Bricking).

## 5. Rebuild this repository

```bash
cd ~/github/hardwarewrighter/X_eTaL-fpga
just xetal            # clone X_eTaL at XETAL_COMMIT into work/xetal, build bin/xetal
just libs-get         # X_eTaL-libraries at XETAL_LIBRARIES_COMMIT into work/libs
just build            # the xetal-fpga tool (release)
just gate             # everything: baselines, doc tests, components, equivalence, Verilog tools
just fit              # each pinned circuit on a Gowin GW1N against the Tang Nano 1K
```

`just xetal` clones X_eTaL from GitHub; to clone from the local
checkout instead, `XETAL_SOURCE=../../softwarewrighter/X_eTaL just
xetal`. The gate must say `gate: ok` and, with the tools installed,
`17 simulated by iverilog, 17 linted by verilator, 17 synthesized by
yosys`, the same as on the Mac. A difference between the two machines
is a finding: note it in this page (or in `docs/xetal-asks.md` if it
is X_eTaL's).

Work happens on `main` on one machine at a time: `git pull` before
starting, push before switching back.

## 6. Then

When the gate passes here and `--detect` finds a board, the board
flow (saga fpga-board, step flow) can start: the `xetal-fpga synth`,
`pnr`, `pack` and `load` wrappers, a constraints file per board, a
blink, then the UART echo.

## Linux validation (2026-10-14)

The installed stable Rust was 1.88, which cannot compile the pinned
X_eTaL's if-let guards. `rustup update stable` upgraded this machine
to 1.99. No dependency pins or vendored source were changed.

The OSS CAD Suite 20261014 linux-x64 archive is extracted at
`/disk1/tmp/oss-cad-suite`. Activate it for this shell:

```bash
export PATH="/disk1/tmp/oss-cad-suite/bin:$PATH"
```

This path is local machine setup, not a portable repository default.
Yosys reports 0.70+16 and openFPGALoader 1.1.1. A temporary 24-bit
counter fixture passed synthesis, placement/routing and packing for
Nano 9K. The tool family is `GW1N-9C`, although the FPGA device
name starts with GW1NR. No board was connected and no bitstream
was loaded. The fixture proves tool operation, not X_eTaL equivalence.

The suite's nextpnr launcher writes font configuration under the
user's home directory; running it in a filesystem sandbox can require
permission to write there. The suite's Python packer reports missing
optional acceleration modules; packing still succeeds.

See [Nano 9K commands](../boards/tangnano9k/README.md). Board tops
for X_eTaL blink and UART echo are the next hardware deliverable.

USB scan succeeds outside the execution sandbox but finds no FPGA
programmer. The initial sandbox reported incomplete group membership; outside
the sandbox, the account is already in uucp. Board-specific USB
access still needs the rule below.


## First attached Nano 9K (2026-10-14)

The user connected a Nano 9K with an SD card inserted. The debugger
appears as 0403:6010, SIPEED JTAG Debugger, with interface A and B
serial ports. Existing uaccess rules did not grant this session USB
write access. The user installed boards/tangnano9k/70-xetal-tang9k.rules
and reloaded/triggered udev; the current uucp membership was sufficient.

`just board-check` reads ID 0x0100481b, Gowin GW1N(R)-9C. The first
`just board-test` loaded the simulated fixture into SRAM with a
successful CRC and returned exactly 542 bytes at 115200 8N1 on the
interface-B UART, /dev/ttyUSB1. Every possible byte value was tested.
Flash and the SD card were not used. The user confirmed LED blinking and the button pausing it while held.
Power-cycle recovery remains unchecked.

`just board-prepare` simulates and builds the fixture. `just board-check`
is read-only. `just board-test` selects a matching USB probe, loads
SRAM, matches its UART and checks exact bytes with local echo disabled.
Diagnostics are saved to work/board/first-board.json. No pyserial or
other Python dependency is needed.
