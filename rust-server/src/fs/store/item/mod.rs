mod output;
pub use output::Output;
pub struct RamFd {
    pub pos: u64,
}

pub struct Input {
    reader: Box<dyn std::io::Read + Send>,
    remaining: u64,
}
impl Input {
    pub fn new(reader: Box<dyn std::io::Read + Send>, limit: u64) -> Self {
        Self {
            reader,
            remaining: limit,
        }
    }
    pub fn read(&mut self, dst: &mut [u8]) -> Result<usize, &'static str> {
        if dst.is_empty() {
            return Ok(0);
        }
        let max = dst.len().min(self.remaining.saturating_add(1) as usize);
        let n = self.reader.read(&mut dst[..max]).map_err(|e| {
            if e.kind() == std::io::ErrorKind::InvalidData {
                "limit"
            } else {
                "io"
            }
        })?;
        if n as u64 > self.remaining {
            return Err("limit");
        }
        self.remaining -= n as u64;
        Ok(n)
    }
}
pub enum Item {
    Input(Input),
    Output(Output),
    File(crate::fs::disk::Entry),
    Dir(crate::fs::dir::Entry),
    Ram(RamFd),
}
