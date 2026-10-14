#!/usr/bin/env python3
"""Build and exercise the HEX1000 calculator on Tang Nano 9K."""
import argparse
from contextlib import contextmanager
import fcntl
import json
import os
from pathlib import Path
import select
import shutil
import subprocess
import sys
import termios
import time
import tty

# Reuse board discovery, identity checks and tool logging conventions.
import importlib.util
ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("probe", ROOT / "scripts/board-probe.py")
probe_tools = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe_tools)
first = probe_tools.first
flow = probe_tools.load_script("board-flow")
OUT = ROOT / "work/board/hex1000"
CASES = (("X12+34=", "      46"), ("XA*B=", "      6E"), ("XC0DE+BEEF=", "   17FCD"))


def run(args, label):
    result = subprocess.run(list(map(str, args)), cwd=ROOT, capture_output=True, text=True)
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / f"{label}.log").write_text(result.stdout + result.stderr)
    if result.returncode:
        raise ValueError(f"{label} failed; see {OUT / (label + '.log')}: {result.stderr[-500:]}")
    return result.stdout


def sources():
    run(["just", "build"], "build-cli")
    board = flow.board_config("tangnano9k")
    clocks = round(board["clock_mhz"] * 1_000_000 / board["uart_baud"])
    source = (ROOT / "boards/examples/uart.xir").read_text()
    source = source.replace("%dm1 = const i16 2", f"%dm1 = const i16 {clocks - 1}")
    source = source.replace("%rxk = const i16 3", f"%rxk = const i16 {clocks + clocks // 2 - 1}")
    program = OUT / "uart.xir"
    program.write_text(source)
    serial = run([probe_tools.CLI, "verilog", program], "uart-verilog")
    design = OUT / "design.v"
    design.write_text((ROOT / "boards/tangnano9k/hex1000-top.v").read_text() + "\n" + serial + "\n" +
                      (ROOT / "libs/Hex1000/rtl/hex1000.v").read_text())
    cst = OUT / "design.cst"
    cst.write_text((ROOT / "boards/tangnano9k/uart.cst").read_text() +
                   'IO_LOC "led" 10;\nIO_PORT "led" IO_TYPE=LVCMOS33 PULL_MODE=UP DRIVE=8;\n')
    return design, cst


def check():
    for expression, expected in CASES:
        reference = OUT / "reference.xtl"
        OUT.mkdir(parents=True, exist_ok=True)
        reference.write_text(f'"hx:" u_se< "Hex1000"\nhx:c_alc "{expression}"\n')
        actual = run([ROOT / "scripts/xt", "run", reference], "reference-" + expression[1:-1])
        if actual != expected + "\n":
            raise ValueError(f"calculator reference changed: {expression}: {actual!r}")
    design, cst = sources()
    if shutil.which("iverilog") and shutil.which("vvp"):
        binary = OUT / "board-rtl.vvp"
        run(["iverilog", "-g2005-sv", "-s", "tb", "-o", binary, design,
             ROOT / "boards/tangnano9k/hex1000-top.tb.v"], "board-rtl-compile")
        output = run(["vvp", binary], "board-rtl")
        if "PASS" not in output:
            raise ValueError("HEX1000 board simulation did not pass")
        print("PASS: full board serial simulation matches three pinned X_eTaL calculator results")
    else:
        print("board-hex1000: SKIPPED board simulation (iverilog/vvp missing); references checked")
    return design, cst


def build():
    design, cst = check()
    run([sys.executable, ROOT / "scripts/board-flow.py", "synth", design, "--top", "hex1000_top",
         "--cst", cst, "--out", OUT / "synth.json"], "synth")
    run([sys.executable, ROOT / "scripts/board-flow.py", "pnr", OUT / "synth.json", "--cst", cst,
         "--sdc", ROOT / "boards/tangnano9k/hex1000.sdc", "--out", OUT / "pnr.json"], "pnr")
    run([sys.executable, ROOT / "scripts/board-flow.py", "pack", OUT / "pnr.json",
         "--out", OUT / "hex1000.fs"], "pack")
    print(f"Built {OUT / 'hex1000.fs'} (27 MHz UART clock, 27/28 MHz calculator)")


@contextmanager
def serial_session(port):
    fd = os.open(port, os.O_RDWR | os.O_NOCTTY | os.O_NONBLOCK)
    old = None
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        old = termios.tcgetattr(fd)
        tty.setraw(fd, termios.TCSANOW)
        attrs = termios.tcgetattr(fd)
        attrs[2] = (attrs[2] & ~(termios.PARENB | termios.CSTOPB | termios.CSIZE | termios.CRTSCTS)) | termios.CS8 | termios.CLOCAL | termios.CREAD
        attrs[4] = attrs[5] = termios.B115200
        termios.tcsetattr(fd, termios.TCSANOW, attrs)
        termios.tcflush(fd, termios.TCIOFLUSH)
        yield fd
    finally:
        if old is not None:
            termios.tcsetattr(fd, termios.TCSANOW, old)
        os.close(fd)


def read_until(fd, seconds):
    data = bytearray()
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if select.select([fd], [], [], max(0, deadline - time.monotonic()))[0]:
            block = os.read(fd, 4096)
            if not block:
                raise ValueError("UART disconnected")
            data.extend(block)
    return bytes(data)


def calculate(fd, expression, pace=0.08):
    data = bytearray()
    for character in expression:
        os.write(fd, character.encode("ascii"))
        data.extend(read_until(fd, pace))
    data.extend(read_until(fd, 0.3))
    frames = [frame.decode("ascii") for frame in data.split(b"\r") if len(frame) == 8]
    if not frames:
        raise ValueError(f"No display frame received for {expression!r}; reset/reload after opening serial")
    return frames[-1]


def hardware(stage):
    report = {"stage": stage, "loaded_sram": False, "persistent_flash_written": False, "results": []}
    try:
        probe = first.choose_probe(first.probes())
        report["usb_probe"] = probe
        first.verify_identity(run(["openFPGALoader", "-b", "tangnano9k", "--busdev-num", probe["busdev"], "--detect"], "detect"))
        ports = first.uart_ports(first.serial_ports(probe))
        if len(ports) != 1:
            raise ValueError(f"Expected one interface-B UART, found {ports}")
        report["uart_port"] = ports[0]
        # Open/configure first, then reset by loading: FTDI reconfiguration can
        # inject a false start. Keep this one connection for the entire session.
        with serial_session(ports[0]) as fd:
            if stage != "uart":
                image = OUT / "hex1000.fs"
                if not image.is_file():
                    raise ValueError("Run just board-calc-build first")
                command = ["openFPGALoader", "-b", "tangnano9k", "--busdev-num", probe["busdev"]]
                run(command + (["-f"] if stage == "flash" else []) + [image], "load")
                report["loaded_sram"] = stage != "flash"
                report["persistent_flash_written"] = stage == "flash"
                time.sleep(0.5)
                termios.tcflush(fd, termios.TCIFLUSH)
            print("HEX1000 on Tang Nano 9K | USB UART 115200 baud", flush=True)
            for expression, expected in CASES:
                actual = calculate(fd, expression)
                report["results"].append({"keys": expression, "display": actual, "expected": expected})
                print(f"{expression[1:]}  ->  {actual.strip()}", flush=True)
                if actual != expected:
                    raise ValueError(f"FPGA display {actual!r}, expected {expected!r}")
            print("PASS: all three results received from the FPGA.", flush=True)
    except (OSError, ValueError) as error:
        report["error"] = str(error)
        raise
    finally:
        OUT.mkdir(parents=True, exist_ok=True)
        (OUT / f"{stage}-report.json").write_text(json.dumps(report, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("check", "build", "test", "demo", "flash", "uart"))
    args = parser.parse_args()
    if args.stage == "check":
        check()
    elif args.stage == "build":
        build()
    else:
        hardware(args.stage)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError) as error:
        print(f"board-hex1000: {error}", file=sys.stderr)
        sys.exit(1)
