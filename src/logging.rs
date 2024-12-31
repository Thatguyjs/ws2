use std::{fmt::Display, io::{Stdout, Write}, sync::{Arc, Mutex}};


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

impl LogLevel {
    fn color(&self) -> &'static str {
        match self {
            LogLevel::Debug => "\x1b[90;3m",
            LogLevel::Warning => "\x1b[93;1m",
            LogLevel::Error => "\x1b[91;1m",
            LogLevel::Info => "\x1b[36m"
        }
    }
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        format!("{}[{:?}]\x1b[0m", self.color(), self)
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

    pub fn log<D: Display>(&self, level: LogLevel, args: D) {
        if level < self.log_level {
            //return;
        }

        if let Ok(mut dest) = self.dest.lock() {
            let prefix = level.to_string();
            let result = dest.write(prefix.as_bytes())
                .and_then(|_| dest.write(b": "))
                .and_then(|_| dest.write(args.to_string().as_bytes()))
                .and_then(|_| dest.write(b"\n"));

            if let Err(e) = result {
                eprintln!("Error while writing to log: {e}");
            }
        }
    }
}
