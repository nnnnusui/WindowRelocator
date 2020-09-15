use crate::position::Position;
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use std::mem;
use winapi::shared::windef::HWND;
use winapi::um::winuser::{GetClassNameW, IsIconic, IsWindowEnabled, IsWindowVisible};
use winapi::{
    shared::minwindef::TRUE,
    um::winuser::{GetForegroundWindow, GetWindowInfo, GetWindowTextW, MoveWindow, WINDOWINFO},
};

#[derive(Clone, Debug)]
pub struct Window {
    pub hwnd: HWND,
    pub title: String,
    pub position: Position,
    pub class_name: String,
    pub visible: bool,
    pub minimized: bool,
    pub can_input: bool,
}
impl Window {
    pub fn from(hwnd: HWND) -> Self {
        Self {
            hwnd,
            title: Self::get_window_title(&hwnd),
            position: Self::get_window_position(&hwnd),
            class_name: Self::get_class_name(&hwnd),
            visible: Self::is_window_visible(&hwnd),
            minimized: Self::is_iconic(&hwnd),
            can_input: Self::is_window_enabled(&hwnd),
        }
    }

    pub fn positioned_to(self, position: Position) -> Result<Self, Self> {
        let success = Self::set_window_position(&self.hwnd, &position);
        if !success {
            return Err(self);
        }
        Ok(Window { position, ..self })
    }

    fn is_iconic(hwnd: &HWND) -> bool {
        unsafe { IsIconic(*hwnd) == TRUE }
    }
    fn is_window_visible(hwnd: &HWND) -> bool {
        unsafe { IsWindowVisible(*hwnd) == TRUE }
    }
    fn is_window_enabled(hwnd: &HWND) -> bool {
        unsafe { IsWindowEnabled(*hwnd) == TRUE }
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
        let mut buf = [0u16; 1024];
        let success = unsafe { GetWindowTextW(*hwnd, &mut buf[0], 1024) > 0 };
        if success {
            Self::decode(&buf)
        } else {
            String::new()
        }
    }
    fn get_class_name(hwnd: &HWND) -> String {
        let mut buf = [0u16; 1024];
        let success = unsafe { GetClassNameW(*hwnd, &mut buf[0], 1024) > 0 };
        if success {
            Self::decode(&buf)
        } else {
            String::new()
        }
    }
    fn decode(source: &[u16]) -> String {
        decode_utf16(source.iter().take_while(|&i| *i != 0).cloned())
            .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
            .collect()
    }
    fn set_window_position(hwnd: &HWND, position: &Position) -> bool {
        unsafe {
            MoveWindow(
                *hwnd,
                position.x,
                position.y,
                position.width,
                position.height,
                TRUE,
            ) == TRUE
        }
    }
}
