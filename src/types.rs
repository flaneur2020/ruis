use std;

#[derive(Eq,PartialEq)]
pub enum RespValue {
    Int(i64),
    NilBulk,
    NilArray,
    Bulk(Vec<u8>),
    Array(Vec<RespValue>),
    Error(Vec<u8>),
}

impl std::fmt::Debug for RespValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RespValue::NilBulk => write!(f, "NilBulk"),
            RespValue::NilArray => write!(f, "NilArray"),
            RespValue::Int(n) => write!(f, "Int({})", n),
            RespValue::Bulk(bs) => write!(f, "Bulk('{}')", String::from_utf8_lossy(bs)),
            RespValue::Error(bs) => write!(f, "Error('{}')", String::from_utf8_lossy(bs)),
            RespValue::Array(arr) => {
                write!(f, "Array([")?;
                for i in 0..arr.len() {
                    arr[i].fmt(f)?;
                    if i != arr.len()-1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "])")
            }
        }
    }
}

#[derive(Debug,PartialEq)]
pub enum RespError {
    IoError(std::io::ErrorKind),
    ParseFailed(String),
    Unexpected(String),
    Unknown
}

impl std::fmt::Display for RespError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RespError::IoError(ref err) => write!(f, "io err: {}", err),
            RespError::ParseFailed(ref s) => write!(f, "parse failed: {}", s),
            RespError::Unexpected(ref s) => write!(f, "unexpected: {}", s),
            RespError::Unknown => write!(f, "unknown error"),
        }
    }
}

impl std::error::Error for RespError {
    fn description(&self) -> &str {
        "resp error"
    }
}
