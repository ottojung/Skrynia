#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fd(pub u64);

impl Fd {
    pub fn new(idx: u32, tag: u32) -> Self {
        Self(((tag as u64) << 32) | idx as u64)
    }
    pub fn idx(self) -> usize {
        self.0 as u32 as usize
    }
    pub fn tag(self) -> u32 {
        (self.0 >> 32) as u32
    }
}
