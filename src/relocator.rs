use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;

use crate::position::Position;
use crate::window::Window;
use regex::Regex;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::fs::File;
use std::mem;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use thiserror::private::PathAsDisplay;
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
            // if !is_target_of_reject(&window) {
            println!("focus window: {:?}", window);
            prev_window = window;
            // }
        }
        match receiver.try_recv() {
            Ok(command) => {
                prev_window = match interpret_command(&command, prev_window, &mut store) {
                    Ok(window) => window,
                    Err(error) => panic!(error),
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
) -> Result<Window, Error> {
    let args: Vec<&str> = command.split_whitespace().collect();
    let command = args[0];
    match command {
        "show" => show(target_window),
        "show-all" => show_all(target_window),
        "state" => show_state(target_window, store),
        "save" => save(target_window, store),
        "save-all" => save_all(target_window, store),
        "save-to" => save_to(target_window, store, args[1]),
        "load" => load(target_window, store, args[1]),
        _ => Ok(target_window),
    }
}

fn show(window: Window) -> Result<Window, Error> {
    println!("target window: {:#?}", window);
    Ok(window)
}
fn show_all(window: Window) -> Result<Window, Error> {
    let windows = get_windows();
    for window in &windows {
        println!("{:?}", window)
    }
    println!("count: {}", windows.len());
    Ok(window)
}
fn show_state(window: Window, store: &mut HashMap<HWND, Window>) -> Result<Window, Error> {
    let indent = " ".repeat(4);
    println!("store state ->");
    for data in store {
        println!("{}{:?}", indent, data)
    }
    println!("<- end store state");
    Ok(window)
}
fn save(window: Window, store: &mut HashMap<HWND, Window>) -> Result<Window, Error> {
    store.insert(window.hwnd, window.clone());
    Ok(window)
}
fn save_all(window: Window, store: &mut HashMap<HWND, Window>) -> Result<Window, Error> {
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
) -> Result<Window, Error> {
    let file_path = file_path.to_string() + ".csv";
    let file_path = Path::new(&file_path);
    if !file_path.exists() {
        File::create(&file_path)?;
    }
    let mut writer = csv::Writer::from_path(&file_path)?;
    writer.write_record(&["title", "class_name", "x", "y", "width", "height"])?;
    for window in store.values() {
        writer.serialize((
            &window.title,
            &window.class_name,
            window.position.x,
            window.position.y,
            window.position.width,
            window.position.height,
        ));
    }
    writer.flush()?;
    Ok(window)
}
fn load(
    window: Window,
    store: &mut HashMap<HWND, Window>,
    file_path: &str,
) -> Result<Window, Error> {
    let file_path = file_path.to_string() + ".csv";
    let file_path = Path::new(&file_path);
    if !file_path.exists() {
        panic!(format!("{} Not Found", file_path.as_display()))
    }
    let mut reader = csv::Reader::from_path(file_path)?;
    for result in reader.deserialize() {
        let record: Record = result?;
        let title_regex = Regex::new(&record.title)?;
        let class_name_regex = Regex::new(&record.class_name)?;
        let windows = get_windows();
        windows
            .iter()
            .filter(|window| {
                title_regex.is_match(&window.title) && class_name_regex.is_match(&window.class_name)
            })
            .map(
                |window| match window.clone().positioned_to(record.get_position()) {
                    Ok(window) => window,
                    Err(window) => window,
                },
            )
            .for_each(|window| {
                store.insert(window.hwnd, window);
            });
    }
    Ok(window)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Record {
    title: String,
    class_name: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}
impl Record {
    fn get_position(&self) -> Position {
        Position {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }
}

#[derive(Error, Debug)]
enum Error {
    #[error("csv error {0}")]
    CsvError(#[from] csv::Error),
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
    #[error("regex error {0}")]
    RegexError(#[from] regex::Error),
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
