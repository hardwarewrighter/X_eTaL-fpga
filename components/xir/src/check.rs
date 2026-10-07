//! Checking: every operand defined before use, names unique, element
//! types and widths agreeing, shapes agreeing (or one operand a
//! scalar), data fitting its type. Gives each node's type.

use crate::{Elem, Error, MapOp, Op, Program, RedOp, Type};
use std::collections::HashMap;

/// Check a program and give the type of every node, in order.
pub fn check(p: &Program) -> Result<Vec<(String, Type)>, Error> {
    let mut env: HashMap<&str, Type> = HashMap::new();
    let mut out = Vec::new();
    for (i, n) in p.nodes.iter().enumerate() {
        let at = |msg: String| Error::new(0, format!("%{} (node {}): {msg}", n.name, i + 1));
        let get = |x: &str| {
            env.get(x)
                .cloned()
                .ok_or_else(|| at(format!("%{x} is not defined before it is used")))
        };
        let ty = match &n.op {
            Op::Input(t, d) | Op::Const(t, d) => {
                if d.len() != t.count() {
                    return Err(at(format!(
                        "{t} holds {} items, {} given",
                        t.count(),
                        d.len()
                    )));
                }
                if let Some(v) = d.iter().find(|v| !fits(t.elem, **v)) {
                    return Err(at(format!("{v} does not fit in {}", t.elem_name())));
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
                match (r, t.elem) {
                    (RedOp::And | RedOp::Or, Elem::Int(_))
                    | (RedOp::Add | RedOp::Max | RedOp::Min, Elem::Int(_)) => {}
                    (RedOp::And | RedOp::Or, Elem::Bool) => {}
                    _ => return Err(at(format!("reduce {} does not take {t}", r.name()))),
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
        };
        if env.contains_key(n.name.as_str()) {
            return Err(at("defined twice".to_string()));
        }
        env.insert(&n.name, ty.clone());
        out.push((n.name.clone(), ty));
    }
    if p.outputs.is_empty() {
        return Err(Error::new(
            0,
            "the program outputs nothing (add `output %name`)",
        ));
    }
    for o in &p.outputs {
        if !env.contains_key(o.as_str()) {
            return Err(Error::new(0, format!("output %{o} is not defined")));
        }
    }
    Ok(out)
}

impl Type {
    fn elem_name(&self) -> String {
        Type::new(self.elem, Vec::new()).to_string()
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
