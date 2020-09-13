use std::mem;
use std::thread::sleep;
use std::time::Duration;

use crate::position::Position;
use crate::window::Window;
use std::char::REPLACEMENT_CHARACTER;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use winapi::_core::char::decode_utf16;
use winapi::{
    shared::{minwindef::TRUE, windef::HWND},
    um::winuser::{GetForegroundWindow, GetWindowInfo, GetWindowTextW, MoveWindow, WINDOWINFO},
};

pub fn input_loop(sender: &Sender<String>) {
    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        sender
            .send(input.trim().to_string())
            .expect("Send Message Failure");
    }
}

pub fn standby_loop(receiver: &Receiver<String>) {
    let mut map = std::collections::HashMap::new();
    let default_window = get_foreground_window();
    let mut prev_window = default_window.clone();

    loop {
        sleep(Duration::from_millis(100));

        let window = get_foreground_window();
        if window.hwnd != prev_window.hwnd && window.hwnd != default_window.hwnd {
            if !window.position.has_imaginary_size() {
                println!("focus window: {:?}", window);
                prev_window = window;
            }
        }
        match receiver.try_recv() {
            Ok(command) => {
                prev_window = match interpret_command(&command, &mut map, prev_window) {
                    Ok(window) => window,
                    Err(window) => window,
                }
            }
            _ => {}
        }
    }
}

fn get_foreground_window() -> Window {
    Window::from(unsafe { GetForegroundWindow() })
}

fn interpret_command(
    command: &str,
    map: &mut HashMap<String, Position>,
    target_window: Window,
) -> Result<Window, Window> {
    println!("target window: {:#?}", target_window);
    let args: Vec<&str> = command.split_whitespace().collect();
    let command = args[0];
    match command {
        "save" => save(target_window, &args[1], map),
        "load" => load(target_window, &args[1], &map),
        _ => Err(target_window),
    }
}

fn save(
    window: Window,
    argument: &str,
    map: &mut HashMap<String, Position>,
) -> Result<Window, Window> {
    map.insert(argument.to_string(), window.position.clone());
    Ok(window)
}
fn load(window: Window, argument: &str, map: &HashMap<String, Position>) -> Result<Window, Window> {
    if !map.contains_key(argument) {
        return Err(window);
    }
    let position = map.get(argument).unwrap();
    window.positioned_to(position.clone())
}
