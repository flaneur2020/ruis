use std::io;
use std::str::FromStr;
use std::io::{BufReader, BufRead};

// https://redis.io/topics/protocol

struct RespReader<R> {
    reader: BufReader<R>
}

enum RespValue {
    String(String),
    Int(i64),
    Error(String)
}

enum RespError {
    IO(io::Error),
    ParseFailed(String),
    Unexpected(&'static str),
    Unknown
}

impl From<io::Error> for RespError {
    fn from(err: io::Error) -> Self {
        RespError::IO(err)
    }
}

impl<R: BufRead> RespReader<R> {
    fn new(r: R) -> Self {
        let reader = BufReader::new(r);

        Self {
            reader: reader,
        }
    }

    fn read(&mut self) -> Result<RespValue, RespError> {
        let mut line = vec![];
        self.reader.read_until('\n' as u8, &mut line)?;
        if line.len() == 0 {
            return Err(RespError::ParseFailed(format!("line empty")));
        }

        match line[0] as char {
            ':' => {
                let n = self.read_int(&line[1..])?;
                return Ok(RespValue::Int(n));
            },
            '+' => {
                let s = self.read_string(&line[1..])?;
                return Ok(RespValue::String(s));
            }
            '-' => {
                let s = self.read_string(&line[1..])?;
                return Ok(RespValue::Error(s));
            }
            ch @ _ => {
                Err(RespError::ParseFailed(format!("unexpected token: {}", ch)))
            }
        }
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