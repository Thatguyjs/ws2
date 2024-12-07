use std::{fmt::Arguments, io::{Stdout, Write}, sync::{Arc, Mutex}};


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


#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum LogLevel {
    Debug,
    Warning,
    Error,
    Info
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        format!("[{:?}]", self)
    }
}


#[derive(Clone)]
pub struct Logger {
    log_level: LogLevel,
    dest: Arc<Mutex<Stdout>>
}

impl Logger {
    pub fn new(log_level: LogLevel) -> Self {
        Logger {
            log_level,
            dest: Arc::new(Mutex::new(std::io::stdout()))
        }
    }

    pub fn log(&self, level: LogLevel, args: Arguments) {
        if level < self.log_level {
            return;
        }

        if let Ok(mut dest) = self.dest.lock() {
            let prefix = level.to_string();
            let result = dest.write(prefix.as_bytes())
                .and_then(|_| dest.write(b": "))
                .and_then(|_| dest.write_fmt(args))
                .and_then(|_| dest.flush());

            if let Err(e) = result {
                eprintln!("Error while writing to log: {e}");
            }
        }
    }
}
