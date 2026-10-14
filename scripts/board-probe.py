#!/usr/bin/env python3
"""X_eTaL counter/UART proof: evaluator, IR, simulation, bitstream, board."""
import argparse
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "work/board/probe"
CLI = ROOT / "target/release/xetal-fpga"
EXAMPLES = ROOT / "boards/examples"


def load_script(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


first = load_script("board-first")


def run(args, label):
    result = subprocess.run(list(map(str, args)), cwd=ROOT, capture_output=True, text=True)
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / f"{label}.log").write_text(result.stdout + result.stderr)
    if result.returncode:
        raise ValueError(f"{label} failed: {result.stderr[-1000:]}; see {OUT / (label + '.log')}")
    return result.stdout


def evaluate(name):
    output = run([ROOT / "scripts/xt", "run", EXAMPLES / f"{name}.xtl"], f"{name}-eval")
    if output != (EXAMPLES / f"{name}.out").read_text():
        raise ValueError(f"{name} evaluator differs from the reviewed baseline")
    return output.splitlines()


def decode_uart(line, clocks):
    values = []
    i = 0
    while i < len(line):
        if line[i] == 1:
            i += 1
            continue
        start = i
        positions = [start + clocks + clocks // 2 + bit * clocks for bit in range(8)]
        stop = start + 9 * clocks + clocks // 2
        if stop >= len(line) or line[stop] != 1:
            raise ValueError("truncated frame or missing UART stop bit")
        values.append(sum(line[position] << bit for bit, position in enumerate(positions)))
        i = start + 10 * clocks
    return values


def check():
    run(["just", "build"], "build-cli")
    rows = evaluate("probe")
    count = evaluate("counter")[0]
    reference_bytes = decode_uart(list(map(int, rows[1].split())), 3)
    if reference_bytes != [0, 255, 85, 170, 125, 126, 127, 128]:
        raise ValueError(f"unexpected reference responses: {reference_bytes}")
    for name, stream, wanted, declaration in (
            ("probe", rows[0], "\n".join(rows[1:]) + "\n", "%rxline = stream i16 1"),
            ("counter", " ".join(["1"] * (len(count.split()) - 1)), count + "\n", "%e = stream i8 1")):
        template = (EXAMPLES / f"{name}.xir").read_text()
        program = OUT / f"{name}-test.xir"
        program.write_text(template.replace(declaration, declaration.rsplit(" ", 1)[0] + " " + stream))
        for command in ("run", "sim"):
            if run([CLI, command, program], f"{name}-{command}") != wanted:
                raise ValueError(f"{name} {command} differs from X_eTaL evaluator")
        verilog = OUT / f"{name}-test.v"
        tb = OUT / f"{name}-test.tb.v"
        verilog.write_text(run([CLI, "verilog", program], f"{name}-verilog"))
        tb.write_text(run([CLI, "testbench", program], f"{name}-testbench"))
        if shutil.which("iverilog") and shutil.which("vvp"):
            run(["iverilog", "-g2005", "-o", OUT / f"{name}.vvp", verilog, tb], f"{name}-compile")
            output = run(["vvp", OUT / f"{name}.vvp"], f"{name}-iverilog")
            if "PASS" not in output or "FAIL" in output:
                raise ValueError(f"{name} Verilog testbench failed")
        else:
            print("board-probe: SKIPPED Verilog simulation (iverilog/vvp missing)")
    print("PASS: reviewed X_eTaL baselines match IR, netlist and available Verilog simulations; UART frames decode correctly")


def build():
    check()
    board = json.loads((ROOT / "boards/tangnano9k/board.json").read_text())
    clocks = round(board["clock_mhz"] * 1_000_000 / board["uart_baud"])
    source = (EXAMPLES / "probe.xir").read_text()
    source = source.replace("%dm1 = const i16 2", f"%dm1 = const i16 {clocks - 1}")
    source = source.replace("%rxk = const i16 3", f"%rxk = const i16 {clocks + clocks // 2 - 1}")
    program = OUT / "probe.xir"
    program.write_text(source)
    probe_v = run([CLI, "verilog", program], "hardware-probe")
    counter_v = run([CLI, "verilog", EXAMPLES / "counter.xir"], "hardware-counter")
    design = OUT / "design.v"
    design.write_text((ROOT / "boards/tangnano9k/probe-top.v").read_text() + "\n" + probe_v + "\n" + counter_v)
    cst = OUT / "design.cst"
    cst.write_text((ROOT / "boards/tangnano9k/uart.cst").read_text() +
                   'IO_LOC "led" 10;\nIO_PORT "led" IO_TYPE=LVCMOS33 PULL_MODE=UP DRIVE=8;\n')
    run([CLI, "synth", design, "--top", "probe_top", "--cst", cst, "--out", OUT / "synth.json"], "synth")
    board_simulation(design)
    run([CLI, "pnr", OUT / "synth.json", "--cst", cst, "--out", OUT / "pnr.json"], "pnr")
    run([CLI, "pack", OUT / "pnr.json", "--out", OUT / "probe.fs"], "pack")
    print(f"Built {OUT / 'probe.fs'}: emitted X_eTaL UART/counter cores, {clocks} clocks per bit")


def board_simulation(design):
    if not shutil.which("iverilog") or not shutil.which("vvp"):
        print("board-probe: SKIPPED board-top simulation (iverilog/vvp missing)")
        return
    yosys = Path(shutil.which("yosys")).resolve()
    cells = yosys.parent.parent / "share/yosys/gowin/cells_sim.v"
    if not cells.is_file():
        # Distro installations normally use /usr/share/yosys.
        cells = Path("/usr/share/yosys/gowin/cells_sim.v")
    if not cells.is_file():
        raise ValueError("Yosys Gowin simulation models missing; activate OSS CAD Suite")
    mapped = OUT / "mapped.v"
    # Paths are quoted and validated by the same helper as CLI synthesis.
    flow = load_script("board-flow")
    run(["yosys", "-p", f"read_json {flow.yosys_path(OUT / 'synth.json')}; "
         f"write_verilog -noattr {flow.yosys_path(mapped)}"], "mapped-verilog")
    for name, sources in (("board-rtl", [design]), ("board-mapped", [cells, mapped])):
        binary = OUT / f"{name}.vvp"
        run(["iverilog", "-g2005-sv", "-s", "tb", "-o", binary, *sources,
             ROOT / "boards/tangnano9k/probe-top.tb.v"], f"{name}-compile")
        output = run(["vvp", binary], name)
        if "PASS" not in output:
            raise ValueError(f"{name} simulation did not pass")
    print("PASS: complete board top at real timing, including mapped Gowin cells")


def hardware(stage):
    report = {"stage": stage, "configuration_loaded": False, "persistent_flash_written": False}
    try:
        probe = first.choose_probe(first.probes())
        report["usb_probe"] = probe
        identity = run(["openFPGALoader", "-b", "tangnano9k", "--busdev-num", probe["busdev"], "--detect"], "detect")
        first.verify_identity(identity)
        if stage in ("test", "flash"):
            bitstream = OUT / "probe.fs"
            if not bitstream.is_file():
                raise ValueError("Run just board-probe-build first")
            command = ["openFPGALoader", "-b", "tangnano9k", "--busdev-num", probe["busdev"]]
            if stage == "flash":
                command += ["-f"]
            run(command + [bitstream], "load")
            report["configuration_loaded"] = True
            report["persistent_flash_written"] = stage == "flash"
            time.sleep(0.5)
        ports = first.uart_ports(first.serial_ports(probe))
        if len(ports) != 1:
            raise ValueError(f"expected one interface-B UART, found {ports}")
        golden = bytes(int(value) % 256 for value in evaluate("counter")[0].split()[1:])
        # Every other byte echoes; '?' is reserved for the counter query.
        echo = bytes(value for value in range(256) if value != 63)
        # Keep one serial session: changing baud/hangup between phases can
        # inject a false start through the debugger's UART bridge.
        first.loopback(ports[0], b"?" * len(golden) + echo,
                       expected=golden + echo, block_size=1)
        report["counter_queries_verified"] = len(golden)
        report["echo_bytes_verified"] = len(echo)
        report["uart_port"] = ports[0]
        print(f"PASS: {len(golden)} physical counter results equal X_eTaL (two full 8-bit wraps), {len(echo)} echo bytes exact")
    except (OSError, ValueError) as error:
        report["error"] = str(error)
        raise
    finally:
        OUT.mkdir(parents=True, exist_ok=True)
        (OUT / f"{stage}-report.json").write_text(json.dumps(report, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("check", "build", "test", "flash", "uart"))
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
        print(f"board-probe: {error}", file=sys.stderr)
        sys.exit(1)
