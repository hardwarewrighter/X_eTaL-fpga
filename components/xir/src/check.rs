//! Checking: every operand defined before use, names unique, element
//! types and widths agreeing, shapes agreeing (or one operand a
//! scalar), data fitting its type; for a clocked program (one with
//! state), every state with exactly one `next` of its own type, one
//! clock count, and no output that depends on a stream (an output is
//! read after the last clock, when no stream item is present). Gives
//! each node's type.

use crate::{Elem, Error, MapOp, Op, Program, RedOp, Type};
use std::collections::{HashMap, HashSet};

/// The values an operation reads.
pub fn operands(op: &Op) -> Vec<&String> {
    match op {
        Op::Input(..) | Op::Const(..) | Op::State(..) | Op::Stream(..) => vec![],
        Op::Map(_, xs) => xs.iter().collect(),
        Op::Select(a, b, c) | Op::Put(a, b, c) => vec![a, b, c],
        Op::Reduce(_, x) | Op::Shl(x, _) | Op::Shr(x, _) | Op::Slice(x, _, _) | Op::Cast(_, x) => {
            vec![x]
        }
        Op::Concat(a, b) | Op::Lookup(a, b) => vec![a, b],
    }
}

/// How many clocks a checked program runs: `None` for a combinational
/// program (no state), else the streams' length or `clocks N`.
pub fn clocks(p: &Program) -> Option<usize> {
    if !p.nodes.iter().any(|n| matches!(n.op, Op::State(..))) {
        return None;
    }
    p.nodes
        .iter()
        .find_map(|n| match &n.op {
            Op::Stream(t, d) => Some(d.len() / t.count().max(1)),
            _ => None,
        })
        .or(p.clocks)
}

/// Check a program and give the type of every node, in order.
pub fn check(p: &Program) -> Result<Vec<(String, Type)>, Error> {
    let mut env: HashMap<&str, Type> = HashMap::new();
    let mut streamed: HashSet<&str> = HashSet::new();
    let mut out = Vec::new();
    let mut stream_len: Option<usize> = None;
    for (i, n) in p.nodes.iter().enumerate() {
        let at = |msg: String| Error::new(0, format!("%{} (node {}): {msg}", n.name, i + 1));
        let get = |x: &str| {
            env.get(x)
                .cloned()
                .ok_or_else(|| at(format!("%{x} is not defined before it is used")))
        };
        let int = |t: &Type, what: &str| match t.elem {
            Elem::Int(_) => Ok(()),
            Elem::Bool => Err(at(format!("{what} must be an integer, not {t}"))),
        };
        let ty = match &n.op {
            Op::Input(t, d) | Op::Const(t, d) | Op::State(t, d) => {
                if d.len() != t.count() {
                    return Err(at(format!(
                        "{t} holds {} items, {} given",
                        t.count(),
                        d.len()
                    )));
                }
                fits_all(t, d).map_err(at)?;
                t.clone()
            }
            Op::Stream(t, d) => {
                let c = t.count().max(1);
                if d.is_empty() || d.len() % c != 0 {
                    return Err(at(format!("a stream of {t} needs a whole number of items ({c} numbers each), {} given", d.len())));
                }
                fits_all(t, d).map_err(at)?;
                let len = d.len() / c;
                match stream_len {
                    Some(l) if l != len => {
                        return Err(at(format!(
                            "streams differ in length: {l} and {len} clocks"
                        )))
                    }
                    _ => stream_len = Some(len),
                }
                t.clone()
            }
            Op::Map(m, xs) => {
                let ts: Vec<Type> = xs.iter().map(|x| get(x)).collect::<Result<_, _>>()?;
                map_type(*m, &ts).map_err(at)?
            }
            Op::Select(m, a, b) => {
                let (tm, ta, tb) = (get(m)?, get(a)?, get(b)?);
                if tm.elem != Elem::Bool {
                    return Err(at(format!("select's mask must be bool, not {tm}")));
                }
                if ta.elem != tb.elem {
                    return Err(at(format!("select's sides differ: {ta} and {tb}")));
                }
                Type::new(
                    ta.elem,
                    extend(&[&tm.shape, &ta.shape, &tb.shape]).map_err(at)?,
                )
            }
            Op::Reduce(r, x) => {
                let t = get(x)?;
                if t.rank() != 1 || t.count() == 0 {
                    return Err(at(format!("reduce needs a non-empty vector, not {t}")));
                }
                if t.elem == Elem::Bool && !matches!(r, RedOp::And | RedOp::Or) {
                    return Err(at(format!("reduce {} does not take {t}", r.name())));
                }
                Type::new(t.elem, Vec::new())
            }
            Op::Shl(x, k) | Op::Shr(x, k) => {
                let t = get(x)?;
                match t.elem {
                    Elem::Int(w) if *k < w => t,
                    _ => {
                        return Err(at(format!(
                            "a shift by {k} needs an integer wider than {k} bits, not {t}"
                        )))
                    }
                }
            }
            Op::Slice(x, s, l) => {
                let t = get(x)?;
                if t.rank() != 1 || s + l > t.count() {
                    return Err(at(format!("slice {s} {l} does not fit in {t}")));
                }
                Type::new(t.elem, vec![*l])
            }
            Op::Concat(a, b) => {
                let (ta, tb) = (get(a)?, get(b)?);
                if ta.elem != tb.elem || ta.rank() != 1 || tb.rank() != 1 {
                    return Err(at(format!(
                        "concat joins two vectors of one element type, not {ta} and {tb}"
                    )));
                }
                Type::new(ta.elem, vec![ta.count() + tb.count()])
            }
            Op::Cast(e, x) => Type::new(*e, get(x)?.shape),
            Op::Lookup(t, x) => {
                let (tt, ti) = (get(t)?, get(x)?);
                if tt.rank() != 1 || tt.count() == 0 {
                    return Err(at(format!(
                        "lookup needs a non-empty vector table, not {tt}"
                    )));
                }
                int(&ti, "lookup's index")?;
                Type::new(tt.elem, ti.shape)
            }
            Op::Put(v, x, y) => {
                let (tv, ti, ty) = (get(v)?, get(x)?, get(y)?);
                if tv.rank() != 1 {
                    return Err(at(format!("put needs a vector, not {tv}")));
                }
                int(&ti, "put's index")?;
                if ti.rank() != 0 || ty.rank() != 0 || ty.elem != tv.elem {
                    return Err(at(format!("put takes one index and one item of {tv}'s element type, not {ti} and {ty}")));
                }
                tv
            }
        };
        if env.contains_key(n.name.as_str()) {
            return Err(at("defined twice".to_string()));
        }
        if matches!(n.op, Op::Stream(..))
            || operands(&n.op)
                .iter()
                .any(|o| streamed.contains(o.as_str()))
        {
            streamed.insert(&n.name);
        }
        env.insert(&n.name, ty.clone());
        out.push((n.name.clone(), ty));
    }

    let states: Vec<&str> = p
        .nodes
        .iter()
        .filter(|n| matches!(n.op, Op::State(..)))
        .map(|n| n.name.as_str())
        .collect();
    let whole = |msg: String| Err(Error::new(0, msg));
    for (s, v) in &p.nexts {
        if !states.contains(&s.as_str()) {
            return whole(format!("next %{s}: %{s} is not a state"));
        }
        let tv = env
            .get(v.as_str())
            .ok_or_else(|| Error::new(0, format!("next %{s} %{v}: %{v} is not defined")))?;
        if *tv != env[s.as_str()] {
            return whole(format!(
                "next %{s} %{v}: %{s} is {} but %{v} is {tv}",
                env[s.as_str()]
            ));
        }
    }
    for s in &states {
        let count = p.nexts.iter().filter(|(t, _)| t == s).count();
        if count != 1 {
            return whole(format!(
                "state %{s} needs exactly one `next`, it has {count}"
            ));
        }
    }
    if states.is_empty() {
        if stream_len.is_some() {
            return whole(
                "a stream needs state to feed (a program with a stream is clocked)".to_string(),
            );
        }
        if p.clocks.is_some() {
            return whole(
                "`clocks` needs state (only a clocked program runs for clocks)".to_string(),
            );
        }
    } else {
        match (stream_len, p.clocks) {
            (Some(l), Some(c)) if l != c => {
                return whole(format!("clocks {c}, but the streams run {l} clocks"))
            }
            (None, None) => {
                return whole("a program with state needs a stream or `clocks N`".to_string())
            }
            _ => {}
        }
    }
    if p.outputs.is_empty() {
        return whole("the program outputs nothing (add `output %name`)".to_string());
    }
    for o in &p.outputs {
        if !env.contains_key(o.name.as_str()) {
            return whole(format!("output %{} is not defined", o.name));
        }
        if o.trace && states.is_empty() {
            return whole(format!(
                "trace %{}: only a clocked program (one with state) has a trace",
                o.name
            ));
        }
        if streamed.contains(o.name.as_str()) {
            return whole(format!(
                "%{} depends on a stream, so it has no value after the last clock",
                o.name
            ));
        }
    }
    Ok(out)
}

fn fits_all(t: &Type, d: &[i64]) -> Result<(), String> {
    match d.iter().find(|v| !fits(t.elem, **v)) {
        Some(v) => Err(format!(
            "{v} does not fit in {}",
            Type::new(t.elem, Vec::new())
        )),
        None => Ok(()),
    }
}

/// Whether a number fits an element type.
pub(crate) fn fits(e: Elem, v: i64) -> bool {
    match e {
        Elem::Bool => v == 0 || v == 1,
        Elem::Int(64) => true,
        Elem::Int(w) => {
            let h = 1i64 << (w - 1);
            -h <= v && v < h
        }
    }
}

/// The shape of operands under scalar extension.
pub(crate) fn extend(shapes: &[&Vec<usize>]) -> Result<Vec<usize>, String> {
    let mut out: Vec<usize> = Vec::new();
    for s in shapes {
        if s.is_empty() {
            continue;
        }
        if out.is_empty() {
            out = (*s).clone();
        } else if out != **s {
            return Err(format!("shapes differ: {out:?} and {s:?}"));
        }
    }
    Ok(out)
}

fn map_type(m: MapOp, ts: &[Type]) -> Result<Type, String> {
    let shape = extend(&ts.iter().map(|t| &t.shape).collect::<Vec<_>>())?;
    let e = ts[0].elem;
    if ts.iter().any(|t| t.elem != e) {
        let names: Vec<String> = ts.iter().map(|t| t.to_string()).collect();
        return Err(format!(
            "{} needs one element type, not {}",
            m.name(),
            names.join(" and ")
        ));
    }
    let ok = match m {
        MapOp::And | MapOp::Or | MapOp::Xor | MapOp::Eq | MapOp::Ne => true,
        MapOp::Not => e == Elem::Bool,
        _ => matches!(e, Elem::Int(_)),
    };
    if !ok {
        return Err(format!("{} does not take {}", m.name(), ts[0]));
    }
    Ok(Type::new(
        if m.is_comparison() { Elem::Bool } else { e },
        shape,
    ))
}
