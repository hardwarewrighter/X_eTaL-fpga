# X_eTaL-fpga tasks. Recipes call scripts/*.sh, which hold the logic and
# work without just too. `just` alone lists the recipes.

set positional-arguments

# List the recipes
default:
    @just --list

# Check the board pin guards and command behavior without hardware
test-board-flow:
    python3 scripts/test-board-flow.py

# Test first-board discovery and serial I/O without a board (Linux)
test-board-first:
    python3 scripts/test-board-first.py

# Build a Nano 9K toolchain fixture without touching hardware
board-toolcheck: build
    bash scripts/board-toolcheck.sh

# Build the first Nano 9K test fixture; no USB access or programming
board-prepare:
    python3 scripts/board-first.py prepare

# Check Nano 9K USB permissions, JTAG detection and serial ports; read-only
board-check *args:
    python3 scripts/board-first.py check {{args}}

# Load the prepared Nano 9K SRAM fixture and verify UART byte loopback
board-test *args:
    python3 scripts/board-first.py test {{args}}

# Read the Nano 9K JTAG ID (no SRAM or flash write)
board-detect board="tangnano9k": build
    target/release/xetal-fpga detect --board "$1"

# Board tools: just board-flow synth FILE.v --top NAME --cst FILE.cst --out FILE.json
board-flow +args: build
    target/release/xetal-fpga {{args}}

# Get and build xetal at the known-good commit in XETAL_COMMIT (clone in work/xetal, binary bin/xetal)
xetal:
    @scripts/xetal.sh

# Pin a newer X_eTaL: a committed ref of ../../softwarewrighter/X_eTaL (default HEAD) into XETAL_COMMIT, built; then gate and commit it on its own
xetal-pin ref="HEAD":
    scripts/xetal-pin.sh "$1"

# The pinned X_eTaL: XETAL_COMMIT and the binary's own report
xetal-version:
    @cat XETAL_COMMIT
    @"$(scripts/xetal.sh)" --version

# Get the libraries used from X_eTaL-libraries at XETAL_LIBRARIES_COMMIT (Check, Bits, Strings; linked under work/libs/)
libs-get:
    @scripts/xetal-libraries.sh

# Pin a newer X_eTaL-libraries: a committed ref of ../../softwarewrighter/X_eTaL-libraries (default HEAD); then gate and commit it on its own
libs-pin ref="HEAD":
    scripts/xetal-libraries.sh --pin "$1"

# Evaluate an expression with the pinned xetal, every library on XETAL_PATH: just eval "16 fx:w_rap 40000"
eval expr:
    @scripts/xt eval -e "$1"

# The Core IR of an expression (what the accelerator lowering starts from): just core "2 * 1 2 3"
core expr:
    @scripts/xt core -e "$1"

# Check the pinned X_eTaL: CLI builds, answers, reports its commit, dumps Core; the pinned libraries load
check-xetal:
    scripts/check-xetal.sh

# The libraries: name, recommended alias, what it is
libs:
    @scripts/libs.py table | column -t -s "$(printf '\t')"

# The directories to put on XETAL_PATH: export XETAL_PATH="$(just path)"
path:
    @scripts/libs.py path

# Start a library from templates/Library: just new-lib Fixed fx: "fixed-width two's complement numbers"
new-lib name alias summary:
    scripts/new-lib.sh "$1" "$2" "$3"

# Run a library's test programs (or one): just run-lib Fixed basics
run-lib name prog="":
    @scripts/run-lib.sh "$1" ${2:+"$2"}

# Run a library's demos (or one): just demo-lib Datapath vector-add
demo-lib name prog="":
    @scripts/run-lib.sh --demos "$1" ${2:+"$2"}

# HEX1000 at the terminal: type at the calculator (q quits); the machine computes every digit
calc:
    @scripts/xt run libs/Hex1000/demos/terminal.xtl

# Build the live site's interactive HEX1000 (trunk): work/web/calc, copied into the site by just pages
web:
    scripts/build-web.sh

# Record HEX1000 at the terminal with vhs: work/tapes/hex1000-calc.gif (shown by the live site)
tape:
    @mkdir -p work/tapes && vhs docs/tapes/hex1000-calc.tape

# A library's demo as a notebook, each statement then its output
show-lib name prog="":
    @scripts/run-lib.sh --echo --demos "$1" ${2:+"$2"}

# A library's exported names and their types: just types Fixed
types name:
    @for f in libs/$1/src/$1.xtl libs/$1/src/$1.xtlm; do [ -f "$f" ] && scripts/xt type "$f"; done; true

# Test one library with reg-rs: pinned types, test programs, demos
test-lib name:
    scripts/test-libs.sh "$1"

# Create missing baselines and accept new output for one library (review the diff!)
bless-lib name:
    XETAL_BLESS=1 scripts/test-libs.sh "$1"

# Test everything: every library's baselines, the Rust components, the equivalence checks, the pinned Verilog
test:
    scripts/test-libs.sh
    scripts/doctest-libs.sh
    @[ -f components/Cargo.toml ] && (cd components && cargo test -q) || true
    @[ -x scripts/check-equiv.sh ] && scripts/check-equiv.sh || true
    @[ -x scripts/check-rtl.sh ] && scripts/check-rtl.sh || true

# Check an XIR program: each value's type: just xir-check libs/Datapath/demos/threshold.xir
xir-check file:
    @cd components && cargo run -q -p xetal-fpga-cli -- check "../$1"

# Run an XIR program with the reference interpreter: just xir-run libs/Datapath/demos/vector-add.xir
xir-run file:
    @cd components && cargo run -q -p xetal-fpga-cli -- run "../$1"

# The equivalence checks: every .xir twin against its demo's baseline (the evaluator)
equiv:
    scripts/check-equiv.sh

# The scheduled netlist of an XIR program (spatial, or N lanes): just netlist libs/Datapath/demos/vector-add.xir 2
netlist file lanes="":
    @cd components && cargo run -q -p xetal-fpga-cli -- netlist "../$1" ${2:+--lanes "$2"}

# The circuit as Verilog-2005: just verilog libs/Datapath/demos/reduce-sum.xir 1
verilog file lanes="":
    @cd components && cargo run -q -p xetal-fpga-cli -- verilog "../$1" ${2:+--lanes "$2"}

# A self-checking Verilog testbench for the circuit
testbench file lanes="":
    @cd components && cargo run -q -p xetal-fpga-cli -- testbench "../$1" ${2:+--lanes "$2"}

# The circuit simulated: its outputs, the clocks on stderr: just sim libs/Datapath/demos/pipeline.xir 3
sim file lanes="":
    @cd components && cargo run -q -p xetal-fpga-cli -- sim "../$1" ${2:+--lanes "$2"}

# What a schedule made: units by kind and width, registers, clocks: just explain libs/Datapath/demos/pipeline.xir 2
explain file lanes="":
    @cd components && cargo run -q -p xetal-fpga-cli -- explain "../$1" ${2:+--lanes "$2"}

# The pinned Verilog of every twin, and iverilog, verilator and yosys on it when installed
rtl:
    scripts/check-rtl.sh

# What each pinned circuit takes on a Gowin GW1N (yosys synth_gowin), against the Tang Nano 1K: just fit [NAME...]
fit *names:
    scripts/fit.sh {{names}}

# A waveform of a twin's circuit under iverilog: work/rtl/NAME.vcd (just wave libs/Clocked/demos/detect.xir)
wave file lanes="":
    scripts/wave.sh "$1" ${2:+"$2"}

# Rewrite the pinned Verilog from what xetal-fpga writes today (review the diff!)
bless-rtl:
    XETAL_BLESS=1 scripts/check-rtl.sh

# Run every library's doc tests: the ## >> examples in libs/*/src (xetal doc --test)
doctest:
    scripts/doctest-libs.sh

# Build the documentation site into pages/doc (xetal doc --out: libraries, demos, built-ins)
doc:
    scripts/doc-site.sh

# Build the live site into pages/ (not tracked): landing page, documentation, Verilog
pages:
    scripts/build-pages.sh

# Publish the site: build pages/ from this commit and make it the gh-pages branch's only commit (needs a clean work tree)
publish:
    scripts/publish-pages.sh

# Check the published site answers: landing page at the latest publish, every library's page, every module
check-live:
    scripts/check-live.sh

# Serve the built pages/ locally: http://127.0.0.1:8417/ (8417 is this repository's port)
serve-pages port="8417":
    cd pages && python3 -m http.server "$1" --bind 127.0.0.1

# Build the FPGA tool, xetal-fpga (components/cli), in release mode
build:
    cd components && cargo build -q --release -p xetal-fpga-cli

# The full pre-commit gate: pinned X_eTaL and libraries, tooling self-tests, baselines, components, equivalence, spelling, markdown
gate:
    scripts/gate.sh

# Show the agentrail saga state and the current step
status:
    agentrail status

# Open the saga plan
plan:
    @cat docs/plan.md
