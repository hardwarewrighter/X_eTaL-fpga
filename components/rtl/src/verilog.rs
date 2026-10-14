//! The Verilog-2005 emitter (plan A7): plain `assign`, `always @(posedge
//! clk)`, no vendor primitives, so any simulator and Yosys read it. A
//! port is a bus of its elements, element 0 in the low bits. Integers
//! are `signed` wires of their width, so the arithmetic wraps as the
//! program's does; a Bool is a 1-bit wire.

use crate::netlist::{Kind, Netlist, Port};
use xetal_fpga_xir::{wrap, Elem, MapOp};

fn decl(e: Elem) -> String {
    match e {
        Elem::Bool => "wire".to_string(),
        Elem::Int(w) => format!("wire signed [{}:0]", w - 1),
    }
}

/// A literal of an element type: its bits in hexadecimal.
fn lit(e: Elem, v: i64) -> String {
    match e {
        Elem::Bool => format!("1'b{}", v & 1),
        Elem::Int(w) => {
            let bits = if w >= 64 {
                v as u64
            } else {
                (v as u64) & ((1u64 << w) - 1)
            };
            format!("{w}'sh{bits:x}")
        }
    }
}

/// A bus literal: the elements, element 0 in the low bits.
fn bus_lit(e: Elem, data: &[i64]) -> String {
    let w = e.bits() as usize;
    let total = w * data.len();
    let mut bits = vec![false; total];
    for (i, v) in data.iter().enumerate() {
        let u = if e == Elem::Bool {
            *v as u64 & 1
        } else {
            wrap(w as u32, *v as i128) as u64
        };
        for b in 0..w {
            bits[i * w + b] = (u >> b) & 1 == 1;
        }
    }
    let mut hex = String::new();
    for chunk in (0..total.div_ceil(4)).rev() {
        let mut d = 0;
        for b in 0..4 {
            let i = chunk * 4 + b;
            if i < total && bits[i] {
                d |= 1 << b;
            }
        }
        hex.push(std::char::from_digit(d, 16).unwrap());
    }
    format!("{total}'h{hex}")
}

/// The output ports, each name once: an output that is also traced
/// (`output %s` and `trace %s`) is one port, checked both ways.
fn output_ports(n: &Netlist) -> Vec<&Port> {
    let mut seen = Vec::new();
    n.outputs
        .iter()
        .filter(|p| {
            let new = !seen.contains(&&p.name);
            seen.push(&p.name);
            new
        })
        .collect()
}

fn bus_width(e: Elem, count: usize) -> usize {
    e.bits() as usize * count
}

fn expr(n: &Netlist, i: usize) -> String {
    let c = &n.cells[i];
    let w = |x: &usize| format!("n{x}");
    match &c.kind {
        Kind::In { port, index } => {
            let p = &n.inputs[*port];
            let b = p.ty.elem.bits() as usize;
            if b == 1 && p.ty.count() == 1 {
                format!("in_{}", p.name)
            } else {
                format!("in_{}[{}:{}]", p.name, (index + 1) * b - 1, index * b)
            }
        }
        Kind::Const(v) => lit(c.elem, *v),
        Kind::Map(op, xs) => {
            let a = w(&xs[0]);
            let b = xs.get(1).map(w).unwrap_or_default();
            match op {
                MapOp::Add => format!("{a} + {b}"),
                MapOp::Sub => format!("{a} - {b}"),
                MapOp::Mul => format!("{a} * {b}"),
                MapOp::Min => format!("({a} < {b}) ? {a} : {b}"),
                MapOp::Max => format!("({a} > {b}) ? {a} : {b}"),
                MapOp::And => format!("{a} & {b}"),
                MapOp::Or => format!("{a} | {b}"),
                MapOp::Xor => format!("{a} ^ {b}"),
                MapOp::Eq => format!("{a} == {b}"),
                MapOp::Ne => format!("{a} != {b}"),
                MapOp::Lt => format!("{a} < {b}"),
                MapOp::Le => format!("{a} <= {b}"),
                MapOp::Gt => format!("{a} > {b}"),
                MapOp::Ge => format!("{a} >= {b}"),
                MapOp::Neg => format!("-{a}"),
                MapOp::Abs => format!("({a} < 0) ? -{a} : {a}"),
                MapOp::Not => format!("~{a}"),
            }
        }
        Kind::Mux(s, a, b) => format!("n{s} ? n{a} : n{b}"),
        Kind::Shl(a, k) => format!("n{a} <<< {k}"),
        Kind::Shr(a, k) => format!("n{a} >>> {k}"),
        Kind::Pick(items, a) => {
            let mut s = String::new();
            for (k, x) in items.iter().enumerate() {
                s.push_str(&format!("(n{a} == {k}) ? n{x} : "));
            }
            s.push_str(&lit(c.elem, 0));
            s
        }
        Kind::Rom(table, a) => format!(
            "((n{a} >= 0) && (n{a} < {})) ? rom{i}[n{a}] : {}",
            table.len(),
            lit(c.elem, 0)
        ),
        // Widths change explicitly: an Int is cut to its low bits or
        // sign-extended, a Bool zero-extended.
        Kind::Cast(a) => match (n.cells[*a].elem, c.elem) {
            (_, Elem::Bool) => format!("n{a} != 0"),
            (Elem::Bool, Elem::Int(1)) => format!("n{a}"),
            (Elem::Bool, Elem::Int(w)) => format!("{{{{{}{{1'b0}}}}, n{a}}}", w - 1),
            (Elem::Int(f), Elem::Int(w)) if w < f => format!("n{a}[{}:0]", w - 1),
            (Elem::Int(f), Elem::Int(w)) if w > f => {
                format!("{{{{{}{{n{a}[{}]}}}}, n{a}}}", w - f, f - 1)
            }
            _ => format!("n{a}"),
        },
        Kind::Reg { .. } => unreachable!("registers are declared as reg"),
    }
}

/// The circuit as a Verilog-2005 module.
pub fn verilog(n: &Netlist) -> String {
    let mut v = String::new();
    v.push_str(&format!(
        "// {}: generated by xetal-fpga from an X_eTaL program; do not edit.\n",
        n.name
    ));
    if n.clocks.is_some() {
        v.push_str("// Clocked: registers from reset, one stream item per clock.\n");
    } else if n.done.is_some() {
        v.push_str(&format!(
            "// Temporal schedule: {} lane(s), done after {} clock(s) from reset.\n",
            n.lanes, n.steps
        ));
    } else {
        v.push_str(&format!(
            "// Spatial schedule: {} lane(s), combinational.\n",
            n.lanes
        ));
    }
    let mut ports = Vec::new();
    if n.is_sequential() {
        ports.push("  input  wire clk".to_string());
        ports.push("  input  wire rst".to_string());
    }
    for p in &n.inputs {
        ports.push(format!(
            "  input  wire [{}:0] in_{}",
            bus_width(p.ty.elem, p.ty.count()) - 1,
            p.name
        ));
    }
    for p in output_ports(n) {
        ports.push(format!(
            "  output wire [{}:0] out_{}",
            bus_width(p.ty.elem, p.ty.count()) - 1,
            p.name
        ));
    }
    if n.done.is_some() {
        ports.push("  output wire done".to_string());
    }
    v.push_str(&format!("module {} (\n{}\n);\n", n.name, ports.join(",\n")));
    for (i, c) in n.cells.iter().enumerate() {
        if let Kind::Reg { .. } = c.kind {
            let r = match c.elem {
                Elem::Bool => "reg".to_string(),
                Elem::Int(w) => format!("reg signed [{}:0]", w - 1),
            };
            v.push_str(&format!("  {r} n{i};\n"));
        }
    }
    // A constant table is a ROM: a memory initialized once (Yosys maps
    // it to LUTs, or to block RAM when the read is registered).
    for (i, c) in n.cells.iter().enumerate() {
        if let Kind::Rom(table, _) = &c.kind {
            let r = match c.elem {
                Elem::Bool => "reg".to_string(),
                Elem::Int(w) => format!("reg signed [{}:0]", w - 1),
            };
            v.push_str(&format!(
                "  {r} rom{i} [0:{}];\n  initial begin\n",
                table.len() - 1
            ));
            for (k, x) in table.iter().enumerate() {
                v.push_str(&format!("    rom{i}[{k}] = {};\n", lit(c.elem, *x)));
            }
            v.push_str("  end\n");
        }
    }
    for (i, c) in n.cells.iter().enumerate() {
        if !matches!(c.kind, Kind::Reg { .. }) {
            v.push_str(&format!("  {} n{i} = {};\n", decl(c.elem), expr(n, i)));
        }
    }
    let regs: Vec<(usize, usize, i64)> = n
        .cells
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if let Kind::Reg { d, init } = c.kind {
                Some((i, d, init))
            } else {
                None
            }
        })
        .collect();
    if !regs.is_empty() {
        v.push_str("  always @(posedge clk) begin\n    if (rst) begin\n");
        for (i, _, init) in &regs {
            v.push_str(&format!(
                "      n{i} <= {};\n",
                lit(n.cells[*i].elem, *init)
            ));
        }
        v.push_str("    end else begin\n");
        for (i, d, _) in &regs {
            v.push_str(&format!("      n{i} <= n{d};\n"));
        }
        v.push_str("    end\n  end\n");
    }
    for p in output_ports(n) {
        let ws: Vec<String> = p.wires.iter().rev().map(|w| format!("n{w}")).collect();
        v.push_str(&format!(
            "  assign out_{} = {{{}}};\n",
            p.name,
            ws.join(", ")
        ));
    }
    if let Some(d) = n.done {
        v.push_str(&format!("  assign done = n{d};\n"));
    }
    v.push_str("endmodule\n");
    v
}

/// One output compared with expected bits; a failure names it and the clock.
fn check(name: &str, bits: &str, at: &str) -> String {
    format!(
        "    if (out_{name} !== {bits}) begin $display(\"FAIL: {name}{at} is %h, expected %h\", out_{name}, {bits}); fails = fails + 1; end\n"
    )
}

/// A self-checking testbench: the program's input data in, the
/// expected outputs (the simulator's, which equal the evaluator's)
/// compared bit for bit; prints PASS or FAIL lines. A clocked module is
/// reset and driven with one stream item per clock, and every traced
/// output is checked after reset and after every clock. The run writes
/// a waveform, NAME.vcd, in the directory it runs in.
pub fn testbench(n: &Netlist) -> Result<String, String> {
    let (want, _) = n.simulate()?;
    let mut v = String::new();
    v.push_str(&format!(
        "// Testbench for {}: generated by xetal-fpga; prints PASS or FAIL.\n`timescale 1ns/1ps\nmodule tb;\n",
        n.name
    ));
    let mut conns = Vec::new();
    if n.is_sequential() {
        v.push_str("  reg clk = 0;\n  reg rst = 1;\n  always #5 clk = ~clk;\n");
        conns.push(".clk(clk)".to_string());
        conns.push(".rst(rst)".to_string());
    }
    if n.done.is_some() {
        v.push_str("  wire done;\n");
        conns.push(".done(done)".to_string());
    }
    for (p, d) in n.inputs.iter().zip(&n.input_data) {
        let w = bus_width(p.ty.elem, p.ty.count());
        v.push_str(&format!(
            "  reg [{}:0] in_{} = {};\n",
            w - 1,
            p.name,
            bus_lit(p.ty.elem, d)
        ));
        conns.push(format!(".in_{0}(in_{0})", p.name));
    }
    for p in output_ports(n) {
        let w = bus_width(p.ty.elem, p.ty.count());
        v.push_str(&format!("  wire [{}:0] out_{};\n", w - 1, p.name));
        conns.push(format!(".out_{0}(out_{0})", p.name));
    }
    v.push_str(&format!("  {} dut ({});\n", n.name, conns.join(", ")));
    v.push_str(&format!(
        "  initial begin\n    $dumpfile(\"{0}.vcd\");\n    $dumpvars(0, tb);\n  end\n",
        n.name
    ));
    v.push_str("  integer fails = 0;\n  initial begin\n");
    // A traced output's expected bits at a row (0: after reset).
    let row = |k: usize, i: usize| {
        let c = n.outputs[k].ty.count();
        bus_lit(n.outputs[k].ty.elem, &want[k].data[i * c..(i + 1) * c])
    };
    let traces = |v: &mut String, i: usize, at: &str| {
        for (k, p) in n.outputs.iter().enumerate() {
            if p.trace {
                v.push_str(&check(&p.name, &row(k, i), at));
            }
        }
    };
    if let Some(clocks) = n.clocks {
        // Reset, then one stream item per clock.
        v.push_str("    @(posedge clk); #1 rst = 0;\n");
        traces(&mut v, 0, " after reset");
        for clock in 0..clocks {
            for (p, s) in n.inputs.iter().zip(&n.stream_data) {
                if let Some(all) = s {
                    let c = p.ty.count().max(1);
                    let item = &all[clock * c..(clock + 1) * c];
                    v.push_str(&format!(
                        "    in_{} = {};\n",
                        p.name,
                        bus_lit(p.ty.elem, item)
                    ));
                }
            }
            v.push_str("    @(posedge clk); #1;\n");
            traces(&mut v, clock + 1, &format!(" after clock {}", clock + 1));
        }
    } else if n.done.is_some() {
        v.push_str(&format!(
            "    @(posedge clk); #1 rst = 0;\n    repeat ({}) @(posedge clk);\n    #1;\n    if (done !== 1'b1) begin $display(\"FAIL: not done after {} clocks\"); fails = fails + 1; end\n",
            n.steps, n.steps
        ));
    } else {
        v.push_str("    #1;\n");
    }
    for (p, val) in n.outputs.iter().zip(&want) {
        if !p.trace {
            v.push_str(&check(&p.name, &bus_lit(p.ty.elem, &val.data), ""));
        }
    }
    v.push_str(&format!(
        "    if (fails == 0) $display(\"PASS: {}\");\n    $finish;\n  end\nendmodule\n",
        n.name
    ));
    Ok(v)
}
