use std::{io, net::{IpAddr, SocketAddr, TcpListener}, rc::Rc, time::Duration};

use polling::{Event, PollMode, Poller};

use crate::{client::Client, config::Config, log, logging::{LogLevel, Logger}};


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
    clients: Vec<Client>,
    config: Config,
    poller: Poller,
    logger: Logger,
}

impl Server {
    pub fn bind(address: IpAddr, port: u16, logger: Logger) -> io::Result<Self> {
        Ok(Server {
            listener: TcpListener::bind(SocketAddr::new(address, port))?,
            clients: vec![],
            config: Config::default(),
            poller: Poller::new()?,
            logger
        })
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn listen<F: Fn(httparse::Request, Rc<Config>, Logger) -> Result>(self, cb: F) {
        if let Err(e) = self.poller.add_with_mode(&self.listener, Event::readable(0), PollMode::Level) {
            log!(self.logger, LogLevel::Error, "Error adding TcpListener to Poller: {}", e);
        }

        let mut events = vec![];
        // TODO: Keep track of client time & subtract every time an event occurs

        loop {
            events.clear();

            if let Err(e) = self.poller.wait(&mut events, self.lowest_lifetime()) {
                log!(self.logger, LogLevel::Error, "Error waiting for events: {}", e);
            }

            for ev in &events {
                // TcpListener event
                if ev.key == 0 {
                    println!("Listener event");

                    let (stream, peer_addr) = self.listener.accept();
                    // TODO: create client or some shit (ALSO error checking pls)
                }

                // Client event
                else {
                    println!("Client event");
                }
            }
        }
    }


    //  -------------
    // Private methods
    //  -------------

    // Get the lowest client lifetime
    fn lowest_lifetime(&self) -> Option<Duration> {
        let mut min = None;

        for client in &self.clients {
            if let Some(m) = min {
                min = Some(client.lifetime.min(m).clone());
            }
            else {
                min = Some(client.lifetime.clone());
            }
        }

        min
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
