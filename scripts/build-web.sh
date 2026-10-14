#!/usr/bin/env bash
# The live site's interactive HEX1000 (web/hex1000): trunk builds the
# page, X_eTaL's engine running libs/Hex1000/demos/terminal.xtl, into
# work/web/calc, which scripts/build-pages.sh copies to pages/calc.
# Needs trunk and the wasm32-unknown-unknown target (never installed
# from here); without trunk it says so and does nothing.
#   scripts/build-web.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
command -v trunk >/dev/null || { echo "build-web: SKIPPED (trunk not installed: cargo install trunk)"; exit 0; }
"$root/scripts/xetal.sh" >/dev/null
"$root/scripts/xetal-libraries.sh"
cd "$root/web/hex1000"
trunk build --release --public-url /X_eTaL-fpga/calc/ --dist "$root/work/web/calc" >/dev/null
echo "build-web: work/web/calc ($(du -sh "$root/work/web/calc" | cut -f1))"
