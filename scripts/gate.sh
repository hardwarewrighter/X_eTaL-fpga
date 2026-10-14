#!/usr/bin/env bash
# The pre-commit gate: the pinned X_eTaL and X_eTaL-libraries
# (scripts/check-xetal.sh; it clones and builds on a fresh checkout),
# the library tooling's self-test, every library's baselines and doc
# tests, the documentation site's build, the
# Rust components (format, clippy, tests), the equivalence checks (the
# evaluator against the XIR interpreter, the netlist simulator and,
# when iverilog and yosys are installed, the generated Verilog), the
# pinned Verilog, American
# spellings, then ASCII-only markdown for the docs we own.
#   scripts/gate.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
"$root/scripts/check-xetal.sh"
"$root/scripts/selftest-libs.sh"
python3 "$root/scripts/test-board-flow.py"
if [ "$(uname -s)" = Linux ]; then
  python3 "$root/scripts/test-board-first.py"
  python3 "$root/scripts/test-board-probe.py"
else
  echo "board-first: SKIPPED Linux USB/serial setup tests on $(uname -s)"
fi
"$root/scripts/test-libs.sh"
"$root/scripts/doctest-libs.sh"
# The documentation site builds (into work/, so pages/ is left alone).
XETAL_DOC_OUT="$root/work/doc" "$root/scripts/doc-site.sh"
if [ -f "$root/components/Cargo.toml" ]; then
  (cd "$root/components" && cargo fmt --all --check) || { echo "FAIL: cargo fmt (run: cd components && cargo fmt --all)"; exit 1; }
  (cd "$root/components" && cargo clippy -q --all-targets -- -D warnings) || { echo "FAIL: cargo clippy"; exit 1; }
  (cd "$root/components" && cargo test -q >/dev/null 2>&1) || { (cd "$root/components" && cargo test); echo "FAIL: cargo test"; exit 1; }
  echo "ok: components (fmt, clippy, tests)"
fi
python3 "$root/scripts/board-probe.py" check
python3 "$root/scripts/board-hex1000.py" check
[ -x "$root/scripts/check-equiv.sh" ] && "$root/scripts/check-equiv.sh"
[ -x "$root/scripts/check-rtl.sh" ] && "$root/scripts/check-rtl.sh"
# Gowin utilization (an estimate; fails only if a module will not synthesize).
"$root/scripts/fit.sh" > "$root/work/rtl/fit.txt" || { cat "$root/work/rtl/fit.txt"; exit 1; }
echo "fit: $(head -c 200 "$root/work/rtl/fit.txt" | grep -q SKIPPED && cat "$root/work/rtl/fit.txt" || echo "$(($(wc -l < "$root/work/rtl/fit.txt") - 2)) circuit(s) synthesized for GW1N (just fit; table in work/rtl/fit.txt)")"
# American spellings only (the checker checks itself first).
"$root/scripts/check-spelling.py" --self-test
"$root/scripts/check-spelling.py"
md=(README.md CHANGES.md docs/*.md)
for f in libs/*/README.md libs/*/docs/README.md components/*/README.md boards/*/README.md; do [ -e "$f" ] && md+=("$f"); done
for f in "${md[@]}"; do sw-markdown-checker -f "$f" >/dev/null || { sw-markdown-checker -f "$f"; exit 1; }; done
echo "gate: ok"
