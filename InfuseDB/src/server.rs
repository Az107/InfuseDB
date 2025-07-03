use crate::command::Command;
use crate::InfuseDB;
use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener},
    sync::{Arc, Mutex},
    thread,
};

pub struct Server {
    addr: SocketAddr,
    db: Arc<Mutex<InfuseDB>>,
}

impl Server {
    pub fn new(host: &str, port: usize) -> Result<Self, &'static str> {
        let db = InfuseDB::load("default.mdb").unwrap();
        let server = Server {
            addr: format!("{}:{}", host, port)
                .parse()
                .map_err(|_| "Invalid address")?,
            db: Arc::new(Mutex::new(db)), //this could explode 💥, fix later
        };
        Ok(server)
    }

    pub fn listen(&mut self) -> Result<(), &'static str> {
        let listener = TcpListener::bind(self.addr).map_err(|_| "")?;

        for socket in listener.incoming() {
            if socket.is_err() {
                continue;
            }
            let db_clone = self.db.clone();
            thread::spawn(move || {
                let mut socket = socket.unwrap();
                println!("new client");
                let _ = socket.write_all("CaddyDB\n".as_bytes());
                let mut reader = BufReader::new(socket.try_clone().unwrap());
                loop {
                    let mut buff = String::new();
                    let r = reader.read_line(&mut buff);
                    if r.is_err() || r.unwrap() == 0 {
                        println!("client disconnected");
                        let _ = db_clone.lock().unwrap().dump();
                        break;
                    }
                    let r = {
                        let mut db = db_clone.lock().unwrap();
                        db.get_collection("default").unwrap().run(&buff)
                    };

                    let mut buffer = if r.is_ok() {
                        let r = r.unwrap();
                        r.to_string()
                    } else {
                        r.err().unwrap().to_string()
                    };
                    buffer.push_str("\n");
                    let _ = socket.write_all(buffer.as_bytes());
                }
            });
        }
        Ok(())
    }
}
