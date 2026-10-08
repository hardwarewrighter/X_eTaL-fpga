# Asks for X_eTaL

What this repository needs from X_eTaL (`../../softwarewrighter/X_eTaL`)
that X_eTaL does not have yet, and bugs it uncovers. This repo does not
change X_eTaL: each ask is filed here (and taken to X_eTaL), the work
here uses the workaround noted below or waits, and the workaround is
removed when the ask lands in the X_eTaL commit this repo pins
(`XETAL_COMMIT`, now d284a8c, X_eTaL main of 2026-10-08; every open
ask below was re-run against it that day and still holds).

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
| F5 | open | feature | **A seeded left fold, and a left scan of the states** (a state machine's run: `s0 'm:s_tep f_old inputs`, the state a vector, the inputs scalars). `r_/` is a right fold of one type (`a -> a -> a`); `p_ower` repeats a monadic function without an input per iteration; `e_ach` returns scalars only (M10); `s_\` is a right scan. Clocked hardware is exactly a left fold of a step function over a stream, so the model should be able to say it, and the lowering could recognize it as registers plus next-state logic. Planned in X_eTaL: Saga 18 (transducers), step 2, "the seeded left fold" (not scheduled); not blocking | the run harness of every state machine here (saga 2's examples, the 4-bit machine of saga 3) | a recursive harness (`r_un := { s xs -> 0 = t_ally xs ? s; (s l:s_tep f_irst xs) l:r_un 1 d_rop xs }`); recursion is not acceleratable, but only the step is lowered, the run is the testbench |
| F6 | open (planned upstream, not blocking) | feature | **Typed, shaped data files in and out** (narrowed 2026-10-07, `docs/calc-approaches.md` section 3). Input is covered today: literals, `@ i_nclude< "file"`, `n_umbers []N_GET "file"`, TOML `[]L_IST` and `[]T_ABLE`; output by `(f_ormat x) []N_PUT "file"`. What files lack is the type and the shape: `n_umbers` gives Floats and a flat vector, so a program restates the shape and floors to Ints, and a written file does not say what it holds. X_eTaL's planned `.xtln` (Saga 37: a data-only notation, a reader crate without the evaluator, a writer with `read (write x)` is `x`) is the answer; this repo would use it for test vectors, expected results, ROM images, weights (saga 5) and board session logs, and its Rust reader in the simulator and testbench generator | randomized equivalence tests at scale; saga 5's weights | text files: `[]N_GET` with the shape restated and `f_loor`; results compared as printed text, exactly (Ints only) |
| F7 | open | feature | **Target selection at the command line** (research7 section 11: `xetal build --target verilog FILE`, `xetal explain --device fpga FILE`), a compiler and build concern, not language semantics; a backend registered by a downstream crate (G6) | the one-command demo | `xetal-fpga verilog` takes the `.xir` twin; `xetal run` the `.xtl`; the equivalence script runs both |
| F8 | open | docs | **`e_ncode` of a negative number is unspecified**: at the pinned commit `2 2 2 2 e_ncode -1` is `1 1 1 1` and `-3` gives `1 1 0 1`, the two's complement residue, which is exactly what a hardware model wants, but `docs/reference.md` says only that "digits beyond the radix are dropped" and nothing about negative numbers, so a program cannot rely on it | `fx:b_its`, the twins' bit patterns | `libs/Fixed` wraps to the unsigned range first (`w fx:u_wrap x`, `x m_od 2^w`) and encodes that, so the behavior is not relied on |
| F9 | open | feature | **A function of three arguments cannot be applied in one line**: `x f y z` is `x f (y z)`, so an ALU's `op a_lu acc entry` or `w fx:a_dd x y` must be curried by a binding (`u:a_dd := fx:a_dd 16`, then `x u:a_dd y`) or take its operands as one vector; `(1 u:f_ 2)_ 3` works but does not read as the operation. Hardware models are full of three-input units (an ALU: op, a, b; a mux: select, a, b; a fixed-width op: width, a, b). A form such as APL's operand binding or a section `(w f_)` usable infix would do | `libs/Fixed` (the width-first functions are bound by the caller), `libs/Datapath` (the wraps written out inside the library), the 4-bit machine's ALU (saga 3) | bind the first argument to a name; or pass the operands as one vector |
| F10 | open (not blocking) | feature | **A Rust API for evaluating X_eTaL in process** (filed 2026-10-07, `docs/calc-approaches.md` section 4; shared with X_eTaL-ML's M5 and X_eTaL-gpu's G5): a plain value type without lifetimes (`Data`: a shape and Int, Float, Bool or Char items), and a `Session` that loads and type-checks a program or library once, then `call`s its functions with `Data` arguments (checked against the function's type per call), `get`s a top-level value and `eval`s an expression; deterministic (a seed), sandboxed (file access only when allowed), native and wasm32; `Data` designed with `.xtln`'s value model so a value can be logged and replayed. Today `xetal-play` runs whole programs and gives printed text, and the evaluator's `Value<'a>` is tied to the program | randomized equivalence of evaluator and simulated circuit in one process; the host bridge that drives a board and asks for expected results live; X_eTaL-gpu's runtime | a process per case and printed text, or the file round trip of F6 |
| F11 | open | bug (inefficiency) | **X_eTaL commits its generated site to `main`**: `pages/` (the live demo's HTML, its wasm, the `xetal doc` site) is built locally and committed, and GitHub Pages serves it from `main` (`.github/workflows/pages.yml`). Every rebuild adds a new copy of every changed file to the repository's history forever, so clones and the pack grow with each publish. Measured on 2026-10-08 at 175acf59: 85 commits touched `pages/`, 859 distinct file versions, 642.5 MiB uncompressed and 20.7 MiB packed, about 44% of the 46.6 MiB pack; the `xetal doc` site alone is 297 versions (0.5 MiB packed) and grows with every library edit. X_eTaL-demos and X_eTaL-libraries already purged the same history (their pre-purge bundles are 23.7 MB and 5.1 MB) and moved to a `gh-pages` branch replaced on every publish. Ask: build `pages/` into a gitignored directory and publish it as a single-commit `gh-pages` branch (or a Pages artifact from an Action), and consider purging the old `pages/` history | the sibling repos copy X_eTaL's process; this repo follows the demos' way instead (decided with the user) | `scripts/publish-pages.sh` here: `pages/` gitignored, `gh-pages` force-replaced on every publish, no history |
| F12 | open | bug | **`xetal doc --out` names a library's pages by its path as found, so an absolute `XETAL_PATH` puts the build machine's directories in page names and links**: `XETAL_PATH=$PWD/libs/Fixed/src xetal doc --out d libs/Datapath/src/Datapath.xtl` writes `Users-<user>-github-...-libs-Fixed-src-Fixed.xtl.html` for the imported Fixed (the same with a relative `XETAL_PATH` gives `libs-Fixed-src-Fixed.xtl.html`). A published site then names the author's home directory and differs from machine to machine. Ask: name pages relative to the working directory, or by library name for libraries found on the path | `scripts/doc-site.sh` | the site is built through `scripts/xt`, whose `XETAL_PATH` is relative; `doc-site.sh` fails if a page names the home directory |
| F13 | open | feature | **`f_ormat` of a tuple** (and through it the Check library's `k:i_s`): at the pin, `f_ormat (1, 2)` is "expected an array, found a tuple", though TU6 defines how a tuple prints; so X_eTaL-libraries' Check, whose `k:i_s` formats both sides of a failed check (`h:s_hown`), does not type-check for tuples: `(1, 2) k:i_s (1, 2)` fails the same way. Two owners: X_eTaL makes `f_ormat` take any value it can print (a tuple as TU6 prints it); then X_eTaL-libraries makes Check's `h:s_hown` ask only an array for its shape, so `k:i_s` takes tuples. This repository changes neither (`docs/tuple-plan.md`) | HEX1000's tests (tuple states, from saga 3 step 4) | `k:t_rue want m_atch got`: works on tuples, but a failure does not show the values |

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
[`components/xir/README.md`](../components/xir/README.md): the
provisional IR is the request, written as code and tests. For hardware the IR needs a
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
