#!/usr/bin/env bash
# Get xetal-x, X_eTaL-extensions' bridge host, as scripts/xetal.sh gets
# xetal (the X_eTaL-gpu way): clone X_eTaL-extensions into
# work/xetal-extensions (gitignored), check out the known-good commit in
# XETAL_EXTENSIONS_COMMIT, let it get its own X_eTaL (its
# scripts/xetal.sh, at the commit it pins), build xetal-x and the
# extensions this repository uses (scene: an interactive 3D window;
# blender: renders) into target/xetal-extensions/release/, write
# bin/xetal-x, and print that path. Only the 3D mockup (mockup/) needs
# it; nothing else in the gate does. Safe to run at any time.
#   scripts/xetal-extensions.sh
#   scripts/xetal-extensions.sh --pin [REF]   # pin a committed ref of ../../softwarewrighter/X_eTaL-extensions (default HEAD)
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
clone="$root/work/xetal-extensions"
source="${XETAL_EXTENSIONS_SOURCE:-https://github.com/softwarewrighter/X_eTaL-extensions.git}"
if [ "${1:-}" = --pin ]; then
    repo="${XETAL_EXTENSIONS_REPO:-$root/../../softwarewrighter/X_eTaL-extensions}"
    sha="$(git -C "$repo" rev-parse --verify "${2:-HEAD}^{commit}")"
    echo "$sha" > "$root/XETAL_EXTENSIONS_COMMIT"
    if [ -d "$clone/.git" ] && ! git -C "$clone" cat-file -e "$sha^{commit}" 2>/dev/null; then
        git -C "$clone" fetch --quiet "$repo" "$sha" 2>/dev/null || true
    fi
    source="${XETAL_EXTENSIONS_SOURCE:-$repo}"
    echo "pinned X_eTaL-extensions ${sha:0:7}: $(git -C "$repo" log -1 --format=%s "$sha")" >&2
fi
commit="$(tr -d '[:space:]' < "$root/XETAL_EXTENSIONS_COMMIT")"
if [ ! -d "$clone/.git" ]; then
    mkdir -p "$root/work"
    echo "xetal-extensions: cloning $source into work/xetal-extensions" >&2
    git clone --quiet "$source" "$clone"
fi
if ! git -C "$clone" cat-file -e "$commit^{commit}" 2>/dev/null; then
    git -C "$clone" fetch --quiet origin
fi
if [ "$(git -C "$clone" rev-parse HEAD)" != "$commit" ]; then
    git -C "$clone" checkout --quiet --detach "$commit"
fi
# work/xetal-extensions is a pristine checkout: never edit it.
[ -z "$(git -C "$clone" status --porcelain --untracked-files=no)" ] || { echo "xetal-extensions: work/xetal-extensions has local changes; it must match XETAL_EXTENSIONS_COMMIT" >&2; exit 1; }
# Its own X_eTaL (xetal-x is built on that clone's crates), from the
# sibling checkout when there is one.
sib="$root/../../softwarewrighter/X_eTaL"
if [ -d "$sib/.git" ]; then export XETAL_SOURCE="${XETAL_SOURCE:-$sib}"; fi
# Its scripts run Python from its own uv environment (.venv in the clone), never the system's.
if [ -x "$clone/scripts/venv.sh" ]; then
    [ -x "$clone/.venv/bin/python3" ] || "$clone/scripts/venv.sh" >&2
    export PATH="$clone/.venv/bin:$PATH"
fi
"$clone/scripts/xetal.sh" >/dev/null
export CARGO_TARGET_DIR="$root/target/xetal-extensions"
(cd "$clone" && cargo build -q --release -p xetal-x -p xetal-ext-scene -p xetal-ext-blender >&2)
# bin/xetal-x is a wrapper, not a symlink: xetal-x looks for native
# libraries beside the path it was started from, unresolved.
mkdir -p "$root/bin"
rm -f "$root/bin/xetal-x"
printf '#!/bin/sh\n# Written by scripts/xetal-extensions.sh: the pinned xetal-x, started by its real path.\nexec "%s" "$@"\n' "$CARGO_TARGET_DIR/release/xetal-x" > "$root/bin/xetal-x"
chmod +x "$root/bin/xetal-x"
echo "$root/bin/xetal-x"
