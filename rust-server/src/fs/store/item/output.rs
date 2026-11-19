pub struct Output {
    writer: Box<dyn std::io::Write + Send>,
}
impl Output {
    pub fn new(writer: Box<dyn std::io::Write + Send>) -> Self {
        Self { writer }
    }
    pub fn write(&mut self, src: &[u8]) -> Result<usize, &'static str> {
        self.writer.write(src).map_err(|_| "io")
    }
}
