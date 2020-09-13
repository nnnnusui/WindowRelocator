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
use window_relocator::window::Window;

fn main() {
    let windows = enumerate_windows();
    for window in windows {
        println!("{:?}", window)
    }
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || input_loop(&sender));
    standby_loop(&receiver);
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
