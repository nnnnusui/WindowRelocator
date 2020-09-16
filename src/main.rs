use std::sync::mpsc;
use std::thread;
extern crate window_relocator;
use window_relocator::relocator::*;
use window_relocator::window::Window;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(|it| it.as_str()).collect();
    if !args.is_empty() {
        match interpret_command(&args, &Window::get_foreground()) {
            Ok(_) => {}
            Err(error) => eprintln!("Application error: {:?}", error),
        };
        return;
    }
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || input_loop(&sender));
    standby_loop(&receiver);
}
