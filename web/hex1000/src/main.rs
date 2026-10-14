//! HEX1000 in the browser: `libs/Hex1000/demos/terminal.xtl`, the
//! terminal calculator, run a slice at a time by X_eTaL's engine
//! (`xetal-play`). The page puts the libraries it imports (and the ROM
//! program Hex1000 includes) in the engine's store, feeds each key the
//! program waits for, and shows its output from the last clear-screen.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::Write;
use std::rc::Rc;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use xetal_play::{Interactive, Step};

const PROGRAM: &str = include_str!("../../../libs/Hex1000/demos/terminal.xtl");
const FILES: [(&str, &str); 4] = [
    ("Hex1000.xtl", include_str!("../../../libs/Hex1000/src/Hex1000.xtl")),
    ("Radix.xtl", include_str!("../../../libs/Radix/src/Radix.xtl")),
    ("Strings.xtl", include_str!("../../../work/libs/Strings/src/Strings.xtl")),
    // Hex1000.xtl includes "../rom/calc.asm"; stored flat, it reads this key.
    ("../rom/calc.asm", include_str!("../../../libs/Hex1000/rom/calc.asm")),
];
/// The keys on the page: label, and the key name fed to []K_EY.
const PAD: [(&str, &str); 24] = [
    ("C", "C"), ("D", "D"), ("E", "E"), ("F", "F"), ("X", "X"), ("q", "q"),
    ("8", "8"), ("9", "9"), ("A", "A"), ("B", "B"), ("*", "*"), ("=", "="),
    ("4", "4"), ("5", "5"), ("6", "6"), ("7", "7"), ("-", "-"), (" ", ""),
    ("0", "0"), ("1", "1"), ("2", "2"), ("3", "3"), ("+", "+"), (" ", ""),
];
/// Engine transitions per slice, and the slices' time per frame.
const SLICE: usize = 20_000;
const FRAME_MS: f64 = 12.0;

/// The program's output, kept for the screen.
#[derive(Clone, Default)]
struct Out(Rc<RefCell<String>>);

impl Write for Out {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().push_str(&String::from_utf8_lossy(bytes));
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct App {
    run: Option<Interactive>,
    out: Out,
    keys: VecDeque<String>,
    waiting: bool,
    done: bool,
}

fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

fn set_text(id: &str, text: &str) {
    if let Some(el) = document().get_element_by_id(id) {
        el.set_text_content(Some(text));
    }
}

/// The screen: the output after the last clear, other escapes dropped.
fn screen(out: &str) -> String {
    let last = out.rfind("\x1b[2J\x1b[H").map_or(0, |i| i + 7);
    out[last..].replace("\x1b[2J", "").replace("\x1b[H", "")
}

impl App {
    /// Run slices until the program waits for a key or the frame's time is up.
    fn tick(&mut self) {
        let Some(run) = self.run.as_mut() else { return };
        let start = js_sys::Date::now();
        while js_sys::Date::now() - start < FRAME_MS {
            if self.waiting {
                match self.keys.pop_front() {
                    Some(k) => {
                        run.feed(k);
                        self.waiting = false;
                    }
                    None => break,
                }
            }
            match run.step(SLICE) {
                Step::Running => {}
                Step::WaitingKey | Step::Waiting | Step::WaitingEvent => {
                    self.waiting = true;
                }
                Step::Done(r) => {
                    self.done = true;
                    let why = r.err;
                    set_text("status", &format!("The program has ended (q). Reload the page to start again. {why}"));
                    self.run = None;
                    break;
                }
            }
        }
        set_text("screen", &screen(&self.out.0.borrow()));
        if !self.done {
            let busy = !self.waiting || !self.keys.is_empty();
            set_text("status", if busy { "computing ..." } else { "ready for a key" });
        }
    }
}

fn main() {
    xetal_store::install(Arc::new(xetal_store::Memory::default()));
    for (name, text) in FILES {
        xetal_store::write(name, text).expect("the store takes the library");
    }
    let out = Out::default();
    let app = Rc::new(RefCell::new(App {
        run: None,
        out: out.clone(),
        keys: VecDeque::new(),
        waiting: false,
        done: false,
    }));
    match Interactive::start(PROGRAM, 1, Box::new(out)) {
        Ok(run) => app.borrow_mut().run = Some(run),
        Err(r) => set_text("screen", &format!("the program did not start: {}", r.err)),
    }

    // Keys typed on the keyboard.
    let a = app.clone();
    let on_key = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
        if e.ctrl_key() || e.meta_key() || e.alt_key() {
            return;
        }
        let k = e.key();
        if k.chars().count() == 1 || k == "Enter" || k == "Escape" {
            e.prevent_default();
            a.borrow_mut().keys.push_back(k);
        }
    });
    document().add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref()).unwrap();
    on_key.forget();

    // The keys on the page.
    let pad = document().get_element_by_id("keys").unwrap();
    for (label, name) in PAD {
        let b = document().create_element("button").unwrap();
        b.set_text_content(Some(label));
        if name.is_empty() {
            b.set_attribute("disabled", "").unwrap();
            b.set_attribute("aria-hidden", "true").unwrap();
        } else {
            b.set_attribute("aria-label", if name == "q" { "quit" } else { name }).unwrap();
            let a = app.clone();
            let name = name.to_string();
            let click = Closure::<dyn FnMut()>::new(move || a.borrow_mut().keys.push_back(name.clone()));
            b.add_event_listener_with_callback("click", click.as_ref().unchecked_ref()).unwrap();
            click.forget();
        }
        pad.append_child(&b).unwrap();
    }

    // The engine, a slice per frame.
    let a = app.clone();
    let frame = Closure::<dyn FnMut()>::new(move || a.borrow_mut().tick());
    web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(frame.as_ref().unchecked_ref(), 16)
        .unwrap();
    frame.forget();
}
