# Asks for X_eTaL

What this repository needs from X_eTaL (`../../softwarewrighter/X_eTaL`)
that X_eTaL does not have yet, and bugs it uncovers. This repo does not
change X_eTaL: each ask is filed here (and taken to X_eTaL), the work
here uses the workaround noted below or waits, and the workaround is
removed when the ask lands in the X_eTaL commit this repo pins
(`XETAL_COMMIT`, now 75e6a5c, X_eTaL main of 2026-10-06).

Each entry: status (open, filed, landed, dropped), kind (feature, bug
or speed), what here needs it, why, a minimal repro or example, and
the workaround in use. The F numbers are this repository's; X_eTaL's
own ledger (`docs/asks.md`, generated from `docs/asks.toml`) may
number them differently, and X_eTaL-gpu's `docs/xetal-asks.md` files
the shared ones as G1 to G7 (named below), so one X_eTaL change can
close both.

The first four asks are the **blockers**: `docs/research7.txt`
describes an accelerator IR and a lowering that belong in X_eTaL
(research7: "the contract belongs to X_eTaL"), and as of 2026-10-06
neither is planned there (X_eTaL's `docs/plan.md` has no saga for it;
`docs/wish-list.md` lists "Parallel and GPU kernels" as an XL wish).
Until they land, this repository carries a provisional IR of its own,
`components/xir`, written to research7's description with the FPGA's
additions, and hand-lowers each example (`libs/*/demos/*.xir` beside
each `.xtl`), so that the hardware side can be built and tested now;
the provisional IR is to be replaced by X_eTaL's, not kept.

| # | Status | Kind | Ask | Needed by | Workaround |
| - | ------ | ---- | --- | --------- | ---------- |
| F1 | open (blocker) | feature | **An accelerator IR in X_eTaL** (research7 section 2, "What belongs in XIR", and "Step 2" of PoC 1; X_eTaL-gpu's G1): a typed dataflow DAG of Input, Const, elementwise ops (add, sub, mul, min, max, neg, abs, comparisons, and, or, xor, not, shifts by a constant), Select, Map/Zip with scalar extension, Reduce along an axis, Lookup, Concat and Slice; for hardware also a **bit width on every value** (i8, i16, i32, a 1-bit Bool) and the sequential forms **Register, Rom/Ram and a state machine** (a step function over a state value); values carry a scalar type, a width and a static shape; a textual form that round-trips; no knowledge of OpenCL, CUDA or Verilog | everything: this repo is a backend of that IR | `components/xir`, a provisional IR in this repo written to the same description (its text form is in `components/xir/README.md`), to be replaced; it is meant to converge with X_eTaL-gpu's provisional XIR (that subset without widths and state) |
| F2 | open (blocker) | feature | **Core to accelerator IR lowering** (research7 "accelerator extraction"; G2): given a typed Core program (or one top-level function), either the IR for the acceleratable region or a reason it is not acceleratable (recursion, dynamic shapes, I/O, nested arrays, general higher-order functions); for hardware, a pure step function `State -> Input -> State` folded over inputs is recognized as registers plus next-state logic, and a `w fx:w_rap` (or a sized type, F3) sets the width | every example: today each `.xtl` is lowered by hand into a `.xir` twin | hand-written `.xir` files, each checked for equivalence against the pinned evaluator's output of the `.xtl` (`scripts/check-equiv.sh`) |
| F3 | open (blocker) | feature | **Sized integer types and static shapes** (research7 section 12, "Types become surprisingly valuable": `i8[16]`, `i16[16]`; G3 and X_eTaL-ML's M12 asked for shapes): hardware needs each value's bit width and each array's extent at compile time; X_eTaL's Int is one 64-bit type and its types carry neither rank nor shape. Smallest useful form: a width ascription built-in or a sized type family (`Int8`, `Int16`, `Int32`, wrapping on overflow) whose arithmetic the evaluator performs at that width, plus argument shapes given to the lowering by the caller | the width of every adder, the extent of every lane; `libs/Fixed` is the stand-in | `libs/Fixed`: the width is a value in the program (`16 fx:w_rap x`, `16 fx:a_dd x y`) and the evaluator computes the wrapped result; the `.xir` twins state widths and shapes |
| F4 | open (blocker) | feature | **A stable Core IR dump** as the contract until F1 and F2 exist (G4): `xetal core FILE` prints s-expressions (`(let b (app2 #* 2 a))`), undocumented and unversioned; a documented, versioned form (JSON, or the text as is with a grammar) would let this repo prototype the lowering out of tree without waiting for F2 | a possible bridge (`xetal core` to `.xir`) while F2 is unplanned | none used: the bridge is not built (plan A5); the hand-written twins stand in |
| F5 | open | feature | **A left fold over items with an accumulator of another type** (a state machine's run: `s0 'calc:s_tep f_old keys`, state a vector, keys scalars). `r_/` is a right fold of one type (`a -> a -> a`); `p_ower` repeats a monadic function without an input per iteration; `e_ach` returns scalars only (M10); `s_\` is a right scan. Clocked hardware is exactly a left fold of a step function over a stream, so the model should be able to say it directly | `libs/Calc`'s `r_un` (the harness that drives `s_tep` through a key sequence) | a recursive function in the library (`r_un := { s keys -> 0 = t_ally keys ? s; (s l:s_tep f_irst keys) l:r_un 1 d_rop keys }`); recursion is not acceleratable, but only `s_tep` is lowered, the fold is the testbench |
| F6 | open | feature | **Bind arrays into a program and read results back without text** (X_eTaL-demos' ask, X_eTaL-ML's M5, G5): a testbench or a board bridge moves arrays into the evaluator and compares; today the only way is source text and printed output | the end-to-end `xetal build --target verilog` and board checks | the examples carry their data as literals; results are compared as printed text, exactly (Ints only) |
| F7 | open | feature | **Target selection at the command line** (research7 section 11: `xetal build --target verilog FILE`, `xetal explain --device fpga FILE`), a compiler and build concern, not language semantics; a backend registered by a downstream crate (G6) | the one-command demo | `xetal-fpga verilog` takes the `.xir` twin; `xetal run` the `.xtl`; the equivalence script runs both |
| F8 | open | docs | **`e_ncode` of a negative number is unspecified**: at the pinned commit `2 2 2 2 e_ncode -1` is `1 1 1 1` and `-3` gives `1 1 0 1`, the two's complement residue, which is exactly what a hardware model wants, but `docs/reference.md` says only that "digits beyond the radix are dropped" and nothing about negative numbers, so a program cannot rely on it | `fx:b_its`, the twins' bit patterns | `libs/Fixed` wraps to the unsigned range first (`w fx:u_wrap x`, `x m_od 2^w`) and encodes that, so the behavior is not relied on |

## Details

### F1 and F2: the IR and the lowering are X_eTaL's

research7 (the user's planning conversation, archival) places the
accelerator IR and the eligibility analysis in X_eTaL:

```
softwarewrighter/X_eTaL
        |  typed Core IR
        v
 accelerator representation/API
        |
        +-----------------------------+
        v                             v
softwarewrighter/X_eTaL-gpu    hardwarewrighter/X_eTaL-fpga
```

The IR "should know nothing about OpenCL, CUDA, Verilog, Yosys, Gowin,
Tang Nano, NVIDIA"; its job is "here is the computation", and the
downstream repositories answer "here is how this device realizes that
computation". What this repository needs from it, concretely, is in
`components/xir/README.md` once step 5 lands: the provisional IR is
the request, written as code and tests. For hardware the IR needs a
little more than the GPU subset (research7, PoC 1 step 2): widths,
Concat and Slice, Lookup, Register, ROM and RAM, and a state machine,
"that last group distinguishes FPGA compilation from ordinary array
acceleration". When X_eTaL's lands, the plan is: `components/xir`
becomes a thin adapter (X_eTaL's IR to this repo's netlist), the
hand-written `.xir` twins are deleted, and the equivalence script
runs `xetal` for both sides.

A suggested first shape of the lowering, from research7 ("Don't
compile all X_eTaL"): accept statically known scalar types and ranks,
pure functions, arithmetic, comparisons, scalar extension, `e_ach` and
elementwise built-ins, `r_/` reductions, soon `i_nner`; reject
recursion, dynamic allocation, nested arrays, dynamic shapes, I/O,
native extensions, general higher-order functions. The evaluator
stays the golden reference.

### F3: widths and shapes

An adder has a width; a lane has an extent. X_eTaL's types carry the
scalar type (`Int`, `Float`, `Bool`), not a width, a rank or a shape.
`libs/Fixed` is the stand-in: a program says `16 fx:a_dd x y`, the
evaluator gives the 16-bit two's complement sum (wrapped), and the
lowering reads the 16 as the adder's width. The smallest useful ask
is a sized type family whose arithmetic wraps (`Int16`, as Rust's
`i16` with `wrapping_add`), so the width is a type and the wrap is
not written at every operation; with it the Fixed library becomes a
shim and the twins' widths come from the types.

### F5: the left fold

A clocked circuit is `state' = step(state, input)` every cycle. In
X_eTaL today the model's `step` is a pure function and the run over
inputs is a recursive function, because `r_/` folds one type from the
right, `p_ower` repeats without an input, and `e_ach` returns only
scalars. A `f_old` (or a left scan whose items are the states) would
let the run be written as the circuit is, and would be the form the
lowering recognizes as a register bank.
