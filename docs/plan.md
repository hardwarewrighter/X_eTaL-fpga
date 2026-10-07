# X_eTaL-fpga -- Implementation Plan

X_eTaL programs as hardware: the RTL generator, the schedule and the
synthesis flow that take a computation written in X_eTaL (the
eXperimental Extensible Typed Array Language, developed in
`../../softwarewrighter/X_eTaL`) to Verilog, to simulation, and to a
real FPGA (a Tang Nano 4K, GW1NSR-4C, through the open Gowin flow).
The goal is to demonstrate the concept, that one typed array program
runs unchanged as sequential software, as massively parallel software
(the sibling project, X_eTaL-gpu) and as spatial hardware, not to be
fast or to compete with an HLS compiler.

Source: `docs/research7.txt`, the user's planning conversation
(archival, NOT normative), which places the work in three
repositories with a strict division of responsibility:

| Repo | Owns |
| ---- | ---- |
| `softwarewrighter/X_eTaL` | the language, the typed Core IR, the accelerator IR and the lowering to it ("here is the computation") |
| `softwarewrighter/X_eTaL-gpu` | scheduling, OpenCL kernels, the host runtime: how an existing GPU realizes that computation |
| **`hardwarewrighter/X_eTaL-fpga`** (here) | **scheduling, Verilog, synthesis, the board: how new hardware realizes it** |

The distinction the user draws between the two organizations: the GPU
is software for hardware someone else designed; the FPGA work
*wrights* hardware from the array program. The X_eTaL part (the IR
and the lowering) is another agent's work in X_eTaL and, as of
2026-10-06, not planned there; the GPU part is a different agent's in
a different repository. This repository changes only itself. What it
needs from X_eTaL is in `docs/xetal-asks.md`, whose first entries
(F1 to F4) are the blockers.

Development is driven by agentrail sagas (one active saga in
`.agentrail/`, finished sagas archived to `.agentrail-archive/`), as in
the sibling repos. Every step ends with the gate (`just gate`), docs
updated (`README.md`, `CHANGES.md`, this plan, `docs/xetal-asks.md`),
a sane `.gitignore`, a detailed commit to `main` (with the
`.agentrail/` changes), `agentrail complete`, and a push.

## Guiding principle

> An X_eTaL array expression describes a spatial computation.

`c := a + b` on eight 16-bit numbers is eight adders. `'+ r_/ v` is
a tree of adders. The evaluator in X_eTaL is the golden reference;
this repository takes the same computation to a circuit and proves,
for every example, that the circuit gives the same answer, bit for
bit:

```
                X_eTaL program (.xtl)
                        |
      +---------+-------+--------+-----------+
      v         v                v           v
  evaluator  XIR interpreter  netlist sim  Verilog (iverilog)
      |         |                |           |
      +---------+-------+--------+-----------+
                        v
                  the same arrays
                        |
                  Yosys, nextpnr, gowin_pack, openFPGALoader
                        v
                   Tang Nano 4K
```

The interesting part is the **schedule**: the same `v_add` of 1024
elements can be 1024 adders (spatial), one adder reused 1024 times
(temporal) or 16 adders over 64 cycles (tiled); the same `s_um` can be
a tree or an accumulator. That space-time choice, made outside the
program, is what this repository exists to show (research7 sections 5
and 6: "How should an array expression become space, time, memory and
parallelism?").

## Architecture decisions

| # | Decision | Why |
| - | -------- | --- |
| A1 | X_eTaL is **pinned, not tracked** (`../../softwarewrighter/X_eTaL/docs/vendoring.md`, as X_eTaL-demos, X_eTaL-ML and X_eTaL-gpu do it): `XETAL_COMMIT` holds the known-good commit; `just xetal` (scripts/xetal.sh) clones X_eTaL into the gitignored `work/xetal/`, checks that commit out, builds the CLI into `target/xetal/` and links `bin/xetal`; `just xetal-pin [REF]` moves the pin to a committed ref, at a saga start or when an ask has landed, never mid-step, in its own commit after the gate passes. X_eTaL-libraries is pinned the same way (`XETAL_LIBRARIES_COMMIT`, scripts/xetal-libraries.sh, `just libs-pin`) for the libraries this repo uses rather than rewrites: Check (tests), Bits (bit vectors, and, or, xor, shifts) and Strings (Check's own import). | X_eTaL moves fast; this repo needs a recent but stable evaluator as its reference, changed deliberately, without carrying a copy of its source. The sibling repos (hardwarewrighter is a different organization) live under `../../softwarewrighter/`. |
| A2 | **The evaluator is the specification.** Every circuit exists first as an ordinary `.xtl` program (or a function of an `.xtl` library) that the pinned `xetal` runs; its printed output is a reg-rs baseline; every realization (the XIR interpreter, the netlist simulator, the Verilog in a simulator, the board) must reproduce it exactly. Only Ints in sagas 1 to 4 (hardware numbers are fixed-width integers); Floats and fixed-point are later. | research7: "X_eTaL becomes the specification"; "run exactly the same .xtl through the normal evaluator, an ancient GPU, and simulated Verilog; compare outputs bit-for-bit". |
| A3 | **Bit width is written in the program.** X_eTaL's Int is 64-bit and has no sized integer types (ask F3), while hardware is i8, i16, i32 (research7 section 12: "i8 + i8 is an 8-bit adder"). The `Fixed` library (alias `fx:`) makes the width part of the computation: `16 fx:w_rap x` is x in 16-bit two's complement, `16 fx:a_dd x y` is what a 16-bit adder gives, and the evaluator computes exactly that. The lowering reads the width from the wrap. When X_eTaL has sized types the wraps become types and the library a shim. | Hardware semantics (wrap-around, saturation, overflow) must be a tested fact of the program, not an accident of the Verilog; the evaluator can only be the reference if it computes the same 16-bit answer. |
| A4 | **The hardware models are libraries**: `libs/Fixed/` (fixed-width numbers), `libs/Datapath/` (alias `dp:`, the vector examples: add, saxpy, threshold, the reduction tree, the acceptance pipeline, every one at a stated width), `libs/Calc/` (alias `calc:`, the TMS1000-like calculator: registers as a state vector, decode, ALU, a step function, a run), later `libs/Neuron/` (a dense layer at i8). Each in X_eTaL-libraries' layout (`src/<Name>.xtl`, `tests/` with reg-rs baselines and `types.rgt` pinning the exports' types, `demos/*.xtl`, `docs/README.md`); the demos are the example programs that become circuits. | What becomes hardware must be ordinary, typed, tested X_eTaL; a library is the form X_eTaL has for that, and the same files can move to X_eTaL-libraries whole. |
| A5 | **A provisional IR, to be replaced.** The accelerator IR and the lowering from Core belong to X_eTaL (asks F1, F2) and do not exist yet. This repo carries `components/xir` (crate `xetal-fpga-xir`): research7's XIR (Input, Const, elementwise ops, Select, Map, Reduce) with the FPGA's additions (a bit width on every value, Slice and Concat, Lookup, Register, Rom, a state machine as a step function over a state value), a textual form that round-trips, and a reference interpreter. Each example's `.xtl` has a hand-lowered `.xir` twin beside it, and `scripts/check-equiv.sh` proves the twin's interpretation equals the evaluator's output. The text form is meant to converge with X_eTaL-gpu's provisional XIR (the GPU subset is this one without widths and state), so that one X_eTaL IR can replace both. | The hardware side (netlist, schedule, Verilog, simulation, the board) can be built and tested now, against the real evaluator, without inventing language semantics here or duplicating X_eTaL's compiler. |
| A6 | **A netlist between the IR and the Verilog**, simulated in Rust (`components/rtl`): the schedule turns an XIR program into a word-level netlist (adders, comparators, muxes, registers, memories, counters, wires of stated widths), the Verilog-2005 emitter prints that netlist, and a cycle-accurate simulator runs it. The simulator is the proof of the schedule (results and cycle counts) that needs no external tool; the printed Verilog is pinned as text. | No Verilog simulator (iverilog, Verilator) and no Yosys are installed on the development Mac; the gate must prove the circuit regardless. The netlist is also what `explain` draws. |
| A7 | **Our own tiny Verilog-2005 emitter, no HDL crate, no vendor primitives**: plain `assign`, `always @(posedge clk)`, `case`, `$readmemh`; Yosys does the synthesis. | research7: "writing our own tiny Verilog emitter is the experiment"; Yosys reads the synthesizable Verilog-2005 subset; plain Verilog reads on any simulator and any FPGA. |
| A8 | **Schedule is separate from semantics**: lanes (spatial, temporal, tiled), the reduction's shape (tree or accumulator), registers between stages, the clock, the board, the I/O (UART, buttons, LEDs) are data the schedule and the emitter read (`xetal-fpga verilog --lanes 16 FILE.xir`, later a TOML per board), never written into an `.xtl` or the IR. | research7 sections 5 and 11 and "How scheduling should work": the model does not change, only the schedule; it is the GPU repo's counterpart too (one `matmul`, two schedules). |
| A9 | **One tool, `xetal-fpga`** (`components/cli`): `check FILE.xir` (parse, types, widths, shapes), `run FILE.xir` (interpret; prints as `xetal` prints), `netlist FILE.xir [--lanes N]` (the scheduled netlist), `verilog FILE.xir [--lanes N]` (the Verilog), `sim FILE.xir [--lanes N]` (the netlist simulated: results and cycles), `explain FILE.xir` (what became what: adders, muxes, registers, cycles, for each schedule), later `synth`, `pnr`, `pack`, `flash` as thin wrappers over the Gowin flow. | One command per question; `explain` is the seed of research7's "visual hardware explanation" (H3). |
| A10 | **External tools are optional and loud**: the gate runs the Verilog through iverilog and the synthesis through Yosys when they are on `PATH` and prints that it skipped them when they are not; nothing in the gate requires a board. Installing them is the user's call (`brew install icarus-verilog yosys verilator`; nextpnr-himbaechel with Gowin, apycula and openFPGALoader on the Arch machine). | The same tests must pass on this Mac, on the Arch machine with the board, and in CI without either. |
| A11 | **Rust components in one workspace**, `components/` (`xir`, `rtl`, `cli`; crates `xetal-fpga-*`), each small, with its README, unit tests, `cargo fmt` and `clippy -D warnings` in the gate; `.cargo/config.toml` points every workspace at one `target/`. | The same component discipline as X_eTaL and X_eTaL-gpu. |
| A12 | **The board is the Tang Nano 4K** (GW1NSR-LV4CQN48PC7/I6, in Project Apicula and openFPGALoader as `tangnano4k`) through Yosys, nextpnr-himbaechel (Gowin), gowin_pack and openFPGALoader; the first physical program is a blinky, the second the calculator over UART (`12 + 34 =` in, `46` out); buttons and a display later. | research7 "What should actually be on the Tang": a fully supported open-source flow, a $20 board, the retro demo. |
| A13 | **Process as the siblings'**: `just` is the entry point (recipes call `scripts/*.sh`); `CHANGES.md` gets a line per commit; American spellings only (`scripts/check-spelling.py` in the gate); docs are ASCII-only markdown (`sw-markdown-checker`); a missing X_eTaL feature or bug goes in `docs/xetal-asks.md` and is not fixed or hidden here; work goes to `main` and is pushed. `.xtlm` macro libraries exist at the pinned X_eTaL (v0.1.0) but nothing in this plan needs one: widths are values, schedules are command-line data (A8). | Same process, same reviewers, same tools. |

## The subset, in X_eTaL, in XIR and in hardware

| X_eTaL | XIR | Hardware (Verilog) | Saga |
| ------ | --- | ------------------ | ---- |
| `w fx:w_rap x`, `w fx:a_dd x y`, `fx:s_ub`, `fx:m_ul`, `fx:n_eg` | a width on the value; `map add` at that width | an adder, subtractor, multiplier of w bits; wrap is free | 1 |
| `a + b`, `-`, `*`, `m_ax`, `m_in`, `a_bs` on arrays, with scalar extension | `map add %a %b` (a rank-0 operand broadcasts) | one unit per element (spatial), or one unit and a counter (temporal), or n lanes | 1 |
| `a > b`, `=`, `<`, `&`, `\|`, `n_ot`, masks in arithmetic | `map gt`, `map and`, ... (Bool as 1 bit) | comparators and gates | 1 |
| `(m * x) + (n_ot m) * y`, a select by mask | `select %m %x %y` | a mux | 1 |
| `b:a_nd`, `b:o_r`, `b:x_or`, `b:s_hl`, `b:s_hr` (X_eTaL-libraries' Bits) | `map and`, `shl` by a constant | gates, wires (a shift by a constant is wiring) | 1 |
| `'+ r_/ v`, `'m_ax r_/ v`, `dp:s_um` (a vector to a scalar) | `reduce add %v` | a balanced tree of adders (log2 n deep), or an accumulator over n cycles | 1 |
| `d_iv`, `m_od` by a power of two | `shr`, `slice` | wiring; other divisors are later | 1 |
| `calc:s_tep state key` (a pure function of a state vector and an input) | `reg` for each state field, the step as combinational next-state logic | `always @(posedge clk)` registers, a `case` decode, an ALU | 2 |
| `calc:r_un keys` (the step folded over inputs) | `rom` of inputs, a counter | the testbench's clock, or a ROM and a program counter | 2 |
| `n s_elect table`, a lookup | `lookup %t %i` | a `case` ROM or `$readmemh` | 2 |
| `x '+ '* i_nner w`, `nr:d_ense` (an i8 dense layer, X0) | `matvec` | MAC lanes, weights in a ROM, an accumulator | 4 |

## Roadmap

Implementable work first (the hardware side, against hand-lowered
examples); what needs an X_eTaL ask (the lowering, the one-command
demo) is in the last saga and moves up when the ask lands. What needs
tools or the board is in sagas 2 and 3 and is gated loudly (A10).

### Saga 1 -- fpga-foundation  [DONE]

Goal: the process, the pinned X_eTaL, the hardware models as tested
libraries (fixed-width numbers, the vector datapath, the calculator),
the provisional IR with an interpreter, the netlist and its simulator,
Verilog for combinational circuits with a lanes schedule and a
reduction tree, and the first proof: one X_eTaL program's result
reproduced by a simulated circuit, bit for bit.

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | scaffold | agentrail saga, CLAUDE.md/AGENTS.md, README, COPYRIGHT, LICENSE, `.gitignore`, justfile, `scripts/gate.sh`, X_eTaL and X_eTaL-libraries pinned (`XETAL_COMMIT`, `XETAL_LIBRARIES_COMMIT`, `scripts/xetal.sh`, `xetal-libraries.sh`, `check-xetal.sh`), the library tooling, this plan, `docs/xetal-asks.md` (the blockers), `CHANGES.md` |
| 2 | fixed-lib | `libs/Fixed` (`fx:`): `w_rap`, `u_wrap`, `a_dd`, `s_ub`, `m_ul`, `n_eg`, `s_at`, `o_verflow?`, `b_its`, `v_alue`, `h_ex` at a stated width, on whole arrays; tests against the arithmetic of 8- and 16-bit two's complement, pinned types, demos (overflow, a 16-bit counter wrapping), the reference page |
| 3 | datapath-lib | `libs/Datapath` (`dp:`): the vector examples at a width: `v_add`, `s_axpy`, `t_hreshold`, `k_eep`, `s_um` (as a reduction tree, shown equal to the sequential sum under wrap), `l_argest`, `d_ot`, `p_ipeline`; demos vector-add, saxpy, threshold, reduce-sum, pipeline (the example programs the circuits realize), with reg-rs baselines (the golden reference) |
| 4 | calc-lib | `libs/Calc` (`calc:`): a TMS1000-like four-function calculator as a state vector (accumulator, operand, pending operation, phase, display), `k_eys "12+34="` (characters to key codes), `d_ecode`, `a_lu`, `s_tep state key` (pure; the circuit's next-state function), `r_un keys` (the step over a key sequence; the harness), `d_isplay`; tests 2+3=5, 12+34=46, 7*8=56, 123-27=96, chained operations, 16-bit wrap; demos |
| 5 | xir | `components/xir`: the IR (widths, shapes, ops, select, reduce, slice, concat), its text form (parse and print, round-trip tested), checking, the reference interpreter; each Datapath demo's `.xir` twin; `scripts/check-equiv.sh` (twin against baseline, exact); `xetal-fpga check` and `run` |
| 6 | verilog | `components/rtl`: the netlist, the lanes schedule (spatial, temporal, tiled) and the reduction tree, the cycle simulator, the Verilog-2005 emitter; every twin's Verilog pinned as text and simulated for every schedule; iverilog and Yosys run when present (a self-checking testbench per example); `xetal-fpga netlist`, `verilog`, `sim`, `explain`; `scripts/check-equiv.sh` runs the netlist simulator (and iverilog when present) too |

Done (2026-10-07): every step as planned. What it showed: the
evaluator's answers for the five Datapath examples are reproduced
exactly by the hand-lowered IR and by the circuits scheduled from it,
spatial and with 1 to 9 lanes; the generated Verilog is pinned but
has not yet been through iverilog or Yosys on a machine that has
them (plan A10). Learned: X_eTaL's application is two-sided, so a
three-input unit is applied curried or with its operands as one
vector (ask F9); the temporal schedule takes single-length,
single-pass programs and says so for the rest.

### Saga 2 -- fpga-state  [ACTIVE]

Goal: sequential circuits. The calculator written in X_eTaL runs as
registers, a decoder and an ALU in simulated Verilog.

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | registers | XIR `reg`, `rom`, `lookup`, the state machine as a step function over a state value; the netlist's registers and memories; the simulator clocks them; Verilog `always @(posedge clk)`, `case` |
| 2 | calc-rtl | the Calc step function hand-lowered to XIR; the calculator as RTL: state registers, next-state logic, a key input and a display output; the key sequences of the tests fed by the testbench; every test's answer reproduced |
| 3 | uart | a UART receiver and transmitter in the netlist (schedule data: the baud rate, the clock), the calculator framed as `12 + 34 =` in and `46` out; simulated end to end |
| 4 | testbenches | a self-checking Verilog testbench per sequential example, iverilog and Verilator when present; a VCD per run for inspection |

### Saga 3 -- fpga-board

Goal: the Tang Nano 4K physically becomes the calculator. Needs the
tools (A10) and the board on the Arch machine; nothing here can be
tested on this Mac.

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | flow | `xetal-fpga synth`, `pnr`, `pack`, `flash` as wrappers over Yosys, nextpnr-himbaechel, gowin_pack, openFPGALoader; the board's constraints (`.cst`: clock, LEDs, UART pins) as schedule data; a blinky from an X_eTaL counter (`Fixed`'s wrapping counter demo) on the board |
| 2 | calc-board | the calculator on the board over UART, driven from the host with the test key sequences; resource and timing report recorded |
| 3 | lanes-board | the vector datapath on the board at several lane counts; LUT and register counts against lanes tabulated (the space-time trade-off, measured) |

### Saga 4 -- fpga-nn

Goal: research7's F4 and X0, a tiny neural layer as hardware, the same
layer the GPU repo schedules as kernels.

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | neuron-lib | `libs/Neuron` (`nr:`): `y = relu(W x + b)` at i8 with an i32 accumulator, quantization as explicit wraps, X_eTaL-ML's weight layout; tests against X_eTaL-ML's NN at the same weights |
| 2 | matvec | XIR `matvec`; MAC lanes, weights in a ROM, an accumulator; the schedule's lanes and cycles; the layer simulated |
| 3 | layer-board | the layer on the Tang Nano 4K; one input, the same decision on the evaluator, the GPU (X_eTaL-gpu) and the board |

### Saga 5 -- xetal-lowering (deferred: asks F1, F2, F3)

Goal: the one-command demo, `xetal build --target verilog demo.xtl`,
once X_eTaL has the accelerator IR and the lowering.

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | adopt-ir | `components/xir` becomes an adapter from X_eTaL's IR; the hand-written twins deleted; the equivalence script runs `xetal` for both sides |
| 2 | one-command | `xetal build --target verilog FILE.xtl` through the backend registration X_eTaL offers (ask F7), or `xetal-fpga verilog FILE.xtl` calling X_eTaL's lowering |
| 3 | explain | `xetal explain --device fpga FILE.xtl`: the program, the IR, the netlist, lanes and cycles, side by side (research7 section 10) |
