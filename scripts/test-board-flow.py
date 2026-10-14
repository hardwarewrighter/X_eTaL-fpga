#!/usr/bin/env python3
"""Exercise pin guards, tool failures and SRAM/flash command selection."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("flow", Path(__file__).with_name("board-flow.py"))
flow = importlib.util.module_from_spec(spec)
spec.loader.exec_module(flow)
ROOT = Path(__file__).resolve().parent.parent


class BoardFlowTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.board = flow.board_config(str(ROOT / "boards/tangnano9k/board.json"))

    def cst(self, source):
        path = self.directory / "test.cst"
        path.write_text(source)
        return path

    def test_rejects_unapproved_jtag_or_external_pin(self):
        for source in ('IO_LOC "led" 1;', 'IO_LOC "jtag" 10;', 'IO_LOC "clk" 10;'):
            with self.subTest(source=source), self.assertRaises(ValueError):
                flow.constraints(self.cst(source), self.board)

    def test_rejects_duplicate_pin_and_unsupported_statements(self):
        for source in ('IO_LOC "led" 10;\nIO_LOC "led" 10;',
                       'IO_LOC "led" 10;\nCONFIG JTAG_AS_GPIO=YES;',
                       'IO_LOC "clk" 52;\nIO_PORT "clk" DRIVE=8;',
                       'IO_PORT "led" IO_TYPE=LVCMOS33;'):
            with self.subTest(source=source), self.assertRaises(ValueError):
                flow.constraints(self.cst(source), self.board)

    def test_constraints_and_top_must_match_and_preserve_inputs(self):
        netlist = self.directory / "net.json"
        pins = flow.constraints(ROOT / "boards/tangnano9k/led.cst", self.board)
        ports = {name: {"direction": self.board["ports"][name]["direction"], "bits": [i]}
                 for i, name in enumerate(pins)}
        data = {"modules": {"top": {"attributes": {"top": "1"}, "ports": ports}}}
        netlist.write_text(json.dumps(data))
        flow.check_ports(netlist, None, pins, self.board)
        ports["clk"]["direction"] = "output"
        netlist.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, "wrong direction"):
            flow.check_ports(netlist, None, pins, self.board)
        ports["clk"]["direction"] = "input"
        netlist.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, "differ"):
            flow.check_ports(netlist, None, {"led": 10}, self.board)

    def test_sram_default_and_explicit_flash(self):
        bitstream = self.directory / "test.fs"
        bitstream.write_text("fixture")
        common = ["load", str(bitstream), "--board", str(ROOT / "boards/tangnano9k/board.json")]
        with patch.object(flow, "execute") as execute:
            flow.main(common)
            self.assertEqual(execute.call_args.args[0],
                             ["openFPGALoader", "-b", "tangnano9k", str(bitstream)])
            flow.main(common + ["--flash"])
            self.assertIn("-f", execute.call_args.args[0])

    def test_divided_clock_constraints_require_pnr_and_existing_file(self):
        with self.assertRaisesRegex(ValueError, "--sdc requires"):
            flow.main(["detect", "--sdc", "missing.sdc"])
        netlist = self.directory / "clocked.json"
        pins = flow.constraints(ROOT / "boards/tangnano9k/led.cst", self.board)
        ports = {name: {"direction": self.board["ports"][name]["direction"], "bits": [i]}
                 for i, name in enumerate(pins)}
        netlist.write_text(json.dumps({"modules": {"top": {"attributes": {"top": "1"}, "ports": ports}}}))
        sdc = self.directory / "clock.sdc"
        sdc.write_text("create_clock -period 37 [get_ports clk]\n")
        with patch.object(flow, "execute") as execute:
            flow.main(["pnr", str(netlist), "--cst", str(ROOT / "boards/tangnano9k/led.cst"),
                       "--out", str(self.directory / "pnr.json"), "--sdc", str(sdc)])
            self.assertEqual(execute.call_args.args[0][-2:], ["--sdc", str(sdc)])

    def test_dry_run_never_invokes_tool(self):
        with patch.object(flow.subprocess, "run") as run, contextlib.redirect_stdout(io.StringIO()):
            flow.execute(["absent-tool"], True)
            run.assert_not_called()

    def test_missing_tool_and_nonzero_exit_are_errors(self):
        with contextlib.redirect_stdout(io.StringIO()), patch.object(flow.shutil, "which", return_value=None):
            with self.assertRaisesRegex(ValueError, "missing on PATH"):
                flow.execute(["yosys"], False)
        with contextlib.redirect_stdout(io.StringIO()), patch.object(flow.shutil, "which", return_value="yosys"):
            with patch.object(flow.subprocess, "run", side_effect=flow.subprocess.CalledProcessError(1, "yosys")):
                with self.assertRaises(flow.subprocess.CalledProcessError):
                    flow.execute(["yosys"], False)

    def test_yosys_path_rejects_command_injection(self):
        for value in ('a; shell.v', 'a".v', 'a\n.v'):
            with self.assertRaises(ValueError):
                flow.yosys_path(value)


if __name__ == "__main__":
    unittest.main()
