pub struct Output {
    writer: Box<dyn std::io::Write + Send>,
    pos: u64,
}
impl Output {
    pub fn new(writer: Box<dyn std::io::Write + Send>) -> Self {
        Self { writer, pos: 0 }
    }
    pub fn write(&mut self, src: &[u8]) -> Result<usize, &'static str> {
        if src.is_empty() {
            return Ok(0);
        }
        let n = self.writer.write(src).map_err(|_| "io")?;
        if n > src.len() {
            return Err("io");
        }
        self.pos = self.pos.checked_add(n as u64).ok_or("overflow")?;
        Ok(n)
    }
    pub fn write1(&mut self, byte: u8) -> Result<(), &'static str> {
        self.writer.write_all(&[byte]).map_err(|_| "io")?;
        self.pos = self.pos.checked_add(1).ok_or("overflow")?;
        Ok(())
    }
    pub fn position(&self) -> u64 {
        self.pos
    }
}

#[cfg(test)]
mod tests;
