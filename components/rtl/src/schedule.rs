//! The schedule: an XIR program and a number of lanes into a netlist.

use crate::netlist::{Kind, Netlist, Port};
use std::collections::{HashMap, HashSet};
use xetal_fpga_xir::{check, Elem, MapOp, Op, Program, RedOp, Type};

/// A value while it is built: its cells (one per element, or one per
/// lane in a temporal schedule) and whether it is a vector.
#[derive(Clone)]
struct Sig {
    cells: Vec<usize>,
    vector: bool,
}

fn red_map(r: RedOp) -> MapOp {
    match r {
        RedOp::Add => MapOp::Add,
        RedOp::Max => MapOp::Max,
        RedOp::Min => MapOp::Min,
        RedOp::And => MapOp::And,
        RedOp::Or => MapOp::Or,
    }
}

/// The value that leaves a reduction unchanged (a lane past the end
/// of the vector contributes it).
fn identity(r: RedOp, e: Elem) -> i64 {
    let (lo, hi) = match e {
        Elem::Int(w) if w < 64 => (-(1i64 << (w - 1)), (1i64 << (w - 1)) - 1),
        Elem::Int(_) => (i64::MIN, i64::MAX),
        Elem::Bool => (0, 1),
    };
    match (r, e) {
        (RedOp::Add | RedOp::Or, _) => 0,
        (RedOp::Max, _) => lo,
        (RedOp::Min, _) => hi,
        (RedOp::And, Elem::Bool) => 1,
        (RedOp::And, _) => -1,
    }
}

/// A balanced tree: the first half combined with the second half,
/// item by item, down to one (an odd last item is carried), as the
/// Datapath library's `s_um` does.
fn tree(n: &mut Netlist, op: MapOp, elem: Elem, mut xs: Vec<usize>) -> usize {
    while xs.len() > 1 {
        let h = xs.len() / 2;
        let mut next: Vec<usize> = (0..h)
            .map(|i| n.add(Kind::Map(op, vec![xs[i], xs[i + h]]), elem))
            .collect();
        if xs.len() % 2 == 1 {
            next.push(xs[2 * h]);
        }
        xs = next;
    }
    xs[0]
}

/// Elementwise cells: the operands' cells item by item, a scalar going
/// with every item.
fn each(sigs: &[&Sig], mut f: impl FnMut(&[usize]) -> usize) -> Sig {
    let vector = sigs.iter().any(|s| s.vector);
    let count = sigs.iter().map(|s| s.cells.len()).max().unwrap_or(1);
    let cells = (0..count)
        .map(|i| {
            let ops: Vec<usize> = sigs
                .iter()
                .map(|s| if s.vector { s.cells[i] } else { s.cells[0] })
                .collect();
            f(&ops)
        })
        .collect();
    Sig { cells, vector }
}

/// Bits enough for the counter and the element index: up to `n + lanes`,
/// signed.
fn index_width(top: usize) -> u32 {
    (usize::BITS - top.leading_zeros()) + 1
}

/// Schedule a program with `lanes` elements at once (`None`: as many
/// as the longest vector, the spatial, combinational circuit).
pub fn schedule(p: &Program, name: &str, lanes: Option<usize>) -> Result<Netlist, String> {
    let types: HashMap<String, Type> = check(p).map_err(|e| e.to_string())?.into_iter().collect();
    let longest = types
        .values()
        .filter(|t| t.rank() > 0)
        .map(|t| t.count())
        .max()
        .unwrap_or(1);
    let lanes = lanes.unwrap_or(longest).max(1);
    let mut n = Netlist {
        name: name.replace(['-', '.'], "_"),
        lanes: lanes.min(longest),
        ..Netlist::default()
    };
    if lanes >= longest {
        spatial(p, &types, &mut n)?;
    } else {
        temporal(p, &types, &mut n, lanes, longest)?;
    }
    Ok(n)
}

fn input(n: &mut Netlist, name: &str, ty: &Type, data: &[i64]) -> Sig {
    let port = n.inputs.len();
    n.inputs.push(Port {
        name: name.to_string(),
        ty: ty.clone(),
        wires: vec![],
    });
    n.input_data.push(data.to_vec());
    let cells = (0..ty.count())
        .map(|index| n.add(Kind::In { port, index }, ty.elem))
        .collect();
    Sig {
        cells,
        vector: ty.rank() > 0,
    }
}

fn konst(n: &mut Netlist, ty: &Type, data: &[i64]) -> Sig {
    Sig {
        cells: data.iter().map(|v| n.konst(*v, ty.elem)).collect(),
        vector: ty.rank() > 0,
    }
}

fn elementwise(n: &mut Netlist, op: &Op, ty: &Type, env: &HashMap<String, Sig>) -> Option<Sig> {
    let e = ty.elem;
    Some(match op {
        Op::Map(m, xs) => {
            let sigs: Vec<&Sig> = xs.iter().map(|x| &env[x]).collect();
            each(&sigs, |c| n.add(Kind::Map(*m, c.to_vec()), e))
        }
        Op::Select(s, a, b) => each(&[&env[s], &env[a], &env[b]], |c| {
            n.add(Kind::Mux(c[0], c[1], c[2]), e)
        }),
        Op::Shl(x, k) => each(&[&env[x]], |c| n.add(Kind::Shl(c[0], *k), e)),
        Op::Shr(x, k) => each(&[&env[x]], |c| n.add(Kind::Shr(c[0], *k), e)),
        _ => return None,
    })
}

fn spatial(p: &Program, types: &HashMap<String, Type>, n: &mut Netlist) -> Result<(), String> {
    let mut env: HashMap<String, Sig> = HashMap::new();
    for node in &p.nodes {
        let ty = &types[&node.name];
        let sig = match &node.op {
            Op::Input(t, d) => input(n, &node.name, t, d),
            Op::Const(t, d) => konst(n, t, d),
            Op::Reduce(r, x) => {
                let cells = env[x].cells.clone();
                Sig {
                    cells: vec![tree(n, red_map(*r), ty.elem, cells)],
                    vector: false,
                }
            }
            Op::Slice(x, s, l) => Sig {
                cells: env[x].cells[*s..s + l].to_vec(),
                vector: true,
            },
            Op::Concat(a, b) => Sig {
                cells: [env[a].cells.clone(), env[b].cells.clone()].concat(),
                vector: true,
            },
            op => elementwise(n, op, ty, &env).expect("every operation is handled"),
        };
        env.insert(node.name.clone(), sig);
    }
    for o in &p.outputs {
        n.outputs.push(Port {
            name: o.clone(),
            ty: types[o].clone(),
            wires: env[o].cells.clone(),
        });
    }
    Ok(())
}

fn temporal(
    p: &Program,
    types: &HashMap<String, Type>,
    n: &mut Netlist,
    lanes: usize,
    len: usize,
) -> Result<(), String> {
    let refuse = |why: String| {
        Err(format!("the temporal schedule (--lanes {lanes}) cannot take this program: {why}; use the spatial schedule"))
    };
    // Every vector has one length; a reduction's result (known only at
    // the end) may feed scalars and outputs, not vectors.
    let mut late: HashSet<&str> = HashSet::new();
    for node in &p.nodes {
        let ty = &types[&node.name];
        if ty.rank() > 1 || (ty.rank() == 1 && ty.count() != len) {
            return refuse(format!(
                "%{} is {ty}, and every vector must have {len} items",
                node.name
            ));
        }
        let operands: Vec<&str> = match &node.op {
            Op::Map(_, xs) => xs.iter().map(|s| s.as_str()).collect(),
            Op::Select(a, b, c) => vec![a, b, c],
            Op::Reduce(_, x) | Op::Shl(x, _) | Op::Shr(x, _) => vec![x],
            Op::Slice(..) | Op::Concat(..) => {
                return refuse(format!("%{} slices or joins vectors", node.name))
            }
            _ => vec![],
        };
        let from_late = operands.iter().any(|o| late.contains(o));
        let reduces = matches!(node.op, Op::Reduce(..));
        if from_late && (reduces || ty.rank() > 0) {
            return refuse(format!(
                "%{} needs a reduction's result before the reduction has finished",
                node.name
            ));
        }
        if from_late || reduces {
            late.insert(&node.name);
        }
    }

    let steps = len.div_ceil(lanes);
    n.steps = steps;
    let iw = Elem::Int(index_width(steps * lanes + lanes));
    // The counter: 0 to steps, one step per clock while running.
    let c = n.add(Kind::Reg { d: 0, init: 0 }, iw);
    let last = n.konst(steps as i64, iw);
    let one = n.konst(1, iw);
    let run = n.add(Kind::Map(MapOp::Lt, vec![c, last]), Elem::Bool);
    let inc = n.add(Kind::Map(MapOp::Add, vec![c, one]), iw);
    let next = n.add(Kind::Mux(run, inc, c), iw);
    n.cells[c].kind = Kind::Reg { d: next, init: 0 };
    n.done = Some(n.add(Kind::Map(MapOp::Ge, vec![c, last]), Elem::Bool));
    // Lane j computes element c * lanes + j.
    let k = n.konst(lanes as i64, iw);
    let base = n.add(Kind::Map(MapOp::Mul, vec![c, k]), iw);
    let index: Vec<usize> = (0..lanes)
        .map(|j| {
            let jj = n.konst(j as i64, iw);
            n.add(Kind::Map(MapOp::Add, vec![base, jj]), iw)
        })
        .collect();
    let valid: Vec<Option<usize>> = if len.is_multiple_of(lanes) {
        vec![None; lanes]
    } else {
        let end = n.konst(len as i64, iw);
        index
            .iter()
            .map(|&i| Some(n.add(Kind::Map(MapOp::Lt, vec![i, end]), Elem::Bool)))
            .collect()
    };

    let mut env: HashMap<String, Sig> = HashMap::new();
    for node in &p.nodes {
        let ty = &types[&node.name];
        let sig = match &node.op {
            Op::Input(t, d) => {
                let all = input(n, &node.name, t, d);
                if all.vector {
                    let cells = index
                        .iter()
                        .map(|&i| n.add(Kind::Pick(all.cells.clone(), i), t.elem))
                        .collect();
                    Sig {
                        cells,
                        vector: true,
                    }
                } else {
                    all
                }
            }
            Op::Const(t, d) if t.rank() > 0 => {
                let all = konst(n, t, d);
                let cells = index
                    .iter()
                    .map(|&i| n.add(Kind::Pick(all.cells.clone(), i), t.elem))
                    .collect();
                Sig {
                    cells,
                    vector: true,
                }
            }
            Op::Const(t, d) => konst(n, t, d),
            Op::Reduce(r, x) => {
                let e = ty.elem;
                let id = n.konst(identity(*r, e), e);
                let lanes_in: Vec<usize> = env[x]
                    .cells
                    .iter()
                    .zip(&valid)
                    .map(|(&v, ok)| match ok {
                        Some(ok) => n.add(Kind::Mux(*ok, v, id), e),
                        None => v,
                    })
                    .collect();
                let part = tree(n, red_map(*r), e, lanes_in);
                let acc = n.add(
                    Kind::Reg {
                        d: 0,
                        init: identity(*r, e),
                    },
                    e,
                );
                let sum = n.add(Kind::Map(red_map(*r), vec![acc, part]), e);
                let d = n.add(Kind::Mux(run, sum, acc), e);
                n.cells[acc].kind = Kind::Reg {
                    d,
                    init: identity(*r, e),
                };
                Sig {
                    cells: vec![acc],
                    vector: false,
                }
            }
            op => elementwise(n, op, ty, &env).expect("checked above"),
        };
        env.insert(node.name.clone(), sig);
    }
    // Vector outputs are collected: element i is lane i % lanes at
    // step i / lanes, held in a register.
    for o in &p.outputs {
        let ty = types[o].clone();
        let sig = env[o].clone();
        let wires = if sig.vector {
            (0..len)
                .map(|i| {
                    let at = n.konst((i / lanes) as i64, iw);
                    let now = n.add(Kind::Map(MapOp::Eq, vec![c, at]), Elem::Bool);
                    let r = n.add(Kind::Reg { d: 0, init: 0 }, ty.elem);
                    let d = n.add(Kind::Mux(now, sig.cells[i % lanes], r), ty.elem);
                    n.cells[r].kind = Kind::Reg { d, init: 0 };
                    r
                })
                .collect()
        } else {
            sig.cells.clone()
        };
        n.outputs.push(Port {
            name: o.clone(),
            ty,
            wires,
        });
    }
    Ok(())
}
