use std::io::{Write, BufWriter};

struct RespWriter<W> {
    writer: BufWriter<W>,
}

enum RespWriteError {
    IoError(String)
}

impl RespWriteError {
}

impl<W: Write> RespWriter<W> {
    fn new(w: W) -> Self {
        let mut writer = BufWriter::new(w);
        Self {
            writer: writer,
        }
    }

    fn write_int(&mut self, n: i64) -> Result<(), RespWriteError> {
        self.writer.write_fmt(":{}\r\n", n).or(|e|
            Err(RespWriteError::IoError(format!("{}", e)))
        )
    }
}