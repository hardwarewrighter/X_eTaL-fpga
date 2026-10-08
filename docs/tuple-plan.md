# Tuples in X_eTaL-fpga: a plan

Written 2026-10-08 at the user's request, after finding that the
pinned X_eTaL (d284a8c) has tuples. It says what tuples can do at the
pin, where this repository will use them and where it will not, how
they map to hardware, and what is missing (with who fixes it).

## What the pinned X_eTaL has

X_eTaL's Saga 39 (lanes/tuples) has landed through its step 4,
tuple patterns (the rules are TU1 to TU12 in X_eTaL's
`docs/lang-choices.md`, section 9d). Checked at the pin:

| What | Example | At d284a8c |
| ---- | ------- | ---------- |
| a tuple literal, parts of any rank and type | `(1, 2 3 4)` | works, prints `(1, 2 3 4)` |
| patterns in lambda parameters | `{ (op, a, b) -> a + b * op }` | works, typed `Num a => (a, a, a) -> a` |
| a dyadic step with a tuple state | `{ (acc, mem) k -> (acc + k, mem + k) }` | works |
| patterns in bindings | `(a, b) := (5, 6 7)` | works |
| `m_atch` on tuples (TU7) | `(1, 2 3) m_atch (1, 2 3)` | works, 1 |
| `p_ower` with a tuple state (TU9) | `3 '{ (n, v) -> (n + 1, v * 2) } p_ower (0, 1 2)` | works, `(3, 8 16)` |
| Clocked's `ck:r_un` with a tuple state | `(0, 0) 'u:s_tep ck:r_un 1 2 3` | works, `(6, 3)` |
| `=` and arithmetic through a tuple (TU7) | `(1, 2) = (1, 2)` | a type error, by design |
| projection by position (TU3) | none | none, by design; a pattern takes a tuple apart |
| boxed tuples, tuples in arrays (TU8) | `e_nclose (1, 2)` | not yet: X_eTaL's Saga 39 step 5 |
| `f_ormat`, `s_hape`, `t_ally`, `r_avel` of a tuple | `f_ormat (1, 2)` | type errors ("expected an array, found a tuple"); see the gap below |

## Where this repository uses tuples

**HEX1000's state (saga 3, step 4, machine-lib).** The machine is a
dozen registers of different widths and a 64-word RAM. Packed into
one Int vector, as the UART and Radix states are, every field is read
by position (`5 s_elect s`) and nothing checks that a step returns
every field in its place. As a tuple the step says what it means and
the checker holds it to it:

```
h:s_tep := { (a, y, x, s, pc, page, buf, ret, call, o, r, ram) k -> ... }
```

The RAM stays an array (64 words, one part of the tuple). The library
gives the reset state as a tuple, and helper functions take the tuple
apart with a pattern and build the next one.

**HEX1000's ALU and decoder.** A unit of three inputs is a monadic
function of a tuple, `{ (op, a, b) -> ... }`, instead of today's
two-item vector argument (Radix's `rx:a_lu`): the parts are named and
their count is checked. This is a partial answer to ask F9 (a function
of three arguments in one line): it gives `u:a_lu (op, a, b)`, not
`op a_lu a b`.

## Where it does not

- **Fixed and Datapath stay width-first and curried**
  (`u:a_dd := fx:a_dd 16`, then `x u:a_dd y`): binding the width once
  is how a hardware parameter works, and the functions work on whole
  arrays, which a tuple cannot carry through arithmetic (TU7).
- **Numbers that are one kind stay arrays**: digits, RAM words, a
  vector of lanes. X_eTaL-libraries' `docs/tuples.md` draws the same
  line: an array where the items are alike, a tuple where the parts
  differ in kind or role.
- **Uart and Radix are not rewritten now.** Their packed states work
  and are tested; they move to tuples only if HEX1000 shows the gain is
  worth the churn, as a step of its own.

## How tuples map to hardware

A tuple state is a set of registers, one per part, each of its own
width. That is what the provisional IR already expresses: a clocked
program may have several `state` nodes, each with its own type. So a
hand-lowered twin of a tuple step declares one `state` per part, and
needs no tuple type in the IR:

```
%a = state i4 0        # (a, y, x, s, ...) -> one register each
%y = state i4 0
%x = state i2 0
%s = state bool 1
%ram = state i4[64] 0 0 ... 0
```

This is a gain in circuit size, not only in clarity: one Int vector
forces every register to one width (the UART loopback's fifteen
16-bit registers hold fields of 1 to 10 bits). When X_eTaL's lowering
exists (asks F1, F2), a tuple in Core becomes this set of states; the
IR gets a tuple type only if a program passes tuples through
combinational logic in a way that splitting cannot express.

## Gaps and who fixes them

| Gap | Who fixes it | Workaround here |
| --- | ------------ | --------------- |
| **`f_ormat`, `s_hape` and the other array built-ins reject tuples**, so the Check library's `k:i_s`, whose failure message formats both sides (`h:s_hown`: `f_ormat t`, and `s_hape t` for a long value), does not type-check for a tuple: `(1, 2) k:i_s (1, 2)` is "expected an array, found a tuple". | First **X_eTaL** (its agent, in `../../softwarewrighter/X_eTaL`): `f_ormat` of a tuple, printing it as TU6 prints it (ask F13 in `docs/xetal-asks.md`). Then **X_eTaL-libraries** (its agent): Check's `h:s_hown` must not ask a tuple for its shape (only an array has one); its `i_s` then works on tuples. This repository changes neither. | compare with `m_atch` and report with `k:t_rue`: `k:t_rue want m_atch got` works on tuples today (it says "ok", or "FAIL: expected true" without the values) |
| **Clocked's `ck:t_race` takes only array states**: it stacks states with `r_eshape` and `c_at`, which do not take tuples. | **This repository** (step 4): a trace of a view of the state (the display, a register), an array per clock, rather than the whole tuple. | trace a view; or step and print by hand |
| **Boxed tuples** (`e_nclose (1, 2)`), the way to keep a tuple per clock in one array (TU8) | **X_eTaL**, Saga 39 step 5, already planned | none needed for step 4 |

## Steps

1. Step 4 (machine-lib): HEX1000's state and ALU as tuples; a tuple
   trace by view in Clocked (or in the machine library) with tests;
   checks with `m_atch` and `k:t_rue` until F13 lands.
2. Step 7 (machine-rtl): the twin with one `state` per part, each of
   its own width; `explain` shows the register bits.
3. When F13 lands in X_eTaL and Check follows in X_eTaL-libraries:
   re-pin both, switch the checks to `k:i_s`, close F13.
4. Later, a step of its own if HEX1000 shows the gain: Uart and Radix
   states as tuples.
