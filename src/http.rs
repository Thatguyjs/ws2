use std::{io::{self, Read}, net::{IpAddr, SocketAddr, TcpListener}, rc::Rc, time::Instant};

use httparse::{Request, Status, EMPTY_HEADER};
use polling::{Event, PollMode, Poller};

use crate::{client::{Client, Clients}, config::Config, log, logging::{LogLevel, Logger}};


pub struct HttpError {
    status: u16,
    mime: &'static str,
    message: String,
}

impl HttpError {
    pub fn new<S: ToString>(status: u16, mime: &'static str, message: S) -> Self {
        HttpError { status, mime, message: message.to_string() }
    }
}


pub struct Server {
    listener: TcpListener,
    clients: Clients,
    config: Config,
    poller: Poller,
    logger: Logger,
}

impl Server {
    pub fn bind(address: IpAddr, port: u16, logger: Logger) -> io::Result<Self> {
        Ok(Server {
            listener: TcpListener::bind(SocketAddr::new(address, port))?,
            clients: Clients::new(),
            config: Config::default(),
            poller: Poller::new()?,
            logger
        })
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn listen<F: Fn(httparse::Request, Rc<Config>, Logger) -> Result>(mut self, cb: F) {
        if let Err(e) = self.poller.add_with_mode(&self.listener, Event::readable(0), PollMode::Level) {
            log!(self.logger, LogLevel::Error, "Error adding TcpListener to Poller: {}", e);
        }

        let mut events = vec![];
        let mut prev_time = Instant::now();

        loop {
            events.clear();

            if let Err(e) = self.poller.wait(&mut events, self.clients.lowest_lifetime()) {
                log!(self.logger, LogLevel::Error, "Error waiting for events: {}", e);
            }

            log!(self.logger, LogLevel::Info, "Poll loop, events: {}", events.len());

            // Subtract the elapsed time from all clients
            let now = Instant::now();
            let delta = now.duration_since(prev_time);
            self.clients.sub_duration(delta);
            prev_time = now;

            // No events, client timeout occurred
            if events.len() == 0 {
                let removed = self.clients.remove_inactive();

                for stream in removed {
                    if let Err(e) = self.poller.delete(&stream.stream) {
                        log!(self.logger, LogLevel::Error, "Error Removing Poller Stream: {}", e);
                    }
                }

                continue;
            }

            // Handle all events
            for ev in &events {
                // TcpListener event
                if ev.key == 0 {
                    println!("Listener event");

                    match self.listener.accept() {
                        // Add a client and register it with the poller
                        Ok((stream, peer_addr)) => {
                            let (key, client) = self.clients.add(stream, peer_addr);

                            if let Err(e) = self.poller.add_with_mode(&client.stream, Event::readable(key), PollMode::Level) {
                                log!(self.logger, LogLevel::Error, "Error adding client to poller: {}", e);
                            }

                            println!("Added client ({} total)", self.clients.len());
                        },

                        Err(e) => log!(self.logger, LogLevel::Error, "Error accepting TcpStream: {}", e)
                    }
                }

                // Client event
                else {
                    println!("Client event {} clients", self.clients.len());
                    let mut buf = Box::new([0u8; 2048]);
                    let read = self.clients.get_mut(ev.key).unwrap().read(buf.as_mut_slice());

                    // TODO: Kill client if 0 bytes read (usually indicates closed socket)

                    println!("{read:?} bytes read");
                }
            }
        }
    }
}


pub struct Response {
    status: u16,
    data: Vec<u8>
}

impl Response {
    pub fn with_status(status: u16) -> Self {
        Response {
            status,
            data: vec![]
        }
    }
}


pub type Result = std::result::Result<Response, HttpError>;
