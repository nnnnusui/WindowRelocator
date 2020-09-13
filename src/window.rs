use crate::position::Position;
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use std::mem;
use winapi::shared::windef::HWND;
use winapi::{
    shared::minwindef::TRUE,
    um::winuser::{GetForegroundWindow, GetWindowInfo, GetWindowTextW, MoveWindow, WINDOWINFO},
};

#[derive(Debug)]
pub struct Window {
    pub hwnd: HWND,
    pub title: String,
    position: Position,
}
impl Window {
    pub fn from(hwnd: HWND) -> Self {
        Self {
            hwnd,
            title: Self::get_window_title(&hwnd),
            position: Self::get_window_position(&hwnd),
        }
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
    fn get_window_title(hwnd: &HWND) -> String {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        let mut buf = [0u16; 1024];
        let success = unsafe { GetWindowTextW(*hwnd, &mut buf[0], 1024) > 0 };
        if success {
            decode_utf16(buf.iter().take_while(|&i| *i != 0).cloned())
                .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
                .collect()
        } else {
            String::new()
        }
    }
}
