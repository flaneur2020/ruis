use std::io;
use std::net::{TcpStream};
use std::rc::Rc;
use std::io::{BufRead, BufReader, Write};

use super::resp::{RespWriter, RespReader};
use super::types::{RespValue, RespError};

struct Connection {
    password: String,
    rs: BufReader<TcpStream>,
    ws: TcpStream,
}

impl Connection {
    pub fn new(addr: &str, password: &str) -> io::Result<Self> {
        let rs = TcpStream::connect(addr)?;
        let ws = rs.try_clone().unwrap();
        return Ok(Self {
            password: String::from(password),
            rs: BufReader::new(rs),
            ws: ws,
        })
    }

    pub fn run(&mut self, cmd: &[&[u8]]) -> Result<RespValue, RespError> {
        let mut r = RespReader::new(&mut self.rs);
        let mut w = RespWriter::new(&mut self.ws);
        w.write_bulks(cmd)?;
        r.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let mut conn = Connection::new("localhost:6379", "").unwrap();
        let r = conn.run(&vec!["info".as_bytes()]).unwrap();
        assert_eq!(r, RespValue::Bulk(b"OK".to_vec()));
    }
}
