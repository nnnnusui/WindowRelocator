use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;
use std::{mem, thread};

use std::fs::File;
use std::path::Path;
use winapi::{
    shared::windef::HWND,
    um::winuser::{GetForegroundWindow, GetWindowInfo, GetWindowTextW, WINDOWINFO},
};

fn main() {
    let csv_path = Path::new("save.csv");
    if !csv_path.exists() {
        File::create(&csv_path);
    }

    let mut prev_selected: HWND = unsafe { GetForegroundWindow() };
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();
        println!("send input: {}", input);
        sender
            .send(input.to_string())
            .expect("Send Message Failure");
    });
    loop {
        sleep(Duration::from_millis(100));

        let window = unsafe { GetForegroundWindow() };
        if prev_selected != window {
            println!("focus window: {}", get_window_title(&window));
            prev_selected = window;
        }
        match receiver.try_recv() {
            Ok(message) => {
                println!("receive message: {}", message);

                println!("focus window: {}", get_window_title(&window));
                let (start, end) = get_window_position(&window);
                println!("window start: ({}, {})", start.0, start.1);
                println!("window end: ({}, {})", end.0, end.1);

                let messages: Vec<&str> = message.split_whitespace().collect();
                let command = messages[0];
                match command {
                    "save" => {
                        let argument = messages[1];
                        let mut writer = csv::Writer::from_path(&csv_path).unwrap();
                        writer.write_record(&[
                            &argument,
                            start.0.to_string().as_str(),
                            start.1.to_string().as_str(),
                            end.0.to_string().as_str(),
                            end.1.to_string().as_str(),
                        ]);
                        writer.flush();
                    }
                    "load" => {
                        let argument = messages[1];
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
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
