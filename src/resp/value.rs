use std::fmt;

#[derive(Eq,PartialEq)]
pub enum RespValue {
    Int(i64),
    NullString,
    NullArray,
    String(Vec<u8>),
    Array(Vec<RespValue>),
    Error(Vec<u8>),
}

impl fmt::Debug for RespValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RespValue::NullString => write!(f, "NullString"),
            RespValue::NullArray => write!(f, "NullArray"),
            RespValue::Int(n) => write!(f, "Int({})", n),
            RespValue::String(bs) => write!(f, "String('{}')", String::from_utf8_lossy(bs)),
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
