use std::{fs, io::ErrorKind, process::exit, rc::Rc};

mod client;
mod config;
mod http;
mod logging;
mod response;

use config::Config;
use httparse::Request;
use logging::{Logger, LogLevel};
use response::{Builder, Response, Status};


fn main() {
    let logger = Logger::new(LogLevel::Warning);

    let cfg = match config::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            log!(logger, LogLevel::Error, "Config error: {}", e);
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


fn on_request(request: Request, config: Rc<Config>, logger: Logger) -> Response {
    log!(logger, LogLevel::Info, "Client request: {} {}", request.method.unwrap_or(""), request.path.unwrap_or(""));

    match request.method {
        Some("GET") => {
            match request.path {
                Some(path) => {
                    let mut path = config.directory.join(&path[1..]);

                    if path.extension().is_none() {
                        path.push("index.html");
                    }

                    match fs::read(&path) {
                        Ok(data) => {
                            let mut builder = Builder::with_status(Status::Ok);

                            if let Some(mime) = response::mime_from_path(&path) {
                                builder = builder.add_header("Content-Type", mime);
                            }

                            builder.set_body(data)
                                .build()
                        },

                        Err(e) if e.kind() == ErrorKind::NotFound => {
                            Response::text(Status::NotFound, "404 Not Found")
                        },
                        Err(e) => {
                            log!(logger, LogLevel::Error, "Error serving {:?}: {}", path, e);
                            Response::text(Status::InternalServerError, "500 Internal Server Error")
                        }
                    }
                },

                None => Response::text(Status::BadRequest, "400 Bad Request")
            }
        },

        _ => Response::text(Status::MethodNotAllowed, "405 Method Not Allowed")
    }
}
