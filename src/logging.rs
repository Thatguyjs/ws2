use std::{fmt::Arguments, io::Write};

#[macro_export]
macro_rules! log {
    // Static str
    ($logger:expr, $lvl:expr, $msg:literal) => (
        $logger.log($lvl, $msg)
    );

    // Format str
    ($logger:expr, $lvl:expr, $msg:literal, $($arg:expr),+) => (
        $logger.log($lvl, format_args!($msg, $($arg),+))
    );

    // Default to an "info" log level
    ($logger:expr, $msg:literal, $($arg:expr),+) => (
        $logger.log(logging::LogLevel::Info, format_args!($msg, $($arg),+))
    );
}


pub enum LogLevel {
    Debug,
    Warning,
    Error,
    Info
}


pub struct Logger<D: Write> {
    log_level: LogLevel,
    dest: D
}

impl<D: Write> Logger<D> {
    pub fn new(log_level: LogLevel, destination: D) -> Self {
        Logger {
            log_level,
            dest: destination
        }
    }

    pub fn log(&mut self, _level: LogLevel, _args: Arguments) {
        todo!("Log stuff");
    }
}
