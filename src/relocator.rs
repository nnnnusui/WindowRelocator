use std::thread::sleep;
use std::time::Duration;

use crate::position::Position;
use crate::window::Window;
use std::collections::HashMap;
use std::mem;
use std::sync::mpsc::{Receiver, Sender};
use winapi::{
    shared::{
        minwindef::{BOOL, LPARAM, TRUE},
        windef::HWND,
    },
    um::winuser::{EnumWindows, GetForegroundWindow},
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
    let default_window = get_foreground_window();
    let mut store = Vec::<Window>::new();
    let mut prev_window = default_window.clone();

    loop {
        sleep(Duration::from_millis(100));

        let window = get_foreground_window();
        if window.hwnd != prev_window.hwnd && window.hwnd != default_window.hwnd {
            if !is_target_of_reject(&window) {
                println!("focus window: {:?}", window);
                prev_window = window;
            }
        }
        match receiver.try_recv() {
            Ok(command) => {
                prev_window = match interpret_command(&command, prev_window, &mut store) {
                    Ok(window) => window,
                    Err(window) => window,
                }
            }
            _ => {}
        }
    }
}

pub fn is_target_of_reject(window: &Window) -> bool {
    window.minimized || !window.visible || window.title.is_empty()
}

fn get_foreground_window() -> Window {
    Window::from(unsafe { GetForegroundWindow() })
}

fn interpret_command(
    command: &str,
    target_window: Window,
    store: &mut Vec<Window>,
) -> Result<Window, Window> {
    let args: Vec<&str> = command.split_whitespace().collect();
    let command = args[0];
    match command {
        "show" => show(target_window),
        "show-all" => show_all(target_window),
        "state" => show_state(target_window, store),
        "save" => save(target_window, store),
        _ => Err(target_window),
    }
}

fn show(window: Window) -> Result<Window, Window> {
    println!("target window: {:#?}", window);
    Ok(window)
}
fn show_all(window: Window) -> Result<Window, Window> {
    let windows = enumerate_windows();
    let windows = windows
        .iter()
        .filter(|it| !is_target_of_reject(it))
        .collect::<Vec<_>>();
    for window in &windows {
        println!("{:?}", window)
    }
    println!("count: {}", windows.len());
    Ok(window)
}
fn show_state(window: Window, store: &mut Vec<Window>) -> Result<Window, Window> {
    let indent = " ".repeat(4);
    println!("store state ->");
    for data in store {
        println!("{}{:?}", indent, data)
    }
    println!("<- end store state");
    Ok(window)
}
fn save(window: Window, store: &mut Vec<Window>) -> Result<Window, Window> {
    store.push(window.clone());
    Ok(window)
}
fn load(window: Window, argument: &str, map: &HashMap<String, Position>) -> Result<Window, Window> {
    if !map.contains_key(argument) {
        return Err(window);
    }
    let position = map.get(argument).unwrap();
    window.positioned_to(position.clone())
}

fn enumerate_windows() -> Vec<Window> {
    let mut windows = Vec::<Window>::new();
    let userdata = &mut windows as *mut _;
    unsafe { EnumWindows(Some(enumerate_windows_callback), userdata as LPARAM) };
    windows
}

unsafe extern "system" fn enumerate_windows_callback(hwnd: HWND, userdata: LPARAM) -> BOOL {
    let windows: &mut Vec<Window> = mem::transmute(userdata);
    windows.push(Window::from(hwnd));
    TRUE
}
