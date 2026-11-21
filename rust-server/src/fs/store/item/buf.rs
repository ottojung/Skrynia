pub struct BufferedOutput {
    data: Vec<u8>,
    cap: usize,
}
impl BufferedOutput {
    pub fn new(cap: usize) -> Self {
        Self {
            data: Vec::new(),
            cap,
        }
    }
    pub fn write(&mut self, src: &[u8]) -> Result<usize, &'static str> {
        if self.data.len().saturating_add(src.len()) > self.cap {
            return Err("limit");
        }
        self.data.extend_from_slice(src);
        Ok(src.len())
    }
    pub fn take(self) -> Vec<u8> {
        self.data
    }
}
