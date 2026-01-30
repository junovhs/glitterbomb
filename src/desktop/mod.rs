//! Desktop implementation using wgpu + winit

mod gpu;
pub mod particle;

use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::thread;

pub use crate::types::{Color, ConfettiOptions, Origin};

static SENDER: Mutex<Option<Sender<gpu::Command>>> = Mutex::new(None);

pub fn fireworks() {
    send(gpu::Command::Fireworks);
}

pub fn celebration() {
    send(gpu::Command::Celebration);
}

pub fn cannon() {}
pub fn snow() {}
pub fn confetti(_: &ConfettiOptions) {}
pub fn confetti_on_canvas(_: &(), _: &ConfettiOptions) {}
pub fn reset() {}

fn send(cmd: gpu::Command) {
    init();
    if let Ok(s) = SENDER.lock() {
        if let Some(tx) = s.as_ref() {
            let _ = tx.send(cmd);
        }
    }
}

fn init() {
    let mut g = SENDER.lock().unwrap();
    if g.is_some() {
        return;
    }
    let (tx, rx) = channel();
    *g = Some(tx);
    drop(g);
    thread::spawn(move || gpu::run_event_loop(rx));
}
