#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fd(pub i32);

impl Fd {
    pub fn new(idx: u32, tag: u32) -> Self {
        Self((((tag & 0xffff) << 16) | (idx & 0xffff)) as i32)
    }
    pub fn idx(self) -> usize {
        (self.0 as u32 & 0xffff) as usize
    }
    pub fn tag(self) -> u32 {
        ((self.0 as u32) >> 16) & 0xffff
    }
}
