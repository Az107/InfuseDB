use crate::command::Command;
use crate::InfuseDB;
use crate::VERSION;

use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Registry, Token};
use std::collections::HashMap;

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
};

pub struct Server {
    addr: SocketAddr,
    db: InfuseDB,
}

fn process_request(db: &Arc<Mutex<InfuseDB>>, cmd: &str) -> String {
    let r = {
        let mut db = db.lock().unwrap();
        db.get_collection("default").unwrap().run(cmd)
    };

    if r.is_ok() {
        let r = r.unwrap();
        r.to_string()
    } else {
        r.err().unwrap().to_string()
    }
}
const SERVER: Token = Token(0);

impl Server {
    pub fn new(host: &str, port: usize) -> Result<Self, &'static str> {
        let db = InfuseDB::load("default.mdb").unwrap();
        let server = Server {
            addr: format!("{}:{}", host, port)
                .parse()
                .map_err(|_| "Invalid address")?,
            db,
        };
        Ok(server)
    }

    pub fn listen(&mut self) -> std::io::Result<()> {
        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(128);
        let mut listener = TcpListener::bind(self.addr)?;
        let mut connections: HashMap<Token, TcpStream> = HashMap::new();
        let mut unique_token = 1;
        poll.registry()
            .register(&mut listener, SERVER, Interest::READABLE)?;

        loop {
            poll.poll(&mut events, None)?;
            for event in events.iter() {
                match event.token() {
                    SERVER => {
                        // Nueva conexión entrante
                        let (mut stream, _) = listener.accept()?;
                        let token = Token(unique_token);
                        unique_token += 1;
                        let header = format!("InfuseDB {}\r\n", VERSION);
                        stream.write_all(header.as_bytes()).unwrap(); // Respuesta simple
                        poll.registry()
                            .register(&mut stream, token, Interest::READABLE)?;
                        connections.insert(token, stream);
                    }
                    token => {
                        // Socket de cliente listo
                        let socket = connections.get_mut(&token).unwrap();
                        let mut buf = [0u8; 1024];
                        match socket.read(&mut buf) {
                            Ok(0) => {
                                // desconectado
                                connections.remove(&token);
                            }
                            Ok(n) => {
                                // procesar datos
                                let data = &buf[..n];
                                let cmd = str::from_utf8(&data).unwrap();
                                let result = self.db.get_collection("default").unwrap().run(cmd);

                                let result = if result.is_ok() {
                                    let r = result.unwrap();
                                    r.to_string()
                                } else {
                                    result.err().unwrap().to_string()
                                };

                                socket.write_all(result.as_bytes()).unwrap();
                                socket.write_all(b"\r\n").unwrap(); // Respuesta simple
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                // no hay nada realmente
                            }
                            Err(e) => {
                                eprintln!("Error en cliente: {}", e);
                                connections.remove(&token);
                            }
                        }
                    }
                }
            }
        }
        //Ok(())
    }
}
