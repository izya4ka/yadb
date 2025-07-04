use std::sync::Mutex;

use crate::lib::logger::file_logger::FileLogger;

pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

pub enum BusterLogger {
    NullLogger(NullLogger),
    FileLogger(Mutex<FileLogger>),
}

pub trait Logger: Send + Sync + 'static {
    fn log(&self, level: LogLevel, msg: String);
}
#[derive(Default)]
pub struct NullLogger {}

impl Logger for NullLogger {
    fn log(&self, _level: LogLevel, _msg: String) {}
}

impl BusterLogger {
    pub fn log(&self, level: LogLevel, msg: String) {
        match self {
            BusterLogger::NullLogger(logger) => logger.log(level, msg),
            BusterLogger::FileLogger(logger) => logger.lock().unwrap().log(level, msg),
        }
    }
}
