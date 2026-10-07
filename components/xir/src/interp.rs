//! The reference interpreter: X_eTaL's semantics at the stated
//! widths, as the Fixed library computes them (every arithmetic
//! result wrapped to its width), so it reproduces the evaluator's
//! output exactly.

use crate::check::extend;
use crate::{check, Elem, Error, MapOp, Op, Program, RedOp, Type};
use std::collections::HashMap;
use std::fmt;

/// A value: its type and its elements in row-major order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Value {
    pub ty: Type,
    pub data: Vec<i64>,
}

/// `x` wrapped to `w` signed bits (two's complement), as `w fx:w_rap x`.
pub fn wrap(w: u32, x: i128) -> i64 {
    if w >= 64 {
        return x as i64;
    }
    let m = 1i128 << w;
    let u = x.rem_euclid(m);
    (if u >= m / 2 { u - m } else { u }) as i64
}

fn fit(e: Elem, x: i128) -> i64 {
    match e {
        Elem::Int(w) => wrap(w, x),
        Elem::Bool => (x != 0) as i64,
    }
}

/// Each element of the operands, scalars extended, as a column.
fn columns<'a>(vs: &'a [&'a Value], n: usize) -> impl Iterator<Item = Vec<i64>> + 'a {
    (0..n).map(move |i| {
        vs.iter()
            .map(|v| {
                if v.ty.rank() == 0 {
                    v.data[0]
                } else {
                    v.data[i]
                }
            })
            .collect()
    })
}

fn map(m: MapOp, vs: &[&Value]) -> Value {
    let shape = extend(&vs.iter().map(|v| &v.ty.shape).collect::<Vec<_>>()).expect("checked");
    let e = vs[0].ty.elem;
    let out = if m.is_comparison() { Elem::Bool } else { e };
    let n = shape.iter().product();
    let data = columns(vs, n)
        .map(|c| {
            let a = c[0] as i128;
            let b = c.get(1).copied().unwrap_or(0) as i128;
            let r = match m {
                MapOp::Add => a + b,
                MapOp::Sub => a - b,
                MapOp::Mul => a * b,
                MapOp::Min => a.min(b),
                MapOp::Max => a.max(b),
                MapOp::And => a & b,
                MapOp::Or => a | b,
                MapOp::Xor => a ^ b,
                MapOp::Eq => (a == b) as i128,
                MapOp::Ne => (a != b) as i128,
                MapOp::Lt => (a < b) as i128,
                MapOp::Le => (a <= b) as i128,
                MapOp::Gt => (a > b) as i128,
                MapOp::Ge => (a >= b) as i128,
                MapOp::Neg => -a,
                MapOp::Abs => a.abs(),
                MapOp::Not => 1 - a,
            };
            fit(out, r)
        })
        .collect();
    Value {
        ty: Type::new(out, shape),
        data,
    }
}

fn reduce(r: RedOp, v: &Value) -> Value {
    let e = v.ty.elem;
    let it = v.data.iter().map(|&x| x as i128);
    let r = match r {
        RedOp::Add => it.sum(),
        RedOp::Max => it.max().expect("checked"),
        RedOp::Min => it.min().expect("checked"),
        RedOp::And => it.fold(-1, |a, b| a & b),
        RedOp::Or => it.fold(0, |a, b| a | b),
    };
    Value {
        ty: Type::new(e, Vec::new()),
        data: vec![fit(e, r)],
    }
}

/// Check and run a program; give its outputs in order.
pub fn run(p: &Program) -> Result<Vec<Value>, Error> {
    check(p)?;
    let mut env: HashMap<&str, Value> = HashMap::new();
    for n in &p.nodes {
        let get = |x: &String| env.get(x.as_str()).expect("checked");
        let v = match &n.op {
            Op::Input(t, d) | Op::Const(t, d) => Value {
                ty: t.clone(),
                data: d.clone(),
            },
            Op::Map(m, xs) => map(*m, &xs.iter().map(get).collect::<Vec<_>>()),
            Op::Select(m, a, b) => {
                let vs = [get(m), get(a), get(b)];
                let shape =
                    extend(&vs.iter().map(|v| &v.ty.shape).collect::<Vec<_>>()).expect("checked");
                let data = columns(&vs, shape.iter().product())
                    .map(|c| if c[0] != 0 { c[1] } else { c[2] })
                    .collect();
                Value {
                    ty: Type::new(vs[1].ty.elem, shape),
                    data,
                }
            }
            Op::Reduce(r, x) => reduce(*r, get(x)),
            Op::Shl(x, k) => {
                let v = get(x);
                Value {
                    ty: v.ty.clone(),
                    data: v
                        .data
                        .iter()
                        .map(|&a| fit(v.ty.elem, (a as i128) << k))
                        .collect(),
                }
            }
            Op::Shr(x, k) => {
                let v = get(x);
                Value {
                    ty: v.ty.clone(),
                    data: v.data.iter().map(|&a| a >> k).collect(),
                }
            }
            Op::Slice(x, s, l) => {
                let v = get(x);
                Value {
                    ty: Type::new(v.ty.elem, vec![*l]),
                    data: v.data[*s..s + l].to_vec(),
                }
            }
            Op::Concat(a, b) => {
                let (va, vb) = (get(a), get(b));
                let data: Vec<i64> = va.data.iter().chain(&vb.data).copied().collect();
                Value {
                    ty: Type::new(va.ty.elem, vec![data.len()]),
                    data,
                }
            }
        };
        env.insert(&n.name, v);
    }
    Ok(p.outputs.iter().map(|o| env[o.as_str()].clone()).collect())
}

/// A value printed as `xetal` prints it: a scalar as a number, a
/// vector on one line, a matrix a row per line with each column
/// right-aligned (a Bool as 0 or 1).
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cells: Vec<String> = self.data.iter().map(|x| x.to_string()).collect();
        match self.ty.rank() {
            0 | 1 => write!(f, "{}", cells.join(" ")),
            _ => {
                let cols = *self.ty.shape.last().unwrap();
                let width: Vec<usize> = (0..cols)
                    .map(|c| {
                        cells
                            .iter()
                            .skip(c)
                            .step_by(cols.max(1))
                            .map(|s| s.len())
                            .max()
                            .unwrap_or(0)
                    })
                    .collect();
                let rows: Vec<String> = cells
                    .chunks(cols.max(1))
                    .map(|r| {
                        r.iter()
                            .zip(&width)
                            .map(|(s, w)| format!("{s:>w$}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect();
                write!(f, "{}", rows.join("\n"))
            }
        }
    }
}
