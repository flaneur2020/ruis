use std::io;
use std::net::{TcpStream};
use std::io::{BufReader};
use std::rc::Rc;

use super::resp::{RespWriter, RespReader};
use super::types::{RespValue, RespError};

struct Connection {
    password: String,
    // w: RespWriter<'a>,
    r: RespReader,
}

impl Connection {
    pub fn new(addr: &str, password: &str) -> io::Result<Connection> {
        let rs = TcpStream::connect(addr)?;
        let mut ws = rs.try_clone()?;
        let reader = RespReader::new(Box::new(BufReader::new(rs)));
        // let writer = RespWriter::new(&mut ws);
        return Ok(Self {
            password: String::from(password),
            // w: writer,
            r: reader,
        })
    }

    pub fn run(&mut self, cmd: &[&[u8]]) -> Result<RespValue, RespError> {
        // self.w.write_bulks(cmd)?;
        self.r.read()
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
