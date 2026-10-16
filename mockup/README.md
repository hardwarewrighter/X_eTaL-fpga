# The 3D calculator mockup

HEX1000 as the physical calculator it is becoming (a 4x4 hexadecimal
pad, four operator keys, an 8-digit 7-segment display; `docs/goals.md`),
drawn before anything is soldered. The panel is arrays computed in
X_eTaL ([`libs/Panel`](../libs/Panel/docs/README.md)); behind it is
HEX1000 running the calculator ROM, every key going through the
machine's own key scan. Two programs draw it, with
[X_eTaL-extensions](https://github.com/softwarewrighter/X_eTaL-extensions)
pinned in `XETAL_EXTENSIONS_COMMIT` (`just xetal-x` clones and builds
it into `work/`):

| Program | What | Run |
| ------- | ---- | --- |
| [`calculator.xtl`](calculator.xtl) | interactive, in the scene extension's window: click the keys (each click is mapped back to a key by projecting the keys with the window's camera) or type them; the line under the panel says what each key cost | `just mockup` |
| [`video.xtl`](video.xtl) | rendered by Blender through the Video library: the calculator on a desk, C0DE+BEEF= pressed a key a second, the display changing as the machine takes each key; it refuses to render unless the last screen is the evaluator's (`hx:c_alc`) | `just mockup-video` |

`just mockup-test` runs the window headless with the events of
[`headless.events`](headless.events) (typed keys, clicks where keys are
drawn, a click on nothing, a key not on the panel) and compares what it
prints with [`headless.out`](headless.out); the gate runs it when
`bin/xetal-x` is built and says it skipped it otherwise.

The video needs Blender (`XETAL_BLENDER`, else
`/Applications/Blender.app`, else `blender` on `PATH`; written for
Blender 5.1) and writes `work/mockup/blender/out/hex1000-mockup.mp4`
and a still; `just pages` puts them on the live site.
