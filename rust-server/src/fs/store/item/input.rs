use std::io::Read;

mod error;

pub struct Input {
    reader: Box<dyn Read + Send>,
    remaining: u64,
    pos: u64,
}
impl Input {
    #[rustfmt::skip]
    pub fn new(reader: Box<dyn Read + Send>, limit: u64) -> Self {
        Self { reader, remaining: limit, pos: 0 }
    }
    pub fn read(&mut self, dst: &mut [u8]) -> Result<usize, &'static str> {
        if dst.is_empty() {
            return Ok(0);
        }
        let max = self.remaining.min(dst.len() as u64) as usize;
        if max == 0 {
            // Probe only after all accepted bytes have been reported. Never
            // put the rejected overflow byte in the caller's destination.
            return match self.reader.read(&mut [0u8; 1]) {
                Ok(0) => Ok(0),
                Ok(_) => Err("limit"),
                Err(e) => Err(error::read_error(e)),
            };
        }
        let n = self
            .reader
            .read(&mut dst[..max])
            .map_err(error::read_error)?;
        self.remaining -= n as u64;
        self.pos += n as u64;
        Ok(n)
    }
    pub fn read1(&mut self) -> Result<Option<u8>, &'static str> {
        let mut b = [0u8; 1];
        let n = self.read(&mut b)?;
        Ok(if n == 0 { None } else { Some(b[0]) })
    }
    pub fn skip(&mut self, mut n: u64) -> Result<u64, &'static str> {
        let mut buf = [0u8; 256];
        while n > 0 {
            let take = n.min(buf.len() as u64) as usize;
            let got = self.read(&mut buf[..take])?;
            n -= got as u64;
            if got == 0 {
                break;
            }
        }
        Ok(self.pos)
    }
}
