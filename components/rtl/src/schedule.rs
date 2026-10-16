//! The schedule: an XIR program and a number of lanes into a netlist.

use crate::netlist::{Kind, Netlist, Port};
use std::collections::{HashMap, HashSet};
use xetal_fpga_xir::{check, clocks, operands, Elem, MapOp, Op, Program, RedOp, Type};

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
    schedule_with(p, name, lanes, false)
}

/// As `schedule`, and with `memories` a state vector that is only
/// written by its `next` (a `put` of itself) and only read by
/// `lookup`s becomes a memory (`Kind::Mem`): a RAM, which Yosys maps
/// to block or LUT RAM, instead of a register per word and a mux tree.
/// A memory is loaded at configuration and not cleared by a reset, so
/// this is schedule data a program opts into (`# schedule: --memories`
/// in an .xir), not the default.
pub fn schedule_with(
    p: &Program,
    name: &str,
    lanes: Option<usize>,
    memories: bool,
) -> Result<Netlist, String> {
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
    n.clocks = clocks(p);
    if lanes >= longest {
        let mems = if memories {
            memory_states(p, &types)
        } else {
            HashSet::new()
        };
        spatial(p, &types, &mut n, &mems)?;
    } else if n.clocks.is_some() {
        return Err(format!(
            "the temporal schedule (--lanes {lanes}) cannot take a clocked program (one with state); use the spatial schedule"
        ));
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
        trace: false,
    });
    n.input_data.push(data.to_vec());
    n.stream_data.push(None);
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
        Op::Cast(_, x) => each(&[&env[x]], |c| n.add(Kind::Cast(c[0]), e)),
        Op::Lookup(t, i) => {
            let table = env[t].cells.clone();
            let consts: Option<Vec<i64>> = table
                .iter()
                .map(|&c| {
                    if let Kind::Const(v) = n.cells[c].kind {
                        Some(v)
                    } else {
                        None
                    }
                })
                .collect();
            // A constant index is wiring: the item itself (or 0 out of range).
            each(&[&env[i]], |c| {
                match (n.cells[c[0]].kind.clone(), &consts) {
                    (Kind::Const(k), _) if k >= 0 && (k as usize) < table.len() => {
                        table[k as usize]
                    }
                    (Kind::Const(_), _) => n.konst(0, e),
                    (_, Some(values)) => n.add(Kind::Rom(values.clone(), c[0]), e),
                    (_, None) => n.add(Kind::Pick(table.clone(), c[0]), e),
                }
            })
        }
        Op::Put(v, i, x) => {
            // A write port: each word takes the item when the index is
            // its address (an index too narrow to name a word never does).
            let (words, at, item) = (env[v].cells.clone(), env[i].cells[0], env[x].cells[0]);
            let ie = n.cells[at].elem;
            if let Kind::Const(k) = n.cells[at].kind {
                // A constant address is wiring: that word is the item.
                let mut cells = words;
                if k >= 0 && (k as usize) < cells.len() {
                    cells[k as usize] = item;
                }
                return Some(Sig {
                    cells,
                    vector: true,
                });
            }
            let cells = words
                .iter()
                .enumerate()
                .map(|(k, &w)| {
                    if !fits(ie, k) {
                        return w;
                    }
                    let addr = n.konst(k as i64, ie);
                    let hit = n.add(Kind::Map(MapOp::Eq, vec![at, addr]), Elem::Bool);
                    n.add(Kind::Mux(hit, item, w), e)
                })
                .collect();
            Sig {
                cells,
                vector: true,
            }
        }
        _ => return None,
    })
}

/// Whether an address fits an index of an element type.
fn fits(e: Elem, k: usize) -> bool {
    match e {
        Elem::Int(w) if w < 64 => (k as u128) < (1u128 << (w - 1)),
        Elem::Int(_) => true,
        Elem::Bool => k < 2,
    }
}

/// The state vectors that can be memories: each written only by its
/// `next`, a `put` of itself used nowhere else, and otherwise read only
/// as the table of `lookup`s (not an output).
fn memory_states(p: &Program, types: &HashMap<String, Type>) -> HashSet<String> {
    let ops: HashMap<&str, &Op> = p.nodes.iter().map(|n| (n.name.as_str(), &n.op)).collect();
    let mut uses: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in &p.nodes {
        for x in operands(&node.op) {
            uses.entry(x.as_str()).or_default().push(node.name.as_str());
        }
    }
    let outputs: HashSet<&str> = p.outputs.iter().map(|o| o.name.as_str()).collect();
    let nexts: Vec<&str> = p.nexts.iter().map(|(_, v)| v.as_str()).collect();
    let mut found = HashSet::new();
    for (s, v) in &p.nexts {
        if types[s].rank() != 1 || outputs.contains(s.as_str()) || outputs.contains(v.as_str()) {
            continue;
        }
        let Some(Op::Put(of, _, _)) = ops.get(v.as_str()) else {
            continue;
        };
        if of != s || nexts.iter().filter(|x| **x == v.as_str()).count() != 1 {
            continue;
        }
        let put_unused = uses.get(v.as_str()).is_none_or(|u| u.is_empty());
        let reads_only = uses.get(s.as_str()).is_some_and(|u| {
            u.iter().all(|user| {
                *user == v.as_str()
                    || matches!(ops.get(user), Some(Op::Lookup(t, i)) if t == s && i != s)
            })
        });
        let put_index_not_self =
            matches!(ops.get(v.as_str()), Some(Op::Put(_, i, x)) if i != s && x != s);
        if put_unused && reads_only && put_index_not_self {
            found.insert(s.clone());
        }
    }
    found
}

fn spatial(
    p: &Program,
    types: &HashMap<String, Type>,
    n: &mut Netlist,
    mems: &HashSet<String>,
) -> Result<(), String> {
    let mut env: HashMap<String, Sig> = HashMap::new();
    // A memory's cell, by state name; a write (the put that is its next).
    let mut mem_of: HashMap<String, usize> = HashMap::new();
    let mut writes: HashMap<String, (usize, usize)> = HashMap::new();
    for node in &p.nodes {
        let ty = &types[&node.name];
        let sig = match &node.op {
            Op::State(t, d) if mems.contains(&node.name) => {
                let m = n.add(
                    Kind::Mem {
                        init: d.clone(),
                        addr: 0,
                        data: 0,
                    },
                    t.elem,
                );
                mem_of.insert(node.name.clone(), m);
                Sig {
                    cells: vec![],
                    vector: true,
                }
            }
            Op::Lookup(t, i) if mem_of.contains_key(t) => {
                let m = mem_of[t];
                each(&[&env[i]], |c| n.add(Kind::MemRead(m, c[0]), ty.elem))
            }
            Op::Put(v, i, x) if mem_of.contains_key(v) => {
                writes.insert(node.name.clone(), (env[i].cells[0], env[x].cells[0]));
                Sig {
                    cells: vec![],
                    vector: true,
                }
            }
            Op::Input(t, d) => input(n, &node.name, t, d),
            Op::Stream(t, d) => {
                let c = t.count().max(1);
                let sig = input(n, &node.name, t, &d[..c]);
                *n.stream_data.last_mut().expect("just pushed") = Some(d.clone());
                sig
            }
            Op::State(t, d) => Sig {
                // Registers, their d patched from `next` below.
                cells: d
                    .iter()
                    .map(|&init| n.add(Kind::Reg { d: 0, init }, t.elem))
                    .collect(),
                vector: t.rank() > 0,
            },
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
    for (s, v) in &p.nexts {
        if let Some(&m) = mem_of.get(s) {
            let (addr, data) = writes[v];
            if let Kind::Mem { init, .. } = n.cells[m].kind.clone() {
                n.cells[m].kind = Kind::Mem { init, addr, data };
            }
            continue;
        }
        for (&r, &d) in env[s].cells.iter().zip(&env[v].cells) {
            if let Kind::Reg { init, .. } = n.cells[r].kind {
                n.cells[r].kind = Kind::Reg { d, init };
            }
        }
    }
    for o in &p.outputs {
        n.outputs.push(Port {
            name: o.name.clone(),
            ty: types[&o.name].clone(),
            wires: env[&o.name].cells.clone(),
            trace: o.trace,
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
        let ty = types[&o.name].clone();
        let sig = env[&o.name].clone();
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
            name: o.name.clone(),
            ty,
            wires,
            trace: false,
        });
    }
    Ok(())
}
