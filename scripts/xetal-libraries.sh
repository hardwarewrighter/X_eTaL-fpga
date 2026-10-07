#!/usr/bin/env bash
# Get the libraries this repository uses from X_eTaL-libraries, as
# scripts/xetal.sh gets X_eTaL: clone it into work/xetal-libraries
# (gitignored), check out the known-good commit in
# XETAL_LIBRARIES_COMMIT, and link each library used (USE below) as
# work/libs/<Name>, so work/libs/*/src goes on XETAL_PATH beside this
# repository's own libs/*/src and into the pages' store. Safe to run
# at any time.
#   scripts/xetal-libraries.sh
#   XETAL_LIBRARIES_SOURCE=../X_eTaL-libraries scripts/xetal-libraries.sh
#   scripts/xetal-libraries.sh --pin [REF]   # pin a committed ref of ../../softwarewrighter/X_eTaL-libraries (default HEAD)
set -euo pipefail
USE=(Check Bits Strings)
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source="${XETAL_LIBRARIES_SOURCE:-https://github.com/softwarewrighter/X_eTaL-libraries.git}"
clone="$root/work/xetal-libraries"
if [ "${1:-}" = --pin ]; then
    repo="${XETAL_LIBRARIES_REPO:-$root/../../softwarewrighter/X_eTaL-libraries}"
    sha="$(git -C "$repo" rev-parse --verify "${2:-HEAD}^{commit}")"
    echo "$sha" > "$root/XETAL_LIBRARIES_COMMIT"
    if [ -d "$clone/.git" ] && ! git -C "$clone" cat-file -e "$sha^{commit}" 2>/dev/null; then
        git -C "$clone" fetch --quiet "$repo" "$sha" 2>/dev/null || true
    fi
    source="${XETAL_LIBRARIES_SOURCE:-$repo}"
    echo "pinned X_eTaL-libraries ${sha:0:7}: $(git -C "$repo" log -1 --format=%s "$sha")" >&2
fi
commit="$(tr -d '[:space:]' < "$root/XETAL_LIBRARIES_COMMIT")"
if [ ! -d "$clone/.git" ]; then
    mkdir -p "$root/work"
    echo "xetal-libraries: cloning $source into work/xetal-libraries" >&2
    git clone --quiet "$source" "$clone"
fi
if ! git -C "$clone" cat-file -e "$commit^{commit}" 2>/dev/null; then
    git -C "$clone" fetch --quiet origin
fi
if [ "$(git -C "$clone" rev-parse HEAD)" != "$commit" ]; then
    git -C "$clone" checkout --quiet --detach "$commit"
fi
mkdir -p "$root/work/libs"
for name in "${USE[@]}"; do
    [ -d "$clone/libs/$name/src" ] || { echo "xetal-libraries: no library $name at ${commit:0:7}" >&2; exit 1; }
    ln -sfn "../xetal-libraries/libs/$name" "$root/work/libs/$name"
done
