//! The hardware side of X_eTaL-fpga: an XIR program (components/xir)
//! is scheduled into a word-level netlist (units of stated widths,
//! muxes, registers), which a cycle simulator runs and a Verilog-2005
//! emitter prints (plan A6, A7, A8).
//!
//! The schedule is data, never part of the program: `lanes` says how
//! many elements of a vector are computed at once. With as many lanes
//! as elements the circuit is spatial and combinational (eight
//! 16-bit numbers added: eight adders, a reduction a tree of adders);
//! with fewer it is temporal: a counter steps through the vector,
//! `lanes` units compute that many elements per clock, outputs are
//! collected in registers and a reduction is a tree of `lanes` units
//! into an accumulator.

mod explain;
mod netlist;
mod schedule;
mod verilog;

pub use explain::explain;
pub use netlist::{Cell, Kind, Netlist, Port};
pub use schedule::schedule;
pub use verilog::{testbench, verilog};
