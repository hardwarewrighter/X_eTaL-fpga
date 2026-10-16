#!/usr/bin/env python3
"""Verify UART display parsing and absence of a host answer fallback."""
import importlib.util
from pathlib import Path
import socket
import threading
import unittest

spec = importlib.util.spec_from_file_location("calc", Path(__file__).with_name("board-hex1000.py"))
calc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(calc)


class CalculatorSerialTests(unittest.TestCase):
    def exchange(self, response):
        host, device = socket.socketpair()
        self.addCleanup(host.close)
        self.addCleanup(device.close)
        keys = "X123*4="
        sent = bytearray()

        def receive():
            while len(sent) < len(keys):
                sent.extend(device.recv(64))
            if response:
                device.sendall(response)

        worker = threading.Thread(target=receive, daemon=True)
        worker.start()
        try:
            result = calc.calculate(host.fileno(), keys, pace=0.001)
            self.assertEqual(bytes(sent), keys.encode())
            return result
        finally:
            worker.join(timeout=1)

    def test_uses_last_received_display_and_preserves_raw_evidence(self):
        raw = b"       0\r     123\r     48C\r"
        self.assertEqual(self.exchange(raw), ("     48C", raw))

    def test_returns_device_answer_even_when_mathematically_wrong(self):
        # This query path is a reader, not an evaluator or expected-value lookup.
        self.assertEqual(self.exchange(b"DEADBEEF\r"), ("DEADBEEF", b"DEADBEEF\r"))

    def test_no_response_or_local_echo_cannot_supply_an_answer(self):
        for response in (b"", b"X123*4=", b"X1234*4=", b"DEADBEEF"):
            with self.subTest(response=response), self.assertRaisesRegex(ValueError, "No display frame"):
                self.exchange(response)


if __name__ == "__main__":
    unittest.main()
