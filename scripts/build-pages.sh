#!/usr/bin/env bash
# Build the live site into pages/ (gitignored, never on main; just
# publish makes it the gh-pages branch): a landing page, the library
# documentation (scripts/doc-site.sh, pages/doc), and the pinned
# Verilog of every circuit (pages/rtl). The landing page names the
# commit of main and of X_eTaL it was built from.
#   scripts/build-pages.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
rm -rf pages
mkdir -p pages/rtl
"$root/scripts/doc-site.sh" > /dev/null
for v in libs/*/rtl/*.v; do
  lib="$(basename "$(dirname "$(dirname "$v")")")"
  mkdir -p "pages/rtl/$lib"
  cp "$v" "pages/rtl/$lib/"
done
# The interactive HEX1000 (just web), when it has been built.
if [ -d work/web/calc ]; then cp -R work/web/calc pages/calc; fi
# The terminal recording (just tape), when it has been made.
if [ -f work/tapes/hex1000-calc.gif ]; then mkdir -p pages/media && cp work/tapes/hex1000-calc.gif pages/media/; fi
python3 "$root/scripts/landing.py" > pages/index.html
touch pages/.nojekyll
echo "pages: $(find pages -type f | wc -l | tr -d ' ') files ($(du -sh pages | cut -f1)) in pages/"
