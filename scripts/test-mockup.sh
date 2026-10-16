#!/usr/bin/env bash
# The 3D mockup's test: mockup/calculator.xtl run headless (the scene
# window without a screen) with the events of mockup/headless.events
# (keys typed, clicks at the pixels where keys are drawn, a click on
# nothing, a key that is not on the panel); what it prints, each key and
# the screen after it, must equal mockup/headless.out. Needs xetal-x
# (scripts/xetal-extensions.sh); without it the test says it is skipped.
# XETAL_BLESS=1 rewrites the baseline (review the diff!).
#   scripts/test-mockup.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
if [ ! -x bin/xetal-x ]; then echo "test-mockup: skipped (no bin/xetal-x: just xetal-x builds it)"; exit 0; fi
events="$(paste -sd, mockup/headless.events)"
got="$(XETAL_HEADLESS=1 XETAL_EVENTS="$events" scripts/xx run mockup/calculator.xtl)"
if [ "${XETAL_BLESS:-}" = 1 ]; then printf '%s\n' "$got" > mockup/headless.out; echo "test-mockup: blessed mockup/headless.out"; exit 0; fi
if [ "$got" != "$(cat mockup/headless.out)" ]; then
  echo "FAIL: mockup/calculator.xtl headless differs from mockup/headless.out"
  diff mockup/headless.out <(printf '%s\n' "$got") | sed 's/^/  /' || true
  exit 1
fi
echo "test-mockup: ok (the 3D calculator, headless: $(grep -c '^key' mockup/headless.out) keys)"
