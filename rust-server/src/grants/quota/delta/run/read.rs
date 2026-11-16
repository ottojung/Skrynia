use super::Value;
use super::{
    Run,
    codec::{Row, WIDTH},
};
use std::{
    fs::File,
    io::{self, BufReader, Read},
};
pub struct Reader {
    file: BufReader<File>,
    remaining: u64,
    previous: Value,
}
impl Reader {
    pub fn new(run: &Run) -> io::Result<Self> {
        Ok(Self {
            file: BufReader::new(File::open(&run.path)?),
            remaining: run.len,
            previous: Value::default(),
        })
    }
    pub fn next(&mut self) -> io::Result<Option<(String, Value)>> {
        if self.remaining == 0 {
            return Ok(None);
        }
        let mut bytes = [0u8; WIDTH];
        self.file.read_exact(&mut bytes)?;
        let row = Row::decode(&bytes)?;
        let delta = row.total.minus(self.previous)?;
        self.previous = row.total;
        self.remaining -= 1;
        Ok(Some((row.key, delta)))
    }
}
