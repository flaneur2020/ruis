use std::io;
use std::str::FromStr;
use std::io::{BufReader, BufRead, Read};

use super::super::types::{RespValue};

// https://redis.io/topics/protocol

pub struct RespReader<R> {
    reader: BufReader<R>
}

#[derive(PartialEq,Debug)]
pub enum RespReadError {
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

    pub fn read(&mut self) -> Result<RespValue, RespReadError> {
        let line = self.read_line()?;
        match line[0] as char {
            ':' => {
                let n = self.parse_int(&line[1..])?;
                return Ok(RespValue::Int(n));
            },
            '+' => {
                return Ok(RespValue::Bulk(line[1..].to_vec()));
            }
            '-' => {
                return Ok(RespValue::Error(line[1..].to_vec()));
            }
            '$' => {
                let n = self.parse_int(&line[1..])?;
                if n == -1 {
                    return Ok(RespValue::NilBulk);
                } else if n < 0 {
                    return Err(RespReadError::ParseFailed(format!("malformed length")))
                }
                let s = self.read_bulk_string(n as usize)?;
                return Ok(RespValue::Bulk(s))
            }
            '*' => {
                let n = self.parse_int(&line[1..])?;
                if n == -1 {
                    return Ok(RespValue::NilArray);
                } else if n < 0 {
                    return Err(RespReadError::ParseFailed(format!("malformed length")))
                }
                let arr = self.read_array(n as usize)?;
                return Ok(RespValue::Array(arr));
            }
            ch @ _ => {
                Err(RespReadError::ParseFailed(format!("unexpected token: {}", ch)))
            }
        }
    }

    fn read_line(&mut self) -> Result<Vec<u8>, RespReadError> {
        let mut line: Vec<u8> = vec![];

        self.reader.read_until('\n' as u8, &mut line).or_else(|e|
            Err(RespReadError::ParseFailed(format!("io err: {}", e)))
        )?;

        if !line.ends_with(&['\r' as u8, '\n' as u8]) {
            return Err(RespReadError::ParseFailed(format!("line not ends with CRLF")));
        }

        line.pop();
        line.pop();
        Ok(line)
    }

    fn read_bulk_string(&mut self, l: usize) -> Result<Vec<u8>, RespReadError> {
        let mut buf = vec![0u8; l];
        self.reader.read_exact(&mut buf).or_else(|e|
            Err(RespReadError::ParseFailed(format!("io err: {}", e)))
        )?;

        let line = self.read_line()?;
        if line.len() != 0 {
            return Err(RespReadError::ParseFailed(format!("bad bulk string format")))
        }
        return Ok(buf);
    }

    fn read_array(&mut self, n: usize) -> Result<Vec<RespValue>, RespReadError> {
        let mut arr: Vec<RespValue> = vec![];
        for _ in 0..n {
            let val = self.read()?;
            arr.push(val)
        }
        return Ok(arr);
    }

    fn parse_int(&mut self, buf: &[u8]) -> Result<i64, RespReadError> {
        if buf.len() == 0 {
            return Err(RespReadError::ParseFailed(format!("malformed integer")));
        }

        let s = std::str::from_utf8(buf).or(
            Err(RespReadError::ParseFailed(format!("bad utf8")))
        )?;
        let n = i64::from_str(s).or(
            Err(RespReadError::ParseFailed(format!("parse int failed")))
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
        assert_eq!(r.unwrap(), RespValue::Bulk(b"OK".to_vec()));

        let br = io::Cursor::new(b"-ERR Bad Request\r\n");
        let r = RespReader::new(br).read();
        assert_eq!(r.unwrap(), RespValue::Error(b"ERR Bad Request".to_vec()));

        let br = io::Cursor::new(b"blah\r\n");
        let r = RespReader::new(br).read();
        assert_eq!(r.unwrap_err(), RespReadError::ParseFailed(format!("unexpected token: b")));

        let br = io::Cursor::new(b"*3\r\n$3\r\nfoo\r\n$-1\r\n$3\r\nbar\r\n");
        let r = RespReader::new(br).read();
        let v = vec![
            RespValue::Bulk(b"foo".to_vec()),
            RespValue::NilBulk,
            RespValue::Bulk(b"bar".to_vec()),
        ];
        assert_eq!(r.unwrap(), RespValue::Array(v));

        let br = io::Cursor::new(b"*5\r\n:1\r\n:2\r\n:3\r\n:4\r\n$6\r\nfoobar\r\n");
        let r = RespReader::new(br).read();
        let v = vec![
            RespValue::Int(1),
            RespValue::Int(2),
            RespValue::Int(3),
            RespValue::Int(4),
            RespValue::Bulk(b"foobar".to_vec()),
        ];
        assert_eq!(r.unwrap(), RespValue::Array(v));

        let br = io::Cursor::new(b"*-1\r\n");
        let r = RespReader::new(br).read();
        assert_eq!(r.unwrap(), RespValue::NilArray);

        let br = io::Cursor::new(b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
        let r = RespReader::new(br).read();
        let v = vec![
            RespValue::Bulk(b"foo".to_vec()),
            RespValue::Bulk(b"bar".to_vec()),
        ];
        assert_eq!(r.unwrap(), RespValue::Array(v));
    }

    #[test]
    fn test_read_array_of_array() {
        let br = io::Cursor::new(b"*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Foo\r\n-Bar\r\n".to_vec());
        let r = RespReader::new(br).read();
        let arr = RespValue::Array(vec![
            RespValue::Array(vec![
                RespValue::Int(1),
                RespValue::Int(2),
                RespValue::Int(3),
            ]),
            RespValue::Array(vec![
                RespValue::Bulk(b"Foo".to_vec()),
                RespValue::Error(b"Bar".to_vec()),
            ])
        ]);
        assert_eq!(r.unwrap(), arr);
    }
}