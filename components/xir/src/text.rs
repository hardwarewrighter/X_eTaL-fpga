//! The text form: one node per line, `#` comments, outputs last.
//!
//! ```text
//! %a = input i16[8] 1 2 3 4 100 -100 32767 30000
//! %b = input i16[8] 10 20 30 40 -100 100 1 30000
//! %c = map add %a %b
//! output %c
//! ```

use crate::{Elem, Error, MapOp, Node, Op, Program, RedOp, Type};

/// Parse a type: `i16`, `i8[8]`, `bool[4]`, `i32[2,3]`.
pub fn parse_type(s: &str) -> Option<Type> {
    let (base, shape) = match s.find('[') {
        Some(i) => {
            let dims = s[i + 1..].strip_suffix(']')?;
            let shape: Option<Vec<usize>> =
                dims.split(',').map(|d| d.trim().parse().ok()).collect();
            (&s[..i], shape?)
        }
        None => (s, Vec::new()),
    };
    let elem = if base == "bool" {
        Elem::Bool
    } else {
        let w: u32 = base.strip_prefix('i')?.parse().ok()?;
        if !(1..=64).contains(&w) {
            return None;
        }
        Elem::Int(w)
    };
    Some(Type::new(elem, shape))
}

fn var(tok: &str, line: usize) -> Result<String, Error> {
    match tok.strip_prefix('%') {
        Some(n) if !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') => {
            Ok(n.to_string())
        }
        _ => Err(Error::new(
            line,
            format!("expected a value name such as %a, found `{tok}`"),
        )),
    }
}

fn number<T: std::str::FromStr>(tok: Option<&&str>, line: usize, what: &str) -> Result<T, Error> {
    tok.and_then(|t| t.parse().ok())
        .ok_or_else(|| Error::new(line, format!("expected {what}")))
}

fn arity(toks: &[&str], n: usize, line: usize, op: &str) -> Result<(), Error> {
    if toks.len() == n {
        Ok(())
    } else {
        Err(Error::new(
            line,
            format!("{op} takes {n} operand(s), found {}", toks.len()),
        ))
    }
}

/// Parse a program from its text.
pub fn parse(src: &str) -> Result<Program, Error> {
    let mut p = Program::default();
    for (i, raw) in src.lines().enumerate() {
        let line = i + 1;
        let text = raw.split('#').next().unwrap_or("").trim();
        if text.is_empty() {
            continue;
        }
        let toks: Vec<&str> = text.split_whitespace().collect();
        if toks[0] == "output" {
            if toks.len() < 2 {
                return Err(Error::new(line, "output names one or more values"));
            }
            for t in &toks[1..] {
                p.outputs.push(var(t, line)?);
            }
            continue;
        }
        if !p.outputs.is_empty() {
            return Err(Error::new(line, "outputs come after every node"));
        }
        if toks.len() < 3 || toks[1] != "=" {
            return Err(Error::new(
                line,
                "expected `%name = op ...` or `output %name ...`",
            ));
        }
        let name = var(toks[0], line)?;
        let (op, args) = (toks[2], &toks[3..]);
        let op = match op {
            "input" | "const" => {
                let ty = args
                    .first()
                    .and_then(|t| parse_type(t))
                    .ok_or_else(|| Error::new(line, format!("{op} needs a type such as i16[8]")))?;
                let data: Result<Vec<i64>, _> =
                    args[1..].iter().map(|t| t.parse::<i64>()).collect();
                let data = data.map_err(|_| Error::new(line, "data must be whole numbers"))?;
                if op == "input" {
                    Op::Input(ty, data)
                } else {
                    Op::Const(ty, data)
                }
            }
            "map" => {
                let m = args
                    .first()
                    .and_then(|t| MapOp::from_name(t))
                    .ok_or_else(|| Error::new(line, "map needs an operation: add sub mul min max and or xor eq ne lt le gt ge neg abs not"))?;
                arity(&args[1..], m.arity(), line, m.name())?;
                Op::Map(
                    m,
                    args[1..]
                        .iter()
                        .map(|t| var(t, line))
                        .collect::<Result<_, _>>()?,
                )
            }
            "select" => {
                arity(args, 3, line, "select")?;
                Op::Select(
                    var(args[0], line)?,
                    var(args[1], line)?,
                    var(args[2], line)?,
                )
            }
            "reduce" => {
                let r = args
                    .first()
                    .and_then(|t| RedOp::from_name(t))
                    .ok_or_else(|| {
                        Error::new(line, "reduce needs an operation: add max min and or")
                    })?;
                arity(&args[1..], 1, line, "reduce")?;
                Op::Reduce(r, var(args[1], line)?)
            }
            "shl" | "shr" => {
                arity(args, 2, line, op)?;
                let k: u32 = number(args.get(1), line, "a shift amount")?;
                let x = var(args[0], line)?;
                if op == "shl" {
                    Op::Shl(x, k)
                } else {
                    Op::Shr(x, k)
                }
            }
            "slice" => {
                arity(args, 3, line, "slice")?;
                Op::Slice(
                    var(args[0], line)?,
                    number(args.get(1), line, "a start")?,
                    number(args.get(2), line, "a length")?,
                )
            }
            "concat" => {
                arity(args, 2, line, "concat")?;
                Op::Concat(var(args[0], line)?, var(args[1], line)?)
            }
            other => return Err(Error::new(line, format!("unknown operation `{other}`"))),
        };
        p.nodes.push(Node { name, op });
    }
    Ok(p)
}

/// Print a program in its canonical text form (comments dropped);
/// `parse(&print(&p))` gives `p` back.
pub fn print(p: &Program) -> String {
    let mut out = String::new();
    for n in &p.nodes {
        let body = match &n.op {
            Op::Input(t, d) | Op::Const(t, d) => {
                let kw = if matches!(n.op, Op::Input(..)) {
                    "input"
                } else {
                    "const"
                };
                let mut s = format!("{kw} {t}");
                for v in d {
                    s.push_str(&format!(" {v}"));
                }
                s
            }
            Op::Map(m, xs) => {
                let xs: Vec<String> = xs.iter().map(|x| format!("%{x}")).collect();
                format!("map {} {}", m.name(), xs.join(" "))
            }
            Op::Select(m, a, b) => format!("select %{m} %{a} %{b}"),
            Op::Reduce(r, x) => format!("reduce {} %{x}", r.name()),
            Op::Shl(x, k) => format!("shl %{x} {k}"),
            Op::Shr(x, k) => format!("shr %{x} {k}"),
            Op::Slice(x, s, l) => format!("slice %{x} {s} {l}"),
            Op::Concat(a, b) => format!("concat %{a} %{b}"),
        };
        out.push_str(&format!("%{} = {}\n", n.name, body));
    }
    if !p.outputs.is_empty() {
        let xs: Vec<String> = p.outputs.iter().map(|x| format!("%{x}")).collect();
        out.push_str(&format!("output {}\n", xs.join(" ")));
    }
    out
}
