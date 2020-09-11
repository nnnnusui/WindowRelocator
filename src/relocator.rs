use std::mem;
use std::thread::sleep;
use std::time::Duration;

use crate::position::Position;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
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
    let default_window: HWND = unsafe { GetForegroundWindow() };
    let mut prev_window: HWND = default_window.clone();

    loop {
        sleep(Duration::from_millis(100));

        let window = unsafe { GetForegroundWindow() };
        if window != prev_window && window != default_window {
            println!("focus window: {}", get_window_title(&window));
            prev_window = window;
        }
        match receiver.try_recv() {
            Ok(command) => interpret_command(&command, &mut map, &prev_window),
            _ => {}
        }
    }
}

fn interpret_command(command: &str, map: &mut HashMap<String, Position>, prev_window: &HWND) {
    println!(
        "focus window: {:?}/{}",
        prev_window,
        get_window_title(&prev_window)
    );
    let position = get_window_position(&prev_window);
    println!("window coord: ({}, {})", position.x, position.y);
    println!("window size: ({}, {})", position.width, position.height);

    let args: Vec<&str> = command.split_whitespace().collect();
    let command = args[0];
    match command {
        "save" => save(&args[1], position, map),
        "load" => load(&prev_window, &args[1], &map),
        _ => {}
    }
}

fn save(argument: &str, position: Position, map: &mut HashMap<String, Position>) {
    map.insert(argument.to_string(), position);
}
fn load(hwnd: &HWND, argument: &str, map: &HashMap<String, Position>) {
    if !map.contains_key(argument) {
        return;
    }
    let position = map.get(argument).unwrap();
    move_window(
        &hwnd,
        &position.x,
        &position.y,
        &position.width,
        &position.height,
    );
}

fn get_window_position(hwnd: &HWND) -> Position {
    let mut window_info = unsafe { mem::zeroed::<WINDOWINFO>() };
    // window_info.cbSize = mem::size_of::<WINDOWINFO>();
    let data = &mut window_info as *mut _;
    unsafe { GetWindowInfo(*hwnd, data) };
    let x = window_info.rcWindow.left;
    let y = window_info.rcWindow.top;
    let width = window_info.rcWindow.right - x;
    let height = window_info.rcWindow.bottom - y;
    Position {
        x,
        y,
        width,
        height,
    }
}

fn move_window(hwnd: &HWND, x: &i32, y: &i32, width: &i32, height: &i32) -> bool {
    unsafe { MoveWindow(*hwnd, *x, *y, *width, *height, TRUE) == TRUE }
}

fn get_window_title(hwnd: &HWND) -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    let mut buf = [0u16; 1024];
    let success = unsafe { GetWindowTextW(*hwnd, &mut buf[0], 1024) > 0 };
    if success {
        OsString::from_wide(&buf[..])
            .to_str()
            .map(|it| it.to_string())
            .unwrap_or(String::new())
    } else {
        String::new()
    }
}
