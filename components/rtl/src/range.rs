//! The values each cell can take, as an interval: what the emitter
//! needs to know that a memory's address is always in range, so a read
//! can index the memory directly (and Yosys can map it to block RAM).
//! Registers and memories are found by iterating to a fixed point;
//! anything still growing after a few rounds is given its type's whole
//! range, which is always safe.

use crate::netlist::{Kind, Netlist};
use xetal_fpga_xir::{Elem, MapOp};

/// The whole range of an element type.
fn full(e: Elem) -> (i128, i128) {
    match e {
        Elem::Bool => (0, 1),
        Elem::Int(w) => (-(1i128 << (w - 1)), (1i128 << (w - 1)) - 1),
    }
}

/// An interval kept to its type: one that does not fit (it would wrap)
/// is the whole range.
fn clamp(e: Elem, r: (i128, i128)) -> (i128, i128) {
    let (lo, hi) = full(e);
    if r.0 < lo || r.1 > hi {
        (lo, hi)
    } else {
        r
    }
}

fn union(a: (i128, i128), b: (i128, i128)) -> (i128, i128) {
    (a.0.min(b.0), a.1.max(b.1))
}

/// The smallest 2^k - 1 at or above x (x >= 0).
fn ones(x: i128) -> i128 {
    let mut m = 0;
    while m < x {
        m = 2 * m + 1;
    }
    m
}

fn map(op: MapOp, a: (i128, i128), b: (i128, i128), e: Elem) -> (i128, i128) {
    let r = match op {
        MapOp::Add => (a.0 + b.0, a.1 + b.1),
        MapOp::Sub => (a.0 - b.1, a.1 - b.0),
        MapOp::Mul => {
            let p = [a.0 * b.0, a.0 * b.1, a.1 * b.0, a.1 * b.1];
            (*p.iter().min().unwrap(), *p.iter().max().unwrap())
        }
        MapOp::Min => (a.0.min(b.0), a.1.min(b.1)),
        MapOp::Max => (a.0.max(b.0), a.1.max(b.1)),
        // With a non-negative operand, an and is between 0 and it.
        MapOp::And if a.0 >= 0 && b.0 >= 0 => (0, a.1.min(b.1)),
        MapOp::And if a.0 >= 0 => (0, a.1),
        MapOp::And if b.0 >= 0 => (0, b.1),
        MapOp::Or | MapOp::Xor if a.0 >= 0 && b.0 >= 0 => (0, ones(a.1.max(b.1))),
        MapOp::Eq | MapOp::Ne | MapOp::Lt | MapOp::Le | MapOp::Gt | MapOp::Ge => (0, 1),
        MapOp::Neg => (-a.1, -a.0),
        MapOp::Not if e == Elem::Bool => (0, 1),
        _ => full(e),
    };
    clamp(e, r)
}

/// Every cell's interval of values.
pub fn ranges(n: &Netlist) -> Vec<(i128, i128)> {
    let cells = &n.cells;
    let mut r: Vec<(i128, i128)> = cells.iter().map(|c| full(c.elem)).collect();
    // Registers and memories start at their reset value and widen.
    let mut seed: Vec<Option<(i128, i128)>> = cells
        .iter()
        .map(|c| match &c.kind {
            Kind::Reg { init, .. } => Some((*init as i128, *init as i128)),
            Kind::Mem { init, .. } => Some((
                init.iter().copied().min().unwrap_or(0) as i128,
                init.iter().copied().max().unwrap_or(0) as i128,
            )),
            _ => None,
        })
        .collect();
    for round in 0..40 {
        for (i, c) in cells.iter().enumerate() {
            r[i] = match &c.kind {
                Kind::In { .. } => full(c.elem),
                Kind::Const(v) => (*v as i128, *v as i128),
                Kind::Map(op, xs) => {
                    let b = xs.get(1).map(|x| r[*x]).unwrap_or((0, 0));
                    map(*op, r[xs[0]], b, c.elem)
                }
                Kind::Mux(_, a, b) => union(r[*a], r[*b]),
                Kind::Shl(a, k) => clamp(c.elem, (r[*a].0 << k, r[*a].1 << k)),
                Kind::Shr(a, k) => (r[*a].0 >> k, r[*a].1 >> k),
                Kind::Pick(items, _) => items.iter().map(|x| r[*x]).fold((0, 0), union),
                Kind::Rom(t, _) => (
                    t.iter().copied().min().unwrap_or(0).min(0) as i128,
                    t.iter().copied().max().unwrap_or(0).max(0) as i128,
                ),
                Kind::Cast(a) => match c.elem {
                    Elem::Bool => (0, 1),
                    _ => clamp(c.elem, r[*a]),
                },
                Kind::Reg { .. } | Kind::Mem { .. } => seed[i].expect("seeded"),
                Kind::MemRead(m, _) => union(seed[*m].expect("a memory"), (0, 0)),
            };
        }
        // Widen each register by what its d can be, each memory by its data.
        let mut grew = false;
        for (i, c) in cells.iter().enumerate() {
            let feed = match &c.kind {
                Kind::Reg { d, .. } => r[*d],
                Kind::Mem { data, .. } => r[*data],
                _ => continue,
            };
            let old = seed[i].expect("seeded");
            let mut new = union(old, feed);
            if round > 30 {
                new = full(c.elem);
            }
            if new != old {
                seed[i] = Some(new);
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    r
}
