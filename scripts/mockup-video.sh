#!/usr/bin/env bash
# Render the 3D mockup with Blender (mockup/video.xtl):
# work/mockup/blender/out/hex1000-mockup.mp4 and a still (-1.png), shown
# by the live site. Needs xetal-x (scripts/xetal-extensions.sh), Blender
# (XETAL_BLENDER, else /Applications/Blender.app, else blender on PATH)
# and the videos' font (X_eTaL-extensions' scripts/font.sh, copied into
# work/fonts, where the player looks).
#   scripts/mockup-video.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
"$root/scripts/xetal-extensions.sh" >/dev/null
if [ -z "${XETAL_BLENDER:-}" ]; then
  if [ -x /Applications/Blender.app/Contents/MacOS/Blender ]; then XETAL_BLENDER=/Applications/Blender.app/Contents/MacOS/Blender
  elif command -v blender >/dev/null; then XETAL_BLENDER="$(command -v blender)"
  else echo "mockup-video: no Blender (set XETAL_BLENDER)" >&2; exit 1; fi
fi
export XETAL_BLENDER
(cd work/xetal-extensions && scripts/font.sh >/dev/null)
mkdir -p work/fonts work/mockup/blender
cp work/xetal-extensions/work/fonts/* work/fonts/
scripts/xx run mockup/video.xtl
