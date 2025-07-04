use std::{
    io::{BufRead, BufReader, BufWriter, Read, Write},
    net::{SocketAddr, TcpStream},
};

use infusedb::DataType;

pub struct Client {
    stream: TcpStream,
    writer: BufWriter<TcpStream>,
    reader: BufReader<TcpStream>,
}

impl Client {
    pub fn new(host: &str, port: u16) -> Result<Self, &'static str> {
        let addr: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .map_err(|_| "Invalid address")?;
        let mut stream = TcpStream::connect(addr).map_err(|_| "Error connecting to host")?;
        let _ = stream.read_to_string(&mut String::new());
        let writer = BufWriter::new(stream.try_clone().map_err(|_| "Error creating writer")?);
        let reader = BufReader::new(stream.try_clone().map_err(|_| "Error creating reader")?);
        Ok(Client {
            stream,
            writer,
            reader,
        })
    }

    fn __call__(&mut self, command: &str) -> Result<DataType, &'static str> {
        let command_formated = format!("{command}\n");
        self.writer
            .write_all(&command_formated.as_bytes())
            .map_err(|_| "Error writing to socket")?;
        let mut response = String::new();
        self.reader
            .read_line(&mut response)
            .map_err(|_| "Error reading socket")?;
        if response.len() == 0 {
            return Err("No data readed");
        }
        let t = DataType::infer_type(&response);
        let dt = DataType::load(t, response).ok_or("Error parsing response")?;
        Ok(dt)
    }

    pub fn get(&mut self, key: &str) -> Result<DataType, &'static str> {
        self.__call__(&format!("get {key}"))
    }

    pub fn set(&mut self, key: &str, value: DataType) -> Result<(), &'static str> {
        let value = value.to_string();
        let r = self.__call__(&format!("set {key} {value}"))?;
        if r.to_boolean() { Ok(()) } else { Err("") }
    }

    pub fn list(&mut self) -> Result<DataType, &'static str> {
        self.__call__(&format!("list"))
    }

    pub fn close(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}

#[cfg(test)]
mod tests {
    use crate::Client;

    #[test]
    fn test_basic() {
        let client = Client::new("0.0.0.0", 1234);
        assert!(client.is_ok());
        let mut client = client.unwrap();
        client.close();
    }

    #[test]
    fn test_get() {
        let client = Client::new("0.0.0.0", 1234);
        assert!(client.is_ok());
        let mut client = client.unwrap();
        let name = client.get("name");
        assert!(name.is_ok());
        let name = name.unwrap();
        let name = name.to_text();
        assert_eq!(name, "Alberto Ruiz");
        client.close();
    }
}
