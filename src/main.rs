use std::{process::exit, rc::Rc};
use std::sync::Arc;

mod client;
mod config;
mod http;
mod logging;

use config::Config;
use logging::{Logger, LogLevel};


fn main() {
    let mut logger = Arc::new(Logger::new(LogLevel::Warning, std::io::stdout()));

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


fn on_request<D: std::io::Write>(request: http::Request, config: Rc<Config>, logger: Arc<Logger<D>>) -> http::Result {
    log!(logger, "Request made: \"{}\"", request.url);

    Ok(http::Response::with_status(200).body("Sample page").build())
}
