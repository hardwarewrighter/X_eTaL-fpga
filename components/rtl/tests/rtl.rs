use xetal_fpga_rtl::{explain, schedule, testbench, verilog};
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
    assert!(tb.contains("    @(posedge clk); #1 rst = 0;\n    in_x = 8'h0a;\n    @(posedge clk); #1;\n    in_x = 8'h14;"), "{tb}");
    // The trace's last value, -96, is what is checked: 0xa0.
    assert!(tb.contains("if (out_s !== 8'ha0)"), "{tb}");
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
