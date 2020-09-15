use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;

use crate::position::Position;
use crate::window::Window;
use regex::Regex;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use winapi::um::winuser::GetForegroundWindow;

pub fn input_loop(sender: &Sender<String>) {
    let terminal_name = "relocator:";
    loop {
        let mut input = String::new();
        println!("{}", terminal_name);
        // std::io::Write::flush(&mut std::io::stdout()).expect("flush failed!");
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
    let mut prev_window = default_window.clone();

    loop {
        sleep(Duration::from_millis(100));

        let window = get_foreground_window();
        if window.hwnd != prev_window.hwnd && window.hwnd != default_window.hwnd {
            println!("focus window: {:?}", window);
            prev_window = window;
        }
        match receiver.try_recv() {
            Ok(command) => {
                match interpret_command(
                    &command.split_whitespace().collect::<Vec<&str>>(),
                    &prev_window,
                ) {
                    Ok(_) => {}
                    Err(_) => print!("error"),
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

fn interpret_command(args: &Vec<&str>, target_window: &Window) -> Result<(), Error> {
    let command = args[0];
    match command {
        "show" => show(&Vec::from([target_window])),
        "show-all" => show(&get_windows().iter().collect()),
        "save" => save(&Vec::from([target_window]), args[1]),
        "save-all" => save(&get_windows().iter().collect(), args[1]),
        "load" => load(args[1], &get_windows()),
        _ => Ok(()),
    }
}

fn show(windows: &Vec<&Window>) -> Result<(), Error> {
    if windows.len() == 1 {
        println!("target window: {:#?}", windows[0]);
        return Ok(());
    }
    for window in windows {
        println!("{:?}", window)
    }
    println!("count: {}", windows.len());
    Ok(())
}
fn save(windows: &Vec<&Window>, file_path: &str) -> Result<(), Error> {
    let file_path = file_path.to_string() + ".csv";
    let file_path = Path::new(&file_path);
    if !file_path.exists() {
        File::create(&file_path)?;
    }
    let mut records = read_csv(file_path)?;
    for window in windows {
        records.push(window.to_record());
    }
    write_csv(file_path, records)?;
    Ok(())
}
fn load(from: &str, to: &Vec<Window>) -> Result<(), Error> {
    let file_path = from.to_string() + ".csv";
    let file_path = Path::new(&file_path);
    if !file_path.exists() {
        panic!("load file not exists");
    }
    let records = read_csv(file_path)?;
    let windows = to;
    for record in records {
        let title_regex = Regex::new(&record.title)?;
        let class_name_regex = Regex::new(&record.class_name)?;
        windows
            .iter()
            .filter(|window| {
                title_regex.is_match(&window.title) && class_name_regex.is_match(&window.class_name)
            })
            .for_each(|window| {
                window.clone().positioned_to(record.get_position());
            });
    }
    Ok(())
}
fn write_csv(file_path: &Path, records: Vec<Record>) -> Result<(), Error> {
    let not_exists = !file_path.exists();
    if not_exists {
        File::create(&file_path)?;
    }
    let mut writer = csv::Writer::from_path(file_path)?;
    if not_exists {
        writer.write_record(&["title", "class_name", "x", "y", "width", "height"])?;
    }
    for record in records {
        writer.serialize(record)?;
    }
    writer.flush()?;
    Ok(())
}
fn read_csv(file_path: &Path) -> Result<Vec<Record>, Error> {
    let mut reader = csv::Reader::from_path(file_path)?;
    let records = reader.deserialize().collect::<Result<_, _>>();
    Ok(records?)
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
trait ToRecordExt {
    fn to_record(&self) -> Record;
}
impl ToRecordExt for Window {
    fn to_record(&self) -> Record {
        Record {
            title: self.title.clone(),
            class_name: self.class_name.clone(),
            x: self.position.x,
            y: self.position.y,
            width: self.position.width,
            height: self.position.height,
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
    Window::enumerate()
        .into_iter()
        .filter(|it| !is_target_of_reject(it))
        .collect::<Vec<_>>()
}
