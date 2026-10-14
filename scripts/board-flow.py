#!/usr/bin/env python3
"""Board tools: explicit stages, checked pins, SRAM loading by default."""
import argparse
import json
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys


def board_config(name):
    path = Path(name)
    if not path.is_file():
        path = Path("boards") / name / "board.json"
    data = json.loads(path.read_text())
    for key in ("device", "family", "loader", "clock_mhz", "ports"):
        if key not in data:
            raise ValueError(f"{path}: missing {key}")
    return data


def constraints(path, board):
    """Accept only known board signals and a small set of electrical options."""
    locations = {}
    options = set()
    allowed = board["ports"]
    for number, raw in enumerate(Path(path).read_text().splitlines(), 1):
        line = raw.split("//", 1)[0].strip()
        if not line:
            continue
        match = re.fullmatch(r'IO_LOC "([\w\[\]]+)" (\d+);', line)
        if match:
            name, pin = match.groups()
            if name not in allowed or int(pin) != allowed[name]["pin"]:
                raise ValueError(f"{path}:{number}: unapproved signal/pin {name}/{pin}")
            if name in locations or int(pin) in locations.values():
                raise ValueError(f"{path}:{number}: duplicate signal or pin")
            locations[name] = int(pin)
            continue
        match = re.fullmatch(r'IO_PORT "([\w\[\]]+)" (.+);', line)
        if match:
            name, settings = match.groups()
            if name not in allowed or name in options:
                raise ValueError(f"{path}:{number}: unknown or duplicate IO_PORT {name}")
            seen = set()
            for item in settings.split():
                option = re.fullmatch(r'(IO_TYPE|PULL_MODE|DRIVE)=(\w+)', item)
                if not option:
                    raise ValueError(f"{path}:{number}: unsupported option {item}")
                key, value = option.groups()
                values = {"IO_TYPE": {"LVCMOS33"}, "PULL_MODE": {"UP", "DOWN", "NONE"},
                          "DRIVE": {"4", "8", "12", "16"}}
                if key in seen or value not in values[key]:
                    raise ValueError(f"{path}:{number}: invalid option {item}")
                if key == "DRIVE" and allowed[name]["direction"] == "input":
                    raise ValueError(f"{path}:{number}: cannot drive board input {name}")
                seen.add(key)
            options.add(name)
            continue
        raise ValueError(f"{path}:{number}: unsupported constraints statement")
    if not locations or not options.issubset(locations):
        raise ValueError(f"{path}: missing IO_LOC assignments")
    return locations


def check_ports(netlist, top, pins, board):
    modules = json.loads(Path(netlist).read_text())["modules"]
    if top is None:
        candidates = [name for name, module in modules.items()
                      if int(str(module.get("attributes", {}).get("top", "0")), 2)]
        if len(candidates) != 1:
            raise ValueError("netlist must have exactly one marked top module")
        top = candidates[0]
    ports = modules[top]["ports"]
    names = set()
    for name, port in ports.items():
        bits = port["bits"]
        names.update([name] if len(bits) == 1 else
                     [f"{name}[{i + port.get('offset', 0)}]" for i in range(len(bits))])
        for signal in ([name] if len(bits) == 1 else
                       [f"{name}[{i + port.get('offset', 0)}]" for i in range(len(bits))]):
            if signal not in board["ports"]:
                raise ValueError(f"unapproved top port {signal}")
            if port["direction"] != board["ports"][signal]["direction"]:
                raise ValueError(f"wrong direction for board port {signal}")
    if names != set(pins):
        raise ValueError(f"constraints/top ports differ: {sorted(names ^ set(pins))}")


def yosys_path(path):
    value = str(Path(path).resolve())
    if any(c in value for c in ('"', '\\', '\n', '\r', ';')):
        raise ValueError("unsupported character in Yosys path")
    return f'"{value}"'


def execute(command, dry_run):
    print(shlex.join(command), flush=True)
    if dry_run:
        return
    if not shutil.which(command[0]):
        raise ValueError(f"{command[0]} missing on PATH; activate the OSS CAD Suite")
    subprocess.run(command, check=True)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("synth", "pnr", "pack", "load", "detect"))
    parser.add_argument("file", nargs="?")
    parser.add_argument("--board", default="tangnano9k", help="board name or board.json path")
    parser.add_argument("--top")
    parser.add_argument("--cst")
    parser.add_argument("--out")
    parser.add_argument("--sdc", help="timing constraints for pnr, including divided clocks")
    parser.add_argument("--flash", action="store_true", help="load persistently instead of SRAM")
    parser.add_argument("--dry-run", action="store_true", help="validate and print without running tools")
    args = parser.parse_args(argv)
    board = board_config(args.board)
    if args.flash and args.stage != "load":
        raise ValueError("--flash applies only to load")
    if args.sdc and (args.stage != "pnr" or not Path(args.sdc).is_file()):
        raise ValueError("--sdc requires an existing timing file and pnr stage")
    if args.stage != "detect" and (not args.file or not Path(args.file).is_file()):
        raise ValueError("an existing input file is required")
    if args.stage in ("synth", "pnr", "pack") and not args.out:
        raise ValueError("--out is required")
    if args.stage in ("synth", "pnr") and not args.cst:
        raise ValueError("--cst is required")
    if args.stage == "synth":
        if not args.top or not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', args.top):
            raise ValueError("synth requires a simple --top module name")
        pins = constraints(args.cst, board)
        command = ["yosys", "-p", f"read_verilog {yosys_path(args.file)}; "
                   f"synth_gowin -top {args.top} -json {yosys_path(args.out)}"]
    elif args.stage == "pnr":
        pins = constraints(args.cst, board)
        check_ports(args.file, args.top, pins, board)
        command = ["nextpnr-himbaechel", "--device", board["device"], "--vopt",
                   f"family={board['family']}", "--vopt", f"cst={Path(args.cst).resolve()}",
                   "--json", args.file, "--write", args.out, "--freq", str(board["clock_mhz"])]
        if args.sdc:
            command += ["--sdc", args.sdc]
    elif args.stage == "pack":
        command = ["gowin_pack", "-d", board["family"], "-o", args.out, args.file]
    else:
        command = ["openFPGALoader", "-b", board["loader"]]
        if args.stage == "detect":
            command += ["--detect"]
        else:
            command += (["-f"] if args.flash else []) + [args.file]
    if args.out and not args.dry_run:
        Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    execute(command, args.dry_run)
    if args.stage == "synth" and not args.dry_run:
        check_ports(args.out, args.top, pins, board)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, KeyError, subprocess.CalledProcessError) as error:
        print(f"board-flow: {error}", file=sys.stderr)
        sys.exit(1)
