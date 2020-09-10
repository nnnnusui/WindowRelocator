use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;
use std::{mem, thread};

use std::fs::File;
use std::path::Path;
use winapi::{
    shared::{minwindef::TRUE, windef::HWND},
    um::winuser::{GetForegroundWindow, GetWindowInfo, GetWindowTextW, MoveWindow, WINDOWINFO},
};
use std::collections::HashMap;
use std::sync::mpsc::Sender;

fn main() {
    let csv_path = Path::new("save.csv");
    if !csv_path.exists() {
        File::create(&csv_path);
        let mut writer = csv::Writer::from_path(&csv_path).unwrap();
        writer.write_record(&["Name", "Start_x", "Start_y", "End_x", "End_y"]);
        writer.flush();
    }

    let default_window: HWND = unsafe { GetForegroundWindow() };
    let mut prev_selected: HWND = default_window.clone();

    let mut map = std::collections::HashMap::new();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || input_loop(&sender));

    loop {
        sleep(Duration::from_millis(100));

        let window = unsafe { GetForegroundWindow() };
        if prev_selected != window && window != default_window {
            println!("focus window: {}", get_window_title(&window));
            prev_selected = window;
        }
        match receiver.try_recv() {
            Ok(message) => {
                println!("receive message: {}", message);

                println!("focus window: {}", get_window_title(&prev_selected));
                let (start, end) = get_window_position(&prev_selected);
                println!("window start: ({}, {})", start.0, start.1);
                println!("window end: ({}, {})", end.0, end.1);

                let messages: Vec<&str> = message.split_whitespace().collect();
                let command = messages[0];
                match command {
                    "save" => save(&mut map, &messages[1], (start, end)),
                    "load" => load(&prev_selected, &map, &messages[1]),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn input_loop(sender: &Sender<String>) {
    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();
        println!("send input: {}", input);
        sender
            .send(input.to_string())
            .expect("Send Message Failure");
    }
}

fn save(map: &mut HashMap<String, ((i32, i32), (i32, i32))>, argument: &str, position: ((i32, i32), (i32, i32))) {
    map.insert(argument.to_string(), position);
}
fn load(hwnd: &HWND, map: &HashMap<String, ((i32, i32), (i32, i32))>, argument: &str) {
    if !map.contains_key(argument) { return; }
    let ((x, y), end) = map.get(argument).unwrap();
    let width = end.0 - x;
    let height = end.1 - y;
    move_window(&hwnd, x, y, &width, &height);
}

fn get_window_position(hwnd: &HWND) -> ((i32, i32), (i32, i32)) {
    let mut window_info = unsafe { mem::zeroed::<WINDOWINFO>() };
    // window_info.cbSize = mem::size_of::<WINDOWINFO>();
    let data = &mut window_info as *mut _;
    unsafe { GetWindowInfo(*hwnd, data) };
    (
        (window_info.rcWindow.left, window_info.rcWindow.top),
        (window_info.rcWindow.right, window_info.rcWindow.bottom),
    )
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
