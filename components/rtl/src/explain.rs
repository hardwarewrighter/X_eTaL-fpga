//! What a schedule made of a program: the units by kind and width,
//! the registers, the clocks (`xetal-fpga explain`).

use crate::netlist::{Kind, Netlist};
use std::collections::BTreeMap;
use xetal_fpga_xir::{Elem, Type};

pub fn explain(n: &Netlist) -> String {
    let ty = |e: Elem| Type::new(e, vec![]).to_string();
    let mut units: BTreeMap<String, usize> = BTreeMap::new();
    let mut regs = 0;
    let mut reg_bits = 0;
    for c in &n.cells {
        let key = match &c.kind {
            // A comparator is named by what it compares, not its 1-bit result.
            Kind::Map(op, xs) if op.is_comparison() => {
                format!("{} {}", op.name(), ty(n.cells[xs[0]].elem))
            }
            Kind::Map(op, _) => format!("{} {}", op.name(), ty(c.elem)),
            Kind::Mux(..) => format!("mux {}", ty(c.elem)),
            Kind::Pick(items, _) => format!("select 1 of {} {}", items.len(), ty(c.elem)),
            Kind::Shl(..) | Kind::Shr(..) => format!("shift {} (wiring)", ty(c.elem)),
            Kind::Reg { .. } => {
                regs += 1;
                reg_bits += c.elem.bits();
                continue;
            }
            Kind::Rom(table, _) => format!("rom {} x {}", table.len(), ty(c.elem)),
            Kind::Cast(a) => format!("cast {} to {} (wiring)", ty(n.cells[*a].elem), ty(c.elem)),
            Kind::Mem { init, .. } => format!("ram {} x {} (a memory)", init.len(), ty(c.elem)),
            Kind::MemRead(..) => format!("ram read {}", ty(c.elem)),
            Kind::In { .. } | Kind::Const(_) => continue,
        };
        *units.entry(key).or_default() += 1;
    }
    let mut s = String::new();
    s.push_str(&format!("circuit: {}\n", n.name));
    let total: usize = n.outputs.iter().map(|p| p.ty.count()).sum::<usize>().max(1);
    if let Some(c) = n.clocks {
        s.push_str(&format!(
            "schedule: clocked, {c} clock(s) from reset, one stream item per clock\n"
        ));
    } else if n.done.is_some() {
        s.push_str(&format!(
            "schedule: temporal, {} lane(s), {} clock(s), a counter steps through the vector\n",
            n.lanes, n.steps
        ));
    } else {
        s.push_str(&format!(
            "schedule: spatial, {} lane(s), combinational (no clock)\n",
            n.lanes
        ));
    }
    s.push_str(&format!(
        "inputs: {}\n",
        n.inputs
            .iter()
            .map(|p| format!("{} {}", p.name, p.ty))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    s.push_str(&format!(
        "outputs: {} ({} element(s))\n",
        n.outputs
            .iter()
            .map(|p| format!("{} {}", p.name, p.ty))
            .collect::<Vec<_>>()
            .join(", "),
        total
    ));
    s.push_str("units:\n");
    for (k, v) in &units {
        s.push_str(&format!("  {v:>4} x {k}\n"));
    }
    s.push_str(&format!("registers: {regs} ({reg_bits} bits)\n"));
    s
}
