#!/usr/bin/env python3
"""Prepare, detect and test a Tang Nano 9K using the SRAM fixture."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import re
import select
import shutil
import subprocess
import sys
import termios
import time
import tty

ROOT = Path(__file__).resolve().parent.parent
FIXTURE = ROOT / "work/board/toolcheck/toolcheck.fs"
REPORT = ROOT / "work/board/first-board.json"
TOOLS = ("yosys", "nextpnr-himbaechel", "gowin_pack", "openFPGALoader", "iverilog", "vvp")


def read(path):
    try:
        return path.read_text().strip()
    except OSError:
        return ""


def probes(usb_root=Path("/sys/bus/usb/devices")):
    found = []
    for path in usb_root.iterdir():
        if read(path / "idVendor") != "0403" or read(path / "idProduct") != "6010":
            continue
        bus, device = read(path / "busnum"), read(path / "devnum")
        if not bus or not device:
            continue
        found.append({"path": str(path.resolve()), "busdev": f"{int(bus)}:{int(device)}",
                      "usb_node": f"/dev/bus/usb/{int(bus):03}/{int(device):03}",
                      "serial": read(path / "serial"), "product": read(path / "product")})
    return found


def choose_probe(found, busdev=None):
    matches = [p for p in found if busdev is None or p["busdev"] == busdev]
    if len(matches) != 1:
        raise ValueError("Connect one Nano 9K; if multiple FTDI probes are attached, select --busdev BUS:DEV")
    return matches[0]


def verify_identity(output):
    match = re.search(r"\bidcode\s+0x([0-9a-fA-F]+)", output)
    if not match or (int(match[1], 16) & 0x0FFFFFFF) != 0x0100481B:
        raise ValueError("JTAG did not identify the expected Gowin 9C FPGA; refusing fixture load")


def serial_ports(probe, tty_root=Path("/sys/class/tty"), dev_root=Path("/dev")):
    parent = Path(probe["path"])
    return sorted(str(dev_root / path.name) for path in tty_root.iterdir()
                  if path.name.startswith(("ttyUSB", "ttyACM"))
                  and parent in (path / "device").resolve().parents
                  and (dev_root / path.name).exists())


def uart_ports(ports, tty_root=Path("/sys/class/tty")):
    """Nano 9K FT2232-compatible debugger: interface B (01) is UART."""
    return [port for port in ports
            if any(read(parent / "bInterfaceNumber") == "01"
                   for parent in (tty_root / Path(port).name / "device").resolve().parents)]


def loopback(port, payload, timeout=3.0):
    """Raw 115200 8N1, no local echo; exact bytes, bounded read/write waits."""
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
        received = bytearray()
        for start in range(0, len(payload), 64):
            block = payload[start:start + 64]
            deadline = time.monotonic() + timeout
            sent = 0
            result = bytearray()
            while sent < len(block) or len(result) < len(block):
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise ValueError(f"UART timeout at byte {start}: received {len(result)}/{len(block)}")
                readable, writable, _ = select.select([fd] if len(result) < len(block) else [],
                                                      [fd] if sent < len(block) else [], [], remaining)
                if writable:
                    sent += os.write(fd, block[sent:])
                if readable:
                    data = os.read(fd, len(block) - len(result))
                    if not data:
                        raise ValueError("UART disconnected during test")
                    result.extend(data)
            if bytes(result) != block:
                raise ValueError(f"UART byte mismatch at block starting {start}")
            received.extend(result)
        if select.select([fd], [], [], 0.1)[0] and os.read(fd, 4096):
            raise ValueError("unexpected extra UART bytes")
        return bytes(received)
    finally:
        if old is not None:
            try:
                termios.tcsetattr(fd, termios.TCSANOW, old)
            except termios.error:
                pass  # A disconnected device cannot have its settings restored.
        os.close(fd)


def run(command, report):
    print("Running: " + " ".join(map(str, command)), flush=True)
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    print(result.stdout, end="")
    print(result.stderr, end="", file=sys.stderr)
    report.setdefault("commands", []).append({"argv": list(map(str, command)),
                                               "returncode": result.returncode,
                                               "stdout": result.stdout, "stderr": result.stderr})
    if result.returncode:
        raise ValueError(f"{command[0]} failed; see {REPORT}")
    return result.stdout


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("prepare", "check", "test"))
    parser.add_argument("--busdev", help="select a USB probe by BUS:DEV")
    parser.add_argument("--port", help="select a serial port belonging to that probe")
    args = parser.parse_args(argv)
    report = {"stage": args.stage, "board": "tangnano9k", "time_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
              "loaded_sram": False, "uart_verified": False, "led_reset_verified": False}
    try:
        missing = [tool for tool in TOOLS if shutil.which(tool) is None]
        if missing:
            raise ValueError("Activate OSS CAD Suite; missing tools: " + ", ".join(missing))
        if args.stage == "prepare":
            run(["just", "board-toolcheck"], report)
            report["fixture_built"] = FIXTURE.is_file()
            print("Ready: connect Nano 9K with a USB data cable, then run just board-check")
            return
        found = probes()
        report["probes"] = found
        probe = choose_probe(found, args.busdev)
        report["selected_probe"] = probe
        if not os.access(probe["usb_node"], os.R_OK | os.W_OK):
            raise ValueError(f"No USB read/write access to {probe['usb_node']}; see boards/tangnano9k/README.md USB setup")
        identity = run(["openFPGALoader", "-b", "tangnano9k", "--busdev-num", probe["busdev"], "--detect"], report)
        verify_identity(identity)
        if args.stage == "test":
            if not FIXTURE.is_file():
                raise ValueError("Run just board-prepare first to build the test fixture")
            run(["openFPGALoader", "-b", "tangnano9k", "--busdev-num", probe["busdev"], str(FIXTURE)], report)
            report["loaded_sram"] = True
            time.sleep(0.5)
        ports = serial_ports(probe)
        report["serial_ports"] = ports
        print("Serial ports for selected probe: " + (", ".join(ports) or "none"))
        if args.stage == "check":
            print("JTAG detection passed. No configuration was loaded.")
            return
        if args.port:
            port = str(Path(args.port).resolve())
            if port not in ports:
                raise ValueError("--port must belong to the selected USB probe")
        else:
            uart = uart_ports(ports)
            if len(uart) == 1:
                port = uart[0]
            else:
                raise ValueError("Choose the debugger UART with --port; close other serial programs first")
        payload = b"X_eTaL Nano 9K SRAM loopback\r\n" + bytes(range(256)) * 2
        report["serial_port"] = port
        report["uart_bytes"] = len(loopback(port, payload))
        report["uart_verified"] = True
        print(f"PASS: {len(payload)} UART bytes returned exactly at 115200 baud, including all byte values.")
        print("Observe LED blinking about 1.6 times/second; hold the pin-3 button to reset it.")
        print("LED/reset observation and power-cycle recovery are manual checks, still pending in the report.")
    except (OSError, ValueError) as error:
        report["error"] = str(error)
        raise
    finally:
        REPORT.parent.mkdir(parents=True, exist_ok=True)
        data = json.dumps(report, indent=2) + "\n"
        REPORT.write_text(data)
        REPORT.with_name(f"first-board-{args.stage}.json").write_text(data)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError) as error:
        print(f"board-first: {error}", file=sys.stderr)
        sys.exit(1)
