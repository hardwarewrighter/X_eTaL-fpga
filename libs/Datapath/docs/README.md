# Datapath

The vector examples at a stated width: the whole-array operations a
circuit realizes directly.

```
"dp:" u_se< "Datapath"
```

Put each library's `src/` directory on `XETAL_PATH` (`just path`
prints them; see the [README](../../../README.md)); `dp:` is the
recommended alias. The library imports Fixed itself.

Each function is one whole-array expression a circuit can take on
directly: arithmetic and comparisons item by item with scalar
extension (one unit per element in a spatial schedule, one unit and a
counter in a temporal one), a select by mask (a mux), and reductions
(a tree of adders). The arithmetic is
[Fixed](../../Fixed/docs/README.md)'s, so every result is what w-bit
units give, and the evaluator is the reference the generated circuit
must match bit for bit: the demos are the programs it checks. The
width comes first; a function of three or four arguments is used
curried, the width bound once (`u:v_add := dp:v_add 16`, then
`x u:v_add y`; `u:s_axpy := 16 dp:s_axpy 3`).

## Functions

| Function | Type | What | In hardware |
| -------- | ---- | ---- | ----------- |
| `w dp:v_add x y` | `Int -> Int -> Int -> Int` | the sum, item by item, wrapped to `w` bits | an adder per element |
| `w dp:s_axpy a x y` | `Int -> Int -> Int -> Int -> Int` | `a` times `x` plus `y`, wrapped at each unit | a multiplier and an adder per element |
| `dp:r_elu x` | `Num a => a -> a` | `x` where positive, 0 elsewhere | a comparator and a mux per element |
| `t dp:t_hreshold x` | `(Num a, Truthy a) => a -> a -> a` | `x` where `x > t`, 0 elsewhere | the mask `x > t` is the mux's select |
| `m dp:k_eep x` | `Num a => a -> a -> a` | `x` where the mask `m` is 1, 0 elsewhere | a mux per element |
| `w dp:s_um x` | `Int -> Int -> Int` | the sum of a vector as a balanced tree of `w`-bit adders | log2 n levels of adders |
| `dp:l_argest x` | `Num a => a -> a` | the largest item | a tree of comparators and muxes |
| `w dp:d_ot x y` | `Int -> Int -> Int -> Int` | the inner product, wrapped | a multiplier per element, then the tree |
| `w dp:p_ipeline x k` | `Int -> Int -> Int -> Int` | the sum of the positive items of `x + x * k`, every unit `w` bits | multiply, add, select, reduce: research7's first end-to-end test |

`dp:s_um` adds the first half of the vector to the second half, item
by item, then that vector likewise, down to one number (an odd length
is padded with a 0): the shape of a reduction tree. Wrapping at every
adder gives the same bits as wrapping the exact sum once, because the
arithmetic is that of the integers modulo `2^w`;
`../tests/checks.xtl` checks the tree against the sequential sum for
several lengths, including ones that overflow at every level.

## Examples

From `../tests/basics.xtl`:

```
      u:v_add := dp:v_add 16
      1 2 3 4 u:v_add 10 20 30 40
11 22 33 44
      30000 u:v_add 30000 -30000
-5536 0
      u:s_axpy := 16 dp:s_axpy 3
      1 2 3 4 u:s_axpy 100 200 300 400
103 206 309 412
      2 dp:t_hreshold 1 2 3 4
0 0 3 4
      16 dp:s_um 1 2 3 4 5
15
      16 dp:s_um 20000 20000 20000 20000
14464
      u:d_ot := dp:d_ot 16
      1 2 3 u:d_ot 4 5 6
32
      u:p_ipeline := dp:p_ipeline 16
      1 2 3 4 u:p_ipeline 1 -2 1 -2
8
```

## Demos

The example programs, each with a reg-rs baseline that is the golden
reference for the circuit generated from it:

- [`demos/vector-add.xtl`](../demos/vector-add.xtl): research7's first
  FPGA experiment, `c := a + b` on eight 16-bit numbers, eight adders,
  one pair overflowing.
- [`demos/saxpy.xtl`](../demos/saxpy.xtl): `a x + y`, a multiplier and
  an adder per element.
- [`demos/threshold.xtl`](../demos/threshold.xtl): a select by mask, a
  comparator and a mux per element.
- [`demos/reduce-sum.xtl`](../demos/reduce-sum.xtl): `'+ r_/ v` as a
  tree of seven adders, three levels, and a sum that wraps.
- [`demos/pipeline.xtl`](../demos/pipeline.xtl): multiply, add,
  select, reduce in one expression.

Each demo has a hand-lowered twin beside it, `demos/NAME.xir`, in the
provisional accelerator IR ([`components/xir`](../../../components/xir/README.md)),
until X_eTaL lowers its programs itself (ask F2). A twin names the
baseline lines it reproduces (`# baseline: demo-vector-add 2`; the
evaluator's own 64-bit lines are not circuit outputs), and
`just equiv` checks that it prints exactly those lines.

## Provenance

Written for X_eTaL-fpga after research7's FPGA milestones F1 to F4
(scalar add, a fixed vector add, configurable lanes, a reduction
tree) and the acceptance pipeline. X_eTaL-gpu's Accel library holds
the same examples for the GPU, without widths; the two are meant to
run the same programs (research7's X0).
