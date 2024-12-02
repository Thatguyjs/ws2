use std::{io::{self, Write}, net::{IpAddr, SocketAddr, TcpListener}, rc::Rc, sync::Arc};

use polling::{Event, PollMode, Poller};

use crate::{client::Client, config::Config, log, logging::{LogLevel, Logger}};


pub struct Server<D: Write> {
    listener: TcpListener,
    clients: Vec<Client>,
    config: Config,
    poller: Poller,
    logger: Arc<Logger<D>>,
}

impl<D: Write> Server<D> {
    pub fn bind(address: IpAddr, port: u16, logger: Arc<Logger<D>>) -> io::Result<Self> {
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

    pub fn listen<F: Fn(Request, Rc<Config>, Arc<Logger<D>>) -> Result>(self, cb: F) {
        if let Err(e) = self.poller.add_with_mode(&self.listener, Event::readable(0), PollMode::Level) {
            log!(self.logger, LogLevel::Error, "Error adding TcpListener to Poller: {}", e);
        }

        loop {

        }
    }
}
