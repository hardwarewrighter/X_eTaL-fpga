#!/usr/bin/env bash
# Check the published site (after just publish; GitHub Pages takes a
# minute or so to serve a new gh-pages commit): the landing page names
# the commit of main it was built from, and every library's
# documentation page, the documentation index and every Verilog
# module answer.
#   scripts/check-live.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
url="${XETAL_PAGES_URL:-https://hardwarewrighter.github.io/X_eTaL-fpga}"
cd "$root"
built="$(git log -1 --format=%s origin/gh-pages 2>/dev/null | sed -n 's/^pages: built from main \([0-9a-f]*\) .*/\1/p')"
page="$(curl -sf "$url/" || true)"
[ -n "$page" ] || { echo "check-live: $url/ does not load (is GitHub Pages enabled for the repository?)" >&2; exit 1; }
[ -n "$built" ] && ! grep -q "$built" <<<"$page" && { echo "check-live: $url/ is not the latest publish ($built)" >&2; exit 1; }
paths=(doc/index.html)
for n in $(scripts/libs.py list); do paths+=("doc/libs-$n-src-$n.xtl.html"); done
for v in libs/*/rtl/*.v; do paths+=("rtl/$(basename "$(dirname "$(dirname "$v")")")/$(basename "$v")"); done
fail=0
for p in "${paths[@]}"; do
  curl -sf -o /dev/null "$url/$p" || { echo "FAIL: live $p"; fail=1; }
done
[ "$fail" = 0 ] && echo "check-live: ok (${#paths[@]} pages at $url)"
exit "$fail"
