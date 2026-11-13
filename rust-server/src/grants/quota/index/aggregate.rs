#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Usage {
    pub bytes: u64,
    pub objects: u64,
}
impl Usage {
    pub fn object(size: u64) -> Self {
        Self {
            bytes: size + 512,
            objects: 1,
        }
    }
    pub fn add(&mut self, other: Self) {
        self.bytes += other.bytes;
        self.objects += other.objects;
    }
    pub fn subtract(&mut self, other: Self) {
        self.bytes -= other.bytes;
        self.objects -= other.objects;
    }
    pub fn update(&mut self, old: Option<u64>, new: Option<u64>) {
        if let Some(size) = old {
            self.subtract(Self::object(size));
        }
        if let Some(size) = new {
            self.add(Self::object(size));
        }
    }
}
