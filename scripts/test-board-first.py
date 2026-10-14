#!/usr/bin/env python3
"""Test discovery and raw serial I/O using Linux pseudoterminals."""
import importlib.util
import os
from pathlib import Path
import pty
import select
import tempfile
import termios
import threading
import unittest

spec = importlib.util.spec_from_file_location("first", Path(__file__).with_name("board-first.py"))
first = importlib.util.module_from_spec(spec)
spec.loader.exec_module(first)


class DiscoveryTests(unittest.TestCase):
    def test_unexpected_fpga_is_rejected_before_load(self):
        first.verify_identity("index 0:\n idcode 0x100481b\n")
        for output in ("idcode 0x01234567", "no FPGA detected"):
            with self.assertRaisesRegex(ValueError, "expected Gowin"):
                first.verify_identity(output)

    def test_probe_and_serial_are_matched_by_usb_ancestor(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            usb = root / "usb"
            usb.mkdir()
            probe = usb / "1-2"
            probe.mkdir()
            for name, value in {"idVendor": "0403", "idProduct": "6010", "busnum": "1",
                                "devnum": "8", "serial": "TEST", "product": "Debugger"}.items():
                (probe / name).write_text(value)
            other = usb / "1-3"
            other.mkdir()
            (other / "idVendor").write_text("1234")
            found = first.probes(usb)
            self.assertEqual(len(found), 1)
            self.assertEqual(found[0]["busdev"], "1:8")
            tty_root, dev = root / "tty", root / "dev"
            tty_root.mkdir()
            dev.mkdir()
            for name, parent in (("ttyUSB0", probe), ("ttyUSB1", other)):
                path = tty_root / name
                path.mkdir()
                interface = parent / "interface"
                interface.mkdir()
                (interface / "bInterfaceNumber").write_text("01" if parent == probe else "00")
                target = interface / "serial"
                target.mkdir()
                (path / "device").symlink_to(target)
                (dev / name).touch()
            self.assertEqual(first.serial_ports(found[0], tty_root, dev), [str(dev / "ttyUSB0")])
            self.assertEqual(first.uart_ports([str(dev / "ttyUSB0"), str(dev / "ttyUSB1")], tty_root),
                             [str(dev / "ttyUSB0")])

    def test_no_probe_and_multiple_probes_require_selection(self):
        for probes in ([], [{"busdev": "1:2"}, {"busdev": "1:3"}]):
            with self.assertRaises(ValueError):
                first.choose_probe(probes)
        self.assertEqual(first.choose_probe([{"busdev": "1:2"}, {"busdev": "1:3"}], "1:3"),
                         {"busdev": "1:3"})


class SerialTests(unittest.TestCase):
    def test_inconsistent_response_length_is_rejected(self):
        with self.assertRaises(ValueError):
            first.loopback("unused", b"??", expected=b"x", block_size=1)

    def setUp(self):
        self.master, self.slave = pty.openpty()
        self.addCleanup(os.close, self.master)
        self.addCleanup(os.close, self.slave)
        self.port = os.ttyname(self.slave)

    def emulator(self, corrupt=False):
        stopped = threading.Event()

        def echo():
            while not stopped.is_set():
                if select.select([self.master], [], [], 0.02)[0]:
                    data = os.read(self.master, 4096)
                    os.write(self.master, bytes(b ^ 1 for b in data) if corrupt else data)
        thread = threading.Thread(target=echo, daemon=True)
        thread.start()

        def cleanup():
            stopped.set()
            thread.join(timeout=1)
        self.addCleanup(cleanup)

    def test_exact_binary_loopback_and_restored_settings(self):
        before = termios.tcgetattr(self.slave)
        self.emulator()
        data = b"hello\r\n\x00" + bytes(range(256)) * 4
        self.assertEqual(first.loopback(self.port, data, timeout=1), data)
        self.assertEqual(termios.tcgetattr(self.slave), before)

    def test_corrupted_bytes_fail(self):
        self.emulator(corrupt=True)
        with self.assertRaisesRegex(ValueError, "mismatch"):
            first.loopback(self.port, b"hello", timeout=1)

    def test_per_byte_transformed_response_in_one_session(self):
        self.emulator(corrupt=True)
        data = bytes(range(256))
        expected = bytes(value ^ 1 for value in data)
        self.assertEqual(first.loopback(self.port, data, expected=expected, block_size=1), expected)

    def test_local_echo_does_not_make_disconnected_loop_pass(self):
        attrs = termios.tcgetattr(self.slave)
        attrs[3] |= termios.ECHO
        termios.tcsetattr(self.slave, termios.TCSANOW, attrs)
        with self.assertRaisesRegex(ValueError, "timeout"):
            first.loopback(self.port, b"hello", timeout=0.1)


if __name__ == "__main__":
    unittest.main()
