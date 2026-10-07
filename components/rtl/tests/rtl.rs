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
