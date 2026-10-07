# Calculators, state and data: approaches

A discussion document (2026-10-07), written for the user before saga 2
(fpga-state) starts. The user's decisions are recorded at the end
("Decided"); where they differ from a proposal above, the decision
wins. It covers four topics:

1. the two calculators, the one that exists and the one to demo;
2. how a clocked machine is written in X_eTaL today, and with the left
   fold and the transducers X_eTaL has planned;
3. getting arrays in and out of the evaluator: what works today, and
   what `.xtln` files would add;
4. a proposed Rust API for evaluating X_eTaL in process.

Each topic separates what can be done now from what would be better
with features X_eTaL has planned, and says whether waiting is worth
it. Nothing here is decided; the decisions for the user are at the
end. Facts about X_eTaL are as of the pinned commit, 75e6a5c, and its
`docs/plan.md`.

## 1. Two calculators

There are two different machines, and they should not be confused.

### Calc: the calculator as a specification (exists)

`libs/Calc` is a four-function calculator described by its behavior:
five 16-bit binary registers (accumulator, entry, pending operation,
phase, display) in one Int vector, and a pure step function from a
state and a key to the next state. Its ALU does `+ - * /` on whole
16-bit numbers in one step. It is not a model of any historical chip.

What it is good for:

- **A specification.** It says what a calculator should display for
  any key sequence; its 22 checks pin that behavior.
- **The first sequential circuit.** Lowered directly, its step becomes
  wired logic: five registers, a decoder, an adder, a multiplier and a
  divider, all in one clock. That is the simplest demonstration that a
  pure X_eTaL step function becomes registers plus next-state logic.

What it is not: small, or historical. A one-clock 16-bit multiplier
and divider are the opposite of how 1970s calculator chips worked, and
they are the largest parts of the circuit.

### TMS1000: the historical machine (the demo the user wants)

The TI TMS1000 is a 4-bit microcontroller: a small processor whose
calculator behavior is a program in its on-chip ROM. From memory, to be
checked against the datasheet before any step relies on it: a 4-bit
accumulator and Y register, a 2-bit X register, a 64 by 4-bit RAM
addressed by X and Y, a 1024 by 8-bit program ROM (16 pages of 64
instructions, a 6-bit program counter that steps as a shift register,
not a binary counter), one level of subroutine return, 4 K input lines
(the keyboard), R output lines (scanning the keyboard and display
digits) and O outputs through a programmable output table (segment
patterns). Its instructions are decoded by a PLA, itself programmable
per customer.

A calculator on it works digit by digit: numbers live in RAM as BCD
digits, one per 4-bit word, and the program adds them one digit per
instruction loop. Everything is 4 bits wide, so the hardware is tiny:
the whole machine plausibly fits a Tang Nano 1K, and comfortably a 4K.

In X_eTaL it would be a library, `libs/Tms1000` (or similar):

- the machine state as one Int vector (A, X, Y, PC, page, return
  registers, status, the R lines, and the 64 RAM words);
- the ROM as an Int vector of 1024 instructions, the output table as a
  lookup;
- `s_tep state k`: execute one instruction with the K inputs, pure;
- a calculator program written for it. TI's own calculator ROMs are
  mask contents owned by TI (dumps circulate for emulators); the
  safer course is to write our own program, in a small assembler (an
  X_eTaL function from instruction text to codes, or a short Rust tool).

### How the two fit together

The two are better together than apart: **Calc is the specification
and the TMS1000 is the implementation.** For every test key sequence,
the TMS1000 running our calculator program (in the X_eTaL evaluator,
then as a simulated circuit, then on the board) must show what
`calc:c_alc` shows, within the digit range of the TMS1000 program
(BCD digits rather than 16-bit binary: Calc would grow a decimal mode
or the comparison would be limited to small numbers). That is a
two-level equivalence: a behavior, a historical architecture running a
program, and a circuit, all agreeing.

Proposed order:

1. Saga 2 as planned: state, streams, lookups and ROMs in the IR and
   the netlist; Calc as the first sequential circuit (it is small to
   lower and checks the machinery).
2. A TMS1000 saga: the library and its instruction set, the assembler,
   the calculator program, checked against Calc in the evaluator; then
   hand-lowered and simulated; then on the board (Tang Nano 1K or 4K).
   The ROM and RAM are where saga 2's `rom` and register work pays off.

## 2. State machines and the fold

### Today

A clocked circuit is `state' = step(state, input)` every cycle. In
X_eTaL the step is an ordinary pure function (`calc:s_tep`), and that
is the part that becomes hardware. Running it over a key sequence is
the harness, and today it is recursive:

```
l:r_un := { s keys -> 0 = t_ally keys ? s; (s l:s_tep f_irst keys) l:r_un 1 d_rop keys }
```

This works, is tested, and costs nothing in hardware: the lowering
never sees `r_un`. It is slower in the evaluator for long streams (one
call and one `d_rop` per item) and reads less directly than the
circuit it models.

### With the seeded left fold (X_eTaL Saga 18, step 2)

X_eTaL's plan has the fold: Saga 18 (transducers) step 2 is "the
seeded left fold (as decided)", the first decision of that saga's
research step. It is not implemented at the pinned commit. With it the
harness would be one line, and the shape of the code would be the
shape of the circuit:

```
calc:Reset 'calc:s_tep f_old keys          # a hypothetical name: the final state
calc:Reset 'calc:s_tep f_olds keys         # a left scan: every state (the trace)
```

The lowering could then recognize `seed 'step f_old stream` directly:
the seed is the registers' reset value, the step the next-state logic,
the stream one input per clock. That is a cleaner contract for ask F2
than "lower this step function, and trust that something feeds it".

**Worth waiting?** No. Recursion carries saga 2 and the TMS1000 work;
the fold is a small, independent X_eTaL step that can land at any
time, and this repo switches when it pins a commit that has it. It is
worth asking for now (ask F5), with the left scan.

### With transducers (X_eTaL Saga 18, the library)

Transducers turn one reducing function into another: map, filter,
take, and their composition, independent of where the items come from
(an array, a file's lines, typed input) and in constant memory. In
hardware terms a transducer is a stream processor: a pipeline stage
between a source and a sink.

Where they would help here:

- **The UART framing** (saga 2 step 3): bytes in, decode to keys,
  run the calculator, format the display, bytes out is a transducer
  pipeline; written that way in X_eTaL, each stage is a circuit block
  with a valid signal.
- **Early termination** (Clojure's `reduced`, a question in X_eTaL's
  Saga 18) is a circuit's `done`.
- **Long streams** in the evaluator: a test sequence of thousands of
  keys without building the intermediate arrays.

**Worth waiting?** Not for saga 2: the UART can be written as a
step function too (a receiver is a small state machine). Worth revisiting
when X_eTaL's Saga 18 lands, as a later saga here ("stream circuits":
transducer pipelines as pipelined hardware). That would be a good
research topic in its own right.

## 3. Arrays in and out of the evaluator

### Today

More works than ask F6 says; F6 needs rewriting.

| Direction | How | Notes |
| --------- | --- | ----- |
| in, at compile time | literals; `@ i_nclude< "file"` (a file's text as a string, Rust's `include_str!`) | the text then needs `n_umbers` |
| in, at run time | `n_umbers []N_GET "file"` | numbers come back as Floats (`f_loor` makes Ints) and as a flat vector (the shape must be restated: `2 3 r_eshape ...`) |
| in, tables | `[]L_IST`, `[]T_ABLE` over TOML files | strings only, data only, nothing evaluated; suited to labels and names, not numeric arrays |
| out, at run time | `(f_ormat x) []N_PUT "file"` | text; a matrix writes as rows, but the shape and type are not recorded |
| out, by printing | the program's printed results | what the reg-rs baselines and `scripts/check-equiv.sh` compare today |

Checked at the pinned commit: a 2 by 3 matrix written with `[]N_PUT`
and read back with `f_loor n_umbers []N_GET` and `2 3 r_eshape` gives
the same matrix.

What this enables now, without waiting:

- **Randomized equivalence tests.** A generator (Rust, in this repo)
  writes `work/cases/NAME.txt` with random inputs; one fixed X_eTaL
  program per example reads it with `[]N_GET`, computes, and writes the
  results with `[]N_PUT`; the netlist simulator reads the same inputs;
  the two result files are compared. One program for any number of
  cases, no source generated per case.
- **Calc and TMS1000 key sessions** from files: a key text per line,
  read and run.

What it does not do well: types and shapes are not in the files (each
program restates them), Ints pass through Floats (exact up to 2^53,
fine for hardware widths), and every value is formatted and parsed as
text.

### With `.xtln` (X_eTaL Saga 37)

`.xtln` is X_eTaL notation: "data only, safe by construction", as
inert as JSON. The plan: a design step with the user, a reader crate
with no evaluator dependency (round-trip and fuzz tested), a writer
(`read (write x)` is `x`, `[]N_PUT`-style built-ins), a general reader
built-in, `[]L_IST` and `[]T_ABLE` over `.xtln` with TOML kept as an
input format.

What it would add here:

- **Typed, shaped values in both directions.** A file says `Int`, the
  shape and the items; no restating, no Float detour.
- **One format for every artifact.** Test vectors, expected results,
  the TMS1000's ROM image, the neural layer's weights (saga 4), board
  session logs.
- **The reader crate is usable from Rust without the evaluator.** The
  netlist simulator and the testbench generator could read the very
  file the X_eTaL program reads; the expected-results file written by
  the evaluator becomes the testbench's input.

**Worth waiting?** Not for saga 2 or the TMS1000: the ROM image and
key sessions are small and fit text today. Worth having before saga 4
(weights are large and typed) and before randomized testing at scale.
The design step is where this repo's needs should be heard: Int widths
(an `i16` annotation, or Int only and the width in the program as
today), bit-exact integers, and large flat arrays read fast.

## 4. A Rust API for evaluating in process

### What exists

X_eTaL's `xetal-play` crate (the live demo's engine, also native) has
`check(src)` and `run(src, seed)`, which run a whole program and give
its printed output as text; `Interactive` runs a program step by step
and feeds it typed lines. The evaluator's `Value<'a>` (crate
`xetal-value`) has a lifetime tied to the program (closures,
primitives), so it is not a type a downstream crate can hold or build
freely. X_eTaL-demos and X_eTaL-ML use `xetal-play` by path from the
`work/xetal` clone, and asked for arrays in and out without text
(X_eTaL-ML's M5); X_eTaL-gpu filed the same as its G5.

### The proposal

A small public crate in X_eTaL (say `xetal-embed`, or a module of
`xetal-play`) with a plain data type and a session that checks a
program once and calls its functions many times:

```rust
/// A plain value: no closures, no lifetimes; what crosses the boundary.
pub struct Data {
    pub shape: Vec<usize>,          // empty for a scalar
    pub items: Items,
}

pub enum Items {
    Int(Vec<i64>),
    Float(Vec<f64>),
    Bool(Vec<bool>),
    Char(Vec<char>),
    Boxed(Vec<Data>),               // a nested array, later
}

pub struct Options {
    pub seed: u64,
    pub path: Vec<PathBuf>,         // XETAL_PATH: where libraries are found
    pub files: bool,                // whether []N_GET and []N_PUT may touch the disk
}

pub struct Session { /* the loaded, checked program and its environment */ }

impl Session {
    /// Load and type-check a program or a library once.
    pub fn load(src: &str, opts: Options) -> Result<Session, Vec<Diagnostic>>;

    /// Call a function by name (`"calc:s_tep"` after an import in the
    /// source, or an exported `l:` name of a loaded library) with
    /// plain values; the arguments are type-checked against the
    /// function's inferred type before it runs.
    pub fn call(&mut self, func: &str, args: &[Data]) -> Result<Data, Diagnostic>;

    /// A top-level variable's value.
    pub fn get(&self, name: &str) -> Result<Data, Diagnostic>;

    /// Evaluate one more expression in the session (the REPL's way).
    pub fn eval(&mut self, expr: &str) -> Result<Data, Diagnostic>;
}
```

Design notes:

- **`call` rather than "bind a variable".** A program is type-checked
  as a whole; rebinding a name it was checked with would need a
  re-check. Calling a function with values is checked per call against
  the function's type (instantiated with the arguments' types), the
  same thing the evaluator does for an application, and needs no
  change to the language.
- **`Data` is the `.xtln` value model.** The two should be designed
  together: `Data` to and from `.xtln` text is the reader and writer of
  Saga 37, so a value can be logged, diffed and replayed.
- **Errors are the evaluator's diagnostics**, with codes, so a caller
  can tell a type error from a run-time `error[domain]`.
- **Deterministic** (the seed) and **sandboxed** (no file access unless
  asked), so tests are repeatable and a host can call untrusted code.
- **Native and wasm32**, as `xetal-play` is, so the browser demos get
  it too.

### What it would do here

- **Randomized equivalence at scale:** a Rust test calls
  `dp:p_ipeline` (or `calc:s_tep`) through the session on thousands of
  random inputs and compares each with the netlist simulator's result,
  in one process, in milliseconds. Today that is a process and a text
  parse per case.
- **The board bridge:** the host program that types keys into the Tang
  over the UART asks the session for the expected display of each key,
  live.
- **The GPU repo's runtime:** arrays from the evaluator to device
  buffers and back, without text (its G5).

**Worth waiting?** Nothing in saga 2 or the TMS1000 needs it: the file
round trip of section 3 does the randomized tests, slower. It is the
right next ask for X_eTaL, shared by four repositories, and small
compared with the language work: the evaluator already has the
values; the crate is a boundary around them.

## Summary

| Topic | Now | Ideal | Wait? |
| ----- | --- | ----- | ----- |
| The calculator demo | Calc as specification and first circuit | the TMS1000 running our calculator program, checked against Calc, on a Tang Nano 1K or 4K | no: plan a TMS1000 saga after saga 2 |
| Running a step over inputs | a recursive harness; the step is what is lowered | `seed 'step f_old stream`, recognized by the lowering as registers plus logic | no; ask for the fold and a left scan now |
| Stream pipelines (UART) | step functions | transducers as pipelined circuits | no; a later research saga |
| Data into and out of programs | `i_nclude<`, `[]N_GET`, `[]N_PUT`, TOML tables; text, shapes restated | `.xtln`: typed, shaped, one format, a Rust reader | not for saga 2; wanted before saga 4 |
| Calling X_eTaL from Rust | run whole programs, parse printed text | a `Session` with `call` and plain `Data` values | no; the next ask, shared with three repos |

## Decisions for the user

1. **The calculators.** Keep Calc as the specification and first
   sequential circuit, and add a TMS1000 saga as the demo? Or move
   straight to the TMS1000 and keep Calc only as the reference?
2. **The TMS1000 program.** Write our own calculator program for it
   (recommended), or use a historical ROM image for authenticity
   (ownership to be checked)?
3. **Calc's number range.** Give Calc a BCD or decimal-digit mode so
   it matches the TMS1000 digit for digit, or compare only within the
   range where both agree?
4. **The asks.** Rewrite F6 as this document describes (input is
   covered; typed output, `.xtln` and the `Session` API remain), and
   file the `Session` API as its own ask to X_eTaL, cross-referenced
   with X_eTaL-ML M5 and X_eTaL-gpu G5?
5. **Boards.** Plan for the Tang Nano 1K and 4K for the TMS1000, and
   the 9K or 20K if the Calc circuit or the neural layer needs room,
   with the board as schedule data (a constraints file per board)?

## Decided (2026-10-07, with the user)

1. **The TMS1000 only.** The demo is a TMS1000-class machine; Calc is
   not the demo, and `libs/Calc` is retired (removed; it remains in
   git history). The machine's reference is the radix library of
   saga 3. Other calculators may
   come later, but the higher-priority larger machines are the RCA
   CDP1802 and the IBM 1130, on larger Tang Nano boards.
2. **Not a replica: a machine in the spirit of the TMS1000.** We design
   our own 4-bit machine of equivalent size and complexity (a 4-bit
   accumulator, a small register file, a 64 by 4-bit RAM, a ROM of
   about 1K 8-bit instructions, keyboard inputs, display outputs),
   with our own instruction set and our own ROM program, testable at
   every level and doing something calculator-like. No TI ROM image
   and no claim of compatibility. Working name for now: the 4-bit
   calculator machine (a name to be chosen with the user).
3. **Radix: hexadecimal first, then octal and decimal.** The plan is
   simplest first:
   - **Hexadecimal** is the simplest because one 4-bit RAM word is
     exactly one digit: the adder's carry out of bit 3 is the digit
     carry, so digit-serial addition needs no adjustment at all.
   - **Octal** uses three bits of each word; the carry is at 8, so a
     digit add needs a compare (or the carry out of bit 2).
   - **Decimal** is BCD, as real calculators were: a digit add needs
     the decimal adjust (add 6 when the sum is past 9), done by the
     program or by one instruction.
   - **All three** then cost one more thing: a radix the program reads
     (a RAM word set by a mode key), with the adjust done by a
     "digit adjust" instruction that takes the radix, or by the
     program with a compare. Whether that instruction exists is an
     instruction-set decision, made when octal is added.
   The reference is a pure X_eTaL function per radix: digit vectors in,
   digit vectors out (`e_ncode` and `d_ecode` do the conversions), the
   specification the machine's ROM program must match for every test
   key sequence.
5. **Board: the Tang Nano 1K first** (GW1NZ-LV1: from memory about
   1152 4-input LUTs, 864 flip-flops and 72 Kbit of block RAM, to be
   checked against Gowin's datasheet and Apicula's device support),
   with the choice of 1K or 4K made when the design is ready to
   program; the board is schedule data (a constraints file per
   board), so both stay buildable. A 9K or larger board is not spent
   on this design.

4. **The asks.** F6 narrowed to typed, shaped data files (input is
   covered today; `.xtln` is the answer, planned upstream, not
   blocking), and the `Session` API filed as F10 (not blocking),
   cross-referenced with X_eTaL-ML's M5 and X_eTaL-gpu's G5.

Sections 1 to 4 above are the discussion as it was; where they speak
of Calc as a specification, read the radix reference library of saga
3 instead.
