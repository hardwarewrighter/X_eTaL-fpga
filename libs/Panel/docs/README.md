# Panel

HEX1000's calculator panel as arrays: what the physical calculator
([`docs/goals.md`](../../../docs/goals.md)) shows and where its keys
are, computed in X_eTaL. A window or a renderer only draws what these
functions give ([`mockup/`](../../../mockup/README.md)).

```
"pn:" u_se< "Panel"
```

Put each library's `src/` directory on `XETAL_PATH` (`just path`);
`pn:` is the recommended alias. It imports Hex1000.

## Coordinates

Scene units, the panel's face in the plane z = 0 facing +z, y up. The
case is 5.5 wide and 6.7 tall; the keys are 0.78 square, 0.95 apart,
standing 0.3 off the face (0.1 when pressed); the operator column sits
0.2 further right; the display's eight digits are 0.58 apart, 0.3 wide
and 0.5 tall. Boxes are rows of x y z (the middle), width, height,
depth; strokes are rows of x1 y1 x2 y2.

## The camera

`Eye` is a fixed first-person camera, in front of the panel and a
little above it, pitched down; `p_roject` is the scene extension's
projection for it (a 70 degree field of view across the window's
smaller side, `View` 560 by 760), so `h_it` can take a click's pixel
position back to a key: each key's top face projected, its corners'
box tested.

## Functions

| Function | What |
| -------- | ---- |
| `pn:Font` | the segments of 0 to F, a number each: bit 0 segment a (top), b to f clockwise, bit 6 g (middle); A b C d E F |
| `pn:s_egs d` | the segments lit for digit latches `d` (O as latched: bit 4 set to show), a row a to g each; blank without bit 4 |
| `pn:Keys` | the keys as they sit, by code: the hexadecimal pad and the operator column |
| `pn:k_eyAt c` | the middle of each key `c`, x y |
| `pn:k_eyBoxes p` | the 20 keys as boxes, by code, key `p` pressed (-1: none) |
| `pn:Case`, `pn:Led` | the case and the display's window; the ready light |
| `pn:b_oxes b` | boxes as quads, 24 corners a box (every box at once, an outer product with a unit cube's signs) |
| `pn:s_trokes (s, z, t)` | strokes as thin quads facing +z at depth `z`, `t` wide |
| `pn:l_egends p` | every key's legend as strokes on its top (the display's font; + - x =), and their depths |
| `pn:d_igits d` | the display's eight digits as strokes: (lit, dark) |
| `pn:View`, `pn:Eye` | the window's size; the camera (x y z, pitch) |
| `pn:p_roject pts` | where the camera draws points, x y in pixels |
| `pn:h_it xy` | the key a click lands on, or -1 |
| `pn:p_ress (st, d, c)` | key `c` pressed at HEX1000 running the calculator ROM, through its key scan: (state, digit latches, instructions) |
| `pn:r_eady st` | 1 while the program waits for a key (R15) |

`just types Panel` prints the full types; `tests/types.rgt` pins them.

## Tests and demos

- [`tests/panel.xtl`](../tests/panel.xtl): the font against the
  segments it names, blank digits, the boxes' shape, every key found
  again where the camera draws it, a click between keys, the segments
  of 17FCd, and keys pressed at the panel against `hx:c_alc`.
- [`demos/font.xtl`](../demos/font.xtl): the font drawn in three lines
  of text, and the display after three calculations.

## Provenance

Written for X_eTaL-fpga: the physical calculator's panel (decided with
the user, 2026-10-15), drawn by `mockup/` in X_eTaL-extensions' scene
window and in Blender.
