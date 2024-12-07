// A singular HTTP connection

use std::{net::{SocketAddr, TcpStream}, time::Duration};


pub struct Client {
    stream: TcpStream,
    pub address: SocketAddr,
    pub lifetime: Duration
}

impl Client {
    pub fn new(stream: TcpStream, address: SocketAddr) -> Self {
        Client {
            stream,
            address,
            lifetime: Duration::from_secs(5) // kill client connection after 5 secs inactivity
        }
    }

    pub fn kill(self) {
        self.stream.shutdown(std::net::Shutdown::Both);
        drop(self);
    }
}
