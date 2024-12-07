use std::{net::{IpAddr, Ipv6Addr}, str::FromStr};

use crate::logging::LogLevel;


#[derive(Debug)]
pub enum ErrorKind {
    UnknownOption,
    MissingArg,
    BadArg,
    IOError
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String
}

impl Error {
    pub fn new<S: ToString>(kind: ErrorKind, message: S) -> Self {
        Error { kind, message: message.to_string() }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}: {}", self.kind, self.message))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::new(ErrorKind::IOError, value.to_string())
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(value: std::net::AddrParseError) -> Self {
        Self::new(ErrorKind::BadArg, value.to_string())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::new(ErrorKind::BadArg, value.to_string())
    }
}


pub struct Config {
    pub ip: IpAddr,
    pub port: u16,
    pub log_level: LogLevel
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ip: IpAddr::V6(Ipv6Addr::LOCALHOST),
            port: 8080,
            log_level: LogLevel::Warning
        }
    }
}


pub fn load_config() -> Result<Config, Error> {
    let mut args = std::env::args().skip(1);
    let mut cfg = Config::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--addr" | "-a" => {
                let ip = args.next().ok_or(Error::new(ErrorKind::MissingArg, "Missing argument for --addr"))?;
                cfg.ip = IpAddr::from_str(&ip)?;
            },

            "--port" | "-p" => {
                let port = args.next().ok_or(Error::new(ErrorKind::MissingArg, "Missing argument for --port"))?;
                cfg.port = port.parse()?;
            },

            _ => return Err(Error::new(ErrorKind::UnknownOption, format!("Unknown option: \"{}\"", arg)))
        }
    }

    Ok(cfg)
    // todo!()
}
