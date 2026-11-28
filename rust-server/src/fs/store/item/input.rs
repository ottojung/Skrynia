use std::io::Read;

mod bulk;
mod error;
#[cfg(test)]
mod tests;
mod transfer;

pub struct Input {
    reader: Box<dyn Read + Send>,
    remaining: u64,
    pos: u64,
    scratch: Vec<u8>,
}
impl Input {
    #[rustfmt::skip]
    pub fn new(reader: Box<dyn Read + Send>, limit: u64) -> Self {
        Self { reader, remaining: limit, pos: 0, scratch: Vec::new() }
    }
    fn read_local(&mut self, dst: &mut [u8]) -> Result<usize, &'static str> {
        transfer::read(&mut *self.reader, &mut self.remaining, &mut self.pos, dst)
    }
    pub fn read1(&mut self) -> Result<Option<u8>, &'static str> {
        let mut b = [0u8; 1];
        let n = self.read_local(&mut b)?;
        Ok(if n == 0 { None } else { Some(b[0]) })
    }
    pub fn skip(&mut self, mut n: u64) -> Result<u64, &'static str> {
        let mut buf = [0u8; 256];
        while n > 0 {
            let take = n.min(buf.len() as u64) as usize;
            let got = self.read_local(&mut buf[..take])?;
            n -= got as u64;
            if got == 0 {
                break;
            }
        }
        Ok(self.pos)
    }
}
