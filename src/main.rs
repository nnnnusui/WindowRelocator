use std::mem;
use std::thread::sleep;
use std::time::Duration;
use winapi::{
    shared::{
        minwindef::{BOOL, LPARAM, TRUE},
        windef::HWND,
    },
    um::winuser::{GetForegroundWindow, GetWindowInfo, GetWindowTextW, WINDOWINFO},
};

fn main() {
    let mut prev_selected: HWND = unsafe { GetForegroundWindow() };
    loop {
        sleep(Duration::from_millis(100));

        let window = unsafe { GetForegroundWindow() };
        if prev_selected != window {
            println!("focus window: {}", get_window_title(&window));
            prev_selected = window;

            let (start, end) = get_window_position(&window);
            println!("window start: ({}, {})", start.0, start.1);
            println!("window end: ({}, {})", end.0, end.1);
        }
    }
}
fn get_window_position(hwnd: &HWND) -> ((i32, i32), (i32, i32)) {
    let mut window_info = unsafe { mem::zeroed::<WINDOWINFO>() };
    // window_info.cbSize = mem::size_of::<WINDOWINFO>();
    let data = &mut window_info as *mut _;
    let result = unsafe { GetWindowInfo(*hwnd, data) };
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
