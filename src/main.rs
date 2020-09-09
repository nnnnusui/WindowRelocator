use std::ffi::OsString;
use winapi::{
    shared::windef::HWND,
    um::winuser::{GetForegroundWindow, GetWindowTextW},
};

fn main() {
    let window = unsafe { GetForegroundWindow() };
    println!("focus window: {}", get_window_title(&window));
}

fn get_window_title(hwnd: &HWND) -> String {
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
