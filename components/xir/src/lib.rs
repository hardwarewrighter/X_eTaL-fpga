//! The provisional accelerator IR of X_eTaL-fpga (plan A5): a typed
//! dataflow program of named values. Every value has an element type
//! (a signed integer of a stated bit width, or a 1-bit Bool) and a
//! static shape. The IR knows nothing of Verilog: it says what is
//! computed, and the schedule (components/rtl) says how a circuit
//! computes it.
//!
//! It stands in for X_eTaL's own accelerator IR (ask F1) until that
//! exists; its text form is in this crate's README.

mod check;
mod interp;
mod text;

pub use check::{check, clocks, operands};
pub use interp::{run, stack, wrap, Value};
pub use text::{parse, print};

use std::fmt;

/// An element type: a signed two's complement integer of `w` bits
/// (1 to 64), or a Bool (0 or 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elem {
    Int(u32),
    Bool,
}

impl Elem {
    /// The width in bits (a Bool is 1).
    pub fn bits(self) -> u32 {
        match self {
            Elem::Int(w) => w,
            Elem::Bool => 1,
        }
    }
}

/// A value's type: its element type and its static shape (empty for a
/// scalar).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type {
    pub elem: Elem,
    pub shape: Vec<usize>,
}

impl Type {
    pub fn new(elem: Elem, shape: Vec<usize>) -> Self {
        Type { elem, shape }
    }
    /// How many elements a value of this type holds.
    pub fn count(&self) -> usize {
        self.shape.iter().product()
    }
    pub fn rank(&self) -> usize {
        self.shape.len()
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.elem {
            Elem::Int(w) => write!(f, "i{w}")?,
            Elem::Bool => write!(f, "bool")?,
        }
        if !self.shape.is_empty() {
            let dims: Vec<String> = self.shape.iter().map(|d| d.to_string()).collect();
            write!(f, "[{}]", dims.join(","))?;
        }
        Ok(())
    }
}

/// An elementwise operation (scalar extension: a rank-0 operand goes
/// with every element of the other).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapOp {
    Add,
    Sub,
    Mul,
    Min,
    Max,
    And,
    Or,
    Xor,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Neg,
    Abs,
    Not,
}

pub const MAP_OPS: [(MapOp, &str); 17] = [
    (MapOp::Add, "add"),
    (MapOp::Sub, "sub"),
    (MapOp::Mul, "mul"),
    (MapOp::Min, "min"),
    (MapOp::Max, "max"),
    (MapOp::And, "and"),
    (MapOp::Or, "or"),
    (MapOp::Xor, "xor"),
    (MapOp::Eq, "eq"),
    (MapOp::Ne, "ne"),
    (MapOp::Lt, "lt"),
    (MapOp::Le, "le"),
    (MapOp::Gt, "gt"),
    (MapOp::Ge, "ge"),
    (MapOp::Neg, "neg"),
    (MapOp::Abs, "abs"),
    (MapOp::Not, "not"),
];

impl MapOp {
    pub fn name(self) -> &'static str {
        MAP_OPS
            .iter()
            .find(|(o, _)| *o == self)
            .map(|(_, n)| *n)
            .unwrap()
    }
    pub fn from_name(s: &str) -> Option<Self> {
        MAP_OPS.iter().find(|(_, n)| *n == s).map(|(o, _)| *o)
    }
    /// How many operands it takes.
    pub fn arity(self) -> usize {
        match self {
            MapOp::Neg | MapOp::Abs | MapOp::Not => 1,
            _ => 2,
        }
    }
    /// Whether it compares, giving a Bool.
    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            MapOp::Eq | MapOp::Ne | MapOp::Lt | MapOp::Le | MapOp::Gt | MapOp::Ge
        )
    }
}

/// A reduction of a vector to a scalar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedOp {
    Add,
    Max,
    Min,
    And,
    Or,
}

pub const RED_OPS: [(RedOp, &str); 5] = [
    (RedOp::Add, "add"),
    (RedOp::Max, "max"),
    (RedOp::Min, "min"),
    (RedOp::And, "and"),
    (RedOp::Or, "or"),
];

impl RedOp {
    pub fn name(self) -> &'static str {
        RED_OPS
            .iter()
            .find(|(o, _)| *o == self)
            .map(|(_, n)| *n)
            .unwrap()
    }
    pub fn from_name(s: &str) -> Option<Self> {
        RED_OPS.iter().find(|(_, n)| *n == s).map(|(o, _)| *o)
    }
}

/// What a node computes. Operands name earlier nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    /// A program input with its type and data (the examples carry
    /// their data, as the X_eTaL programs do).
    Input(Type, Vec<i64>),
    /// A constant.
    Const(Type, Vec<i64>),
    Map(MapOp, Vec<String>),
    /// `select m a b`: a where the Bool m is 1, b elsewhere (a mux).
    Select(String, String, String),
    Reduce(RedOp, String),
    /// Shift left by a constant number of bits (wiring), wrapped.
    Shl(String, u32),
    /// Arithmetic shift right by a constant (floor division by 2^k).
    Shr(String, u32),
    /// `slice x start len` of a vector (start from 0).
    Slice(String, usize, usize),
    /// Two vectors joined.
    Concat(String, String),
    /// `lookup %table %i`: the item of a vector at each index (from 0;
    /// out of range gives 0). A constant table is a ROM.
    Lookup(String, String),
    /// `put %v %i %x`: the vector with the item at the index replaced
    /// (out of range: unchanged). With a state vector, a RAM's write.
    Put(String, String, String),
    /// `cast E %x`: each item as element type E (an integer wrapped to
    /// E's width, sign-extended when wider; a Bool is 0 or 1; to Bool,
    /// whether it is not 0). Only the element type is written: `cast i8 %x`.
    Cast(Elem, String),
    /// `state T init...`: registers, their value at reset; the value
    /// after each clock is given by a `next` binding.
    State(Type, Vec<i64>),
    /// `stream T items...`: one item of type T per clock (an input
    /// port in hardware; the data is the testbench's).
    Stream(Type, Vec<i64>),
}

/// A node: a name (written `%name`) bound to an operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub op: Op,
}

/// A program: nodes in order (each refers only to earlier ones) and
/// the values it outputs, in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Program {
    pub nodes: Vec<Node>,
    /// `next %state %value`: what each state holds after a clock.
    pub nexts: Vec<(String, String)>,
    /// `clocks N`: how many clocks to run a program with state but no
    /// stream (with a stream, its length).
    pub clocks: Option<usize>,
    pub outputs: Vec<Output>,
}

/// An output: a value after the last clock, or (`trace`) its value
/// after reset and after every clock, stacked along a new first axis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub name: String,
    pub trace: bool,
}

impl Output {
    pub fn new(name: &str) -> Self {
        Output {
            name: name.to_string(),
            trace: false,
        }
    }
}

/// An error with the line it is about (0 when it is about the
/// program as a whole).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub line: usize,
    pub msg: String,
}

impl Error {
    pub fn new(line: usize, msg: impl Into<String>) -> Self {
        Error {
            line,
            msg: msg.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line > 0 {
            write!(f, "line {}: {}", self.line, self.msg)
        } else {
            write!(f, "{}", self.msg)
        }
    }
}

impl std::error::Error for Error {}
