use std::mem;
use std::sync::mpsc;
use std::thread;
use winapi::{
    shared::{
        minwindef::{BOOL, LPARAM, TRUE},
        windef::HWND,
    },
    um::winuser::EnumWindows,
};
extern crate window_relocator;
use window_relocator::relocator::*;

fn main() {
    let windows: Vec<HWND> = enumerate_windows();
    let windows = windows
        .iter()
        .filter(|window| !get_window_position(window).has_imaginary_size())
        .filter(|window| !get_window_title(window).is_empty());
    for window in windows {
        let title = get_window_title(&window);
        let position = get_window_position(&window);
        println!("{:?}: {}", position, title)
    }
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || input_loop(&sender));
    standby_loop(&receiver);
}

fn enumerate_windows() -> Vec<HWND> {
    let mut windows = Vec::<HWND>::new();
    let userdata = &mut windows as *mut _;
    unsafe { EnumWindows(Some(enumerate_windows_callback), userdata as LPARAM) };

    windows
}

unsafe extern "system" fn enumerate_windows_callback(hwnd: HWND, userdata: LPARAM) -> BOOL {
    // Get the userdata where we will store the result
    let windows: &mut Vec<HWND> = mem::transmute(userdata);
    windows.push(hwnd);

    TRUE
}
