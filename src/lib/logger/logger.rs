use super::traits::LogLevel;
use anyhow::Result;
use chrono::Local;
use std::{fs::File, io::Write};

use crate::lib::logger::traits::Logger;

#[derive(Default)]
pub struct FileLogger {
    file: Option<File>,
}

impl FileLogger {
    pub fn new(path: String) -> Result<Self> {
        let file = File::create(path)?;
        Ok(FileLogger { file: Some(file) })
    }
}

impl Logger for FileLogger {
    fn log(&mut self, level: LogLevel, msg: String) {
        if let Some(mut file) = self.file.as_ref() {
            let mut str = String::default();

            str += &Local::now().format("[%H:%M:%S] ").to_string();

            str += match level {
                LogLevel::INFO => "[INFO] ",
                LogLevel::WARN => "[WARN] ",
                LogLevel::ERROR => "[ERROR] ",
                LogLevel::CRITICAL => "[CRITICAL] ",
            };

            str += &msg;
            str += "\n";

            let _ = file.write(str.as_bytes());
        }
    }
}
