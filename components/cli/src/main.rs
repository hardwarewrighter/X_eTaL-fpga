//! xetal-fpga: the FPGA tool of X_eTaL-fpga, one command per question
//! about a program in the provisional accelerator IR (components/xir)
//! and the circuit a schedule makes of it (components/rtl).
//!
//!   xetal-fpga check FILE.xir                 # each value's type
//!   xetal-fpga run FILE.xir                   # the outputs, by the IR interpreter
//!   xetal-fpga netlist FILE.xir [--lanes N]   # the scheduled netlist
//!   xetal-fpga verilog FILE.xir [--lanes N]   # the circuit as Verilog-2005
//!   xetal-fpga testbench FILE.xir [--lanes N] # a self-checking testbench for it
//!   xetal-fpga sim FILE.xir [--lanes N]       # the netlist simulated: outputs (clocks on stderr)
//!   xetal-fpga explain FILE.xir [--lanes N]   # units, registers, clocks
//!
//! Without --lanes the schedule is spatial (a unit per element).

use std::path::Path;
use std::process::ExitCode;
use xetal_fpga_rtl::{explain, schedule, testbench, verilog};
use xetal_fpga_xir::{check, parse, run};

const USAGE: &str =
    "usage: xetal-fpga check|run|netlist|verilog|testbench|sim|explain FILE.xir [--lanes N]\n\
     board: xetal-fpga synth|pnr|pack|load|detect [FILE] [--board NAME] [--help]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match go(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xetal-fpga: {e}");
            ExitCode::FAILURE
        }
    }
}

fn go(args: &[String]) -> Result<(), String> {
    if matches!(
        args.first().map(String::as_str),
        Some("synth" | "pnr" | "pack" | "load" | "detect")
    ) {
        let status = std::process::Command::new("python3")
            .arg("-c")
            .arg(include_str!("../../../scripts/board-flow.py"))
            .args(args)
            .status()
            .map_err(|e| format!("board flow requires python3: {e}"))?;
        return if status.success() {
            Ok(())
        } else {
            Err("board tool failed".to_string())
        };
    }
    if matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        println!("{USAGE}");
        return Ok(());
    }
    let (cmd, file, lanes) = match args {
        [c, f] => (c.as_str(), f, None),
        [c, f, l, n] if l == "--lanes" => {
            let n: usize = n
                .parse()
                .map_err(|_| format!("--lanes takes a number, not {n}"))?;
            if n == 0 {
                return Err("--lanes must be at least 1".to_string());
            }
            (c.as_str(), f, Some(n))
        }
        _ => return Err(USAGE.to_string()),
    };
    let src = std::fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let p = parse(&src).map_err(|e| format!("{file}: {e}"))?;
    let name = Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("circuit");
    let net = || schedule(&p, name, lanes).map_err(|e| format!("{file}: {e}"));
    match cmd {
        "check" => {
            for (name, ty) in check(&p).map_err(|e| format!("{file}: {e}"))? {
                println!("%{name} : {ty}");
            }
        }
        "run" => {
            for v in run(&p).map_err(|e| format!("{file}: {e}"))? {
                println!("{v}");
            }
        }
        "netlist" => print!("{}", net()?),
        "verilog" => print!("{}", verilog(&net()?)),
        "testbench" => print!("{}", testbench(&net()?)?),
        "explain" => print!("{}", explain(&net()?)),
        "sim" => {
            let (outs, clocks) = net()?.simulate()?;
            for v in outs {
                println!("{v}");
            }
            eprintln!("clocks: {clocks}");
        }
        _ => return Err(USAGE.to_string()),
    }
    Ok(())
}
