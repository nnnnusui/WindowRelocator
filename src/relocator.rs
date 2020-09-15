use std::thread::sleep;
use std::time::Duration;

use crate::window::Window;
use std::collections::HashMap;
use std::fs::File;
use std::mem;
use std::path::Path;
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
    let mut store = HashMap::<HWND, Window>::new();
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
    store: &mut HashMap<HWND, Window>,
) -> Result<Window, Window> {
    let args: Vec<&str> = command.split_whitespace().collect();
    let command = args[0];
    match command {
        "show" => show(target_window),
        "show-all" => show_all(target_window),
        "state" => show_state(target_window, store),
        "save" => save(target_window, store),
        "save-all" => save_all(target_window, store),
        "save-to" => save_to(target_window, store, args[1]),
        _ => Err(target_window),
    }
}

fn show(window: Window) -> Result<Window, Window> {
    println!("target window: {:#?}", window);
    Ok(window)
}
fn show_all(window: Window) -> Result<Window, Window> {
    let windows = get_windows();
    for window in &windows {
        println!("{:?}", window)
    }
    println!("count: {}", windows.len());
    Ok(window)
}
fn show_state(window: Window, store: &mut HashMap<HWND, Window>) -> Result<Window, Window> {
    let indent = " ".repeat(4);
    println!("store state ->");
    for data in store {
        println!("{}{:?}", indent, data)
    }
    println!("<- end store state");
    Ok(window)
}
fn save(window: Window, store: &mut HashMap<HWND, Window>) -> Result<Window, Window> {
    store.insert(window.hwnd, window.clone());
    Ok(window)
}
fn save_all(window: Window, store: &mut HashMap<HWND, Window>) -> Result<Window, Window> {
    let windows = get_windows();
    for window in windows {
        store.insert(window.hwnd, window);
    }
    Ok(window)
}
fn save_to(
    window: Window,
    store: &mut HashMap<HWND, Window>,
    file_path: &str,
) -> Result<Window, Window> {
    let file_path = file_path.to_string() + ".csv";
    let file_path = Path::new(&file_path);
    if !file_path.exists() {
        File::create(&file_path).expect("Failed to create csv");
    }
    let mut writer = csv::Writer::from_path(&file_path).expect("Failed to create csv writer");
    writer
        .write_record(&["title", "class_name", "x", "y", "width", "height"])
        .expect("Failed to write csv record");
    for window in store.values() {
        writer
            .write_record(&[
                &window.title,
                &window.class_name,
                &window.position.x.to_string(),
                &window.position.y.to_string(),
                &window.position.width.to_string(),
                &window.position.height.to_string(),
            ])
            .expect("Failed to write csv record");
    }
    writer.flush().expect("Failed to save csv");
    Ok(window)
}

fn get_windows() -> Vec<Window> {
    enumerate_windows()
        .into_iter()
        .filter(|it| !is_target_of_reject(it))
        .collect::<Vec<_>>()
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
