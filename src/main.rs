use std::{process::exit, rc::Rc};

mod client;
mod config;
mod http;
mod logging;

use config::Config;
use httparse::Request;
use logging::{Logger, LogLevel};


fn main() {
    let logger = Logger::new(LogLevel::Warning);

    let cfg = match config::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            log!(logger, LogLevel::Error, "Configuration error: {}", e);
            exit(1);
        }
    };

    let server = http::Server::bind(cfg.ip, cfg.port, logger.clone());

    match server {
        Ok(server) => server
            .with_config(cfg)
            .listen(on_request),
        Err(e) => log!(logger, LogLevel::Error, "Server error: {}", e)
    }
}


fn on_request(request: Request, config: Rc<Config>, logger: Logger) -> http::Result {
    log!(logger, "Request made: \"{:?}\"", request.path);

    Ok(http::Response::with_status(200))
}
