use std::io;
use std::net::{TcpStream};
use std::rc::Rc;
use std::io::{BufRead, BufReader, Write};

use super::resp::{RespWriter, RespReader};
use super::types::{RespValue, RespError};

struct Connection<W: Write> {
    w: RespWriter<W>,
    r: RespReader,
}

fn connect(addr: &str) -> io::Result<Connection<std::net::TcpStream>> {
    let ws = TcpStream::connect(addr)?;
    let rs = BufReader::new(ws.try_clone()?);
    let r = RespReader::new(Box::new(rs));
    let w = RespWriter::new(ws);
    let conn = Connection {
        r: r,
        w: w,
    };
    return Ok(conn)
}

impl<W: Write> Connection<W> {

    pub fn auth(&mut self, password: &str) -> Result<RespValue, RespError> {
       self.execute(&vec!["auth".as_bytes(), password.as_bytes()])
    }

    pub fn execute(&mut self, cmd: &[&[u8]]) -> Result<RespValue, RespError> {
        self.w.write_bulks(cmd)?;
        self.r.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let mut conn = connect("localhost:6379").unwrap();
        let r = conn.execute(&vec!["ping".as_bytes()]).unwrap();
        assert_eq!(r, RespValue::Bulk(b"PONG".to_vec()));
    }
}
