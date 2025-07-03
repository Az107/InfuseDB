use std::{
    io::Read,
    net::{SocketAddr, TcpStream},
};

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(host: &str, port: u16) -> Result<Self, &'static str> {
        let addr: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .map_err(|_| "Invalid address")?;
        let mut stream = TcpStream::connect(addr).map_err(|_| "Error connecting to host")?;
        let _ = stream.read_to_string(&mut String::new());
        Ok(Client { stream })
    }
}

#[cfg(test)]
mod tests {}
