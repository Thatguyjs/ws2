// A singular HTTP connection

use std::net::TcpStream;


pub struct Client {
    stream: TcpStream
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }
}
