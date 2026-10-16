# Panel

HEX1000's calculator panel as arrays, in X_eTaL: the 7-segment font,
the keys and display in 3D, the camera that draws them (so a click is
mapped back to a key), and a key pressed at the machine through its
own key scan. The physical calculator's design ([`docs/goals.md`](../../docs/goals.md));
[`mockup/`](../../mockup/README.md) draws it.

```
"pn:" u_se< "Panel"
```

| Directory | What |
| --------- | ---- |
| [`src/`](src/) | the library, `Panel.xtl` |
| [`docs/`](docs/README.md) | the reference: every function, the coordinates, the camera |
| [`demos/`](demos/) | the font drawn in text, and screens after calculations |
| [`tests/`](tests/) | reg-rs baselines: the font, the shapes, every key found where it is drawn, keys pressed against `hx:c_alc`, the pinned export types |

```bash
just demo-lib Panel        # run its demos
just test-lib Panel        # check every baseline
```
