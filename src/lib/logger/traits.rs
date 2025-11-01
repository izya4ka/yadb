use std::sync::Mutex;

use crate::lib::logger::file_logger::FileLogger;

pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

#[derive(Debug)]
pub enum WorkerLogger {
    NullLogger(NullLogger),
    FileLogger(Mutex<FileLogger>),
}

pub trait Logger: Send + Sync + 'static {
    fn log(&self, level: LogLevel, msg: String);
}
#[derive(Default, Debug)]
pub struct NullLogger {}

impl Logger for NullLogger {
    fn log(&self, _level: LogLevel, _msg: String) {}
}

impl WorkerLogger {
    pub fn log(&self, level: LogLevel, msg: String) {
        match self {
            WorkerLogger::NullLogger(logger) => logger.log(level, msg),
            WorkerLogger::FileLogger(logger) => logger.lock().unwrap().log(level, msg),
        }
    }
}
