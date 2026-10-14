//! The page's engine natively, for comparison with the browser: the
//! same program, libraries and keys through xetal-play's Interactive,
//! reporting per key the slices, transitions and time.
//!   cargo run --release --bin bench -- 12+34=

use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

use xetal_play::{Interactive, Step};

const PROGRAM: &str = include_str!("../../../../libs/Hex1000/demos/terminal.xtl");
const FILES: [(&str, &str); 4] = [
    ("Hex1000.xtl", include_str!("../../../../libs/Hex1000/src/Hex1000.xtl")),
    ("Radix.xtl", include_str!("../../../../libs/Radix/src/Radix.xtl")),
    ("Strings.xtl", include_str!("../../../../work/libs/Strings/src/Strings.xtl")),
    ("../rom/calc.asm", include_str!("../../../../libs/Hex1000/rom/calc.asm")),
];
const SLICE: usize = 20_000;

struct Sink;
impl Write for Sink {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn main() {
    let keys: Vec<String> = std::env::args().nth(1).unwrap_or_default().chars().map(String::from).collect();
    xetal_store::install(Arc::new(xetal_store::Memory::default()));
    for (n, t) in FILES {
        xetal_store::write(n, t).unwrap();
    }
    let t0 = Instant::now();
    let mut run = Interactive::start(PROGRAM, 1, Box::new(Sink)).unwrap_or_else(|r| panic!("{}", r.err));
    println!("start (load, check): {:.0} ms", t0.elapsed().as_secs_f64() * 1e3);
    let mut queue = keys.into_iter();
    let mut label = "reset".to_string();
    loop {
        let t = Instant::now();
        let mut slices = 0;
        let done = loop {
            slices += 1;
            match run.step(SLICE) {
                Step::Running => continue,
                Step::Done(_) => break true,
                _ => break false,
            }
        };
        let ms = t.elapsed().as_secs_f64() * 1e3;
        println!("key {label:>6}: {slices:>5} slices (about {:>9} transitions), {ms:>8.1} ms, {:>6.0} transitions/ms", slices * SLICE, (slices * SLICE) as f64 / ms);
        if done {
            break;
        }
        match queue.next() {
            Some(k) => {
                label = k.clone();
                run.feed(k);
            }
            None => {
                label = "Escape".into();
                run.feed("Escape".into());
            }
        }
    }
}
