#!/usr/bin/env bash
# Build the documentation site (xetal doc --out) into pages/doc: every
# library (libs/*/src) and every demo program (libs/*/demos/*.xtl),
# with the built-ins, a search index and source views. It runs through
# scripts/xt, so the libraries are on XETAL_PATH by paths relative to
# the repository and the pages are named by those (an absolute path
# would name them after this machine's directories: ask F12). Run by
# scripts/build-pages.sh and by `just doc`.
#   scripts/doc-site.sh             # into pages/doc
#   XETAL_DOC_OUT=DIR scripts/doc-site.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
out="${XETAL_DOC_OUT:-pages/doc}"
rm -rf "$out"
files=(libs/*/src/*.xtl)
for f in libs/*/src/*.xtlm libs/*/demos/*.xtl; do [ -e "$f" ] && files+=("$f"); done
scripts/xt doc --out "$out" "${files[@]}" > /dev/null
if grep -rlq -- "$(basename "$HOME")" "$out"; then
  echo "doc-site: a page names this machine's home directory" >&2; exit 1
fi
echo "doc: $(find "$out" -name '*.html' | wc -l | tr -d ' ') pages in $out"
