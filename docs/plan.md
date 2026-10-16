# X_eTaL-fpga -- Implementation Plan

X_eTaL programs as hardware: the RTL generator, the schedule and the
synthesis flow that take a computation written in X_eTaL (the
eXperimental Extensible Typed Array Language, developed in
`../../softwarewrighter/X_eTaL`) to Verilog, to simulation, and to a
real FPGA (Sipeed's Tang boards, from the Tang Nano 1K up, through
the open Gowin flow).
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
                 Tang Nano 1K or 4K
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
| A4 | **The hardware models are libraries**: `libs/Fixed/` (fixed-width numbers), `libs/Datapath/` (alias `dp:`, the vector examples: add, saxpy, threshold, the reduction tree, the acceptance pipeline, every one at a stated width), the 4-bit machine of saga 3 (its state as a vector, its ROM program, a pure step function), later `libs/Neuron/` (a dense layer at i8). Each in X_eTaL-libraries' layout (`src/<Name>.xtl`, `tests/` with reg-rs baselines and `types.rgt` pinning the exports' types, `demos/*.xtl`, `docs/README.md`); the demos are the example programs that become circuits. | What becomes hardware must be ordinary, typed, tested X_eTaL; a library is the form X_eTaL has for that, and the same files can move to X_eTaL-libraries whole. |
| A5 | **A provisional IR, to be replaced.** The accelerator IR and the lowering from Core belong to X_eTaL (asks F1, F2) and do not exist yet. This repo carries `components/xir` (crate `xetal-fpga-xir`): research7's XIR (Input, Const, elementwise ops, Select, Map, Reduce) with the FPGA's additions (a bit width on every value, Slice and Concat, Lookup, Register, Rom, a state machine as a step function over a state value), a textual form that round-trips, and a reference interpreter. Each example's `.xtl` has a hand-lowered `.xir` twin beside it, and `scripts/check-equiv.sh` proves the twin's interpretation equals the evaluator's output. The text form is meant to converge with X_eTaL-gpu's provisional XIR (the GPU subset is this one without widths and state), so that one X_eTaL IR can replace both. | The hardware side (netlist, schedule, Verilog, simulation, the board) can be built and tested now, against the real evaluator, without inventing language semantics here or duplicating X_eTaL's compiler. |
| A6 | **A netlist between the IR and the Verilog**, simulated in Rust (`components/rtl`): the schedule turns an XIR program into a word-level netlist (adders, comparators, muxes, registers, memories, counters, wires of stated widths), the Verilog-2005 emitter prints that netlist, and a cycle-accurate simulator runs it. The simulator is the proof of the schedule (results and cycle counts) that needs no external tool; the printed Verilog is pinned as text. | No Verilog simulator (iverilog, Verilator) and no Yosys are installed on the development Mac; the gate must prove the circuit regardless. The netlist is also what `explain` draws. |
| A7 | **Our own tiny Verilog-2005 emitter, no HDL crate, no vendor primitives**: plain `assign`, `always @(posedge clk)`, `case`, `$readmemh`; Yosys does the synthesis. | research7: "writing our own tiny Verilog emitter is the experiment"; Yosys reads the synthesizable Verilog-2005 subset; plain Verilog reads on any simulator and any FPGA. |
| A8 | **Schedule is separate from semantics**: lanes (spatial, temporal, tiled), the reduction's shape (tree or accumulator), registers between stages, the clock, the board, the I/O (UART, buttons, LEDs) are data the schedule and the emitter read (`xetal-fpga verilog --lanes 16 FILE.xir`, later a TOML per board), never written into an `.xtl` or the IR. | research7 sections 5 and 11 and "How scheduling should work": the model does not change, only the schedule; it is the GPU repo's counterpart too (one `matmul`, two schedules). |
| A9 | **One tool, `xetal-fpga`** (`components/cli`): `check FILE.xir` (parse, types, widths, shapes), `run FILE.xir` (interpret; prints as `xetal` prints), `netlist FILE.xir [--lanes N]` (the scheduled netlist), `verilog FILE.xir [--lanes N]` (the Verilog), `sim FILE.xir [--lanes N]` (the netlist simulated: results and cycles), `explain FILE.xir` (what became what: adders, muxes, registers, cycles, for each schedule), later `synth`, `pnr`, `pack`, `flash` as thin wrappers over the Gowin flow. | One command per question; `explain` is the seed of research7's "visual hardware explanation" (H3). |
| A10 | **External tools are optional and loud**: the gate runs the Verilog through iverilog and the synthesis through Yosys when they are on `PATH` and prints that it skipped them when they are not; nothing in the gate requires a board. Installing them is the user's call (`brew install icarus-verilog yosys verilator`, done on the Mac 2026-10-14; nextpnr-himbaechel with Gowin, apycula and openFPGALoader on the Arch machine). | The same tests must pass on this Mac, on the Arch machine with the board, and in CI without either. |
| A11 | **Rust components in one workspace**, `components/` (`xir`, `rtl`, `cli`; crates `xetal-fpga-*`), each small, with its README, unit tests, `cargo fmt` and `clippy -D warnings` in the gate; `.cargo/config.toml` points every workspace at one `target/`. | The same component discipline as X_eTaL and X_eTaL-gpu. |
| A12 | **The boards are Sipeed's Tang family, the board is schedule data**: the user has the Tang Nano 1K, 4K, 9K and 20K, the Tang Primer 25K and other Tang Primer boards (README, Boards). A design is built for a board through a constraints file per board (clock, pins, LEDs, serial lines), so it stays buildable for several and the smallest that holds it is chosen when it is ready to program; a larger board is never spent on a design a smaller one holds. HEX1000 targets the **Tang Nano 1K** first (GW1NZ-1), the 4K if needed, nothing larger (decided with the user). The larger machines (the RCA CDP1802, the IBM 1130), the neural layer and the lane measurements go to the 9K, 20K, 25K and Primers. The flow is Yosys, nextpnr-himbaechel (Gowin), gowin_pack, openFPGALoader, on the Arch machine where the boards are; each board's support in the open flow is checked when a design first goes to it (`docs/boards.md`: the GW1N and GW2A boards are supported; the Tang Primer 25K's GW5A is not yet, so the 25K waits for it: the open flow only, never the Gowin IDE, decided with the user 2026-10-14). | The smallest project on the smallest board, every board in reach for the larger ones (decided 2026-10-07, widened to the whole family 2026-10-12). |
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
| a machine's `s_tep state input` (a pure function of a state vector and an input) | `reg` for each state field, the step as combinational next-state logic | `always @(posedge clk)` registers, a `case` decode, an ALU | 2 |
| its run (the step folded over inputs; ask F5) | `rom` of inputs, a counter | the testbench's clock, or a ROM and a program counter | 2 |
| `n s_elect table`, a lookup | `lookup %t %i` | a `case` ROM or `$readmemh` | 2 |
| `x '+ '* i_nner w`, `nr:d_ense` (an i8 dense layer, X0) | `matvec` | MAC lanes, weights in a ROM, an accumulator | 4 |

## Goals

The goals and their order (the user's, 2026-10-14) are in
[`docs/goals.md`](goals.md): HEX1000 on a Tang Nano 4K or 9K agreeing
with X_eTaL first; then serial-data matmul pipelines; then our own
soft CPU; the 1K is a long-term optimization.

## Parallel lanes (from 2026-10-14)

Two agents work this repository at once, as CLAUDE.md's "Working a
branch in parallel" describes: each lane's saga and step directories
carry its prefix, so `.agentrail/` merges without collisions.

| Lane | Where | Owns | Saga |
| ---- | ----- | ---- | ---- |
| `sw` | the Mac, on `main` | the X_eTaL libraries (`libs/`), the software HEX1000 and its live demo, the site (`scripts/*pages*`, `landing.py`), the IR and emitter (`components/xir`, `components/rtl`): the circuit of the machine | `sw-nibble-machine`, in `lanes/sw/.agentrail` (`agentrail --saga lanes/sw ...`) since the lanes share `main`: `.agentrail/` is the board lane's |
| `board` | the Arch Linux machine with the boards, on a `board/...` branch merged to `main` | the open FPGA flow on real boards: `xetal-fpga synth`, `pnr`, `pack`, `load` (`components/cli`), constraints files per board, `scripts/board-*.sh`, `docs/linux-nvidia.md`, `docs/boards.md`, the board half of `docs/hardware.md`; blink, the UART echo, the pinned circuits placed and routed | `board-fpga-board` (saga 4: flow, cell-sim, machine-board, round-trip, lanes-board) |

Shared files take a line each, never a rewrite: `CHANGES.md` (both
lanes add lines; a merge keeps both), `README.md` (each lane edits its
own section), this plan (each lane its own saga's section). A change
one lane needs in the other's files is asked for in its commit message
or a note here, not made. When the board lane needs the machine's
circuit (machine-board), it takes it from `main` once machine-rtl has
landed. Each lane pulls (rebasing its own commits) before it pushes.

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
| 4 | calc-lib | (retired 2026-10-07 with the user: the demo is the 4-bit machine of saga 3) `libs/Calc` (`calc:`): a four-function calculator as a state vector (accumulator, operand, pending operation, phase, display), `k_eys "12+34="` (characters to key codes), `d_ecode`, `a_lu`, `s_tep state key` (pure; the circuit's next-state function), `r_un keys` (the step over a key sequence; the harness), `d_isplay`; tests 2+3=5, 12+34=46, 7*8=56, 123-27=96, chained operations, 16-bit wrap; demos |
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

### Saga 2 -- fpga-state  [DONE]

Goal: sequential circuits from X_eTaL step functions: state, input
streams, lookups, ROM and RAM in the IR, the netlist and the Verilog,
and a serial line. What the 4-bit machine of saga 3 is built from.

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | registers (done) | XIR state (a state value with its init, the next-state binding), an input stream (one item per clock), `lookup` (a ROM when constant), `put` (a RAM's write port) and `cast`; the netlist's registers and memories; the simulator clocks them; Verilog; tests (a counter, an accumulator, a shift register, a small state machine) and a twin of Fixed's counter demo |
| 2 | uart (done) | a UART receiver and transmitter as step functions (clock and baud rate as schedule data), simulated end to end with a loopback |
| 3 | testbenches (done; the tools themselves not yet run) | self-checking Verilog testbenches for the sequential examples, iverilog and Verilator when present; a VCD per run |

Done (2026-10-07). What it showed: clocked programs (state, streams,
ROMs, a RAM, a UART) go from X_eTaL step functions to clocked circuits
and match the evaluator clock for clock; reading and writing fields
of a state vector at constant indexes must be wiring (the loopback
fell from 112 comparators to 11 when the schedule learned that).
Not shown in this saga: that any module passes iverilog, Verilator
or Yosys (no tools on this Mac then; shown in nibble-machine's
verilog-tools step, which found and fixed one emitter bug). Learned about X_eTaL: a
step with a parameter (the clocks per bit) is bound by applying it,
`ua:t_x 3`, and a bound function's name needs an underline after a
letter (`u:t_x`, not `u:l3_`).

### Saga 3 -- nibble-machine  [ACTIVE]

Goal: the first demo machine, decided with the user
(`docs/calc-approaches.md`): a 4-bit machine in the spirit of the
TMS1000, our own design of equivalent size (a 4-bit accumulator, a
small register file, 64 by 4-bit RAM, a ROM of about 1K 8-bit
instructions, keyboard inputs, display outputs), not a replica, with
our own ROM program that makes it a calculator: hexadecimal first,
then octal and decimal. Written in X_eTaL, checked against a pure
X_eTaL reference, then lowered and simulated, sized for the Tang
Nano 1K. The machine is HEX1000 (`docs/machine.md`, decided with the user).

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | isa (done) | `docs/machine.md`: HEX1000, the registers, memories, instruction set and encoding, the shift-register program counter, I/O (a terminal over the UART), sized against the Tang Nano 1K; decided with the user |
| 2 | radix-lib (done) | the reference: a pure X_eTaL library of digit-serial arithmetic in a radix (digit vectors in and out, `+ - *`, perhaps `/`), hexadecimal, octal and decimal; the specification for the calculator program |
| 3 | doc-site (done; inserted, decided with the user) | doc comments (`##`, `###`, `## >>` examples run as doc tests) in every library, `xetal doc --test` in the gate, every export required to have an example; `scripts/doc-site.sh` (`just doc`), `build-pages.sh` (a landing page, the documentation, the Verilog), `publish-pages.sh` (`gh-pages`, force-replaced, the X_eTaL-demos way), `check-live.sh`; asks F11 (X_eTaL's committed `pages/`) and F12 (absolute page names) |
| 4 | verilog-tools (done; inserted, decided with the user) | the tools on the Mac (`brew install icarus-verilog verilator yosys`) and in the gate: all 17 pinned modules pass the iverilog testbench, Verilator's lint and Yosys; the bug they found fixed (an output also traced was declared twice); casts change width explicitly; `just fit` (Yosys's Gowin synthesis, LUTs, flip-flops, ALUs, block RAM, multipliers against the Tang Nano 1K); `docs/hardware.md` (Verilog first, the checks from the evaluator to the bitstream, simulators, the board test plan, bricking) |
| 5 | machine-lib (done) | `libs/Hex1000` (`hx:`): the state as a 12-part tuple (A Y X S PC PA PB SR CL O R RAM), `s_tep` (one instruction, pure, a guarded line per instruction reading as the reference card), the shift-register PC (`n_extPc`, `Order`: all 64 visited), the decoder (`k_ind`, `d_isasm`), a run harness without recursion (`r_un`, `r_unKeys`, `t_race`: `p_ower` loops over tuples), a text assembler (labels, `PAGE`, the PC order, errors with the line); 58 per-instruction checks, small programs (a counter, two digits with the carry, keys scanned), demos; details settled in `docs/machine.md` (codes, pages, CALL and RETN, O); found: a key cannot be code 0 and K has too few codes, so the keyboard is a matrix through the R latches (open, for hex-calc) |
| 6 | loops (done; inserted, asked by the user) | Clocked's `r_un` and `t_race`, Radix's `r_un` and Datapath's `s_um` (a loop over the tree's levels) as `p_ower` loops over tuples, as Hex1000's: no recursion left in `libs/`, every baseline and pinned type unchanged; and the Hex1000 demos made readable (asked by the user): `l_isting` (a page in run order, disassembled), `t_able` (numbers under column names), `RegNames` |
| 7 | hex-calc (done) | `libs/Hex1000/rom/calc.asm`, the calculator in HEX1000's instructions (7 pages, about 330 instructions): the key matrix (R8 to R13 select columns, K the row; decided with the user), the display strobed on R0 to R7 with O's bit 4 as blank, the R15 ready handshake; the harness types at the machine through its ports only (`hx:t_ype`, `hx:c_alc`, `hx:c_ost`); 33 key sequences equal to `16 rx:c_alc`, including FFFFFFFF times FFFFFFFF, which found a 64-bit overflow in Radix's own multiply (fixed: two halves; checks added); `docs/calc.md` with the RAM map, the pages and the costs (at most 16406 instructions, 1.2 ms on a board) |
| 8 | parallel-handoff (done; the board lane's acknowledgment, docs/parallel-work.md) | |
| 9 | live-demo (done; inserted, asked by the user) | `libs/Hex1000/demos/terminal.xtl` (`just calc`: the calculator at a terminal, a key at a time through the key matrix, each key's cost; its scripted session a baseline through `terminal.keys`); `web/hex1000`, the same program in the browser around X_eTaL's engine (`just web`; works, slow: diagnosed with a per-key log and a native bench); a VHS tape (`just tape`); the landing page's HEX1000 section; ask F14 |
| 10 | machine-rtl (moved ahead of radixes: the board lane needs the circuit) | the machine hand-lowered to XIR (registers, RAM, ROM, decode), simulated with the ROM program, every key sequence reproduced; memory cells in the emitter (a computed-address RAM and the ROM as block RAM, not flip-flops and muxes); the Verilog pinned and run under iverilog with a bit-level UART testbench (key bytes in on the receive line, the display decoded from the transmit line) and under Verilator for long runs; the simulated circuit's UART bridged to a terminal (the sw-tos pattern: a stdin/stdout UART, a pty relay), so the live demo's session can run on the circuit too; `just fit` against the 1K (`docs/hardware.md`) |
| 11 | uart-glue (done; inserted, asked by the user) | the board's glue in X_eTaL (`hx:g_lue`: a 16-key queue, the key matrix and R15 handshake, the digit latches, the frame; `hx:b_oard`, `hx:s_erial`, `hx:w_ires`), 14 checks against `hx:c_alc`; the machine and glue lowered together (`demos/hex1000-board.xir`) and emitted (`rtl/hex1000-board.v`, module `hex1000_board`), equal clock for clock in the IR, the netlist simulator and iverilog; 2338 LUTs, 540 flip-flops. The board lane can replace its hand-written adapter logic with it |
| 12 | mockup (done; inserted, asked by the user) | the 3D calculator from one X_eTaL description: `libs/Panel` (the 7-segment font, the keys and display as boxes and strokes, the camera's projection and a click's key, a key pressed at the machine; 12 checks, every key found where it is drawn); `mockup/calculator.xtl`, interactive in X_eTaL-extensions' scene window (pinned in `XETAL_EXTENSIONS_COMMIT`, `just xetal-x`), its headless baseline in the gate; `mockup/video.xtl`, rendered by Blender through the Video library, refusing to render unless the display ends on `hx:c_alc`'s answer; on the live site |
| 13 | rtl-memories (done; inserted: the calculator on the 1K, decided with the user 2026-10-15) | memories in the emitter: a constant table read at a computed index as a block RAM (`$readmemh`, a registered read: Yosys maps it to one SPX9; the ROM measured at about 500 LUTs and 440 wide muxes as logic), the RAM as block or LUT RAM; the machine's two-clock schedule (fetch, then execute) as schedule data, the checks comparing the ports at each instruction's last clock; `just fit` before and after. Measured: HEX1000 from 2220 LUTs and 379 flip-flops to 438 LUTs, 81 flip-flops and 2 block RAMs (the instruction and the RAM word registered, read at the next state's address, traces unchanged; `--memories` schedule data; a range analysis proves the addresses in range); it fits the 1K. |
| 14 | rtl-narrow (inserted) | unsigned types in the IR (a nibble 4 bits, not i5), dead nodes removed, common subexpressions shared, constants folded; the lowering's one shared adder and operand muxes instead of an adder per instruction; the 1K's budget measured per lever; the goal: machine plus panel within 1152 LUTs |
| 15 | calc-rules (inserted, decided with the user 2026-10-15) | negatives stay two's complement; `=` twice clears to 0 (no Clear key, no reset button); the program clears its own RAM at reset (a block RAM is not cleared by the reset line, found in rtl-memories); overflow (a carry out of digit 7 on `+` or `*`) sets R14 and shows `OuErFLO` until the next key; Radix's specification first, then the program, the harness and tests |
| 16 | mockup-rules (inserted) | the mockup and its render updated for calc-rules (`==` clears, `OuErFLO`, negatives) |
| 17 | panel (inserted) | the physical calculator's panel, modeled in X_eTaL and emitted checked: the 4x4 hex pad and the 4-button operator board on the machine's key scan, debounced; the MAX7219 transmitter (SPI-style, no-decode, the hex font and `OuErFLO`) writing the 8-digit display on each frame; LEDs; the 1K top (the 4K as fallback); `docs/calculator-build.md` (wiring, pins, parts, power); the board lane builds, flashes and wires it |
| 18 | web-perf (inserted, decided with the user) | the browser HEX1000 faster: the engine in a Web Worker, typing budgets in chunks that skip once done, a cheaper step and harness; measured before and after; then perhaps the circuit simulated in the page |
| 19 | radixes | octal and decimal (a radix mode, the digit adjust as an instruction or in the program, an instruction-set decision made here); all three against the reference |

### Saga 4 -- fpga-board

Goal: the machine on a Tang Nano 1K (or 4K, chosen then). Needs the
tools (A10) and the boards on the Arch machine.

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | flow | `xetal-fpga synth`, `pnr`, `pack`, `load` as wrappers over Yosys, nextpnr-himbaechel, gowin_pack, openFPGALoader; a constraints file per board (1K, 4K) as schedule data; loading into SRAM by default (the flash only when asked); a guard refusing a constraints file that assigns a JTAG pin or drives a pin the board drives; a blinky from an X_eTaL counter, then the UART echo through a 3.3 V USB-serial adapter (`docs/hardware.md`) |
| 2 | cell-sim | a cell-level simulator in Rust: Yosys's JSON after `synth_gowin` (LUT1-LUT4, flip-flops, ALU, wide-LUT muxes, block RAM) simulated against the same traces; Yosys's mapping checked without an outside simulator |
| 3 | machine-board | the machine on the board, keys in and digits out over the serial line (or buttons and LEDs), driven from the host with the test key sequences; resources and timing recorded for 1K and 4K |
| 4 | round-trip | the bitstream unpacked by Apicula and simulated against the traces (placement, routing and packing checked; to confirm the unpacker's output first) |
| 5 | lanes-board | the vector datapath on a board at several lane counts; LUTs and registers against lanes tabulated (the space-time trade-off, measured) |

### Saga 5 -- fpga-nn

Goal: research7's F4 and X0, a tiny neural layer as hardware, the same
layer the GPU repo schedules as kernels.

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | neuron-lib | `libs/Neuron` (`nr:`): `y = relu(W x + b)` at i8 with an i32 accumulator, quantization as explicit wraps, X_eTaL-ML's weight layout; tests against X_eTaL-ML's NN at the same weights |
| 2 | matvec | XIR `matvec`; MAC lanes, weights in a ROM, an accumulator; the schedule's lanes and cycles; the layer simulated |
| 3 | layer-board | the layer on a Tang board (the smallest that holds it, likely the 9K); one input, the same decision on the evaluator, the GPU (X_eTaL-gpu) and the board |

### Saga 6 -- xetal-lowering (deferred: asks F1, F2, F3)

Goal: the one-command demo, `xetal build --target verilog demo.xtl`,
once X_eTaL has the accelerator IR and the lowering.

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | adopt-ir | `components/xir` becomes an adapter from X_eTaL's IR; the hand-written twins deleted; the equivalence script runs `xetal` for both sides |
| 2 | one-command | `xetal build --target verilog FILE.xtl` through the backend registration X_eTaL offers (ask F7), or `xetal-fpga verilog FILE.xtl` calling X_eTaL's lowering |
| 3 | explain | `xetal explain --device fpga FILE.xtl`: the program, the IR, the netlist, lanes and cycles, side by side (research7 section 10) |

### Later machines (on larger boards)

Higher priority than other calculators, decided with the user: the
RCA CDP1802 (COSMAC) and the IBM 1130, each larger and more complex
than the 4-bit machine, on the Tang Nano 9K or 20K, the Tang Primer
25K or another Tang Primer, whichever is the smallest that holds it.
Planned after the 4-bit machine has run on a board.

Linux acknowledgment: [parallel work](parallel-work.md) records file
ownership and integration. Start hardware bring-up on Nano 9K; defer
Nano 1K sizing. Linux owns board flow; Mac owns memory cells and machine RTL.


### Board lane progress (2026-10-14)

`board/flow` owns CLI board tools and Nano 9K constraints. Commands
synth/pnr/pack/load/detect are implemented with pin and port guards.
The OSS CAD Suite built a counter fixture to a bitstream. Next:
board adapters for X_eTaL blink/echo and physical SRAM tests; consume
Mac's machine RTL when it lands. Hardware validation is pending.
The branch-local archive of the inherited sw saga is an incomplete
copy, not completion of Mac's live software lane. During integration,
preserve main's live sw saga and merge the board history into its
archive; never replace the software cursor with the board cursor.


Board-first-setup was inserted before X_eTaL blink/echo at the user's
request. The fixture is simulated before synthesis; Linux USB probe
selection and raw serial handling have six no-hardware tests. The
attached Nano 9K now passes JTAG, SRAM load/CRC and 542-byte UART
loopback after the user installed the USB rule. The user confirmed LED blinking and button reset. Power-cycle
recovery is unchecked; X_eTaL tops remain next.

Board status update: persistent flash programming and power-cycle boot
now pass on 9K, including a 541-byte UART check without loading any
configuration after reconnection. X_eTaL board execution remains next.


Board-blink-echo now proves emitted X_eTaL arithmetic/UART on actual
9K: 512 counter queries and 255 echoes pass in SRAM and after flash
writing. The whole-top and mapped-cell simulations caught TX pin
glitches; a registered TX fixes them. A single serial session fixes
the query/echo phase transition. Next board-hex1000 consumes the
landed core and implements the R15/matrix and digit-strobe contract;
then board-demo-video records the real calculator session. A separate
counter/UART VHS tape records the current hardware proof.


Board-hex1000: complete adapter passes 27 MHz UART and 27/28 MHz
calculator timing, SRAM CRC and three physical calculations (46,
6E, 17FCD). Hardware video recorded and copied to the blog agent.
See docs/hex1000-hardware.md. Reconnect reset/reload workaround,
32-key queue, coalesced display frames and unverified calculator
flash boot are explicit remaining work.
