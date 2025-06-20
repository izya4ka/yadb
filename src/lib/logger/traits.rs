pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

pub trait Logger: Send + Sync + 'static {
    fn log(&mut self, level: LogLevel, msg: String);
}

pub struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _level: LogLevel, _msg: String) {}
}

impl Default for NullLogger {
    fn default() -> Self {
        NullLogger {}
    }
}
