use std::sync::mpsc;
use std::thread;

extern crate window_relocator;
use window_relocator::relocator::*;

fn main() {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || input_loop(&sender));
    standby_loop(&receiver);
}
