use crate::command::Command;
use crate::infusedb::DataType;
use crate::InfuseDB;
use crate::VERSION;

use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Registry, Token};
use std::array;
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

pub struct Context {
    socket: TcpStream,
    collection: Option<String>,
}

fn process_request(db: &Arc<Mutex<InfuseDB>>, collection: String, cmd: &str) -> String {
    let r = {
        let mut db = db.lock().unwrap();
        db.get_collection(&collection).unwrap().run(cmd)
    };

    if r.is_ok() {
        let r = r.unwrap();
        r.to_string()
    } else {
        r.err().unwrap().to_string()
    }
}

enum ProcessError {
    InvalidCommand,
    NotFound,
    Other(&'static str),
}

fn process_cmd(cmd: &str, ctx: &mut Context, db: &InfuseDB) -> Result<DataType, ProcessError> {
    let args: Vec<&str> = cmd.split_whitespace().collect();
    match *args.first().ok_or(ProcessError::InvalidCommand)? {
        "echo" => {
            let args2 = args.iter().skip(1).map(|s| *s).collect::<Vec<_>>();
            let result = args2.join(" ");
            Ok(DataType::Text(result))
        }
        "select" => {
            let col_name = args.get(1);
            if col_name.is_none() {
                return Err(ProcessError::Other("No collection name provided"));
            }
            let col_name = col_name.unwrap();
            if !db.get_collection_list().contains(&col_name.to_string()) {
                return Err(ProcessError::Other("Collection does not exist"));
            }
            ctx.collection = Some(col_name.to_string());
            Ok(DataType::Boolean(true))
        }
        "unselect" => {
            let pre_exists = ctx.collection.is_some();
            ctx.collection = None;
            Ok(DataType::Boolean(pre_exists))
        }
        "list" => {
            let list = db
                .get_collection_list()
                .iter()
                .map(|item| DataType::Text(item.to_owned()))
                .collect();
            Ok(DataType::Array(list))
        }
        _ => Err(ProcessError::NotFound),
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
        let mut connections: HashMap<Token, Context> = HashMap::new();
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
                        connections.insert(
                            token,
                            Context {
                                socket: stream,
                                collection: None,
                            },
                        );
                    }
                    token => {
                        // Socket de cliente listo
                        let ctx = connections.get_mut(&token).unwrap();

                        let mut buf = [0u8; 1024];
                        match ctx.socket.read(&mut buf) {
                            Ok(0) => {
                                // desconectado
                                connections.remove(&token);
                            }
                            Ok(n) => {
                                // procesar datos
                                let data = &buf[..n];
                                let cmd = str::from_utf8(&data).unwrap();
                                let cmd_result: Result<DataType, ProcessError> =
                                    process_cmd(cmd, ctx, &self.db);
                                let result: Result<DataType, &str> = match cmd_result {
                                    Ok(result) => Ok(result),
                                    Err(err) => match err {
                                        ProcessError::InvalidCommand => Err("Invalid Command"),
                                        ProcessError::NotFound => {
                                            if let Some(collection) = ctx.collection.clone() {
                                                self.db
                                                    .get_collection(&collection)
                                                    .unwrap()
                                                    .run(cmd)
                                            } else {
                                                Err("No collection selected")
                                            }
                                        }
                                        ProcessError::Other(text) => Err(text),
                                    },
                                };

                                let result = if result.is_ok() {
                                    let r = result.unwrap();
                                    r.to_string()
                                } else {
                                    format!("err: {}", result.err().unwrap().to_string())
                                };

                                ctx.socket.write_all(result.as_bytes()).unwrap();
                                ctx.socket.write_all(b"\r\n").unwrap(); // Respuesta simple
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
