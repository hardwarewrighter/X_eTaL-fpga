# X_eTaL-fpga tasks. Recipes call scripts/*.sh, which hold the logic and
# work without just too. `just` alone lists the recipes.

set positional-arguments

# List the recipes
default:
    @just --list

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

# Test everything: every library's baselines, the Rust components, the equivalence checks
test:
    scripts/test-libs.sh
    @[ -f components/Cargo.toml ] && (cd components && cargo test -q) || true
    @[ -x scripts/check-equiv.sh ] && scripts/check-equiv.sh || true

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
