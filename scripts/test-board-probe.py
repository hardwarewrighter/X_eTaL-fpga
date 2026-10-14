#!/usr/bin/env python3
"""Check the independent UART waveform decoder used for reference results."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("probe", Path(__file__).with_name("board-probe.py"))
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class DecoderTests(unittest.TestCase):
    def test_frames_with_all_byte_values(self):
        for clocks in (3, 4, 234):
            line = [1] * clocks
            for value in range(256):
                frame = [0] + [(value >> bit) & 1 for bit in range(8)] + [1]
                line += [bit for bit in frame for _ in range(clocks)] + [1]
            self.assertEqual(probe.decode_uart(line, clocks), list(range(256)))

    def test_missing_stop_or_truncated_frame_is_rejected(self):
        for line in ([0] * 30, [0] * 8):
            with self.assertRaises(ValueError):
                probe.decode_uart(line, 3)


if __name__ == "__main__":
    unittest.main()
