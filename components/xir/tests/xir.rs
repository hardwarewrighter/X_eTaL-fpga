use xetal_fpga_xir::{check, parse, print, run, wrap, Elem, Type};

fn out(src: &str) -> Vec<String> {
    run(&parse(src).unwrap())
        .unwrap()
        .iter()
        .map(|v| v.to_string())
        .collect()
}

fn err(src: &str) -> String {
    match parse(src) {
        Err(e) => e.to_string(),
        Ok(p) => run(&p).unwrap_err().to_string(),
    }
}

const VADD: &str = "# eight 16-bit adders
%a = input i16[8] 1 2 3 4 100 -100 32767 30000
%b = input i16[8] 10 20 30 40 -100 100 1 30000
%c = map add %a %b   # the sum
output %c
";

#[test]
fn wrap_is_twos_complement() {
    assert_eq!(wrap(8, 200), -56);
    assert_eq!(wrap(8, -129), 127);
    assert_eq!(wrap(8, 256), 0);
    assert_eq!(wrap(16, 40000), -25536);
    assert_eq!(wrap(16, 65535), -1);
    assert_eq!(wrap(64, -5), -5);
    assert_eq!(wrap(1, 1), -1);
}

#[test]
fn vector_add_wraps_at_sixteen_bits() {
    assert_eq!(out(VADD), ["11 22 33 44 0 0 -32768 -5536"]);
}

#[test]
fn text_round_trips() {
    let p = parse(VADD).unwrap();
    let text = print(&p);
    assert_eq!(parse(&text).unwrap(), p);
    assert_eq!(text, "%a = input i16[8] 1 2 3 4 100 -100 32767 30000\n%b = input i16[8] 10 20 30 40 -100 100 1 30000\n%c = map add %a %b\noutput %c\n");
    let every = "%x = input i8[4] 1 -2 3 -4
%y = const i8 3
%m = map gt %x %y
%s = select %m %x %y
%r = reduce max %s
%l = shl %x 2
%h = shr %x 1
%c = slice %x 1 2
%j = concat %c %c
%n = map neg %x
%k = map not %m
output %r %l %h %j %n %k
";
    let p = parse(every).unwrap();
    assert_eq!(print(&p), every);
}

#[test]
fn types_are_inferred() {
    let p = parse("%x = input i8[4] 1 2 3 4\n%t = const i8 2\n%m = map gt %x %t\n%s = reduce add %x\noutput %m %s\n").unwrap();
    let ts = check(&p).unwrap();
    assert_eq!(ts[2].1, Type::new(Elem::Bool, vec![4]));
    assert_eq!(ts[3].1, Type::new(Elem::Int(8), vec![]));
    assert_eq!(ts[2].1.to_string(), "bool[4]");
}

#[test]
fn every_operation() {
    let src = "%x = input i8[4] 100 -2 3 -128
%z = const i8 0
%two = const i8 2
%add = map add %x %x
%sub = map sub %z %x
%mul = map mul %x %two
%min = map min %x %z
%max = map max %x %z
%neg = map neg %x
%abs = map abs %x
%lt = map lt %x %z
%ge = map ge %x %z
%eq = map eq %x %two
%not = map not %lt
%and = map and %x %two
%or = map or %x %two
%xor = map xor %x %two
%sel = select %lt %z %x
%sum = reduce add %x
%big = reduce max %x
%shl = shl %x 1
%shr = shr %x 1
%sl = slice %x 1 2
%cat = concat %sl %x
output %add %sub %mul %min %max %neg %abs %lt %ge %eq %not %and %or %xor %sel %sum %big %shl %shr %sl %cat
";
    assert_eq!(
        out(src),
        [
            "-56 -4 6 0",
            "-100 2 -3 -128",
            "-56 -4 6 0",
            "0 -2 0 -128",
            "100 0 3 0",
            "-100 2 -3 -128",
            "100 2 3 -128",
            "0 1 0 1",
            "1 0 1 0",
            "0 0 0 0",
            "1 0 1 0",
            "0 2 2 0",
            "102 -2 3 -126",
            "102 -4 1 -126",
            "100 0 3 0",
            "-27",
            "100",
            "-56 -4 6 0",
            "50 -1 1 -64",
            "-2 3",
            "-2 3 100 -2 3 -128",
        ]
    );
}

#[test]
fn matrices_print_aligned() {
    assert_eq!(
        out("%m = input i16[2,3] 0 12 3 46 0 -2\noutput %m\n"),
        [" 0 12  3\n46  0 -2"]
    );
}

#[test]
fn errors_name_the_problem() {
    assert!(err("%a = input i8[2] 1\noutput %a").contains("holds 2 items, 1 given"));
    assert!(err("%a = input i8 200\noutput %a").contains("200 does not fit in i8"));
    assert!(
        err("%a = input i8 1\n%b = input i16 1\n%c = map add %a %b\noutput %c")
            .contains("one element type")
    );
    assert!(
        err("%a = input i8[2] 1 2\n%b = input i8[3] 1 2 3\n%c = map add %a %b\noutput %c")
            .contains("shapes differ")
    );
    assert!(err("%c = map add %a %a\noutput %c").contains("%a is not defined"));
    assert!(err("%a = input i8 1\n%m = map not %a\noutput %m").contains("not does not take i8"));
    assert!(
        err("%a = input i8[2] 1 2\n%s = select %a %a %a\noutput %s").contains("mask must be bool")
    );
    assert!(err("%a = input i8 1\n%r = reduce add %a\noutput %r").contains("non-empty vector"));
    assert!(err("%a = input i8 1\n").contains("outputs nothing"));
    assert!(err("%a = input i8 1\n%a = input i8 1\noutput %a").contains("defined twice"));
    assert!(err("%a = frob i8 1").contains("line 1: unknown operation `frob`"));
    assert!(err("%a = map add %a").contains("add takes 2 operand(s)"));
    assert!(err("%a = input i65 1").contains("needs a type"));
    assert!(err("output %a\n%a = input i8 1")
        .contains("nodes come before next, clocks and the outputs"));
}

const ACC: &str = "%x = stream i8 10 20 30 40 50 -60 70
%s = state i8 0
%t = map add %s %x
next %s %t
output %s
trace %s
";

#[test]
fn a_clocked_program_runs_one_item_per_clock() {
    assert_eq!(out(ACC), ["-96", "0 10 30 60 100 -106 90 -96"]);
    let p = parse(ACC).unwrap();
    assert_eq!(xetal_fpga_xir::clocks(&p), Some(7));
    assert_eq!(parse(&print(&p)).unwrap(), p);
    assert_eq!(print(&p), ACC);
}

#[test]
fn clocks_without_a_stream_and_vector_state() {
    let src = "%s = state i8[3] 0 0 0
%one = const i8 1
%t = map add %s %one
next %s %t
clocks 2
trace %s
";
    assert_eq!(out(src), ["0 0 0\n1 1 1\n2 2 2"]);
    assert_eq!(print(&parse(src).unwrap()), src);
}

#[test]
fn lookup_put_and_cast() {
    let src = "%t = const i8[4] 10 20 30 40
%i = input i8[3] 0 3 9
%r = lookup %t %i
%k = input i8 2
%x = input i16 300
%c = cast i8 %x
%p = put %t %k %c
%q = put %t %x %c
%b = cast bool %i
%back = cast i16 %b
output %r %p %q %b %back
";
    assert_eq!(
        out(src),
        ["10 40 0", "10 20 44 40", "10 20 30 40", "0 1 1", "0 1 1"]
    );
    assert_eq!(print(&parse(src).unwrap()), src);
}

#[test]
fn clocked_errors_name_the_problem() {
    let base = "%x = stream i8 1 2\n%s = state i8 0\n%t = map add %s %x\n";
    assert!(err(&format!("{base}output %s")).contains("needs exactly one `next`, it has 0"));
    assert!(err(&format!("{base}next %s %t\nnext %s %t\noutput %s")).contains("it has 2"));
    assert!(err(&format!("{base}next %s %t\noutput %t")).contains("%t depends on a stream"));
    assert!(err(&format!("{base}next %s %t\nclocks 5\noutput %s"))
        .contains("clocks 5, but the streams run 2 clocks"));
    assert!(err(&format!("{base}next %t %s\noutput %s")).contains("%t is not a state"));
    assert!(
        err("%s = state i8 0\n%t = map add %s %s\nnext %s %t\noutput %s")
            .contains("needs a stream or `clocks N`")
    );
    assert!(err("%a = input i8 1\ntrace %a").contains("only a clocked program"));
    assert!(err("%x = stream i8 1\n%a = input i8 1\noutput %a").contains("a stream needs state"));
    assert!(err("%x = stream i8[2] 1 2 3\n").contains("whole number of items"));
    assert!(
        err("%s = state i8 0\n%w = state i16 0\nnext %s %w\nnext %w %w\nclocks 1\noutput %s")
            .contains("%s is i8 but %w is i16")
    );
    assert!(
        err("%t = const i8[2] 1 2\n%b = input bool 1\n%r = lookup %t %b\noutput %r")
            .contains("lookup's index must be an integer")
    );
    assert!(err("next %s %t\n%s = state i8 0").contains("nodes come before"));
}
