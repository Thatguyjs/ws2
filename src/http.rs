use std::{io::{self, Read}, net::{IpAddr, SocketAddr, TcpListener}, rc::Rc, time::Instant};

use httparse::{Request, EMPTY_HEADER};
use polling::{Event, PollMode, Poller};

use crate::{client::Clients, config::Config, log, logging::{LogLevel, Logger}, response::{Status, Response, Builder}};


pub struct Server {
    listener: TcpListener,
    clients: Clients,
    config: Rc<Config>,
    poller: Poller,
    logger: Logger,
}

impl Server {
    pub fn bind(address: IpAddr, port: u16, logger: Logger) -> io::Result<Self> {
        Ok(Server {
            listener: TcpListener::bind(SocketAddr::new(address, port))?,
            clients: Clients::new(),
            config: Rc::new(Config::default()),
            poller: Poller::new()?,
            logger
        })
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Rc::new(config);
        self
    }

    pub fn listen<F: Fn(httparse::Request, Rc<Config>, Logger) -> Response>(mut self, cb: F) {
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

            log!(self.logger, LogLevel::Debug, "Processing {} event(s)", events.len());

            // Subtract the elapsed time from all clients
            let now = Instant::now();
            let delta = now.duration_since(prev_time);
            self.clients.sub_duration(delta);
            prev_time = now;

            // No events, client timeout occurred
            if events.len() == 0 {
                let removed = self.clients.remove_inactive();

                for client in &removed {
                    if let Err(e) = self.poller.delete(&client.stream) {
                        log!(self.logger, LogLevel::Error, "Error removing poller stream: {}", e);
                    }
                }

                log!(self.logger, LogLevel::Info, "Timed out {} client(s) (total: {})", removed.len(), self.clients.len());
                continue;
            }

            // Handle all events
            for ev in &events {
                // TcpListener event
                if ev.key == 0 {
                    match self.listener.accept() {
                        // Add a client and register it with the poller
                        Ok((stream, peer_addr)) => {
                            let (key, client) = self.clients.add(stream, peer_addr);

                            match self.poller.add_with_mode(&client.stream, Event::readable(key), PollMode::Level) {
                                Err(e) => log!(self.logger, LogLevel::Error, "Error adding client to poller: {}", e),
                                Ok(_) => log!(self.logger, LogLevel::Info, "New client: {} (total: {})", key, self.clients.len())
                            }
                        },

                        Err(e) => log!(self.logger, LogLevel::Error, "Error accepting TcpStream: {}", e)
                    }
                }

                // Client event
                else if let Some(client) = self.clients.get_mut(ev.key) {
                    let mut buf = Box::new([0u8; 2048]);

                    match client.read(buf.as_mut_slice()) {
                        // No bytes to read, usually indicates closed remote side, so we just
                        // remove the client
                        Ok(n) if n == 0 => {
                            match self.clients.remove(&ev.key) {
                                Some(client) => {
                                    match self.poller.delete(&client.stream) {
                                        Err(e) => log!(self.logger, LogLevel::Error, "Error removing poller stream: {}", e),
                                        Ok(_) => log!(self.logger, LogLevel::Info, "Client {} removed (total: {})", ev.key, self.clients.len())
                                    }
                                },

                                None => log!(self.logger, LogLevel::Warning, "Failed to find client with key: {}", ev.key)
                            }
                        },

                        // Some bytes read, parse the Request and do something with it
                        Ok(n) if n > 0 => {
                            let mut headers = [EMPTY_HEADER; 32];
                            let mut req = Request::new(&mut headers);

                            // TODO: Move this match block into a separate function to make error
                            // handling (failed to send response) easier
                            match req.parse(&buf[0..n]) {
                                Ok(httparse::Status::Complete(_)) => {
                                    let response = cb(req, self.config.clone(), self.logger.clone());
                                    let status = response.status.clone();

                                    match client.send(response) {
                                        Ok(()) => log!(self.logger, LogLevel::Debug, "Sent response: {:?}", status),
                                        Err(e) => log!(self.logger, LogLevel::Error, "Error sending response: {}", e)
                                    }
                                },
                                Ok(httparse::Status::Partial) => {
                                    log!(self.logger, LogLevel::Warning, "Partial request, replying with 400 Bad Request");

                                    let res = Response::text(Status::BadRequest, "400 Bad Request");

                                    if let Err(e) = client.send(res) {
                                        log!(self.logger, LogLevel::Error, "Error sending response: {}", e);
                                    }
                                },
                                Err(e) => {
                                    log!(self.logger, LogLevel::Error, "Bad request: {}", e);

                                    let res = Response::text(Status::BadRequest, "400 Bad Request");

                                    if let Err(e) = client.send(res) {
                                        log!(self.logger, LogLevel::Error, "Error sending response: {}", e);
                                    }
                                }
                            }
                        },

                        Err(e) => log!(self.logger, LogLevel::Error, "Error reading from socket: {}", e),
                        Ok(_) => unreachable!()
                    }
                }

                // Couldn't find Client matching `ev.key`
                else {
                    log!(self.logger, LogLevel::Warning, "Failed to find client with key: {}", ev.key);
                }
            }
        }
    }
}
