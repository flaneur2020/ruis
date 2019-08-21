use std::io;
use std::str::FromStr;
use std::io::{BufReader, BufRead, Read};
use std::io::{Write, BufWriter};
use std::error::Error;

use super::types::{RespValue, RespError};

// https://redis.io/topics/protocol

pub struct RespReader {
    reader: Box<BufRead>
}

struct RespWriter<W: Write> {
    writer: BufWriter<W>,
}

impl RespReader {
    pub fn new(r: Box<BufRead>) -> Self {
        Self {
            reader: r,
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
                    return Err(RespError::ParseFailed(format!("malformed length")))
                }
                let s = self.read_bulk_string(n as usize)?;
                return Ok(RespValue::Bulk(s))
            }
            '*' => {
                let n = self.parse_int(&line[1..])?;
                if n == -1 {
                    return Ok(RespValue::NilArray);
                } else if n < 0 {
                    return Err(RespError::ParseFailed(format!("malformed length")))
                }
                let arr = self.read_array(n as usize)?;
                return Ok(RespValue::Array(arr));
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

        let line = self.read_line()?;
        if line.len() != 0 {
            return Err(RespError::ParseFailed(format!("bad bulk string format")))
        }
        return Ok(buf);
    }

    fn read_array(&mut self, n: usize) -> Result<Vec<RespValue>, RespError> {
        let mut arr: Vec<RespValue> = vec![];
        for _ in 0..n {
            let val = self.read()?;
            arr.push(val)
        }
        return Ok(arr);
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

impl<W: Write> RespWriter<W> {
    pub fn new(w: W) -> Self {
        let mut writer = BufWriter::new(w);
        Self {
            writer: writer,
        }
    }

    pub fn write_int(&mut self, n: i64) -> Result<(), RespError> {
        self.writer.write_fmt(format_args!(":{}\r\n", n)).or_else(|e|
            Err(RespError::IoError(e))
        )
    }

    pub fn write_bulk(&mut self, b: Vec<u8>) -> Result<(), RespError> {
        Ok(())
    }

    pub fn write_array(&mut self, arr: Vec<RespValue>) -> Result<(), RespError> {
        Ok(())
    }

    pub fn write_error(&mut self, s: String) -> Result<(), RespError> {
        Ok(())
    }

    pub fn write_status(&mut self, s: String) -> Result<(), RespError> {
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), RespError> {
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let br = Box::new(io::Cursor::new(b"+OK\r\n"));
        let r = RespReader::new(br).read();
        assert_eq!(r.unwrap(), RespValue::Bulk(b"OK".to_vec()));

        let br = Box::new(io::Cursor::new(b"-ERR Bad Request\r\n"));
        let r = RespReader::new(br).read();
        assert_eq!(r.unwrap(), RespValue::Error(b"ERR Bad Request".to_vec()));

        let br = Box::new(io::Cursor::new(b"blah\r\n"));
        let r = RespReader::new(br).read();
        assert_eq!(format!("{}", r.unwrap_err()), format!("parse failed: unexpected token: b"));

        let br = Box::new(io::Cursor::new(b"*3\r\n$3\r\nfoo\r\n$-1\r\n$3\r\nbar\r\n"));
        let r = RespReader::new(br).read();
        let v = vec![
            RespValue::Bulk(b"foo".to_vec()),
            RespValue::NilBulk,
            RespValue::Bulk(b"bar".to_vec()),
        ];
        assert_eq!(r.unwrap(), RespValue::Array(v));

        let br = Box::new(io::Cursor::new(b"*5\r\n:1\r\n:2\r\n:3\r\n:4\r\n$6\r\nfoobar\r\n"));
        let r = RespReader::new(br).read();
        let v = vec![
            RespValue::Int(1),
            RespValue::Int(2),
            RespValue::Int(3),
            RespValue::Int(4),
            RespValue::Bulk(b"foobar".to_vec()),
        ];
        assert_eq!(r.unwrap(), RespValue::Array(v));

        let br = Box::new(io::Cursor::new(b"*-1\r\n"));
        let r = RespReader::new(br).read();
        assert_eq!(r.unwrap(), RespValue::NilArray);

        let br = Box::new(io::Cursor::new(b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n"));
        let r = RespReader::new(br).read();
        let v = vec![
            RespValue::Bulk(b"foo".to_vec()),
            RespValue::Bulk(b"bar".to_vec()),
        ];
        assert_eq!(r.unwrap(), RespValue::Array(v));
    }

    #[test]
    fn test_read_array_of_array() {
        let br = Box::new(io::Cursor::new(b"*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Foo\r\n-Bar\r\n".to_vec()));
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
