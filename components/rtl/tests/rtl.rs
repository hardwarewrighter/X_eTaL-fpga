use xetal_fpga_rtl::{explain, ranges, schedule, schedule_with, testbench, verilog, Kind};
use xetal_fpga_xir::{parse, run, Program};

const TWINS: [&str; 5] = ["vector-add", "saxpy", "threshold", "reduce-sum", "pipeline"];

fn twin(name: &str) -> Program {
    let path = format!(
        "{}/../../libs/Datapath/demos/{name}.xir",
        env!("CARGO_MANIFEST_DIR")
    );
    parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Every twin, every number of lanes from 1 to 9 (8 is spatial for
/// these eight-element programs, 3 and 5 leave lanes idle at the end):
/// the simulated circuit gives what the IR interpreter gives.
#[test]
fn circuits_equal_the_interpreter() {
    for t in TWINS {
        let p = twin(t);
        let want = run(&p).unwrap();
        for lanes in 1..=9 {
            let n = schedule(&p, t, Some(lanes)).unwrap();
            let (got, clocks) = n.simulate().unwrap();
            assert_eq!(got, want, "{t} with {lanes} lane(s)");
            let steps = if lanes >= 8 {
                0
            } else {
                8usize.div_ceil(lanes)
            };
            assert_eq!(clocks, steps, "{t} with {lanes} lane(s)");
        }
    }
}

#[test]
fn eight_adders_spatially_one_temporally() {
    let p = twin("vector-add");
    let e = explain(&schedule(&p, "vector-add", None).unwrap());
    assert!(e.contains("spatial, 8 lane(s), combinational"), "{e}");
    assert!(e.contains("   8 x add i16"), "{e}");
    assert!(e.contains("registers: 0"), "{e}");
    let e = explain(&schedule(&p, "vector-add", Some(1)).unwrap());
    assert!(e.contains("temporal, 1 lane(s), 8 clock(s)"), "{e}");
    assert!(e.contains("   1 x add i16\n"), "{e}");
    assert!(e.contains("registers: 9 (133 bits)"), "{e}");
}

#[test]
fn a_reduction_is_a_tree_of_seven_adders() {
    let p = parse("%v = input i16[8] 1 2 3 4 5 6 7 8\n%s = reduce add %v\noutput %s\n").unwrap();
    let e = explain(&schedule(&p, "sum", None).unwrap());
    assert!(e.contains("   7 x add i16"), "{e}");
    let p = parse("%v = input i16[5] 1 2 3 4 5\n%s = reduce add %v\noutput %s\n").unwrap();
    assert!(explain(&schedule(&p, "sum", None).unwrap()).contains("   4 x add i16"));
}

#[test]
fn temporal_refuses_what_it_cannot_schedule() {
    let p = parse("%v = input i8[4] 1 2 3 4\n%s = slice %v 0 2\noutput %s\n").unwrap();
    assert!(schedule(&p, "s", Some(1))
        .unwrap_err()
        .contains("every vector must have 4 items"));
    let p = parse("%v = input i8[4] 1 2 3 4\n%s = concat %v %v\n%t = slice %s 0 4\noutput %t\n")
        .unwrap();
    assert!(schedule(&p, "s", Some(1)).is_err());
    let p = parse("%v = input i8[4] 1 2 3 4\n%s = reduce add %v\n%d = map sub %v %s\noutput %d\n")
        .unwrap();
    assert!(schedule(&p, "s", Some(2))
        .unwrap_err()
        .contains("before the reduction has finished"));
    assert!(schedule(&p, "s", None).is_ok());
    let p = parse("%v = input i8[4] 1 2 3 4\n%s = reduce add %v\n%t = map add %s %s\noutput %t\n")
        .unwrap();
    let (got, _) = schedule(&p, "s", Some(3)).unwrap().simulate().unwrap();
    assert_eq!(got[0].to_string(), "20");
}

#[test]
fn verilog_is_plain_and_wraps_by_width() {
    let p = twin("vector-add");
    let v = verilog(&schedule(&p, "vector-add", None).unwrap());
    assert!(v.contains("module vector_add ("));
    assert!(v.contains("  input  wire [127:0] in_a,"));
    assert!(v.contains("  output wire [127:0] out_c\n"));
    assert!(v.contains("  wire signed [15:0] n0 = in_a[15:0];"));
    assert!(v.contains(" = n0 + n8;"));
    assert!(!v.contains("always"));
    let v = verilog(&schedule(&p, "vector-add", Some(2)).unwrap());
    assert!(
        v.contains("input  wire clk")
            && v.contains("always @(posedge clk)")
            && v.contains("assign done =")
    );
}

#[test]
fn testbench_expects_the_wrapped_answer() {
    let p = twin("vector-add");
    let tb = testbench(&schedule(&p, "vector-add", None).unwrap()).unwrap();
    // 11 22 33 44 0 0 -32768 -5536, element 0 in the low bits.
    assert!(tb.contains("128'hea60800000000000002c00210016000b"), "{tb}");
    assert!(tb.contains("vector_add dut ("));
}

const CLOCKED: [(&str, &str); 7] = [
    ("Clocked", "counter"),
    ("Clocked", "accumulator"),
    ("Clocked", "shift"),
    ("Clocked", "detect"),
    ("Clocked", "ram"),
    ("Fixed", "counter"),
    ("Uart", "loopback"),
];

fn clocked(lib: &str, name: &str) -> Program {
    let path = format!(
        "{}/../../libs/{lib}/demos/{name}.xir",
        env!("CARGO_MANIFEST_DIR")
    );
    parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Every clocked twin: the circuit, clocked from reset, gives the
/// interpreter's outputs and traces, in as many clocks as its stream.
#[test]
fn clocked_circuits_equal_the_interpreter() {
    for (lib, t) in CLOCKED {
        let p = clocked(lib, t);
        let n = schedule(&p, t, None).unwrap();
        let (got, clocks) = n.simulate().unwrap();
        assert_eq!(got, run(&p).unwrap(), "{lib}/{t}");
        assert_eq!(Some(clocks), xetal_fpga_xir::clocks(&p), "{lib}/{t}");
        assert!(n.is_sequential() && n.done.is_none());
        // One lane is a temporal schedule only when there is a vector
        // longer than one; a clocked program refuses it then.
        match schedule(&p, t, Some(1)) {
            Ok(one) => assert_eq!(one.simulate().unwrap().0, got, "{lib}/{t}"),
            Err(e) => assert!(e.contains("cannot take a clocked program"), "{e}"),
        }
    }
}

#[test]
fn a_constant_table_is_a_rom_and_a_put_is_a_write_port() {
    let p = clocked("Clocked", "detect");
    let n = schedule(&p, "detect", None).unwrap();
    let e = explain(&n);
    assert!(e.contains("clocked, 11 clock(s)"), "{e}");
    assert!(e.contains("   1 x rom 8 x i8"), "{e}");
    assert!(e.contains("registers: 1 (8 bits)"), "{e}");
    let v = verilog(&n);
    assert!(
        v.contains("reg signed [7:0] rom14 [0:7];") && v.contains("rom14[6] = 8'sh3;"),
        "{v}"
    );
    let p = clocked("Clocked", "ram");
    let e = explain(&schedule(&p, "ram", None).unwrap());
    assert!(e.contains("registers: 8 (64 bits)"), "{e}");
    assert!(e.contains("   8 x eq i16"), "{e}");
    // An index of 2 bits (0 and 1 as signed) addresses only words 0 and 1.
    let p = parse("%m = state i8[4] 0 0 0 0\n%a = stream i2 1 0\n%x = const i8 7\n%w = put %m %a %x\nnext %m %w\noutput %m\n").unwrap();
    let n = schedule(&p, "narrow", None).unwrap();
    assert_eq!(n.simulate().unwrap().0[0].to_string(), "7 7 0 0");
    assert!(explain(&n).contains("   2 x eq i2"));
}

#[test]
fn a_clocked_testbench_feeds_one_item_per_clock() {
    let p = clocked("Clocked", "accumulator");
    let tb = testbench(&schedule(&p, "accumulator", None).unwrap()).unwrap();
    let reset = tb.find("    @(posedge clk); #1 rst = 0;\n").unwrap();
    let first = tb
        .find("    in_x = 8'h0a;\n    @(posedge clk); #1;\n")
        .unwrap();
    let second = tb
        .find("    in_x = 8'h14;\n    @(posedge clk); #1;\n")
        .unwrap();
    assert!(reset < first && first < second, "{tb}");
    // The trace is checked after reset (0) and after every clock, the
    // last -96 (0xa0); the final output too (also -96).
    assert!(tb.contains("FAIL: s after reset is"), "{tb}");
    assert!(
        tb.contains("if (out_s !== 8'h0a) begin $display(\"FAIL: s after clock 1 is"),
        "{tb}"
    );
    assert!(tb.contains("FAIL: s after clock 7 is"), "{tb}");
    assert_eq!(tb.matches("if (out_s !== 8'ha0)").count(), 2, "{tb}");
    assert!(tb.contains("$dumpfile(\"accumulator.vcd\");"));
    assert!(!tb.contains("done"));
}

#[test]
fn a_constant_index_is_wiring() {
    let p = parse("%v = input i8[4] 5 6 7 8\n%i = const i8 2\n%r = lookup %v %i\n%x = const i8 9\n%w = put %v %i %x\n%o = const i8 7\n%z = lookup %v %o\noutput %r %w %z\n").unwrap();
    let n = schedule(&p, "wires", None).unwrap();
    assert_eq!(n.simulate().unwrap().0, run(&p).unwrap());
    let e = explain(&n);
    assert!(
        !e.contains("select") && !e.contains("eq") && !e.contains("mux"),
        "{e}"
    );
}

#[test]
fn an_output_also_traced_is_one_port() {
    // Caught by iverilog: `output %s` and `trace %s` once declared out_s twice.
    let p = clocked("Clocked", "accumulator");
    let n = schedule(&p, "accumulator", None).unwrap();
    let v = verilog(&n);
    assert_eq!(v.matches("output wire [7:0] out_s").count(), 1);
    assert_eq!(v.matches("assign out_s").count(), 1);
    let tb = testbench(&n).unwrap();
    assert_eq!(tb.matches("wire [7:0] out_s;").count(), 1);
    assert!(tb.contains("after clock 7"));
}

#[test]
fn casts_change_width_explicitly() {
    let src = "%x = input i16 300
%b = input bool 1
%n = cast i8 %x
%w = cast i32 %x
%e = cast i16 %b
output %n %w %e
";
    let v = verilog(&schedule(&parse(src).unwrap(), "casts", None).unwrap());
    assert!(v.contains("[7:0];"), "{v}");
    assert!(v.contains("{{16{n0[15]}}, n0}"), "{v}");
    assert!(v.contains("{{15{1'b0}}, n1}"), "{v}");
}

/// A RAM of 64 words, written at a counter's address and read back a
/// clock behind through a registered read at the next address, as
/// HEX1000's lowering does; and a 64-word ROM read the same way.
const MEMORY: &str = "\
%x = stream i8 5 6 7 8 9 10 11 12
%ram = state i8[64] 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
%i = state i8 0
%q = state i8 0
%rq = state i8 7
%c63 = const i8 63
%one = const i8 1
%rom = const i8[64] 7 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
%inc = map add %i %one
%i2 = map and %inc %c63
%ram2 = put %ram %i %x
%q2 = lookup %ram %i
%rq2 = lookup %rom %i2
next %ram %ram2
next %i %i2
next %q %q2
next %rq %rq2
trace %q
trace %rq
";

fn memory_program() -> Program {
    parse(MEMORY).unwrap()
}

#[test]
fn memories_are_scheduled_only_when_asked() {
    let p = memory_program();
    let plain = schedule(&p, "mem", None).unwrap();
    let mems = schedule_with(&p, "mem", None, true).unwrap();
    assert!(!plain
        .cells
        .iter()
        .any(|c| matches!(c.kind, Kind::Mem { .. })));
    let rams = mems
        .cells
        .iter()
        .filter(|c| matches!(c.kind, Kind::Mem { .. }))
        .count();
    assert_eq!(rams, 1);
    // Both simulate to the interpreter's traces.
    let want: Vec<String> = run(&p).unwrap().iter().map(|v| v.to_string()).collect();
    for n in [plain, mems] {
        let (got, _) = n.simulate().unwrap();
        let got: Vec<String> = got.iter().map(|v| v.to_string()).collect();
        assert_eq!(got, want);
    }
}

#[test]
fn an_address_always_in_range_is_proven_so() {
    let p = memory_program();
    let n = schedule_with(&p, "mem", None, true).unwrap();
    let r = ranges(&n);
    // The ROM's address (%i2, masked to 63) is within the table.
    let rom_addr = n
        .cells
        .iter()
        .find_map(|c| match c.kind {
            Kind::Rom(_, a) => Some(a),
            _ => None,
        })
        .unwrap();
    assert!(
        r[rom_addr].0 >= 0 && r[rom_addr].1 <= 63,
        "{:?}",
        r[rom_addr]
    );
}

#[test]
fn a_registered_read_becomes_a_block_ram_read() {
    let p = memory_program();
    let v = verilog(&schedule_with(&p, "mem", None, true).unwrap());
    assert!(v.contains("(* ram_style = \"block\" *) reg signed [7:0] mem"));
    // The ROM read, at an address proven in range, is registered; at
    // reset it reads the word holding the register's reset value (7, at 0).
    assert!(v.contains("<= rom"), "{v}");
    assert!(v.contains("[rst ? 8'sh0 : n"), "{v}");
    // The RAM's write is gated by the reset and the range.
    assert!(v.contains("if (!rst && (n"), "{v}");
    // Without the option the RAM is registers and muxes.
    let plain = verilog(&schedule(&p, "mem", None).unwrap());
    assert!(!plain.contains("ram_style"));
}
