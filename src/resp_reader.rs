use std::io;
use std::str::FromStr;
use std::io::{BufReader, BufRead};

// https://redis.io/topics/protocol

struct RespReader<R> {
    reader: BufReader<R>
}

#[derive(Eq,PartialEq,Debug)]
enum RespValue {
    String(String),
    Int(i64),
    Error(String)
}

#[derive(PartialEq,Debug)]
enum RespError {
    ParseFailed(String),
    Unexpected(String),
    Unknown
}

impl<R: BufRead> RespReader<R> {
    pub fn new(r: R) -> Self {
        let reader = BufReader::new(r);

        Self {
            reader: reader,
        }
    }

    pub fn read(&mut self) -> Result<RespValue, RespError> {
        let mut line = vec![];
        let n = self.read_line(&mut line)?;

        match line[0] as char {
            ':' => {
                let n = self.read_int(&line[1..n])?;
                return Ok(RespValue::Int(n));
            },
            '+' => {
                let s = self.read_string(&line[1..n])?;
                return Ok(RespValue::String(s));
            }
            '-' => {
                let s = self.read_string(&line[1..n])?;
                return Ok(RespValue::Error(s));
            }
            ch @ _ => {
                Err(RespError::ParseFailed(format!("unexpected token: {}", ch)))
            }
        }
    }

    fn read_line(&mut self, line: &mut Vec<u8>) -> Result<usize, RespError> {
        self.reader.read_until('\n' as u8, line).or_else(|e|
            Err(RespError::ParseFailed(format!("io err: {}", e)))
        )?;

        if !line.ends_with(&['\r' as u8, '\n' as u8]) {
            return Err(RespError::ParseFailed(format!("line not ends with CRLF")));
        }

        Ok(line.len()-2)
    }

    fn read_string(&mut self, buf: &[u8]) -> Result<String, RespError> {
        let s = std::str::from_utf8(buf).or(
            Err(RespError::ParseFailed(format!("bad utf8")))
        )?;
        return Ok(String::from(s));
    }

    fn read_bulk_string(&mut self, buf: &[u8]) -> Result<String, RespError> {
        return Ok(String::from("miao"))
    }

    fn read_int(&mut self, buf: &[u8]) -> Result<i64, RespError> {
        if buf.len() == 0 {
            return Err(RespError::ParseFailed(format!("malformed integer")));
        }

        let s = std::str::from_utf8(buf).or(
            Err(RespError::ParseFailed(format!("bad utf8")))
        )?;
        let n = i64::from_str(s).or(
            Err(RespError::ParseFailed(format!("parse int failed")))
        )?;
        return Ok(n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let br = io::Cursor::new(b"+OK\r\n");
        let r = RespReader::new(br).read();
        assert_eq!(r.unwrap(), RespValue::String(format!("OK")));

        let br = io::Cursor::new(b"-ERR Bad Request\r\n");
        let r = RespReader::new(br).read();
        assert_eq!(r.unwrap(), RespValue::Error(format!("ERR Bad Request")));

        let br = io::Cursor::new(b"blah\r\n");
        let r = RespReader::new(br).read();
        assert_eq!(r.unwrap_err(), RespError::ParseFailed(format!("unexpected token: b")));
    }
}