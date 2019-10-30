use std::io;
use std::net::{TcpStream};
use std::rc::Rc;
use std::io::{BufRead, BufReader, Write};

use super::resp::{RespWriter, RespReader};
use super::types::{RespValue, RespError};

pub struct GenericConnection<W: Write, R: BufRead> {
    w: RespWriter<W>,
    r: RespReader<R>,
}

impl<W: Write, R: BufRead> GenericConnection<W, R> {
    pub fn new(r: RespReader<R>, w: RespWriter<W>) -> Self {
        Self {
            w: w,
            r: r
        }
    }

    pub fn auth(&mut self, password: &str) -> Result<RespValue, RespError> {
       self.execute(&vec!["auth".as_bytes(), password.as_bytes()])
    }

    pub fn execute(&mut self, cmd: &[&[u8]]) -> Result<RespValue, RespError> {
        self.w.write_bulks(cmd)?;
        self.r.read()
    }
}

pub type TcpConnection = GenericConnection<std::net::TcpStream, BufReader<std::net::TcpStream>>;

impl TcpConnection {
    pub fn connect(addr: &str, password_opt: Option<&str>) -> io::Result<TcpConnection> {
        let ws = TcpStream::connect(addr)?;
        let rs = BufReader::new(ws.try_clone()?);
        let r = RespReader::new(rs);
        let w = RespWriter::new(ws);
        let mut conn = GenericConnection::new(r, w);

        if let Some(password) = password_opt {
            conn.auth(password).or_else(|e|
                Err(io::Error::new(io::ErrorKind::PermissionDenied, format!("failed on auth: {}", e)))
            )?;
        }
        return Ok(conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let mut conn = TcpConnection::connect("localhost:6379", None).unwrap();
        let r = conn.execute(&vec!["ping".as_bytes()]).unwrap();
        assert_eq!(r, RespValue::Bulk(b"PONG".to_vec()));
    }
}
