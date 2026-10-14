#!/usr/bin/env python3
"""The live site's landing page (scripts/build-pages.sh): what this
repository is, HEX1000 (its calculator's screens, from running the
calc demo at build time), its libraries (each linked to its documentation page),
the circuits (each pinned Verilog module), the project documents (on
GitHub), and the commits it was built from. Printed to stdout."""
import html
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REPO = "https://github.com/hardwarewrighter/X_eTaL-fpga"


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True).stdout.strip()


def libs():
    out = subprocess.run([str(ROOT / "scripts/libs.py"), "table"], capture_output=True, text=True).stdout
    for line in out.splitlines():
        name, alias, summary = line.split("\t")
        yield name, alias, summary


def circuits():
    for v in sorted(ROOT.glob("libs/*/rtl/*.v")):
        lib = v.parent.parent.name
        text = v.read_text()
        sched = re.search(r"^// (Spatial|Temporal|Clocked)[^\n]*", text, re.M)
        yield lib, v.name, sched.group(0)[3:] if sched else ""


def sessions():
    """The calc demo's key sequences and the screens HEX1000 shows for
    them, run now with the pinned xetal (so the page shows the
    machine's real output), and the two costs it ends with."""
    demo = ROOT / "libs/Hex1000/demos/calc.xtl"
    keys = re.findall(r'^hx:c_alc "([^"]*)"', demo.read_text(), re.M)
    out = subprocess.run([str(ROOT / "scripts/xt"), "run", str(demo)], cwd=ROOT,
                         capture_output=True, text=True, check=True).stdout.splitlines()
    return list(zip(keys, out)), out[len(keys):]


head = git("rev-parse", "--short", "HEAD")
xetal = (ROOT / "XETAL_COMMIT").read_text().strip()[:7]
e = html.escape
screens, costs = sessions()
try_web = ('<strong><a href="calc/">Try HEX1000 in your browser</a></strong>: the same program, X_eTaL running in the page. '
           if (ROOT / "pages/calc/index.html").exists() else "")
# The terminal recording, when scripts/build-pages.sh copied one in.
tape = ('<p><img src="media/hex1000-calc.gif" alt="HEX1000 at a terminal: 12+34= shows 46, C0DE+BEEF= shows 17FCD, A*B= shows 6E, FFFFFFFF*FFFFFFFF= shows 1" '
        'style="max-width:100%;height:auto;border-radius:6px"></p>'
        if (ROOT / "pages/media/hex1000-calc.gif").exists() else "")
rows_calc = "\n".join(f"<tr><td><code>{e(k)}</code></td><td><pre class=\"lcd\">{e(v)}</pre></td></tr>" for k, v in screens)
rows_libs = "\n".join(
    f'<tr><td><a href="doc/libs-{e(n)}-src-{e(n)}.xtl.html">{e(n)}</a></td><td><code>{e(a)}</code></td><td>{e(s)}</td></tr>'
    for n, a, s in libs())
rows_rtl = "\n".join(
    f'<tr><td>{e(lib)}</td><td><a href="rtl/{e(lib)}/{e(name)}">{e(name)}</a></td><td>{e(s)}</td></tr>'
    for lib, name, s in circuits())
docs = [("docs/plan.md", "the plan: architecture decisions and the roadmap"),
        ("docs/machine.md", "HEX1000, the 4-bit machine"),
        ("libs/Hex1000/docs/calc.md", "HEX1000's calculator program: its ports, RAM, pages and costs"),
        ("docs/hardware.md", "Verilog, the checks from the evaluator to the bitstream, testing on a board"),
        ("docs/boards.md", "the Tang boards: families, resources, software"),
        ("docs/calc-approaches.md", "calculators, state and data: the discussion and its decisions"),
        ("docs/xetal-asks.md", "what this repository needs from X_eTaL"),
        ("CHANGES.md", "every commit")]
rows_docs = "\n".join(f'<li><a href="{REPO}/blob/main/{p}">{e(p)}</a>: {e(w)}</li>' for p, w in docs)

print(f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>X_eTaL-fpga</title>
<style>
:root {{ --bg: #fbfaf7; --fg: #1d1d1b; --muted: #5d5b55; --rule: #dedbd2; --link: #0b5cad; --code: #f0ede5; }}
@media (prefers-color-scheme: dark) {{
  :root {{ --bg: #171716; --fg: #e8e6e1; --muted: #a8a49b; --rule: #34332f; --link: #7fb3ec; --code: #24231f; }}
}}
body {{ background: var(--bg); color: var(--fg); font: 16px/1.55 system-ui, sans-serif; margin: 0; }}
main {{ max-width: 60rem; margin: 0 auto; padding: 2rem 16px 4rem; }}
h1 {{ margin: 0 0 .25rem; font-size: 2rem; }}
h2 {{ margin-top: 2.5rem; border-bottom: 1px solid var(--rule); padding-bottom: .25rem; }}
p.lede {{ color: var(--muted); margin-top: 0; }}
a {{ color: var(--link); }}
code {{ background: var(--code); padding: 0 .25em; border-radius: 3px; }}
table {{ border-collapse: collapse; width: 100%; display: block; overflow-x: auto; }}
th, td {{ text-align: left; padding: .35rem .75rem .35rem 0; border-bottom: 1px solid var(--rule); vertical-align: top; }}
pre {{ background: var(--code); padding: .75rem; overflow-x: auto; }}
pre.lcd {{ margin: 0; padding: .2rem .6rem; display: inline-block; background: #20301f; color: #9cf59a; font-size: 1.15rem; letter-spacing: .08em; border-radius: 4px; }}
footer {{ color: var(--muted); font-size: .9rem; margin-top: 3rem; }}
</style>
</head>
<body>
<main>
<h1>X_eTaL-fpga</h1>
<p class="lede">X_eTaL programs as hardware: a typed array program becomes a circuit, simulated and checked bit for bit against the X_eTaL evaluator, and on its way to a Tang Nano.</p>
<p><code>c := a + b</code> on eight 16-bit numbers is eight adders; <code>'+ r_/ v</code> is a tree of adders; a step function over a state vector is registers and the logic between them. Every example here is an X_eTaL program first. Its output is the reference, and the intermediate form, the simulated circuit and the generated Verilog must all reproduce it. The first machine, HEX1000, is a 4-bit calculator in the spirit of the TI TMS1000, sized for a Tang Nano 1K; larger machines go to the Tang Nano 9K and 20K and the Tang Primer boards.</p>
<pre>
            X_eTaL program (.xtl)
                    |
   +----------------+----------------+
   v                v                v
evaluator    intermediate form   simulated circuit, Verilog
   |                |                |
   +----------------+----------------+
                    v
             the same answers
</pre>

<h2>HEX1000</h2>
<p>A 4-bit machine of our own design in the spirit of the TMS1000: a 4-bit accumulator, 64 words of 4-bit RAM, a ROM of 1024 instructions, a program counter that steps as a shift register (<a href="{REPO}/blob/main/docs/machine.md">docs/machine.md</a>). It is written in X_eTaL (<a href="doc/libs-Hex1000-src-Hex1000.xtl.html">the Hex1000 library</a>: one instruction is a pure step function of the machine's state), and its ROM holds a program in its own instructions (<a href="{REPO}/blob/main/libs/Hex1000/rom/calc.asm">rom/calc.asm</a>, about 330 of them) that makes it an eight-digit hexadecimal calculator. Keys go in through its key matrix and the digits come out on its output latches, nothing else computing; every screen is checked against the specification, the <a href="doc/libs-Radix-src-Radix.xtl.html">Radix</a> library. The screens below were made by running it while this page was built:</p>
{tape}
<table>
<tr><th>Keys typed</th><th>Screen</th></tr>
{rows_calc}
</table>
<p>12+34= takes the machine {e(costs[0])} instructions, FFFFFFFF*FFFFFFFF= {e(costs[1])}: on a Tang Nano at 13.5 million instructions a second, about a millisecond. {try_web}Or at a terminal: clone the repository and run <code>just calc</code> (keys 0-9 A-F + - * = X, q to quit); <code>just demo-lib Hex1000 listing</code> shows its ROM, page by page. Next: the machine as a circuit, then on a Tang Nano 1K.</p>

<h2>Libraries</h2>
<p>Each links to its documentation, generated by <code>xetal doc</code> from the <code>##</code> comments in its source; every <code>## &gt;&gt;</code> example there is run as a test. The <a href="doc/index.html">documentation index</a> also has the demo programs, a search, the built-ins, and each source file.</p>
<table>
<tr><th>Library</th><th>Alias</th><th>What</th></tr>
{rows_libs}
</table>

<h2>Circuits</h2>
<p>The Verilog-2005 of every example circuit, as pinned in the repository. Each is checked by a simulator of the same netlist against the X_eTaL evaluator, and every module passes its self-checking testbench under iverilog, Verilator's lint and Yosys's synthesis.</p>
<table>
<tr><th>Library</th><th>Module</th><th>Schedule</th></tr>
{rows_rtl}
</table>

<h2>Documents</h2>
<ul>
{rows_docs}
</ul>

<footer>Built from <a href="{REPO}/commit/{head}">{head}</a> with X_eTaL {xetal}. Source: <a href="{REPO}">{REPO}</a>. Copyright (c) 2026 Michael A Wright, MIT license.</footer>
</main>
</body>
</html>""")
