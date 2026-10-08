//! The netlist: cells, each producing one word of a stated element
//! type, and the simulator that clocks it.

use std::fmt;
use xetal_fpga_xir::{stack, wrap, Elem, MapOp, Type, Value};

/// What a cell computes. Operands are indexes of earlier cells, except
/// a register's `d`, which may be any cell (the loop through the
/// register is how state feeds back).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Element `index` of input port `port`.
    In {
        port: usize,
        index: usize,
    },
    Const(i64),
    Map(MapOp, Vec<usize>),
    /// `sel ? a : b`.
    Mux(usize, usize, usize),
    Shl(usize, u32),
    Shr(usize, u32),
    /// The item of `items` at the address (a mux tree; out of range
    /// gives 0).
    Pick(Vec<usize>, usize),
    /// A constant table read at the address (a ROM; out of range gives 0).
    Rom(Vec<i64>, usize),
    /// The operand as this cell's element type (wrapped, extended, or
    /// tested for 0 into a Bool).
    Cast(usize),
    /// A register: `init` on reset, then `d` at every clock.
    Reg {
        d: usize,
        init: i64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub kind: Kind,
    pub elem: Elem,
}

/// A port: a program input or output, a bus of elements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Port {
    pub name: String,
    pub ty: Type,
    /// The cells of an output's elements (empty for an input).
    pub wires: Vec<usize>,
    /// An output traced: its value after reset and after every clock.
    pub trace: bool,
}

/// A scheduled circuit.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Netlist {
    pub name: String,
    pub cells: Vec<Cell>,
    pub inputs: Vec<Port>,
    /// The data the program gives its inputs (for the simulator and
    /// the testbench).
    pub input_data: Vec<Vec<i64>>,
    /// For a stream input, every clock's item, one after another (the
    /// testbench drives the port with one item per clock).
    pub stream_data: Vec<Option<Vec<i64>>>,
    /// A clocked program's clock count (its state machine runs this
    /// many clocks from reset; `None` otherwise).
    pub clocks: Option<usize>,
    pub outputs: Vec<Port>,
    /// The cell that is 1 when a sequential circuit has finished
    /// (`None` for a combinational one).
    pub done: Option<usize>,
    /// Elements computed at once, and the clocks a sequential circuit
    /// takes.
    pub lanes: usize,
    pub steps: usize,
}

impl Netlist {
    pub(crate) fn add(&mut self, kind: Kind, elem: Elem) -> usize {
        self.cells.push(Cell { kind, elem });
        self.cells.len() - 1
    }

    pub(crate) fn konst(&mut self, v: i64, elem: Elem) -> usize {
        self.add(Kind::Const(v), elem)
    }

    /// Whether the circuit has registers (it then has a clock and a reset).
    pub fn is_sequential(&self) -> bool {
        self.cells
            .iter()
            .any(|c| matches!(c.kind, Kind::Reg { .. }))
    }

    fn eval(&self, data: &[Vec<i64>], state: &[i64]) -> Vec<i64> {
        let mut v = vec![0i64; self.cells.len()];
        for (i, c) in self.cells.iter().enumerate() {
            v[i] = match &c.kind {
                Kind::In { port, index } => data[*port][*index],
                Kind::Const(k) => *k,
                Kind::Map(op, xs) => {
                    apply(*op, v[xs[0]], xs.get(1).map(|x| v[*x]).unwrap_or(0), c.elem)
                }
                Kind::Mux(s, a, b) => {
                    if v[*s] != 0 {
                        v[*a]
                    } else {
                        v[*b]
                    }
                }
                Kind::Shl(a, k) => fit(c.elem, (v[*a] as i128) << k),
                Kind::Shr(a, k) => v[*a] >> k,
                Kind::Pick(items, addr) => {
                    let a = v[*addr];
                    if a >= 0 && (a as usize) < items.len() {
                        v[items[a as usize]]
                    } else {
                        0
                    }
                }
                Kind::Rom(table, addr) => {
                    let a = v[*addr];
                    if a >= 0 && (a as usize) < table.len() {
                        table[a as usize]
                    } else {
                        0
                    }
                }
                Kind::Cast(a) => fit(c.elem, v[*a] as i128),
                Kind::Reg { .. } => state[i],
            };
        }
        v
    }

    /// Run the circuit on its input data: reset, then clock until
    /// `done` (at once for a combinational circuit). Gives the outputs
    /// and the clocks taken.
    pub fn simulate(&self) -> Result<(Vec<Value>, usize), String> {
        let mut state: Vec<i64> = self
            .cells
            .iter()
            .map(|c| {
                if let Kind::Reg { init, .. } = c.kind {
                    init
                } else {
                    0
                }
            })
            .collect();
        if let Some(n) = self.clocks {
            return Ok((self.run_clocked(n, &mut state), n));
        }
        let mut v = self.eval(&self.input_data, &state);
        let mut clocks = 0;
        if let Some(done) = self.done {
            while v[done] == 0 {
                if clocks > self.steps + 2 {
                    return Err(format!(
                        "{} did not finish in {} clocks",
                        self.name,
                        self.steps + 2
                    ));
                }
                for (i, c) in self.cells.iter().enumerate() {
                    if let Kind::Reg { d, .. } = c.kind {
                        state[i] = v[d];
                    }
                }
                clocks += 1;
                v = self.eval(&self.input_data, &state);
            }
        }
        let outs = self
            .outputs
            .iter()
            .map(|p| Value {
                ty: p.ty.clone(),
                data: p.wires.iter().map(|w| v[*w]).collect(),
            })
            .collect();
        Ok((outs, clocks))
    }
}

impl Netlist {
    /// The inputs at a clock: every stream port its item (zeros after
    /// the last clock, when outputs read only state).
    fn data_at(&self, clock: usize, n: usize) -> Vec<Vec<i64>> {
        self.input_data
            .iter()
            .zip(&self.stream_data)
            .zip(&self.inputs)
            .map(|((d, s), p)| match s {
                Some(all) => {
                    let c = p.ty.count().max(1);
                    if clock < n {
                        all[clock * c..(clock + 1) * c].to_vec()
                    } else {
                        vec![0; c]
                    }
                }
                None => d.clone(),
            })
            .collect()
    }

    /// A clocked circuit from reset for `n` clocks: the outputs after
    /// the last, a traced output stacked over reset and every clock.
    fn run_clocked(&self, n: usize, state: &mut [i64]) -> Vec<Value> {
        let read = |p: &Port, v: &[i64]| Value {
            ty: p.ty.clone(),
            data: p.wires.iter().map(|w| v[*w]).collect(),
        };
        let mut traces: Vec<Vec<Value>> = vec![Vec::new(); self.outputs.len()];
        let mut v = Vec::new();
        for clock in 0..=n {
            v = self.eval(&self.data_at(clock, n), state);
            for (p, t) in self.outputs.iter().zip(traces.iter_mut()) {
                if p.trace {
                    t.push(read(p, &v));
                }
            }
            if clock < n {
                for (i, c) in self.cells.iter().enumerate() {
                    if let Kind::Reg { d, .. } = c.kind {
                        state[i] = v[d];
                    }
                }
            }
        }
        self.outputs
            .iter()
            .zip(traces)
            .map(|(p, t)| if p.trace { stack(&t) } else { read(p, &v) })
            .collect()
    }
}

pub(crate) fn fit(e: Elem, x: i128) -> i64 {
    match e {
        Elem::Int(w) => wrap(w, x),
        Elem::Bool => (x != 0) as i64,
    }
}

/// One unit's result, at its width (the IR interpreter's semantics).
pub(crate) fn apply(op: MapOp, a: i64, b: i64, out: Elem) -> i64 {
    let (a, b) = (a as i128, b as i128);
    let r = match op {
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
}

/// The netlist as text, a cell per line (`xetal-fpga netlist`).
impl fmt::Display for Netlist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ty = |e: Elem| Type::new(e, vec![]).to_string();
        for p in &self.inputs {
            writeln!(f, "input {} : {}", p.name, p.ty)?;
        }
        for (i, c) in self.cells.iter().enumerate() {
            let body = match &c.kind {
                Kind::In { port, index } => format!("{}[{index}]", self.inputs[*port].name),
                Kind::Const(k) => format!("const {k}"),
                Kind::Map(op, xs) => {
                    let xs: Vec<String> = xs.iter().map(|x| format!("n{x}")).collect();
                    format!("{} {}", op.name(), xs.join(" "))
                }
                Kind::Mux(s, a, b) => format!("mux n{s} n{a} n{b}"),
                Kind::Shl(a, k) => format!("shl n{a} {k}"),
                Kind::Shr(a, k) => format!("shr n{a} {k}"),
                Kind::Pick(items, a) => {
                    let xs: Vec<String> = items.iter().map(|x| format!("n{x}")).collect();
                    format!("pick n{a} of {}", xs.join(" "))
                }
                Kind::Rom(table, a) => format!("rom n{a} of {} items", table.len()),
                Kind::Cast(a) => format!("cast n{a}"),
                Kind::Reg { d, init } => format!("reg d=n{d} init={init}"),
            };
            writeln!(f, "n{i} : {} = {body}", ty(c.elem))?;
        }
        for p in &self.outputs {
            let ws: Vec<String> = p.wires.iter().map(|w| format!("n{w}")).collect();
            let kw = if p.trace { "trace" } else { "output" };
            writeln!(f, "{kw} {} : {} = {}", p.name, p.ty, ws.join(" "))?;
        }
        if let Some(c) = self.clocks {
            writeln!(f, "clocked: {c} clocks from reset")?;
        }
        if let Some(d) = self.done {
            writeln!(f, "done = n{d} after {} clocks", self.steps)?;
        }
        Ok(())
    }
}
