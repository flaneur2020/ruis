use std::io;
use std::str::FromStr;
use std::io::{BufReader, BufRead, Read};

// https://redis.io/topics/protocol

struct RespReader<R> {
    reader: BufReader<R>
}

#[derive(Eq,PartialEq,Debug)]
enum RespValue {
    Int(i64),
    NullBulkString,
    String(Vec<u8>),
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
        let line = self.read_line()?;
        match line[0] as char {
            ':' => {
                let n = self.parse_int(&line[1..])?;
                return Ok(RespValue::Int(n));
            },
            '+' => {
                return Ok(RespValue::String(line[1..].to_vec()));
            }
            '-' => {
                let s = std::str::from_utf8(&line[1..]).or_else(|e|
                    Err(RespError::ParseFailed(format!("bad utf8: {}", e)))
                )?;
                return Ok(RespValue::Error(String::from(s)));
            }
            '$' => {
                let l = self.parse_int(&line[1..])?;
                if l == -1 {
                    return Ok(RespValue::NullBulkString);
                } else if l < 0 {
                    return Err(RespError::ParseFailed(format!("malformed bulk string")))
                }
                let s = self.read_bulk_string(l as usize)?;
                return Ok(RespValue::String(s))
            }
            ch @ _ => {
                Err(RespError::ParseFailed(format!("unexpected token: {}", ch)))
            }
        }
    }

    fn read_line(&mut self) -> Result<Vec<u8>, RespError> {
        let mut line: Vec<u8> = vec![];

        self.reader.read_until('\n' as u8, &mut line).or_else(|e|
            Err(RespError::ParseFailed(format!("io err: {}", e)))
        )?;

        if !line.ends_with(&['\r' as u8, '\n' as u8]) {
            return Err(RespError::ParseFailed(format!("line not ends with CRLF")));
        }

        line.pop();
        line.pop();
        Ok(line)
    }

    fn read_bulk_string(&mut self, l: usize) -> Result<Vec<u8>, RespError> {
        let mut buf = vec![0u8; l];
        self.reader.read_exact(&mut buf).or_else(|e|
            Err(RespError::ParseFailed(format!("io err: {}", e)))
        )?;
        self.read_line();
        return Ok(buf);
    }

    fn parse_int(&mut self, buf: &[u8]) -> Result<i64, RespError> {
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
        assert_eq!(r.unwrap(), RespValue::String(b"OK".to_vec()));

        let br = io::Cursor::new(b"-ERR Bad Request\r\n");
        let r = RespReader::new(br).read();
        assert_eq!(r.unwrap(), RespValue::Error(format!("ERR Bad Request")));

        let br = io::Cursor::new(b"blah\r\n");
        let r = RespReader::new(br).read();
        assert_eq!(r.unwrap_err(), RespError::ParseFailed(format!("unexpected token: b")));
    }
}