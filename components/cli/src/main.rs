//! xetal-fpga: the FPGA tool of X_eTaL-fpga. Today it checks and runs
//! programs in the provisional accelerator IR (components/xir); the
//! netlist, Verilog, simulation and explain commands come with
//! components/rtl.
//!
//!   xetal-fpga check FILE.xir   # each value's type
//!   xetal-fpga run FILE.xir     # the outputs, printed as xetal prints

use std::process::ExitCode;
use xetal_fpga_xir::{check, parse, run};

const USAGE: &str = "usage: xetal-fpga check FILE.xir | run FILE.xir";

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
    let (cmd, file) = match args {
        [c, f] => (c.as_str(), f),
        [c] if c == "--help" || c == "-h" => {
            println!("{USAGE}");
            return Ok(());
        }
        _ => return Err(USAGE.to_string()),
    };
    let src = std::fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let p = parse(&src).map_err(|e| format!("{file}: {e}"))?;
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
        _ => return Err(USAGE.to_string()),
    }
    Ok(())
}
