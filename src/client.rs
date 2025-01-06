// A singular HTTP connection

use std::{collections::{HashMap, VecDeque}, io::{self, Write}, net::{SocketAddr, TcpStream}, time::Duration};

use crate::response::Response;


#[derive(Debug)]
pub struct Clients {
    clients: HashMap<usize, Client>,
    lifetimes: VecDeque<usize> // Indices into the HashMap, lowest (front) -> highest (end)
}

impl Clients {
    pub fn new() -> Self {
        Clients {
            clients: HashMap::new(),
            lifetimes: VecDeque::new()
        }
    }

    pub fn len(&self) -> usize {
        self.clients.len()
    }

    // Creates & adds a client, returns its associated key
    pub fn add(&mut self, stream: TcpStream, address: SocketAddr) -> (usize, &Client) {
        let key = address.port() as usize;
        let client = Client::new(stream, address);

        self.clients.insert(key, client);
        self.lifetimes.push_back(key);
        (key, &self.clients[&key])
    }

    pub fn get(&self, key: usize) -> Option<&Client> {
        self.clients.get(&key)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut Client> {
        self.clients.get_mut(&key)
    }

    // Subtract a specified duration from all client lifetimes
    pub fn sub_duration(&mut self, d: Duration) {
        self.clients.iter_mut().for_each(|(_, cl)|
            cl.lifetime = cl.lifetime.saturating_sub(d)
        );
    }

    // Returns a vec of "dead" clients (lifetimes that == 0)
    pub fn remove_inactive(&mut self) -> Vec<Client> {
        let mut dead = vec![];
        let mut i = 0;

        while i < self.lifetimes.len() {
            let c = &self.lifetimes[i].clone();

            if self.clients[c].lifetime.is_zero() {
                self.lifetimes.remove(i);
                dead.push(self.clients.remove(&c).unwrap());
            }
            else {
                i += 1;
            }
        }

        dead
    }

    // Remove a specific client
    pub fn remove(&mut self, key: &usize) -> Option<Client> {
        match self.clients.remove(key) {
            Some(client) => {
                self.lifetimes.retain(|k| k != key);
                Some(client)
            },
            None => None
        }
    }

    // Returns the lowest lifetime
    pub fn lowest_lifetime(&self) -> Option<Duration> {
        let key = self.lifetimes.front()?;
        Some(self.clients.get(key)?.lifetime)
    }
}


#[derive(Debug)]
pub struct Client {
    pub stream: TcpStream,
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

    pub fn send(&mut self, response: Response) -> io::Result<()> {
        self.stream.write_all(&response.try_into_bytes()?)
    }
}

impl io::Read for Client {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl io::Write for Client {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}
