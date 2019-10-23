use std::io;
use std::net::{TcpStream};
use std::rc::Rc;
use std::io::{BufRead, BufReader, Write};

use super::resp::{RespWriter, RespReader};
use super::types::{RespValue, RespError};

struct Connection {
    rs: BufReader<TcpStream>,
    ws: TcpStream,
}

impl Connection {
    pub fn new(addr: &str) -> io::Result<Self> {
        let ws = TcpStream::connect(addr)?;
        let rs = BufReader::new(ws.try_clone()?);
        let conn = Self {
            rs: rs,
            ws: ws,
        };
        return Ok(conn)
    }

    pub fn auth(&mut self, password: &str) -> Result<RespValue, RespError> {
       self.execute(&vec!["auth".as_bytes(), password.as_bytes()])
    } 

    pub fn execute(&mut self, cmd: &[&[u8]]) -> Result<RespValue, RespError> {
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
        let mut conn = Connection::new("localhost:6379").unwrap();
        let r = conn.execute(&vec!["ping".as_bytes()]).unwrap();
        assert_eq!(r, RespValue::Bulk(b"PONG".to_vec()));
    }
}