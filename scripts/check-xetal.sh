#!/usr/bin/env bash
# Check the pinned X_eTaL: the CLI builds, answers, reports the commit
# in XETAL_COMMIT, runs a demo, imports a standard library, dumps the
# Core IR (the form the accelerator lowering will start from), and the
# libraries pinned from X_eTaL-libraries (Check, Bits, Strings) load.
#   scripts/check-xetal.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
xetal="$("$root/scripts/xetal.sh")"
short="$(cut -c1-7 "$root/XETAL_COMMIT")"
got="$("$xetal" eval -e "'+ r_/_2 2 3 r_eshape r_ange 6")"
[ "$got" = "6 15" ] || { echo "check-xetal: eval gave '$got', expected '6 15'" >&2; exit 1; }
"$xetal" --version | grep -q "$short" || { echo "check-xetal: xetal --version does not report $short" >&2; "$xetal" --version >&2; exit 1; }
"$xetal" run "$root/work/xetal/demos/life.xtl" >/dev/null
got="$("$xetal" eval -e '"s:" u_se< "Stats"
s:m_ean 1 2 3 4')"
[ "$got" = "2.5" ] || { echo "check-xetal: Stats gave '$got', expected '2.5'" >&2; exit 1; }
got="$("$xetal" core -e 'a := 1 2 3 4
2 * a' | tail -1)"
[ "$got" = "(eval (app2 #* 2 a))" ] || { echo "check-xetal: core gave '$got'" >&2; exit 1; }
"$root/scripts/xetal-libraries.sh"
got="$("$root/scripts/xt" eval -e '"k:" u_se< "Check"
"b:" u_se< "Bits"
6 k:i_s 2 * 3
12 b:x_or 10' | tr '\n' ' ')"
[ "$got" = "ok 6 " ] || { echo "check-xetal: Check and Bits (X_eTaL-libraries) gave '$got', expected 'ok 6'" >&2; exit 1; }
echo "check-xetal: ok ($short, libraries $(cut -c1-7 "$root/XETAL_LIBRARIES_COMMIT"))"
