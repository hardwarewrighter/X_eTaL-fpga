#!/usr/bin/env python3
"""HEX1000 hand-lowered to the provisional IR: writes
libs/Hex1000/demos/hex1000.xir, the circuit of hx:s_tep (one instruction
per clock), from docs/machine.md's definition, with the calculator ROM
(libs/Hex1000/rom/calc.hex) as a constant table and, as its stream, the
K inputs of the demo libs/Hex1000/demos/circuit.xtl (12+34= typed at
the calculator; line 1 of its baseline). The circuit must reproduce the
demo's O and R traces clock for clock (scripts/check-equiv.sh).

The lowering is written here, one rule per instruction, rather than
node by node in the .xir: each register's next value is a chain of
selects, the first instruction that sets it winning, the register kept
otherwise. Until the IR has unsigned types every 4-bit value is held
one bit wider (i5) and kept in 0 to 15 by masks.

  scripts/gen-hex1000-xir.py [--check]   (--check: fail if the pinned
                                          .xir differs)
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "libs/Hex1000/demos/hex1000.xir"
BASELINE = ROOT / "libs/Hex1000/tests/demo-circuit.out"
ROM = ROOT / "libs/Hex1000/rom/calc.hex"


class Ir:
    def __init__(self):
        self.lines, self.types, self.n = [], {}, 0

    def node(self, ty, text, name=None):
        if name is None:
            self.n += 1
            name = f"t{self.n}"
        self.lines.append(f"%{name} = {text}")
        self.types[name] = ty
        return name

    def const(self, ty, v):
        return self.node(ty, f"const {ty} {v}")

    def ty(self, x):
        return self.types[x]

    def cast(self, ty, x):
        return x if self.ty(x) == ty else self.node(ty, f"cast {ty} %{x}")

    def map(self, op, a, b=None):
        cmp = op in ("eq", "ne", "lt", "le", "gt", "ge")
        ty = "bool" if cmp else self.ty(a)
        return self.node(ty, f"map {op} %{a}" + (f" %{b}" if b else ""))

    def sel(self, m, a, b):
        return self.node(self.ty(a), f"select %{m} %{a} %{b}")

    def chain(self, cases, default):
        """The first case whose condition holds; else default."""
        out = default
        for cond, val in reversed(cases):
            out = self.sel(cond, val, out)
        return out


def main():
    out_lines = BASELINE.read_text().splitlines()
    ks = out_lines[0].split()
    rom = [int(b, 16) for b in ROM.read_text().split()]
    assert len(rom) == 1024
    ir = Ir()
    c = lambda ty, v: ir.const(ty, v)
    # Inputs and state.
    k = ir.node("i5", "stream i5 " + " ".join(ks), "k")
    a = ir.node("i5", "state i5 0", "a")
    y = ir.node("i5", "state i5 0", "y")
    x = ir.node("i3", "state i3 0", "x")
    s = ir.node("bool", "state bool 1", "s")
    pc = ir.node("i7", "state i7 0", "pc")
    pa = ir.node("i5", "state i5 0", "pa")
    pb = ir.node("i5", "state i5 0", "pb")
    sr = ir.node("i7", "state i7 0", "sr")
    cl = ir.node("bool", "state bool 0", "cl")
    o = ir.node("i6", "state i6 0", "o")
    r = ir.node("bool[16]", "state bool[16] " + " ".join(["0"] * 16), "r")
    ram = ir.node("i5[64]", "state i5[64] " + " ".join(["0"] * 64), "ram")
    romt = ir.node("i9[1024]", "const i9[1024] " + " ".join(map(str, rom)), "rom")
    # Fetch: the instruction at 64 PA + PC.
    addr = ir.map("add", ir.node("i12", f"shl %{ir.cast('i12', pa)} 6"), ir.cast("i12", pc))
    op = ir.node("i9", f"lookup %{romt} %{addr}", "op")
    # Decode.
    f15 = c("i9", 15)
    n = ir.cast("i5", ir.map("and", op, f15))                      # the operand, 0 to 15
    hi4 = ir.node("i9", f"shr %{op} 4")
    hi2 = ir.node("i9", f"shr %{op} 6")
    eq = lambda v, val, ty="i9": ir.map("eq", v, c(ty, val))
    fixed = ["NOP", "TAY", "TYA", "CLA", "TAM", "TMA", "TMY", "TAMIY", "TAMZA", "XMA", "AMAAC",
             "SAMAN", "IMAC", "DMAN", "IA", "IYC", "DYN", "CPAIZ", "ALEM", "MNEZ", "YNEA", "KNEZ",
             "TKA", "SETR", "RSTR", "TDO", "CLO", "COMX", "RETN"]
    is_ = {name: eq(op, code) for code, name in enumerate(fixed) if name != "NOP"}
    ff = ir.map("and", ir.node("i9", f"shr %{op} 2"), c("i9", 3))
    bitgrp = eq(hi4, 2)
    for i, name in enumerate(["SBIT", "RBIT", "TBIT", "LDX"]):
        is_[name] = ir.map("and", bitgrp, eq(ff, i))
    for v, name in [(3, "LDP"), (4, "TCY"), (5, "TCMIY"), (6, "YNEC"), (7, "ACC")]:
        is_[name] = eq(hi4, v)
    is_["BR"], is_["CALL"] = eq(hi2, 2), eq(hi2, 3)
    # Values the instructions use.
    at = ir.map("add", ir.node("i8", f"shl %{ir.cast('i8', x)} 4"), ir.cast("i8", y))
    m = ir.node("i5", f"lookup %{ram} %{at}", "m")
    m15 = c("i5", 15)
    mask4 = lambda v: ir.map("and", v, c(ir.ty(v), 15))
    six = lambda v: ir.cast("i6", v)
    sum_am = ir.map("add", six(a), six(m))
    sum_an = ir.map("add", six(a), six(n))
    one5, zero5 = c("i5", 1), c("i5", 0)
    y1 = mask4(ir.map("add", y, one5))
    t, f = c("bool", 1), c("bool", 0)
    bitv = ir.node("i5", f"lookup %{c('i5[4]', '1 2 4 8')} %{ir.map('and', op, c('i9', 3))}")
    bitset = ir.map("ne", ir.map("and", m, bitv), zero5)
    go = ir.cast("i7", ir.map("and", op, c("i9", 63)))
    # The next PC, the shift register's rule.
    b5 = ir.map("and", ir.node("i7", f"shr %{pc} 5"), c("i7", 1))
    b4 = ir.map("and", ir.node("i7", f"shr %{pc} 4"), c("i7", 1))
    low5 = ir.map("and", pc, c("i7", 31))
    fb = ir.map("xor", ir.map("eq", b5, b4), ir.map("eq", low5, c("i7", 31)))
    p1 = ir.map("add", ir.node("i7", f"shl %{low5} 1"), ir.cast("i7", fb))
    taken = lambda name: ir.map("and", is_[name], s)
    call = ir.map("and", taken("CALL"), ir.map("not", cl))
    retn = ir.map("and", is_["RETN"], cl)
    br_page = ir.map("and", taken("BR"), ir.map("not", cl))
    # Each register's next value: the first instruction that sets it.
    a2 = ir.chain([(is_["TYA"], y), (is_["CLA"], zero5), (is_["TAMZA"], zero5),
                   (is_["TMA"], m), (is_["XMA"], m),
                   (is_["AMAAC"], ir.cast("i5", mask4(sum_am))),
                   (is_["SAMAN"], mask4(ir.map("sub", m, a))),
                   (is_["IMAC"], mask4(ir.map("add", m, one5))),
                   (is_["DMAN"], mask4(ir.map("sub", m, one5))),
                   (is_["IA"], mask4(ir.map("add", a, one5))),
                   (is_["CPAIZ"], mask4(ir.map("sub", zero5, a))),
                   (is_["TKA"], k),
                   (is_["ACC"], ir.cast("i5", mask4(sum_an)))], a)
    y2 = ir.chain([(is_["TAY"], a), (is_["TMY"], m), (is_["TAMIY"], y1), (is_["TCMIY"], y1),
                   (is_["IYC"], y1), (is_["DYN"], mask4(ir.map("sub", y, one5))),
                   (is_["TCY"], n)], y)
    x2 = ir.chain([(is_["LDX"], ir.cast("i3", ir.map("and", op, c("i9", 3)))),
                   (is_["COMX"], ir.map("sub", c("i3", 3), x))], x)
    s2 = ir.chain([(is_["AMAAC"], ir.map("gt", sum_am, c("i6", 15))),
                   (is_["SAMAN"], ir.map("ge", m, a)),
                   (is_["IMAC"], ir.map("eq", m, m15)),
                   (is_["DMAN"], ir.map("gt", m, zero5)),
                   (is_["IYC"], ir.map("eq", y, m15)),
                   (is_["DYN"], ir.map("gt", y, zero5)),
                   (is_["CPAIZ"], ir.map("eq", a, zero5)),
                   (is_["ALEM"], ir.map("le", a, m)),
                   (is_["MNEZ"], ir.map("ne", m, zero5)),
                   (is_["YNEA"], ir.map("ne", y, a)),
                   (is_["KNEZ"], ir.map("ne", k, zero5)),
                   (is_["TBIT"], bitset),
                   (is_["YNEC"], ir.map("ne", y, n)),
                   (is_["ACC"], ir.map("gt", sum_an, c("i6", 15)))], t)
    pc2 = ir.chain([(retn, sr), (taken("BR"), go), (taken("CALL"), go)], p1)
    pa2 = ir.chain([(retn, pb), (br_page, pb), (call, pb)], pa)
    pb2 = ir.chain([(is_["LDP"], n), (call, pa)], pb)
    sr2 = ir.chain([(call, p1)], sr)
    cl2 = ir.chain([(call, t), (retn, f)], cl)
    o2 = ir.chain([(is_["TDO"], ir.map("add", six(a), ir.sel(s, c("i6", 16), c("i6", 0)))),
                   (is_["CLO"], c("i6", 0))], o)
    r2 = ir.chain([(is_["SETR"], ir.node("bool[16]", f"put %{r} %{y} %{t}")),
                   (is_["RSTR"], ir.node("bool[16]", f"put %{r} %{y} %{f}"))], r)
    writes = ["TAM", "TAMIY", "TAMZA", "XMA", "TCMIY", "SBIT", "RBIT"]
    we = is_[writes[0]]
    for w in writes[1:]:
        we = ir.map("or", we, is_[w])
    wd = ir.chain([(is_["TCMIY"], n), (is_["SBIT"], ir.map("or", m, bitv)),
                   (is_["RBIT"], ir.map("and", m, ir.map("xor", bitv, m15)))], a)
    # Out of range (64) when not writing: the RAM is unchanged.
    waddr = ir.sel(we, at, c("i8", 64))
    ram2 = ir.node("i5[64]", f"put %{ram} %{waddr} %{wd}")
    head = [
        "# HEX1000, the circuit of hx:s_tep: one instruction per clock.",
        "# Written by scripts/gen-hex1000-xir.py; do not edit (regenerate).",
        "# The ROM is rom/calc.hex; the stream is the K inputs of the",
        "# demo circuit.xtl (12+34= typed at the calculator from reset), and",
        "# O and the R latches must match its trace at every clock.",
        "# baseline: demo-circuit 2-" + str(len(out_lines)),
    ]
    body = ir.lines + [f"next %{reg} %{nxt}" for reg, nxt in
                       [(a, a2), (y, y2), (x, x2), (s, s2), (pc, pc2), (pa, pa2), (pb, pb2),
                        (sr, sr2), (cl, cl2), (o, o2), (r, r2), (ram, ram2)]]
    body += ["trace %o", "trace %r"]
    text = "\n".join(head + body) + "\n"
    if "--check" in sys.argv:
        if not OUT.exists() or OUT.read_text() != text:
            print(f"FAIL: {OUT.relative_to(ROOT)} is stale (scripts/gen-hex1000-xir.py)")
            sys.exit(1)
        print(f"gen-hex1000-xir: {OUT.relative_to(ROOT)} is current")
    else:
        OUT.write_text(text)
        print(f"gen-hex1000-xir: wrote {OUT.relative_to(ROOT)} ({len(body)} lines)")


main()
